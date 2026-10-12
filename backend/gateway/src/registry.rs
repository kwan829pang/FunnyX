//! Poll Config Server registry into the local Gateway memory map.
//!
//! Upstream liveness for routing comes from Config Server **socket** heartbeat
//! (`device-status` / last successful PING→PONG), not from HTTP `GET /health`.

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
    socket_url: Option<String>,
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
    #[serde(default)]
    status: String,
    #[serde(default)]
    last_heartbeat_ms: i64,
}

pub fn spawn_registry_refresh(http: Client, config: Config, memory: SharedGatewayMemory) {
    tokio::spawn(async move {
        let interval = Duration::from_millis(config.registry_refresh_ms);
        {
            let mut map = memory.write().await;
            map.self_instance_id = Some(config.instance_id.clone());
        }
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

    let resp = http
        .get(format!("{base}/v1/config/services"))
        .header("x-internal-key", &config.internal_api_key)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("config services {}", resp.status()));
    }
    let list: ServiceListResponse = resp.json().await.map_err(|e| e.to_string())?;

    // Socket-heartbeat truth from Config Server probes (not HTTP /health).
    let mut socket_health: std::collections::HashMap<String, (bool, i64, String)> =
        std::collections::HashMap::new();
    if let Ok(ds) = http
        .get(format!("{base}/v1/config/device-status"))
        .header("x-internal-key", &config.internal_api_key)
        .send()
        .await
    {
        if ds.status().is_success() {
            if let Ok(body) = ds.json::<DeviceStatusResponse>().await {
                for d in body.devices {
                    socket_health.insert(
                        d.instance_id,
                        (d.healthy, d.last_heartbeat_ms, d.status),
                    );
                }
            }
        }
    }

    let mut map = memory.write().await;
    let mut roles_from_registry = std::collections::HashSet::new();
    let mut seen_ids = std::collections::HashSet::new();

    for svc in list.services {
        let role = role_from_service_name(&svc.service_name);
        if matches!(role, ServiceRole::Unknown | ServiceRole::Config) {
            continue;
        }
        // Include Gateway peers so mesh discovery works.
        let (host, port) = parse_http_url(&svc.http_url);
        let socket_url = svc
            .socket_url
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let status = match socket_health.get(&svc.instance_id) {
            Some((true, _, st)) => {
                // Config socket probe OK — preserve degraded label if reported.
                let s = status_from_str(st);
                if matches!(s, ServiceStatus::Degraded) {
                    ServiceStatus::Degraded
                } else {
                    ServiceStatus::Healthy
                }
            }
            Some((false, _, _)) => ServiceStatus::Unhealthy,
            None => {
                // No device-status row yet: treat missing socket as degraded.
                if socket_url.is_none() {
                    ServiceStatus::Degraded
                } else if svc.status.is_empty() {
                    ServiceStatus::Degraded
                } else {
                    status_from_str(&svc.status)
                }
            }
        };

        let hb = socket_health
            .get(&svc.instance_id)
            .map(|(_, ms, _)| *ms)
            .unwrap_or(svc.last_heartbeat_ms)
            .max(0) as u64;

        roles_from_registry.insert(role);
        seen_ids.insert(svc.instance_id.clone());
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
                socket_url,
                protocol: "http".into(),
                region: "local".into(),
                status,
                last_heartbeat_ms: hb,
                uptime_ms: 0,
            },
        );
    }

    // Remove stale registry entries (except env-* fallbacks).
    let stale: Vec<String> = map
        .service_registry
        .keys()
        .filter(|id| !id.starts_with("env-") && !seen_ids.contains(*id))
        .cloned()
        .collect();
    for id in stale {
        map.service_registry.remove(&id);
        map.service_health.remove(&id);
    }

    // Drop env fallbacks when live registry nodes exist for that role.
    let env_ids: Vec<String> = map
        .service_registry
        .iter()
        .filter(|(id, n)| id.starts_with("env-") && roles_from_registry.contains(&n.role))
        .map(|(id, _)| id.clone())
        .collect();
    for id in env_ids {
        map.service_registry.remove(&id);
        map.service_health.remove(&id);
    }
    map.rebuild_failover_map();

    // Refresh allowlist snapshot from Config Server (Admin path gate).
    if let Ok(wl) = http
        .get(format!("{base}/v1/config/whitelist"))
        .header("x-internal-key", &config.internal_api_key)
        .send()
        .await
    {
        if wl.status().is_success() {
            if let Ok(body) = wl.json::<serde_json::Value>().await {
                let allowlist = body
                    .get("allowlist")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let version = body
                    .get("version")
                    .or_else(|| body.get("config_version"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                map.config_snapshot = Some(crate::memory::ConfigSnapshot {
                    config_version: version,
                    feature_flags: Default::default(),
                    rate_limits: Default::default(),
                    allowlist,
                    thresholds: Default::default(),
                });
            }
        }
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
                socket_url: None,
                protocol: "http".into(),
                region: "local".into(),
                status: ServiceStatus::Healthy,
                last_heartbeat_ms: 0,
                uptime_ms: 0,
            },
        );
    }
}
