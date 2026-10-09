//! Corp market pair + pending trade pool.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, AppState};
use crate::corp;
use crate::market_store::SubmitMarket;
use crate::models::ErrorBody;

#[derive(Debug, Deserialize)]
pub struct SubmitBody {
    pub game_id: i64,
    pub base_game_coin_id: i64,
    pub quote_game_coin_id: i64,
    pub market_name: String,
    #[serde(default = "default_funding")]
    pub funding_source: String,
    pub pool_depth: f64,
    pub initial_price: f64,
    pub base_amount: f64,
    pub quote_amount: f64,
}

fn default_funding() -> String {
    "gamecoin_lockup".into()
}

#[derive(Debug, Deserialize)]
pub struct PoolBody {
    pub pool_depth: f64,
    pub initial_price: f64,
    pub base_amount: f64,
    pub quote_amount: f64,
}

pub async fn list_markets(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    Ok(Json(serde_json::json!({
        "markets": state.markets.list(ident.corporate_user_id).await,
        "source": "client_center",
    })))
}

pub async fn get_market(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let Some(market) = state.markets.get(ident.corporate_user_id, id).await else {
        return Err(err_status(StatusCode::NOT_FOUND, "market pair not found"));
    };
    Ok(Json(serde_json::json!({
        "market": market,
        "source": "client_center",
    })))
}

pub async fn submit_market(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SubmitBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    if body.market_name.trim().is_empty() {
        return Err(err_status(StatusCode::BAD_REQUEST, "market_name required"));
    }
    let market = state
        .markets
        .submit(
            ident.corporate_user_id,
            SubmitMarket {
                game_id: body.game_id,
                base_game_coin_id: body.base_game_coin_id,
                quote_game_coin_id: body.quote_game_coin_id,
                market_name: body.market_name.trim().into(),
                funding_source: body.funding_source,
                pool_depth: body.pool_depth,
                initial_price: body.initial_price,
                base_amount: body.base_amount,
                quote_amount: body.quote_amount,
            },
        )
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "market": market,
            "source": "client_center",
        })),
    ))
}

pub async fn create_pool(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<PoolBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let market = state
        .markets
        .ensure_pool(
            ident.corporate_user_id,
            id,
            body.pool_depth,
            body.initial_price,
            body.base_amount,
            body.quote_amount,
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
        "market": market,
        "source": "client_center",
    })))
}
