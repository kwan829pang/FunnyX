//! Admin Game Partner Game Coin CRUD.

use axum::extract::{Path, State};
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
pub struct CreateBody {
    pub code: String,
    pub name: String,
    #[serde(rename = "type", default = "default_type")]
    pub coin_type: String,
    #[serde(default = "default_asset")]
    pub asset_kind: String,
    #[serde(default)]
    pub is_platform_token: bool,
}

fn default_type() -> String {
    "game".into()
}

fn default_asset() -> String {
    "token".into()
}

#[derive(Debug, Deserialize)]
pub struct UpdateBody {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub asset_kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusBody {
    pub status: String,
}

pub async fn list_coins(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    Ok(Json(serde_json::json!({
        "game_coins": state.game_coins.list().await,
        "source": "admin_api",
    })))
}

pub async fn create_coin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin_id = require_admin(&state, &headers).await?;
    let coin = state
        .game_coins
        .create(
            &body.code,
            &body.name,
            &body.coin_type,
            &body.asset_kind,
            body.is_platform_token,
            admin_id,
        )
        .await
        .map_err(|e| {
            if e.contains("duplicate") || e.contains("unique") {
                err_status(StatusCode::CONFLICT, e)
            } else {
                err_status(StatusCode::BAD_REQUEST, e)
            }
        })?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "game_coin": coin, "source": "admin_api" })),
    ))
}

pub async fn update_coin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<UpdateBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin_id = require_admin(&state, &headers).await?;
    let coin = state
        .game_coins
        .update(
            id,
            body.name.as_deref(),
            body.asset_kind.as_deref(),
            admin_id,
        )
        .await
        .map_err(|e| {
            if e.contains("not found") {
                err_status(StatusCode::NOT_FOUND, e)
            } else {
                err_status(StatusCode::BAD_REQUEST, e)
            }
        })?;
    Ok(Json(serde_json::json!({ "game_coin": coin, "source": "admin_api" })))
}

pub async fn patch_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<StatusBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin_id = require_admin(&state, &headers).await?;
    let coin = state
        .game_coins
        .set_status(id, &body.status, admin_id)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                err_status(StatusCode::NOT_FOUND, e)
            } else {
                err_status(StatusCode::BAD_REQUEST, e)
            }
        })?;
    Ok(Json(serde_json::json!({ "game_coin": coin, "source": "admin_api" })))
}
