//! Admin approve / reject Company Basic Tokens.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, extract_bearer, require_admin_session, AppState};
use crate::models::ErrorBody;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub status: Option<String>,
}

async fn require_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<i64, (StatusCode, Json<ErrorBody>)> {
    let token = extract_bearer(headers)
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "Authorization Bearer required"))?;
    let (admin, _) = require_admin_session(state, &token).await?;
    Ok(admin.admin_user_id)
}

pub async fn list_tokens(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    if let Some(s) = q.status.as_deref() {
        match s {
            "submitted" | "pending" | "approved" | "rejected" => {}
            _ => return Err(err_status(StatusCode::BAD_REQUEST, "invalid status filter")),
        }
    }
    Ok(Json(serde_json::json!({
        "tokens": state.cbt.list(q.status.as_deref()).await,
        "source": "admin_api",
    })))
}

pub async fn get_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let Some(token) = state.cbt.get(id).await else {
        return Err(err_status(StatusCode::NOT_FOUND, "token not found"));
    };
    Ok(Json(serde_json::json!({
        "token": token,
        "source": "admin_api",
    })))
}

pub async fn approve(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin_id = require_admin(&state, &headers).await?;
    let token = state
        .cbt
        .approve(id, admin_id)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                err_status(StatusCode::NOT_FOUND, e)
            } else {
                err_status(StatusCode::BAD_REQUEST, e)
            }
        })?;
    Ok(Json(serde_json::json!({
        "token": token,
        "source": "admin_api",
    })))
}

pub async fn reject(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin_id = require_admin(&state, &headers).await?;
    let token = state
        .cbt
        .reject(id, admin_id)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                err_status(StatusCode::NOT_FOUND, e)
            } else {
                err_status(StatusCode::BAD_REQUEST, e)
            }
        })?;
    Ok(Json(serde_json::json!({
        "token": token,
        "source": "admin_api",
    })))
}
