//! Socket network layer: compression, framing helpers, PING/PONG.
//!
//! Message layouts: `funnyx-socket-msg`. Spec: `doc/socket_message.md`.

use funnyx_error::{FunnyxError, Result};
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Socket heartbeat opcodes (Event List 00).
pub const PING: &[u8] = b"PING";
pub const PONG: &[u8] = b"PONG";

/// Compress payload for S2S sockets (lz4 with size prefix).
pub fn compress_s2s(plain: &[u8]) -> Vec<u8> {
    compress_prepend_size(plain)
}

/// Decompress S2S lz4 payload.
pub fn decompress_s2s(compressed: &[u8]) -> Result<Vec<u8>> {
    decompress_size_prepended(compressed)
        .map_err(|e| FunnyxError::Protocol(format!("lz4 decompress: {e}")))
}

/// Gzip-compress UTF-8 JSON for client-facing socket payloads.
pub fn compress_client_json(json: &str) -> Result<Vec<u8>> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(json.as_bytes())
        .map_err(|e| FunnyxError::Network(format!("gzip write: {e}")))?;
    encoder
        .finish()
        .map_err(|e| FunnyxError::Network(format!("gzip finish: {e}")))
}

/// Gunzip client JSON bytes to UTF-8 string.
pub fn decompress_client_json(compressed: &[u8]) -> Result<String> {
    use flate2::read::GzDecoder;
    let mut decoder = GzDecoder::new(compressed);
    let mut out = String::new();
    decoder
        .read_to_string(&mut out)
        .map_err(|e| FunnyxError::Network(format!("gunzip: {e}")))?;
    Ok(out)
}

/// Length-prefixed frame: 4-byte big-endian length + body.
pub fn encode_frame(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + body.len());
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(body);
    out
}

/// Split one length-prefixed frame from `buf`. Returns `(frame_body, rest)`.
pub fn decode_frame(buf: &[u8]) -> Result<Option<(Vec<u8>, usize)>> {
    if buf.len() < 4 {
        return Ok(None);
    }
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if buf.len() < 4 + len {
        return Ok(None);
    }
    Ok(Some((buf[4..4 + len].to_vec(), 4 + len)))
}

/// Classify heartbeat bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Heartbeat {
    Ping,
    Pong,
}

impl Heartbeat {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data == PING {
            Some(Self::Ping)
        } else if data == PONG {
            Some(Self::Pong)
        } else {
            None
        }
    }

    pub fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Ping => PING,
            Self::Pong => PONG,
        }
    }
}

/// Encode a JSON value then gzip for client sockets.
pub fn encode_client_json_message<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let json = serde_json::to_string(value)
        .map_err(|e| FunnyxError::Protocol(format!("json encode: {e}")))?;
    compress_client_json(&json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lz4_roundtrip() {
        let plain = b"hello funnyx s2s";
        let c = compress_s2s(plain);
        let d = decompress_s2s(&c).unwrap();
        assert_eq!(d, plain);
    }

    #[test]
    fn frame_roundtrip() {
        let body = b"abc";
        let framed = encode_frame(body);
        let (decoded, consumed) = decode_frame(&framed).unwrap().unwrap();
        assert_eq!(decoded, body);
        assert_eq!(consumed, framed.len());
    }

    #[test]
    fn heartbeat_parse() {
        assert_eq!(Heartbeat::parse(PING), Some(Heartbeat::Ping));
        assert_eq!(Heartbeat::parse(PONG), Some(Heartbeat::Pong));
    }
}
