//! Company Basic Token catalog, orders, fee ledger (memory + PostgreSQL).

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use serde::Serialize;
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

use crate::db::now_ms;

const ORDER_TTL_MS: i64 = 24 * 60 * 60 * 1000;
const DEFAULT_FEE_RATE: f64 = 0.001;

#[derive(Debug, Clone, Serialize)]
pub struct BasicTokenView {
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

#[derive(Debug, Clone, Serialize)]
pub struct CbtOrderView {
    pub id: i64,
    pub company_basic_token_id: i64,
    pub end_user_id: i64,
    pub game_account_id: i64,
    pub partner_order_no: Option<String>,
    pub pay_amount: f64,
    pub pay_game_coin_id: i64,
    pub coin_amount: f64,
    pub fee_coin_amount: f64,
    pub credited_coin_amount: f64,
    pub game_coin_id: i64,
    pub status: String,
    pub expires_at: i64,
    pub paid_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub token_code: Option<String>,
    pub corporate_user_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeeLedgerView {
    pub id: i64,
    pub corporate_user_id: i64,
    pub company_basic_token_id: i64,
    pub corp_token_order_id: Option<i64>,
    pub game_coin_id: i64,
    pub fee_rate: f64,
    pub gross_coin_amount: f64,
    pub fee_coin_amount: f64,
    pub net_coin_amount: f64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CbtSettleResult {
    pub order: CbtOrderView,
    pub partner_callback_url: Option<String>,
    pub corporate_user_id: i64,
}

struct MemoryCbt {
    tokens: RwLock<HashMap<i64, BasicTokenView>>,
    orders: RwLock<HashMap<i64, CbtOrderView>>,
    fees: RwLock<Vec<FeeLedgerView>>,
    next_token: AtomicI64,
    next_order: AtomicI64,
    next_fee: AtomicI64,
    /// corporate_user_id -> callback URL (memory demo)
    callbacks: RwLock<HashMap<i64, String>>,
}

#[derive(Clone)]
pub struct CbtStore {
    pool: Option<PgPool>,
    memory: Option<Arc<MemoryCbt>>,
}

impl CbtStore {
    pub fn new(pool: Option<PgPool>) -> Self {
        if pool.is_some() {
            Self { pool, memory: None }
        } else {
            let mut tokens = HashMap::new();
            tokens.insert(1, demo_token());
            let mut callbacks = HashMap::new();
            callbacks.insert(1, "http://127.0.0.1:18100/api/balance/callback".into());
            Self {
                pool: None,
                memory: Some(Arc::new(MemoryCbt {
                    tokens: RwLock::new(tokens),
                    orders: RwLock::new(HashMap::new()),
                    fees: RwLock::new(Vec::new()),
                    next_token: AtomicI64::new(2),
                    next_order: AtomicI64::new(7001),
                    next_fee: AtomicI64::new(1),
                    callbacks: RwLock::new(callbacks),
                })),
            }
        }
    }

    pub async fn has_approved_for_coin(&self, corp_id: i64, game_coin_id: i64) -> bool {
        if let Some(pool) = &self.pool {
            let ok: Option<bool> = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM fx_corp_token.company_basic_tokens \
                 WHERE corporate_user_id = $1 AND game_coin_id = $2 \
                   AND status = 'approved' AND buyable = TRUE)",
            )
            .bind(corp_id)
            .bind(game_coin_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
            return ok.unwrap_or(false);
        }
        let Some(mem) = &self.memory else {
            return false;
        };
        mem.tokens
            .read()
            .await
            .values()
            .any(|t| {
                t.corporate_user_id == corp_id
                    && t.game_coin_id == game_coin_id
                    && t.status == "approved"
                    && t.buyable
            })
    }

    pub async fn list_for_corp(&self, corp_id: i64) -> Vec<BasicTokenView> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                        status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                        approved_by_admin_id, approved_at, created_at, updated_at \
                 FROM fx_corp_token.company_basic_tokens \
                 WHERE corporate_user_id = $1 ORDER BY id",
            )
            .bind(corp_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows.into_iter().map(map_token).collect();
        }
        let Some(mem) = &self.memory else {
            return Vec::new();
        };
        let mut v: Vec<_> = mem
            .tokens
            .read()
            .await
            .values()
            .filter(|t| t.corporate_user_id == corp_id)
            .cloned()
            .collect();
        v.sort_by_key(|t| t.id);
        v
    }

