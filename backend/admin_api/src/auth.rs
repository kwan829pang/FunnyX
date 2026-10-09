//! Admin login / logout / me — credentials verified here; tokens from Session Token Server.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, extract_bearer, map_sts_err, require_admin_session, AppState};
use crate::models::{AdminMeResponse, AdminSessionResponse, ErrorBody};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

pub async fn admin_login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin = state
        .admins
        .authenticate(&req.username, &req.password)
        .await
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "invalid username or password"))?;

    let session = state
        .sts
        .issue_admin_session(admin.admin_user_id, &admin.username)
        .await
        .map_err(map_sts_err)?;

    Ok((
        StatusCode::CREATED,
        Json(AdminSessionResponse {
            access_token: session.access_token,
            refresh_token: session.refresh_token,
            token_type: session.token_type,
            expires_in: session.expires_in,
            expires_at_ms: session.expires_at_ms,
            admin_user_id: admin.admin_user_id,
            username: admin.username,
            role: admin.role,
            scope: session.scope,
            actor_type: session.actor_type,
            grant_type: "admin_login".into(),
            source: "admin_api".into(),
        }),
    ))
}

pub async fn admin_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LogoutRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let access = req
        .access_token
        .filter(|s| !s.is_empty())
        .or_else(|| extract_bearer(&headers));
    let refresh = req.refresh_token.filter(|s| !s.is_empty());
    if access.is_none() && refresh.is_none() {
        return Err(err_status(
            StatusCode::UNAUTHORIZED,
            "access_token / Bearer or refresh_token required",
        ));
    }
    let body = state
        .sts
        .revoke(access.as_deref(), refresh.as_deref())
        .await
        .map_err(map_sts_err)?;
    Ok(Json(serde_json::json!({
        "revoked": body.get("revoked").and_then(|v| v.as_bool()).unwrap_or(true),
        "source": "admin_api",
    })))
}

pub async fn admin_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let token = extract_bearer(&headers).ok_or_else(|| {
        err_status(StatusCode::UNAUTHORIZED, "Authorization Bearer required")
    })?;
    let (admin, validated) = require_admin_session(&state, &token).await?;
    Ok(Json(AdminMeResponse {
        admin_user_id: admin.admin_user_id,
        username: admin.username,
        role: admin.role,
        status: admin.status,
        scope: validated.scope,
        source: "admin_api".into(),
    }))
}
