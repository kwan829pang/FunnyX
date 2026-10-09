//! Corp market pair submit + pending trade pool (lock client Game Coin; no live transfer).

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use serde::Serialize;
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

use crate::db::now_ms;

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
pub struct MarketPairView {
    pub id: i64,
    pub corporate_user_id: i64,
    pub game_id: i64,
    pub base_game_coin_id: i64,
    pub quote_game_coin_id: i64,
    pub market_name: String,
    pub funding_source: String,
    pub status: String,
    pub lock_game_coin_id: Option<i64>,
    pub lock_amount: Option<f64>,
    pub pool: Option<MarketPoolView>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct SubmitMarket {
    pub game_id: i64,
    pub base_game_coin_id: i64,
    pub quote_game_coin_id: i64,
    pub market_name: String,
    pub funding_source: String,
    pub pool_depth: f64,
    pub initial_price: f64,
    pub base_amount: f64,
    pub quote_amount: f64,
}

struct Memory {
    pairs: RwLock<HashMap<i64, MarketPairView>>,
    next_pair: AtomicI64,
    next_pool: AtomicI64,
    /// game_id + coin_id -> available
    balances: RwLock<HashMap<(i64, i64), f64>>,
}

#[derive(Clone)]
pub struct MarketStore {
    pool: Option<PgPool>,
    memory: Option<Arc<Memory>>,
}

impl MarketStore {
    pub fn new(pool: Option<PgPool>) -> Self {
        if pool.is_some() {
            Self { pool, memory: None }
        } else {
            let mut balances = HashMap::new();
            balances.insert((1, 2), 1_000_000.0);
            Self {
                pool: None,
                memory: Some(Arc::new(Memory {
                    pairs: RwLock::new(HashMap::new()),
                    next_pair: AtomicI64::new(1),
                    next_pool: AtomicI64::new(1),
                    balances: RwLock::new(balances),
                })),
            }
        }
    }

