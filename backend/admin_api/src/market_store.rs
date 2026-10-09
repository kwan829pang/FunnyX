//! Admin market pair list / approve / reject on shared Postgres.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use serde::Serialize;
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketPoolView {
    pub id: i64,
    pub market_pair_id: i64,
    pub pool_depth: f64,
    pub initial_price: f64,
    pub base_amount: f64,
    pub quote_amount: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminMarketView {
    pub id: i64,
    pub corporate_user_id: i64,
    pub game_id: i64,
    pub base_game_coin_id: i64,
    pub quote_game_coin_id: i64,
    pub base_game_coin: Option<String>,
    pub quote_game_coin: Option<String>,
    pub market_name: String,
    pub funding_source: String,
    pub status: String,
    pub lock_id: Option<i64>,
    pub lock_game_coin_id: Option<i64>,
    pub lock_amount: Option<f64>,
    pub pool: Option<MarketPoolView>,
    pub created_at: i64,
    pub updated_at: i64,
}

struct Memory {
    pairs: RwLock<HashMap<i64, AdminMarketView>>,
    next_id: AtomicI64,
}

#[derive(Clone)]
pub struct AdminMarketStore {
    pool: Option<PgPool>,
    memory: Option<Arc<Memory>>,
}

impl AdminMarketStore {
    pub fn new(pool: Option<PgPool>) -> Self {
        if pool.is_some() {
            Self { pool, memory: None }
        } else {
            Self {
                pool: None,
                memory: Some(Arc::new(Memory {
                    pairs: RwLock::new(HashMap::new()),
                    next_id: AtomicI64::new(1),
                })),
            }
        }
    }

    /// Seed a pending market for memory-mode demos (tests / no Postgres).
    #[allow(dead_code)]
    pub async fn seed_pending_demo(&self) {
        let Some(mem) = &self.memory else {
            return;
        };
        let id = mem.next_id.fetch_add(1, Ordering::Relaxed);
        let view = AdminMarketView {
            id,
            corporate_user_id: 1,
            game_id: 1,
            base_game_coin_id: 2,
            quote_game_coin_id: 1,
            base_game_coin: Some("GCA".into()),
            quote_game_coin: Some("PLT".into()),
            market_name: "GCA/PLT".into(),
            funding_source: "gamecoin_lockup".into(),
            status: "pending_locked".into(),
            lock_id: Some(1),
            lock_game_coin_id: Some(2),
            lock_amount: Some(200_000.0),
            pool: Some(MarketPoolView {
                id: 1,
                market_pair_id: id,
                pool_depth: 5_000_000.0,
                initial_price: 0.025,
                base_amount: 200_000.0,
                quote_amount: 5_000.0,
                status: "pending".into(),
            }),
            created_at: now_ms(),
            updated_at: 0,
        };
        mem.pairs.write().await.insert(id, view);
    }

    pub async fn list(&self, status: Option<&str>) -> Vec<AdminMarketView> {
        if let Some(pool) = &self.pool {
            let rows = if let Some(st) = status.filter(|s| !s.is_empty()) {
                sqlx::query(crate::query::market::LIST_FILTER)
                    .bind(st)
                    .fetch_all(pool)
                    .await
                    .unwrap_or_default()
            } else {
                sqlx::query(crate::query::market::LIST)
                    .fetch_all(pool)
                    .await
                    .unwrap_or_default()
            };
            return rows.into_iter().map(map_row).collect();
        }
        let mem = match &self.memory {
            Some(m) => m,
            None => return Vec::new(),
        };
        let mut rows: Vec<_> = mem.pairs.read().await.values().cloned().collect();
        if let Some(st) = status.filter(|s| !s.is_empty()) {
            rows.retain(|p| p.status == st);
        }
        rows.sort_by_key(|p| std::cmp::Reverse(p.id));
        rows
    }

    pub async fn get(&self, id: i64) -> Option<AdminMarketView> {
        if let Some(pool) = &self.pool {
            return sqlx::query(crate::query::market::LIST_ONE)
                .bind(id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
                .map(map_row);
        }
        self.memory.as_ref()?.pairs.read().await.get(&id).cloned()
    }

    /// Approve: transfer lock → pool wallets; activate pair/pool.
    pub async fn approve(&self, id: i64, admin_id: i64) -> Result<AdminMarketView, String> {
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
            let pair = sqlx::query(crate::query::market::SELECT_PAIR_FOR_APPROVE)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "market not found".to_string())?;
            let status: String = pair.get("status");
            if status != "pending_locked" {
                return Err(format!("market status must be pending_locked (got {status})"));
            }
            let game_id: i64 = pair.get("game_id");
            let base_coin: i64 = pair.get("base_game_coin_id");
            let quote_coin: i64 = pair.get("quote_game_coin_id");
            let funding: String = pair.get("funding_source");
            let corporate_user_id: i64 = pair.get("corporate_user_id");
            let market_name: String = pair.get("market_name");

            let lock = sqlx::query(crate::query::market::SELECT_LOCK_FOR_UPDATE)
                .bind(id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "no locked balance for market".to_string())?;
            let lock_id: i64 = lock.get("id");
            let lock_coin: i64 = lock.get("game_coin_id");
            let lock_amt: f64 = lock.get("locked_amount");

            let pool_row = sqlx::query(crate::query::market::SELECT_POOL_FOR_UPDATE)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "market pool not found".to_string())?;
            let pool_id: i64 = pool_row.get("id");
            let pool_status: String = pool_row.get("status");
            if pool_status != "pending" {
                return Err(format!("pool status must be pending (got {pool_status})"));
            }
            let base_amount: f64 = pool_row.get("base_amount");
            let quote_amount: f64 = pool_row.get("quote_amount");
            let pool_depth: f64 = pool_row.get("pool_depth");
            let initial_price: f64 = pool_row.get("initial_price");

            // Release client locked balance (consumed into pool).
            let unlocked = sqlx::query(crate::query::market::UPDATE_GAME_BALANCE_CONSUME_LOCK)
            .bind(lock_amt)
            .bind(now)
            .bind(game_id)
            .bind(lock_coin)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            if unlocked.rows_affected() == 0 {
                return Err("insufficient locked_balance to transfer".into());
            }

            // Credit pool wallets.
            for (coin, amt) in [(base_coin, base_amount), (quote_coin, quote_amount)] {
                sqlx::query(crate::query::market::UPSERT_POOL_WALLET)
                .bind(pool_id)
                .bind(coin)
                .bind(amt)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            }

            sqlx::query(crate::query::market::UPDATE_LOCK_TRANSFERRED)
            .bind(lock_id)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query(crate::query::market::UPDATE_POOL_ACTIVE)
            .bind(pool_id)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query(crate::query::market::UPDATE_PAIR_ACTIVE)
            .bind(id)
            .bind(admin_id)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query(crate::query::market::INSERT_POOL_TRANSFER_EVENT)
            .bind(id)
            .bind(pool_id)
            .bind(lock_id)
            .bind(lock_amt)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query(crate::query::market::INSERT_STATUS_LOG_APPROVE)
            .bind(id)
            .bind(admin_id)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            let _ = sqlx::query(crate::query::market::INSERT_ACTION_LOG_APPROVE)
            .bind(admin_id)
            .bind(
                serde_json::json!({
                    "market_pair_id": id,
                    "market_name": market_name,
                    "funding_source": funding,
                    "corporate_user_id": corporate_user_id,
                    "pool_depth": pool_depth,
                    "initial_price": initial_price,
                    "base_amount": base_amount,
                    "quote_amount": quote_amount,
                })
                .to_string(),
            )
            .bind(now)
            .execute(&mut *tx)
            .await;

            tx.commit().await.map_err(|e| e.to_string())?;
            return self
                .get(id)
                .await
                .ok_or_else(|| "market missing after approve".into());
        }

        // Memory path
        let mem = self.memory.as_ref().ok_or("POSTGRES_URL required for market approve")?;
        let mut g = mem.pairs.write().await;
        let rec = g
            .get_mut(&id)
            .ok_or_else(|| "market not found".to_string())?;
        if rec.status != "pending_locked" {
            return Err(format!(
                "market status must be pending_locked (got {})",
                rec.status
            ));
        }
        rec.status = "active".into();
        rec.updated_at = now;
        if let Some(p) = &mut rec.pool {
            p.status = "active".into();
        }
        Ok(rec.clone())
    }

    pub async fn reject(&self, id: i64, admin_id: i64) -> Result<AdminMarketView, String> {
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
            let pair = sqlx::query(crate::query::market::SELECT_PAIR_FOR_REJECT)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "market not found".to_string())?;
            let status: String = pair.get("status");
            if status != "pending_locked" && status != "submitted" {
                return Err(format!(
                    "market status must be pending_locked or submitted (got {status})"
                ));
            }
            let game_id: i64 = pair.get("game_id");

            if let Some(lock) = sqlx::query(crate::query::market::SELECT_LOCK_FOR_UPDATE)
                .bind(id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| e.to_string())?
            {
                let lock_id: i64 = lock.get("id");
                let lock_coin: i64 = lock.get("game_coin_id");
                let lock_amt: f64 = lock.get("locked_amount");
                sqlx::query(crate::query::market::UPDATE_GAME_BALANCE_UNLOCK)
                    .bind(lock_amt)
                    .bind(now)
                    .bind(game_id)
                    .bind(lock_coin)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query(crate::query::market::UPDATE_LOCK_UNLOCKED)
                    .bind(lock_id)
                    .bind(now)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
            }

            sqlx::query(crate::query::market::UPDATE_POOL_CANCELLED)
                .bind(id)
                .bind(now)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

            sqlx::query(crate::query::market::UPDATE_PAIR_REJECTED)
                .bind(id)
                .bind(admin_id)
                .bind(now)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

            sqlx::query(crate::query::market::INSERT_STATUS_LOG_REJECT)
                .bind(id)
                .bind(&status)
                .bind(admin_id)
                .bind(now)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

            let _ = sqlx::query(crate::query::market::INSERT_ACTION_LOG_REJECT)
                .bind(admin_id)
                .bind(serde_json::json!({ "market_pair_id": id }).to_string())
                .bind(now)
                .execute(&mut *tx)
                .await;

            tx.commit().await.map_err(|e| e.to_string())?;
            return self
                .get(id)
                .await
                .ok_or_else(|| "market missing after reject".into());
        }

        let mem = self.memory.as_ref().ok_or("POSTGRES_URL required for market reject")?;
        let mut g = mem.pairs.write().await;
        let rec = g
            .get_mut(&id)
            .ok_or_else(|| "market not found".to_string())?;
        if rec.status != "pending_locked" && rec.status != "submitted" {
            return Err(format!(
                "market status must be pending_locked or submitted (got {})",
                rec.status
            ));
        }
        rec.status = "rejected".into();
        rec.updated_at = now;
        if let Some(p) = &mut rec.pool {
            p.status = "cancelled".into();
        }
        Ok(rec.clone())
    }
}

fn map_row(row: sqlx::postgres::PgRow) -> AdminMarketView {
    let pool_id: Option<i64> = row.try_get("pool_id").ok();
    let pool = pool_id.map(|pid| MarketPoolView {
        id: pid,
        market_pair_id: row.get("id"),
        pool_depth: row.get("pool_depth"),
        initial_price: row.get("initial_price"),
        base_amount: row.get("base_amount"),
        quote_amount: row.get("quote_amount"),
        status: row.get("pool_status"),
    });
    AdminMarketView {
        id: row.get("id"),
        corporate_user_id: row.get("corporate_user_id"),
        game_id: row.get("game_id"),
        base_game_coin_id: row.get("base_game_coin_id"),
        quote_game_coin_id: row.get("quote_game_coin_id"),
        base_game_coin: row.try_get("base_game_coin").ok(),
        quote_game_coin: row.try_get("quote_game_coin").ok(),
        market_name: row.get("market_name"),
        funding_source: row.get("funding_source"),
        status: row.get("status"),
        lock_id: row.try_get("lock_id").ok(),
        lock_game_coin_id: row.try_get("lock_game_coin_id").ok(),
        lock_amount: row.try_get("lock_amount").ok(),
        pool,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
