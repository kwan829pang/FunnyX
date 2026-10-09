//! Corp e-shop product create / update / activate.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, AppState};
use crate::corp;
use crate::models::ErrorBody;
use crate::shop_store::CorpProductView;

#[derive(Debug, Deserialize)]
pub struct ProductBody {
    pub code: String,
    pub name: String,
    pub product_type: String,
    #[serde(default)]
    pub game_id: Option<i64>,
    #[serde(default)]
    pub credit_game_coin_id: Option<i64>,
    #[serde(default)]
    pub credit_amount: Option<f64>,
    #[serde(default)]
    pub item_code: Option<String>,
    pub fiat_price: f64,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusBody {
    pub status: String,
}

fn to_view(corp_id: i64, body: ProductBody, id: i64, status: String) -> CorpProductView {
    CorpProductView {
        id,
        corporate_user_id: corp_id,
        game_id: body.game_id,
        code: body.code,
        name: body.name,
        product_type: body.product_type,
        credit_game_coin_id: body.credit_game_coin_id,
        credit_game_coin: None,
        credit_amount: body.credit_amount,
        item_code: body.item_code,
        fiat_price: body.fiat_price,
        seller_type: "corp".into(),
        status,
    }
}

pub async fn list_products(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    Ok(Json(serde_json::json!({
        "products": state.shop.list_corp_products_owned(ident.corporate_user_id).await,
        "source": "client_center",
    })))
}

pub async fn get_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let Some(product) = state.shop.get_corp_product_owned(ident.corporate_user_id, id).await else {
        return Err(err_status(StatusCode::NOT_FOUND, "product not found"));
    };
    Ok(Json(serde_json::json!({
        "product": product,
        "source": "client_center",
    })))
}

async fn require_approved_cbt_for_coin_product(
    state: &AppState,
    corp_id: i64,
    product_type: &str,
    credit_game_coin_id: Option<i64>,
    activating: bool,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    if !activating {
        return Ok(());
    }
    if product_type != "company_coin_package" {
        return Ok(());
    }
    let Some(coin_id) = credit_game_coin_id else {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            "company_coin_package requires credit_game_coin_id",
        ));
    };
    if !state.cbt.has_approved_for_coin(corp_id, coin_id).await {
        return Err(err_status(
            StatusCode::FORBIDDEN,
            "Company Basic Token must be Admin-approved before activating company_coin_package products",
        ));
    }
    Ok(())
}

pub async fn create_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ProductBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let status = body.status.clone().unwrap_or_else(|| "draft".into());
    require_approved_cbt_for_coin_product(
        &state,
        ident.corporate_user_id,
        &body.product_type,
        body.credit_game_coin_id,
        status == "active",
    )
    .await?;
    let rec = to_view(ident.corporate_user_id, body, 0, status);
    let product = state
        .shop
        .create_corp_product(rec)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "product": product,
        "source": "client_center",
    }))))
}

pub async fn update_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<ProductBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let patch = to_view(
        ident.corporate_user_id,
        body,
        id,
        "draft".into(),
    );
    let product = state
        .shop
        .update_corp_product(ident.corporate_user_id, id, patch)
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
        "source": "client_center",
    })))
}

pub async fn patch_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<StatusBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let status = body.status.trim();
    if status == "active" {
        let Some(existing) = state
            .shop
            .get_corp_product_owned(ident.corporate_user_id, id)
            .await
        else {
            return Err(err_status(StatusCode::NOT_FOUND, "product not found"));
        };
        require_approved_cbt_for_coin_product(
            &state,
            ident.corporate_user_id,
            &existing.product_type,
            existing.credit_game_coin_id,
            true,
        )
        .await?;
    }
    let product = state
        .shop
        .set_corp_product_status(ident.corporate_user_id, id, status)
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
        "source": "client_center",
    })))
}
