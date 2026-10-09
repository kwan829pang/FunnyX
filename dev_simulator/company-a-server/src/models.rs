use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub partner_id: String,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PartnerEndpoint {
    #[serde(rename = "type")]
    pub endpoint_type: String,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PartnerSetupResponse {
    pub partner_id: String,
    pub company_name: String,
    pub server_name: String,
    pub endpoints: Vec<PartnerEndpoint>,
    pub auth_type: String,
    pub api_key: String,
    pub status: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserInfoResponse {
    pub partner_user_id: String,
    pub game_id: String,
    pub game_account_id: String,
    pub username: String,
    pub status: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerLookupQuery {
    /// Partner-side game id (company-a demo: `game_001`).
    pub game_id: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub partner_user_id: Option<String>,
    /// Optional platform end_user_id (mapping key on Client Center; demo uses username match).
    #[serde(default)]
    pub platform_end_user_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerLookupResponse {
    pub partner_user_id: String,
    pub game_id: String,
    pub game_account_id: String,
    pub username: String,
    pub status: String,
    pub playing: bool,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TransferRequest {
    pub request_id: String,
    pub partner_id: String,
    pub from_game_account_id: String,
    pub to_game_account_id: String,
    pub asset_type: String,
    #[serde(default)]
    pub item_ref_id: Option<String>,
    #[serde(default)]
    pub game_coin: Option<String>,
    pub amount: f64,
    #[serde(default)]
    pub deal_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferResponse {
    pub request_id: String,
    pub transfer_id: String,
    pub status: String,
    pub sim_status: String,
    pub sim_delay_ms: u64,
    pub asset_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_ref_id: Option<String>,
    pub amount: f64,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ItemListQuery {
    pub game_account_id: String,
    #[serde(default)]
    pub game_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameItem {
    pub item_ref_id: String,
    pub name: String,
    pub qty: i64,
    pub game_account_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemListResponse {
    pub game_account_id: String,
    pub items: Vec<GameItem>,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BalanceQuery {
    pub game_account_id: String,
    #[serde(default)]
    pub game_coin: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BalanceResponse {
    pub game_account_id: String,
    pub game_coin: String,
    pub balance: f64,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MoneyRequest {
    pub request_id: String,
    pub partner_id: String,
    pub user_id: String,
    pub game_id: String,
    pub game_account_id: String,
    pub transaction_type: String,
    pub amount: f64,
    pub game_coin: String,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default = "default_source_test")]
    pub source: String,
    #[serde(default)]
    pub metadata: Option<Value>,
}

fn default_source_test() -> String {
    "test".into()
}

#[derive(Debug, Clone, Serialize)]
pub struct MoneyResponse {
    pub request_id: String,
    pub partner_txn_id: String,
    pub transaction_type: String,
    pub status: String,
    pub sim_status: String,
    pub sim_delay_ms: u64,
    pub amount: f64,
    pub game_coin: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferView {
    pub transfer_id: String,
    pub request_id: String,
    pub status: String,
    pub asset_type: String,
    pub from_game_account_id: String,
    pub to_game_account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_ref_id: Option<String>,
    pub amount: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: String,
}
