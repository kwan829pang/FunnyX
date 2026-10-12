//! Session exchange: markets + orders via private Core Engine.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::api::{err_status, AppState};
use crate::engine_client::PlaceOrderReq;
use crate::game_accounts::require_end_user;
use crate::models::ErrorBody;

#[derive(Clone, Default)]
pub struct ExchangeOrderStore {
    next_id: Arc<AtomicI64>,
    by_id: Arc<RwLock<HashMap<i64, SessionOrder>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOrder {
    pub id: i64,
    pub end_user_id: i64,
    pub game_account_id: i64,
    pub market_pair_id: Option<i64>,
    pub symbol: String,
    pub engine_order_id: String,
    pub side: String,
    pub order_type: String,
    pub price: u64,
    pub quantity: u64,
    pub remaining: u64,
    pub status: String,
    pub created_at: i64,
}

impl ExchangeOrderStore {
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicI64::new(1)),
            by_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PlaceBody {
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    #[serde(default)]
    pub price: u64,
    pub quantity: u64,
    pub game_account_id: i64,
    #[serde(default)]
    pub market_pair_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BookQuery {
    #[serde(default = "default_levels")]
    pub levels: u32,
}

fn default_levels() -> u32 {
    10
}

pub async fn list_markets(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_end_user(&state, &headers).await?;
    let markets = load_active_markets(&state).await?;
    Ok(Json(serde_json::json!({
        "markets": markets,
        "source": "client_center",
    })))
}

pub async fn get_market_book(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Query(q): Query<BookQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_end_user(&state, &headers).await?;
    if !state.engine.configured() {
        return Err(err_status(
            StatusCode::SERVICE_UNAVAILABLE,
            "CORE_ENGINE_URL not configured",
        ));
    }
    let markets = load_active_markets(&state).await?;
    let m = markets
        .into_iter()
        .find(|m| m.get("id").and_then(|v| v.as_i64()) == Some(id))
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "market not found"))?;
    let symbol = m
        .get("symbol")
        .and_then(|v| v.as_str())
        .ok_or_else(|| err_status(StatusCode::BAD_REQUEST, "market missing symbol"))?;
    let book = state
        .engine
        .book(symbol, q.levels.max(1))
        .await
        .map_err(|e| err_status(StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(serde_json::json!({
        "market_id": id,
        "symbol": symbol,
        "book": book,
        "source": "client_center",
    })))
}

pub async fn list_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    let map = state.exchange_orders.by_id.read().await;
    let mut items: Vec<_> = map
        .values()
        .filter(|o| o.end_user_id == end_user_id)
        .cloned()
        .collect();
    items.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(Json(serde_json::json!({
        "orders": items,
        "source": "client_center",
    })))
}

pub async fn get_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    let map = state.exchange_orders.by_id.read().await;
    let order = map
        .get(&id)
        .filter(|o| o.end_user_id == end_user_id)
        .cloned()
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "order not found"))?;
    Ok(Json(serde_json::json!({
        "order": order,
        "source": "client_center",
    })))
}

