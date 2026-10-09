//! `fx_game.game_accounts` mapping (end_user ↔ game ↔ partner game account).

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

use crate::db::now_ms;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAccountBinding {
    pub id: i64,
    pub end_user_id: i64,
    pub game_id: i64,
    pub game_code: String,
    pub game_account_id: String,
    pub partner_id: String,
    pub partner_user_id: Option<String>,
    pub bind_source: String,
    pub status: String,
}

#[derive(Clone)]
pub struct GameAccountStore {
    pool: Option<PgPool>,
    memory: Option<MemoryBindings>,
}

#[derive(Clone)]
struct MemoryBindings {
    by_id: Arc<RwLock<HashMap<i64, GameAccountBinding>>>,
    by_user_game: Arc<RwLock<HashMap<(i64, i64), i64>>>,
    next_id: Arc<AtomicI64>,
}

impl GameAccountStore {
    pub fn memory() -> Self {
        Self {
            pool: None,
            memory: Some(MemoryBindings {
                by_id: Arc::new(RwLock::new(HashMap::new())),
                by_user_game: Arc::new(RwLock::new(HashMap::new())),
                next_id: Arc::new(AtomicI64::new(1)),
            }),
        }
    }

    pub fn postgres(pool: PgPool) -> Self {
        Self {
            pool: Some(pool),
            memory: None,
        }
    }

    pub async fn list_for_user(&self, end_user_id: i64) -> Vec<GameAccountBinding> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                crate::query::bindings::LIST_FOR_USER,
            )
            .bind(end_user_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows.into_iter().map(map_binding).collect();
        }
        let Some(mem) = &self.memory else {
            return Vec::new();
        };
        let ids = mem.by_user_game.read().await;
        let store = mem.by_id.read().await;
        ids.iter()
            .filter(|((uid, _), _)| *uid == end_user_id)
            .filter_map(|(_, id)| store.get(id).cloned())
            .collect()
    }

    pub async fn upsert(
        &self,
        end_user_id: i64,
        game_id: i64,
        game_code: String,
        game_account_id: String,
        partner_id: String,
        partner_user_id: Option<String>,
        bind_source: String,
    ) -> Result<GameAccountBinding, String> {
        if game_account_id.trim().is_empty() {
            return Err("game_account_id required".into());
        }
        if let Some(pool) = &self.pool {
            return upsert_pg(
                pool,
                end_user_id,
                game_id,
                game_account_id,
                partner_user_id,
                bind_source,
            )
            .await;
        }
        let Some(mem) = &self.memory else {
            return Err("store unavailable".into());
        };
        let key = (end_user_id, game_id);
        let mut index = mem.by_user_game.write().await;
        let mut store = mem.by_id.write().await;
        if let Some(existing_id) = index.get(&key).copied() {
            if let Some(row) = store.get_mut(&existing_id) {
                row.game_account_id = game_account_id;
                row.partner_user_id = partner_user_id;
                row.bind_source = bind_source;
                row.status = "active".into();
                return Ok(row.clone());
            }
        }
        let id = mem.next_id.fetch_add(1, Ordering::Relaxed);
        let row = GameAccountBinding {
            id,
            end_user_id,
            game_id,
            game_code,
            game_account_id,
            partner_id,
            partner_user_id,
            bind_source,
            status: "active".into(),
        };
        index.insert(key, id);
        store.insert(id, row.clone());
        Ok(row)
    }
}

async fn upsert_pg(
    pool: &PgPool,
    end_user_id: i64,
    game_id: i64,
    game_account_id: String,
    partner_user_id: Option<String>,
    bind_source: String,
) -> Result<GameAccountBinding, String> {
    let now = now_ms();
    if let Some(row) = sqlx::query(
        crate::query::bindings::BY_USER_AND_GAME,
    )
    .bind(end_user_id)
    .bind(game_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    {
        let id: i64 = row.get("id");
        sqlx::query(
            crate::query::bindings::UPDATE,
        )
        .bind(&game_account_id)
        .bind(&partner_user_id)
        .bind(&bind_source)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        return sqlx::query(
            crate::query::bindings::BY_ID,
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .map(map_binding)
        .map_err(|e| e.to_string());
    }

    sqlx::query(
        crate::query::bindings::INSERT_UPSERT,
    )
    .bind(game_id)
    .bind(end_user_id)
    .bind(&game_account_id)
    .bind(&partner_user_id)
    .bind(&bind_source)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        crate::query::bindings::BY_GAME_AND_ACCOUNT,
    )
    .bind(game_id)
    .bind(&game_account_id)
    .fetch_one(pool)
    .await
    .map(map_binding)
    .map_err(|e| e.to_string())
}

fn map_binding(row: sqlx::postgres::PgRow) -> GameAccountBinding {
    GameAccountBinding {
        id: row.get("id"),
        end_user_id: row.get("end_user_id"),
        game_id: row.get("game_id"),
        game_code: row.get("game_code"),
        game_account_id: row.get("game_account_id"),
        partner_id: row.get("partner_id"),
        partner_user_id: row.get("partner_user_id"),
        bind_source: row.get("bind_source"),
        status: row.get("status"),
    }
}
