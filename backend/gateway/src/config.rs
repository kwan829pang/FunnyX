use std::env;

use funnyx_config::constants::{
    DEFAULT_GATEWAY_HTTP_PORT, DEFAULT_GATEWAY_SOCKET_PORT, DEFAULT_HTTP_HOST,
    DEFAULT_HTTP_TIMEOUT_MS,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub socket_host: String,
    pub socket_port: u16,
    pub service_name: String,
    pub public_base_url: String,
    pub client_center_url: String,
    pub admin_api_url: String,
    pub message_center_url: String,
    pub session_token_server_url: String,
    pub session_internal_api_key: String,
    pub internal_api_key: String,
    pub config_server_url: String,
    pub registry_refresh_ms: u64,
    pub http_timeout_ms: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port: u16 = env::var("HTTP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_GATEWAY_HTTP_PORT);
        Ok(Self {
            host: env::var("HTTP_HOST").unwrap_or_else(|_| DEFAULT_HTTP_HOST.into()),
            port,
            socket_host: env::var("SOCKET_HOST").unwrap_or_else(|_| DEFAULT_HTTP_HOST.into()),
            socket_port: env::var("SOCKET_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_GATEWAY_SOCKET_PORT),
            service_name: env::var("SERVICE_NAME").unwrap_or_else(|_| "funnyx-gateway".into()),
            public_base_url: env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| format!("http://127.0.0.1:{port}"))
                .trim_end_matches('/')
                .to_string(),
            client_center_url: env::var("CLIENT_CENTER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8083".into())
                .trim_end_matches('/')
                .to_string(),
            admin_api_url: env::var("ADMIN_API_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:18300".into())
                .trim_end_matches('/')
                .to_string(),
            message_center_url: env::var("MESSAGE_CENTER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8085".into())
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
            registry_refresh_ms: env::var("REGISTRY_REFRESH_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5_000)
                .max(1_000),
            http_timeout_ms: env::var("HTTP_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_HTTP_TIMEOUT_MS)
                .max(1_000),
        })
    }
}
