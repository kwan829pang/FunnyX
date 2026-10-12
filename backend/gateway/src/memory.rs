//! In-memory Gateway runtime map ([doc/gateway_core_engine_logic.md] §2.7).

#![allow(dead_code)] // Route/pool/market tables reserved for LB metrics.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::sync::RwLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ServiceRole {
    Config,
    Gateway,
    ClientCenter,
    MessageCenter,
    Webhook,
    CoreEngine,
    SessionToken,
    AdminApi,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

#[derive(Clone, Debug)]
pub struct ServiceNode {
    pub service_id: String,
    pub service_name: String,
    pub role: ServiceRole,
    pub host: String,
    pub port: u16,
    pub http_url: String,
    /// Private socket for Config / peer PING/PONG (`tcp://host:port`).
    pub socket_url: Option<String>,
    pub protocol: String,
    pub region: String,
    /// From Config Server socket-heartbeat (device-status), not HTTP `/health`.
    pub status: ServiceStatus,
    pub last_heartbeat_ms: u64,
    pub uptime_ms: u64,
}

#[derive(Clone, Debug)]
pub struct GatewayPeer {
    pub instance_id: String,
    pub http_url: String,
    pub socket_url: String,
    pub status: ServiceStatus,
    pub last_ping_ok_ms: u64,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RouteEntry {
    pub route_key: String,
    pub path: String,
    pub target_service: String,
    pub route_policy: String,
    pub timeout_ms: u64,
    pub retry_count: u8,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct ConnectionPoolInfo {
    pub service_id: String,
    pub active_conn: usize,
    pub max_conn: usize,
    pub queue_depth: usize,
    pub pending_count: usize,
    pub last_reset_ms: u64,
}

#[derive(Clone, Debug)]
pub struct MarketEngineMapping {
    pub symbol: String,
    pub market_id: String,
    pub engine_id: String,
    pub matching_node: String,
    pub status: ServiceStatus,
}

#[derive(Clone, Debug)]
pub struct FailoverTarget {
    pub primary_service_id: String,
    pub backup_service_id: String,
    pub mode: String,
}

#[derive(Clone, Debug)]
pub struct ConfigSnapshot {
    pub config_version: String,
    pub feature_flags: HashMap<String, bool>,
    pub rate_limits: HashMap<String, u64>,
    pub allowlist: Vec<String>,
    pub thresholds: HashMap<String, u64>,
}

#[derive(Clone, Debug)]
pub struct ClientSocketConn {
    pub connection_id: String,
    pub end_user_id: i64,
    pub connected_at_ms: i64,
}

#[derive(Debug, Default)]
pub struct GatewayMemoryMap {
    pub service_registry: HashMap<String, ServiceNode>,
    pub service_health: HashMap<String, ServiceStatus>,
    pub route_map: HashMap<String, RouteEntry>,
    pub connection_pool: HashMap<String, ConnectionPoolInfo>,
    pub market_engine_map: HashMap<String, MarketEngineMapping>,
    pub failover_map: HashMap<String, FailoverTarget>,
    pub config_snapshot: Option<ConfigSnapshot>,
    pub client_sockets: HashMap<String, ClientSocketConn>,
    /// Other Gateway instances from Config listing + local socket PING.
    pub gateway_peers: HashMap<String, GatewayPeer>,
    pub self_instance_id: Option<String>,
}

impl GatewayMemoryMap {
    pub fn seed_static_routes(&mut self) {
        let routes = [
            ("client", "/v1/client", "client_center"),
            ("shop", "/v1/shop", "client_center"),
            ("corp", "/v1/corp", "client_center"),
            ("corp_tokens", "/v1/corp-tokens", "client_center"),
            ("markets", "/v1/markets", "client_center"),
            ("orders", "/v1/orders", "client_center"),
            ("marketplace", "/v1/marketplace", "client_center"),
            ("oauth", "/v1/oauth", "client_center"),
            ("session_token", "/v1/session/token", "client_center"),
            ("notifications", "/v1/notifications", "message_center"),
            ("admin", "/v1/admin", "admin_api"),
        ];
        for (key, path, target) in routes {
            self.route_map.insert(
                key.into(),
                RouteEntry {
                    route_key: key.into(),
                    path: path.into(),
                    target_service: target.into(),
                    route_policy: "health_rr".into(),
                    timeout_ms: 15_000,
                    retry_count: 0,
                    enabled: true,
                },
            );
        }
    }

    /// Round-robin pick: prefer `Healthy`, else `Degraded`; skip Unhealthy/Offline.
    /// When a role has a `failover_map` entry and the primary is down, prefer the backup.
    pub fn pick_healthy(&self, role: ServiceRole, counter: &AtomicUsize) -> Option<&ServiceNode> {
        if let Some(node) = self.pick_failover(role) {
            return Some(node);
        }
        let healthy: Vec<&ServiceNode> = self
            .service_registry
            .values()
            .filter(|n| n.role == role && n.status == ServiceStatus::Healthy)
            .collect();
        let pool = if !healthy.is_empty() {
            healthy
        } else {
            self.service_registry
                .values()
                .filter(|n| n.role == role && n.status == ServiceStatus::Degraded)
                .collect()
        };
        if pool.is_empty() {
            return None;
        }
        let idx = counter.fetch_add(1, Ordering::Relaxed);
        Some(pool[idx % pool.len()])
    }

    /// Only intervenes when the configured primary is down — then return backup if up.
    /// When primary is healthy/degraded, return `None` so normal RR continues.
    fn pick_failover(&self, role: ServiceRole) -> Option<&ServiceNode> {
        let fo = self.failover_map.values().find(|f| {
            self.service_registry
                .get(&f.primary_service_id)
                .map(|n| n.role == role)
                .unwrap_or(false)
                || self
                    .service_registry
                    .get(&f.backup_service_id)
                    .map(|n| n.role == role)
                    .unwrap_or(false)
        })?;
        let primary = self.service_registry.get(&fo.primary_service_id);
        let primary_ok = primary
            .map(|n| {
                matches!(
                    n.status,
                    ServiceStatus::Healthy | ServiceStatus::Degraded
                )
            })
            .unwrap_or(false);
        if primary_ok {
            return None;
        }
        let backup = self.service_registry.get(&fo.backup_service_id)?;
        if matches!(
            backup.status,
            ServiceStatus::Healthy | ServiceStatus::Degraded
        ) {
            tracing::info!(
                role = ?role,
                primary = %fo.primary_service_id,
                backup = %fo.backup_service_id,
                "gateway failover: using backup upstream"
            );
            return Some(backup);
        }
        None
    }

    /// Rebuild same-role primary/backup pairs (sorted by service_id).
    /// Includes Gateway peers for mesh failover awareness.
    pub fn rebuild_failover_map(&mut self) {
        self.failover_map.clear();
        let mut by_role: HashMap<ServiceRole, Vec<String>> = HashMap::new();
        for (id, node) in &self.service_registry {
            if matches!(node.role, ServiceRole::Unknown | ServiceRole::Config) {
                continue;
            }
            by_role.entry(node.role).or_default().push(id.clone());
        }
        for (role, mut ids) in by_role {
            ids.sort();
            if ids.len() < 2 {
                continue;
            }
            let key = format!("{:?}", role).to_ascii_lowercase();
            self.failover_map.insert(
                key,
                FailoverTarget {
                    primary_service_id: ids[0].clone(),
                    backup_service_id: ids[1].clone(),
                    mode: "active_passive".into(),
                },
            );
        }
    }

    pub fn gateway_peer_summary(&self) -> Vec<serde_json::Value> {
        self.gateway_peers
            .values()
            .map(|p| {
                serde_json::json!({
                    "instance_id": p.instance_id,
                    "http_url": p.http_url,
                    "socket_url": p.socket_url,
                    "status": format!("{:?}", p.status).to_ascii_lowercase(),
                    "last_ping_ok_ms": p.last_ping_ok_ms,
                    "last_error": p.last_error,
                })
            })
            .collect()
    }

    /// Summary of upstream roles for `/health` detail (status from socket heartbeat).
    pub fn upstream_role_summary(&self) -> Vec<serde_json::Value> {
        use std::collections::BTreeMap;
        let mut counts: BTreeMap<&'static str, (usize, usize, usize, usize)> = BTreeMap::new();
        for n in self.service_registry.values() {
            let label = match n.role {
                ServiceRole::ClientCenter => "client_center",
                ServiceRole::MessageCenter => "message_center",
                ServiceRole::AdminApi => "admin_api",
                ServiceRole::SessionToken => "session_token",
                ServiceRole::Webhook => "webhook",
                ServiceRole::CoreEngine => "core_engine",
                ServiceRole::Config => "config",
                ServiceRole::Gateway => "gateway",
                ServiceRole::Unknown => "unknown",
            };
            let e = counts.entry(label).or_insert((0, 0, 0, 0));
            e.0 += 1;
            match n.status {
                ServiceStatus::Healthy => e.1 += 1,
                ServiceStatus::Degraded => e.2 += 1,
                ServiceStatus::Unhealthy | ServiceStatus::Offline => e.3 += 1,
            }
        }
        counts
            .into_iter()
            .map(|(role, (total, healthy, degraded, down))| {
                serde_json::json!({
                    "role": role,
                    "total": total,
                    "healthy": healthy,
                    "degraded": degraded,
                    "down": down,
                })
            })
            .collect()
    }

    pub fn any_critical_upstream_down(&self) -> bool {
        for role in [
            ServiceRole::ClientCenter,
            ServiceRole::SessionToken,
        ] {
            let has_ok = self.service_registry.values().any(|n| {
                n.role == role
                    && matches!(
                        n.status,
                        ServiceStatus::Healthy | ServiceStatus::Degraded
                    )
            });
            if !has_ok && self.service_registry.values().any(|n| n.role == role) {
                return true;
            }
        }
        false
    }
}

pub type SharedGatewayMemory = Arc<RwLock<GatewayMemoryMap>>;

pub fn new_shared_memory() -> SharedGatewayMemory {
    let mut map = GatewayMemoryMap::default();
    map.seed_static_routes();
    Arc::new(RwLock::new(map))
}

pub fn role_from_service_name(name: &str) -> ServiceRole {
    let n = name.to_ascii_lowercase();
    if n.contains("client-center") || n.contains("client_center") {
        ServiceRole::ClientCenter
    } else if n.contains("message-center") || n.contains("message_center") {
        ServiceRole::MessageCenter
    } else if n.contains("admin") {
        ServiceRole::AdminApi
    } else if n.contains("session") || n.contains("token") {
        ServiceRole::SessionToken
    } else if n.contains("config") {
        ServiceRole::Config
    } else if n.contains("webhook") {
        ServiceRole::Webhook
    } else if n.contains("core") || n.contains("engine") {
        ServiceRole::CoreEngine
    } else if n.contains("gateway") {
        ServiceRole::Gateway
    } else {
        ServiceRole::Unknown
    }
}

pub fn status_from_str(s: &str) -> ServiceStatus {
    match s.to_ascii_lowercase().as_str() {
        "healthy" | "ok" | "up" | "active" => ServiceStatus::Healthy,
        "degraded" => ServiceStatus::Degraded,
        "unhealthy" | "down" => ServiceStatus::Unhealthy,
        "offline" => ServiceStatus::Offline,
        _ => ServiceStatus::Degraded,
    }
}

pub fn parse_http_url(url: &str) -> (String, u16) {
    let trimmed = url.trim().trim_end_matches('/');
    let without = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    let hostport = without.split('/').next().unwrap_or(without);
    if let Some((host, port_s)) = hostport.rsplit_once(':') {
        let port = port_s.parse().unwrap_or(80);
        (host.to_string(), port)
    } else {
        (hostport.to_string(), 80)
    }
}
