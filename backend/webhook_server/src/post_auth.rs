//! Inbound HTTP Post_Auth: optional X-Callback-Source + wild Host domain + SHA-256 hash.

use std::net::SocketAddr;

use axum::http::HeaderMap;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct PostAuthRow {
    pub source_code: String,
    pub domain_pattern: String,
    pub auth_key_suffix: String,
    pub ppk: Option<String>,
    pub allowed_ips: String,
}

pub fn plain_token(suffix: &str) -> String {
    format!("Post_Auth_{suffix}")
}

pub fn hash_post_auth(plain: &str) -> String {
    let dig = Sha256::digest(plain.as_bytes());
    hex::encode(dig)
}

pub fn host_without_port(host: &str) -> String {
    let h = host.trim().to_ascii_lowercase();
    if let Some((name, port)) = h.rsplit_once(':') {
        if port.chars().all(|c| c.is_ascii_digit()) {
            return name.to_string();
        }
    }
    h
}

/// Exact or `*.suffix` wildcard (does not match the bare suffix alone).
pub fn domain_matches(pattern: &str, host: &str) -> bool {
    let pat = pattern.trim().to_ascii_lowercase();
    let host = host_without_port(host);
    if pat.is_empty() || pat == "*" || host.is_empty() {
        return false;
    }
    if let Some(suffix) = pat.strip_prefix("*.") {
        if suffix.is_empty() || suffix.contains('*') {
            return false;
        }
        host.ends_with(&format!(".{suffix}")) && host != suffix
    } else {
        pat == host
    }
}

pub fn ip_allowed(client_ip: &str, allowed_ips: &str) -> bool {
    let ip = normalize_ip(client_ip);
    allowed_ips
        .split(&[',', '|'][..])
        .map(|s| normalize_ip(s.trim()))
        .filter(|s| !s.is_empty())
        .any(|allowed| allowed == ip)
}

pub fn normalize_ip(ip: &str) -> String {
    let s = ip.trim();
    if let Some(rest) = s.strip_prefix(":ffff:") {
        return rest.to_string();
    }
    if let Some(rest) = s.strip_prefix("::ffff:") {
        return rest.to_string();
    }
    s.to_string()
}

pub fn resolve_client_ip(
    peer: SocketAddr,
    headers: &HeaderMap,
    trust_proxy: bool,
) -> String {
    if trust_proxy {
        if let Some(xff) = header_str(headers, "x-forwarded-for") {
            if let Some(first) = xff.split(',').next() {
                let t = first.trim();
                if !t.is_empty() {
                    return normalize_ip(t);
                }
            }
        }
        if let Some(real) = header_str(headers, "x-real-ip") {
            let t = real.trim();
            if !t.is_empty() {
                return normalize_ip(t);
            }
        }
    }
    normalize_ip(&peer.ip().to_string())
}

fn resolve_host(headers: &HeaderMap, trust_proxy: bool) -> Option<String> {
    if trust_proxy {
        if let Some(h) = header_str(headers, "x-forwarded-host") {
            let t = h.split(',').next().unwrap_or(h).trim();
            if !t.is_empty() {
                return Some(host_without_port(t));
            }
        }
    }
    header_str(headers, "host").map(|h| host_without_port(h.trim()))
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

fn ct_eq_hex(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut v = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        v |= x ^ y;
    }
    v == 0
}

/// Verify HTTP Post_Auth. Returns the matched row on success.
pub async fn verify(
    pool: Option<&PgPool>,
    config: &Config,
    peer: SocketAddr,
    headers: &HeaderMap,
) -> Result<PostAuthRow, String> {
    let post_auth = header_str(headers, "x-post-auth")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "X-Post-Auth required".to_string())?;

    let client_ip = resolve_client_ip(peer, headers, config.trust_proxy);
    let source = header_str(headers, "x-callback-source")
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let row = if let Some(key) = source {
        find_by_source_key(pool, config, key)
            .await?
            .ok_or_else(|| "unknown source".to_string())?
    } else {
        let host = resolve_host(headers, config.trust_proxy)
            .ok_or_else(|| "Host required when X-Callback-Source is omitted".to_string())?;
        find_by_domain(pool, config, &host)
            .await?
            .ok_or_else(|| "unknown source".to_string())?
    };

    if !ip_allowed(&client_ip, &row.allowed_ips) {
        return Err("source/ip mismatch".into());
    }

    let expected = hash_post_auth(&plain_token(&row.auth_key_suffix));
    let got = post_auth.to_ascii_lowercase();
    if !ct_eq_hex(&expected, &got) {
        return Err("invalid post auth".into());
    }

    tracing::debug!(
        source_code = %row.source_code,
        client_ip = %client_ip,
        "post_auth ok"
    );
    Ok(row)
}

async fn find_by_source_key(
    pool: Option<&PgPool>,
    config: &Config,
    key: &str,
) -> Result<Option<PostAuthRow>, String> {
    let rows = load_active(pool, config).await?;
    let key_norm = normalize_ip(key);
    Ok(rows.into_iter().find(|r| {
        ip_allowed(key, &r.allowed_ips)
            || r.auth_key_suffix == key
            || r.ppk.as_deref() == Some(key)
            || r.allowed_ips
                .split(&[',', '|'][..])
                .any(|s| normalize_ip(s.trim()) == key_norm)
    }))
}

async fn find_by_domain(
    pool: Option<&PgPool>,
    config: &Config,
    host: &str,
) -> Result<Option<PostAuthRow>, String> {
    let rows = load_active(pool, config).await?;
    Ok(rows
        .into_iter()
        .find(|r| domain_matches(&r.domain_pattern, host)))
}

async fn load_active(
    pool: Option<&PgPool>,
    config: &Config,
) -> Result<Vec<PostAuthRow>, String> {
    if let Some(pool) = pool {
        let rows = sqlx::query(crate::query::post_auth::LIST_ACTIVE)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        return Ok(rows
            .into_iter()
            .map(|r| PostAuthRow {
                source_code: r.get("source_code"),
                domain_pattern: r.get("domain_pattern"),
                auth_key_suffix: r.get("auth_key_suffix"),
                ppk: r.try_get("ppk").ok(),
                allowed_ips: r.get("allowed_ips"),
            })
            .collect());
    }
    Ok(config.demo_post_auth_rows.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wild_domain() {
        assert!(domain_matches("*.company-a.local", "pay.company-a.local"));
        assert!(!domain_matches("*.company-a.local", "company-a.local"));
        assert!(domain_matches("payment-gate.local", "payment-gate.local"));
        assert!(!domain_matches("*", "anything"));
    }

    #[test]
    fn hash_stable() {
        let h = hash_post_auth("Post_Auth_demo_key");
        assert_eq!(h.len(), 64);
        assert_eq!(h, hash_post_auth("Post_Auth_demo_key"));
    }
}
