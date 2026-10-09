mod models;
mod sim_status;
mod state;

use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use models::{
    valid_delivery_status, valid_event_type, valid_source_type, AckRequest, DrainRequest,
    EnqueueNoticeRequest, ErrorBody, HealthResponse, ListQuery,
};
use sim_status::{SimDirective, SimStatus, HEADER_NAME};
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let host = std::env::var("HTTP_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("HTTP_PORT")
        .unwrap_or_else(|_| "18101".into())
        .parse()?;
    let default_callback_url = std::env::var("NOTICE_CALLBACK_URL").ok().filter(|s| !s.is_empty());

    let app = Router::new()
        .route("/health", get(health))
        .route(
            "/v1/notices",
            get(list_notices).post(enqueue_notice).delete(clear_notices),
        )
        .route("/v1/notices/stats", get(stats))
        .route("/v1/notices/drain", post(drain_notices))
        .route("/v1/notices/{id}", get(get_notice))
        .route("/v1/notices/{id}/ack", post(ack_notice))
        .route("/v1/notices/{id}/retry", post(retry_notice))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(AppState::new(default_callback_url));

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!(
        "message-queue listening on http://{addr} (header {HEADER_NAME}='STATUS' or 'STATUS ms')"
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        service: "message-queue".into(),
        timestamp_ms: Utc::now().timestamp_millis(),
    })
}

async fn enqueue_notice(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<EnqueueNoticeRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let directive = SimDirective::from_headers(&headers)?;
    if !valid_source_type(&req.source_type) {
        return Err(err(
            StatusCode::BAD_REQUEST,
            format!("invalid source_type '{}'", req.source_type),
        ));
    }
    if !valid_event_type(&req.event_type) {
        return Err(err(
            StatusCode::BAD_REQUEST,
            format!("invalid event_type '{}'", req.event_type),
        ));
    }
    if req.title.is_empty() || req.body.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "title and body required"));
    }

    let record = state.enqueue(req).await;
    let (sim_status, sim_delay_ms, header_echo) = schedule_notice(&state, record.id, directive);
    let view = state
        .get(record.id)
        .await
        .map(|r| r.to_view_with_sim(&sim_status, sim_delay_ms))
        .unwrap_or_else(|| record.to_view_with_sim(&sim_status, sim_delay_ms));

    Ok((
        StatusCode::CREATED,
        [(HEADER_NAME, header_echo)],
        Json(view),
    ))
}

async fn list_notices(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    if let Some(ref s) = q.delivery_status {
        if !valid_delivery_status(s) {
            return Err(err(
                StatusCode::BAD_REQUEST,
                format!("invalid delivery_status '{s}'"),
            ));
        }
    }
    Ok(Json(state.list(&q).await))
}

async fn get_notice(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let record = state
        .get(id)
        .await
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "notice not found"))?;
    Ok(Json(record.to_view()))
}

async fn drain_notices(
    State(state): State<AppState>,
    body: Option<Json<DrainRequest>>,
) -> impl IntoResponse {
    let limit = body.map(|b| b.0.limit).unwrap_or(10);
    let claimed = state.drain(limit).await;
    let views: Vec<_> = claimed.iter().map(|n| n.to_view()).collect();
    Json(views)
}

async fn ack_notice(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<AckRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let record = state.ack(id, req).await.map_err(map_state_err)?;
    Ok(Json(record.to_view()))
}

async fn retry_notice(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let record = state.retry(id).await.map_err(map_state_err)?;
    Ok(Json(record.to_view()))
}

async fn clear_notices(State(state): State<AppState>) -> impl IntoResponse {
    state.clear().await;
    StatusCode::NO_CONTENT
}

async fn stats(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.stats().await)
}

fn schedule_notice(
    state: &AppState,
    notice_id: i64,
    directive: Option<SimDirective>,
) -> (String, u64, String) {
    let Some(directive) = directive.filter(|d| d.schedules_change()) else {
        return (
            SimStatus::Pending.as_str().into(),
            0,
            format!("{} 0", SimStatus::Pending.as_str()),
        );
    };

    let state = state.clone();
    let delay_ms = directive.delay_ms;
    let target = directive.status;
    tokio::spawn(async move {
        if delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
        match state
            .apply_sim_status(notice_id, target, target.should_fire_callback())
            .await
        {
            Ok(_) => tracing::info!(
                notice_id,
                sim_status = target.as_str(),
                delay_ms,
                "applied scheduled notice status + webhook"
            ),
            Err(e) => tracing::warn!(
                notice_id,
                error = %e,
                "scheduled notice status skipped or failed"
            ),
        }
    });

    (
        target.as_str().into(),
        delay_ms,
        directive.header_value(),
    )
}

fn map_state_err(e: anyhow::Error) -> (StatusCode, Json<ErrorBody>) {
    let code = if e.to_string().contains("not found") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::BAD_REQUEST
    };
    err(code, e.to_string())
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.into() }))
}
