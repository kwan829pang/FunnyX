//! Session storage: in-memory (dev) or Redis (`token:{id}`, `session:{user_id}`, `refresh:{id}`).

use std::collections::HashMap;
use std::sync::Arc;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use tokio::sync::Mutex;

use crate::models::SessionRecord;
use crate::token::{is_expired, now_ms};

#[derive(Clone)]
pub enum SessionStore {
    Memory(Arc<Mutex<MemoryInner>>),
    Redis(RedisStore),
}

#[derive(Default)]
pub struct MemoryInner {
    by_access: HashMap<String, SessionRecord>,
    by_refresh: HashMap<String, String>,
    by_user: HashMap<i64, String>,
}

#[derive(Clone)]
pub struct RedisStore {
    conn: ConnectionManager,
}

impl SessionStore {
    pub async fn memory() -> Self {
        Self::Memory(Arc::new(Mutex::new(MemoryInner::default())))
    }

    pub async fn redis(url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self::Redis(RedisStore { conn }))
    }

    pub async fn put(&self, record: SessionRecord) -> anyhow::Result<()> {
        match self {
            Self::Memory(inner) => {
                let mut g = inner.lock().await;
                if let Some(old) = g.by_user.get(&record.end_user_id).cloned() {
                    if let Some(old_rec) = g.by_access.remove(&old) {
                        g.by_refresh.remove(&old_rec.refresh_token);
                    }
                }
                g.by_refresh
                    .insert(record.refresh_token.clone(), record.access_token.clone());
                g.by_user
                    .insert(record.end_user_id, record.access_token.clone());
                g.by_access
                    .insert(record.access_token.clone(), record);
                Ok(())
            }
            Self::Redis(store) => store.put(record).await,
        }
    }

    pub async fn get_by_access(&self, access_token: &str) -> anyhow::Result<Option<SessionRecord>> {
        match self {
            Self::Memory(inner) => {
                let mut g = inner.lock().await;
                let Some(rec) = g.by_access.get(access_token).cloned() else {
                    return Ok(None);
                };
                if is_expired(rec.expires_at_ms) {
                    g.by_refresh.remove(&rec.refresh_token);
                    g.by_user.remove(&rec.end_user_id);
                    g.by_access.remove(access_token);
                    return Ok(None);
                }
                Ok(Some(rec))
            }
            Self::Redis(store) => store.get_by_access(access_token).await,
        }
    }

    pub async fn get_by_refresh(
        &self,
        refresh_token: &str,
    ) -> anyhow::Result<Option<SessionRecord>> {
        match self {
            Self::Memory(inner) => {
                let access = {
                    let g = inner.lock().await;
                    g.by_refresh.get(refresh_token).cloned()
                };
                match access {
                    Some(access) => self.get_by_access(&access).await,
                    None => Ok(None),
                }
            }
            Self::Redis(store) => store.get_by_refresh(refresh_token).await,
        }
    }

    pub async fn revoke_access(&self, access_token: &str) -> anyhow::Result<bool> {
        match self {
            Self::Memory(inner) => {
                let mut g = inner.lock().await;
                let Some(rec) = g.by_access.remove(access_token) else {
                    return Ok(false);
                };
                g.by_refresh.remove(&rec.refresh_token);
                if g.by_user.get(&rec.end_user_id).map(|s| s.as_str()) == Some(access_token) {
                    g.by_user.remove(&rec.end_user_id);
                }
                Ok(true)
            }
            Self::Redis(store) => store.revoke_access(access_token).await,
        }
    }

    pub async fn revoke_refresh(&self, refresh_token: &str) -> anyhow::Result<bool> {
        match self {
            Self::Memory(inner) => {
                let access = {
                    let g = inner.lock().await;
                    g.by_refresh.get(refresh_token).cloned()
                };
                match access {
                    Some(access) => self.revoke_access(&access).await,
                    None => Ok(false),
                }
            }
            Self::Redis(store) => store.revoke_refresh(refresh_token).await,
        }
    }
}

impl RedisStore {
    fn token_key(token_id: &str) -> String {
        format!("token:{token_id}")
    }

    fn session_key(user_id: i64) -> String {
        format!("session:{user_id}")
    }

    fn refresh_key(refresh: &str) -> String {
        format!("refresh:{refresh}")
    }

    async fn put(&self, record: SessionRecord) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        let ttl_access = ((record.expires_at_ms - now_ms()).max(1000) / 1000) as u64;
        let ttl_refresh = ((record.refresh_expires_at_ms - now_ms()).max(1000) / 1000) as u64;
        let body = serde_json::to_string(&record)?;

        // Drop previous session for this user if present.
        if let Ok(Some(old_token)) = conn
            .get::<_, Option<String>>(Self::session_key(record.end_user_id))
            .await
        {
            if old_token != record.access_token {
                let _: () = conn.del(Self::token_key(&old_token)).await.unwrap_or(());
            }
        }

        let _: () = conn
            .set_ex(Self::token_key(&record.access_token), body, ttl_access)
            .await?;
        let _: () = conn
            .set_ex(
                Self::session_key(record.end_user_id),
                &record.access_token,
                ttl_access,
            )
            .await?;
        let _: () = conn
            .set_ex(
                Self::refresh_key(&record.refresh_token),
                &record.access_token,
                ttl_refresh,
            )
            .await?;
        Ok(())
    }

    async fn get_by_access(&self, access_token: &str) -> anyhow::Result<Option<SessionRecord>> {
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn.get(Self::token_key(access_token)).await?;
        let Some(raw) = raw else {
            return Ok(None);
        };
        let rec: SessionRecord = serde_json::from_str(&raw)?;
        if is_expired(rec.expires_at_ms) {
            let _ = self.revoke_access(access_token).await;
            return Ok(None);
        }
        Ok(Some(rec))
    }

    async fn get_by_refresh(&self, refresh_token: &str) -> anyhow::Result<Option<SessionRecord>> {
        let mut conn = self.conn.clone();
        let access: Option<String> = conn.get(Self::refresh_key(refresh_token)).await?;
        match access {
            Some(access) => {
                let rec = self.get_by_access(&access).await?;
                if let Some(ref r) = rec {
                    if is_expired(r.refresh_expires_at_ms) {
                        let _ = self.revoke_access(&access).await;
                        return Ok(None);
                    }
                }
                Ok(rec)
            }
            None => Ok(None),
        }
    }

    async fn revoke_access(&self, access_token: &str) -> anyhow::Result<bool> {
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn.get(Self::token_key(access_token)).await?;
        let Some(raw) = raw else {
            return Ok(false);
        };
        let rec: SessionRecord = serde_json::from_str(&raw)?;
        let _: () = conn.del(Self::token_key(access_token)).await?;
        let _: () = conn.del(Self::refresh_key(&rec.refresh_token)).await?;
        let current: Option<String> = conn.get(Self::session_key(rec.end_user_id)).await?;
        if current.as_deref() == Some(access_token) {
            let _: () = conn.del(Self::session_key(rec.end_user_id)).await?;
        }
        Ok(true)
    }

    async fn revoke_refresh(&self, refresh_token: &str) -> anyhow::Result<bool> {
        let mut conn = self.conn.clone();
        let access: Option<String> = conn.get(Self::refresh_key(refresh_token)).await?;
        match access {
            Some(access) => self.revoke_access(&access).await,
            None => Ok(false),
        }
    }
}
