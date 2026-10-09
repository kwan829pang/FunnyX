use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, patch};
use axum::{Json, Router};
use funnyx_health::HealthPayload;
use funnyx_net_api::paths;
use serde::Deserialize;

use crate::config::Config;
use crate::models::{
    valid_event_type, valid_source_type, EnqueueNoticeRequest, ErrorBody, MarkReadBody,
};
use crate::store::NoticeStore;
use crate::sts::StsClient;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub store: NoticeStore,
    pub sts: StsClient,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(paths::HEALTH, get(health))
        .route(paths::NOTICES, get(list_notices).post(enqueue_notice))
        .route(paths::NOTICES_STATS, get(notice_stats))
        .route(paths::NOTICE, get(get_notice))
        .route(paths::NOTIFICATIONS, get(list_notifications))
        .route(paths::NOTIFICATION, patch(mark_notification_read))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match HealthPayload::ok(state.config.service_name.clone()) {
        Ok(payload) => Json(payload).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn enqueue_notice(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<EnqueueNoticeRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal(&state, &headers)?;
    if body.end_user_id <= 0 {
        return Err(err_tuple(StatusCode::BAD_REQUEST, "end_user_id required"));
    }
    if !valid_source_type(&body.source_type) {
        return Err(err_tuple(StatusCode::BAD_REQUEST, "invalid source_type"));
    }
    if !valid_event_type(&body.event_type) {
        return Err(err_tuple(StatusCode::BAD_REQUEST, "invalid event_type"));
    }
    if body.title.trim().is_empty() {
        return Err(err_tuple(StatusCode::BAD_REQUEST, "title required"));
    }
    let notice = state
        .store
        .enqueue(
            body.end_user_id,
            &body.source_type,
            body.source_id,
            &body.event_type,
            &body.title,
            &body.body,
            body.payload,
            body.scheduled_at,
            body.callback_url,
        )
        .await
        .map_err(|e| err_tuple(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok((StatusCode::CREATED, Json(notice)))
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    delivery_status: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    50
}

async fn list_notices(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal(&state, &headers)?;
    let list = state
        .store
        .list_notices(q.delivery_status.as_deref(), q.limit)
        .await
        .map_err(|e| err_tuple(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({ "notices": list })))
}

async fn notice_stats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal(&state, &headers)?;
    let stats = state
        .store
        .stats()
        .await
        .map_err(|e| err_tuple(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(stats))
}

async fn get_notice(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal(&state, &headers)?;
    match state
        .store
        .get_notice(id)
        .await
        .map_err(|e| err_tuple(StatusCode::INTERNAL_SERVER_ERROR, e))?
    {
        Some(n) => Ok(Json(n)),
        None => Err(err_tuple(StatusCode::NOT_FOUND, "notice not found")),
    }
}

async fn list_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_session(&state, &headers).await?;
    let list = state
        .store
        .list_notifications(user_id, q.limit)
        .await
        .map_err(|e| err_tuple(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({ "notifications": list })))
}

async fn mark_notification_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<MarkReadBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_session(&state, &headers).await?;
    let now = crate::store::now_ms();
    match state
        .store
        .mark_notification_read(user_id, id, body.read, now)
        .await
        .map_err(|e| err_tuple(StatusCode::INTERNAL_SERVER_ERROR, e))?
    {
        Some(n) => Ok(Json(n)),
        None => Err(err_tuple(StatusCode::NOT_FOUND, "notification not found")),
    }
}

fn require_internal(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    let key = headers
        .get("x-internal-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if key == state.config.internal_api_key {
        Ok(())
    } else {
        Err(err_tuple(StatusCode::UNAUTHORIZED, "X-Internal-Key required"))
    }
}

async fn require_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<i64, (StatusCode, Json<ErrorBody>)> {
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth
        .strip_prefix("Bearer ")
        .or_else(|| auth.strip_prefix("bearer "))
        .unwrap_or("")
        .trim();
    if token.is_empty() {
        return Err(err_tuple(StatusCode::UNAUTHORIZED, "Bearer token required"));
    }
    let v = state.sts.validate(token).await.map_err(|(s, e)| (s, Json(e)))?;
    if !v.valid {
        return Err(err_tuple(
            StatusCode::UNAUTHORIZED,
            v.reason.unwrap_or_else(|| "invalid session".into()),
        ));
    }
    v.end_user_id
        .filter(|&id| id > 0)
        .ok_or_else(|| err_tuple(StatusCode::UNAUTHORIZED, "session missing end_user_id"))
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
