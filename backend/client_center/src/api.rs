use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{any, get, patch, post};
use axum::{Json, Router};
use funnyx_health::HealthPayload;
use funnyx_net_api::paths;

use crate::auth;
use crate::bindings::GameAccountStore;
use crate::cbt;
use crate::cbt_store::CbtStore;
use crate::config::Config;
use crate::corp_cbt;
use crate::corp_markets;
use crate::corp_shop;
use crate::game_accounts;
use crate::games::GameCatalog;
use crate::internal;
use crate::market_store::MarketStore;
use crate::models::ErrorBody;
use crate::oauth;
use crate::partner::PartnerClient;
use crate::pay::PaymentGateClient;
use crate::shop;
use crate::shop_store::ShopStore;
use crate::sts::StsClient;
use crate::users::UserDirectory;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub users: UserDirectory,
    pub sts: StsClient,
    pub games: GameCatalog,
    pub bindings: GameAccountStore,
    pub partner: PartnerClient,
    pub shop: ShopStore,
    pub markets: MarketStore,
    pub cbt: CbtStore,
    pub pay: PaymentGateClient,
    pub pool: Option<PgPool>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(paths::HEALTH, get(health))
        .route(paths::CLIENT_LOGIN, post(auth::client_login))
        .route(paths::CLIENT_REGISTER, post(auth::client_register))
        .route(paths::CLIENT_LOGOUT, post(auth::client_logout))
        .route(paths::CLIENT_PROFILE, get(auth::client_profile))
        .route(paths::CLIENT_GAMES, get(game_accounts::list_games))
        .route(paths::CLIENT_GAME_ACCOUNTS, get(game_accounts::list_game_accounts))
        .route(
            paths::CLIENT_GAME_ACCOUNTS_BIND,
            post(game_accounts::bind_game_account),
        )
        // Partner OAuth (brokered by Session Token Server)
        .route(
            "/v1/client/oauth/partner/{partner_id}/start",
            get(oauth::partner_oauth_start),
        )
        .route(
            paths::OAUTH_PARTNER_COMPLETE,
            get(oauth::partner_oauth_complete),
        )
        .route(
            paths::OAUTH_PARTNER_CALLBACK,
            get(oauth::partner_oauth_callback_alias),
        )
        // Platform OAuth IdP — entry on CC, runtime on STS
        .route(paths::OAUTH_AUTHORIZE, get(oauth::platform_authorize_redirect))
        .route(paths::OAUTH_LOGIN, any(oauth::proxy_sts_oauth))
        .route(paths::OAUTH_TOKEN, post(oauth::proxy_sts_oauth))
        .route(paths::OAUTH_USERINFO, get(oauth::proxy_sts_oauth))
        .route(paths::SHOP_BASE_CURRENCY, get(shop::shop_base_currency))
        .route(paths::SHOP_PACKAGES, get(shop::list_packages))
        .route(paths::SHOP_PACKAGE, get(shop::get_package))
        .route(paths::SHOP_CORP_PRODUCTS, get(shop::list_corp_products))
        .route(paths::SHOP_CORP_PRODUCT, get(shop::get_corp_product))
        .route(paths::SHOP_ORDERS, get(shop::list_orders).post(shop::create_order))
        .route(paths::SHOP_ORDER, get(shop::get_order))
        .route(paths::SHOP_ORDER_CANCEL, post(shop::cancel_order))
        .route(paths::INTERNAL_SHOP_SETTLE, post(internal::shop_settle))
        .route(
            paths::INTERNAL_CORP_TOKEN_SETTLE,
            post(internal::cbt_settle),
        )
        .route(
            paths::CORP_BASIC_TOKENS,
            get(corp_cbt::list_tokens).post(corp_cbt::create_token),
        )
        .route(
            paths::CORP_BASIC_TOKEN,
            get(corp_cbt::get_token).put(corp_cbt::update_token),
        )
        .route(
            paths::CORP_BASIC_TOKEN_BUY_ORDERS,
            get(corp_cbt::list_buy_orders),
        )
        .route(paths::CORP_BASIC_TOKEN_FEES, get(corp_cbt::list_fees))
        .route(paths::CORP_TOKEN_ORDERS, get(cbt::list_orders))
        .route(paths::CORP_TOKEN_ORDER, get(cbt::get_order))
        .route(paths::CORP_TOKEN_ORDER_CANCEL, post(cbt::cancel_order))
        .route(paths::CORP_TOKENS, get(cbt::list_buyable))
        .route(paths::CORP_TOKEN, get(cbt::get_buyable))
        .route(paths::CORP_TOKEN_CREATE_ORDER, post(cbt::create_order))
        .route(
            paths::CORP_SHOP_PRODUCTS,
            get(corp_shop::list_products).post(corp_shop::create_product),
        )
        .route(
            paths::CORP_SHOP_PRODUCT,
            get(corp_shop::get_product).put(corp_shop::update_product),
        )
        .route(
            paths::CORP_SHOP_PRODUCT_STATUS,
            patch(corp_shop::patch_status),
        )
        .route(
            paths::CORP_MARKETS,
            get(corp_markets::list_markets).post(corp_markets::submit_market),
        )
        .route(paths::CORP_MARKET, get(corp_markets::get_market))
        .route(paths::CORP_MARKET_POOL, post(corp_markets::create_pool))
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
