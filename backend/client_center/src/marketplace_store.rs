//! C7 Marketplace deals: eligibility, place (1 PLT fee), take/settle, cancel, notices.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::config::Config;
use crate::db::now_ms;
use crate::partner::{PartnerClient, TransferRequest};
use crate::query::marketplace as q;

pub const LISTING_FEE_PLT: f64 = q::LISTING_FEE_PLT;

#[derive(Debug, Clone, Serialize)]
pub struct Eligibility {
    pub can_place: bool,
    pub verified_platform_token: bool,
    pub playing_listed_game: bool,
    pub transfer_api_open: bool,
    pub listing_fee_plt: f64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceDeal {
    pub id: i64,
    pub poster_end_user_id: i64,
    pub poster_game_account_id: i64,
    pub game_id: i64,
    pub game_code: String,
    pub corporate_user_id: i64,
    pub partner_endpoint_id: i64,
    pub offer_asset_type: String,
    pub offer_game_coin_id: Option<i64>,
    pub offer_game_coin: String,
    pub offer_item_ref_id: Option<String>,
    pub offer_amount: f64,
    pub want_asset_type: String,
    pub want_game_coin_id: i64,
    pub want_game_coin: String,
    pub want_amount: f64,
    pub status: String,
    pub matched_request_id: Option<i64>,
    pub cancelled_at: i64,
    pub completed_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub listing_fee_plt: f64,
}

#[derive(Debug, Deserialize)]
pub struct PlaceDealRequest {
    pub poster_game_account_id: i64,
    pub offer_asset_type: String,
    #[serde(default)]
    pub offer_game_coin_id: Option<i64>,
    #[serde(default)]
    pub offer_item_ref_id: Option<String>,
    pub offer_amount: f64,
    pub want_asset_type: String,
    pub want_game_coin_id: i64,
    pub want_amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct TakeDealRequest {
    pub requester_game_account_id: i64,
}

#[derive(Clone)]
pub struct MarketplaceStore {
    pool: Option<PgPool>,
    http: Client,
    message_center_url: String,
}

impl MarketplaceStore {
    pub fn new(pool: Option<PgPool>, config: &Config, http: Client) -> Self {
        Self {
            pool,
            http,
            message_center_url: config.message_center_url.clone(),
        }
    }

    fn pool(&self) -> Result<&PgPool, String> {
        self.pool
            .as_ref()
            .ok_or_else(|| "POSTGRES_URL required for marketplace".into())
    }