    pub async fn get_owned(&self, corp_id: i64, id: i64) -> Option<BasicTokenView> {
        self.get(id)
            .await
            .filter(|t| t.corporate_user_id == corp_id)
    }

    pub async fn get(&self, id: i64) -> Option<BasicTokenView> {
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
            return Some(map_token(row));
        }
        let mem = self.memory.as_ref()?;
        mem.tokens.read().await.get(&id).cloned()
    }

    pub async fn create(
        &self,
        corp_id: i64,
        game_id: i64,
        game_coin_id: i64,
        token_code: &str,
        token_name: &str,
        status: &str,
    ) -> Result<BasicTokenView, String> {
        let code = token_code.trim().to_uppercase();
        let name = token_name.trim();
        if code.is_empty() || name.is_empty() {
            return Err("token_code and token_name required".into());
        }
        if !matches!(status, "submitted" | "pending") {
            return Err("status must be submitted or pending".into());
        }
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let row = sqlx::query(
                "INSERT INTO fx_corp_token.company_basic_tokens ( \
                    corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                    status, buyable, buy_fee_rate, approved_at, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, $6, FALSE, $7, 0, $8, 0) \
                 RETURNING id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                           status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                           approved_by_admin_id, approved_at, created_at, updated_at",
            )
            .bind(corp_id)
            .bind(game_id)
            .bind(game_coin_id)
            .bind(&code)
            .bind(name)
            .bind(status)
            .bind(DEFAULT_FEE_RATE)
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(map_token(row));
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let id = mem.next_token.fetch_add(1, Ordering::SeqCst);
        let rec = BasicTokenView {
            id,
            corporate_user_id: corp_id,
            game_id,
            game_coin_id,
            token_code: code,
            token_name: name.into(),
            status: status.into(),
            buyable: false,
            buy_fee_rate: DEFAULT_FEE_RATE,
            approved_by_admin_id: None,
            approved_at: 0,
            created_at: now,
            updated_at: 0,
        };
        mem.tokens.write().await.insert(id, rec.clone());
        Ok(rec)
    }

    pub async fn update_owned(
        &self,
        corp_id: i64,
        id: i64,
        game_id: Option<i64>,
        game_coin_id: Option<i64>,
        token_code: Option<&str>,
        token_name: Option<&str>,
        status: Option<&str>,
    ) -> Result<BasicTokenView, String> {
        let mut rec = self
            .get_owned(corp_id, id)
            .await
            .ok_or_else(|| "token not found".to_string())?;
        if rec.status == "approved" {
            return Err("approved token cannot be updated".into());
        }
        if !matches!(rec.status.as_str(), "submitted" | "pending" | "rejected") {
            return Err(format!("cannot update status={}", rec.status));
        }
        if let Some(g) = game_id {
            rec.game_id = g;
        }
        if let Some(c) = game_coin_id {
            rec.game_coin_id = c;
        }
        if let Some(code) = token_code {
            let c = code.trim().to_uppercase();
            if c.is_empty() {
                return Err("token_code required".into());
            }
            rec.token_code = c;
        }
        if let Some(name) = token_name {
            let n = name.trim();
            if n.is_empty() {
                return Err("token_name required".into());
            }
            rec.token_name = n.into();
        }
        if let Some(s) = status {
            if !matches!(s, "submitted" | "pending") {
                return Err("status must be submitted or pending".into());
            }
            rec.status = s.into();
            rec.buyable = false;
            rec.approved_by_admin_id = None;
            rec.approved_at = 0;
        } else if rec.status == "rejected" {
            // re-submit after edit
            rec.status = "submitted".into();
            rec.buyable = false;
            rec.approved_by_admin_id = None;
            rec.approved_at = 0;
        }
        let now = now_ms();
        rec.updated_at = now;
        if let Some(pool) = &self.pool {
            let row = sqlx::query(
                "UPDATE fx_corp_token.company_basic_tokens SET \
                    game_id = $1, game_coin_id = $2, token_code = $3, token_name = $4, \
                    status = $5, buyable = FALSE, approved_by_admin_id = NULL, approved_at = 0, \
                    updated_at = $6 \
                 WHERE id = $7 AND corporate_user_id = $8 \
                 RETURNING id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                           status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                           approved_by_admin_id, approved_at, created_at, updated_at",
            )
            .bind(rec.game_id)
            .bind(rec.game_coin_id)
            .bind(&rec.token_code)
            .bind(&rec.token_name)
            .bind(&rec.status)
            .bind(now)
            .bind(id)
            .bind(corp_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "token not found".to_string())?;
            return Ok(map_token(row));
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        mem.tokens.write().await.insert(id, rec.clone());
        Ok(rec)
    }

    pub async fn list_buyable(&self) -> Vec<BasicTokenView> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                        status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                        approved_by_admin_id, approved_at, created_at, updated_at \
                 FROM fx_corp_token.company_basic_tokens \
                 WHERE status = 'approved' AND buyable = TRUE ORDER BY id",
            )
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows.into_iter().map(map_token).collect();
        }
        let Some(mem) = &self.memory else {
            return Vec::new();
        };
        let mut v: Vec<_> = mem
            .tokens
            .read()
            .await
            .values()
            .filter(|t| t.status == "approved" && t.buyable)
            .cloned()
            .collect();
        v.sort_by_key(|t| t.id);
        v
    }

    pub async fn create_order(
        &self,
        token_id: i64,
        end_user_id: i64,
        game_account_id: i64,
        coin_amount: f64,
        partner_order_no: Option<&str>,
    ) -> Result<CbtOrderView, String> {
        if coin_amount <= 0.0 {
            return Err("coin_amount must be > 0".into());
        }
        let token = self
            .get(token_id)
            .await
            .ok_or_else(|| "token not found".to_string())?;
        if !token.buyable || token.status != "approved" {
            return Err("token is not buyable".into());
        }
        let fee = (coin_amount * token.buy_fee_rate * 1_000_000.0).round() / 1_000_000.0;
        let credited = coin_amount - fee;
        let now = now_ms();
        let expires = now + ORDER_TTL_MS;
        let partner_no = partner_order_no
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        if let Some(pool) = &self.pool {
            let row = sqlx::query(
                "INSERT INTO fx_corp_token.corp_token_orders ( \
                    company_basic_token_id, end_user_id, game_account_id, partner_order_no, \
                    pay_amount, pay_game_coin_id, coin_amount, fee_coin_amount, credited_coin_amount, \
                    game_coin_id, status, expires_at, paid_at, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, $6, $5, $7, $8, $6, 'pending', $9, 0, $10, 0) \
                 RETURNING id, company_basic_token_id, end_user_id, game_account_id, partner_order_no, \
                           pay_amount::float8 AS pay_amount, pay_game_coin_id, \
                           coin_amount::float8 AS coin_amount, fee_coin_amount::float8 AS fee_coin_amount, \
                           credited_coin_amount::float8 AS credited_coin_amount, game_coin_id, \
                           status, expires_at, paid_at, created_at, updated_at",
            )
            .bind(token_id)
            .bind(end_user_id)
            .bind(game_account_id)
            .bind(&partner_no)
            .bind(coin_amount)
            .bind(token.game_coin_id)
            .bind(fee)
            .bind(credited)
            .bind(expires)
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            let mut order = map_order(row);
            order.token_code = Some(token.token_code);
            order.corporate_user_id = Some(token.corporate_user_id);
            return Ok(order);
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let id = mem.next_order.fetch_add(1, Ordering::SeqCst);
        let order = CbtOrderView {
            id,
            company_basic_token_id: token_id,
            end_user_id,
            game_account_id,
            partner_order_no: partner_no,
            pay_amount: coin_amount,
            pay_game_coin_id: token.game_coin_id,
            coin_amount,
            fee_coin_amount: fee,
            credited_coin_amount: credited,
            game_coin_id: token.game_coin_id,
            status: "pending".into(),
            expires_at: expires,
            paid_at: 0,
            created_at: now,
            updated_at: 0,
            token_code: Some(token.token_code),
            corporate_user_id: Some(token.corporate_user_id),
        };
        mem.orders.write().await.insert(id, order.clone());
        Ok(order)
    }

    pub async fn list_orders_for_user(&self, end_user_id: i64) -> Vec<CbtOrderView> {
        self.expire_pending().await;
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT o.id, o.company_basic_token_id, o.end_user_id, o.game_account_id, o.partner_order_no, \
                        o.pay_amount::float8 AS pay_amount, o.pay_game_coin_id, \
                        o.coin_amount::float8 AS coin_amount, o.fee_coin_amount::float8 AS fee_coin_amount, \
                        o.credited_coin_amount::float8 AS credited_coin_amount, o.game_coin_id, \
                        o.status, o.expires_at, o.paid_at, o.created_at, o.updated_at, \
                        t.token_code, t.corporate_user_id \
                 FROM fx_corp_token.corp_token_orders o \
                 JOIN fx_corp_token.company_basic_tokens t ON t.id = o.company_basic_token_id \
                 WHERE o.end_user_id = $1 ORDER BY o.id DESC",
            )
            .bind(end_user_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows.into_iter().map(map_order_joined).collect();
        }
        let Some(mem) = &self.memory else {
            return Vec::new();
        };
        let mut v: Vec<_> = mem
            .orders
            .read()
            .await
            .values()
            .filter(|o| o.end_user_id == end_user_id)
            .cloned()
            .collect();
        v.sort_by_key(|o| std::cmp::Reverse(o.id));
        v
    }

    pub async fn get_order(&self, order_id: i64) -> Option<CbtOrderView> {
        self.expire_pending().await;
        if let Some(pool) = &self.pool {
            let row = sqlx::query(
                "SELECT o.id, o.company_basic_token_id, o.end_user_id, o.game_account_id, o.partner_order_no, \
                        o.pay_amount::float8 AS pay_amount, o.pay_game_coin_id, \
                        o.coin_amount::float8 AS coin_amount, o.fee_coin_amount::float8 AS fee_coin_amount, \
                        o.credited_coin_amount::float8 AS credited_coin_amount, o.game_coin_id, \
                        o.status, o.expires_at, o.paid_at, o.created_at, o.updated_at, \
                        t.token_code, t.corporate_user_id \
                 FROM fx_corp_token.corp_token_orders o \
                 JOIN fx_corp_token.company_basic_tokens t ON t.id = o.company_basic_token_id \
                 WHERE o.id = $1",
            )
            .bind(order_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()?;
            return Some(map_order_joined(row));
        }
        let mem = self.memory.as_ref()?;
        mem.orders.read().await.get(&order_id).cloned()
    }

    pub async fn cancel_order(&self, end_user_id: i64, order_id: i64) -> Result<CbtOrderView, String> {
        let mut order = self
            .get_order(order_id)
            .await
            .ok_or_else(|| "order not found".to_string())?;
        if order.end_user_id != end_user_id {
            return Err("order not found".into());
        }
        if order.status != "pending" {
            return Err(format!("cannot cancel status={}", order.status));
        }
        let now = now_ms();
        if let Some(pool) = &self.pool {
            sqlx::query(
                "UPDATE fx_corp_token.corp_token_orders SET status = 'cancelled', updated_at = $1 WHERE id = $2",
            )
            .bind(now)
            .bind(order_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }
        if let Some(mem) = &self.memory {
            if let Some(o) = mem.orders.write().await.get_mut(&order_id) {
                o.status = "cancelled".into();
                o.updated_at = now;
                order = o.clone();
            }
        } else {
            order.status = "cancelled".into();
            order.updated_at = now;
        }
        Ok(order)
    }

    pub async fn list_buy_orders_for_token(
        &self,
        corp_id: i64,
        token_id: i64,
    ) -> Result<Vec<CbtOrderView>, String> {
        let _ = self
            .get_owned(corp_id, token_id)
            .await
            .ok_or_else(|| "token not found".to_string())?;
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT o.id, o.company_basic_token_id, o.end_user_id, o.game_account_id, o.partner_order_no, \
                        o.pay_amount::float8 AS pay_amount, o.pay_game_coin_id, \
                        o.coin_amount::float8 AS coin_amount, o.fee_coin_amount::float8 AS fee_coin_amount, \
                        o.credited_coin_amount::float8 AS credited_coin_amount, o.game_coin_id, \
                        o.status, o.expires_at, o.paid_at, o.created_at, o.updated_at, \
                        t.token_code, t.corporate_user_id \
                 FROM fx_corp_token.corp_token_orders o \
                 JOIN fx_corp_token.company_basic_tokens t ON t.id = o.company_basic_token_id \
                 WHERE o.company_basic_token_id = $1 ORDER BY o.id DESC",
            )
            .bind(token_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(rows.into_iter().map(map_order_joined).collect());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut v: Vec<_> = mem
            .orders
            .read()
            .await
            .values()
            .filter(|o| o.company_basic_token_id == token_id)
            .cloned()
            .collect();
        v.sort_by_key(|o| std::cmp::Reverse(o.id));
        Ok(v)
    }

    pub async fn list_fees_for_token(
        &self,
        corp_id: i64,
        token_id: i64,
    ) -> Result<Vec<FeeLedgerView>, String> {
        let _ = self
            .get_owned(corp_id, token_id)
            .await
            .ok_or_else(|| "token not found".to_string())?;
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT id, corporate_user_id, company_basic_token_id, corp_token_order_id, game_coin_id, \
                        fee_rate::float8 AS fee_rate, gross_coin_amount::float8 AS gross_coin_amount, \
                        fee_coin_amount::float8 AS fee_coin_amount, net_coin_amount::float8 AS net_coin_amount, \
                        created_at \
                 FROM fx_corp_token.coin_fee_ledger \
                 WHERE company_basic_token_id = $1 ORDER BY id DESC",
            )
            .bind(token_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(rows.into_iter().map(map_fee).collect());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        Ok(mem
            .fees
            .read()
            .await
            .iter()
            .filter(|f| f.company_basic_token_id == token_id)
            .cloned()
            .collect())
    }

    pub async fn apply_settle(
        &self,
        corp_token_order_id: i64,
        partner_order_no: &str,
        status: &str,
        coin_amount: Option<f64>,
        event_id: &str,
    ) -> Result<CbtSettleResult, String> {
        let mut order = self
            .get_order(corp_token_order_id)
            .await
            .ok_or_else(|| format!("order {corp_token_order_id} not found"))?;
        if order.status == "paid" || order.status == "failed" || order.status == "cancelled" {
            let cb = self
                .partner_callback_url(order.corporate_user_id.unwrap_or(0))
                .await;
            return Ok(CbtSettleResult {
                corporate_user_id: order.corporate_user_id.unwrap_or(0),
                partner_callback_url: cb,
                order,
            });
        }
        if order.status != "pending" && order.status != "expired" {
            return Err(format!("cannot settle status={}", order.status));
        }
        if status == "paid" {
            if let Some(amt) = coin_amount {
                if (amt - order.coin_amount).abs() > 0.000_000_1 {
                    return Err("coin_amount mismatch".into());
                }
            }
        }
        let new_status = match status {
            "paid" => "paid",
            "failed" => "failed",
            "cancelled" => "cancelled",
            other => return Err(format!("unsupported settle status: {other}")),
        };
        let now = now_ms();
        let corp_id = order.corporate_user_id.unwrap_or(0);
        let token_id = order.company_basic_token_id;

        if let Some(pool) = &self.pool {
            let paid_at = if new_status == "paid" { now } else { 0_i64 };
            sqlx::query(
                "UPDATE fx_corp_token.corp_token_orders SET status = $1, paid_at = $2, \
                    partner_order_no = COALESCE($3, partner_order_no), updated_at = $4 \
                 WHERE id = $5",
            )
            .bind(new_status)
            .bind(paid_at)
            .bind(if partner_order_no.is_empty() {
                None
            } else {
                Some(partner_order_no)
            })
            .bind(now)
            .bind(corp_token_order_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            let _ = sqlx::query(
                "INSERT INTO fx_corp_token.corp_token_payment_events \
                    (corp_token_order_id, event_id, event_type, partner_order_no, status, payload, received_at, created_at) \
                 VALUES ($1, $2, 'corp_token_payment', $3, $4, $5::jsonb, $6, $6) \
                 ON CONFLICT (event_id) DO NOTHING",
            )
            .bind(corp_token_order_id)
            .bind(event_id)
            .bind(partner_order_no)
            .bind(new_status)
            .bind(
                serde_json::json!({
                    "status": new_status,
                    "coin_amount": order.coin_amount,
                })
                .to_string(),
            )
            .bind(now)
            .execute(pool)
            .await;

            if new_status == "paid" {
                sqlx::query(
                    "INSERT INTO fx_user.user_wallets (game_account_id, game_coin_id, available, locked, updated_at) \
                     VALUES ($1, $2, $3, 0, $4) \
                     ON CONFLICT (game_account_id, game_coin_id) DO UPDATE SET \
                        available = fx_user.user_wallets.available + EXCLUDED.available, \
                        updated_at = EXCLUDED.updated_at",
                )
                .bind(order.game_account_id)
                .bind(order.game_coin_id)
                .bind(order.credited_coin_amount)
                .bind(now)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;

                let _ = sqlx::query(
                    "INSERT INTO fx_corp_token.coin_fee_ledger ( \
                        corporate_user_id, company_basic_token_id, corp_token_order_id, game_coin_id, \
                        fee_rate, gross_coin_amount, fee_coin_amount, net_coin_amount, created_at \
                     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                )
                .bind(corp_id)
                .bind(token_id)
                .bind(corp_token_order_id)
                .bind(order.game_coin_id)
                .bind(DEFAULT_FEE_RATE)
                .bind(order.coin_amount)
                .bind(order.fee_coin_amount)
                .bind(order.credited_coin_amount)
                .bind(now)
                .execute(pool)
                .await;
            }
        }

        if let Some(mem) = &self.memory {
            if let Some(o) = mem.orders.write().await.get_mut(&corp_token_order_id) {
                o.status = new_status.into();
                o.paid_at = if new_status == "paid" { now } else { o.paid_at };
                if !partner_order_no.is_empty() {
                    o.partner_order_no = Some(partner_order_no.into());
                }
                o.updated_at = now;
                order = o.clone();
            }
            if new_status == "paid" {
                let fee_id = mem.next_fee.fetch_add(1, Ordering::SeqCst);
                mem.fees.write().await.push(FeeLedgerView {
                    id: fee_id,
                    corporate_user_id: corp_id,
                    company_basic_token_id: token_id,
                    corp_token_order_id: Some(corp_token_order_id),
                    game_coin_id: order.game_coin_id,
                    fee_rate: DEFAULT_FEE_RATE,
                    gross_coin_amount: order.coin_amount,
                    fee_coin_amount: order.fee_coin_amount,
                    net_coin_amount: order.credited_coin_amount,
                    created_at: now,
                });
            }
        } else {
            order.status = new_status.into();
            order.paid_at = if new_status == "paid" { now } else { order.paid_at };
            order.updated_at = now;
            if !partner_order_no.is_empty() {
                order.partner_order_no = Some(partner_order_no.into());
            }
        }

        let cb = self.partner_callback_url(corp_id).await;
        Ok(CbtSettleResult {
            order,
            partner_callback_url: cb,
            corporate_user_id: corp_id,
        })
    }

    async fn partner_callback_url(&self, corporate_user_id: i64) -> Option<String> {
        if corporate_user_id <= 0 {
            return None;
        }
        if let Some(pool) = &self.pool {
            if let Ok(Some(url)) = sqlx::query_scalar::<_, String>(
                "SELECT callback_url FROM fx_events.webhook_endpoints \
                 WHERE corporate_user_id = $1 AND kind = 'corp_token' AND status = 'active' \
                 LIMIT 1",
            )
            .bind(corporate_user_id)
            .fetch_optional(pool)
            .await
            {
                return Some(url);
            }
            if let Ok(Some(url)) = sqlx::query_scalar::<_, String>(
                "SELECT endpoint FROM fx_corp.partner_endpoints \
                 WHERE corporate_user_id = $1 AND type = 'callback' AND status = 'active' \
                 LIMIT 1",
            )
            .bind(corporate_user_id)
            .fetch_optional(pool)
            .await
            {
                return Some(url);
            }
            return None;
        }
        let mem = self.memory.as_ref()?;
        mem.callbacks.read().await.get(&corporate_user_id).cloned()
    }

    async fn expire_pending(&self) {
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let _ = sqlx::query(
                "UPDATE fx_corp_token.corp_token_orders SET status = 'expired', updated_at = $1 \
                 WHERE status = 'pending' AND expires_at <= $1",
            )
            .bind(now)
            .execute(pool)
            .await;
        }
        if let Some(mem) = &self.memory {
            for o in mem.orders.write().await.values_mut() {
                if o.status == "pending" && o.expires_at <= now {
                    o.status = "expired".into();
                }
            }
        }
    }
}

fn demo_token() -> BasicTokenView {
    BasicTokenView {
        id: 1,
        corporate_user_id: 1,
        game_id: 1,
        game_coin_id: 2,
        token_code: "DEMO_CBT".into(),
        token_name: "Demo Company Basic Token".into(),
        status: "approved".into(),
        buyable: true,
        buy_fee_rate: DEFAULT_FEE_RATE,
        approved_by_admin_id: Some(1),
        approved_at: 1,
        created_at: 1,
        updated_at: 0,
    }
}

fn map_token(row: sqlx::postgres::PgRow) -> BasicTokenView {
    BasicTokenView {
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

fn map_order(row: sqlx::postgres::PgRow) -> CbtOrderView {
    CbtOrderView {
        id: row.get("id"),
        company_basic_token_id: row.get("company_basic_token_id"),
        end_user_id: row.get("end_user_id"),
        game_account_id: row.get("game_account_id"),
        partner_order_no: row.get("partner_order_no"),
        pay_amount: row.get("pay_amount"),
        pay_game_coin_id: row.get("pay_game_coin_id"),
        coin_amount: row.get("coin_amount"),
        fee_coin_amount: row.get("fee_coin_amount"),
        credited_coin_amount: row.get("credited_coin_amount"),
        game_coin_id: row.get("game_coin_id"),
        status: row.get("status"),
        expires_at: row.get("expires_at"),
        paid_at: row.get("paid_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        token_code: None,
        corporate_user_id: None,
    }
}

fn map_order_joined(row: sqlx::postgres::PgRow) -> CbtOrderView {
    CbtOrderView {
        id: row.get("id"),
        company_basic_token_id: row.get("company_basic_token_id"),
        end_user_id: row.get("end_user_id"),
        game_account_id: row.get("game_account_id"),
        partner_order_no: row.get("partner_order_no"),
        pay_amount: row.get("pay_amount"),
        pay_game_coin_id: row.get("pay_game_coin_id"),
        coin_amount: row.get("coin_amount"),
        fee_coin_amount: row.get("fee_coin_amount"),
        credited_coin_amount: row.get("credited_coin_amount"),
        game_coin_id: row.get("game_coin_id"),
        status: row.get("status"),
        expires_at: row.get("expires_at"),
        paid_at: row.get("paid_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        token_code: row.get("token_code"),
        corporate_user_id: row.get("corporate_user_id"),
    }
}

fn map_fee(row: sqlx::postgres::PgRow) -> FeeLedgerView {
    FeeLedgerView {
        id: row.get("id"),
        corporate_user_id: row.get("corporate_user_id"),
        company_basic_token_id: row.get("company_basic_token_id"),
        corp_token_order_id: row.get("corp_token_order_id"),
        game_coin_id: row.get("game_coin_id"),
        fee_rate: row.get("fee_rate"),
        gross_coin_amount: row.get("gross_coin_amount"),
        fee_coin_amount: row.get("fee_coin_amount"),
        net_coin_amount: row.get("net_coin_amount"),
        created_at: row.get("created_at"),
    }
}
