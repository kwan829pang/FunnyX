//! Socket PING/PONG helpers (length-prefixed frames).

pub use funnyx_socket::{Heartbeat, PING, PONG};

/// If `frame_body` is a PING, return an encoded PONG frame ready to write.
pub fn pong_frame_if_ping(frame_body: &[u8]) -> Option<Vec<u8>> {
    match Heartbeat::parse(frame_body) {
        Some(Heartbeat::Ping) => Some(funnyx_socket::encode_frame(PONG)),
        _ => None,
    }
}

/// Encode a PING frame for outbound liveness probes.
pub fn ping_frame() -> Vec<u8> {
    funnyx_socket::encode_frame(PING)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_gets_pong_frame() {
        let framed = funnyx_socket::encode_frame(PING);
        let (body, _) = funnyx_socket::decode_frame(&framed).unwrap().unwrap();
        let pong = pong_frame_if_ping(&body).expect("pong");
        let (pong_body, _) = funnyx_socket::decode_frame(&pong).unwrap().unwrap();
        assert_eq!(Heartbeat::parse(&pong_body), Some(Heartbeat::Pong));
    }
}
