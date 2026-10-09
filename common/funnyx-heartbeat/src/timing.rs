//! Heartbeat interval / timeout / retry defaults.

use funnyx_config::constants::{
    DEFAULT_HEARTBEAT_INTERVAL_MS, DEFAULT_SOCKET_TIMEOUT_MS,
};
use serde::{Deserialize, Serialize};

/// Shared timing knobs (whitelist JSON + service env).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HeartbeatTiming {
    pub interval_ms: u64,
    pub timeout_ms: u64,
    pub retry_count: u32,
}

impl Default for HeartbeatTiming {
    fn default() -> Self {
        Self {
            interval_ms: DEFAULT_HEARTBEAT_INTERVAL_MS,
            timeout_ms: DEFAULT_SOCKET_TIMEOUT_MS,
            retry_count: 3,
        }
    }
}

impl HeartbeatTiming {
    pub fn from_env_or_default() -> Self {
        let mut t = Self::default();
        if let Ok(v) = std::env::var("HEARTBEAT_INTERVAL_MS") {
            if let Ok(n) = v.parse() {
                t.interval_ms = n;
            }
        }
        if let Ok(v) = std::env::var("HEARTBEAT_TIMEOUT_MS") {
            if let Ok(n) = v.parse() {
                t.timeout_ms = n;
            }
        }
        if let Ok(v) = std::env::var("HEARTBEAT_RETRY_COUNT") {
            if let Ok(n) = v.parse() {
                t.retry_count = n;
            }
        }
        t
    }
}

/// True when `now_ms - last_heartbeat_ms` exceeds `timeout_ms`.
pub fn is_stale(last_heartbeat_ms: i64, timeout_ms: i64, now_ms: i64) -> bool {
    if last_heartbeat_ms <= 0 {
        return true;
    }
    now_ms.saturating_sub(last_heartbeat_ms) > timeout_ms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_when_old() {
        assert!(is_stale(1_000, 5_000, 10_000));
        assert!(!is_stale(8_000, 5_000, 10_000));
    }
}
