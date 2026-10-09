use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use funnyx_health::HealthPayload;
use funnyx_net_api::paths;
use serde_json::json;
use uuid::Uuid;

use crate::config::Config;
use crate::models::{
    DeviceStatus, DeviceStatusResponse, ErrorBody, HeartbeatRequest, RegisterServiceRequest,
    ReloadResponse, ServiceInstance, ServiceListResponse,
};
use crate::store::{now_ms, ConfigStore};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub store: ConfigStore,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(paths::HEALTH, get(health))
        .route(paths::CONFIG_RELOAD, post(reload))
        .route(paths::CONFIG_WHITELIST, get(whitelist))
        .route(
            paths::CONFIG_SERVICES,
            get(list_services).post(register_service),
        )
        // Legacy HTTP heartbeat (compat); core liveness is socket probe in `probe.rs`.
        .route(paths::CONFIG_SERVICE_HEARTBEAT, post(heartbeat))
        .route(paths::CONFIG_DEVICE_STATUS, get(device_status))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match HealthPayload::ok(state.config.service_name.clone()) {
        Ok(payload) => Json(payload).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn reload(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal_or_allowlist(&state, &headers, addr).await?;
    let snap = state
        .store
        .reload()
        .await
        .map_err(|e| err_tuple(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(ReloadResponse {
        reloaded: true,
        path: snap.path,
        loaded_at_ms: snap.loaded_at_ms,
        reload_count: snap.reload_count,
        allowlist_len: snap.allowlist.len(),
    }))
}

async fn whitelist(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal_or_allowlist(&state, &headers, addr).await?;
    Ok(Json(state.store.whitelist_snapshot().await))
}

async fn list_services(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal_or_allowlist(&state, &headers, addr).await?;
    Ok(Json(ServiceListResponse {
        services: state.store.list_services().await,
        source: "config_server".into(),
    }))
}

async fn register_service(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<RegisterServiceRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal_or_allowlist(&state, &headers, addr).await?;
    if req.service_name.trim().is_empty() || req.http_url.trim().is_empty() {
        return Err(err_tuple(
            StatusCode::BAD_REQUEST,
            "service_name and http_url required",
        ));
    }
    let now = now_ms();
    let inst = ServiceInstance {
        instance_id: req
            .instance_id
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("{}-{}", req.service_name, Uuid::new_v4().simple())),
        service_name: req.service_name,
        http_url: req.http_url,
        socket_url: req.socket_url.filter(|s| !s.is_empty()),
        group: req.group.filter(|s| !s.is_empty()),
        status: "up".into(),
        registered_at_ms: now,
        last_heartbeat_ms: now,
        metadata: if req.metadata.is_null() {
            json!({})
        } else {
            req.metadata
        },
    };
    let saved = state.store.register(inst).await;
    Ok((StatusCode::CREATED, Json(saved)))
}

async fn heartbeat(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<HeartbeatRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal_or_allowlist(&state, &headers, addr).await?;
    if req.instance_id.trim().is_empty() {
        return Err(err_tuple(StatusCode::BAD_REQUEST, "instance_id required"));
    }
    let rec = state
        .store
        .heartbeat(&req.instance_id, req.status)
        .await
        .ok_or_else(|| err_tuple(StatusCode::NOT_FOUND, "unknown instance_id"))?;
    Ok(Json(rec))
}

async fn device_status(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal_or_allowlist(&state, &headers, addr).await?;
    let timeout = state.config.heartbeat_timeout_ms;
    let now = now_ms();
    let devices = state
        .store
        .list_services()
        .await
        .into_iter()
        .map(|s| {
            let age = now.saturating_sub(s.last_heartbeat_ms);
            let stale = funnyx_heartbeat::is_stale(s.last_heartbeat_ms, timeout, now);
            let status = if stale {
                "stale".into()
            } else {
                s.status.clone()
            };
            // Healthy = recent private socket probe succeeded.
            let healthy = !stale && status == "up";
            DeviceStatus {
                instance_id: s.instance_id,
                service_name: s.service_name,
                http_url: s.http_url,
                status,
                healthy,
                last_heartbeat_ms: s.last_heartbeat_ms,
                age_ms: age,
            }
        })
        .collect();
    Ok(Json(DeviceStatusResponse {
        devices,
        heartbeat_timeout_ms: timeout,
        source: "config_server".into(),
    }))
}

async fn require_internal_or_allowlist(
    state: &AppState,
    headers: &HeaderMap,
    addr: std::net::SocketAddr,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    let key = headers
        .get("x-internal-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if key == state.config.internal_api_key {
        return Ok(());
    }
    let ip = client_ip(headers, addr);
    if !ip.is_empty() && state.store.allowlist_contains(&ip).await {
        return Ok(());
    }
    Err(err_tuple(
        StatusCode::UNAUTHORIZED,
        "X-Internal-Key or allowlisted IP required",
    ))
}

fn client_ip(headers: &HeaderMap, addr: std::net::SocketAddr) -> String {
    if let Some(fwd) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        return fwd;
    }
    match addr.ip() {
        std::net::IpAddr::V4(v4) => v4.to_string(),
        std::net::IpAddr::V6(v6) => v6
            .to_ipv4_mapped()
            .map(|v4| v4.to_string())
            .unwrap_or_else(|| v6.to_string()),
    }
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    err_tuple(status, msg)
}

fn err_tuple(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        status,
        Json(ErrorBody {
            error: msg.into(),
        }),
    )
}
