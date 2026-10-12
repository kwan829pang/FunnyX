//! Partner HTTP helpers (company-a: lookup, deposit, withdrawal, transfer, item list).

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::models::ErrorBody;

#[derive(Clone)]
pub struct PartnerClient {
    http: Client,
    base: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct PartnerPlayer {
    pub partner_user_id: String,
    #[allow(dead_code)]
    pub game_id: String,
    pub game_account_id: String,
    #[allow(dead_code)]
    pub username: String,
    pub playing: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoneyRequest {
    pub request_id: String,
    pub partner_id: String,
    pub user_id: String,
    pub game_id: String,
    pub game_account_id: String,
    pub transaction_type: String,
    pub amount: f64,
    pub game_coin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MoneyResponse {
    #[allow(dead_code)]
    pub request_id: String,
    pub partner_txn_id: String,
    #[allow(dead_code)]
    pub transaction_type: String,
    pub status: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub sim_status: String,
    #[serde(default)]
    pub sim_delay_ms: u64,
    #[allow(dead_code)]
    pub amount: f64,
    #[allow(dead_code)]
    pub game_coin: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub source: String,
}

impl PartnerClient {
    pub fn new(config: &Config, http: Client) -> Self {
        Self {
            http,
            base: config.partner_a_base_url.clone(),
            api_key: config.partner_a_api_key.clone(),
        }
    }

    pub async fn lookup_player(
        &self,
        partner_game_id: &str,
        platform_end_user_id: i64,
        partner_username: Option<&str>,
        partner_user_id: Option<&str>,
    ) -> Result<PartnerPlayer, (reqwest::StatusCode, ErrorBody)> {
        let mut url = reqwest::Url::parse(&format!("{}/api/players/lookup", self.base))
            .map_err(|e| gateway(format!("partner url: {e}")))?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("game_id", partner_game_id);
            q.append_pair("platform_end_user_id", &platform_end_user_id.to_string());
            if let Some(u) = partner_username.filter(|s| !s.is_empty()) {
                q.append_pair("username", u);
            }
            if let Some(p) = partner_user_id.filter(|s| !s.is_empty()) {
                q.append_pair("partner_user_id", p);
            }
        }
        let resp = self
            .http
            .get(url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| gateway(format!("partner lookup: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let err = resp.json::<ErrorBody>().await.unwrap_or(ErrorBody {
                error: format!("partner lookup failed ({status})"),
            });
            return Err((status, err));
        }
        resp.json::<PartnerPlayer>()
            .await
            .map_err(|e| gateway(format!("partner lookup json: {e}")))
    }

    pub async fn deposit(
        &self,
        req: &MoneyRequest,
    ) -> Result<MoneyResponse, (reqwest::StatusCode, ErrorBody)> {
        self.money_call("/api/deposit", req).await
    }

    pub async fn withdraw(
        &self,
        req: &MoneyRequest,
    ) -> Result<MoneyResponse, (reqwest::StatusCode, ErrorBody)> {
        self.money_call("/api/withdrawal", req).await
    }

    /// Partner Item List API — opaque `item_ref_id` for Marketplace place-deal.
    pub async fn list_items(
        &self,
        endpoint_url: Option<&str>,
        game_account_id: &str,
        game_id: Option<&str>,
    ) -> Result<Vec<PartnerGameItem>, (reqwest::StatusCode, ErrorBody)> {
        let base = endpoint_url
            .map(|u| u.trim_end_matches('/').to_string())
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| format!("{}/api/items", self.base));
        let mut url = reqwest::Url::parse(&base).map_err(|e| gateway(format!("items url: {e}")))?;
        // If endpoint is a full path ending in /api/items keep it; else treat as base.
        if !url.path().contains("/api/items") && endpoint_url.is_none() {
            // already /api/items
        } else if endpoint_url.is_some() && !url.path().ends_with("/api/items") && !url.path().contains("items") {
            // partner_endpoints may store full transfer/item URL already
        }
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("game_account_id", game_account_id);
            if let Some(g) = game_id.filter(|s| !s.is_empty()) {
                q.append_pair("game_id", g);
            }
        }
        let resp = self
            .http
            .get(url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| gateway(format!("partner items: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let err = resp.json::<ErrorBody>().await.unwrap_or(ErrorBody {
                error: format!("partner items failed ({status})"),
            });
            return Err((status, err));
        }
        let body: PartnerItemListResponse = resp
            .json()
            .await
            .map_err(|e| gateway(format!("partner items json: {e}")))?;
        Ok(body.items)
    }

    /// Partner Transfer API — move offer assets on Marketplace settle.
    pub async fn transfer(
        &self,
        endpoint_url: Option<&str>,
        req: &TransferRequest,
    ) -> Result<TransferResponse, (reqwest::StatusCode, ErrorBody)> {
        let url = endpoint_url
            .map(|u| u.trim_end_matches('/').to_string())
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| format!("{}/api/transfer", self.base));
        let resp = self
            .http
            .post(&url)
            .header("x-api-key", &self.api_key)
            .json(req)
            .send()
            .await
            .map_err(|e| gateway(format!("partner transfer: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let err = resp.json::<ErrorBody>().await.unwrap_or(ErrorBody {
                error: format!("partner transfer failed ({status})"),
            });
            return Err((status, err));
        }
        resp.json::<TransferResponse>()
            .await
            .map_err(|e| gateway(format!("partner transfer json: {e}")))
    }

    async fn money_call(
        &self,
        path: &str,
        req: &MoneyRequest,
    ) -> Result<MoneyResponse, (reqwest::StatusCode, ErrorBody)> {
        let url = format!("{}{path}", self.base);
        let resp = self
            .http
            .post(&url)
            .header("x-api-key", &self.api_key)
            .json(req)
            .send()
            .await
            .map_err(|e| gateway(format!("partner money: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let err = resp.json::<ErrorBody>().await.unwrap_or(ErrorBody {
                error: format!("partner money failed ({status})"),
            });
            return Err((status, err));
        }
        resp.json::<MoneyResponse>()
            .await
            .map_err(|e| gateway(format!("partner money json: {e}")))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferRequest {
    pub request_id: String,
    pub partner_id: String,
    pub from_game_account_id: String,
    pub to_game_account_id: String,
    pub asset_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_ref_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_coin: Option<String>,
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deal_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransferResponse {
    #[allow(dead_code)]
    pub request_id: String,
    pub transfer_id: String,
    pub status: String,
    #[serde(default)]
    pub sim_status: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub sim_delay_ms: u64,
    #[serde(default)]
    #[allow(dead_code)]
    pub asset_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnerGameItem {
    pub item_ref_id: String,
    pub name: String,
    pub qty: i64,
    pub game_account_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PartnerItemListResponse {
    #[serde(default)]
    items: Vec<PartnerGameItem>,
}

fn gateway(msg: String) -> (reqwest::StatusCode, ErrorBody) {
    (
        reqwest::StatusCode::BAD_GATEWAY,
        ErrorBody { error: msg },
    )
}
