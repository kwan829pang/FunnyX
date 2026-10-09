//! Game catalog from `fx_game.games` (direct Postgres) or in-memory fallback.

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRecord {
    pub game_id: i64,
    pub game_code: String,
    pub game_name: String,
    pub partner_id: String,
    pub partner_game_id: String,
    pub status: String,
}

#[derive(Clone)]
pub struct GameCatalog {
    pool: Option<PgPool>,
    memory: Vec<GameRecord>,
}

impl GameCatalog {
    pub fn memory(default_partner_id: &str) -> Self {
        Self {
            pool: None,
            memory: vec![GameRecord {
                game_id: 1,
                game_code: "DEMO_GAME".into(),
                game_name: "Demo Adventure".into(),
                partner_id: default_partner_id.to_string(),
                partner_game_id: "game_001".into(),
                status: "active".into(),
            }],
        }
    }

    pub fn postgres(pool: PgPool) -> Self {
        Self {
            pool: Some(pool),
            memory: Vec::new(),
        }
    }

    pub async fn list_active(&self) -> Vec<GameRecord> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                crate::query::games::LIST_ACTIVE,
            )
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows.into_iter().map(map_game).collect();
        }
        self.memory
            .iter()
            .filter(|g| g.status == "active")
            .cloned()
            .collect()
    }

    pub async fn get(&self, game_id: i64) -> Option<GameRecord> {
        if let Some(pool) = &self.pool {
            return sqlx::query(
                crate::query::games::BY_ID,
            )
            .bind(game_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .map(map_game);
        }
        self.memory.iter().find(|g| g.game_id == game_id).cloned()
    }

    pub async fn get_by_code(&self, game_code: &str) -> Option<GameRecord> {
        if let Some(pool) = &self.pool {
            return sqlx::query(
                crate::query::games::BY_CODE,
            )
            .bind(game_code)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .map(map_game);
        }
        self.memory
            .iter()
            .find(|g| g.game_code.eq_ignore_ascii_case(game_code))
            .cloned()
    }

    pub async fn get_by_partner_game_id(&self, partner_game_id: &str) -> Option<GameRecord> {
        if let Some(pool) = &self.pool {
            return sqlx::query(
                crate::query::games::BY_PARTNER_GAME_ID,
            )
            .bind(partner_game_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .map(map_game);
        }
        self.memory
            .iter()
            .find(|g| g.partner_game_id == partner_game_id)
            .cloned()
    }

    pub async fn get_by_partner_code(&self, partner_id: &str) -> Option<GameRecord> {
        if let Some(pool) = &self.pool {
            return sqlx::query(
                crate::query::games::BY_PARTNER_CODE,
            )
            .bind(partner_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .map(map_game);
        }
        self.memory
            .iter()
            .find(|g| g.partner_id == partner_id && g.status == "active")
            .cloned()
    }
}

fn map_game(row: sqlx::postgres::PgRow) -> GameRecord {
    GameRecord {
        game_id: row.get("id"),
        game_code: row.get("game_code"),
        game_name: row.get("game_name"),
        partner_id: row.get("partner_id"),
        partner_game_id: row.get("partner_game_id"),
        status: row.get("status"),
    }
}
