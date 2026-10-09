use std::env;

use funnyx_config::constants::{
    DEFAULT_HTTP_HOST, DEFAULT_MESSAGE_CENTER_HTTP_PORT, DEFAULT_MESSAGE_CENTER_SOCKET_PORT,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub socket_host: String,
    pub socket_port: u16,
    pub service_name: String,
    pub public_base_url: String,
    pub session_token_server_url: String,
    pub session_internal_api_key: String,
    pub internal_api_key: String,
    pub config_server_url: String,
    pub postgres_url: Option<String>,
    pub mongo_url: Option<String>,
    pub mongo_db: String,
    pub partner_notice_url: Option<String>,
    pub drain_interval_ms: u64,
    pub drain_batch_size: i64,
    pub max_delivery_retries: i32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port: u16 = env::var("HTTP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_MESSAGE_CENTER_HTTP_PORT);
        Ok(Self {
            host: env::var("HTTP_HOST").unwrap_or_else(|_| DEFAULT_HTTP_HOST.into()),
            port,
            socket_host: env::var("SOCKET_HOST").unwrap_or_else(|_| DEFAULT_HTTP_HOST.into()),
            socket_port: env::var("SOCKET_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_MESSAGE_CENTER_SOCKET_PORT),
            service_name: env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "funnyx-message-center".into()),
            public_base_url: env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| format!("http://127.0.0.1:{port}"))
                .trim_end_matches('/')
                .to_string(),
            session_token_server_url: env::var("SESSION_TOKEN_SERVER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8082".into())
                .trim_end_matches('/')
                .to_string(),
            session_internal_api_key: env::var("SESSION_INTERNAL_API_KEY")
                .unwrap_or_else(|_| "demo-internal-key".into()),
            internal_api_key: env::var("INTERNAL_API_KEY")
                .or_else(|_| env::var("SESSION_INTERNAL_API_KEY"))
                .unwrap_or_else(|_| "demo-internal-key".into()),
            config_server_url: env::var("CONFIG_SERVER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8090".into())
                .trim_end_matches('/')
                .to_string(),
            postgres_url: env::var("POSTGRES_URL")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && !s.contains("xxx.xxx.xxx.xxx")),
            mongo_url: env::var("MONGO_URL")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && !s.contains("xxx.xxx.xxx.xxx")),
            mongo_db: env::var("MONGO_DB").unwrap_or_else(|_| "funnyx_message".into()),
            partner_notice_url: env::var("PARTNER_NOTICE_URL")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            drain_interval_ms: env::var("DRAIN_INTERVAL_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2_000),
            drain_batch_size: env::var("DRAIN_BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(32)
                .max(1),
            max_delivery_retries: env::var("MAX_DELIVERY_RETRIES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5)
                .max(1),
        })
    }
}
