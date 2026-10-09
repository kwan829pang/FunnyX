//! Partner HTTP helpers (company-a: lookup, deposit, withdrawal).

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

fn gateway(msg: String) -> (reqwest::StatusCode, ErrorBody) {
    (
        reqwest::StatusCode::BAD_GATEWAY,
        ErrorBody { error: msg },
    )
}
