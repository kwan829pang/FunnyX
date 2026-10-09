use anyhow::{anyhow, Context};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub core_engine_url: String,
    pub symbols: Vec<String>,
    pub poll_interval_ms: u64,
    pub jitter_ms: u64,
    pub min_qty: u64,
    pub max_qty: u64,
    pub default_price: u64,
    pub take_partial_pct: u8,
    pub take_full_pct: u8,
    pub make_pct: u8,
    pub account_id: String,
    pub end_user_id: Option<i64>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let take_partial_pct = env_u8("TAKE_PARTIAL_PCT", 40)?;
        let take_full_pct = env_u8("TAKE_FULL_PCT", 25)?;
        let make_pct = env_u8("MAKE_PCT", 25)?;
        if take_partial_pct as u16 + take_full_pct as u16 + make_pct as u16 > 100 {
            return Err(anyhow!(
                "TAKE_PARTIAL_PCT + TAKE_FULL_PCT + MAKE_PCT must be <= 100"
            ));
        }
        let min_qty = env_u64("MIN_QTY", 1)?.max(1);
        let max_qty = env_u64("MAX_QTY", 3)?.max(min_qty);
        Ok(Self {
            host: std::env::var("HTTP_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env_u64("HTTP_PORT", 18103)? as u16,
            core_engine_url: std::env::var("CORE_ENGINE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:18200".into())
                .trim_end_matches('/')
                .to_string(),
            symbols: std::env::var("SYMBOLS")
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_ascii_uppercase())
                .collect(),
            poll_interval_ms: env_u64("POLL_INTERVAL_MS", 800)?.max(100),
            jitter_ms: env_u64("JITTER_MS", 1200)?,
            min_qty,
            max_qty,
            default_price: env_u64("DEFAULT_PRICE", 50000)?.max(1),
            take_partial_pct,
            take_full_pct,
            make_pct,
            account_id: std::env::var("ACCOUNT_ID").unwrap_or_else(|_| "market_bot".into()),
            end_user_id: std::env::var("END_USER_ID")
                .ok()
                .filter(|s| !s.is_empty())
                .map(|s| s.parse().context("END_USER_ID"))
                .transpose()?,
        })
    }
}

fn env_u64(key: &str, default: u64) -> anyhow::Result<u64> {
    match std::env::var(key) {
        Ok(v) if !v.is_empty() => v.parse().with_context(|| format!("parse {key}")),
        _ => Ok(default),
    }
}

fn env_u8(key: &str, default: u8) -> anyhow::Result<u8> {
    Ok(env_u64(key, default as u64)? as u8)
}
