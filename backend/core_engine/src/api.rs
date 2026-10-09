use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::engine::CoreEngine;
use crate::types::{AdminCreatePairInput, OrderType, Side, TradingPair};

#[derive(Clone)]
pub struct AppState {
    pub engine: CoreEngine,
    pub admin_api_key: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/engine", get(engine_info))
        .route("/v1/engine/pairs", get(list_pairs).post(add_pair))
        .route("/v1/engine/pairs/{symbol}", delete(remove_pair))
        .route("/v1/engine/pools", get(list_pools))
        .route("/v1/engine/pools/{symbol}", get(get_pool))
        // Admin: create pair + init market pool (requires X-Admin-Key)
        .route("/v1/admin/pairs", post(admin_create_pair))
        // Maintenance: persist pools/orders locally, stop/start engine memory
        .route("/v1/admin/maintenance", get(maintenance_status))
        .route("/v1/admin/maintenance/save", post(maintenance_save))
        .route("/v1/admin/maintenance/stop", post(maintenance_stop))
        .route("/v1/admin/maintenance/start", post(maintenance_start))
        .route("/v1/orders", post(place_order))
        .route(
            "/v1/orders/{symbol}/{order_id}",
            get(get_order).delete(cancel_order),
        )
        .route("/v1/books/{symbol}", get(book_snapshot))
        .route("/v1/trades/{symbol}", get(recent_trades))
        .route("/v1/notices", get(recent_notices))
        .with_state(state)
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    engine_id: String,
    quote_asset: String,
    timestamp_ms: i64,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        service: "funnyx-core-engine".into(),
        engine_id: state.engine.engine_id.clone(),
        quote_asset: state.engine.quote_asset.clone(),
        timestamp_ms: Utc::now().timestamp_millis(),
    })
}

async fn engine_info(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.engine.info().await)
}

async fn list_pairs(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.engine.list_pairs().await)
}

#[derive(Deserialize)]
struct AddPairRequest {
    symbol: String,
    #[serde(default)]
    market_id: Option<String>,
    #[serde(default)]
    tick_size: Option<u64>,
    #[serde(default)]
    lot_size: Option<u64>,
}

async fn add_pair(
    State(state): State<AppState>,
    Json(req): Json<AddPairRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let market_id = req
        .market_id
        .unwrap_or_else(|| format!("mkt_{}", req.symbol.replace('/', "_")));
    let mut pair = TradingPair::parse(&req.symbol, market_id).map_err(map_err)?;
    if let Some(t) = req.tick_size {
        pair.tick_size = t;
    }
    if let Some(l) = req.lot_size {
        pair.lot_size = l;
    }
    let pair = state.engine.add_pair(pair).await.map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(pair)))
}

#[derive(Deserialize)]
struct AdminCreatePairRequest {
    symbol: String,
    #[serde(default)]
    market_id: Option<String>,
    #[serde(default)]
    tick_size: Option<u64>,
    #[serde(default)]
    lot_size: Option<u64>,
    pool_depth: f64,
    initial_price: f64,
    base_amount: f64,
    quote_amount: f64,
    #[serde(default = "default_funding")]
    funding_source: String,
    admin_id: String,
    #[serde(default)]
    corporate_user_id: Option<i64>,
    /// When true, rest a sell from pool base at initial_price on the book.
    #[serde(default)]
    seed_book: bool,
}

fn default_funding() -> String {
    "gamecoin_lockup".into()
}

async fn admin_create_pair(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AdminCreatePairRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_admin(&headers, &state.admin_api_key)?;
    let market_id = req
        .market_id
        .unwrap_or_else(|| format!("mkt_{}", req.symbol.replace('/', "_")));
    let result = state
        .engine
        .admin_create_pair_with_pool(AdminCreatePairInput {
            symbol: req.symbol,
            market_id,
            tick_size: req.tick_size.unwrap_or(1),
            lot_size: req.lot_size.unwrap_or(1),
            pool_depth: req.pool_depth,
            initial_price: req.initial_price,
            base_amount: req.base_amount,
            quote_amount: req.quote_amount,
            funding_source: req.funding_source,
            admin_id: req.admin_id,
            corporate_user_id: req.corporate_user_id,
            seed_book: req.seed_book,
        })
        .await
        .map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(result)))
}

async fn list_pools(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.engine.list_pools().await)
}