    pub async fn eligibility(&self, end_user_id: i64) -> Result<Eligibility, String> {
        let pool = self.pool()?;
        let verified: bool = sqlx::query_scalar(q::HAS_PLT_PURCHASE)
            .bind(end_user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
        let playing: bool = sqlx::query_scalar(q::HAS_ACTIVE_GAME_BIND)
            .bind(end_user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
        let transfer: bool = sqlx::query_scalar(q::HAS_TRANSFER_FOR_ANY_GAME)
            .bind(end_user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
        let mut reasons = Vec::new();
        if !verified {
            reasons.push("buy_platform_token".into());
        }
        if !playing {
            reasons.push("bind_listed_game".into());
        }
        if !transfer {
            reasons.push("partner_transfer_required".into());
        }
        Ok(Eligibility {
            can_place: verified && playing && transfer,
            verified_platform_token: verified,
            playing_listed_game: playing,
            transfer_api_open: transfer,
            listing_fee_plt: LISTING_FEE_PLT,
            reasons,
        })
    }

    pub async fn list_open(&self, limit: i64) -> Result<Vec<MarketplaceDeal>, String> {
        let pool = self.pool()?;
        let rows = sqlx::query(q::LIST_OPEN)
            .bind(limit.clamp(1, 200))
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(deal_from_row).collect())
    }

    pub async fn get(&self, id: i64) -> Result<Option<MarketplaceDeal>, String> {
        let pool = self.pool()?;
        let row = sqlx::query(q::GET_BY_ID)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(row.map(deal_from_row))
    }

    pub async fn place(
        &self,
        end_user_id: i64,
        req: PlaceDealRequest,
        partner: &PartnerClient,
    ) -> Result<MarketplaceDeal, String> {
        let pool = self.pool()?;
        let elig = self.eligibility(end_user_id).await?;
        if !elig.can_place {
            return Err(format!(
                "not eligible to place deal: {}",
                elig.reasons.join(", ")
            ));
        }

        let offer_type = req.offer_asset_type.trim();
        if !matches!(offer_type, "game_coin" | "company_coin" | "game_item") {
            return Err("offer_asset_type must be game_coin, company_coin, or game_item".into());
        }
        let want_type = req.want_asset_type.trim();
        if !matches!(want_type, "platform_token" | "company_coin") {
            return Err("want_asset_type must be platform_token or company_coin".into());
        }
        if req.offer_amount <= 0.0 || req.want_amount <= 0.0 {
            return Err("offer_amount and want_amount must be > 0".into());
        }

        let ga = sqlx::query(q::OWNED_GAME_ACCOUNT)
            .bind(req.poster_game_account_id)
            .bind(end_user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "poster_game_account_id not found or not owned".to_string())?;
        let game_id: i64 = ga.get("game_id");
        let corporate_user_id: i64 = ga.get("corporate_user_id");
        let partner_code: String = ga.get("partner_code");
        let partner_game_id: String = ga.get("partner_game_id");
        let poster_partner_ga: String = ga.get("game_account_id");

        let ep = sqlx::query(q::TRANSFER_ENDPOINT)
            .bind(corporate_user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "partner Transfer API not open for this game".to_string())?;
        let partner_endpoint_id: i64 = ep.get("id");

        let (offer_coin_id, offer_item_ref) = match offer_type {
            "game_item" => {
                let item = req
                    .offer_item_ref_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| "offer_item_ref_id required for game_item".to_string())?
                    .to_string();
                let item_ep: Option<String> = sqlx::query_scalar(q::ITEM_LIST_ENDPOINT)
                    .bind(corporate_user_id)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                let items = partner
                    .list_items(
                        item_ep.as_deref(),
                        &poster_partner_ga,
                        Some(partner_game_id.as_str()).filter(|s| !s.is_empty()),
                    )
                    .await
                    .map_err(|(_, e)| e.error)?;
                if !items.iter().any(|i| i.item_ref_id == item) {
                    return Err("offer_item_ref_id not returned by partner Item List".into());
                }
                let _ = partner_code;
                (None, Some(item))
            }
            _ => {
                let coin_id = req
                    .offer_game_coin_id
                    .ok_or_else(|| "offer_game_coin_id required".to_string())?;
                let coin = sqlx::query(q::COIN_BY_ID)
                    .bind(coin_id)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "offer_game_coin_id not found".to_string())?;
                let code: String = coin.get("code");
                if offer_type == "company_coin" {
                    let ok: bool = sqlx::query_scalar(q::COMPANY_COIN_FOR_CORP)
                        .bind(corporate_user_id)
                        .bind(coin_id)
                        .fetch_one(pool)
                        .await
                        .map_err(|e| e.to_string())?;
                    if !ok {
                        return Err("offer company_coin must belong to parent company".into());
                    }
                }
                let _ = code;
                (Some(coin_id), None)
            }
        };

        let want_coin = sqlx::query(q::COIN_BY_ID)
            .bind(req.want_game_coin_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "want_game_coin_id not found".to_string())?;
        let want_is_plt: bool = want_coin.get("is_platform_token");
        if want_type == "platform_token" && !want_is_plt {
            return Err("want_game_coin_id must be Platform Token".into());
        }
        if want_type == "company_coin" {
            if want_is_plt {
                return Err("company_coin want cannot be Platform Token".into());
            }
            let ok: bool = sqlx::query_scalar(q::COMPANY_COIN_FOR_CORP)
                .bind(corporate_user_id)
                .bind(req.want_game_coin_id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
            if !ok {
                return Err("want company_coin must belong to parent company".into());
            }
        }

        let plt_id: i64 = sqlx::query_scalar(q::PLT_COIN_ID)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Platform Token coin not configured".to_string())?;

        let now = now_ms();
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        let fee_rows = sqlx::query(q::DEBIT_WALLET)
            .bind(req.poster_game_account_id)
            .bind(plt_id)
            .bind(LISTING_FEE_PLT)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();
        if fee_rows == 0 {
            return Err(format!(
                "insufficient PLT for listing fee ({LISTING_FEE_PLT})"
            ));
        }

        let deal_id: i64 = sqlx::query_scalar(q::INSERT_DEAL)
            .bind(end_user_id)
            .bind(req.poster_game_account_id)
            .bind(game_id)
            .bind(corporate_user_id)
            .bind(partner_endpoint_id)
            .bind(offer_type)
            .bind(offer_coin_id)
            .bind(offer_item_ref.as_deref())
            .bind(req.offer_amount)
            .bind(want_type)
            .bind(req.want_game_coin_id)
            .bind(req.want_amount)
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;

        let deal = self
            .get(deal_id)
            .await?
            .ok_or_else(|| "deal missing after insert".to_string())?;
        self.enqueue_notice(
            end_user_id,
            "marketplace_deal",
            deal_id,
            "created",
            "Marketplace deal posted",
            format!("Deal #{deal_id} is open on the Marketplace."),
            json!({
                "corporate_user_id": corporate_user_id,
                "deal_id": deal_id,
                "game_id": game_id,
                "status": "open",
                "listing_fee_plt": LISTING_FEE_PLT,
            }),
        )
        .await;
        Ok(deal)
    }

    pub async fn take(
        &self,
        end_user_id: i64,
        deal_id: i64,
        req: TakeDealRequest,
        partner: &PartnerClient,
        default_partner_id: &str,
    ) -> Result<MarketplaceDeal, String> {
        let pool = self.pool()?;
        let now = now_ms();

        let taker_ga = sqlx::query(q::OWNED_GAME_ACCOUNT)
            .bind(req.requester_game_account_id)
            .bind(end_user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "requester_game_account_id not found or not owned".to_string())?;
        let taker_game_id: i64 = taker_ga.get("game_id");
        let taker_partner_ga: String = taker_ga.get("game_account_id");
        let partner_code: String = taker_ga.get("partner_code");

        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
        let locked = sqlx::query(q::LOCK_OPEN)
            .bind(deal_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "deal not found".to_string())?;
        let status: String = locked.get("status");
        if status != "open" {
            return Err(format!("deal is not open (status={status})"));
        }
        let poster_id: i64 = locked.get("poster_end_user_id");
        if poster_id == end_user_id {
            return Err("cannot take your own deal".into());
        }
        let deal_game_id: i64 = locked.get("game_id");
        if deal_game_id != taker_game_id {
            return Err("requester game account must match deal game".into());
        }
        let poster_ga_id: i64 = locked.get("poster_game_account_id");
        let corporate_user_id: i64 = locked.get("corporate_user_id");
        let partner_endpoint_id: i64 = locked.get("partner_endpoint_id");
        let offer_asset_type: String = locked.get("offer_asset_type");
        let offer_game_coin_id: Option<i64> = locked.get("offer_game_coin_id");
        let offer_item_ref_id: Option<String> = locked.get("offer_item_ref_id");
        let offer_amount: f64 = locked.get("offer_amount");
        let want_game_coin_id: i64 = locked.get("want_game_coin_id");
        let want_amount: f64 = locked.get("want_amount");

        let request_id: i64 = sqlx::query_scalar(q::INSERT_REQUEST)
            .bind(deal_id)
            .bind(end_user_id)
            .bind(req.requester_game_account_id)
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                if e.to_string().contains("unique") {
                    "already requested this deal".into()
                } else {
                    e.to_string()
                }
            })?;

