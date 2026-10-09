use std::env;

use funnyx_config::constants::{
    DEFAULT_HTTP_HOST, DEFAULT_WEBHOOK_HTTP_PORT, DEFAULT_WEBHOOK_SOCKET_PORT,
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
    pub internal_api_key: String,
    /// Empty disables Config Server registry register.
    pub config_server_url: String,
    pub callback_signature: String,
    pub demo_master_code: String,
    pub demo_master_id: String,
    pub demo_api_key: String,
    pub demo_master_secret: String,
    pub postgres_url: Option<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port: u16 = env::var("HTTP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_WEBHOOK_HTTP_PORT);
        let socket_port: u16 = env::var("SOCKET_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_WEBHOOK_SOCKET_PORT);
        Ok(Self {
            host: env::var("HTTP_HOST").unwrap_or_else(|_| DEFAULT_HTTP_HOST.into()),
            port,
            socket_host: env::var("SOCKET_HOST").unwrap_or_else(|_| DEFAULT_HTTP_HOST.into()),
            socket_port,
            service_name: env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "funnyx-webhook-server".into()),
            public_base_url: env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| format!("http://127.0.0.1:{port}"))
                .trim_end_matches('/')
                .to_string(),
            client_center_url: env::var("CLIENT_CENTER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8083".into())
                .trim_end_matches('/')
                .to_string(),
            internal_api_key: env::var("INTERNAL_API_KEY")
                .unwrap_or_else(|_| "demo-internal-key".into()),
            config_server_url: env::var("CONFIG_SERVER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8090".into())
                .trim_end_matches('/')
                .to_string(),
            callback_signature: env::var("CALLBACK_SIGNATURE")
                .unwrap_or_else(|_| "test-signature".into()),
            demo_master_code: env::var("DEMO_MASTER_CODE")
                .unwrap_or_else(|_| "DEMO_MASTER_CODE".into()),
            demo_master_id: env::var("DEMO_MASTER_ID").unwrap_or_else(|_| "DEMO_MASTER_ID".into()),
            demo_api_key: env::var("DEMO_API_KEY")
                .unwrap_or_else(|_| "demo_api_key_do_not_use_live".into()),
            demo_master_secret: env::var("DEMO_MASTER_SECRET")
                .unwrap_or_else(|_| "demo_secret_do_not_use_live".into()),
            postgres_url: env::var("POSTGRES_URL")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && !s.contains("xxx.xxx.xxx.xxx")),
        })
    }
}
