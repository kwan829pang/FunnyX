use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, patch, post, put};
use axum::{Json, Router};
use funnyx_health::HealthPayload;
use funnyx_net_api::paths;

use crate::admins::{AdminDirectory, AdminUser};
use crate::auth;
use crate::cbt;
use crate::cbt_store::BasicTokenStore;
use crate::config::Config;
use crate::models::ErrorBody;
use crate::shop;
use crate::shop_store::CorpProductStore;
use crate::sts::{StsClient, StsValidate};
use crate::system;
use crate::system_store::SystemStore;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub admins: AdminDirectory,
    pub sts: StsClient,
    pub shop: CorpProductStore,
    pub cbt: BasicTokenStore,
    pub system: SystemStore,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(paths::HEALTH, get(health))
        .route(paths::ADMIN_LOGIN, post(auth::admin_login))
        .route(paths::ADMIN_LOGOUT, post(auth::admin_logout))
        .route(paths::ADMIN_ME, get(auth::admin_me))
        .route(paths::ADMIN_SHOP_CORP_PRODUCTS, get(shop::list_corp_products))
        .route(paths::ADMIN_SHOP_CORP_PRODUCT, get(shop::get_corp_product))
        .route(
            paths::ADMIN_SHOP_CORP_PRODUCT_STATUS,
            patch(shop::patch_corp_product_status),
        )
        .route(paths::ADMIN_BASIC_TOKENS, get(cbt::list_tokens))
        .route(paths::ADMIN_BASIC_TOKEN, get(cbt::get_token))
        .route(paths::ADMIN_BASIC_TOKEN_APPROVE, post(cbt::approve))
        .route(paths::ADMIN_BASIC_TOKEN_REJECT, post(cbt::reject))
        .route(paths::ADMIN_SYSTEM_SETUP, get(system::get_setup))
        .route(
            paths::ADMIN_SYSTEM_SETUP_COMPLETE,
            post(system::complete_setup),
        )
        .route(
            paths::ADMIN_SYSTEM_BASE_CURRENCY,
            get(system::get_base_currency).put(system::put_base_currency),
        )
        .route(
            paths::ADMIN_SHOP_PACKAGES,
            get(system::list_packages).post(system::create_package),
        )
        .route(paths::ADMIN_SHOP_PACKAGE, put(system::update_package))
        .route(
            paths::ADMIN_SHOP_PACKAGE_STATUS,
            patch(system::patch_package_status),
        )
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match HealthPayload::ok(state.config.service_name.clone()) {
        Ok(payload) => Json(payload).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            let v = v.trim();
            v.strip_prefix("Bearer ")
                .or_else(|| v.strip_prefix("bearer "))
                .map(|rest| rest.trim().to_string())
        })
        .filter(|t| !t.is_empty())
}

pub fn err_status(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        status,
        Json(ErrorBody {
            error: msg.into(),
        }),
    )
}

pub fn map_sts_err(
    (status, body): (reqwest::StatusCode, ErrorBody),
) -> (StatusCode, Json<ErrorBody>) {
    let mapped = StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    (mapped, Json(body))
}

pub async fn require_admin_session(
    state: &AppState,
    access_token: &str,
) -> Result<(AdminUser, StsValidate), (StatusCode, Json<ErrorBody>)> {
    let validated = state
        .sts
        .validate(access_token)
        .await
        .map_err(map_sts_err)?;
    if !validated.valid {
        return Err(err_status(
            StatusCode::UNAUTHORIZED,
            validated
                .reason
                .unwrap_or_else(|| "invalid_or_expired".into()),
        ));
    }
    let actor = validated
        .actor_type
        .as_deref()
        .unwrap_or("end_user");
    let scope = validated.scope.as_deref().unwrap_or("");
    if actor != "admin" && !scope.split(',').any(|s| s.trim() == "admin") {
        return Err(err_status(
            StatusCode::FORBIDDEN,
            "admin session required",
        ));
    }
    let admin_id = validated
        .end_user_id
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "session missing subject id"))?;
    let admin = state
        .admins
        .get_by_id(admin_id)
        .await
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "unknown admin subject"))?;
    Ok((admin, validated))
}
