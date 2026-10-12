//! Admin store for `fx_events.webhook_post_auth`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPostAuthView {
    pub id: i64,
    pub source_code: String,
    pub domain_pattern: String,
    pub auth_key_suffix: String,
    pub ppk: Option<String>,
    pub allowed_ips: String,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone)]
pub struct WebhookPostAuthStore {
    pool: Option<PgPool>,
    memory: Option<Arc<Memory>>,
}

struct Memory {
    by_id: RwLock<HashMap<i64, WebhookPostAuthView>>,
    next_id: AtomicI64,
}

impl WebhookPostAuthStore {
    pub fn new(pool: Option<PgPool>) -> Self {
        if pool.is_some() {
            Self { pool, memory: None }
        } else {
            let mut by_id = HashMap::new();
            by_id.insert(
                1,
                WebhookPostAuthView {
                    id: 1,
                    source_code: "payment_gate".into(),
                    domain_pattern: "payment-gate.local".into(),
                    auth_key_suffix: "demo_key".into(),
                    ppk: Some("ppk_payment_gate".into()),
                    allowed_ips: "127.0.0.1,::1".into(),
                    status: "active".into(),
                    created_at: 1,
                    updated_at: 0,
                },
            );
            by_id.insert(
                2,
                WebhookPostAuthView {
                    id: 2,
                    source_code: "partner_2001".into(),
                    domain_pattern: "*.company-a.local".into(),
                    auth_key_suffix: "demo_key".into(),
                    ppk: Some("ppk_partner_2001".into()),
                    allowed_ips: "127.0.0.1,::1".into(),
                    status: "active".into(),
                    created_at: 1,
                    updated_at: 0,
                },
            );
            Self {
                pool: None,
                memory: Some(Arc::new(Memory {
                    by_id: RwLock::new(by_id),
                    next_id: AtomicI64::new(3),
                })),
            }
        }
    }

