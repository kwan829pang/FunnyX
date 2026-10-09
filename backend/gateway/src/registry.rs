//! Poll Config Server registry into the local Gateway memory map.

use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;
use tokio::time::sleep;

use crate::config::Config;
use crate::memory::{
    parse_http_url, role_from_service_name, status_from_str, ServiceNode, ServiceRole,
    ServiceStatus, SharedGatewayMemory,
};

#[derive(Debug, Deserialize)]
struct ServiceListResponse {
    services: Vec<RemoteService>,
}

#[derive(Debug, Deserialize)]
struct RemoteService {
    instance_id: String,
    service_name: String,
    http_url: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    last_heartbeat_ms: i64,
}

#[derive(Debug, Deserialize)]
struct DeviceStatusResponse {
    devices: Vec<DeviceStatus>,
}

#[derive(Debug, Deserialize)]
struct DeviceStatus {
    instance_id: String,
    healthy: bool,
    status: String,
}

pub fn spawn_registry_refresh(http: Client, config: Config, memory: SharedGatewayMemory) {
    tokio::spawn(async move {
        let interval = Duration::from_millis(config.registry_refresh_ms);
        loop {
            if let Err(e) = refresh_once(&http, &config, &memory).await {
                tracing::debug!("gateway registry refresh: {e}");
            }
            sleep(interval).await;
        }
    });
}

async fn refresh_once(
    http: &Client,
    config: &Config,
    memory: &SharedGatewayMemory,
) -> Result<(), String> {
    seed_env_fallbacks(config, memory).await;

    let base = config.config_server_url.trim();
    if base.is_empty() {
        return Ok(());
    }

    let url = format!("{}/v1/config/services", base);
    let resp = http
        .get(&url)
        .header("x-internal-key", &config.internal_api_key)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("config services {}", resp.status()));
    }
    let list: ServiceListResponse = resp.json().await.map_err(|e| e.to_string())?;

    let mut health_override: std::collections::HashMap<String, bool> =
        std::collections::HashMap::new();
    if let Ok(ds) = http
        .get(format!("{}/v1/config/device-status", base))
        .header("x-internal-key", &config.internal_api_key)
        .send()
        .await
    {
        if ds.status().is_success() {
            if let Ok(body) = ds.json::<DeviceStatusResponse>().await {
                for d in body.devices {
                    health_override.insert(d.instance_id, d.healthy);
                    let _ = d.status;
                }
            }
        }
    }

    let mut map = memory.write().await;
    for svc in list.services {
        let role = role_from_service_name(&svc.service_name);
        if matches!(
            role,
            ServiceRole::Gateway | ServiceRole::Unknown | ServiceRole::Config
        ) {
            continue;
        }
        let (host, port) = parse_http_url(&svc.http_url);
        let healthy = health_override
            .get(&svc.instance_id)
            .copied()
            .unwrap_or_else(|| {
                matches!(
                    status_from_str(&svc.status),
                    ServiceStatus::Healthy | ServiceStatus::Degraded
                ) || svc.status.is_empty()
            });
        let status = if healthy {
            ServiceStatus::Healthy
        } else {
            ServiceStatus::Unhealthy
        };
        let id = svc.instance_id.clone();
        map.service_health.insert(id.clone(), status);
        map.service_registry.insert(
            id.clone(),
            ServiceNode {
                service_id: id,
                service_name: svc.service_name,
                role,
                host,
                port,
                http_url: svc.http_url.trim_end_matches('/').to_string(),
                protocol: "http".into(),
                region: "local".into(),
                status,
                last_heartbeat_ms: svc.last_heartbeat_ms.max(0) as u64,
                uptime_ms: 0,
            },
        );
    }
    Ok(())
}

async fn seed_env_fallbacks(config: &Config, memory: &SharedGatewayMemory) {
    let mut map = memory.write().await;
    let seeds = [
        (
            "env-client-center",
            "funnyx-client-center",
            ServiceRole::ClientCenter,
            config.client_center_url.as_str(),
        ),
        (
            "env-admin-api",
            "funnyx-admin-api",
            ServiceRole::AdminApi,
            config.admin_api_url.as_str(),
        ),
        (
            "env-message-center",
            "funnyx-message-center",
            ServiceRole::MessageCenter,
            config.message_center_url.as_str(),
        ),
        (
            "env-session-token",
            "funnyx-session-token-server",
            ServiceRole::SessionToken,
            config.session_token_server_url.as_str(),
        ),
    ];
    for (id, name, role, url) in seeds {
        if map.service_registry.contains_key(id) {
            continue;
        }
        let (host, port) = parse_http_url(url);
        map.service_health.insert(id.into(), ServiceStatus::Healthy);
        map.service_registry.insert(
            id.into(),
            ServiceNode {
                service_id: id.into(),
                service_name: name.into(),
                role,
                host,
                port,
                http_url: url.trim_end_matches('/').to_string(),
                protocol: "http".into(),
                region: "local".into(),
                status: ServiceStatus::Healthy,
                last_heartbeat_ms: 0,
                uptime_ms: 0,
            },
        );
    }
}
