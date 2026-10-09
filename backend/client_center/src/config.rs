use std::env;

use funnyx_config::constants::DEFAULT_CLIENT_CENTER_SOCKET_PORT;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub socket_host: String,
    pub socket_port: u16,
    pub service_name: String,
    pub public_base_url: String,
    pub client_web_redirect: String,
    pub session_token_server_url: String,
    pub session_internal_api_key: String,
    /// Empty disables Config Server registry register.
    pub config_server_url: String,
    pub internal_api_key: String,
    pub default_partner_id: String,
    pub partner_a_base_url: String,
    pub partner_a_api_key: String,
    pub postgres_url: Option<String>,
    pub payment_gate_url: String,
    pub shop_payment_callback_url: String,
    pub demo_master_code: String,
    pub demo_master_id: String,
    pub demo_api_key: String,
    pub demo_master_secret: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: env::var("HTTP_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("HTTP_PORT")
                .unwrap_or_else(|_| "8083".into())
                .parse()?,
            socket_host: env::var("SOCKET_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            socket_port: env::var("SOCKET_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_CLIENT_CENTER_SOCKET_PORT),
            service_name: env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "funnyx-client-center".into()),
            public_base_url: env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8083".into())
                .trim_end_matches('/')
                .to_string(),
            client_web_redirect: env::var("CLIENT_WEB_REDIRECT")
                .unwrap_or_else(|_| "http://127.0.0.1:3000/oauth/callback".into()),
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
            default_partner_id: env::var("DEFAULT_PARTNER_ID")
                .unwrap_or_else(|_| "partner_2001".into()),
            partner_a_base_url: env::var("PARTNER_A_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:18102".into())
                .trim_end_matches('/')
                .to_string(),
            partner_a_api_key: env::var("PARTNER_A_API_KEY")
                .unwrap_or_else(|_| "demo-api-key".into()),
            postgres_url: env::var("POSTGRES_URL")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && !s.contains("xxx.xxx.xxx.xxx")),
            payment_gate_url: env::var("PAYMENT_GATE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:18100".into())
                .trim_end_matches('/')
                .to_string(),
            shop_payment_callback_url: env::var("SHOP_PAYMENT_CALLBACK_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8084/v1/webhook/shop/payment".into()),
            demo_master_code: env::var("DEMO_MASTER_CODE")
                .unwrap_or_else(|_| "DEMO_MASTER_CODE".into()),
            demo_master_id: env::var("DEMO_MASTER_ID").unwrap_or_else(|_| "DEMO_MASTER_ID".into()),
            demo_api_key: env::var("DEMO_API_KEY")
                .unwrap_or_else(|_| "demo_api_key_do_not_use_live".into()),
            demo_master_secret: env::var("DEMO_MASTER_SECRET")
                .unwrap_or_else(|_| "demo_secret_do_not_use_live".into()),
        })
    }

    pub fn partner_oauth_complete_url(&self) -> String {
        format!(
            "{}/v1/client/oauth/partner/complete",
            self.public_base_url
        )
    }
}