        let updated = sqlx::query(q::SET_SETTLING)
            .bind(deal_id)
            .bind(request_id)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();
        if updated == 0 {
            return Err("deal no longer open".into());
        }

        // Want payment: debit taker, credit poster (platform wallets on game accounts).
        let debit = sqlx::query(q::DEBIT_WALLET)
            .bind(req.requester_game_account_id)
            .bind(want_game_coin_id)
            .bind(want_amount)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();
        if debit == 0 {
            return Err("insufficient want-token balance to take deal".into());
        }
        sqlx::query(q::CREDIT_WALLET)
            .bind(poster_ga_id)
            .bind(want_game_coin_id)
            .bind(want_amount)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        let xfer_req_id = format!("xfer_mp_{deal_id}_{}", Uuid::new_v4().simple());
        let xfer_event_id: i64 = sqlx::query_scalar(q::INSERT_TRANSFER_EVENT)
            .bind(deal_id)
            .bind(request_id)
            .bind(partner_endpoint_id)
            .bind(&xfer_req_id)
            .bind(poster_ga_id)
            .bind(req.requester_game_account_id)
            .bind(&offer_asset_type)
            .bind(offer_game_coin_id)
            .bind(offer_item_ref_id.as_deref())
            .bind(offer_amount)
            .bind(json!({ "phase": "pending" }).to_string())
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;

