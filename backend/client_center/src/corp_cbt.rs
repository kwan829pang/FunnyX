//! Corp MasterSigned Company Basic Token submit / list / update.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, AppState};
use crate::corp;
use crate::models::ErrorBody;

#[derive(Debug, Deserialize)]
pub struct CreateBody {
    pub game_id: i64,
    pub game_coin_id: i64,
    pub token_code: String,
    pub token_name: String,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBody {
    #[serde(default)]
    pub game_id: Option<i64>,
    #[serde(default)]
    pub game_coin_id: Option<i64>,
    #[serde(default)]
    pub token_code: Option<String>,
    #[serde(default)]
    pub token_name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

pub async fn list_tokens(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    Ok(Json(serde_json::json!({
        "tokens": state.cbt.list_for_corp(ident.corporate_user_id).await,
        "source": "client_center",
    })))
}

pub async fn get_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let Some(token) = state.cbt.get_owned(ident.corporate_user_id, id).await else {
        return Err(err_status(StatusCode::NOT_FOUND, "token not found"));
    };
    Ok(Json(serde_json::json!({
        "token": token,
        "source": "client_center",
    })))
}

pub async fn create_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let status = body.status.as_deref().unwrap_or("submitted");
    let token = state
        .cbt
        .create(
            ident.corporate_user_id,
            body.game_id,
            body.game_coin_id,
            &body.token_code,
            &body.token_name,
            status,
        )
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "token": token,
            "source": "client_center",
        })),
    ))
}

pub async fn update_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<UpdateBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let token = state
        .cbt
        .update_owned(
            ident.corporate_user_id,
            id,
            body.game_id,
            body.game_coin_id,
            body.token_code.as_deref(),
            body.token_name.as_deref(),
            body.status.as_deref(),
        )
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
        "source": "client_center",
    })))
}

pub async fn list_buy_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let orders = state
        .cbt
        .list_buy_orders_for_token(ident.corporate_user_id, id)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                err_status(StatusCode::NOT_FOUND, e)
            } else {
                err_status(StatusCode::BAD_REQUEST, e)
            }
        })?;
    Ok(Json(serde_json::json!({
        "orders": orders,
        "source": "client_center",
    })))
}

pub async fn list_fees(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let fees = state
        .cbt
        .list_fees_for_token(ident.corporate_user_id, id)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                err_status(StatusCode::NOT_FOUND, e)
            } else {
                err_status(StatusCode::BAD_REQUEST, e)
            }
        })?;
    Ok(Json(serde_json::json!({
        "fees": fees,
        "source": "client_center",
    })))
}
