//! Admin directory: demo memory + optional `fx_admin.admin_users` (Postgres).

use std::collections::HashMap;
use std::sync::Arc;

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use password_hash::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub admin_user_id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: String,
    pub status: String,
}

#[derive(Clone)]
pub struct AdminDirectory {
    by_username: Arc<RwLock<HashMap<String, AdminUser>>>,
    by_id: Arc<RwLock<HashMap<i64, String>>>,
    pool: Option<PgPool>,
}

impl AdminDirectory {
    /// Demo memory users + optional `fx_admin` Postgres (seed: `seed_admin` / `admin` / `viewer`).
    pub fn with_demo_and_pool(pool: Option<PgPool>) -> Self {
        let mut by_username = HashMap::new();
        let mut by_id = HashMap::new();
        for u in [
            AdminUser {
                admin_user_id: 1,
                username: "seed_admin".into(),
                password: "demo".into(),
                role: "super_admin".into(),
                status: "active".into(),
            },
            AdminUser {
                admin_user_id: 2,
                username: "admin".into(),
                password: "demo".into(),
                role: "admin".into(),
                status: "active".into(),
            },
            AdminUser {
                admin_user_id: 3,
                username: "viewer".into(),
                password: "demo".into(),
                role: "viewer".into(),
                status: "active".into(),
            },
        ] {
            by_id.insert(u.admin_user_id, u.username.clone());
            by_username.insert(u.username.clone(), u);
        }
        Self {
            by_username: Arc::new(RwLock::new(by_username)),
            by_id: Arc::new(RwLock::new(by_id)),
            pool,
        }
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Option<AdminUser> {
        if let Some(pool) = &self.pool {
            if let Ok(Some(row)) = sqlx::query(
                "SELECT id, username, role, status, password_hash \
                 FROM fx_admin.admin_users WHERE username = $1",
            )
            .bind(username)
            .fetch_optional(pool)
            .await
            {
                let status: String = row.get("status");
                if status != "active" {
                    return None;
                }
                let hash: Option<String> = row.get("password_hash");
                let ok = match hash.as_deref().filter(|s| !s.is_empty()) {
                    Some(h) => verify_password(password, h),
                    None => password == "demo",
                };
                if !ok {
                    return None;
                }
                if hash.as_deref().filter(|s| !s.is_empty()).is_none() {
                    if let Ok(new_hash) = hash_password(password) {
                        let _ = sqlx::query(
                            "UPDATE fx_admin.admin_users SET password_hash = $2, updated_at = $3 \
                             WHERE id = $1",
                        )
                        .bind(row.get::<i64, _>("id"))
                        .bind(&new_hash)
                        .bind(now_ms())
                        .execute(pool)
                        .await;
                    }
                }
                let user = AdminUser {
                    admin_user_id: row.get("id"),
                    username: row.get("username"),
                    password: String::new(),
                    role: row.get("role"),
                    status,
                };
                self.cache_user(&user).await;
                return Some(user);
            }
        }

        let users = self.by_username.read().await;
        users
            .get(username)
            .filter(|u| u.password == password && u.status == "active")
            .cloned()
    }

    pub async fn get_by_id(&self, admin_user_id: i64) -> Option<AdminUser> {
        if let Some(pool) = &self.pool {
            if let Ok(Some(row)) = sqlx::query(
                "SELECT id, username, role, status FROM fx_admin.admin_users WHERE id = $1",
            )
            .bind(admin_user_id)
            .fetch_optional(pool)
            .await
            {
                let user = AdminUser {
                    admin_user_id: row.get("id"),
                    username: row.get("username"),
                    password: String::new(),
                    role: row.get("role"),
                    status: row.get("status"),
                };
                self.cache_user(&user).await;
                return Some(user);
            }
        }
        let by_id = self.by_id.read().await;
        let username = by_id.get(&admin_user_id)?.clone();
        self.by_username.read().await.get(&username).cloned()
    }

    async fn cache_user(&self, user: &AdminUser) {
        let mut by_u = self.by_username.write().await;
        let mut by_i = self.by_id.write().await;
        by_i.insert(user.admin_user_id, user.username.clone());
        by_u.insert(user.username.clone(), user.clone());
    }
}

pub fn role_allowed(role: &str, allowed: &[&str]) -> bool {
    allowed.iter().any(|r| *r == role)
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
