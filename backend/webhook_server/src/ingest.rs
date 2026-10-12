//! Partner payment callback bodies (shop + Company Basic Token).

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShopPaymentCallback {
    pub event_id: String,
    pub partner_order_no: String,
    pub shop_order_id: i64,
    #[serde(default)]
    pub seller_type: Option<String>,
    pub status: String,
    pub fiat_currency: String,
    #[serde(default)]
    pub fiat_paid: f64,
    #[serde(default)]
    pub credit_game_coin: Option<String>,
    #[serde(default)]
    pub credit_amount: Option<f64>,
    #[serde(default)]
    pub paid_at: Option<Value>,
    pub signature: String,
}

impl ShopPaymentCallback {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        serde_json::from_value(v.clone()).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorpTokenPaymentCallback {
    pub event_id: String,
    pub partner_order_no: String,
    pub corp_token_order_id: i64,
    pub status: String,
    #[serde(default)]
    pub coin_amount: Option<f64>,
    #[serde(default)]
    pub token_code: Option<String>,
    #[serde(default)]
    pub paid_at: Option<Value>,
    pub signature: String,
}

/// Partner deposit / withdrawal finalize callback.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MoneyCallback {
    pub event_id: String,
    pub request_id: String,
    pub partner_txn_id: String,
    pub transaction_type: String,
    /// Partner final status: `success` | `failed` | `cancelled`.
    pub status: String,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub game_coin: String,
    pub signature: String,
}

impl MoneyCallback {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        serde_json::from_value(v.clone()).map_err(|e| e.to_string())
    }
}
