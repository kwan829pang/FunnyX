//! Internal settle from Webhook Server (`X-Internal-Key`).

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, AppState};
use crate::models::ErrorBody;

#[derive(Debug, Deserialize)]
pub struct ShopSettleRequest {
    pub shop_order_id: i64,
    pub partner_order_no: String,
    pub status: String,
    pub event_id: String,
    pub fiat_currency: String,
    #[serde(default)]
    pub fiat_paid: f64,
    #[serde(default)]
    pub seller_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CbtSettleRequest {
    pub corp_token_order_id: i64,
    pub partner_order_no: String,
    pub status: String,
    pub event_id: String,
    #[serde(default)]
    pub coin_amount: Option<f64>,
}

fn require_internal(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    let key = headers
        .get("x-internal-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if key != state.config.session_internal_api_key {
        return Err(err_status(StatusCode::UNAUTHORIZED, "X-Internal-Key required"));
    }
    Ok(())
}

pub async fn shop_settle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ShopSettleRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal(&state, &headers)?;
    if req.event_id.trim().is_empty() {
        return Err(err_status(StatusCode::BAD_REQUEST, "event_id required"));
    }
    let order = state
        .shop
        .apply_settle(
            req.shop_order_id,
            &req.partner_order_no,
            &req.status,
            &req.fiat_currency,
            req.fiat_paid,
            &req.event_id,
        )
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    let _ = req.seller_type;
    Ok(Json(serde_json::json!({
        "settled": true,
        "order": order,
        "source": "client_center",
    })))
}

pub async fn cbt_settle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CbtSettleRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal(&state, &headers)?;
    if req.event_id.trim().is_empty() {
        return Err(err_status(StatusCode::BAD_REQUEST, "event_id required"));
    }
    let result = state
        .cbt
        .apply_settle(
            req.corp_token_order_id,
            &req.partner_order_no,
            &req.status,
            req.coin_amount,
            &req.event_id,
        )
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::json!({
        "settled": true,
        "order": result.order,
        "corporate_user_id": result.corporate_user_id,
        "partner_callback_url": result.partner_callback_url,
        "source": "client_center",
    })))
}
