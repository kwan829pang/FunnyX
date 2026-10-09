//! Minimal socket listener: answer PING with PONG (heartbeat core).

use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::socket::pong_frame_if_ping;

/// Bind `addr` and reply to length-prefixed PING frames with PONG.
/// Non-PING frames are ignored (services with richer sockets use their own handler).
pub fn spawn_ping_pong_listener(addr: SocketAddr) {
    tokio::spawn(async move {
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                tracing::info!(%addr, "funnyx-heartbeat: PING/PONG listener ready");
                loop {
                    match listener.accept().await {
                        Ok((stream, peer)) => {
                            tokio::spawn(async move {
                                if let Err(e) = handle_conn(stream).await {
                                    tracing::debug!(%peer, "heartbeat socket closed: {e}");
                                }
                            });
                        }
                        Err(e) => tracing::warn!("heartbeat accept: {e}"),
                    }
                }
            }
            Err(e) => tracing::error!(%addr, "heartbeat socket bind failed: {e}"),
        }
    });
}

async fn handle_conn(mut stream: tokio::net::TcpStream) -> Result<(), String> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1024];
    loop {
        let n = stream
            .read(&mut tmp)
            .await
            .map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        loop {
            match funnyx_socket::decode_frame(&buf).map_err(|e| e.to_string())? {
                None => break,
                Some((body, consumed)) => {
                    buf.drain(..consumed);
                    if let Some(pong) = pong_frame_if_ping(&body) {
                        stream
                            .write_all(&pong)
                            .await
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }
    Ok(())
}
