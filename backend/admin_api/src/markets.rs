//! Admin market list / approve / reject.

use axum::extract::{Path, Query, State};
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
pub struct ListQuery {
    #[serde(default)]
    pub status: Option<String>,
}

pub async fn list_markets(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    Ok(Json(serde_json::json!({
        "markets": state.markets.list(q.status.as_deref()).await,
        "source": "admin_api",
    })))
}

pub async fn get_market(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let Some(market) = state.markets.get(id).await else {
        return Err(err_status(StatusCode::NOT_FOUND, "market not found"));
    };
    Ok(Json(serde_json::json!({
        "market": market,
        "source": "admin_api",
    })))
}

pub async fn approve(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin_id = require_admin(&state, &headers).await?;
    let market = state.markets.approve(id, admin_id).await.map_err(|e| {
        if e.contains("not found") {
            err_status(StatusCode::NOT_FOUND, e)
        } else {
            err_status(StatusCode::BAD_REQUEST, e)
        }
    })?;

    let pool = market
        .pool
        .as_ref()
        .ok_or_else(|| err_status(StatusCode::BAD_REQUEST, "market pool missing"))?;
    let engine_result = state
        .engine
        .create_pair(
            &market.market_name,
            &format!("mkt_{}", market.id),
            pool.pool_depth,
            pool.initial_price,
            pool.base_amount,
            pool.quote_amount,
            &market.funding_source,
            admin_id,
            market.corporate_user_id,
        )
        .await;

    let (engine_ok, engine_body) = match engine_result {
        Ok(v) => (true, Some(v)),
        Err((status, body)) => {
            tracing::warn!(
                market_id = id,
                status = %status,
                error = %body.error,
                "core engine activate failed after DB approve"
            );
            (false, Some(serde_json::json!({ "error": body.error })))
        }
    };

    Ok(Json(serde_json::json!({
        "market": market,
        "engine_activated": engine_ok,
        "engine": engine_body,
        "source": "admin_api",
    })))
}

pub async fn reject(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin_id = require_admin(&state, &headers).await?;
    let market = state.markets.reject(id, admin_id).await.map_err(|e| {
        if e.contains("not found") {
            err_status(StatusCode::NOT_FOUND, e)
        } else {
            err_status(StatusCode::BAD_REQUEST, e)
        }
    })?;
    Ok(Json(serde_json::json!({
        "market": market,
        "source": "admin_api",
    })))
}
