//! Combined Gateway socket: Config PING/PONG + Session client auth.

use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

use crate::memory::{ClientSocketConn, SharedGatewayMemory};
use crate::sts::StsClient;

#[derive(Debug, Deserialize)]
struct AuthLine {
    access_token: String,
}

/// Bind `addr`: answer length-prefixed PING with PONG; accept JSON auth lines for Client Web.
pub fn spawn_gateway_socket(addr: SocketAddr, sts: StsClient, memory: SharedGatewayMemory) {
    tokio::spawn(async move {
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                tracing::info!(%addr, "gateway socket (PING/PONG + client auth) ready");
                loop {
                    match listener.accept().await {
                        Ok((stream, peer)) => {
                            let sts = sts.clone();
                            let memory = memory.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_conn(stream, sts, memory).await {
                                    tracing::debug!(%peer, "gateway socket closed: {e}");
                                }
                            });
                        }
                        Err(e) => tracing::warn!("gateway socket accept: {e}"),
                    }
                }
            }
            Err(e) => tracing::error!(%addr, "gateway socket bind failed: {e}"),
        }
    });
}

async fn handle_conn(
    stream: TcpStream,
    sts: StsClient,
    memory: SharedGatewayMemory,
) -> Result<(), String> {
    stream.set_nodelay(true).ok();
    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];
    let mut stream = stream;

    // Peek first chunk to decide protocol.
    let n = stream.read(&mut tmp).await.map_err(|e| e.to_string())?;
    if n == 0 {
        return Ok(());
    }
    buf.extend_from_slice(&tmp[..n]);

    // Length-prefixed frames (Config heartbeat): first 4 bytes are BE length.
    if looks_like_length_frame(&buf) {
        return handle_heartbeat_frames(stream, buf).await;
    }

    // Client auth: newline-delimited JSON `{"access_token":"..."}`.
    handle_client_auth(stream, buf, sts, memory).await
}

fn looks_like_length_frame(buf: &[u8]) -> bool {
    if buf.len() < 4 {
        return false;
    }
    // JSON auth starts with `{` or whitespace; AUTH text starts with letter.
    let first = buf[0];
    if first == b'{' || first == b'A' || first == b'a' || first.is_ascii_whitespace() {
        return false;
    }
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    // Reasonable frame size for PING/PONG
    len > 0 && len <= 64 && buf.len() >= 4
}

async fn handle_heartbeat_frames(mut stream: TcpStream, mut buf: Vec<u8>) -> Result<(), String> {
    let mut tmp = [0u8; 1024];
    loop {
        loop {
            match funnyx_socket::decode_frame(&buf).map_err(|e| e.to_string())? {
                None => break,
                Some((body, consumed)) => {
                    buf.drain(..consumed);
                    if let Some(pong) = funnyx_heartbeat::pong_frame_if_ping(&body) {
                        stream
                            .write_all(&pong)
                            .await
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        let n = stream.read(&mut tmp).await.map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
    }
    Ok(())
}

async fn handle_client_auth(
    stream: TcpStream,
    initial: Vec<u8>,
    sts: StsClient,
    memory: SharedGatewayMemory,
) -> Result<(), String> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Rebuild a line from initial bytes + rest.
    let mut line_buf = String::new();
    let initial_str = String::from_utf8_lossy(&initial);
    line_buf.push_str(&initial_str);
    if !line_buf.contains('\n') {
        let mut extra = String::new();
        reader
            .read_line(&mut extra)
            .await
            .map_err(|e| e.to_string())?;
        line_buf.push_str(&extra);
    }

    let line = line_buf.lines().next().unwrap_or("").trim();
    let token = parse_auth_token(line)?;
    let v = sts
        .validate(&token)
        .await
        .map_err(|(_, e)| e.error)?;
    if !v.valid {
        let msg = format!(
            "{{\"ok\":false,\"error\":\"{}\"}}\n",
            v.reason.unwrap_or_else(|| "invalid session".into()).replace('"', "'")
        );
        writer
            .write_all(msg.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    let end_user_id = v.end_user_id.unwrap_or(0);
    let connection_id = Uuid::new_v4().to_string();
    let connected_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    {
        let mut map = memory.write().await;
        map.client_sockets.insert(
            connection_id.clone(),
            ClientSocketConn {
                connection_id: connection_id.clone(),
                end_user_id,
                connected_at_ms,
            },
        );
    }

    let ok = format!(
        "{{\"ok\":true,\"connection_id\":\"{connection_id}\",\"end_user_id\":{end_user_id}}}\n"
    );
    writer
        .write_all(ok.as_bytes())
        .await
        .map_err(|e| e.to_string())?;

    // Keep connection open until client disconnects (push fan-out later).
    let mut sink = [0u8; 256];
    let mut reader = reader;
    loop {
        let n = reader.read(&mut sink).await.map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        // Ignore client payload for now; length-prefixed PING on client conn optional.
        if let Ok(Some((body, _))) = funnyx_socket::decode_frame(&sink[..n]) {
            if let Some(pong) = funnyx_heartbeat::pong_frame_if_ping(&body) {
                writer.write_all(&pong).await.map_err(|e| e.to_string())?;
            }
        }
    }

    let mut map = memory.write().await;
    map.client_sockets.remove(&connection_id);
    Ok(())
}

fn parse_auth_token(line: &str) -> Result<String, String> {
    let line = line.trim();
    if line.is_empty() {
        return Err("empty auth line".into());
    }
    if let Some(rest) = line
        .strip_prefix("AUTH ")
        .or_else(|| line.strip_prefix("auth "))
        .or_else(|| line.strip_prefix("Bearer "))
        .or_else(|| line.strip_prefix("bearer "))
    {
        let t = rest.trim();
        if t.is_empty() {
            return Err("empty token".into());
        }
        return Ok(t.to_string());
    }
    let auth: AuthLine =
        serde_json::from_str(line).map_err(|e| format!("auth json: {e}"))?;
    if auth.access_token.trim().is_empty() {
        return Err("empty access_token".into());
    }
    Ok(auth.access_token)
}
