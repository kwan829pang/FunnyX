//! Session wallet / deposit / withdraw / transactions.

use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::{err_status, map_sts_err, AppState};
use crate::game_accounts::require_end_user;
use crate::models::ErrorBody;
use crate::partner::MoneyRequest;
use crate::wallet_store::MoneyTxnView;

#[derive(Debug, Deserialize)]
pub struct MoneyBody {
    /// Platform `fx_game.game_accounts.id` (row id from GET /v1/client/game-accounts).
    pub game_account_id: i64,
    /// Game coin code (e.g. `PLT`, `GCA`).
    pub game_coin: String,
    pub amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct TxnQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Serialize)]
pub struct MoneySubmitResponse {
    pub txn: MoneyTxnView,
    pub partner_txn_id: Option<String>,
    pub partner_status: Option<String>,
    pub sim_delay_ms: Option<u64>,
}

pub async fn get_wallet(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    let rows = state
        .wallets
        .list_wallets(end_user_id)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({ "wallets": rows })))
}

pub async fn list_transactions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<TxnQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    let rows = state
        .wallets
        .list_transactions(end_user_id, q.limit)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({ "transactions": rows })))
}

pub async fn deposit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MoneyBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    submit_money(&state, &headers, &body, "deposit").await
}

pub async fn withdraw(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MoneyBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    submit_money(&state, &headers, &body, "withdrawal").await
}

async fn submit_money(
    state: &AppState,
    headers: &HeaderMap,
    body: &MoneyBody,
    transaction_type: &str,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    if !state.wallets.has_postgres() {
        return Err(err_status(
            StatusCode::SERVICE_UNAVAILABLE,
            "POSTGRES_URL required for deposit/withdraw",
        ));
    }
    let end_user_id = require_end_user(state, headers).await?;
    if body.amount <= 0.0 || !body.amount.is_finite() {
        return Err(err_status(StatusCode::BAD_REQUEST, "amount must be > 0"));
    }
    let coin = body.game_coin.trim();
    if coin.is_empty() {
        return Err(err_status(StatusCode::BAD_REQUEST, "game_coin required"));
    }

    let account = state
        .wallets
        .get_owned_account(end_user_id, body.game_account_id)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| {
            err_status(
                StatusCode::NOT_FOUND,
                "game_account_id not found for this user",
            )
        })?;

    let (game_coin_id, game_coin_code) = state
        .wallets
        .resolve_coin_id(coin)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| err_status(StatusCode::BAD_REQUEST, format!("unknown game_coin: {coin}")))?;

    if transaction_type == "withdrawal" {
        let avail = state
            .wallets
            .available_balance(account.row_id, game_coin_id)
            .await
            .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        if avail < body.amount {
            return Err(err_status(
                StatusCode::BAD_REQUEST,
                format!("insufficient balance (available={avail})"),
            ));
        }
    }

    let request_id = format!("req_{}", Uuid::new_v4().simple());
    let txn_id = state
        .wallets
        .insert_pending_txn(
            end_user_id,
            account.row_id,
            transaction_type,
            body.amount,
            game_coin_id,
            &request_id,
        )
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let partner_id = if account.partner_id.is_empty() {
        state.config.default_partner_id.clone()
    } else {
        account.partner_id.clone()
    };
    let partner_game_id = if account.partner_game_id.is_empty() {
        account.game_code.clone()
    } else {
        account.partner_game_id.clone()
    };

    let money_req = MoneyRequest {
        request_id: request_id.clone(),
        partner_id,
        user_id: end_user_id.to_string(),
        game_id: partner_game_id,
        game_account_id: account.game_account_id.clone(),
        transaction_type: transaction_type.into(),
        amount: body.amount,
        game_coin: game_coin_code.clone(),
        channel: Some("partner_token".into()),
        source: "test".into(),
    };

    let partner_resp = match if transaction_type == "deposit" {
        state.partner.deposit(&money_req).await
    } else {
        state.partner.withdraw(&money_req).await
    } {
        Ok(r) => r,
        Err((status, body_err)) => {
            let _ = state
                .wallets
                .mark_failed(
                    txn_id,
                    &body_err.error,
                    serde_json::json!({ "request_id": request_id, "error": body_err.error }),
                )
                .await;
            return Err(map_sts_err((status, body_err)));
        }
    };

    // Schedule completion when partner simulates async settle (company-a).
    let delay_ms = partner_resp.sim_delay_ms.max(100);
    let wallets = state.wallets.clone();
    let txn_type = transaction_type.to_string();
    let ga_row = account.row_id;
    let amount = body.amount;
    let partner_txn_id = partner_resp.partner_txn_id.clone();
    let req_id = request_id.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        if let Err(e) = wallets
            .complete_txn(
                txn_id,
                &txn_type,
                ga_row,
                game_coin_id,
                amount,
                &partner_txn_id,
                &req_id,
            )
            .await
        {
            tracing::warn!(txn_id, error = %e, "money txn complete failed");
        }
    });

    let txn = state
        .wallets
        .get_txn(txn_id)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| err_status(StatusCode::INTERNAL_SERVER_ERROR, "txn missing after insert"))?;

    Ok((
        StatusCode::CREATED,
        Json(MoneySubmitResponse {
            txn,
            partner_txn_id: Some(partner_resp.partner_txn_id),
            partner_status: Some(partner_resp.status),
            sim_delay_ms: Some(partner_resp.sim_delay_ms),
        }),
    ))
}
