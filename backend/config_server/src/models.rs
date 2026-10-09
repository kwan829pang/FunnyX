use std::collections::HashMap;

use funnyx_heartbeat::http::HttpHeartbeatRequest;
use funnyx_heartbeat::HeartbeatTiming;
use serde::{Deserialize, Serialize};

pub type HeartbeatConfig = HeartbeatTiming;
pub type HeartbeatRequest = HttpHeartbeatRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistFile {
    pub server_name: String,
    #[serde(default)]
    pub allowlist: Vec<String>,
    #[serde(default)]
    pub routes: HashMap<String, RouteRule>,
    #[serde(default)]
    pub heartbeat: HeartbeatConfig,
    #[serde(default)]
    pub notes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub allowlist_only: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistSnapshot {
    pub path: String,
    pub loaded_at_ms: i64,
    pub reload_count: u64,
    pub server_name: String,
    pub allowlist: Vec<String>,
    pub routes: HashMap<String, RouteRule>,
    pub heartbeat: HeartbeatConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReloadResponse {
    pub reloaded: bool,
    pub path: String,
    pub loaded_at_ms: i64,
    pub reload_count: u64,
    pub allowlist_len: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterServiceRequest {
    pub service_name: String,
    pub instance_id: Option<String>,
    pub http_url: String,
    #[serde(default)]
    pub socket_url: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub instance_id: String,
    pub service_name: String,
    pub http_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    pub status: String,
    pub registered_at_ms: i64,
    pub last_heartbeat_ms: i64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceListResponse {
    pub services: Vec<ServiceInstance>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceStatusResponse {
    pub devices: Vec<DeviceStatus>,
    pub heartbeat_timeout_ms: i64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceStatus {
    pub instance_id: String,
    pub service_name: String,
    pub http_url: String,
    pub status: String,
    pub healthy: bool,
    pub last_heartbeat_ms: i64,
    pub age_ms: i64,
}
