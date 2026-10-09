//! Admin monitor / suspend Corp e-shop products.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, extract_bearer, require_admin_session, AppState};
use crate::models::ErrorBody;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub corporate_user_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct StatusBody {
    pub status: String,
}

async fn require_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<i64, (StatusCode, Json<ErrorBody>)> {
    let token = extract_bearer(headers)
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "Authorization Bearer required"))?;
    let (admin, _) = require_admin_session(state, &token).await?;
    Ok(admin.admin_user_id)
}

pub async fn list_corp_products(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    if let Some(s) = q.status.as_deref() {
        match s {
            "draft" | "active" | "inactive" | "archived" => {}
            _ => return Err(err_status(StatusCode::BAD_REQUEST, "invalid status filter")),
        }
    }
    let products = state
        .shop
        .list(q.status.as_deref(), q.corporate_user_id)
        .await;
    Ok(Json(serde_json::json!({
        "products": products,
        "source": "admin_api",
    })))
}

pub async fn get_corp_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let Some(product) = state.shop.get(id).await else {
        return Err(err_status(StatusCode::NOT_FOUND, "product not found"));
    };
    Ok(Json(serde_json::json!({
        "product": product,
        "source": "admin_api",
    })))
}

pub async fn patch_corp_product_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<StatusBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin_id = require_admin(&state, &headers).await?;
    let product = state
        .shop
        .set_status(id, body.status.trim(), admin_id)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                err_status(StatusCode::NOT_FOUND, e)
            } else {
                err_status(StatusCode::BAD_REQUEST, e)
            }
        })?;
    Ok(Json(serde_json::json!({
        "product": product,
        "source": "admin_api",
    })))
}