        let transfer_url: Option<String> = sqlx::query_scalar(q::ENDPOINT_URL_BY_ID)
            .bind(partner_endpoint_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

        let poster_partner_ga: String = sqlx::query_scalar(q::GAME_ACCOUNT_PARTNER_ID)
            .bind(poster_ga_id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

        let offer_coin_code: Option<String> = if let Some(cid) = offer_game_coin_id {
            sqlx::query(q::COIN_BY_ID)
                .bind(cid)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
                .map(|r| r.get::<String, _>("code"))
        } else {
            None
        };

        let partner_id = if partner_code.is_empty() {
            default_partner_id.to_string()
        } else {
            partner_code
        };

        let xfer_req = TransferRequest {
            request_id: xfer_req_id.clone(),
            partner_id,
            from_game_account_id: poster_partner_ga,
            to_game_account_id: taker_partner_ga,
            asset_type: offer_asset_type.clone(),
            item_ref_id: offer_item_ref_id.clone(),
            game_coin: offer_coin_code,
            amount: offer_amount,
            deal_id: Some(deal_id.to_string()),
        };

        let transfer_result = partner
            .transfer(transfer_url.as_deref(), &xfer_req)
            .await;

        let transfer_ok = match &transfer_result {
            Ok(resp) => {
                let st = resp.status.to_lowercase();
                let sim = resp.sim_status.to_uppercase();
                !resp.transfer_id.is_empty()
                    && (st == "succeeded"
                        || st == "success"
                        || st == "pending"
                        || st == "created"
                        || st == "accepted"
                        || sim == "SUCCESS"
                        || sim == "OK")
            }
            Err(_) => false,
        };

        if transfer_ok {
            let resp = transfer_result.expect("checked Ok");
            let now2 = now_ms();
            let mut tx2 = pool.begin().await.map_err(|e| e.to_string())?;
            sqlx::query(q::SET_TRANSFER_STATUS)
                .bind(xfer_event_id)
                .bind("succeeded")
                .bind(json!({ "transfer": resp.transfer_id, "status": resp.status }).to_string())
                .bind(now2)
                .execute(&mut *tx2)
                .await
                .map_err(|e| e.to_string())?;
            sqlx::query(q::COMPLETE_REQUEST)
                .bind(request_id)
                .bind(now2)
                .execute(&mut *tx2)
                .await
                .map_err(|e| e.to_string())?;
            sqlx::query(q::SET_COMPLETED)
                .bind(deal_id)
                .bind(now2)
                .execute(&mut *tx2)
                .await
                .map_err(|e| e.to_string())?;
            tx2.commit().await.map_err(|e| e.to_string())?;

            let payload = json!({
                "corporate_user_id": corporate_user_id,
                "deal_id": deal_id,
                "game_id": deal_game_id,
                "status": "completed",
                "request_id": request_id,
            });
            self.enqueue_notice(
                poster_id,
                "marketplace_deal",
                deal_id,
                "completed",
                "Marketplace deal completed",
                format!("Deal #{deal_id} was taken and settled."),
                payload.clone(),
            )
            .await;
            self.enqueue_notice(
                end_user_id,
                "marketplace_deal",
                deal_id,
                "completed",
                "Marketplace deal completed",
                format!("You took deal #{deal_id}; assets transferred."),
                payload,
            )
            .await;
        } else {
            let err_msg = match transfer_result {
                Ok(r) => format!("partner transfer status={}", r.status),
                Err((_, e)) => e.error,
            };
            let now2 = now_ms();
            let mut tx2 = pool.begin().await.map_err(|e| e.to_string())?;
            let _ = sqlx::query(q::DEBIT_WALLET)
                .bind(poster_ga_id)
                .bind(want_game_coin_id)
                .bind(want_amount)
                .bind(now2)
                .execute(&mut *tx2)
                .await;
            let _ = sqlx::query(q::CREDIT_WALLET)
                .bind(req.requester_game_account_id)
                .bind(want_game_coin_id)
                .bind(want_amount)
                .bind(now2)
                .execute(&mut *tx2)
                .await;
            sqlx::query(q::SET_TRANSFER_STATUS)
                .bind(xfer_event_id)
                .bind("failed")
                .bind(json!({ "error": err_msg }).to_string())
                .bind(now2)
                .execute(&mut *tx2)
                .await
                .map_err(|e| e.to_string())?;
            sqlx::query(q::FAIL_REQUEST)
                .bind(request_id)
                .bind(now2)
                .execute(&mut *tx2)
                .await
                .map_err(|e| e.to_string())?;
            sqlx::query(q::SET_FAILED)
                .bind(deal_id)
                .bind(now2)
                .execute(&mut *tx2)
                .await
                .map_err(|e| e.to_string())?;
            tx2.commit().await.map_err(|e| e.to_string())?;

            let payload = json!({
                "corporate_user_id": corporate_user_id,
                "deal_id": deal_id,
                "game_id": deal_game_id,
                "status": "failed",
                "error": err_msg,
            });
            self.enqueue_notice(
                poster_id,
                "marketplace_deal",
                deal_id,
                "failed",
                "Marketplace deal failed",
                format!("Deal #{deal_id} settle failed."),
                payload.clone(),
            )
            .await;
            self.enqueue_notice(
                end_user_id,
                "marketplace_deal",
                deal_id,
                "failed",
                "Marketplace deal failed",
                format!("Take of deal #{deal_id} failed: {err_msg}"),
                payload,
            )
            .await;
            return Err(format!("transfer failed: {err_msg}"));
        }

        self.get(deal_id)
            .await?
            .ok_or_else(|| "deal missing after take".to_string())
    }

    pub async fn cancel(&self, end_user_id: i64, deal_id: i64) -> Result<MarketplaceDeal, String> {
        let pool = self.pool()?;
        let before = self
            .get(deal_id)
            .await?
            .ok_or_else(|| "deal not found".to_string())?;
        if before.poster_end_user_id != end_user_id {
            return Err("only poster can cancel".into());
        }
        if before.status != "open" {
            return Err("only open deals can be cancelled".into());
        }
        let now = now_ms();
        let rows = sqlx::query(q::CANCEL_OPEN)
            .bind(deal_id)
            .bind(now)
            .bind(end_user_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();
        if rows == 0 {
            return Err("cancel failed (deal not open)".into());
        }
        let deal = self
            .get(deal_id)
            .await?
            .ok_or_else(|| "deal missing after cancel".to_string())?;
        self.enqueue_notice(
            end_user_id,
            "marketplace_deal",
            deal_id,
            "cancelled",
            "Marketplace deal cancelled",
            format!("Deal #{deal_id} was cancelled. Listing fee is not refunded."),
            json!({
                "corporate_user_id": deal.corporate_user_id,
                "deal_id": deal_id,
                "game_id": deal.game_id,
                "status": "cancelled",
            }),
        )
        .await;
        Ok(deal)
    }

    async fn enqueue_notice(
        &self,
        end_user_id: i64,
        source_type: &str,
        source_id: i64,
        event_type: &str,
        title: &str,
        body: String,
        payload: Value,
    ) {
        let url = format!("{}/v1/notices", self.message_center_url.trim_end_matches('/'));
        let req_body = json!({
            "end_user_id": end_user_id,
            "source_type": source_type,
            "source_id": source_id,
            "event_type": event_type,
            "title": title,
            "body": body,
            "payload": payload,
        });
        match self.http.post(&url).json(&req_body).send().await {
            Ok(resp) if resp.status().is_success() => return,
            Ok(resp) => {
                tracing::warn!(%url, status = %resp.status(), "message center enqueue failed; falling back to DB");
            }
            Err(e) => {
                tracing::warn!(%url, error = %e, "message center unreachable; falling back to DB");
            }
        }
        if let Some(pool) = &self.pool {
            let now = now_ms();
            if let Err(e) = sqlx::query(q::INSERT_OUTBOUND_NOTICE)
                .bind(end_user_id)
                .bind(source_type)
                .bind(source_id)
                .bind(event_type)
                .bind(title)
                .bind(&body)
                .bind(payload.to_string())
                .bind(now)
                .execute(pool)
                .await
            {
                tracing::error!(error = %e, "failed to insert outbound_notices fallback");
            }
        }
    }
}

fn deal_from_row(row: sqlx::postgres::PgRow) -> MarketplaceDeal {
    MarketplaceDeal {
        id: row.get("id"),
        poster_end_user_id: row.get("poster_end_user_id"),
        poster_game_account_id: row.get("poster_game_account_id"),
        game_id: row.get("game_id"),
        game_code: row.get("game_code"),
        corporate_user_id: row.get("corporate_user_id"),
        partner_endpoint_id: row.get("partner_endpoint_id"),
        offer_asset_type: row.get("offer_asset_type"),
        offer_game_coin_id: row.get("offer_game_coin_id"),
        offer_game_coin: row.get("offer_game_coin"),
        offer_item_ref_id: row.get("offer_item_ref_id"),
        offer_amount: row.get("offer_amount"),
        want_asset_type: row.get("want_asset_type"),
        want_game_coin_id: row.get("want_game_coin_id"),
        want_game_coin: row.get("want_game_coin"),
        want_amount: row.get("want_amount"),
        status: row.get("status"),
        matched_request_id: row.get("matched_request_id"),
        cancelled_at: row.get("cancelled_at"),
        completed_at: row.get("completed_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        listing_fee_plt: LISTING_FEE_PLT,
    }
}
