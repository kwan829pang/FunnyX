//! Session APIs: buyable Company Basic Tokens (separate from Corp e-shop products).

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, AppState};
use crate::game_accounts::require_end_user;
use crate::models::ErrorBody;

#[derive(Debug, Deserialize)]
pub struct CreateOrderBody {
    pub game_account_id: i64,
    pub coin_amount: f64,
    #[serde(default)]
    pub partner_order_no: Option<String>,
}

pub async fn list_buyable(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_end_user(&state, &headers).await?;
    Ok(Json(serde_json::json!({
        "tokens": state.cbt.list_buyable().await,
        "buy_fee_rate": 0.001,
        "source": "client_center",
    })))
}

pub async fn get_buyable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_end_user(&state, &headers).await?;
    let Some(token) = state.cbt.get(id).await else {
        return Err(err_status(StatusCode::NOT_FOUND, "token not found"));
    };
    if !token.buyable || token.status != "approved" {
        return Err(err_status(StatusCode::NOT_FOUND, "token not buyable"));
    }
    Ok(Json(serde_json::json!({
        "token": token,
        "source": "client_center",
    })))
}

pub async fn create_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<CreateOrderBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    let bindings = state.bindings.list_for_user(end_user_id).await;
    let _binding = bindings
        .iter()
        .find(|b| b.id == body.game_account_id && b.status == "active")
        .ok_or_else(|| {
            err_status(
                StatusCode::BAD_REQUEST,
                "game_account_id must be an active binding for this user",
            )
        })?;
    let order = state
        .cbt
        .create_order(
            id,
            end_user_id,
            body.game_account_id,
            body.coin_amount,
            body.partner_order_no.as_deref(),
        )
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "order": order,
            "source": "client_center",
        })),
    ))
}

pub async fn list_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    Ok(Json(serde_json::json!({
        "orders": state.cbt.list_orders_for_user(end_user_id).await,
        "source": "client_center",
    })))
}

pub async fn get_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    let Some(order) = state.cbt.get_order(order_id).await else {
        return Err(err_status(StatusCode::NOT_FOUND, "order not found"));
    };
    if order.end_user_id != end_user_id {
        return Err(err_status(StatusCode::NOT_FOUND, "order not found"));
    }
    Ok(Json(serde_json::json!({
        "order": order,
        "source": "client_center",
    })))
}

pub async fn cancel_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    let order = state
        .cbt
        .cancel_order(end_user_id, order_id)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                err_status(StatusCode::NOT_FOUND, e)
            } else {
                err_status(StatusCode::BAD_REQUEST, e)
            }
        })?;
    Ok(Json(serde_json::json!({
        "order": order,
        "source": "client_center",
    })))
}
