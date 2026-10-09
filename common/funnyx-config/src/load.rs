//! Load `.env` into [`GlobalConfig`], filling missing keys from [`crate::constants`].

use crate::global::GlobalConfig;
use funnyx_error::Result;
use std::env;
use std::path::Path;

/// Load environment configuration.
///
/// - Attempts to load `.env` from the current directory (ignored if missing).
/// - Optionally loads an extra file when `env_file` is provided.
/// - For each field: if the env value is missing or empty, the constant default is used.
///
/// # Errors
///
/// Returns [`funnyx_error::FunnyxError::Config`] only for unrecoverable parse failures
/// (e.g. non-numeric port). Missing keys never fail when a default exists.
pub fn load(env_file: Option<&Path>) -> Result<GlobalConfig> {
    let _ = dotenvy::dotenv();
    if let Some(path) = env_file {
        let _ = dotenvy::from_path(path);
    }

    let mut cfg = GlobalConfig::defaults();

    cfg.service_name = env_string("SERVICE_NAME", &cfg.service_name);
    cfg.runtime_mode = env_string("RUNTIME_MODE", &cfg.runtime_mode);
    cfg.log_level = env_string("LOG_LEVEL", &cfg.log_level);
    cfg.http_host = env_string("HTTP_HOST", &cfg.http_host);
    cfg.http_port = env_u16("HTTP_PORT", cfg.http_port)?;
    cfg.socket_port = env_u16("SOCKET_PORT", cfg.socket_port)?;
    cfg.redis_url = env_string("REDIS_URL", &cfg.redis_url);
    cfg.postgres_url = env_string("POSTGRES_URL", &cfg.postgres_url);
    cfg.mongo_url = env_string("MONGO_URL", &cfg.mongo_url);
    cfg.heartbeat_interval_ms =
        env_u64("HEARTBEAT_INTERVAL_MS", cfg.heartbeat_interval_ms)?;
    cfg.socket_timeout_ms = env_u64("SOCKET_TIMEOUT_MS", cfg.socket_timeout_ms)?;
    cfg.http_timeout_ms = env_u64("HTTP_TIMEOUT_MS", cfg.http_timeout_ms)?;
    cfg.whitelist_path = env_string("WHITELIST_PATH", &cfg.whitelist_path);
    // Secret: no safe default; leave empty when missing.
    if let Ok(v) = env::var("SIGNING_SECRET") {
        if !v.trim().is_empty() {
            cfg.signing_secret = v;
        }
    }

    Ok(cfg)
}

/// Convenience: load with only process env + optional `.env` in CWD.
pub fn load_default() -> Result<GlobalConfig> {
    load(None)
}

fn env_string(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(v) if !v.trim().is_empty() => v,
        _ => default.to_string(),
    }
}

fn env_u16(key: &str, default: u16) -> Result<u16> {
    match env::var(key) {
        Ok(v) if !v.trim().is_empty() => v.parse::<u16>().map_err(|e| {
            funnyx_error::FunnyxError::Config(format!("{key} parse error: {e}"))
        }),
        _ => Ok(default),
    }
}

fn env_u64(key: &str, default: u64) -> Result<u64> {
    match env::var(key) {
        Ok(v) if !v.trim().is_empty() => v.parse::<u64>().map_err(|e| {
            funnyx_error::FunnyxError::Config(format!("{key} parse error: {e}"))
        }),
        _ => Ok(default),
    }
}

/// Documented env key names aligned with `env.sample` conventions.
pub mod keys {
    pub const SERVICE_NAME: &str = "SERVICE_NAME";
    pub const RUNTIME_MODE: &str = "RUNTIME_MODE";
    pub const LOG_LEVEL: &str = "LOG_LEVEL";
    pub const HTTP_HOST: &str = "HTTP_HOST";
    pub const HTTP_PORT: &str = "HTTP_PORT";
    pub const SOCKET_PORT: &str = "SOCKET_PORT";
    pub const REDIS_URL: &str = "REDIS_URL";
    pub const POSTGRES_URL: &str = "POSTGRES_URL";
    pub const MONGO_URL: &str = "MONGO_URL";
    pub const HEARTBEAT_INTERVAL_MS: &str = "HEARTBEAT_INTERVAL_MS";
    pub const SOCKET_TIMEOUT_MS: &str = "SOCKET_TIMEOUT_MS";
    pub const HTTP_TIMEOUT_MS: &str = "HTTP_TIMEOUT_MS";
    pub const WHITELIST_PATH: &str = "WHITELIST_PATH";
    pub const SIGNING_SECRET: &str = "SIGNING_SECRET";
}

/// Ensures defaults still match constants after a blank-env load.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants;
    use std::env;

    #[test]
    fn missing_env_uses_defaults() {
        // Clear keys we care about for this process (best-effort).
        for k in [
            "SERVICE_NAME",
            "HTTP_PORT",
            "SOCKET_PORT",
            "REDIS_URL",
            "SIGNING_SECRET",
        ] {
            env::remove_var(k);
        }
        let cfg = load_default().expect("load");
        assert_eq!(cfg.service_name, constants::DEFAULT_SERVICE_NAME);
        assert_eq!(cfg.http_port, constants::DEFAULT_GATEWAY_HTTP_PORT);
        assert_eq!(cfg.socket_port, constants::DEFAULT_GATEWAY_SOCKET_PORT);
        assert_eq!(cfg.redis_url, constants::DEFAULT_REDIS_URL);
        assert!(cfg.signing_secret.is_empty());
        assert!(cfg.require_signing_secret().is_err());
    }
}
