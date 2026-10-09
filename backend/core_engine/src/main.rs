mod api;
mod book;
mod engine;
mod notice;
mod order;
mod persist;
mod types;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use api::{router, AppState};
use engine::CoreEngine;
use notice::NoticePublisher;
use persist::EngineSnapshot;
use types::TradingPair;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let host = std::env::var("HTTP_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("HTTP_PORT")
        .unwrap_or_else(|_| "18200".into())
        .parse()?;
    let engine_id = std::env::var("ENGINE_ID").unwrap_or_else(|_| "core-engine-usdt-1".into());
    let quote_asset = std::env::var("QUOTE_ASSET").unwrap_or_else(|_| "USDT".into());
    let data_dir = PathBuf::from(
        std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".into()),
    );
    let message_center_url = std::env::var("MESSAGE_CENTER_URL").ok().filter(|s| !s.is_empty());
    let partner_notice_url = std::env::var("PARTNER_NOTICE_URL").ok().filter(|s| !s.is_empty());
    let bootstrap_pairs = std::env::var("BOOTSTRAP_PAIRS").unwrap_or_else(|_| {
        "BTC/USDT,ETH/USDT".into()
    });
    let save_interval_secs: u64 = std::env::var("SNAPSHOT_INTERVAL_SECS")
        .unwrap_or_else(|_| "30".into())
        .parse()
        .unwrap_or(30);

    let notices = NoticePublisher::new(message_center_url, partner_notice_url);
    let engine = CoreEngine::new(
        engine_id.clone(),
        quote_asset.clone(),
        data_dir.clone(),
        notices,
    );
    let admin_api_key =
        std::env::var("ADMIN_API_KEY").unwrap_or_else(|_| "demo-admin-key".into());

    // Startup: restore local snapshot into memory if present; else bootstrap pairs.
    if EngineSnapshot::exists(&data_dir) {
        match engine.load_snapshot_from_disk().await {
            Ok(()) => {
                let info = engine.maintenance_info().await;
                tracing::info!(
                    path = %info.snapshot_path,
                    pairs = info.listed_pairs,
                    orders = info.open_orders,
                    pools = info.pools,
                    "startup restored local snapshot into memory"
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "failed to load snapshot; starting empty");
            }
        }
    } else {
        for symbol in bootstrap_pairs
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            match TradingPair::parse(symbol, format!("mkt_{}", symbol.replace('/', "_"))) {
                Ok(pair) => match engine.add_pair(pair).await {
                    Ok(p) => tracing::info!(symbol = %p.symbol, "bootstrap pair listed"),
                    Err(e) => tracing::warn!(%symbol, error = %e, "bootstrap pair skipped"),
                },
                Err(e) => tracing::warn!(%symbol, error = %e, "invalid bootstrap pair"),
            }
        }
        if let Err(e) = engine.save_snapshot().await {
            tracing::warn!(error = %e, "initial snapshot save failed");
        }
    }

    // Periodic checkpoint while running.
    let engine_bg = engine.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(save_interval_secs.max(5)));
        loop {
            ticker.tick().await;
            if engine_bg.maintenance_status().await != types::MaintenanceStatus::Running {
                continue;
            }
            if let Err(e) = engine_bg.save_snapshot().await {
                tracing::warn!(error = %e, "periodic snapshot save failed");
            }
        }
    });

    let app = router(AppState {
        engine: engine.clone(),
        admin_api_key,
    })
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!(
        "funnyx-core-engine {engine_id} quote={quote_asset} data_dir={} listening on http://{addr} (max {} pairs)",
        data_dir.display(),
        types::MAX_PAIRS_PER_ENGINE
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(engine.clone()))
        .await?;
    Ok(())
}

async fn shutdown_signal(engine: CoreEngine) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received — saving snapshot");
    match engine.save_snapshot().await {
        Ok(path) => tracing::info!(path = %path.display(), "shutdown snapshot saved"),
        Err(e) => tracing::error!(error = %e, "shutdown snapshot save failed"),
    }
}
