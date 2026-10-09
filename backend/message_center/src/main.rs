mod api;
mod config;
mod drain;
mod models;
mod mongo;
mod partner;
mod query;
mod store;
mod sts;

use std::net::SocketAddr;
use std::time::Duration;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use api::{router, AppState};
use config::Config;
use mongo::NoticeHistory;
use store::NoticeStore;
use sts::StsClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv_override().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = Config::from_env()?;
    let pool = if let Some(url) = &config.postgres_url {
        match sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await
        {
            Ok(p) => {
                tracing::info!("message center postgres connected");
                Some(p)
            }
            Err(e) => {
                tracing::warn!("POSTGRES_URL failed ({e}); memory outbox");
                None
            }
        }
    } else {
        tracing::info!("no POSTGRES_URL; memory outbox (dev only)");
        None
    };

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;
    let store = NoticeStore::new(pool);
    let history = NoticeHistory::connect(&config).await;

    let sock_addr: SocketAddr = format!("{}:{}", config.socket_host, config.socket_port).parse()?;
    funnyx_heartbeat::spawn_ping_pong_listener(sock_addr);
    funnyx_heartbeat::spawn_registry_register(
        http.clone(),
        funnyx_heartbeat::RegistryHeartbeatOpts {
            config_server_url: config.config_server_url.clone(),
            internal_api_key: config.internal_api_key.clone(),
            service_name: config.service_name.clone(),
            http_url: config.public_base_url.clone(),
            socket_url: Some(format!("tcp://127.0.0.1:{}", config.socket_port)),
            group: Some("message".into()),
            instance_id: None,
            timing: funnyx_heartbeat::HeartbeatTiming::from_env_or_default(),
        },
    );

    drain::spawn_drain_worker(store.clone(), config.clone(), http.clone(), history);

    let app = router(AppState {
        sts: StsClient::new(&config, http.clone()),
        store,
        config: config.clone(),
    })
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!(
        "funnyx-message-center listening on http://{addr} socket={} public_base={} sts={}",
        config.socket_port,
        config.public_base_url,
        config.session_token_server_url
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
