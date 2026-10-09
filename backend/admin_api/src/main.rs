mod admins;
mod api;
mod auth;
mod cbt;
mod cbt_store;
mod config;
mod engine;
mod game_coin_store;
mod game_coins;
mod market_store;
mod markets;
mod models;
mod query;
mod shop;
mod shop_store;
mod sts;
mod system;
mod system_store;

use std::net::SocketAddr;
use std::time::Duration;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use admins::AdminDirectory;
use api::{router, AppState};
use cbt_store::BasicTokenStore;
use config::Config;
use engine::EngineClient;
use game_coin_store::GameCoinStore;
use market_store::AdminMarketStore;
use shop_store::CorpProductStore;
use sts::StsClient;
use system_store::SystemStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv_override().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = Config::from_env()?;
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;

    let pool = if let Some(url) = config.postgres_url.as_deref() {
        match sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await
        {
            Ok(p) => {
                tracing::info!("admin_api postgres connected");
                Some(p)
            }
            Err(e) => {
                tracing::warn!("POSTGRES_URL failed ({e}); using memory stores");
                None
            }
        }
    } else {
        tracing::info!("no POSTGRES_URL; admin stores in memory");
        None
    };

    let markets = AdminMarketStore::new(pool.clone());
    if pool.is_none() {
        markets.seed_pending_demo().await;
    }

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
            group: Some("admin".into()),
            instance_id: None,
            timing: funnyx_heartbeat::HeartbeatTiming::from_env_or_default(),
        },
    );

    let engine = EngineClient::new(&config, http.clone());
    let app = router(AppState {
        sts: StsClient::new(&config, http.clone()),
        admins: AdminDirectory::with_demo_admins(),
        shop: CorpProductStore::new(pool.clone()),
        cbt: BasicTokenStore::new(pool.clone()),
        system: SystemStore::new(pool.clone()),
        game_coins: GameCoinStore::new(pool.clone()),
        markets,
        engine,
        http,
        config: config.clone(),
    })
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!(
        "funnyx-admin-api listening on http://{addr} socket={} public_base={} sts={} engine={}",
        config.socket_port,
        config.public_base_url,
        config.session_token_server_url,
        config.core_engine_url
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
