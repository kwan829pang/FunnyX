//! Demo admin directory (seed-aligned) until PostgreSQL fx_admin is wired.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub admin_user_id: i64,
    pub username: String,
    pub password: String,
    pub role: String,
    pub status: String,
}

#[derive(Clone)]
pub struct AdminDirectory {
    by_username: Arc<RwLock<HashMap<String, AdminUser>>>,
    by_id: Arc<RwLock<HashMap<i64, String>>>,
}

impl AdminDirectory {
    /// Matches `database/seed/00_seed_data.sql` username `seed_admin` (+ demo alias).
    pub fn with_demo_admins() -> Self {
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
        ] {
            by_id.insert(u.admin_user_id, u.username.clone());
            by_username.insert(u.username.clone(), u);
        }
        Self {
            by_username: Arc::new(RwLock::new(by_username)),
            by_id: Arc::new(RwLock::new(by_id)),
        }
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Option<AdminUser> {
        let users = self.by_username.read().await;
        users
            .get(username)
            .filter(|u| u.password == password && u.status == "active")
            .cloned()
    }

    pub async fn get_by_id(&self, admin_user_id: i64) -> Option<AdminUser> {
        let by_id = self.by_id.read().await;
        let username = by_id.get(&admin_user_id)?.clone();
        self.by_username.read().await.get(&username).cloned()
    }
}
