use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderName, HeaderValue, Method, Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use funnyx_health::HealthPayload;
use funnyx_net_api::paths;
use reqwest::Client;

use crate::auth::{
    classify, gate_session, is_blocked, require_bearer, require_corp_headers, AuthKind,
};
use crate::config::Config;
use crate::memory::SharedGatewayMemory;
use crate::proxy::{err_response, forward, resolve_upstream, RrCounters};
use crate::sts::StsClient;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub http: Client,
    pub sts: StsClient,
    pub memory: SharedGatewayMemory,
    pub rr: Arc<RrCounters>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(paths::HEALTH, get(health))
        .fallback(proxy_fallback)
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match HealthPayload::ok(state.config.service_name.clone()) {
        Ok(payload) => Json(payload).into_response(),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn proxy_fallback(
    State(state): State<AppState>,
    req: Request<Body>,
) -> axum::response::Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();
    let path = uri.path().to_string();

    if method == Method::OPTIONS {
        return StatusCode::NO_CONTENT.into_response();
    }

    if is_blocked(&path) {
        return err_response(StatusCode::FORBIDDEN, "path not exposed via gateway");
    }

    let Some((target, auth)) = classify(&path) else {
        return err_response(StatusCode::NOT_FOUND, "no gateway route");
    };

    let mut extra: Vec<(HeaderName, HeaderValue)> = Vec::new();
    match auth {
        AuthKind::Public => {}
        AuthKind::Session => match gate_session(&state.sts, &headers).await {
            Ok(v) => {
                if let Some(uid) = v.end_user_id {
                    if let Ok(hv) = HeaderValue::from_str(&uid.to_string()) {
                        extra.push((
                            HeaderName::from_static("x-end-user-id"),
                            hv,
                        ));
                    }
                }
            }
            Err((status, body)) => return (status, Json(body)).into_response(),
        },
        AuthKind::Corp => {
            if let Err((status, body)) = require_corp_headers(&headers) {
                return (status, Json(body)).into_response();
            }
        }
        AuthKind::Admin => {
            if let Err((status, body)) = require_bearer(&headers) {
                return (status, Json(body)).into_response();
            }
        }
    }

    let body = match axum::body::to_bytes(req.into_body(), 32 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return err_response(StatusCode::BAD_REQUEST, format!("read body: {e}"));
        }
    };

    let upstream = resolve_upstream(&state.memory, &state.config, target, &state.rr).await;
    forward(
        &state.http,
        &upstream,
        method,
        uri,
        headers,
        body,
        extra,
    )
    .await
}