pub async fn place_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PlaceBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    if !state.engine.configured() {
        return Err(err_status(
            StatusCode::SERVICE_UNAVAILABLE,
            "CORE_ENGINE_URL not configured",
        ));
    }
    let side = body.side.trim().to_ascii_lowercase();
    if side != "buy" && side != "sell" {
        return Err(err_status(StatusCode::BAD_REQUEST, "side must be buy|sell"));
    }
    let order_type = normalize_order_type(&body.order_type)?;
    if body.quantity == 0 {
        return Err(err_status(StatusCode::BAD_REQUEST, "quantity must be > 0"));
    }
    if order_type == "price" && body.price == 0 {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            "price required for price orders",
        ));
    }
    let acct = state
        .wallets
        .get_owned_account(end_user_id, body.game_account_id)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "game_account_id not found"))?;

    let symbol = body.symbol.trim().to_string();
    if symbol.is_empty() {
        return Err(err_status(StatusCode::BAD_REQUEST, "symbol required"));
    }

    let result = state
        .engine
        .place(&PlaceOrderReq {
            symbol: symbol.clone(),
            side: side.clone(),
            order_type: order_type.clone(),
            price: body.price,
            quantity: body.quantity,
            account_id: format!("eu_{end_user_id}_ga_{}", acct.row_id),
            end_user_id,
        })
        .await
        .map_err(|e| err_status(StatusCode::BAD_GATEWAY, e))?;

    let id = state.exchange_orders.next_id.fetch_add(1, Ordering::Relaxed);
    let now = crate::db::now_ms();
    let order = SessionOrder {
        id,
        end_user_id,
        game_account_id: body.game_account_id,
        market_pair_id: body.market_pair_id,
        symbol,
        engine_order_id: result.order.order_id.clone(),
        side,
        order_type,
        price: result.order.price,
        quantity: result.order.original_quantity,
        remaining: result.order.remaining_quantity,
        status: result.order.status.clone(),
        created_at: now,
    };
    state
        .exchange_orders
        .by_id
        .write()
        .await
        .insert(id, order.clone());

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "order": order,
            "trades": result.trades,
            "source": "client_center",
        })),
    ))
}

pub async fn cancel_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    if !state.engine.configured() {
        return Err(err_status(
            StatusCode::SERVICE_UNAVAILABLE,
            "CORE_ENGINE_URL not configured",
        ));
    }
    let existing = {
        let map = state.exchange_orders.by_id.read().await;
        map.get(&id)
            .filter(|o| o.end_user_id == end_user_id)
            .cloned()
            .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "order not found"))?
    };
    let view = state
        .engine
        .cancel(&existing.symbol, &existing.engine_order_id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_GATEWAY, e))?;
    let mut map = state.exchange_orders.by_id.write().await;
    if let Some(o) = map.get_mut(&id) {
        o.status = view.status.clone();
        o.remaining = view.remaining_quantity;
    }
    let order = map.get(&id).cloned().unwrap_or(existing);
    Ok(Json(serde_json::json!({
        "order": order,
        "source": "client_center",
    })))
}

fn normalize_order_type(raw: &str) -> Result<String, (StatusCode, Json<ErrorBody>)> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "market" | "1" | "01" => Ok("market".into()),
        "price" | "2" | "02" => Ok("price".into()),
        _ => Err(err_status(
            StatusCode::BAD_REQUEST,
            "order_type must be market|price",
        )),
    }
}

async fn load_active_markets(
    state: &AppState,
) -> Result<Vec<serde_json::Value>, (StatusCode, Json<ErrorBody>)> {
    let Some(pool) = &state.pool else {
        return Ok(vec![serde_json::json!({
            "id": 1,
            "symbol": "PLT/USDT",
            "market_name": "PLT/USDT",
            "status": "active",
            "demo": true,
        })]);
    };
    let rows = sqlx::query(
        "SELECT mp.id, mp.market_name, mp.status, \
                bc.code AS base_code, qc.code AS quote_code \
         FROM fx_market.market_pairs mp \
         JOIN fx_game.game_coins bc ON bc.id = mp.base_game_coin_id \
         JOIN fx_game.game_coins qc ON qc.id = mp.quote_game_coin_id \
         WHERE mp.status = 'active' \
         ORDER BY mp.id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    use sqlx::Row;
    Ok(rows
        .into_iter()
        .map(|r| {
            let base: String = r.get("base_code");
            let quote: String = r.get("quote_code");
            let name: String = r.get("market_name");
            let symbol = format!("{base}/{quote}");
            serde_json::json!({
                "id": r.get::<i64, _>("id"),
                "market_name": name,
                "symbol": symbol,
                "status": r.get::<String, _>("status"),
                "base_game_coin": base,
                "quote_game_coin": quote,
            })
        })
        .collect())
}
