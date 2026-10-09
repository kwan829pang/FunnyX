use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::models::{ServiceInstance, WhitelistFile, WhitelistSnapshot};

#[derive(Clone)]
pub struct ConfigStore {
    inner: Arc<RwLock<StoreInner>>,
}

struct StoreInner {
    path: PathBuf,
    file: WhitelistFile,
    loaded_at_ms: i64,
    reload_count: u64,
    services: HashMap<String, ServiceInstance>,
}

impl ConfigStore {
    pub async fn load(path: PathBuf) -> anyhow::Result<Self> {
        let file = load_file(&path)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(StoreInner {
                path,
                file,
                loaded_at_ms: now_ms(),
                reload_count: 0,
                services: HashMap::new(),
            })),
        })
    }

    pub async fn reload(&self) -> anyhow::Result<WhitelistSnapshot> {
        let mut inner = self.inner.write().await;
        let file = load_file(&inner.path)?;
        inner.file = file;
        inner.loaded_at_ms = now_ms();
        inner.reload_count += 1;
        Ok(snapshot(&inner))
    }

    pub async fn whitelist_snapshot(&self) -> WhitelistSnapshot {
        let inner = self.inner.read().await;
        snapshot(&inner)
    }

    pub async fn register(&self, inst: ServiceInstance) -> ServiceInstance {
        let mut inner = self.inner.write().await;
        inner
            .services
            .insert(inst.instance_id.clone(), inst.clone());
        inst
    }

    pub async fn heartbeat(
        &self,
        instance_id: &str,
        status: Option<String>,
    ) -> Option<ServiceInstance> {
        let mut inner = self.inner.write().await;
        let rec = inner.services.get_mut(instance_id)?;
        rec.last_heartbeat_ms = now_ms();
        if let Some(s) = status.filter(|s| !s.is_empty()) {
            rec.status = s;
        } else {
            rec.status = "up".into();
        }
        Some(rec.clone())
    }

    /// Update status without refreshing `last_heartbeat_ms` (failed probe).
    pub async fn set_status(&self, instance_id: &str, status: String) -> Option<ServiceInstance> {
        let mut inner = self.inner.write().await;
        let rec = inner.services.get_mut(instance_id)?;
        rec.status = status;
        Some(rec.clone())
    }

    pub async fn list_services(&self) -> Vec<ServiceInstance> {
        let inner = self.inner.read().await;
        let mut v: Vec<_> = inner.services.values().cloned().collect();
        v.sort_by(|a, b| a.service_name.cmp(&b.service_name).then(a.instance_id.cmp(&b.instance_id)));
        v
    }

    pub async fn allowlist_contains(&self, ip: &str) -> bool {
        let inner = self.inner.read().await;
        inner.file.allowlist.iter().any(|e| e == ip || e == "*")
    }
}

fn snapshot(inner: &StoreInner) -> WhitelistSnapshot {
    WhitelistSnapshot {
        path: inner.path.display().to_string(),
        loaded_at_ms: inner.loaded_at_ms,
        reload_count: inner.reload_count,
        server_name: inner.file.server_name.clone(),
        allowlist: inner.file.allowlist.clone(),
        routes: inner.file.routes.clone(),
        heartbeat: inner.file.heartbeat.clone(),
    }
}

fn load_file(path: &Path) -> anyhow::Result<WhitelistFile> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("read whitelist {}: {e}", path.display()))?;
    serde_json::from_str(&raw)
        .map_err(|e| anyhow::anyhow!("parse whitelist {}: {e}", path.display()))
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
