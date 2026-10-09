mod api;
mod config;
mod corp;
mod ingest;
mod models;
mod settle;
mod socket;
mod store;

use std::net::SocketAddr;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use api::{router, AppState};
use config::Config;
use store::EventStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv_override().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = Config::from_env()?;
    let pool = if let Some(url) = &config.postgres_url {
        match sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await
        {
            Ok(p) => {
                tracing::info!("webhook postgres connected");
                Some(p)
            }
            Err(e) => {
                tracing::warn!("POSTGRES_URL failed ({e}); using memory store");
                None
            }
        }
    } else {
        tracing::info!("no POSTGRES_URL; webhook events in memory");
        None
    };

    let store = EventStore::new(pool.clone());
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    settle::spawn_retry_worker(store.clone(), config.clone(), http.clone());

    funnyx_heartbeat::spawn_registry_register(
        http.clone(),
        funnyx_heartbeat::RegistryHeartbeatOpts {
            config_server_url: config.config_server_url.clone(),
            internal_api_key: config.internal_api_key.clone(),
            service_name: config.service_name.clone(),
            http_url: config.public_base_url.clone(),
            socket_url: Some(format!(
                "tcp://127.0.0.1:{}",
                config.socket_port
            )),
            group: Some("webhook".into()),
            instance_id: None,
            timing: funnyx_heartbeat::HeartbeatTiming::from_env_or_default(),
        },
    );

    let sock_addr: SocketAddr = format!("{}:{}", config.socket_host, config.socket_port).parse()?;
    socket::spawn_socket(sock_addr, store.clone(), config.clone(), http.clone());

    let app = router(AppState {
        config: config.clone(),
        store,
        http,
        pool,
    })
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!(
        "funnyx-webhook-server listening on http://{addr} socket={} public_base={}",
        config.socket_port,
        config.public_base_url
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
