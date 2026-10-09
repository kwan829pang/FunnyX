//! Cross-service `GlobalConfig` loaded once at process startup.

use crate::constants;
use serde::{Deserialize, Serialize};

/// Shared runtime configuration for FunnyX Rust services.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub service_name: String,
    pub runtime_mode: String,
    pub log_level: String,
    pub http_host: String,
    pub http_port: u16,
    pub socket_port: u16,
    pub redis_url: String,
    pub postgres_url: String,
    pub mongo_url: String,
    pub heartbeat_interval_ms: u64,
    pub socket_timeout_ms: u64,
    pub http_timeout_ms: u64,
    pub whitelist_path: String,
    /// Optional signing secret; empty means unset (fail if required by caller).
    pub signing_secret: String,
}

impl GlobalConfig {
    /// Built-in defaults when no `.env` is present.
    pub fn defaults() -> Self {
        Self {
            service_name: constants::DEFAULT_SERVICE_NAME.to_string(),
            runtime_mode: constants::DEFAULT_RUNTIME_MODE.to_string(),
            log_level: constants::DEFAULT_LOG_LEVEL.to_string(),
            http_host: constants::DEFAULT_HTTP_HOST.to_string(),
            http_port: constants::DEFAULT_GATEWAY_HTTP_PORT,
            socket_port: constants::DEFAULT_GATEWAY_SOCKET_PORT,
            redis_url: constants::DEFAULT_REDIS_URL.to_string(),
            postgres_url: constants::DEFAULT_POSTGRES_URL.to_string(),
            mongo_url: constants::DEFAULT_MONGO_URL.to_string(),
            heartbeat_interval_ms: constants::DEFAULT_HEARTBEAT_INTERVAL_MS,
            socket_timeout_ms: constants::DEFAULT_SOCKET_TIMEOUT_MS,
            http_timeout_ms: constants::DEFAULT_HTTP_TIMEOUT_MS,
            whitelist_path: constants::DEFAULT_WHITELIST_PATH.to_string(),
            signing_secret: String::new(),
        }
    }

    /// Returns an error when a signing secret is required but missing.
    pub fn require_signing_secret(&self) -> funnyx_error::Result<&str> {
        if self.signing_secret.is_empty() {
            return Err(funnyx_error::FunnyxError::Config(
                "SIGNING_SECRET is required and has no safe default".into(),
            ));
        }
        Ok(&self.signing_secret)
    }
}
