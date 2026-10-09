//! In-memory + PostgreSQL catalog/orders for Client Center e-shop.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use serde::Serialize;
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

use crate::db::now_ms;

/// Pending shop orders remain payable for 24 hours, then expire.
const ORDER_TTL_MS: i64 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, Serialize)]
pub struct PackageView {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub game_coin_id: i64,
    pub credit_game_coin: String,
    pub coin_amount: f64,
    pub fiat_price: f64,
    pub seller_type: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorpProductView {
    pub id: i64,
    pub corporate_user_id: i64,
    pub game_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub product_type: String,
    pub credit_game_coin_id: Option<i64>,
    pub credit_game_coin: Option<String>,
    pub credit_amount: Option<f64>,
    pub item_code: Option<String>,
    pub fiat_price: f64,
    pub seller_type: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShopOrderView {
    pub id: i64,
    pub end_user_id: i64,
    pub game_account_id: i64,
    pub seller_type: String,
    pub package_id: Option<i64>,
    pub corp_product_id: Option<i64>,
    pub package_code: Option<String>,
    pub product_code: Option<String>,
    pub credit_game_coin: Option<String>,
    pub credit_amount: Option<f64>,
    pub item_code: Option<String>,
    pub fiat_currency: String,
    pub fiat_price: f64,
    pub partner_order_no: Option<String>,
    pub checkout_url: Option<String>,
    pub status: String,
    pub expires_at: i64,
    pub paid_at: i64,
    pub created_at: i64,
    #[serde(skip)]
    pub game_coin_id: Option<i64>,
    #[serde(skip)]
    pub corporate_user_id: Option<i64>,
}

#[derive(Clone)]
pub struct ShopStore {
    pool: Option<PgPool>,
    memory: Option<Arc<MemoryShop>>,
}

struct MemoryShop {
    packages: Vec<PackageView>,
    products: RwLock<HashMap<i64, CorpProductView>>,
    next_product: AtomicI64,
    orders: RwLock<HashMap<i64, ShopOrderView>>,
    next_order: AtomicI64,
    base_fiat: String,
}

impl ShopStore {
    pub fn memory() -> Self {
        Self {
            pool: None,
            memory: Some(Arc::new(MemoryShop {
                packages: demo_packages(),
                products: {
                    let mut m = HashMap::new();
                    for p in demo_corp_products() {
                        m.insert(p.id, p);
                    }
                    RwLock::new(m)
                },
                next_product: AtomicI64::new(2),
                orders: RwLock::new(HashMap::new()),
                next_order: AtomicI64::new(5001),
                base_fiat: "HKD".into(),
            })),
        }
    }

    pub fn postgres(pool: PgPool) -> Self {
        Self {
            pool: Some(pool),
            memory: None,
        }
    }

    pub async fn base_fiat(&self) -> String {
        if let Some(pool) = &self.pool {
            if let Ok(Some(v)) = sqlx::query_scalar::<_, String>(
                "SELECT setting_value FROM fx_config.system_settings \
                 WHERE setting_key = 'base_fiat_currency'",
            )
            .fetch_optional(pool)
            .await
            {
                let u = v.to_ascii_uppercase();
                if u == "HKD" || u == "USD" {
                    return u;
                }
            }
        }
        self.memory
            .as_ref()
            .map(|m| m.base_fiat.clone())
            .unwrap_or_else(|| "HKD".into())
    }

    pub async fn list_packages(&self) -> Vec<PackageView> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT p.id, p.code, p.name, p.game_coin_id, c.code AS credit_game_coin, \
                        p.coin_amount::float8 AS coin_amount, p.fiat_price::float8 AS fiat_price, \
                        p.seller_type, p.status \
                 FROM fx_shop.shop_packages p \
                 JOIN fx_game.game_coins c ON c.id = p.game_coin_id \
                 WHERE p.status = 'active' \
                 ORDER BY p.id",
            )
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows.into_iter().map(map_package).collect();
        }
        self.memory
            .as_ref()
            .map(|m| m.packages.clone())
            .unwrap_or_default()
    }

    pub async fn get_package(&self, id: i64) -> Option<PackageView> {
        self.list_packages().await.into_iter().find(|p| p.id == id)
    }

    pub async fn list_corp_products(&self) -> Vec<CorpProductView> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT pr.id, pr.corporate_user_id, pr.game_id, pr.code, pr.name, pr.product_type, \
                        pr.credit_game_coin_id, gc.code AS credit_game_coin, \
                        pr.credit_amount::float8 AS credit_amount, pr.item_code, \
                        pr.fiat_price::float8 AS fiat_price, pr.seller_type, pr.status \
                 FROM fx_shop.corp_shop_products pr \
                 LEFT JOIN fx_game.game_coins gc ON gc.id = pr.credit_game_coin_id \
                 WHERE pr.status = 'active' \
                 ORDER BY pr.id",
            )
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows.into_iter().map(map_corp).collect();
        }
        let Some(mem) = &self.memory else {
            return Vec::new();
        };
        let mut v: Vec<_> = mem
            .products
            .read()
            .await
            .values()
            .filter(|p| p.status == "active")
            .cloned()
            .collect();
        v.sort_by_key(|p| p.id);
        v
    }

    pub async fn get_corp_product(&self, id: i64) -> Option<CorpProductView> {
        self.list_corp_products()
            .await
            .into_iter()
            .find(|p| p.id == id)
    }

    pub async fn list_corp_products_owned(&self, corporate_user_id: i64) -> Vec<CorpProductView> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT pr.id, pr.corporate_user_id, pr.game_id, pr.code, pr.name, pr.product_type, \
                        pr.credit_game_coin_id, gc.code AS credit_game_coin, \
                        pr.credit_amount::float8 AS credit_amount, pr.item_code, \
                        pr.fiat_price::float8 AS fiat_price, pr.seller_type, pr.status \
                 FROM fx_shop.corp_shop_products pr \
                 LEFT JOIN fx_game.game_coins gc ON gc.id = pr.credit_game_coin_id \
                 WHERE pr.corporate_user_id = $1 \
                 ORDER BY pr.id",
            )
            .bind(corporate_user_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows.into_iter().map(map_corp).collect();
        }
        let Some(mem) = &self.memory else {
            return Vec::new();
        };
        let mut v: Vec<_> = mem
            .products
            .read()
            .await
            .values()
            .filter(|p| p.corporate_user_id == corporate_user_id)
            .cloned()
            .collect();
        v.sort_by_key(|p| p.id);
        v
    }

    pub async fn get_corp_product_owned(
        &self,
        corporate_user_id: i64,
        id: i64,
    ) -> Option<CorpProductView> {
        self.list_corp_products_owned(corporate_user_id)
            .await
            .into_iter()
            .find(|p| p.id == id)
    }

    pub async fn create_corp_product(
        &self,
        mut rec: CorpProductView,
    ) -> Result<CorpProductView, String> {
        validate_corp_product(&rec, rec.status == "active")?;
        rec.seller_type = "corp".into();
        if rec.status.is_empty() {
            rec.status = "draft".into();
        }
        if let Some(pool) = &self.pool {
            let now = now_ms();
            let row = sqlx::query(
                "INSERT INTO fx_shop.corp_shop_products ( \
                    corporate_user_id, game_id, code, name, product_type, credit_game_coin_id, \
                    credit_amount, item_code, fiat_price, seller_type, status, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'corp', $10, $11, 0) \
                 RETURNING id",
            )
            .bind(rec.corporate_user_id)
            .bind(rec.game_id)
            .bind(&rec.code)
            .bind(&rec.name)
            .bind(&rec.product_type)
            .bind(rec.credit_game_coin_id)
            .bind(rec.credit_amount)
            .bind(&rec.item_code)
            .bind(rec.fiat_price)
            .bind(&rec.status)
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            rec.id = row.get("id");
            return Ok(rec);
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        rec.id = mem.next_product.fetch_add(1, Ordering::Relaxed);
        mem.products.write().await.insert(rec.id, rec.clone());
        Ok(rec)
    }

    pub async fn update_corp_product(
        &self,
        corporate_user_id: i64,
        id: i64,
        patch: CorpProductView,
    ) -> Result<CorpProductView, String> {
        let mut rec = self
            .get_corp_product_owned(corporate_user_id, id)
            .await
            .ok_or_else(|| "product not found".to_string())?;
        if rec.status != "draft" && rec.status != "inactive" {
            return Err(format!("cannot update status={}", rec.status));
        }
        rec.game_id = patch.game_id;
        rec.name = patch.name;
        rec.product_type = patch.product_type;
        rec.credit_game_coin_id = patch.credit_game_coin_id;
        rec.credit_game_coin = patch.credit_game_coin;
        rec.credit_amount = patch.credit_amount;
        rec.item_code = patch.item_code;
        rec.fiat_price = patch.fiat_price;
        if !patch.code.is_empty() {
            rec.code = patch.code;
        }
        validate_corp_product(&rec, false)?;
        if let Some(pool) = &self.pool {
            sqlx::query(
                "UPDATE fx_shop.corp_shop_products SET game_id = $1, code = $2, name = $3, product_type = $4, \
                    credit_game_coin_id = $5, credit_amount = $6, item_code = $7, fiat_price = $8, updated_at = $9 \
                 WHERE id = $10 AND corporate_user_id = $11",
            )
            .bind(rec.game_id)
            .bind(&rec.code)
            .bind(&rec.name)
            .bind(&rec.product_type)
            .bind(rec.credit_game_coin_id)
            .bind(rec.credit_amount)
            .bind(&rec.item_code)
            .bind(rec.fiat_price)
            .bind(now_ms())
            .bind(id)
            .bind(corporate_user_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(rec);
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        mem.products.write().await.insert(id, rec.clone());
        Ok(rec)
    }

    pub async fn set_corp_product_status(
        &self,
        corporate_user_id: i64,
        id: i64,
        status: &str,
    ) -> Result<CorpProductView, String> {
        match status {
            "draft" | "active" | "inactive" | "archived" => {}
            other => return Err(format!("invalid status: {other}")),
        }
        let rec = self
            .get_corp_product_owned(corporate_user_id, id)
            .await
            .ok_or_else(|| "product not found".to_string())?;
        if status == "active" {
            validate_corp_product(&rec, true)?;
        }
        if let Some(pool) = &self.pool {
            sqlx::query(
                "UPDATE fx_shop.corp_shop_products SET status = $1, updated_at = $2 \
                 WHERE id = $3 AND corporate_user_id = $4",
            )
            .bind(status)
            .bind(now_ms())
            .bind(id)
            .bind(corporate_user_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            return self
                .get_corp_product_owned(corporate_user_id, id)
                .await
                .ok_or_else(|| "product not found".into());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut g = mem.products.write().await;
        let rec = g.get_mut(&id).ok_or_else(|| "product not found".to_string())?;
        if rec.corporate_user_id != corporate_user_id {
            return Err("product not found".into());
        }
        rec.status = status.into();
        Ok(rec.clone())
    }

    pub async fn list_orders(&self, end_user_id: i64) -> Vec<ShopOrderView> {
        self.expire_pending().await;
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(&format!("{ORDER_SELECT} ORDER BY o.id DESC"))
                .bind(end_user_id)
                .fetch_all(pool)
                .await
                .unwrap_or_default();
            return rows.into_iter().map(map_order).collect();
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
        v.sort_by(|a, b| b.id.cmp(&a.id));
        v
    }

    pub async fn get_order(&self, end_user_id: i64, id: i64) -> Option<ShopOrderView> {
        self.expire_pending().await;
        if let Some(pool) = &self.pool {
            return sqlx::query(&format!("{ORDER_SELECT} AND o.id = $2"))
                .bind(end_user_id)
                .bind(id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
                .map(map_order);
        }
        let mem = self.memory.as_ref()?;
        let rec = mem.orders.read().await.get(&id).cloned()?;
        (rec.end_user_id == end_user_id).then_some(rec)
    }

    pub async fn get_order_by_id(&self, id: i64) -> Option<ShopOrderView> {
        self.expire_pending().await;
        if let Some(pool) = &self.pool {
            return sqlx::query(
                "SELECT o.id, o.end_user_id, o.game_account_id, o.seller_type, \
                        o.package_id, o.corp_product_id, p.code AS package_code, c.code AS product_code, \
                        gc.code AS credit_game_coin, o.credit_amount::float8 AS credit_amount, o.item_code, \
                        o.fiat_currency, o.fiat_price::float8 AS fiat_price, o.partner_order_no, \
                        o.status, o.expires_at, o.paid_at, o.created_at, o.game_coin_id, o.corporate_user_id \
                 FROM fx_shop.shop_orders o \
                 LEFT JOIN fx_shop.shop_packages p ON p.id = o.package_id \
                 LEFT JOIN fx_shop.corp_shop_products c ON c.id = o.corp_product_id \
                 LEFT JOIN fx_game.game_coins gc ON gc.id = o.game_coin_id \
                 WHERE o.id = $1",
            )
            .bind(id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .map(map_order_full);
        }
        let mem = self.memory.as_ref()?;
        mem.orders.read().await.get(&id).cloned()
    }

    /// Apply partner webhook status. Idempotent if already paid/failed/cancelled.
    pub async fn apply_settle(
        &self,
        shop_order_id: i64,
        partner_order_no: &str,
        status: &str,
        fiat_currency: &str,
        fiat_paid: f64,
        event_id: &str,
    ) -> Result<ShopOrderView, String> {
        let mut order = self
            .get_order_by_id(shop_order_id)
            .await
            .ok_or_else(|| format!("order {shop_order_id} not found"))?;
        if order.status == "paid" || order.status == "failed" || order.status == "cancelled" {
            // Backfill money txn + status log if a prior paid settle credited wallet but failed mid-way.
            if order.status == "paid" {
                if let (Some(pool), Some(coin_id)) = (&self.pool, order.game_coin_id) {
                    let txn_type = if order.seller_type == "corp" {
                        "corp_shop_purchase"
                    } else {
                        "shop_topup"
                    };
                    insert_money_txn_with_status_log(
                        pool,
                        order.end_user_id,
                        order.game_account_id,
                        order.corporate_user_id,
                        shop_order_id,
                        txn_type,
                        order.credit_amount.unwrap_or(0.0),
                        coin_id,
                        event_id,
                        partner_order_no,
                        now_ms(),
                    )
                    .await
                    .map_err(|e| format!("money txn/status log: {e}"))?;
                }
            }
            return Ok(order);
        }
        if order.status != "pending" && order.status != "expired" {
            return Err(format!("cannot settle status={}", order.status));
        }
        if status == "paid" {
            if order.fiat_currency != fiat_currency {
                return Err("fiat_currency mismatch".into());
            }
            let snap = order.fiat_price;
            if (fiat_paid - snap).abs() > 0.0001 {
                return Err("fiat_paid mismatch".into());
            }
        }
        let now = now_ms();
        let new_status = match status {
            "paid" => "paid",
            "failed" => "failed",
            "cancelled" => "cancelled",
            other => return Err(format!("unsupported settle status: {other}")),
        };
        if let Some(pool) = &self.pool {
            let paid_at = if new_status == "paid" { now } else { 0_i64 };
            sqlx::query(
                "UPDATE fx_shop.shop_orders SET status = $1, paid_at = $2, partner_order_no = COALESCE($3, partner_order_no), updated_at = $4 \
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
            .bind(shop_order_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            let _ = sqlx::query(
                "INSERT INTO fx_shop.shop_payment_events \
                    (shop_order_id, event_id, event_type, partner_order_no, status, payload, received_at, created_at) \
                 VALUES ($1, $2, 'shop_payment', $3, $4, $5::jsonb, $6, $6) \
                 ON CONFLICT (event_id) DO NOTHING",
            )
            .bind(shop_order_id)
            .bind(event_id)
            .bind(partner_order_no)
            .bind(new_status)
            .bind(serde_json::json!({"status": new_status, "fiat_paid": fiat_paid}).to_string())
            .bind(now)
            .execute(pool)
            .await;
            if new_status == "paid" {
                if let Some(coin_id) = order.game_coin_id {
                    let amt = order.credit_amount.unwrap_or(0.0);
                    if amt > 0.0 {
                        sqlx::query(
                            "INSERT INTO fx_user.user_wallets (game_account_id, game_coin_id, available, locked, updated_at) \
                             VALUES ($1, $2, $3, 0, $4) \
                             ON CONFLICT (game_account_id, game_coin_id) DO UPDATE SET \
                                available = fx_user.user_wallets.available + EXCLUDED.available, \
                                updated_at = EXCLUDED.updated_at",
                        )
                        .bind(order.game_account_id)
                        .bind(coin_id)
                        .bind(amt)
                        .bind(now)
                        .execute(pool)
                        .await
                        .map_err(|e| e.to_string())?;
                    }
                }
                let txn_type = if order.seller_type == "corp" {
                    "corp_shop_purchase"
                } else {
                    "shop_topup"
                };
                if let Some(coin_id) = order.game_coin_id {
                    insert_money_txn_with_status_log(
                        pool,
                        order.end_user_id,
                        order.game_account_id,
                        order.corporate_user_id,
                        shop_order_id,
                        txn_type,
                        order.credit_amount.unwrap_or(0.0),
                        coin_id,
                        event_id,
                        partner_order_no,
                        now,
                    )
                    .await
                    .map_err(|e| format!("money txn/status log: {e}"))?;
                }
            }
        }
        if let Some(mem) = &self.memory {
            if let Some(o) = mem.orders.write().await.get_mut(&shop_order_id) {
                o.status = new_status.into();
                o.paid_at = if new_status == "paid" { now } else { o.paid_at };
                if !partner_order_no.is_empty() {
                    o.partner_order_no = Some(partner_order_no.into());
                }
                order = o.clone();
            }
        } else {
            order.status = new_status.into();
            if new_status == "paid" {
                order.paid_at = now;
            }
        }
        Ok(order)
    }

    pub async fn insert_order(&self, mut order: ShopOrderView) -> Result<ShopOrderView, String> {
        if let Some(pool) = &self.pool {
            let row = sqlx::query(
                "INSERT INTO fx_shop.shop_orders ( \
                    game_account_id, end_user_id, seller_type, package_id, corp_product_id, \
                    corporate_user_id, partner_order_no, fiat_currency, fiat_price, \
                    credit_amount, game_coin_id, item_code, status, expires_at, paid_at, \
                    created_at, updated_at \
                 ) VALUES ( \
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'pending', $13, 0, $14, 0 \
                 ) RETURNING id",
            )
            .bind(order.game_account_id)
            .bind(order.end_user_id)
            .bind(&order.seller_type)
            .bind(order.package_id)
            .bind(order.corp_product_id)
            .bind(order.corporate_user_id)
            .bind(&order.partner_order_no)
            .bind(&order.fiat_currency)
            .bind(order.fiat_price)
            .bind(order.credit_amount)
            .bind(order.game_coin_id)
            .bind(&order.item_code)
            .bind(order.expires_at)
            .bind(order.created_at)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            order.id = row.get("id");
            return Ok(order);
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let id = mem.next_order.fetch_add(1, Ordering::Relaxed);
        order.id = id;
        mem.orders.write().await.insert(id, order.clone());
        Ok(order)
    }

    pub async fn set_payment(
        &self,
        id: i64,
        partner_order_no: &str,
        checkout_url: &str,
    ) -> Result<(), String> {
        if let Some(pool) = &self.pool {
            sqlx::query(
                "UPDATE fx_shop.shop_orders \
                 SET partner_order_no = $1, updated_at = $2 WHERE id = $3",
            )
            .bind(partner_order_no)
            .bind(now_ms())
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }
        if let Some(mem) = &self.memory {
            if let Some(o) = mem.orders.write().await.get_mut(&id) {
                o.partner_order_no = Some(partner_order_no.to_string());
                o.checkout_url = Some(checkout_url.to_string());
            }
        }
        Ok(())
    }

    pub async fn cancel(&self, end_user_id: i64, id: i64) -> Result<ShopOrderView, String> {
        let Some(mut rec) = self.get_order(end_user_id, id).await else {
            return Err("order not found".into());
        };
        if rec.status != "pending" {
            return Err(format!("cannot cancel status={}", rec.status));
        }
        rec.status = "cancelled".into();
        rec.checkout_url = rec.checkout_url.clone();
        if let Some(pool) = &self.pool {
            sqlx::query(
                "UPDATE fx_shop.shop_orders SET status = 'cancelled', updated_at = $1 WHERE id = $2 \
                 AND end_user_id = $3 AND status = 'pending'",
            )
            .bind(now_ms())
            .bind(id)
            .bind(end_user_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }
        if let Some(mem) = &self.memory {
            mem.orders.write().await.insert(id, rec.clone());
        }
        Ok(rec)
    }

    async fn expire_pending(&self) {
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let _ = sqlx::query(
                "UPDATE fx_shop.shop_orders SET status = 'expired', updated_at = $1 \
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

pub fn order_ttl_ms() -> i64 {
    ORDER_TTL_MS
}

/// Insert money txn as `completed` and append `deposit_withdrawal_status_logs`.
/// Idempotent per `(shop_order_id, transaction_type)` for settle retries.
async fn insert_money_txn_with_status_log(
    pool: &PgPool,
    end_user_id: i64,
    game_account_id: i64,
    corporate_user_id: Option<i64>,
    shop_order_id: i64,
    transaction_type: &str,
    amount: f64,
    game_coin_id: i64,
    event_id: &str,
    partner_order_no: &str,
    now: i64,
) -> Result<i64, String> {
    if let Some(existing_id) = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM fx_money.deposit_withdrawal_txns \
         WHERE shop_order_id = $1 AND transaction_type = $2 \
         ORDER BY id LIMIT 1",
    )
    .bind(shop_order_id)
    .bind(transaction_type)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    {
        ensure_completed_status_log(pool, existing_id, shop_order_id, transaction_type, event_id, partner_order_no, amount, game_coin_id, now).await?;
        return Ok(existing_id);
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let txn_id: i64 = sqlx::query_scalar(
        "INSERT INTO fx_money.deposit_withdrawal_txns ( \
            end_user_id, game_account_id, corporate_user_id, shop_order_id, \
            transaction_type, amount, game_coin_id, status, created_at, updated_at \
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'completed', $8, 0) \
         RETURNING id",
    )
    .bind(end_user_id)
    .bind(game_account_id)
    .bind(corporate_user_id)
    .bind(shop_order_id)
    .bind(transaction_type)
    .bind(amount)
    .bind(game_coin_id)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    insert_completed_status_log(
        &mut *tx,
        txn_id,
        shop_order_id,
        transaction_type,
        event_id,
        partner_order_no,
        amount,
        game_coin_id,
        now,
    )
    .await?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(txn_id)
}

async fn ensure_completed_status_log(
    pool: &PgPool,
    txn_id: i64,
    shop_order_id: i64,
    transaction_type: &str,
    event_id: &str,
    partner_order_no: &str,
    amount: f64,
    game_coin_id: i64,
    now: i64,
) -> Result<(), String> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS( \
            SELECT 1 FROM fx_money.deposit_withdrawal_status_logs \
            WHERE deposit_withdrawal_txn_id = $1 AND event_type = 'completed' \
         )",
    )
    .bind(txn_id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    if exists {
        return Ok(());
    }
    insert_completed_status_log(
        pool,
        txn_id,
        shop_order_id,
        transaction_type,
        event_id,
        partner_order_no,
        amount,
        game_coin_id,
        now,
    )
    .await
}

async fn insert_completed_status_log<'e, E>(
    executor: E,
    txn_id: i64,
    shop_order_id: i64,
    transaction_type: &str,
    event_id: &str,
    partner_order_no: &str,
    amount: f64,
    game_coin_id: i64,
    now: i64,
) -> Result<(), String>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let payload = serde_json::json!({
        "shop_order_id": shop_order_id,
        "event_id": event_id,
        "partner_order_no": partner_order_no,
        "transaction_type": transaction_type,
        "amount": amount,
        "game_coin_id": game_coin_id,
    });
    let note = format!("shop settle paid → {transaction_type} (order {shop_order_id})");
    sqlx::query(
        "INSERT INTO fx_money.deposit_withdrawal_status_logs ( \
            deposit_withdrawal_txn_id, old_status, new_status, event_type, note, payload, created_at \
         ) VALUES ($1, NULL, 'completed', 'completed', $2, $3::jsonb, $4)",
    )
    .bind(txn_id)
    .bind(&note)
    .bind(payload.to_string())
    .bind(now)
    .execute(executor)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn validate_corp_product(rec: &CorpProductView, for_active: bool) -> Result<(), String> {
    if rec.code.trim().is_empty() || rec.name.trim().is_empty() {
        return Err("code and name required".into());
    }
    if rec.fiat_price <= 0.0 {
        return Err("fiat_price must be > 0".into());
    }
    match rec.product_type.as_str() {
        "game_coin_package" | "company_coin_package" => {
            if rec.credit_game_coin_id.is_none() || rec.credit_amount.unwrap_or(0.0) <= 0.0 {
                return Err("coin package requires credit_game_coin_id and credit_amount".into());
            }
        }
        "game_item" => {
            if rec.item_code.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
                return Err("game_item requires item_code".into());
            }
        }
        other => return Err(format!("invalid product_type: {other}")),
    }
    let _ = for_active;
    Ok(())
}

fn demo_packages() -> Vec<PackageView> {
    [
        (1, "PLT_1000", "Platform Token 1000", 1000.0, 8.8),
        (2, "PLT_1500", "Platform Token 1500", 1500.0, 13.0),
        (3, "PLT_3000", "Platform Token 3000", 3000.0, 27.0),
        (4, "PLT_10000", "Platform Token 10000", 10000.0, 75.0),
    ]
    .into_iter()
    .map(|(id, code, name, amt, price)| PackageView {
        id,
        code: code.into(),
        name: name.into(),
        game_coin_id: 1,
        credit_game_coin: "PLT".into(),
        coin_amount: amt,
        fiat_price: price,
        seller_type: "platform".into(),
        status: "active".into(),
    })
    .collect()
}

fn demo_corp_products() -> Vec<CorpProductView> {
    vec![CorpProductView {
        id: 1,
        corporate_user_id: 1,
        game_id: Some(1),
        code: "DEMO_PACK_100".into(),
        name: "Demo Coin Pack 100".into(),
        product_type: "company_coin_package".into(),
        credit_game_coin_id: Some(2),
        credit_game_coin: Some("DEMO_COIN".into()),
        credit_amount: Some(100.0),
        item_code: None,
        fiat_price: 9.9,
        seller_type: "corp".into(),
        status: "active".into(),
    }]
}

const ORDER_SELECT: &str = "SELECT o.id, o.end_user_id, o.game_account_id, o.seller_type, \
    o.package_id, o.corp_product_id, p.code AS package_code, c.code AS product_code, \
    gc.code AS credit_game_coin, o.credit_amount::float8 AS credit_amount, o.item_code, \
    o.fiat_currency, o.fiat_price::float8 AS fiat_price, o.partner_order_no, \
    o.status, o.expires_at, o.paid_at, o.created_at \
 FROM fx_shop.shop_orders o \
 LEFT JOIN fx_shop.shop_packages p ON p.id = o.package_id \
 LEFT JOIN fx_shop.corp_shop_products c ON c.id = o.corp_product_id \
 LEFT JOIN fx_game.game_coins gc ON gc.id = o.game_coin_id \
 WHERE o.end_user_id = $1";

fn map_package(row: sqlx::postgres::PgRow) -> PackageView {
    PackageView {
        id: row.get("id"),
        code: row.get("code"),
        name: row.get("name"),
        game_coin_id: row.get("game_coin_id"),
        credit_game_coin: row.get("credit_game_coin"),
        coin_amount: row.get("coin_amount"),
        fiat_price: row.get("fiat_price"),
        seller_type: row.get("seller_type"),
        status: row.get("status"),
    }
}

fn map_corp(row: sqlx::postgres::PgRow) -> CorpProductView {
    CorpProductView {
        id: row.get("id"),
        corporate_user_id: row.get("corporate_user_id"),
        game_id: row.get("game_id"),
        code: row.get("code"),
        name: row.get("name"),
        product_type: row.get("product_type"),
        credit_game_coin_id: row.get("credit_game_coin_id"),
        credit_game_coin: row.get("credit_game_coin"),
        credit_amount: row.get("credit_amount"),
        item_code: row.get("item_code"),
        fiat_price: row.get("fiat_price"),
        seller_type: row.get("seller_type"),
        status: row.get("status"),
    }
}

fn map_order(row: sqlx::postgres::PgRow) -> ShopOrderView {
    map_order_full(row)
}

fn map_order_full(row: sqlx::postgres::PgRow) -> ShopOrderView {
    ShopOrderView {
        id: row.get("id"),
        end_user_id: row.get("end_user_id"),
        game_account_id: row.get("game_account_id"),
        seller_type: row.get("seller_type"),
        package_id: row.get("package_id"),
        corp_product_id: row.get("corp_product_id"),
        package_code: row.get("package_code"),
        product_code: row.get("product_code"),
        credit_game_coin: row.get("credit_game_coin"),
        credit_amount: row.get("credit_amount"),
        item_code: row.get("item_code"),
        fiat_currency: row.get("fiat_currency"),
        fiat_price: row.get("fiat_price"),
        partner_order_no: row.get("partner_order_no"),
        checkout_url: None,
        status: row.get("status"),
        expires_at: row.get("expires_at"),
        paid_at: row.get("paid_at"),
        created_at: row.get("created_at"),
        game_coin_id: row.try_get("game_coin_id").ok().flatten(),
        corporate_user_id: row.try_get("corporate_user_id").ok().flatten(),
    }
}
