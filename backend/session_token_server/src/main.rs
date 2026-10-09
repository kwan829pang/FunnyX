mod api;
mod config;
mod models;
mod oauth;
mod oauth_store;
mod store;
mod token;
mod users;

use std::net::SocketAddr;
use std::time::Duration;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use api::{router, AppState};
use config::{Config, StoreBackend};
use oauth_store::OauthStore;
use store::SessionStore;
use users::UserDirectory;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = Config::from_env()?;
    let store = match config.store_backend {
        StoreBackend::Memory => {
            tracing::info!("session store: in-memory");
            SessionStore::memory().await
        }
        StoreBackend::Redis => {
            tracing::info!(url = %config.redis_url, "session store: redis");
            SessionStore::redis(&config.redis_url).await?
        }
    };

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;

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
            group: Some("session".into()),
            instance_id: None,
            timing: funnyx_heartbeat::HeartbeatTiming::from_env_or_default(),
        },
    );

    let app = router(AppState {
        config: config.clone(),
        store,
        oauth: OauthStore::new(),
        users: UserDirectory::with_demo_users(),
        http,
    })
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!(
        "funnyx-session-token-server listening on http://{addr} socket={} public_base={} client_web_redirect={} ttl={}s",
        config.socket_port,
        config.public_base_url,
        config.client_web_redirect,
        config.token_ttl_secs
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
