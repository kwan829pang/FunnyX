//! **Public** HTTP `GET /health` for third parties (Cloudflare, Game Partners, LB probes).
//!
//! In-system liveness between FunnyX services uses **private** socket PING/PONG
//! (`funnyx-heartbeat`). Do not treat `/health` as the platform registry heartbeat.
//!
//! Spec: `doc/socket_message.md` §4, `doc/project.md` §11.

use funnyx_error::Result;
use funnyx_time::TimestampMs;
use serde::{Deserialize, Serialize};

/// Standard **public** health response body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthPayload {
    pub status: HealthStatus,
    pub service: String,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Ok,
    Degraded,
    Down,
}

impl HealthPayload {
    /// Build an OK health payload for `service` with current UTC timestamp.
    pub fn ok(service: impl Into<String>) -> Result<Self> {
        Ok(Self {
            status: HealthStatus::Ok,
            service: service.into(),
            timestamp_ms: TimestampMs::now()?.as_i64(),
        })
    }

    pub fn degraded(service: impl Into<String>) -> Result<Self> {
        Ok(Self {
            status: HealthStatus::Degraded,
            service: service.into(),
            timestamp_ms: TimestampMs::now()?.as_i64(),
        })
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| funnyx_error::FunnyxError::Internal(format!("health json: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_payload_serializes() {
        let p = HealthPayload::ok("funnyx-gateway").unwrap();
        let s = p.to_json().unwrap();
        assert!(s.contains("ok"));
        assert!(s.contains("funnyx-gateway"));
    }
}