    pub async fn list(&self, corporate_user_id: i64) -> Vec<MarketPairView> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT p.id, p.corporate_user_id, p.game_id, p.base_game_coin_id, p.quote_game_coin_id, \
                        p.market_name, p.funding_source, p.status, p.created_at, p.updated_at, \
                        po.id AS pool_id, po.pool_depth::float8 AS pool_depth, \
                        po.initial_price::float8 AS initial_price, po.base_amount::float8 AS base_amount, \
                        po.quote_amount::float8 AS quote_amount, po.status AS pool_status, \
                        l.game_coin_id AS lock_game_coin_id, l.locked_amount::float8 AS lock_amount \
                 FROM fx_market.market_pairs p \
                 LEFT JOIN fx_market.market_pools po ON po.market_pair_id = p.id \
                 LEFT JOIN fx_market.market_balance_locks l ON l.market_pair_id = p.id AND l.status = 'locked' \
                 WHERE p.corporate_user_id = $1 \
                 ORDER BY p.id",
            )
            .bind(corporate_user_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows.into_iter().map(map_pair).collect();
        }
        let Some(mem) = &self.memory else {
            return Vec::new();
        };
        let mut v: Vec<_> = mem
            .pairs
            .read()
            .await
            .values()
            .filter(|p| p.corporate_user_id == corporate_user_id)
            .cloned()
            .collect();
        v.sort_by_key(|p| p.id);
        v
    }

    pub async fn get(&self, corporate_user_id: i64, id: i64) -> Option<MarketPairView> {
        self.list(corporate_user_id).await.into_iter().find(|p| p.id == id)
    }

    pub async fn submit(
        &self,
        corporate_user_id: i64,
        req: SubmitMarket,
    ) -> Result<MarketPairView, String> {
        if req.base_game_coin_id == req.quote_game_coin_id {
            return Err("base and quote must differ".into());
        }
        if req.base_amount <= 0.0 || req.quote_amount <= 0.0 || req.pool_depth <= 0.0 {
            return Err("pool_depth, base_amount, quote_amount must be > 0".into());
        }
        if req.funding_source != "gamecoin_lockup" && req.funding_source != "platform_token_deposit"
        {
            return Err("funding_source must be gamecoin_lockup or platform_token_deposit".into());
        }
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let owned: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM fx_game.games WHERE id = $1 AND corporate_user_id = $2",
            )
            .bind(req.game_id)
            .bind(corporate_user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
            if owned.is_none() {
                return Err("game not found for this corporate user".into());
            }
            let coins: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM fx_game.game_coins WHERE id IN ($1, $2) AND status = 'active'",
            )
            .bind(req.base_game_coin_id)
            .bind(req.quote_game_coin_id)
            .fetch_one(pool)
            .await
            .unwrap_or(0);
            if coins < 2 {
                return Err("base_game_coin_id and quote_game_coin_id must be active game coins".into());
            }
            let existing: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM fx_market.market_pairs WHERE game_id = $1 \
                 AND status NOT IN ('rejected', 'inactive')",
            )
            .bind(req.game_id)
            .fetch_one(pool)
            .await
            .unwrap_or(0);
            let base_plt: bool = sqlx::query_scalar(
                "SELECT COALESCE(is_platform_token, FALSE) FROM fx_game.game_coins WHERE id = $1",
            )
            .bind(req.base_game_coin_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .unwrap_or(false);
            let quote_plt: bool = sqlx::query_scalar(
                "SELECT COALESCE(is_platform_token, FALSE) FROM fx_game.game_coins WHERE id = $1",
            )
            .bind(req.quote_game_coin_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .unwrap_or(false);
            if existing == 0 && !base_plt && !quote_plt {
                return Err("first market for a game must pair with Platform Token".into());
            }
            let (lock_coin, lock_amt) = lock_side(&req, base_plt);
            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
            let upd = sqlx::query(
                "UPDATE fx_game.game_balances SET available_balance = available_balance - $1, \
                    locked_balance = locked_balance + $1, updated_at = $2 \
                 WHERE game_id = $3 AND game_coin_id = $4 AND available_balance >= $1",
            )
            .bind(lock_amt)
            .bind(now)
            .bind(req.game_id)
            .bind(lock_coin)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            if upd.rows_affected() == 0 {
                return Err("insufficient Game Partner Game Coin available_balance to lock pool".into());
            }
            let pair = sqlx::query(
                "INSERT INTO fx_market.market_pairs ( \
                    corporate_user_id, game_id, base_game_coin_id, quote_game_coin_id, market_name, \
                    funding_source, status, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, $6, 'pending_locked', $7, 0) RETURNING id",
            )
            .bind(corporate_user_id)
            .bind(req.game_id)
            .bind(req.base_game_coin_id)
            .bind(req.quote_game_coin_id)
            .bind(&req.market_name)
            .bind(&req.funding_source)
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            let pair_id: i64 = pair.get("id");
            sqlx::query(
                "INSERT INTO fx_market.market_balance_locks ( \
                    market_pair_id, game_id, game_coin_id, locked_amount, status, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, 'locked', $5, 0)",
            )
            .bind(pair_id)
            .bind(req.game_id)
            .bind(lock_coin)
            .bind(lock_amt)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            let pool_row = sqlx::query(
                "INSERT INTO fx_market.market_pools ( \
                    market_pair_id, pool_depth, initial_price, base_amount, quote_amount, status, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, 'pending', $6, 0) RETURNING id",
            )
            .bind(pair_id)
            .bind(req.pool_depth)
            .bind(req.initial_price)
            .bind(req.base_amount)
            .bind(req.quote_amount)
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            let pool_id: i64 = pool_row.get("id");
            for coin in [req.base_game_coin_id, req.quote_game_coin_id] {
                sqlx::query(
                    "INSERT INTO fx_market.pool_wallets (market_pool_id, game_coin_id, balance, wallet_type, status) \
                     VALUES ($1, $2, 0, 'market_pool', 'active') \
                     ON CONFLICT (market_pool_id, game_coin_id) DO NOTHING",
                )
                .bind(pool_id)
                .bind(coin)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            }
            sqlx::query(
                "INSERT INTO fx_market.market_status_logs (market_pair_id, old_status, new_status, updated_at, created_at) \
                 VALUES ($1, 'submitted', 'pending_locked', $2, $2)",
            )
            .bind(pair_id)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            tx.commit().await.map_err(|e| e.to_string())?;
            return self
                .get(corporate_user_id, pair_id)
                .await
                .ok_or_else(|| "pair missing after insert".into());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let first = mem.pairs.read().await.values().filter(|p| p.game_id == req.game_id).count() == 0;
        let base_plt = req.base_game_coin_id == 1;
        let quote_plt = req.quote_game_coin_id == 1;
        if first && !base_plt && !quote_plt {
            return Err("first market for a game must pair with Platform Token".into());
        }
        let (lock_coin, lock_amt) = lock_side(&req, base_plt);
        {
            let mut bal = mem.balances.write().await;
            let avail = bal.entry((req.game_id, lock_coin)).or_insert(0.0);
            if *avail < lock_amt {
                return Err("insufficient Game Partner Game Coin available_balance to lock pool".into());
            }
            *avail -= lock_amt;
        }
        let id = mem.next_pair.fetch_add(1, Ordering::Relaxed);
        let pool_id = mem.next_pool.fetch_add(1, Ordering::Relaxed);
        let rec = MarketPairView {
            id,
            corporate_user_id,
            game_id: req.game_id,
            base_game_coin_id: req.base_game_coin_id,
            quote_game_coin_id: req.quote_game_coin_id,
            market_name: req.market_name,
            funding_source: req.funding_source,
            status: "pending_locked".into(),
            lock_game_coin_id: Some(lock_coin),
            lock_amount: Some(lock_amt),
            pool: Some(MarketPoolView {
                id: pool_id,
                market_pair_id: id,
                pool_depth: req.pool_depth,
                initial_price: req.initial_price,
                base_amount: req.base_amount,
                quote_amount: req.quote_amount,
                status: "pending".into(),
            }),
            created_at: now,
            updated_at: 0,
        };
        mem.pairs.write().await.insert(id, rec.clone());
        Ok(rec)
    }

    pub async fn ensure_pool(
        &self,
        corporate_user_id: i64,
        pair_id: i64,
        pool_depth: f64,
        initial_price: f64,
        base_amount: f64,
        quote_amount: f64,
    ) -> Result<MarketPairView, String> {
        let pair = self
            .get(corporate_user_id, pair_id)
            .await
            .ok_or_else(|| "market pair not found".to_string())?;
        if let Some(p) = &pair.pool {
            return Ok(MarketPairView {
                pool: Some(p.clone()),
                ..pair
            });
        }
        if pool_depth <= 0.0 || base_amount <= 0.0 || quote_amount <= 0.0 {
            return Err("pool_depth, base_amount, quote_amount must be > 0".into());
        }
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let row = sqlx::query(
                "INSERT INTO fx_market.market_pools ( \
                    market_pair_id, pool_depth, initial_price, base_amount, quote_amount, status, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, 'pending', $6, 0) RETURNING id",
            )
            .bind(pair_id)
            .bind(pool_depth)
            .bind(initial_price)
            .bind(base_amount)
            .bind(quote_amount)
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            let pool_id: i64 = row.get("id");
            for coin in [pair.base_game_coin_id, pair.quote_game_coin_id] {
                let _ = sqlx::query(
                    "INSERT INTO fx_market.pool_wallets (market_pool_id, game_coin_id, balance, wallet_type, status) \
                     ON CONFLICT (market_pool_id, game_coin_id) DO NOTHING",
                )
                .bind(pool_id)
                .bind(coin)
                .execute(pool)
                .await;
            }
            return self
                .get(corporate_user_id, pair_id)
                .await
                .ok_or_else(|| "pair missing".into());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let pool_id = mem.next_pool.fetch_add(1, Ordering::Relaxed);
        let mut g = mem.pairs.write().await;
        let rec = g.get_mut(&pair_id).ok_or_else(|| "market pair not found".to_string())?;
        rec.pool = Some(MarketPoolView {
            id: pool_id,
            market_pair_id: pair_id,
            pool_depth,
            initial_price,
            base_amount,
            quote_amount,
            status: "pending".into(),
        });
        Ok(rec.clone())
    }
}

