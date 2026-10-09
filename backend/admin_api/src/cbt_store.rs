//! Admin Company Basic Token approve / reject store.

use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminBasicToken {
    pub id: i64,
    pub corporate_user_id: i64,
    pub game_id: i64,
    pub game_coin_id: i64,
    pub token_code: String,
    pub token_name: String,
    pub status: String,
    pub buyable: bool,
    pub buy_fee_rate: f64,
    pub approved_by_admin_id: Option<i64>,
    pub approved_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone)]
pub struct BasicTokenStore {
    pool: Option<PgPool>,
    memory: Option<Arc<RwLock<HashMap<i64, AdminBasicToken>>>>,
}

impl BasicTokenStore {
    pub fn new(pool: Option<PgPool>) -> Self {
        if pool.is_some() {
            Self { pool, memory: None }
        } else {
            let mut m = HashMap::new();
            m.insert(
                1,
                AdminBasicToken {
                    id: 1,
                    corporate_user_id: 1,
                    game_id: 1,
                    game_coin_id: 2,
                    token_code: "DEMO_CBT".into(),
                    token_name: "Demo Company Basic Token".into(),
                    status: "pending".into(),
                    buyable: false,
                    buy_fee_rate: 0.001,
                    approved_by_admin_id: None,
                    approved_at: 0,
                    created_at: 1,
                    updated_at: 0,
                },
            );
            Self {
                pool: None,
                memory: Some(Arc::new(RwLock::new(m))),
            }
        }
    }

    pub async fn list(&self, status: Option<&str>) -> Vec<AdminBasicToken> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                        status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                        approved_by_admin_id, approved_at, created_at, updated_at \
                 FROM fx_corp_token.company_basic_tokens \
                 WHERE ($1::text IS NULL OR status = $1) \
                 ORDER BY id",
            )
            .bind(status)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows.into_iter().map(map_row).collect();
        }
        let Some(mem) = &self.memory else {
            return Vec::new();
        };
        let mut v: Vec<_> = mem
            .read()
            .await
            .values()
            .filter(|t| status.map(|s| t.status == s).unwrap_or(true))
            .cloned()
            .collect();
        v.sort_by_key(|t| t.id);
        v
    }

    pub async fn get(&self, id: i64) -> Option<AdminBasicToken> {
        if let Some(pool) = &self.pool {
            let row = sqlx::query(
                "SELECT id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                        status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                        approved_by_admin_id, approved_at, created_at, updated_at \
                 FROM fx_corp_token.company_basic_tokens WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()?;
            return Some(map_row(row));
        }
        let mem = self.memory.as_ref()?;
        mem.read().await.get(&id).cloned()
    }

    pub async fn approve(&self, id: i64, admin_id: i64) -> Result<AdminBasicToken, String> {
        self.set_decision(id, admin_id, true).await
    }

    pub async fn reject(&self, id: i64, admin_id: i64) -> Result<AdminBasicToken, String> {
        self.set_decision(id, admin_id, false).await
    }

    async fn set_decision(
        &self,
        id: i64,
        admin_id: i64,
        approve: bool,
    ) -> Result<AdminBasicToken, String> {
        let now = now_ms();
        let (status, buyable) = if approve {
            ("approved", true)
        } else {
            ("rejected", false)
        };
        if let Some(pool) = &self.pool {
            let row = sqlx::query(
                "UPDATE fx_corp_token.company_basic_tokens SET \
                    status = $1, buyable = $2, approved_by_admin_id = $3, approved_at = $4, updated_at = $4 \
                 WHERE id = $5 AND status IN ('submitted', 'pending', 'rejected') \
                 RETURNING id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                           status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                           approved_by_admin_id, approved_at, created_at, updated_at",
            )
            .bind(status)
            .bind(buyable)
            .bind(admin_id)
            .bind(now)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "token not found or not reviewable".to_string())?;

            let token = map_row(row);
            let notice_type = "other";
            let title = if approve {
                "Company Basic Token approved"
            } else {
                "Company Basic Token rejected"
            };
            let body = format!(
                "Token {} ({}) is now {}.",
                token.token_code, token.id, token.status
            );
            let _ = sqlx::query(
                "INSERT INTO fx_corp.corp_partner_notices ( \
                    corporate_user_id, notice_type, title, body, deadline_at, \
                    created_by_admin_id, read_at, created_at \
                 ) VALUES ($1, $2, $3, $4, 0, $5, 0, $6)",
            )
            .bind(token.corporate_user_id)
            .bind(notice_type)
            .bind(title)
            .bind(&body)
            .bind(admin_id)
            .bind(now)
            .execute(pool)
            .await;
            return Ok(token);
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut g = mem.write().await;
        let rec = g
            .get_mut(&id)
            .ok_or_else(|| "token not found or not reviewable".to_string())?;
        if !matches!(rec.status.as_str(), "submitted" | "pending" | "rejected") {
            return Err("token not found or not reviewable".into());
        }
        rec.status = status.into();
        rec.buyable = buyable;
        rec.approved_by_admin_id = Some(admin_id);
        rec.approved_at = now;
        rec.updated_at = now;
        Ok(rec.clone())
    }
}

fn map_row(row: sqlx::postgres::PgRow) -> AdminBasicToken {
    AdminBasicToken {
        id: row.get("id"),
        corporate_user_id: row.get("corporate_user_id"),
        game_id: row.get("game_id"),
        game_coin_id: row.get("game_coin_id"),
        token_code: row.get("token_code"),
        token_name: row.get("token_name"),
        status: row.get("status"),
        buyable: row.get("buyable"),
        buy_fee_rate: row.get("buy_fee_rate"),
        approved_by_admin_id: row.get("approved_by_admin_id"),
        approved_at: row.get("approved_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
