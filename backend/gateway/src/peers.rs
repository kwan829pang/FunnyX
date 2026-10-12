//! Gateway ↔ Gateway mesh: discover peers from Config Server listing, socket PING/PONG.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::time::sleep;

use crate::config::Config;
use crate::memory::{GatewayPeer, ServiceRole, ServiceStatus, SharedGatewayMemory};

pub fn spawn_gateway_peer_mesh(config: Config, memory: SharedGatewayMemory) {
    tokio::spawn(async move {
        let interval = Duration::from_millis(config.registry_refresh_ms.max(1_000));
        let timeout_ms = config.peer_ping_timeout_ms;
        let self_id = config.instance_id.clone();
        {
            let mut map = memory.write().await;
            map.self_instance_id = Some(self_id.clone());
        }
        loop {
            if let Err(e) = ping_peers_once(&memory, &self_id, timeout_ms).await {
                tracing::debug!("gateway peer mesh: {e}");
            }
            sleep(interval).await;
        }
    });
}

async fn ping_peers_once(
    memory: &SharedGatewayMemory,
    self_id: &str,
    timeout_ms: u64,
) -> Result<(), String> {
    let targets: Vec<(String, String, String)> = {
        let map = memory.read().await;
        map.service_registry
            .values()
            .filter(|n| n.role == ServiceRole::Gateway)
            .filter(|n| n.service_id != self_id)
            .filter_map(|n| {
                let sock = n.socket_url.as_ref()?.trim();
                if sock.is_empty() {
                    return None;
                }
                Some((n.service_id.clone(), n.http_url.clone(), sock.to_string()))
            })
            .collect()
    };

    let now = now_ms();
    let mut updates: Vec<(String, GatewayPeer)> = Vec::new();
    for (id, http_url, socket_url) in targets {
        match funnyx_heartbeat::ping_peer(&socket_url, timeout_ms).await {
            Ok(()) => {
                tracing::debug!(peer = %id, %socket_url, "gateway peer PING ok");
                updates.push((
                    id.clone(),
                    GatewayPeer {
                        instance_id: id,
                        http_url,
                        socket_url,
                        status: ServiceStatus::Healthy,
                        last_ping_ok_ms: now,
                        last_error: None,
                    },
                ));
            }
            Err(e) => {
                tracing::warn!(peer = %id, %socket_url, error = %e, "gateway peer PING failed");
                updates.push((
                    id.clone(),
                    GatewayPeer {
                        instance_id: id,
                        http_url,
                        socket_url,
                        status: ServiceStatus::Unhealthy,
                        last_ping_ok_ms: 0,
                        last_error: Some(e),
                    },
                ));
            }
        }
    }

    let mut map = memory.write().await;
    // Drop peers that left the registry.
    let live: std::collections::HashSet<String> = updates.iter().map(|(id, _)| id.clone()).collect();
    map.gateway_peers.retain(|id, _| live.contains(id));
    for (id, peer) in updates {
        // Mirror peer socket health onto registry Gateway node for failover_map.
        if let Some(node) = map.service_registry.get_mut(&id) {
            node.status = peer.status;
            map.service_health.insert(id.clone(), peer.status);
        }
        map.gateway_peers.insert(id, peer);
    }
    map.rebuild_failover_map();
    Ok(())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
