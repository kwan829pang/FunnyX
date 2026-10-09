mod bot;
mod config;
mod engine;
mod models;

use std::net::SocketAddr;

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use bot::Bot;
use config::Config;
use engine::EngineClient;
use models::{ErrorBody, HealthResponse};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let cfg = Config::from_env()?;
    let engine = EngineClient::new(cfg.core_engine_url.clone())?;
    let bot = Bot::new(cfg.clone(), engine);
    bot.clone().spawn_loop();

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/bot", get(bot_status))
        .route("/v1/bot/actions", get(bot_actions))
        .route("/v1/bot/pause", post(pause))
        .route("/v1/bot/resume", post(resume))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(bot);

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    tracing::info!(
        "market-bot listening on http://{addr} → {}",
        cfg.core_engine_url
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        service: "funnyx-market-bot".into(),
        timestamp_ms: Utc::now().timestamp_millis(),
    })
}

async fn bot_status(State(bot): State<Bot>) -> impl IntoResponse {
    Json(bot.status().await)
}

#[derive(Deserialize)]
struct LimitQuery {
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    50
}

async fn bot_actions(State(bot): State<Bot>, Query(q): Query<LimitQuery>) -> impl IntoResponse {
    Json(bot.recent_actions(q.limit).await)
}

async fn pause(State(bot): State<Bot>) -> impl IntoResponse {
    bot.set_paused(true).await;
    Json(bot.status().await)
}

async fn resume(State(bot): State<Bot>) -> Result<impl IntoResponse, (axum::http::StatusCode, Json<ErrorBody>)> {
    bot.set_paused(false).await;
    Ok(Json(bot.status().await))
}
