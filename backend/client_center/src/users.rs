//! End-user directory: PostgreSQL (`fx_user.end_users`) or in-memory fallback.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

use crate::db::now_ms;
use crate::password::{hash_password, verify_password};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndUser {
    pub end_user_id: i64,
    pub username: String,
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub email: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnerLink {
    pub partner_id: String,
    pub partner_user_id: String,
    pub end_user_id: i64,
    pub game_id: Option<String>,
    pub game_account_id: Option<String>,
}

#[derive(Clone)]
pub struct UserDirectory {
    pool: Option<PgPool>,
    memory: Option<MemoryUsers>,
}

#[derive(Clone)]
struct MemoryUsers {
    users: Arc<RwLock<HashMap<String, EndUser>>>,
    by_id: Arc<RwLock<HashMap<i64, String>>>,
    partner_links: Arc<RwLock<HashMap<(String, String), PartnerLink>>>,
    next_id: Arc<AtomicI64>,
}

impl UserDirectory {
    pub fn memory() -> anyhow::Result<Self> {
        let mut users = HashMap::new();
        let mut by_id = HashMap::new();
        for (id, username, email) in [
            (1_i64, "demo_user", "demo_user@example.local"),
            (2, "alice_plat", "alice_plat@example.local"),
        ] {
            let u = EndUser {
                end_user_id: id,
                username: username.into(),
                password_hash: Some(hash_password("demo").map_err(anyhow::Error::msg)?),
                email: Some(email.into()),
                status: "active".into(),
            };
            by_id.insert(u.end_user_id, u.username.clone());
            users.insert(u.username.clone(), u);
        }
        Ok(Self {
            pool: None,
            memory: Some(MemoryUsers {
                users: Arc::new(RwLock::new(users)),
                by_id: Arc::new(RwLock::new(by_id)),
                partner_links: Arc::new(RwLock::new(HashMap::new())),
                next_id: Arc::new(AtomicI64::new(100)),
            }),
        })
    }

    pub fn postgres(pool: PgPool) -> Self {
        Self {
            pool: Some(pool),
            memory: None,
        }
    }

    /// Fill Argon2 hashes for seeded demo users that still have NULL `password_hash`.
    pub async fn ensure_demo_password_hashes(&self) -> anyhow::Result<()> {
        let Some(pool) = &self.pool else {
            return Ok(());
        };
        let rows = sqlx::query(
            "SELECT id, username, password_hash FROM fx_user.end_users \
             WHERE username IN ('demo_user', 'alice_plat')",
        )
        .fetch_all(pool)
        .await?;
        for row in rows {
            let hash: Option<String> = row.get("password_hash");
            if hash.as_deref().filter(|s| !s.is_empty()).is_some() {
                continue;
            }
            let id: i64 = row.get("id");
            let hashed = hash_password("demo").map_err(|e| anyhow::anyhow!(e))?;
            sqlx::query(
                "UPDATE fx_user.end_users SET password_hash = $1, updated_at = $2 WHERE id = $3",
            )
            .bind(&hashed)
            .bind(now_ms())
            .bind(id)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Option<EndUser> {
        let user = self.get_by_username(username).await?;
        if user.status != "active" {
            return None;
        }
        let hash = user.password_hash.as_deref()?;
        if verify_password(password, hash) {
            Some(user)
        } else {
            None
        }
    }

    pub async fn get_by_id(&self, end_user_id: i64) -> Option<EndUser> {
        if let Some(pool) = &self.pool {
            return fetch_user(
                sqlx::query(
                    "SELECT id, username, email, password_hash, status \
                     FROM fx_user.end_users WHERE id = $1",
                )
                .bind(end_user_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten(),
            );
        }
        let mem = self.memory.as_ref()?;
        let by_id = mem.by_id.read().await;
        let username = by_id.get(&end_user_id)?.clone();
        mem.users.read().await.get(&username).cloned()
    }

    pub async fn get_by_username(&self, username: &str) -> Option<EndUser> {
        if let Some(pool) = &self.pool {
            return fetch_user(
                sqlx::query(
                    "SELECT id, username, email, password_hash, status \
                     FROM fx_user.end_users WHERE username = $1",
                )
                .bind(username)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten(),
            );
        }
        self.memory.as_ref()?.users.read().await.get(username).cloned()
    }

    pub async fn register(
        &self,
        username: String,
        password: String,
        email: Option<String>,
    ) -> Result<EndUser, String> {
        if username.trim().is_empty() || password.trim().is_empty() {
            return Err("username and password required".into());
        }
        let password_hash = hash_password(&password)?;
        if let Some(pool) = &self.pool {
            if self.get_by_username(&username).await.is_some() {
                return Err("username already exists".into());
            }
            let now = now_ms();
            let row = sqlx::query(
                "INSERT INTO fx_user.end_users (username, email, password_hash, status, created_at, updated_at) \
                 VALUES ($1, $2, $3, 'active', $4, 0) \
                 RETURNING id, username, email, password_hash, status",
            )
            .bind(&username)
            .bind(&email)
            .bind(&password_hash)
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(map_user(row));
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut users = mem.users.write().await;
        if users.contains_key(&username) {
            return Err("username already exists".into());
        }
        let end_user_id = mem.next_id.fetch_add(1, Ordering::Relaxed);
        let user = EndUser {
            end_user_id,
            username: username.clone(),
            password_hash: Some(password_hash),
            email,
            status: "active".into(),
        };
        mem.by_id
            .write()
            .await
            .insert(end_user_id, username.clone());
        users.insert(username, user.clone());
        Ok(user)
    }

    /// Upsert after STS Partner OAuth (STS may have assigned end_user_id).
    pub async fn upsert_from_session(
        &self,
        end_user_id: i64,
        username: Option<String>,
        partner_id: Option<String>,
        partner_user_id: Option<String>,
        game_account_id: Option<String>,
        game_id: Option<i64>,
    ) -> EndUser {
        let username = username
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("user_{end_user_id}"));

        if let Some(existing) = self.get_by_id(end_user_id).await {
            self.store_partner_link(
                partner_id.as_deref(),
                partner_user_id.as_deref(),
                existing.end_user_id,
                game_id,
                game_account_id.as_deref(),
            )
            .await;
            return existing;
        }

        if let Some(by_name) = self.get_by_username(&username).await {
            self.store_partner_link(
                partner_id.as_deref(),
                partner_user_id.as_deref(),
                by_name.end_user_id,
                game_id,
                game_account_id.as_deref(),
            )
            .await;
            return by_name;
        }

        let user = if let Some(pool) = &self.pool {
            let now = now_ms();
            match sqlx::query(
                "INSERT INTO fx_user.end_users \
                    (id, username, email, password_hash, status, created_at, updated_at) \
                 VALUES ($1, $2, NULL, NULL, 'active', $3, 0) \
                 ON CONFLICT (id) DO UPDATE SET updated_at = EXCLUDED.updated_at \
                 RETURNING id, username, email, password_hash, status",
            )
            .bind(end_user_id)
            .bind(&username)
            .bind(now)
            .fetch_one(pool)
            .await
            {
                Ok(r) => map_user(r),
                Err(_) => self
                    .get_by_id(end_user_id)
                    .await
                    .or(self.get_by_username(&username).await)
                    .unwrap_or(EndUser {
                        end_user_id,
                        username: username.clone(),
                        password_hash: None,
                        email: None,
                        status: "active".into(),
                    }),
            }
        } else {
            let user = EndUser {
                end_user_id,
                username: username.clone(),
                password_hash: None,
                email: None,
                status: "active".into(),
            };
            if let Some(mem) = &self.memory {
                mem.by_id
                    .write()
                    .await
                    .insert(end_user_id, username.clone());
                mem.users.write().await.insert(username.clone(), user.clone());
                let cur = mem.next_id.load(Ordering::Relaxed);
                if end_user_id >= cur {
                    mem.next_id.store(end_user_id + 1, Ordering::Relaxed);
                }
            }
            user
        };

        self.store_partner_link(
            partner_id.as_deref(),
            partner_user_id.as_deref(),
            user.end_user_id,
            game_id,
            game_account_id.as_deref(),
        )
        .await;
        user
    }

    async fn store_partner_link(
        &self,
        partner_id: Option<&str>,
        partner_user_id: Option<&str>,
        end_user_id: i64,
        game_id: Option<i64>,
        game_account_id: Option<&str>,
    ) {
        let (Some(pid), Some(puid)) = (partner_id, partner_user_id) else {
            return;
        };
        if pid.is_empty() || puid.is_empty() {
            return;
        }
        if let Some(pool) = &self.pool {
            let now = now_ms();
            let _ = sqlx::query(
                "INSERT INTO fx_user.oauth_identities \
                    (end_user_id, provider, partner_id, partner_user_id, game_id, game_account_id, status, created_at, updated_at) \
                 VALUES ($1, 'partner', $2, $3, $4, $5, 'active', $6, 0) \
                 ON CONFLICT (provider, partner_id, partner_user_id) DO UPDATE SET \
                    end_user_id = EXCLUDED.end_user_id, \
                    game_id = COALESCE(EXCLUDED.game_id, fx_user.oauth_identities.game_id), \
                    game_account_id = COALESCE(EXCLUDED.game_account_id, fx_user.oauth_identities.game_account_id), \
                    updated_at = $6",
            )
            .bind(end_user_id)
            .bind(pid)
            .bind(puid)
            .bind(game_id)
            .bind(game_account_id)
            .bind(now)
            .execute(pool)
            .await;
            return;
        }
        if let Some(mem) = &self.memory {
            let key = (pid.to_string(), puid.to_string());
            let link = PartnerLink {
                partner_id: pid.to_string(),
                partner_user_id: puid.to_string(),
                end_user_id,
                game_id: game_id.map(|g| g.to_string()),
                game_account_id: game_account_id.map(|s| s.to_string()),
            };
            mem.partner_links.write().await.insert(key, link);
        }
    }

    pub async fn links_for_user(&self, end_user_id: i64) -> Vec<PartnerLink> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT partner_id, partner_user_id, end_user_id, game_id, game_account_id \
                 FROM fx_user.oauth_identities \
                 WHERE end_user_id = $1 AND status = 'active'",
            )
            .bind(end_user_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows
                .into_iter()
                .map(|r| PartnerLink {
                    partner_id: r.get("partner_id"),
                    partner_user_id: r.get("partner_user_id"),
                    end_user_id: r.get("end_user_id"),
                    game_id: r
                        .get::<Option<i64>, _>("game_id")
                        .map(|id| id.to_string()),
                    game_account_id: r.get("game_account_id"),
                })
                .collect();
        }
        let Some(mem) = &self.memory else {
            return Vec::new();
        };
        mem.partner_links
            .read()
            .await
            .values()
            .filter(|l| l.end_user_id == end_user_id)
            .cloned()
            .collect()
    }

    pub async fn oauth_identity_id(
        &self,
        partner_id: &str,
        partner_user_id: &str,
    ) -> Option<i64> {
        let pool = self.pool.as_ref()?;
        sqlx::query_scalar(
            "SELECT id FROM fx_user.oauth_identities \
             WHERE provider = 'partner' AND partner_id = $1 AND partner_user_id = $2",
        )
        .bind(partner_id)
        .bind(partner_user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
    }
}

fn fetch_user(row: Option<sqlx::postgres::PgRow>) -> Option<EndUser> {
    row.map(map_user)
}

fn map_user(row: sqlx::postgres::PgRow) -> EndUser {
    EndUser {
        end_user_id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        status: row.get("status"),
    }
}
