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
        let row = sqlx::query(crate::query::wallet::GET_TXN)
            .bind(txn_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(row.map(map_txn_row))
    }

    pub async fn find_by_partner_ref(
        &self,
        request_id: &str,
        partner_txn_id: &str,
    ) -> Result<Option<MoneyTxnView>, String> {
        let Some(pool) = &self.pool else {
            return Ok(None);
        };
        let row = sqlx::query(crate::query::wallet::FIND_BY_PARTNER_REF)
            .bind(request_id.trim())
            .bind(partner_txn_id.trim())
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(row.map(map_txn_row))
    }

    /// Record partner_txn_id on the pending txn for webhook settle lookup.
    pub async fn attach_partner_ref(
        &self,
        txn_id: i64,
        request_id: &str,
        partner_txn_id: &str,
    ) -> Result<(), String> {
        let pool = self.pool.as_ref().ok_or("POSTGRES_URL required")?;
        let now = now_ms();
        let payload = serde_json::json!({
            "request_id": request_id,
            "partner_txn_id": partner_txn_id,
        });
        sqlx::query(crate::query::wallet::INSERT_STATUS_LOG_PARTNER_REF)
            .bind(txn_id)
            .bind(format!("partner accepted ({partner_txn_id})"))
            .bind(payload.to_string())
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn mark_terminal(
        &self,
        txn_id: i64,
        status: &str,
        note: &str,
        payload: serde_json::Value,
    ) -> Result<(), String> {
        let event_type = match status {
            "failed" => "failed",
            "cancelled" => "cancelled",
            "rejected" => "rejected",
            other => return Err(format!("unsupported terminal status: {other}")),
        };
        let pool = self.pool.as_ref().ok_or("POSTGRES_URL required")?;
        let now = now_ms();
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
        let updated = sqlx::query(crate::query::wallet::MARK_TERMINAL)
            .bind(txn_id)
            .bind(status)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();
        if updated == 0 {
            tx.rollback().await.ok();
            return Ok(());
        }
        sqlx::query(crate::query::wallet::INSERT_STATUS_LOG_TERMINAL)
            .bind(txn_id)
            .bind(status)
            .bind(event_type)
            .bind(note)
            .bind(payload.to_string())
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Webhook settle: success → credit/debit; failed/cancelled → terminal status.
    /// Only existing `deposit` / `withdrawal` rows are handled (never creates a txn).
    pub async fn apply_settle(
        &self,
        request_id: &str,
        partner_txn_id: &str,
        transaction_type: &str,
        status: &str,
        amount: f64,
        game_coin: &str,
        event_id: &str,
    ) -> Result<MoneyTxnView, String> {
        if request_id.trim().is_empty() || partner_txn_id.trim().is_empty() {
            return Err("request_id and partner_txn_id required".into());
        }
        let txn = self
            .find_by_partner_ref(request_id, partner_txn_id)
            .await?
            .ok_or_else(|| "deposit/withdrawal txn not found".to_string())?;

        if txn.transaction_type != "deposit" && txn.transaction_type != "withdrawal" {
            return Err("deposit/withdrawal txn not found".into());
        }
        if !transaction_type.is_empty() && txn.transaction_type != transaction_type {
            return Err(format!(
                "transaction_type mismatch (have {}, got {transaction_type})",
                txn.transaction_type
            ));
        }
        if amount > 0.0 && (txn.amount - amount).abs() > 1e-6 {
            return Err(format!(
                "amount mismatch (have {}, got {amount})",
                txn.amount
            ));
        }
        if !game_coin.is_empty() {
            if let Some(code) = txn.game_coin.as_deref() {
                if !code.eq_ignore_ascii_case(game_coin) {
                    return Err(format!("game_coin mismatch (have {code}, got {game_coin})"));
                }
            }
        }

        let ga_row = txn
            .game_account_id
            .ok_or_else(|| "txn missing game_account_id".to_string())?;

        match status {
            "success" | "completed" | "paid" => {
                if txn.status == "completed" {
                    return Ok(txn);
                }
                if txn.status != "pending" && txn.status != "processing" {
                    return Err(format!(
                        "txn not open for settle (status={})",
                        txn.status
                    ));
                }
                self.complete_txn(
                    txn.id,
                    &txn.transaction_type,
                    ga_row,
                    txn.game_coin_id,
                    txn.amount,
                    partner_txn_id,
                    request_id,
                )
                .await?;
            }
            "failed" | "cancelled" | "rejected" => {
                let terminal = if status == "failed" {
                    "failed"
                } else if status == "rejected" {
                    "rejected"
                } else {
                    "cancelled"
                };
                if txn.status == terminal || txn.status == "completed" {
                    return self
                        .get_txn(txn.id)
                        .await?
                        .ok_or_else(|| "txn missing after settle".into());
                }
                self.mark_terminal(
                    txn.id,
                    terminal,
                    &format!("{terminal} via webhook ({event_id})"),
                    serde_json::json!({
                        "event_id": event_id,
                        "request_id": request_id,
                        "partner_txn_id": partner_txn_id,
                        "status": status,
                    }),
                )
                .await?;
            }
            other => return Err(format!("unsupported settle status: {other}")),
        }

        let updated = self
            .get_txn(txn.id)
            .await?
            .ok_or_else(|| "txn missing after settle".to_string())?;

        let notice_type = match updated.status.as_str() {
            "completed" => "completed",
            "failed" => "failed",
            "cancelled" => "cancelled",
            "rejected" => "rejected",
            _ => "status_changed",
        };
        let title = format!("{} {}", updated.transaction_type, updated.status);
        let body = format!(
            "{} txn {} is {}",
            updated.transaction_type, updated.id, updated.status
        );
        let payload = serde_json::json!({
            "event_id": event_id,
            "request_id": request_id,
            "partner_txn_id": partner_txn_id,
            "txn": updated,
        });
        if let Err(e) = self
            .emit_outbound_notice(
                updated.end_user_id,
                updated.id,
                notice_type,
                &title,
                &body,
                &payload,
            )
            .await
        {
            tracing::warn!(txn_id = updated.id, error = %e, "money settle notice insert failed");
        }
        Ok(updated)
    }

    async fn emit_outbound_notice(
        &self,
        end_user_id: i64,
        source_id: i64,
        event_type: &str,
        title: &str,
        body: &str,
        payload: &serde_json::Value,
    ) -> Result<(), String> {
        let pool = self.pool.as_ref().ok_or("POSTGRES_URL required")?;
        let now = now_ms();
        sqlx::query(crate::query::wallet::INSERT_OUTBOUND_NOTICE)
            .bind(end_user_id)
            .bind(source_id)
            .bind(event_type)
            .bind(title)
            .bind(body)
            .bind(payload.to_string())
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn map_txn_row(r: sqlx::postgres::PgRow) -> MoneyTxnView {
    MoneyTxnView {
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
    }
}
