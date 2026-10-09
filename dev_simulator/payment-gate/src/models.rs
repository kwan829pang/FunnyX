use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Shop fiat payment create request (doc/e-shop.md §7.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShopPaymentRequest {
    pub request_id: String,
    pub shop_order_id: i64,
    #[serde(default = "default_seller_type")]
    pub seller_type: String,
    pub partner_id: String,
    pub user_id: String,
    #[serde(default)]
    pub game_account_id: Option<String>,
    #[serde(default)]
    pub package_code: Option<String>,
    #[serde(default)]
    pub product_code: Option<String>,
    #[serde(default)]
    pub credit_game_coin: Option<String>,
    #[serde(default)]
    pub credit_amount: Option<f64>,
    pub fiat_currency: String,
    pub fiat_price: f64,
    pub callback_url: String,
    #[serde(default)]
    pub return_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
}

fn default_seller_type() -> String {
    "platform".into()
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateShopPaymentResponse {
    pub partner_order_no: String,
    pub checkout_url: String,
    /// Always `pending` on submit; final status applied after `sim_delay_ms`.
    pub status: String,
    /// Target status from `X-Sim-Status` (PENDING if header omitted).
    pub sim_status: String,
    /// Milliseconds until `sim_status` is applied (and webhook fired when applicable).
    pub sim_delay_ms: u64,
    pub expires_at: DateTime<Utc>,
    /// Always `test` so demo traffic is identifiable.
    pub source: String,
}

/// Deposit / withdrawal create request (doc/partner.md §3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDepositRequest {
    pub request_id: String,
    pub partner_id: String,
    pub user_id: String,
    pub game_id: String,
    pub game_account_id: String,
    #[serde(default = "default_txn_deposit")]
    pub transaction_type: String,
    pub amount: f64,
    pub game_coin: String,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default = "default_source_test")]
    pub source: String,
    #[serde(default)]
    pub callback_url: Option<String>,
    #[serde(default)]
    pub metadata: Option<Value>,
}

fn default_txn_deposit() -> String {
    "deposit".into()
}

fn default_source_test() -> String {
    "test".into()
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateDepositResponse {
    pub partner_order_no: String,
    /// Always `pending` on submit; final status applied after `sim_delay_ms`.
    pub status: String,
    pub sim_status: String,
    pub sim_delay_ms: u64,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletePaymentRequest {
    /// `paid` | `failed` | `cancelled` (shop) or `success` | `failed` | `rejected` | `cancelled` (deposit).
    /// Ignored when `X-Sim-Status` is set.
    #[serde(default)]
    pub status: Option<String>,
    /// When true (default), POST the callback_url immediately.
    #[serde(default = "default_true")]
    pub fire_callback: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
pub struct ShopCallbackPayload {
    pub event_id: String,
    pub partner_order_no: String,
    pub shop_order_id: i64,
    pub seller_type: String,
    pub status: String,
    pub fiat_currency: String,
    pub fiat_paid: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_game_coin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_amount: Option<f64>,
    pub paid_at: DateTime<Utc>,
    pub signature: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DepositCallbackPayload {
    pub event_id: String,
    pub partner_order_no: String,
    pub request_id: String,
    pub partner_id: String,
    pub user_id: String,
    pub game_id: String,
    pub game_account_id: String,
    pub transaction_type: String,
    pub amount: f64,
    pub game_coin: String,
    pub status: String,
    pub paid_at: DateTime<Utc>,
    pub signature: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentView {
    pub kind: String,
    pub partner_order_no: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sim_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sim_delay_ms: Option<u64>,
    pub source: String,
    pub callback_url: Option<String>,
    pub callback_attempts: u32,
    pub last_callback_status: Option<u16>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub detail: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    pub error: String,
}
