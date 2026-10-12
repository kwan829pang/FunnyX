//! Admin CRUD for inbound webhook Post_Auth mapping.

use std::net::IpAddr;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, extract_bearer, require_admin_session, AppState};
use crate::models::ErrorBody;

async fn require_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<i64, (StatusCode, Json<ErrorBody>)> {
    let token = extract_bearer(headers)
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "Authorization Bearer required"))?;
    let (admin, _) = require_admin_session(state, &token).await?;
    Ok(admin.admin_user_id)
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBody {
    pub source_code: String,
    pub domain_pattern: String,
    pub auth_key_suffix: String,
    #[serde(default)]
    pub ppk: Option<String>,
    pub allowed_ips: String,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBody {
    pub source_code: String,
    pub domain_pattern: String,
    pub auth_key_suffix: String,
    #[serde(default)]
    pub ppk: Option<String>,
    pub allowed_ips: String,
}

#[derive(Debug, Deserialize)]
pub struct StatusBody {
    pub status: String,
}

pub async fn list_items(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    if let Some(s) = q.status.as_deref() {
        validate_status(s)?;
    }
    let items = state
        .webhook_post_auth
        .list(q.status.as_deref())
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({
        "items": items,
        "source": "admin_api",
    })))
}

pub async fn get_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let item = state
        .webhook_post_auth
        .get(id)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "not found"))?;
    Ok(Json(serde_json::json!({
        "item": item,
        "source": "admin_api",
    })))
}

pub async fn create_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let source_code = body.source_code.trim();
    let domain_pattern = body.domain_pattern.trim();
    let auth_key_suffix = body.auth_key_suffix.trim();
    let allowed_ips = normalize_ips(&body.allowed_ips)?;
    let ppk = body
        .ppk
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let status = body.status.as_deref().unwrap_or("active").trim();
    validate_fields(source_code, domain_pattern, auth_key_suffix, &allowed_ips)?;
    validate_status(status)?;
    let item = state
        .webhook_post_auth
        .create(
            source_code,
            domain_pattern,
            auth_key_suffix,
            ppk,
            &allowed_ips,
            status,
        )
        .await
        .map_err(map_store_err)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "item": item,
            "source": "admin_api",
        })),
    ))
}

pub async fn update_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<UpdateBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let source_code = body.source_code.trim();
    let domain_pattern = body.domain_pattern.trim();
    let auth_key_suffix = body.auth_key_suffix.trim();
    let allowed_ips = normalize_ips(&body.allowed_ips)?;
    let ppk = body
        .ppk
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    validate_fields(source_code, domain_pattern, auth_key_suffix, &allowed_ips)?;
    let item = state
        .webhook_post_auth
        .update(
            id,
            source_code,
            domain_pattern,
            auth_key_suffix,
            ppk,
            &allowed_ips,
        )
        .await
        .map_err(map_store_err)?;
    Ok(Json(serde_json::json!({
        "item": item,
        "source": "admin_api",
    })))
}

pub async fn patch_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<StatusBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let status = body.status.trim();
    validate_status(status)?;
    let item = state
        .webhook_post_auth
        .set_status(id, status)
        .await
        .map_err(map_store_err)?;
    Ok(Json(serde_json::json!({
        "item": item,
        "source": "admin_api",
    })))
}

fn validate_fields(
    source_code: &str,
    domain_pattern: &str,
    auth_key_suffix: &str,
    allowed_ips: &str,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    if source_code.is_empty() {
        return Err(err_status(StatusCode::BAD_REQUEST, "source_code required"));
    }
    if auth_key_suffix.is_empty() {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            "auth_key_suffix required",
        ));
    }
    validate_domain_pattern(domain_pattern)?;
    if allowed_ips.is_empty() {
        return Err(err_status(StatusCode::BAD_REQUEST, "allowed_ips required"));
    }
    Ok(())
}

fn validate_domain_pattern(pattern: &str) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    let p = pattern.trim().to_ascii_lowercase();
    if p.is_empty() {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            "domain_pattern required",
        ));
    }
    if p == "*" {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            "bare * domain_pattern rejected",
        ));
    }
    if let Some(suffix) = p.strip_prefix("*.") {
        if suffix.is_empty() || suffix.contains('*') {
            return Err(err_status(
                StatusCode::BAD_REQUEST,
                "invalid wildcard domain_pattern",
            ));
        }
        return Ok(());
    }
    if p.contains('*') {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            "invalid domain_pattern",
        ));
    }
    Ok(())
}

fn validate_status(status: &str) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    match status {
        "active" | "inactive" => Ok(()),
        _ => Err(err_status(
            StatusCode::BAD_REQUEST,
            "status must be active or inactive",
        )),
    }
}

fn normalize_ips(raw: &str) -> Result<String, (StatusCode, Json<ErrorBody>)> {
    let mut out = Vec::new();
    for tok in raw.split(&[',', '|'][..]) {
        let t = tok.trim();
        if t.is_empty() {
            continue;
        }
        if t.parse::<IpAddr>().is_err() {
            return Err(err_status(
                StatusCode::BAD_REQUEST,
                format!("invalid IP in allowed_ips: {t}"),
            ));
        }
        out.push(t.to_string());
    }
    if out.is_empty() {
        return Err(err_status(StatusCode::BAD_REQUEST, "allowed_ips required"));
    }
    Ok(out.join(","))
}

fn map_store_err(e: String) -> (StatusCode, Json<ErrorBody>) {
    if e.contains("not found") {
        err_status(StatusCode::NOT_FOUND, e)
    } else if e.contains("already") {
        err_status(StatusCode::CONFLICT, e)
    } else {
        err_status(StatusCode::BAD_REQUEST, e)
    }
}
