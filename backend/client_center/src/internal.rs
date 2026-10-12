//! Internal settle from Webhook Server (`X-Internal-Key`).

use axum::extract::{Query, State};
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

#[derive(Debug, Deserialize)]
pub struct MoneySettleRequest {
    pub event_id: String,
    pub request_id: String,
    pub partner_txn_id: String,
    pub transaction_type: String,
    /// `success` | `failed` | `cancelled` (also accepts `completed` / `paid`).
    pub status: String,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub game_coin: String,
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

#[derive(Debug, Deserialize)]
pub struct MoneyTxnLookupQuery {
    #[serde(default)]
    pub request_id: String,
    #[serde(default)]
    pub partner_txn_id: String,
}

/// Confirm a deposit/withdrawal txn already exists before webhook settle.
pub async fn money_txn_lookup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<MoneyTxnLookupQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal(&state, &headers)?;
    if q.request_id.trim().is_empty() && q.partner_txn_id.trim().is_empty() {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            "request_id or partner_txn_id required",
        ));
    }
    let txn = state
        .wallets
        .find_by_partner_ref(q.request_id.trim(), q.partner_txn_id.trim())
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| {
            err_status(
                StatusCode::NOT_FOUND,
                "deposit/withdrawal txn not found",
            )
        })?;
    if txn.transaction_type != "deposit" && txn.transaction_type != "withdrawal" {
        return Err(err_status(
            StatusCode::NOT_FOUND,
            "deposit/withdrawal txn not found",
        ));
    }
    Ok(Json(serde_json::json!({
        "exists": true,
        "txn": txn,
        "source": "client_center",
    })))
}

pub async fn money_settle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<MoneySettleRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal(&state, &headers)?;
    if req.event_id.trim().is_empty() {
        return Err(err_status(StatusCode::BAD_REQUEST, "event_id required"));
    }
    if req.request_id.trim().is_empty() || req.partner_txn_id.trim().is_empty() {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            "request_id and partner_txn_id required",
        ));
    }
    let txn_type = req.transaction_type.trim();
    if txn_type != "deposit" && txn_type != "withdrawal" {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            "transaction_type must be deposit or withdrawal",
        ));
    }
    let txn = state
        .wallets
        .apply_settle(
            req.request_id.trim(),
            req.partner_txn_id.trim(),
            txn_type,
            req.status.trim(),
            req.amount,
            req.game_coin.trim(),
            req.event_id.trim(),
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
        "settled": true,
        "txn": txn,
        "source": "client_center",
    })))
}