    pub async fn list(&self, status: Option<&str>) -> Result<Vec<WebhookPostAuthView>, String> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(crate::query::webhook_post_auth::LIST)
                .bind(status)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;
            return Ok(rows.into_iter().map(map_row).collect());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut rows: Vec<_> = mem.by_id.read().await.values().cloned().collect();
        if let Some(s) = status {
            rows.retain(|r| r.status == s);
        }
        rows.sort_by_key(|r| r.id);
        Ok(rows)
    }

    pub async fn get(&self, id: i64) -> Result<Option<WebhookPostAuthView>, String> {
        if let Some(pool) = &self.pool {
            return sqlx::query(crate::query::webhook_post_auth::GET)
                .bind(id)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())
                .map(|o| o.map(map_row));
        }
        Ok(self
            .memory
            .as_ref()
            .ok_or_else(|| "webhook_post_auth store unavailable".to_string())?
            .by_id
            .read()
            .await
            .get(&id)
            .cloned())
    }

    pub async fn create(
        &self,
        source_code: &str,
        domain_pattern: &str,
        auth_key_suffix: &str,
        ppk: Option<&str>,
        allowed_ips: &str,
        status: &str,
    ) -> Result<WebhookPostAuthView, String> {
        self.ensure_unique_source(source_code, None).await?;
        if let Some(p) = ppk.filter(|s| !s.is_empty()) {
            self.ensure_unique_ppk(p, None).await?;
        }
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let row = sqlx::query(crate::query::webhook_post_auth::INSERT)
                .bind(source_code)
                .bind(domain_pattern)
                .bind(auth_key_suffix)
                .bind(ppk)
                .bind(allowed_ips)
                .bind(status)
                .bind(now)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
            return Ok(map_row(row));
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let id = mem.next_id.fetch_add(1, Ordering::Relaxed);
        let view = WebhookPostAuthView {
            id,
            source_code: source_code.into(),
            domain_pattern: domain_pattern.into(),
            auth_key_suffix: auth_key_suffix.into(),
            ppk: ppk.filter(|s| !s.is_empty()).map(|s| s.to_string()),
            allowed_ips: allowed_ips.into(),
            status: status.into(),
            created_at: now,
            updated_at: 0,
        };
        mem.by_id.write().await.insert(id, view.clone());
        Ok(view)
    }

    pub async fn update(
        &self,
        id: i64,
        source_code: &str,
        domain_pattern: &str,
        auth_key_suffix: &str,
        ppk: Option<&str>,
        allowed_ips: &str,
    ) -> Result<WebhookPostAuthView, String> {
        self.ensure_unique_source(source_code, Some(id)).await?;
        if let Some(p) = ppk.filter(|s| !s.is_empty()) {
            self.ensure_unique_ppk(p, Some(id)).await?;
        }
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let row = sqlx::query(crate::query::webhook_post_auth::UPDATE)
                .bind(id)
                .bind(source_code)
                .bind(domain_pattern)
                .bind(auth_key_suffix)
                .bind(ppk)
                .bind(allowed_ips)
                .bind(now)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "not found".to_string())?;
            return Ok(map_row(row));
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut guard = mem.by_id.write().await;
        let rec = guard.get_mut(&id).ok_or_else(|| "not found".to_string())?;
        rec.source_code = source_code.into();
        rec.domain_pattern = domain_pattern.into();
        rec.auth_key_suffix = auth_key_suffix.into();
        rec.ppk = ppk.filter(|s| !s.is_empty()).map(|s| s.to_string());
        rec.allowed_ips = allowed_ips.into();
        rec.updated_at = now;
        Ok(rec.clone())
    }

    pub async fn set_status(&self, id: i64, status: &str) -> Result<WebhookPostAuthView, String> {
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let row = sqlx::query(crate::query::webhook_post_auth::UPDATE_STATUS)
                .bind(id)
                .bind(status)
                .bind(now)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "not found".to_string())?;
            return Ok(map_row(row));
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut guard = mem.by_id.write().await;
        let rec = guard.get_mut(&id).ok_or_else(|| "not found".to_string())?;
        if status == "active" {
            if let Some(p) = rec.ppk.clone() {
                drop(guard);
                self.ensure_unique_ppk(&p, Some(id)).await?;
                let mut guard = mem.by_id.write().await;
                let rec = guard.get_mut(&id).ok_or_else(|| "not found".to_string())?;
                rec.status = status.into();
                rec.updated_at = now;
                return Ok(rec.clone());
            }
        }
        rec.status = status.into();
        rec.updated_at = now;
        Ok(rec.clone())
    }

    async fn ensure_unique_source(
        &self,
        source_code: &str,
        exclude_id: Option<i64>,
    ) -> Result<(), String> {
        if let Some(pool) = &self.pool {
            let exists: bool = sqlx::query_scalar(crate::query::webhook_post_auth::EXISTS_SOURCE)
                .bind(source_code)
                .bind(exclude_id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
            if exists {
                return Err("source_code already exists".into());
            }
            return Ok(());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let dup = mem.by_id.read().await.values().any(|r| {
            r.source_code == source_code && exclude_id.map(|id| r.id != id).unwrap_or(true)
        });
        if dup {
            return Err("source_code already exists".into());
        }
        Ok(())
    }

    async fn ensure_unique_ppk(
        &self,
        ppk: &str,
        exclude_id: Option<i64>,
    ) -> Result<(), String> {
        if let Some(pool) = &self.pool {
            let exists: bool =
                sqlx::query_scalar(crate::query::webhook_post_auth::EXISTS_PPK_ACTIVE)
                    .bind(ppk)
                    .bind(exclude_id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            if exists {
                return Err("ppk already used by an active row".into());
            }
            return Ok(());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let dup = mem.by_id.read().await.values().any(|r| {
            r.status == "active"
                && r.ppk.as_deref() == Some(ppk)
                && exclude_id.map(|id| r.id != id).unwrap_or(true)
        });
        if dup {
            return Err("ppk already used by an active row".into());
        }
        Ok(())
    }
}

fn map_row(r: sqlx::postgres::PgRow) -> WebhookPostAuthView {
    WebhookPostAuthView {
        id: r.get("id"),
        source_code: r.get("source_code"),
        domain_pattern: r.get("domain_pattern"),
        auth_key_suffix: r.get("auth_key_suffix"),
        ppk: r.try_get("ppk").ok(),
        allowed_ips: r.get("allowed_ips"),
        status: r.get("status"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }
}
