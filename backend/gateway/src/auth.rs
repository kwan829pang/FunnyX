//! Path auth classification and Gateway-side credential checks.

use axum::http::{HeaderMap, StatusCode};

use funnyx_net_api::headers;

use crate::sts::{ErrorBody, StsClient, StsValidate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthKind {
    Public,
    Session,
    Corp,
    Admin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpstreamTarget {
    ClientCenter,
    MessageCenter,
    AdminApi,
}

/// Paths that must never be proxied through the public Gateway.
pub fn is_blocked(path: &str) -> bool {
    let p = normalize(path);
    p.starts_with("/v1/internal/")
        || p == "/v1/internal"
        || p.starts_with("/v1/config/")
        || p == "/v1/config"
        || p.starts_with("/v1/webhook/")
        || p == "/v1/webhook"
        || p.starts_with("/v1/notices")
        || p == "/v1/session/validate"
        || p == "/v1/session/revoke"
        || p.starts_with("/v1/engine")
}

fn normalize(path: &str) -> &str {
    if path.is_empty() {
        "/"
    } else {
        path.split('?').next().unwrap_or(path)
    }
}

pub fn classify(path: &str) -> Option<(UpstreamTarget, AuthKind)> {
    let p = normalize(path);

    if is_blocked(p) {
        return None;
    }

    if p == "/v1/admin/login" {
        return Some((UpstreamTarget::AdminApi, AuthKind::Public));
    }
    if p.starts_with("/v1/admin/") || p == "/v1/admin" {
        return Some((UpstreamTarget::AdminApi, AuthKind::Admin));
    }

    if p.starts_with("/v1/notifications") {
        return Some((UpstreamTarget::MessageCenter, AuthKind::Session));
    }

    if is_public_client_path(p) {
        return Some((UpstreamTarget::ClientCenter, AuthKind::Public));
    }

    if p.starts_with("/v1/corp/") || p == "/v1/corp" {
        // corp/register is Public; other corp paths are MasterSigned
        if p == "/v1/corp/register" {
            return Some((UpstreamTarget::ClientCenter, AuthKind::Public));
        }
        return Some((UpstreamTarget::ClientCenter, AuthKind::Corp));
    }

    if p.starts_with("/v1/client/")
        || p.starts_with("/v1/shop/")
        || p == "/v1/shop"
        || p.starts_with("/v1/corp-tokens")
        || p.starts_with("/v1/markets")
        || p.starts_with("/v1/orders")
        || p.starts_with("/v1/marketplace")
        || p.starts_with("/v1/oauth/")
        || p == "/v1/session/token"
    {
        return Some((UpstreamTarget::ClientCenter, AuthKind::Session));
    }

    None
}

fn is_public_client_path(p: &str) -> bool {
    matches!(
        p,
        "/v1/client/register"
            | "/v1/client/login"
            | "/v1/client/games"
            | "/v1/session/token"
            | "/v1/oauth/authorize"
            | "/v1/oauth/token"
            | "/v1/oauth/userinfo"
            | "/v1/oauth/login"
            | "/v1/client/oauth/partner/complete"
            | "/v1/client/oauth/partner/callback"
            | "/v1/corp/register"
    ) || p.starts_with("/v1/client/oauth/partner/") && p.ends_with("/start")
}

pub fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    let raw = headers
        .get(headers::AUTHORIZATION)
        .or_else(|| headers.get("authorization"))?
        .to_str()
        .ok()?;
    let token = raw.strip_prefix("Bearer ").or_else(|| raw.strip_prefix("bearer "))?;
    let t = token.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

pub fn require_corp_headers(headers: &HeaderMap) -> Result<(), (StatusCode, ErrorBody)> {
    let required = [
        headers::MASTER_ACCOUNT_CODE,
        headers::MASTER_ID,
        headers::API_KEY,
        headers::SIGNATURE,
        headers::TIMESTAMP,
    ];
    for name in required {
        let ok = headers
            .get(name)
            .or_else(|| {
                // try lowercase
                headers.get(name.to_ascii_lowercase())
            })
            .and_then(|v| v.to_str().ok())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if !ok {
            return Err((
                StatusCode::UNAUTHORIZED,
                ErrorBody {
                    error: format!("missing {name}"),
                },
            ));
        }
    }
    Ok(())
}

pub async fn gate_session(
    sts: &StsClient,
    headers: &HeaderMap,
) -> Result<StsValidate, (StatusCode, ErrorBody)> {
    let token = extract_bearer(headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            ErrorBody {
                error: "missing Bearer token".into(),
            },
        )
    })?;
    let v = sts.validate(&token).await?;
    if !v.valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            ErrorBody {
                error: v.reason.unwrap_or_else(|| "invalid session".into()),
            },
        ));
    }
    Ok(v)
}

pub fn require_bearer(headers: &HeaderMap) -> Result<String, (StatusCode, ErrorBody)> {
    extract_bearer(headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            ErrorBody {
                error: "missing Bearer token".into(),
            },
        )
    })
}
