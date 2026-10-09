//! Outbound socket PING → expect PONG (Config Server / peer probes).

use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::socket::{ping_frame, Heartbeat};

/// Parse `tcp://host:port`, `host:port`, or bare `host:port` into [`SocketAddr`].
pub fn parse_tcp_addr(socket_url: &str) -> Result<SocketAddr, String> {
    let s = socket_url.trim();
    let s = s
        .strip_prefix("tcp://")
        .or_else(|| s.strip_prefix("TCP://"))
        .unwrap_or(s);
    s.parse::<SocketAddr>()
        .map_err(|e| format!("invalid socket_url '{socket_url}': {e}"))
}

/// Connect, send length-prefixed PING, read one frame, require PONG.
pub async fn ping_peer(socket_url: &str, timeout_ms: u64) -> Result<(), String> {
    let addr = parse_tcp_addr(socket_url)?;
    let dur = Duration::from_millis(timeout_ms.max(100));
    timeout(dur, ping_peer_addr(addr))
        .await
        .map_err(|_| format!("socket ping timeout after {timeout_ms}ms to {addr}"))?
}

async fn ping_peer_addr(addr: SocketAddr) -> Result<(), String> {
    let mut stream = TcpStream::connect(addr)
        .await
        .map_err(|e| format!("connect {addr}: {e}"))?;
    stream
        .write_all(&ping_frame())
        .await
        .map_err(|e| format!("write PING: {e}"))?;

    let mut buf = Vec::new();
    let mut tmp = [0u8; 256];
    loop {
        let n = stream
            .read(&mut tmp)
            .await
            .map_err(|e| format!("read: {e}"))?;
        if n == 0 {
            return Err("peer closed before PONG".into());
        }
        buf.extend_from_slice(&tmp[..n]);
        match funnyx_socket::decode_frame(&buf).map_err(|e| e.to_string())? {
            None => continue,
            Some((body, _)) => {
                return match Heartbeat::parse(&body) {
                    Some(Heartbeat::Pong) => Ok(()),
                    Some(Heartbeat::Ping) => Err("peer replied PING, expected PONG".into()),
                    None => Err(format!("unexpected frame ({} bytes), expected PONG", body.len())),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tcp_url() {
        assert_eq!(
            parse_tcp_addr("tcp://127.0.0.1:9004").unwrap(),
            "127.0.0.1:9004".parse().unwrap()
        );
        assert_eq!(
            parse_tcp_addr("127.0.0.1:9001").unwrap(),
            "127.0.0.1:9001".parse().unwrap()
        );
    }
}
