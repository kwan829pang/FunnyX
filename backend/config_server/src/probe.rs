//! Config Server **private** socket heartbeat loop (in-system only).
//!
//! Probes each registered `socket_url` with PING→PONG.
//! HTTP `GET /health` is **not** used here — that endpoint is public for
//! third parties (Cloudflare, Game Partners) via `funnyx-health`.

use std::time::Duration;

use funnyx_heartbeat::HeartbeatTiming;
use tokio::time::sleep;
use tracing::{debug, warn};

use crate::store::ConfigStore;

pub fn spawn_registry_socket_probe(store: ConfigStore, timing: HeartbeatTiming) {
    tokio::spawn(async move {
        let interval = Duration::from_millis(timing.interval_ms.max(1_000));
        let timeout_ms = timing.timeout_ms.max(500);
        loop {
            let services = store.list_services().await;
            for inst in services {
                let Some(sock) = inst.socket_url.as_ref().filter(|s| !s.trim().is_empty()) else {
                    debug!(
                        instance = %inst.instance_id,
                        "skip probe: no socket_url"
                    );
                    continue;
                };
                match funnyx_heartbeat::ping_peer(sock, timeout_ms).await {
                    Ok(()) => {
                        let _ = store
                            .heartbeat(&inst.instance_id, Some("up".into()))
                            .await;
                    }
                    Err(e) => {
                        warn!(
                            instance = %inst.instance_id,
                            socket = %sock,
                            error = %e,
                            "socket heartbeat failed"
                        );
                        let _ = store
                            .set_status(&inst.instance_id, "down".into())
                            .await;
                    }
                }
            }
            sleep(interval).await;
        }
    });
}
