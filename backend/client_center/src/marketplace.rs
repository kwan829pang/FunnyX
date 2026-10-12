//! Session Marketplace APIs (C7): eligibility, items, deals place/take/cancel.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;

use crate::api::{err_status, AppState};
use crate::game_accounts::require_end_user;
use crate::marketplace_store::{PlaceDealRequest, TakeDealRequest};
use crate::models::ErrorBody;
use crate::query::marketplace as q;

#[derive(Debug, Deserialize)]
pub struct ItemsQuery {
    pub game_account_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListDealsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn eligibility(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let uid = require_end_user(&state, &headers).await?;
    let elig = state
        .marketplace
        .eligibility(uid)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({
        "eligibility": elig,
        "source": "client_center",
    })))
}

pub async fn list_items(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(qparams): Query<ItemsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let uid = require_end_user(&state, &headers).await?;
    let pool = state
        .pool
        .as_ref()
        .ok_or_else(|| err_status(StatusCode::SERVICE_UNAVAILABLE, "POSTGRES_URL required"))?;
    let ga = sqlx::query(q::OWNED_GAME_ACCOUNT)
        .bind(qparams.game_account_id)
        .bind(uid)
        .fetch_optional(pool)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "game account not found"))?;
    let corporate_user_id: i64 = ga.get("corporate_user_id");
    let partner_ga: String = ga.get("game_account_id");
    let partner_game_id: String = ga.get("partner_game_id");
    let item_ep: Option<String> = sqlx::query_scalar(q::ITEM_LIST_ENDPOINT)
        .bind(corporate_user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let items = state
        .partner
        .list_items(
            item_ep.as_deref(),
            &partner_ga,
            Some(partner_game_id.as_str()).filter(|s| !s.is_empty()),
        )
        .await
        .map_err(|(st, body)| {
            let mapped = StatusCode::from_u16(st.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            (mapped, Json(body))
        })?;
    Ok(Json(json!({
        "game_account_id": qparams.game_account_id,
        "items": items,
        "source": "client_center",
    })))
}

pub async fn list_deals(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(qparams): Query<ListDealsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_end_user(&state, &headers).await?;
    let deals = state
        .marketplace
        .list_open(qparams.limit)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({
        "deals": deals,
        "listing_fee_plt": crate::marketplace_store::LISTING_FEE_PLT,
        "source": "client_center",
    })))
}

pub async fn get_deal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_end_user(&state, &headers).await?;
    let deal = state
        .marketplace
        .get(id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "deal not found"))?;
    Ok(Json(deal))
}

pub async fn place_deal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<PlaceDealRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let uid = require_end_user(&state, &headers).await?;
    let deal = state
        .marketplace
        .place(uid, req, &state.partner)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok((StatusCode::CREATED, Json(deal)))
}

pub async fn take_deal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(req): Json<TakeDealRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let uid = require_end_user(&state, &headers).await?;
    let deal = state
        .marketplace
        .take(
            uid,
            id,
            req,
            &state.partner,
            &state.config.default_partner_id,
        )
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(deal))
}

pub async fn cancel_deal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let uid = require_end_user(&state, &headers).await?;
    let deal = state
        .marketplace
        .cancel(uid, id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(deal))
}