fn lock_side(req: &SubmitMarket, base_is_plt: bool) -> (i64, f64) {
    if base_is_plt {
        (req.quote_game_coin_id, req.quote_amount)
    } else {
        (req.base_game_coin_id, req.base_amount)
    }
}

fn map_pair(row: sqlx::postgres::PgRow) -> MarketPairView {
    let pool = match row.try_get::<Option<i64>, _>("pool_id") {
        Ok(Some(id)) => Some(MarketPoolView {
            id,
            market_pair_id: row.get("id"),
            pool_depth: row.try_get("pool_depth").unwrap_or(0.0),
            initial_price: row.try_get("initial_price").unwrap_or(0.0),
            base_amount: row.try_get("base_amount").unwrap_or(0.0),
            quote_amount: row.try_get("quote_amount").unwrap_or(0.0),
            status: row.try_get("pool_status").unwrap_or_else(|_| "pending".into()),
        }),
        _ => None,
    };
    MarketPairView {
        id: row.get("id"),
        corporate_user_id: row.get("corporate_user_id"),
        game_id: row.get("game_id"),
        base_game_coin_id: row.get("base_game_coin_id"),
        quote_game_coin_id: row.get("quote_game_coin_id"),
        market_name: row.get("market_name"),
        funding_source: row.get("funding_source"),
        status: row.get("status"),
        lock_game_coin_id: row.try_get("lock_game_coin_id").ok().flatten(),
        lock_amount: row.try_get("lock_amount").ok().flatten(),
        pool,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
