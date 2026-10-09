//! Local identity directory until Client Center owns accounts.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::token::now_ms;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformUser {
    pub end_user_id: i64,
    pub username: String,
    pub password: String,
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
    pub linked_at_ms: i64,
}

#[derive(Clone)]
pub struct UserDirectory {
    users: Arc<RwLock<HashMap<String, PlatformUser>>>,
    by_id: Arc<RwLock<HashMap<i64, String>>>,
    partner_links: Arc<RwLock<HashMap<(String, String), PartnerLink>>>,
    next_id: Arc<AtomicI64>,
}

impl UserDirectory {
    pub fn with_demo_users() -> Self {
        let mut users = HashMap::new();
        let mut by_id = HashMap::new();
        for u in [
            PlatformUser {
                end_user_id: 1,
                username: "demo_user".into(),
                password: "demo".into(),
                email: Some("demo_user@example.local".into()),
                status: "active".into(),
            },
            PlatformUser {
                end_user_id: 2,
                username: "alice_plat".into(),
                password: "demo".into(),
                email: Some("alice_plat@example.local".into()),
                status: "active".into(),
            },
        ] {
            by_id.insert(u.end_user_id, u.username.clone());
            users.insert(u.username.clone(), u);
        }
        Self {
            users: Arc::new(RwLock::new(users)),
            by_id: Arc::new(RwLock::new(by_id)),
            partner_links: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(AtomicI64::new(100)),
        }
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Option<PlatformUser> {
        let users = self.users.read().await;
        users
            .get(username)
            .filter(|u| u.password == password && u.status == "active")
            .cloned()
    }

    pub async fn get_by_id(&self, end_user_id: i64) -> Option<PlatformUser> {
        let by_id = self.by_id.read().await;
        let username = by_id.get(&end_user_id)?.clone();
        self.users.read().await.get(&username).cloned()
    }

    pub async fn get_by_username(&self, username: &str) -> Option<PlatformUser> {
        self.users.read().await.get(username).cloned()
    }

    pub async fn register(
        &self,
        username: String,
        password: String,
        email: Option<String>,
    ) -> Result<PlatformUser, String> {
        if username.trim().is_empty() || password.trim().is_empty() {
            return Err("username and password required".into());
        }
        let mut users = self.users.write().await;
        if users.contains_key(&username) {
            return Err("username already exists".into());
        }
        let end_user_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let user = PlatformUser {
            end_user_id,
            username: username.clone(),
            password,
            email,
            status: "active".into(),
        };
        self.by_id
            .write()
            .await
            .insert(end_user_id, username.clone());
        users.insert(username, user.clone());
        Ok(user)
    }

    pub async fn link_or_create_from_partner(
        &self,
        partner_id: &str,
        partner_user_id: &str,
        username_hint: Option<String>,
        game_id: Option<String>,
        game_account_id: Option<String>,
    ) -> PlatformUser {
        let key = (partner_id.to_string(), partner_user_id.to_string());
        if let Some(link) = self.partner_links.read().await.get(&key).cloned() {
            if let Some(user) = self.get_by_id(link.end_user_id).await {
                return user;
            }
        }

        let username = username_hint
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("{partner_id}_{partner_user_id}"));

        let user = if let Some(existing) = self.get_by_username(&username).await {
            existing
        } else {
            self.register(
                username,
                format!("oauth_{}", Uuid::new_v4().simple()),
                None,
            )
            .await
            .expect("unique username")
        };

        let link = PartnerLink {
            partner_id: partner_id.to_string(),
            partner_user_id: partner_user_id.to_string(),
            end_user_id: user.end_user_id,
            game_id,
            game_account_id,
            linked_at_ms: now_ms(),
        };
        self.partner_links.write().await.insert(key, link);
        user
    }
}
