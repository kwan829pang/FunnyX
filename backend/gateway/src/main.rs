mod api;
mod auth;
mod config;
mod memory;
mod proxy;
mod registry;
mod socket;
mod sts;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use api::{router, AppState};
use config::Config;
use memory::new_shared_memory;
use proxy::RrCounters;
use sts::StsClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv_override().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = Config::from_env()?;
    let http = reqwest::Client::builder()
        .timeout(Duration::from_millis(config.http_timeout_ms))
        .build()?;
    let memory = new_shared_memory();
    let sts = StsClient::new(&config, http.clone());

    let sock_addr: SocketAddr = format!("{}:{}", config.socket_host, config.socket_port).parse()?;
    socket::spawn_gateway_socket(sock_addr, sts.clone(), memory.clone());

    funnyx_heartbeat::spawn_registry_register(
        http.clone(),
        funnyx_heartbeat::RegistryHeartbeatOpts {
            config_server_url: config.config_server_url.clone(),
            internal_api_key: config.internal_api_key.clone(),
            service_name: config.service_name.clone(),
            http_url: config.public_base_url.clone(),
            socket_url: Some(format!("tcp://127.0.0.1:{}", config.socket_port)),
            group: Some("gateway".into()),
            instance_id: None,
            timing: funnyx_heartbeat::HeartbeatTiming::from_env_or_default(),
        },
    );

    registry::spawn_registry_refresh(http.clone(), config.clone(), memory.clone());

    let app = router(AppState {
        sts,
        http,
        memory,
        config: config.clone(),
        rr: Arc::new(RrCounters::default()),
    })
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!(
        "funnyx-gateway listening on http://{addr} socket={} public_base={} cc={} admin={} mc={} sts={} config={}",
        config.socket_port,
        config.public_base_url,
        config.client_center_url,
        config.admin_api_url,
        config.message_center_url,
        config.session_token_server_url,
        config.config_server_url
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
