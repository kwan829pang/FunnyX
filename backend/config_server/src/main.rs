mod api;
mod config;
mod models;
mod probe;
mod store;

use std::net::SocketAddr;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use api::{router, AppState};
use config::Config;
use store::ConfigStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv_override().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = Config::from_env()?;
    let store = ConfigStore::load(config.whitelist_path.clone()).await?;
    tracing::info!(
        path = %config.whitelist_path.display(),
        "whitelist loaded"
    );

    probe::spawn_registry_socket_probe(
        store.clone(),
        funnyx_heartbeat::HeartbeatTiming::from_env_or_default(),
    );

    let app = router(AppState {
        config: config.clone(),
        store,
    })
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!(
        "funnyx-config-server listening on http://{addr} public_base={} whitelist={}",
        config.public_base_url,
        config.whitelist_path.display()
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;
    Ok(())
}
