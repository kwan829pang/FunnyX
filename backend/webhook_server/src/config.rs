use std::env;

use funnyx_config::constants::{
    DEFAULT_HTTP_HOST, DEFAULT_WEBHOOK_HTTP_PORT, DEFAULT_WEBHOOK_SOCKET_PORT,
};

use crate::post_auth::PostAuthRow;

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
    /// When true, client IP / Host may come from X-Forwarded-* / X-Real-IP.
    pub trust_proxy: bool,
    /// Memory fallback rows when POSTGRES_URL unset (from POST_AUTH_DEMO_SOURCES).
    pub demo_post_auth_rows: Vec<PostAuthRow>,
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
            trust_proxy: env::var("TRUST_PROXY")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            demo_post_auth_rows: parse_demo_post_auth(
                &env::var("POST_AUTH_DEMO_SOURCES").unwrap_or_else(|_| {
                    "payment_gate:payment-gate.local:demo_key:ppk_payment_gate:127.0.0.1|::1,\
partner_2001:*.company-a.local:demo_key:ppk_partner_2001:127.0.0.1|::1"
                        .into()
                }),
            ),
        })
    }
}

/// `source:domain:auth_key:ppk:ip|ip,...` comma-separated entries.
fn parse_demo_post_auth(raw: &str) -> Vec<PostAuthRow> {
    raw.split(',')
        .filter_map(|entry| {
            let parts: Vec<&str> = entry.trim().split(':').collect();
            if parts.len() < 5 {
                return None;
            }
            let source_code = parts[0].trim().to_string();
            let domain_pattern = parts[1].trim().to_string();
            let auth_key_suffix = parts[2].trim().to_string();
            let ppk = parts[3].trim().to_string();
            let allowed_ips = parts[4..].join(":").replace('|', ",");
            if source_code.is_empty() || domain_pattern.is_empty() || auth_key_suffix.is_empty() {
                return None;
            }
            Some(PostAuthRow {
                source_code,
                domain_pattern,
                auth_key_suffix,
                ppk: if ppk.is_empty() { None } else { Some(ppk) },
                allowed_ips,
            })
        })
        .collect()
}
