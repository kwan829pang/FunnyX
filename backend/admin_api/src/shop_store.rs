//! Corp e-shop products for Admin monitor / suspend.

use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize)]
pub struct AdminCorpProduct {
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
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone)]
pub struct CorpProductStore {
    pool: Option<PgPool>,
    memory: Option<Arc<RwLock<HashMap<i64, AdminCorpProduct>>>>,
}

impl CorpProductStore {
    pub fn new(pool: Option<PgPool>) -> Self {
        if pool.is_some() {
            Self {
                pool,
                memory: None,
            }
        } else {
            let mut m = HashMap::new();
            m.insert(1, demo_product());
            Self {
                pool: None,
                memory: Some(Arc::new(RwLock::new(m))),
            }
        }
    }

    pub async fn list(
        &self,
        status: Option<&str>,
        corporate_user_id: Option<i64>,
    ) -> Vec<AdminCorpProduct> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(crate::query::shop::LIST)
            .bind(status)
            .bind(corporate_user_id)
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
            .filter(|p| status.map(|s| p.status == s).unwrap_or(true))
            .filter(|p| corporate_user_id.map(|c| p.corporate_user_id == c).unwrap_or(true))
            .cloned()
            .collect();
        v.sort_by_key(|p| p.id);
        v
    }

    pub async fn get(&self, id: i64) -> Option<AdminCorpProduct> {
        if let Some(pool) = &self.pool {
            return sqlx::query(crate::query::shop::GET)
            .bind(id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .map(map_row);
        }
        let mem = self.memory.as_ref()?;
        mem.read().await.get(&id).cloned()
    }

    pub async fn set_status(
        &self,
        id: i64,
        status: &str,
        admin_id: i64,
    ) -> Result<AdminCorpProduct, String> {
        match status {
            "active" | "inactive" | "archived" => {}
            other => return Err(format!("invalid status: {other} (use active, inactive, archived)")),
        }
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let n = sqlx::query(crate::query::shop::UPDATE_STATUS)
            .bind(status)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();
            if n == 0 {
                return Err("product not found".into());
            }
            let _ = sqlx::query(crate::query::shop::INSERT_ACTION_LOG)
            .bind(admin_id)
            .bind(
                serde_json::json!({ "corp_product_id": id, "status": status }).to_string(),
            )
            .bind(now)
            .execute(pool)
            .await;
            return self
                .get(id)
                .await
                .ok_or_else(|| "product not found after update".into());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut g = mem.write().await;
        let rec = g.get_mut(&id).ok_or_else(|| "product not found".to_string())?;
        rec.status = status.into();
        rec.updated_at = now;
        Ok(rec.clone())
    }
}

fn demo_product() -> AdminCorpProduct {
    AdminCorpProduct {
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
        created_at: 1_704_153_600_000,
        updated_at: 0,
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn map_row(row: sqlx::postgres::PgRow) -> AdminCorpProduct {
    AdminCorpProduct {
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
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
