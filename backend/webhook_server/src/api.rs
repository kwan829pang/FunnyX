use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use funnyx_health::HealthPayload;
use funnyx_net_api::paths;
use serde::Deserialize;

use crate::config::Config;
use crate::corp;
use crate::ingest::{CorpTokenPaymentCallback, ShopPaymentCallback};
use crate::models::ErrorBody;
use crate::settle;
use crate::store::{EventStore, WebhookEndpoint};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub store: EventStore,
    pub http: reqwest::Client,
    pub pool: Option<sqlx::PgPool>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(paths::HEALTH, get(health))
        .route(paths::WEBHOOK_SHOP_PAYMENT, post(shop_payment))
        .route(paths::WEBHOOK_CORP_TOKEN_PAYMENT, post(corp_token_payment))
        .route(paths::WEBHOOK_ENDPOINTS, get(list_endpoints).post(create_endpoint))
        .route(paths::WEBHOOK_ENDPOINT, put(update_endpoint))
        .route(paths::WEBHOOK_EVENT, get(get_event))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match HealthPayload::ok(state.config.service_name.clone()) {
        Ok(payload) => Json(payload).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn shop_payment(
    State(state): State<AppState>,
    Json(body): Json<ShopPaymentCallback>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let out = settle::accept_shop_payment(
        &state.store,
        &state.config,
        &state.http,
        body,
        "http",
    )
    .await
    .map_err(|e| {
        if e.contains("signature") {
            err_status(StatusCode::UNAUTHORIZED, e)
        } else {
            err_status(StatusCode::BAD_REQUEST, e)
        }
    })?;
    Ok(Json(out))
}

async fn corp_token_payment(
    State(state): State<AppState>,
    Json(body): Json<CorpTokenPaymentCallback>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let out = settle::accept_corp_token_payment(
        &state.store,
        &state.config,
        &state.http,
        body,
        "http",
    )
    .await
    .map_err(|e| {
        if e.contains("signature") {
            err_status(StatusCode::UNAUTHORIZED, e)
        } else {
            err_status(StatusCode::BAD_REQUEST, e)
        }
    })?;
    Ok(Json(out))
}

async fn get_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(event_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal(&state, &headers)?;
    let Some(ev) = state.store.get_by_event_id(&event_id).await else {
        return Err(err_status(StatusCode::NOT_FOUND, "event not found"));
    };
    Ok(Json(ev))
}

#[derive(Debug, Deserialize)]
struct EndpointBody {
    pub kind: String,
    pub callback_url: String,
    #[serde(default)]
    pub auth_type: Option<String>,
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EndpointPatch {
    #[serde(default)]
    pub callback_url: Option<String>,
    #[serde(default)]
    pub auth_type: Option<String>,
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

fn hint_from_secret(secret: Option<&str>) -> Option<String> {
    secret.filter(|s| !s.is_empty()).map(|s| {
        let n = s.chars().count();
        if n <= 4 {
            "****".into()
        } else {
            format!("****{}", s.chars().skip(n.saturating_sub(4)).collect::<String>())
        }
    })
}

fn validate_kind(kind: &str) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    match kind {
        "shop_payment" | "deposit" | "withdrawal" | "corp_token" => Ok(()),
        _ => Err(err_status(StatusCode::BAD_REQUEST, "invalid kind")),
    }
}

async fn list_endpoints(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<WebhookEndpoint>>, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let rows = state
        .store
        .list_endpoints(ident.corporate_user_id)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(rows))
}

async fn create_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<EndpointBody>,
) -> Result<(StatusCode, Json<WebhookEndpoint>), (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    validate_kind(&body.kind)?;
    if body.callback_url.trim().is_empty() {
        return Err(err_status(StatusCode::BAD_REQUEST, "callback_url required"));
    }
    let rec = state
        .store
        .upsert_endpoint(
            ident.corporate_user_id,
            &body.kind,
            body.callback_url.trim(),
            body.auth_type.as_deref().unwrap_or("signature"),
            hint_from_secret(body.secret.as_deref()).as_deref(),
            body.status.as_deref().unwrap_or("active"),
        )
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok((StatusCode::CREATED, Json(rec)))
}

async fn update_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<EndpointPatch>,
) -> Result<Json<WebhookEndpoint>, (StatusCode, Json<ErrorBody>)> {
    let ident = corp::require_master(&state, &headers).await?;
    let rec = state
        .store
        .update_endpoint(
            ident.corporate_user_id,
            id,
            body.callback_url.as_deref(),
            body.auth_type.as_deref(),
            hint_from_secret(body.secret.as_deref()).as_deref(),
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
    Ok(Json(rec))
}

fn require_internal(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    let key = headers
        .get("x-internal-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if key != state.config.internal_api_key {
        return Err(err_status(StatusCode::UNAUTHORIZED, "X-Internal-Key required"));
    }
    Ok(())
}

pub fn err_status(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.into() }))
}

fn err(status: StatusCode, msg: String) -> (StatusCode, Json<ErrorBody>) {
    err_status(status, msg)
}
