//! Admin system setup: base fiat + platform packages wizard APIs.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, extract_bearer, require_admin_session, AppState};
use crate::models::ErrorBody;

#[derive(Debug, Deserialize)]
pub struct BaseCurrencyBody {
    pub base_fiat_currency: String,
}

#[derive(Debug, Deserialize)]
pub struct PackagePatchBody {
    pub fiat_price: f64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePackageBody {
    pub code: String,
    pub name: String,
    pub coin_amount: f64,
    pub fiat_price: f64,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PackageStatusBody {
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

pub async fn get_setup(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let status = state.system.setup_status().await;
    Ok(Json(serde_json::json!({
        "setup": status,
        "source": "admin_api",
    })))
}

pub async fn complete_setup(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin_id = require_admin(&state, &headers).await?;
    let status = state
        .system
        .complete_setup(admin_id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::json!({
        "setup": status,
        "source": "admin_api",
    })))
}

pub async fn get_base_currency(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let code = state.system.get_base_currency().await;
    Ok(Json(serde_json::json!({
        "base_fiat_currency": code,
        "source": "admin_api",
    })))
}

pub async fn put_base_currency(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<BaseCurrencyBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin_id = require_admin(&state, &headers).await?;
    let code = state
        .system
        .set_base_currency(&body.base_fiat_currency, admin_id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::json!({
        "base_fiat_currency": code,
        "source": "admin_api",
    })))
}

pub async fn list_packages(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    Ok(Json(serde_json::json!({
        "packages": state.system.list_packages().await,
        "source": "admin_api",
    })))
}

pub async fn create_package(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreatePackageBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin_id = require_admin(&state, &headers).await?;
    let package = state
        .system
        .create_package(
            &body.code,
            &body.name,
            body.coin_amount,
            body.fiat_price,
            body.status.as_deref(),
            admin_id,
        )
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "package": package,
            "source": "admin_api",
        })),
    ))
}

pub async fn update_package(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<PackagePatchBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let package = state
        .system
        .update_package_price(
            id,
            body.fiat_price,
            body.name.as_deref(),
            body.status.as_deref(),
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
        "package": package,
        "source": "admin_api",
    })))
}

pub async fn patch_package_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<PackageStatusBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_admin(&state, &headers).await?;
    let package = state
        .system
        .set_package_status(id, body.status.trim())
        .await
        .map_err(|e| {
            if e.contains("not found") {
                err_status(StatusCode::NOT_FOUND, e)
            } else {
                err_status(StatusCode::BAD_REQUEST, e)
            }
        })?;
    Ok(Json(serde_json::json!({
        "package": package,
        "source": "admin_api",
    })))
}
