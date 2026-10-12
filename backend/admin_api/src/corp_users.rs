//! Admin corporate-user and game onboarding review.

use std::path::Path;

use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use tokio::fs;

use crate::api::{err_status, extract_bearer, require_admin_session, AppState};
use crate::models::ErrorBody;

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RejectBody {
    #[serde(default)]
    pub reason: String,
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

pub async fn list_corps(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<StatusQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let list = state
        .corp_users
        .list_corps(q.status.as_deref())
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::json!({
        "corporate_users": list,
        "source": "admin_api",
    })))
}

pub async fn get_corp(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let corp = state
        .corp_users
        .get_corp(id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "corporate user not found"))?;
    Ok(Json(corp))
}

pub async fn approve_corp(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin_id = require_admin(&state, &headers).await?;
    let corp = state
        .corp_users
        .approve_corp(id, admin_id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(corp))
}

pub async fn reject_corp(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<i64>,
    Json(body): Json<RejectBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let reason = if body.reason.trim().is_empty() {
        "rejected by admin"
    } else {
        body.reason.trim()
    };
    let corp = state
        .corp_users
        .reject_corp(id, reason)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(corp))
}

pub async fn list_games(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<StatusQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let list = state
        .corp_users
        .list_games(q.status.as_deref())
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::json!({
        "games": list,
        "source": "admin_api",
    })))
}

pub async fn get_game(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let game = state
        .corp_users
        .get_game(id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "game not found"))?;
    Ok(Json(game))
}

pub async fn approve_game(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let game = state
        .corp_users
        .approve_game(id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(game))
}

pub async fn reject_game(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<i64>,
    Json(body): Json<RejectBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let reason = if body.reason.trim().is_empty() {
        "rejected by admin"
    } else {
        body.reason.trim()
    };
    let game = state
        .corp_users
        .reject_game(id, reason)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(game))
}

pub async fn download_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<i64>,
) -> Result<Response, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let (doc, stored_path) = state
        .corp_users
        .get_doc(id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "document not found"))?;
    let path = Path::new(state.corp_users.upload_dir()).join(&stored_path);
    let bytes = fs::read(&path)
        .await
        .map_err(|e| err_status(StatusCode::NOT_FOUND, format!("file missing: {e}")))?;
    Ok((
        [
            (
                header::CONTENT_TYPE,
                doc.content_type.parse().unwrap_or_else(|_| {
                    header::HeaderValue::from_static("application/octet-stream")
                }),
            ),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", doc.file_name)
                    .parse()
                    .unwrap_or_else(|_| header::HeaderValue::from_static("attachment")),
            ),
        ],
        bytes,
    )
        .into_response())
}
