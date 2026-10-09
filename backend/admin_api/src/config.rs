use std::env;

use funnyx_config::constants::DEFAULT_ADMIN_API_SOCKET_PORT;

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
    pub config_server_url: String,
    pub internal_api_key: String,
    pub postgres_url: Option<String>,
    /// Fallback when Config Server has no core-engine instance.
    pub core_engine_url: String,
    pub core_engine_admin_key: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: env::var("HTTP_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("HTTP_PORT")
                .unwrap_or_else(|_| "18300".into())
                .parse()?,
            socket_host: env::var("SOCKET_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            socket_port: env::var("SOCKET_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_ADMIN_API_SOCKET_PORT),
            service_name: env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "funnyx-admin-api".into()),
            public_base_url: env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:18300".into())
                .trim_end_matches('/')
                .to_string(),
            session_token_server_url: env::var("SESSION_TOKEN_SERVER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8082".into())
                .trim_end_matches('/')
                .to_string(),
            session_internal_api_key: env::var("SESSION_INTERNAL_API_KEY")
                .unwrap_or_else(|_| "demo-internal-key".into()),
            config_server_url: env::var("CONFIG_SERVER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8090".into())
                .trim_end_matches('/')
                .to_string(),
            internal_api_key: env::var("INTERNAL_API_KEY")
                .or_else(|_| env::var("SESSION_INTERNAL_API_KEY"))
                .unwrap_or_else(|_| "demo-internal-key".into()),
            postgres_url: env::var("POSTGRES_URL")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && !s.contains("xxx.xxx.xxx.xxx")),
            core_engine_url: env::var("CORE_ENGINE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:18200".into())
                .trim_end_matches('/')
                .to_string(),
            core_engine_admin_key: env::var("CORE_ENGINE_ADMIN_KEY")
                .or_else(|_| env::var("ADMIN_API_KEY"))
                .unwrap_or_else(|_| "demo-admin-key".into()),
        })
    }
}
