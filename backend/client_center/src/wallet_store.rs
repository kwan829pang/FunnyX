//! Postgres wallet reads + deposit/withdrawal txn lifecycle.

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::db::now_ms;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub game_account_row_id: i64,
    pub game_account_id: String,
    pub game_id: i64,
    pub game_code: String,
    pub game_coin_id: i64,
    pub game_coin: String,
    pub available: f64,
    pub locked: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneyTxnView {
    pub id: i64,
    pub end_user_id: i64,
    pub game_account_id: Option<i64>,
    pub transaction_type: String,
    pub amount: f64,
    pub game_coin_id: i64,
    pub game_coin: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub struct OwnedGameAccount {
    pub row_id: i64,
    #[allow(dead_code)]
    pub game_id: i64,
    pub game_code: String,
    pub partner_id: String,
    pub partner_game_id: String,
    pub game_account_id: String,
}

#[derive(Clone)]
pub struct WalletStore {
    pool: Option<PgPool>,
}

impl WalletStore {
    pub fn new(pool: Option<PgPool>) -> Self {
        Self { pool }
    }

    pub fn has_postgres(&self) -> bool {
        self.pool.is_some()
    }

    pub async fn list_wallets(&self, end_user_id: i64) -> Result<Vec<WalletBalance>, String> {
        let Some(pool) = &self.pool else {
            return Ok(Vec::new());
        };
        let rows = sqlx::query(
            crate::query::wallet::LIST_FOR_USER,
        )
        .bind(end_user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(rows
            .into_iter()
            .map(|r| WalletBalance {
                game_account_row_id: r.get("row_id"),
                game_account_id: r.get("ga_str"),
                game_id: r.get("game_id"),
                game_code: r.get("game_code"),
                game_coin_id: r.get("game_coin_id"),
                game_coin: r.get("game_coin"),
                available: r.get("available"),
                locked: r.get("locked"),
            })
            .collect())
    }

    pub async fn list_transactions(
        &self,
        end_user_id: i64,
        limit: i64,
    ) -> Result<Vec<MoneyTxnView>, String> {
        let Some(pool) = &self.pool else {
            return Ok(Vec::new());
        };
        let lim = limit.clamp(1, 200);
        let rows = sqlx::query(
            crate::query::wallet::LIST_TRANSACTIONS,
        )
        .bind(end_user_id)
        .bind(lim)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(rows
            .into_iter()
            .map(|r| MoneyTxnView {
                id: r.get("id"),
                end_user_id: r.get("end_user_id"),
                game_account_id: r.try_get("game_account_id").ok(),
                transaction_type: r.get("transaction_type"),
                amount: r.get("amount"),
                game_coin_id: r.get("game_coin_id"),
                game_coin: r.try_get("game_coin").ok(),
                status: r.get("status"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    pub async fn get_owned_account(
        &self,
        end_user_id: i64,
        game_account_row_id: i64,
    ) -> Result<Option<OwnedGameAccount>, String> {
        let Some(pool) = &self.pool else {
            return Ok(None);
        };
        let row = sqlx::query(
            crate::query::wallet::OWNED_ACCOUNT,
        )
        .bind(game_account_row_id)
        .bind(end_user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(row.map(|r| OwnedGameAccount {
            row_id: r.get("id"),
            game_id: r.get("game_id"),
            game_code: r.get("game_code"),
            partner_id: r.get("partner_id"),
            partner_game_id: r.get("partner_game_id"),
            game_account_id: r.get("game_account_id"),
        }))
    }

    pub async fn resolve_coin_id(&self, game_coin: &str) -> Result<Option<(i64, String)>, String> {
        let Some(pool) = &self.pool else {
            return Ok(None);
        };
        let row = sqlx::query(
            crate::query::wallet::RESOLVE_COIN,
        )
        .bind(game_coin.trim())
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(row.map(|r| (r.get("id"), r.get("code"))))
    }

    pub async fn available_balance(
        &self,
        game_account_row_id: i64,
        game_coin_id: i64,
    ) -> Result<f64, String> {
        let Some(pool) = &self.pool else {
            return Ok(0.0);
        };
        let bal: Option<f64> = sqlx::query_scalar(
            crate::query::wallet::AVAILABLE_BALANCE,
        )
        .bind(game_account_row_id)
        .bind(game_coin_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(bal.unwrap_or(0.0))
    }

    /// Insert pending deposit/withdrawal + created status log. Returns txn id.
    pub async fn insert_pending_txn(
        &self,
        end_user_id: i64,
        game_account_row_id: i64,
        transaction_type: &str,
        amount: f64,
        game_coin_id: i64,
        request_id: &str,
    ) -> Result<i64, String> {
        let pool = self.pool.as_ref().ok_or("POSTGRES_URL required")?;
        let now = now_ms();
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
        let txn_id: i64 = sqlx::query_scalar(
            crate::query::wallet::INSERT_PENDING_TXN,
        )
        .bind(end_user_id)
        .bind(game_account_row_id)
        .bind(transaction_type)
        .bind(amount)
        .bind(game_coin_id)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        let payload = serde_json::json!({
            "request_id": request_id,
            "transaction_type": transaction_type,
            "amount": amount,
            "game_coin_id": game_coin_id,
        });
        sqlx::query(
            crate::query::wallet::INSERT_STATUS_LOG_CREATED,
        )
        .bind(txn_id)
        .bind(format!("{transaction_type} created ({request_id})"))
        .bind(payload.to_string())
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(txn_id)
    }

    pub async fn mark_failed(
        &self,
        txn_id: i64,
        note: &str,
        payload: serde_json::Value,
    ) -> Result<(), String> {
        let pool = self.pool.as_ref().ok_or("POSTGRES_URL required")?;
        let now = now_ms();
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
        sqlx::query(
            crate::query::wallet::MARK_FAILED,
        )
        .bind(txn_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        sqlx::query(
            crate::query::wallet::INSERT_STATUS_LOG_FAILED,
        )
        .bind(txn_id)
        .bind(note)
        .bind(payload.to_string())
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Idempotent complete: status → completed, status log, wallet adjust.
    pub async fn complete_txn(
        &self,
        txn_id: i64,
        transaction_type: &str,
        game_account_row_id: i64,
        game_coin_id: i64,
        amount: f64,
        partner_txn_id: &str,
        request_id: &str,
    ) -> Result<(), String> {
        let pool = self.pool.as_ref().ok_or("POSTGRES_URL required")?;
        let now = now_ms();

        let exists: bool = sqlx::query_scalar(
            crate::query::wallet::COMPLETED_LOG_EXISTS,
        )
        .bind(txn_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        if exists {
            return Ok(());
        }

        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
        let updated = sqlx::query(
            crate::query::wallet::MARK_COMPLETED,
        )
        .bind(txn_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .rows_affected();
        if updated == 0 {
            // Already terminal or missing — still try idempotent wallet skip
            tx.rollback().await.ok();
            return Ok(());
        }

        if transaction_type == "deposit" {
            sqlx::query(
                crate::query::wallet::UPSERT_WALLET_CREDIT,
            )
            .bind(game_account_row_id)
            .bind(game_coin_id)
            .bind(amount)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        } else if transaction_type == "withdrawal" {
            let rows = sqlx::query(
                crate::query::wallet::DEBIT_WALLET,
            )
            .bind(game_account_row_id)
            .bind(game_coin_id)
            .bind(amount)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();
            if rows == 0 {
                tx.rollback().await.ok();
                return Err("insufficient balance at complete".into());
            }
        }

        let payload = serde_json::json!({
            "request_id": request_id,
            "partner_txn_id": partner_txn_id,
            "transaction_type": transaction_type,
            "amount": amount,
            "game_coin_id": game_coin_id,
        });
        sqlx::query(
            crate::query::wallet::INSERT_STATUS_LOG_COMPLETED,
        )
        .bind(txn_id)
        .bind(format!("{transaction_type} completed ({partner_txn_id})"))
        .bind(payload.to_string())
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn get_txn(&self, txn_id: i64) -> Result<Option<MoneyTxnView>, String> {
        let Some(pool) = &self.pool else {
            return Ok(None);
        };
        let row = sqlx::query(
            crate::query::wallet::GET_TXN,
        )
        .bind(txn_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(row.map(|r| MoneyTxnView {
            id: r.get("id"),
            end_user_id: r.get("end_user_id"),
            game_account_id: r.try_get("game_account_id").ok(),
            transaction_type: r.get("transaction_type"),
            amount: r.get("amount"),
            game_coin_id: r.get("game_coin_id"),
            game_coin: r.try_get("game_coin").ok(),
            status: r.get("status"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }
}