async fn get_pool(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let pool = state.engine.get_pool(&symbol).await.map_err(map_err)?;
    Ok(Json(pool))
}

async fn remove_pair(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    state.engine.remove_pair(&symbol).await.map_err(map_err)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct PlaceOrderRequest {
    symbol: String,
    side: Side,
    /// `market` | `price` (or numeric 1 / 2)
    order_type: OrderType,
    #[serde(default)]
    price: u64,
    quantity: u64,
    account_id: String,
    #[serde(default)]
    end_user_id: Option<i64>,
    #[serde(default)]
    corporate_user_id: Option<i64>,
}

async fn place_order(
    State(state): State<AppState>,
    Json(req): Json<PlaceOrderRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let result = state
        .engine
        .place_order(
            &req.symbol,
            req.side,
            req.order_type,
            req.price,
            req.quantity,
            req.account_id,
            req.end_user_id,
            req.corporate_user_id,
        )
        .await
        .map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(result)))
}

async fn get_order(
    State(state): State<AppState>,
    Path((symbol, order_id)): Path<(String, u64)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let order = state
        .engine
        .get_order(&symbol, order_id)
        .await
        .map_err(map_err)?;
    Ok(Json(order))
}

async fn cancel_order(
    State(state): State<AppState>,
    Path((symbol, order_id)): Path<(String, u64)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let order = state
        .engine
        .cancel_order(&symbol, order_id)
        .await
        .map_err(map_err)?;
    Ok(Json(order))
}

#[derive(Deserialize)]
struct DepthQuery {
    #[serde(default = "default_levels")]
    levels: usize,
}

fn default_levels() -> usize {
    10
}

async fn book_snapshot(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<DepthQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let snap = state
        .engine
        .book_snapshot(&symbol, q.levels)
        .await
        .map_err(map_err)?;
    Ok(Json(snap))
}

#[derive(Deserialize)]
struct LimitQuery {
    #[serde(default = "default_trade_limit")]
    limit: usize,
}

fn default_trade_limit() -> usize {
    50
}

async fn recent_trades(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<LimitQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let trades = state
        .engine
        .recent_trades(&symbol, q.limit)
        .await
        .map_err(map_err)?;
    Ok(Json(trades))
}

async fn recent_notices(
    State(state): State<AppState>,
    Query(q): Query<LimitQuery>,
) -> impl IntoResponse {
    Json(state.engine.notices().recent(q.limit).await)
}

async fn maintenance_status(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.engine.maintenance_info().await)
}

async fn maintenance_save(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_admin(&headers, &state.admin_api_key)?;
    let path = state.engine.save_snapshot().await.map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "snapshot_path": path.display().to_string(),
        "info": state.engine.maintenance_info().await,
    })))
}

async fn maintenance_stop(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_admin(&headers, &state.admin_api_key)?;
    let info = state.engine.maintenance_stop().await.map_err(map_err)?;
    Ok(Json(info))
}

async fn maintenance_start(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_admin(&headers, &state.admin_api_key)?;
    let info = state.engine.maintenance_start().await.map_err(map_err)?;
    Ok(Json(info))
}

fn require_admin(
    headers: &HeaderMap,
    expected: &str,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    let key = headers
        .get("x-admin-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if key.is_empty() || key != expected {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorBody {
                error: "X-Admin-Key required".into(),
            }),
        ));
    }
    Ok(())
}

fn map_err(e: crate::types::EngineError) -> (StatusCode, Json<ErrorBody>) {
    let status = match &e {
        crate::types::EngineError::PairNotFound(_)
        | crate::types::EngineError::OrderNotFound(_)
        | crate::types::EngineError::PoolNotFound(_) => StatusCode::NOT_FOUND,
        crate::types::EngineError::Maintenance(_) => StatusCode::SERVICE_UNAVAILABLE,
        crate::types::EngineError::Persist(_) => StatusCode::INTERNAL_SERVER_ERROR,
        crate::types::EngineError::PairLimit { .. }
        | crate::types::EngineError::PairExists(_)
        | crate::types::EngineError::QuoteMismatch { .. }
        | crate::types::EngineError::InvalidOrder(_)
        | crate::types::EngineError::InvalidPool(_)
        | crate::types::EngineError::CancelRejected(_) => StatusCode::BAD_REQUEST,
    };
    (status, Json(ErrorBody { error: e.to_string() }))
}
