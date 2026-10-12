use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderName, HeaderValue, Method, Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
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
use crate::push::{internal_socket_push, ws_session_upgrade};
use crate::push_hub::PushHub;
use crate::sts::StsClient;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub http: Client,
    pub sts: StsClient,
    pub memory: SharedGatewayMemory,
    pub push: PushHub,
    pub rr: Arc<RrCounters>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(paths::HEALTH, get(health))
        // Gateway-owned (not proxied): Session WebSocket + internal MC push.
        .route(paths::GATEWAY_WS_SESSION, get(ws_session_upgrade))
        .route(paths::INTERNAL_SOCKET_PUSH, post(internal_socket_push))
        .fallback(proxy_fallback)
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    // Public liveness only. Routing failover uses Config socket-heartbeat + peer PING,
    // not this HTTP endpoint.
    let map = state.memory.read().await;
    let upstreams = map.upstream_role_summary();
    let peers = map.gateway_peer_summary();
    let self_id = map.self_instance_id.clone();
    let degraded = map.any_critical_upstream_down();
    drop(map);
    let base = if degraded {
        HealthPayload::degraded(state.config.service_name.clone())
    } else {
        HealthPayload::ok(state.config.service_name.clone())
    };
    match base {
        Ok(payload) => Json(serde_json::json!({
            "status": payload.status,
            "service": payload.service,
            "timestamp_ms": payload.timestamp_ms,
            "instance_id": self_id,
            "liveness": "socket_heartbeat",
            "upstreams": upstreams,
            "gateway_peers": peers,
        }))
        .into_response(),
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
            // Client Web corp session Bearer, or partner MasterSigned headers.
            let has_bearer = crate::auth::extract_bearer(&headers).is_some();
            if !has_bearer {
                if let Err((status, body)) = require_corp_headers(&headers) {
                    return (status, Json(body)).into_response();
                }
            }
        }
        AuthKind::Admin => {
            if let Err((status, body)) = require_bearer(&headers) {
                return (status, Json(body)).into_response();
            }
            if let Err(msg) = check_admin_allowlist(&state, &headers).await {
                return err_response(StatusCode::FORBIDDEN, msg);
            }
        }
    }

    let body = match axum::body::to_bytes(req.into_body(), 32 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return err_response(StatusCode::BAD_REQUEST, format!("read body: {e}"));
        }
    };

    let Some(upstream) = resolve_upstream(&state.memory, &state.config, target, &state.rr).await
    else {
        return err_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "no healthy upstream for route",
        );
    };
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

/// When Config whitelist is loaded and is not `*`-open, require client IP match for Admin proxy.
async fn check_admin_allowlist(state: &AppState, headers: &axum::http::HeaderMap) -> Result<(), String> {
    let map = state.memory.read().await;
    let Some(snap) = map.config_snapshot.as_ref() else {
        return Ok(());
    };
    if snap.allowlist.is_empty() || snap.allowlist.iter().any(|e| e == "*") {
        return Ok(());
    }
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_default();
    if ip.is_empty() {
        return Err("allowlisted IP required for admin via gateway".into());
    }
    if snap.allowlist.iter().any(|e| e == &ip) {
        Ok(())
    } else {
        Err(format!("IP {ip} not on config allowlist"))
    }
}
