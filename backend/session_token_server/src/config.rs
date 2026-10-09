use std::collections::HashMap;

use funnyx_config::constants::{
    DEFAULT_HTTP_HOST, DEFAULT_SESSION_HTTP_PORT, DEFAULT_SESSION_SOCKET_PORT,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub socket_host: String,
    pub socket_port: u16,
    pub service_name: String,
    pub public_base_url: String,
    pub store_backend: StoreBackend,
    pub redis_url: String,
    pub token_ttl_secs: u64,
    pub refresh_ttl_secs: u64,
    pub oauth_code_ttl_secs: u64,
    pub internal_api_key: String,
    pub config_server_url: String,
    pub client_web_redirect: String,
    pub platform_oauth_client: OauthClientConfig,
    pub partners: HashMap<String, PartnerOauthConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreBackend {
    Memory,
    Redis,
}

#[derive(Debug, Clone)]
pub struct OauthClientConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PartnerOauthConfig {
    pub partner_id: String,
    pub authorize_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub client_id: String,
    pub client_secret: String,
    /// Registered redirect on partner side (points at STS callback).
    pub redirect_uri: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let store = match std::env::var("SESSION_STORE")
            .unwrap_or_else(|_| "memory".into())
            .to_ascii_lowercase()
            .as_str()
        {
            "redis" => StoreBackend::Redis,
            _ => StoreBackend::Memory,
        };
        let host = std::env::var("HTTP_HOST").unwrap_or_else(|_| DEFAULT_HTTP_HOST.into());
        let port = env_u16("HTTP_PORT", DEFAULT_SESSION_HTTP_PORT)?;
        let public_base_url = std::env::var("PUBLIC_BASE_URL")
            .unwrap_or_else(|_| format!("http://127.0.0.1:{port}"));

        let platform_oauth_client = OauthClientConfig {
            client_id: std::env::var("PLATFORM_OAUTH_CLIENT_ID")
                .unwrap_or_else(|_| "platform_partner_client".into()),
            client_secret: std::env::var("PLATFORM_OAUTH_CLIENT_SECRET")
                .unwrap_or_else(|_| "platform_partner_secret".into()),
            redirect_uris: std::env::var("PLATFORM_OAUTH_REDIRECT_URIS")
                .unwrap_or_else(|_| {
                    "http://127.0.0.1:18102/oauth/callback,http://127.0.0.1:3000/oauth/callback"
                        .into()
                })
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect(),
        };

        let mut partners = HashMap::new();
        let partner_id =
            std::env::var("PARTNER_A_ID").unwrap_or_else(|_| "partner_2001".into());
        let partner_redirect = std::env::var("PARTNER_A_REDIRECT_URI").unwrap_or_else(|_| {
            format!("{public_base_url}/v1/client/oauth/partner/callback")
        });
        partners.insert(
            partner_id.clone(),
            PartnerOauthConfig {
                partner_id: partner_id.clone(),
                authorize_url: std::env::var("PARTNER_A_OAUTH_AUTHORIZE")
                    .unwrap_or_else(|_| "http://127.0.0.1:18102/oauth/authorize".into()),
                token_url: std::env::var("PARTNER_A_OAUTH_TOKEN")
                    .unwrap_or_else(|_| "http://127.0.0.1:18102/oauth/token".into()),
                userinfo_url: std::env::var("PARTNER_A_OAUTH_USERINFO")
                    .unwrap_or_else(|_| "http://127.0.0.1:18102/oauth/userinfo".into()),
                client_id: std::env::var("PARTNER_A_CLIENT_ID")
                    .unwrap_or_else(|_| "company_a_client".into()),
                client_secret: std::env::var("PARTNER_A_CLIENT_SECRET")
                    .unwrap_or_else(|_| "company_a_secret".into()),
                redirect_uri: partner_redirect,
            },
        );

        let socket_host =
            std::env::var("SOCKET_HOST").unwrap_or_else(|_| DEFAULT_HTTP_HOST.into());
        let socket_port = env_u16("SOCKET_PORT", DEFAULT_SESSION_SOCKET_PORT)?;

        Ok(Self {
            host,
            port,
            socket_host,
            socket_port,
            service_name: std::env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "funnyx-session-token-server".into()),
            public_base_url,
            store_backend: store,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            token_ttl_secs: env_u64("TOKEN_TTL_SECS", 3600)?.max(60),
            refresh_ttl_secs: env_u64("REFRESH_TTL_SECS", 604_800)?.max(60),
            oauth_code_ttl_secs: env_u64("OAUTH_CODE_TTL_SECS", 300)?.max(30),
            internal_api_key: std::env::var("INTERNAL_API_KEY")
                .unwrap_or_else(|_| "demo-internal-key".into()),
            config_server_url: std::env::var("CONFIG_SERVER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8090".into())
                .trim_end_matches('/')
                .to_string(),
            client_web_redirect: std::env::var("CLIENT_WEB_REDIRECT")
                .unwrap_or_else(|_| "http://127.0.0.1:3000/oauth/callback".into()),
            platform_oauth_client,
            partners,
        })
    }

    pub fn partner(&self, partner_id: &str) -> Option<&PartnerOauthConfig> {
        self.partners.get(partner_id)
    }
}

fn env_u16(key: &str, default: u16) -> anyhow::Result<u16> {
    Ok(env_u64(key, default as u64)? as u16)
}

fn env_u64(key: &str, default: u64) -> anyhow::Result<u64> {
    match std::env::var(key) {
        Ok(v) if !v.is_empty() => Ok(v.parse()?),
        _ => Ok(default),
    }
}
