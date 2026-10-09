//! HTTP register with Config Server (discovery only; still private/internal API).
//!
//! Liveness is **private** socket PING/PONG from Config Server (see [`crate::probe`]).
//! Public third-party checks use `funnyx-health` `GET /health` — not this module.

use std::time::Duration;

use funnyx_net_api::paths;
use tokio::time::sleep;
use tracing::{debug, warn};

use crate::http::{RegisterServiceRequest, ServiceInstanceView};
use crate::timing::HeartbeatTiming;

/// Options for [`spawn_registry_register`].
#[derive(Debug, Clone)]
pub struct RegistryHeartbeatOpts {
    pub config_server_url: String,
    pub internal_api_key: String,
    pub service_name: String,
    pub http_url: String,
    /// Required for Config Server socket probes (`tcp://host:port`).
    pub socket_url: Option<String>,
    pub group: Option<String>,
    pub instance_id: Option<String>,
    pub timing: HeartbeatTiming,
}

/// Register (and periodically re-register) with Config Server so the instance
/// appears in the registry with `socket_url`. Does **not** POST HTTP heartbeat;
/// Config Server probes via socket PING.
///
/// No-op if `config_server_url` is empty.
pub fn spawn_registry_register(http: reqwest::Client, opts: RegistryHeartbeatOpts) {
    let base = opts.config_server_url.trim().trim_end_matches('/').to_string();
    if base.is_empty() {
        debug!("funnyx-heartbeat: CONFIG_SERVER_URL unset; registry register disabled");
        return;
    }
    if opts.socket_url.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
        warn!(
            service = %opts.service_name,
            "registry register without socket_url; Config Server cannot socket-heartbeat this instance"
        );
    }
    tokio::spawn(async move {
        let mut instance_id = opts.instance_id.clone().filter(|s| !s.is_empty());
        let interval = Duration::from_millis(opts.timing.interval_ms.max(1_000));
        loop {
            match register(&http, &base, &opts, instance_id.as_deref()).await {
                Ok(inst) => {
                    instance_id = Some(inst.instance_id.clone());
                    debug!(
                        service = %opts.service_name,
                        instance = %inst.instance_id,
                        socket = ?opts.socket_url,
                        "registered with Config Server (socket heartbeat expected)"
                    );
                }
                Err(e) => {
                    warn!(error = %e, "Config Server register failed; will retry");
                }
            }
            sleep(interval).await;
        }
    });
}

/// Alias kept for call-site compatibility; same as [`spawn_registry_register`].
pub fn spawn_registry_heartbeat(http: reqwest::Client, opts: RegistryHeartbeatOpts) {
    spawn_registry_register(http, opts);
}

async fn register(
    http: &reqwest::Client,
    base: &str,
    opts: &RegistryHeartbeatOpts,
    instance_id: Option<&str>,
) -> Result<ServiceInstanceView, String> {
    let url = format!("{}{}", base, paths::CONFIG_SERVICES);
    let body = RegisterServiceRequest {
        service_name: opts.service_name.clone(),
        instance_id: instance_id.map(|s| s.to_string()).or_else(|| opts.instance_id.clone()),
        http_url: opts.http_url.clone(),
        socket_url: opts.socket_url.clone(),
        group: opts.group.clone(),
        metadata: serde_json::json!({}),
    };
    let res = http
        .post(&url)
        .header("x-internal-key", &opts.internal_api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("register {status}: {text}"));
    }
    serde_json::from_str(&text).map_err(|e| format!("register decode: {e} body={text}"))
}
