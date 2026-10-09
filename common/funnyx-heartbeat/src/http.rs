//! HTTP bodies for Config Server registry (register). Heartbeat core is socket.

use serde::{Deserialize, Serialize};

/// POST `/v1/config/services`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterServiceRequest {
    pub service_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    pub http_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

/// POST `/v1/config/services/heartbeat`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHeartbeatRequest {
    pub instance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Subset of Config Server `ServiceInstance` used by clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstanceView {
    pub instance_id: String,
    pub service_name: String,
    pub http_url: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub last_heartbeat_ms: i64,
}
