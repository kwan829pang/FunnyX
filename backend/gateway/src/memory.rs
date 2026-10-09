//! In-memory Gateway runtime map ([doc/gateway_core_engine_logic.md] §2.7).

#![allow(dead_code)] // Reserved map tables for LB / failover / market routing.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::sync::RwLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub protocol: String,
    pub region: String,
    pub status: ServiceStatus,
    pub last_heartbeat_ms: u64,
    pub uptime_ms: u64,
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

    /// Round-robin pick among healthy nodes for `role`.
    pub fn pick_healthy(&self, role: ServiceRole, counter: &AtomicUsize) -> Option<&ServiceNode> {
        let healthy: Vec<&ServiceNode> = self
            .service_registry
            .values()
            .filter(|n| n.role == role && n.status == ServiceStatus::Healthy)
            .collect();
        if healthy.is_empty() {
            return None;
        }
        let idx = counter.fetch_add(1, Ordering::Relaxed);
        Some(healthy[idx % healthy.len()])
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
