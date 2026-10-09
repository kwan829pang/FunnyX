//! Client Web e-shop: catalog, create pending order, list/status/cancel.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, AppState};
use crate::db::now_ms;
use crate::game_accounts::require_end_user;
use crate::models::ErrorBody;
use crate::shop_store::{order_ttl_ms, ShopOrderView};

#[derive(Debug, Deserialize)]
pub struct CreateShopOrderRequest {
    pub seller_type: String,
    #[serde(default)]
    pub package_id: Option<i64>,
    #[serde(default)]
    pub corp_product_id: Option<i64>,
    pub game_account_id: i64,
    #[serde(default)]
    pub return_url: Option<String>,
}

pub async fn shop_base_currency(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_end_user(&state, &headers).await?;
    let code = state.shop.base_fiat().await;
    Ok(Json(serde_json::json!({
        "base_fiat_currency": code,
        "source": "client_center",
    })))
}

pub async fn list_packages(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_end_user(&state, &headers).await?;
    let fiat = state.shop.base_fiat().await;
    Ok(Json(serde_json::json!({
        "fiat_currency": fiat,
        "packages": state.shop.list_packages().await,
        "source": "client_center",
    })))
}

pub async fn get_package(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_end_user(&state, &headers).await?;
    let pkg = state
        .shop
        .get_package(id)
        .await
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "package not found or inactive"))?;
    Ok(Json(pkg))
}

pub async fn list_corp_products(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_end_user(&state, &headers).await?;
    let fiat = state.shop.base_fiat().await;
    Ok(Json(serde_json::json!({
        "fiat_currency": fiat,
        "products": state.shop.list_corp_products().await,
        "source": "client_center",
    })))
}

pub async fn get_corp_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_end_user(&state, &headers).await?;
    let p = state
        .shop
        .get_corp_product(id)
        .await
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "corp product not found or inactive"))?;
    Ok(Json(p))
}

pub async fn create_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateShopOrderRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    let bindings = state.bindings.list_for_user(end_user_id).await;
    let binding = bindings
        .iter()
        .find(|b| b.id == req.game_account_id && b.status == "active")
        .ok_or_else(|| {
            err_status(
                StatusCode::BAD_REQUEST,
                "game_account_id must be an active binding for this user",
            )
        })?;

    let seller = req.seller_type.to_ascii_lowercase();
    let now = now_ms();
    let fiat = state.shop.base_fiat().await;
    let mut draft = match seller.as_str() {
        "platform" => {
            let id = req.package_id.ok_or_else(|| {
                err_status(StatusCode::BAD_REQUEST, "package_id required for platform")
            })?;
            let pkg = state.shop.get_package(id).await.ok_or_else(|| {
                err_status(StatusCode::BAD_REQUEST, "unknown or inactive package")
            })?;
            ShopOrderView {
                id: 0,
                end_user_id,
                game_account_id: binding.id,
                seller_type: "platform".into(),
                package_id: Some(pkg.id),
                corp_product_id: None,
                package_code: Some(pkg.code.clone()),
                product_code: None,
                credit_game_coin: Some(pkg.credit_game_coin.clone()),
                credit_amount: Some(pkg.coin_amount),
                item_code: None,
                fiat_currency: fiat.clone(),
                fiat_price: pkg.fiat_price,
                partner_order_no: None,
                checkout_url: None,
                status: "pending".into(),
                expires_at: now + order_ttl_ms(),
                paid_at: 0,
                created_at: now,
                game_coin_id: Some(pkg.game_coin_id),
                corporate_user_id: None,
            }
        }
        "corp" => {
            let id = req.corp_product_id.ok_or_else(|| {
                err_status(
                    StatusCode::BAD_REQUEST,
                    "corp_product_id required for corp",
                )
            })?;
            let p = state.shop.get_corp_product(id).await.ok_or_else(|| {
                err_status(StatusCode::BAD_REQUEST, "unknown or inactive corp product")
            })?;
            ShopOrderView {
                id: 0,
                end_user_id,
                game_account_id: binding.id,
                seller_type: "corp".into(),
                package_id: None,
                corp_product_id: Some(p.id),
                package_code: None,
                product_code: Some(p.code.clone()),
                credit_game_coin: p.credit_game_coin.clone(),
                credit_amount: p.credit_amount,
                item_code: p.item_code.clone(),
                fiat_currency: fiat.clone(),
                fiat_price: p.fiat_price,
                partner_order_no: None,
                checkout_url: None,
                status: "pending".into(),
                expires_at: now + order_ttl_ms(),
                paid_at: 0,
                created_at: now,
                game_coin_id: p.credit_game_coin_id,
                corporate_user_id: Some(p.corporate_user_id),
            }
        }
        other => {
            return Err(err_status(
                StatusCode::BAD_REQUEST,
                format!("seller_type must be platform|corp, got {other}"),
            ))
        }
    };

    let mut order = state
        .shop
        .insert_order(draft.clone())
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    draft.id = order.id;
    order.package_code = draft.package_code.clone();
    order.product_code = draft.product_code.clone();
    order.credit_game_coin = draft.credit_game_coin.clone();

    match state
        .pay
        .create_shop_payment(
            &state.config,
            &order,
            req.return_url.as_deref(),
            Some(&binding.game_account_id),
        )
        .await
    {
        Ok(pay) => {
            let _ = state
                .shop
                .set_payment(order.id, &pay.partner_order_no, &pay.checkout_url)
                .await;
            order.partner_order_no = Some(pay.partner_order_no);
            order.checkout_url = Some(pay.checkout_url);
        }
        Err(e) => {
            tracing::warn!(error = %e.1.error, "payment-gate unavailable; order left pending without checkout");
            let stub = format!("LOCAL-{}", order.id);
            let _ = state.shop.set_payment(order.id, &stub, "").await;
            order.partner_order_no = Some(stub);
        }
    }

    Ok((StatusCode::CREATED, Json(order)))
}

pub async fn list_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    Ok(Json(serde_json::json!({
        "orders": state.shop.list_orders(end_user_id).await,
        "source": "client_center",
    })))
}

pub async fn get_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    let order = state
        .shop
        .get_order(end_user_id, id)
        .await
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "order not found"))?;
    Ok(Json(order))
}

pub async fn cancel_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    let order = state
        .shop
        .cancel(end_user_id, id)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                err_status(StatusCode::NOT_FOUND, e)
            } else {
                err_status(StatusCode::BAD_REQUEST, e)
            }
        })?;
    Ok(Json(order))
}
