//! Admin Game Partner Game Coin store (`fx_game.game_coins`).

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
pub struct GameCoinView {
    pub id: i64,
    pub code: String,
    pub name: String,
    #[serde(rename = "type")]
    pub coin_type: String,
    pub asset_kind: String,
    pub is_platform_token: bool,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone)]
pub struct GameCoinStore {
    pool: Option<PgPool>,
    memory: Option<Arc<Memory>>,
}

struct Memory {
    by_id: RwLock<HashMap<i64, GameCoinView>>,
    next_id: AtomicI64,
}

impl GameCoinStore {
    pub fn new(pool: Option<PgPool>) -> Self {
        if pool.is_some() {
            Self { pool, memory: None }
        } else {
            let mut by_id = HashMap::new();
            by_id.insert(
                1,
                GameCoinView {
                    id: 1,
                    code: "PLT".into(),
                    name: "Platform Token".into(),
                    coin_type: "platform".into(),
                    asset_kind: "token".into(),
                    is_platform_token: true,
                    status: "active".into(),
                    created_at: 1,
                    updated_at: 0,
                },
            );
            by_id.insert(
                2,
                GameCoinView {
                    id: 2,
                    code: "GCA".into(),
                    name: "Demo Game Coin A".into(),
                    coin_type: "game".into(),
                    asset_kind: "token".into(),
                    is_platform_token: false,
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

    pub async fn list(&self) -> Vec<GameCoinView> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(crate::query::game_coin::LIST)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows.into_iter().map(map_row).collect();
        }
        let mem = self.memory.as_ref().unwrap();
        let mut rows: Vec<_> = mem.by_id.read().await.values().cloned().collect();
        rows.sort_by_key(|c| c.id);
        rows
    }

    pub async fn get(&self, id: i64) -> Option<GameCoinView> {
        if let Some(pool) = &self.pool {
            return sqlx::query(crate::query::game_coin::GET)
            .bind(id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .map(map_row);
        }
        self.memory.as_ref()?.by_id.read().await.get(&id).cloned()
    }

    pub async fn create(
        &self,
        code: &str,
        name: &str,
        coin_type: &str,
        asset_kind: &str,
        is_platform_token: bool,
        admin_id: i64,
    ) -> Result<GameCoinView, String> {
        let code = code.trim().to_ascii_uppercase();
        let name = name.trim().to_string();
        if code.is_empty() || name.is_empty() {
            return Err("code and name required".into());
        }
        validate_type(coin_type)?;
        validate_asset_kind(asset_kind)?;
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let row = sqlx::query(crate::query::game_coin::INSERT)
            .bind(&code)
            .bind(&name)
            .bind(coin_type)
            .bind(asset_kind)
            .bind(is_platform_token)
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            let view = map_row(row);
            let _ = sqlx::query(crate::query::game_coin::INSERT_CREATE_ACTION_LOG)
            .bind(admin_id)
            .bind(serde_json::json!({ "game_coin_id": view.id, "code": view.code }).to_string())
            .bind(now)
            .execute(pool)
            .await;
            return Ok(view);
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let id = mem.next_id.fetch_add(1, Ordering::Relaxed);
        let view = GameCoinView {
            id,
            code,
            name,
            coin_type: coin_type.into(),
            asset_kind: asset_kind.into(),
            is_platform_token,
            status: "active".into(),
            created_at: now,
            updated_at: 0,
        };
        mem.by_id.write().await.insert(id, view.clone());
        Ok(view)
    }

    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        asset_kind: Option<&str>,
        admin_id: i64,
    ) -> Result<GameCoinView, String> {
        if let Some(ak) = asset_kind {
            validate_asset_kind(ak)?;
        }
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let existing = self
                .get(id)
                .await
                .ok_or_else(|| "game coin not found".to_string())?;
            let name = name
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(existing.name.as_str());
            let asset_kind = asset_kind.unwrap_or(existing.asset_kind.as_str());
            sqlx::query(crate::query::game_coin::UPDATE)
            .bind(id)
            .bind(name)
            .bind(asset_kind)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            let _ = sqlx::query(crate::query::game_coin::INSERT_UPDATE_ACTION_LOG)
            .bind(admin_id)
            .bind(serde_json::json!({ "game_coin_id": id }).to_string())
            .bind(now)
            .execute(pool)
            .await;
            return self
                .get(id)
                .await
                .ok_or_else(|| "game coin not found after update".into());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut g = mem.by_id.write().await;
        let rec = g
            .get_mut(&id)
            .ok_or_else(|| "game coin not found".to_string())?;
        if let Some(n) = name.map(str::trim).filter(|s| !s.is_empty()) {
            rec.name = n.into();
        }
        if let Some(ak) = asset_kind {
            rec.asset_kind = ak.into();
        }
        rec.updated_at = now;
        Ok(rec.clone())
    }

    pub async fn set_status(
        &self,
        id: i64,
        status: &str,
        admin_id: i64,
    ) -> Result<GameCoinView, String> {
        if status != "active" && status != "inactive" {
            return Err("status must be active or inactive".into());
        }
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let upd = sqlx::query(crate::query::game_coin::UPDATE_STATUS)
            .bind(id)
            .bind(status)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            if upd.rows_affected() == 0 {
                return Err("game coin not found".into());
            }
            let _ = sqlx::query(crate::query::game_coin::INSERT_STATUS_ACTION_LOG)
            .bind(admin_id)
            .bind(serde_json::json!({ "game_coin_id": id, "status": status }).to_string())
            .bind(now)
            .execute(pool)
            .await;
            return self
                .get(id)
                .await
                .ok_or_else(|| "game coin not found after update".into());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut g = mem.by_id.write().await;
        let rec = g
            .get_mut(&id)
            .ok_or_else(|| "game coin not found".to_string())?;
        rec.status = status.into();
        rec.updated_at = now;
        Ok(rec.clone())
    }
}

fn validate_type(t: &str) -> Result<(), String> {
    match t {
        "platform" | "company" | "game" | "internal" => Ok(()),
        _ => Err("type must be platform|company|game|internal".into()),
    }
}

fn validate_asset_kind(k: &str) -> Result<(), String> {
    match k {
        "token" | "crypto_token" | "stablecoin" => Ok(()),
        _ => Err("asset_kind must be token|crypto_token|stablecoin".into()),
    }
}

fn map_row(row: sqlx::postgres::PgRow) -> GameCoinView {
    GameCoinView {
        id: row.get("id"),
        code: row.get("code"),
        name: row.get("name"),
        coin_type: row.get("type"),
        asset_kind: row.get("asset_kind"),
        is_platform_token: row.get("is_platform_token"),
        status: row.get("status"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
