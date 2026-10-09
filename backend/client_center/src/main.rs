mod api;
mod auth;
mod bindings;
mod cbt;
mod cbt_store;
mod chat;
mod chat_store;
mod config;
mod corp;
mod corp_cbt;
mod corp_markets;
mod corp_shop;
mod db;
mod game_accounts;
mod games;
mod internal;
mod market_store;
mod models;
mod oauth;
mod partner;
mod password;
mod pay;
mod query;
mod sessions;
mod shop;
mod shop_store;
mod sts;
mod users;
mod wallet;
mod wallet_store;

use std::net::SocketAddr;
use std::time::Duration;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use api::{router, AppState};
use bindings::GameAccountStore;
use chat_store::ChatStore;
use config::Config;
use games::GameCatalog;
use cbt_store::CbtStore;
use market_store::MarketStore;
use partner::PartnerClient;
use pay::PaymentGateClient;
use shop_store::ShopStore;
use sts::StsClient;
use users::UserDirectory;
use wallet_store::WalletStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv_override().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = Config::from_env()?;
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let (users, games, bindings, pool) = if let Some(url) = config.postgres_url.as_deref() {
        let pool = db::connect(url).await?;
        let users = UserDirectory::postgres(pool.clone());
        users.ensure_demo_password_hashes().await?;
        tracing::info!("client_center using PostgreSQL for users, sessions, games, game_accounts");
        (
            users,
            GameCatalog::postgres(pool.clone()),
            GameAccountStore::postgres(pool.clone()),
            Some(pool),
        )
    } else {
        tracing::warn!(
            "POSTGRES_URL unset; Client Center using in-memory users/games (hashed passwords still)"
        );
        (
            UserDirectory::memory()?,
            GameCatalog::memory(&config.default_partner_id),
            GameAccountStore::memory(),
            None,
        )
    };

    let shop = if let Some(p) = &pool {
        ShopStore::postgres(p.clone())
    } else {
        ShopStore::memory()
    };
    let markets = MarketStore::new(pool.clone());
    let cbt = CbtStore::new(pool.clone());
    let wallets = WalletStore::new(pool.clone());
    let chat = ChatStore::connect(&config).await;

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
            group: Some("client".into()),
            instance_id: None,
            timing: funnyx_heartbeat::HeartbeatTiming::from_env_or_default(),
        },
    );

    let app = router(AppState {
        sts: StsClient::new(&config, http.clone()),
        partner: PartnerClient::new(&config, http.clone()),
        pay: PaymentGateClient::new(&config, http),
        users,
        games,
        bindings,
        shop,
        markets,
        cbt,
        wallets,
        chat,
        pool,
        config: config.clone(),
    })
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!(
        "funnyx-client-center listening on http://{addr} socket={} public_base={} sts={} partner_a={} pay={} mongo={}",
        config.socket_port,
        config.public_base_url,
        config.session_token_server_url,
        config.partner_a_base_url,
        config.payment_gate_url,
        config.mongo_url.as_deref().unwrap_or("(unset)")
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
