use std::env;
use std::path::PathBuf;

use funnyx_config::constants::{
    DEFAULT_CONFIG_HTTP_PORT, DEFAULT_HTTP_HOST, DEFAULT_WHITELIST_PATH,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub service_name: String,
    pub public_base_url: String,
    pub whitelist_path: PathBuf,
    pub internal_api_key: String,
    pub heartbeat_timeout_ms: i64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port: u16 = env::var("HTTP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_CONFIG_HTTP_PORT);
        Ok(Self {
            host: env::var("HTTP_HOST").unwrap_or_else(|_| DEFAULT_HTTP_HOST.into()),
            port,
            service_name: env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "funnyx-config-server".into()),
            public_base_url: env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| format!("http://127.0.0.1:{port}"))
                .trim_end_matches('/')
                .to_string(),
            whitelist_path: resolve_whitelist_path(
                env::var("WHITELIST_PATH").unwrap_or_else(|_| DEFAULT_WHITELIST_PATH.into()),
            ),
            internal_api_key: env::var("INTERNAL_API_KEY")
                .unwrap_or_else(|_| "demo-internal-key".into()),
            heartbeat_timeout_ms: env::var("HEARTBEAT_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30_000),
        })
    }
}

fn resolve_whitelist_path(raw: String) -> PathBuf {
    let path = PathBuf::from(&raw);
    if path.exists() {
        return path;
    }
    let from_crate = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&raw);
    if from_crate.exists() {
        return from_crate;
    }
    let sample = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/whitelist.sample.json");
    if sample.exists() {
        return sample;
    }
    path
}
