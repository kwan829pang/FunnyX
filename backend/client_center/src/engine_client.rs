//! Private HTTP client for Core Engine (never via Gateway).

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::Config;

#[derive(Clone)]
pub struct EngineClient {
    http: Client,
    base: String,
}

#[derive(Debug, Serialize)]
pub struct PlaceOrderReq {
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub price: u64,
    pub quantity: u64,
    pub account_id: String,
    pub end_user_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineOrderView {
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub price: u64,
    pub original_quantity: u64,
    pub remaining_quantity: u64,
    pub status: String,
    pub timestamp_ms: u64,
    pub account_id: String,
    #[serde(default)]
    pub end_user_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PlaceResult {
    pub order: EngineOrderView,
    #[serde(default)]
    pub trades: Vec<Value>,
}

impl EngineClient {
    pub fn new(config: &Config, http: Client) -> Self {
        Self {
            http,
            base: config.core_engine_url.clone(),
        }
    }

    pub fn configured(&self) -> bool {
        !self.base.is_empty()
    }

    pub async fn place(&self, req: &PlaceOrderReq) -> Result<PlaceResult, String> {
        let url = format!("{}/v1/orders", self.base);
        let resp = self
            .http
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| format!("engine place: {e}"))?;
        let status = resp.status();
        let body = resp
            .json::<Value>()
            .await
            .map_err(|e| format!("engine place json: {e}"))?;
        if !status.is_success() {
            let err = body
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("place failed");
            return Err(err.to_string());
        }
        serde_json::from_value(body).map_err(|e| format!("engine place parse: {e}"))
    }

    pub async fn cancel(&self, symbol: &str, order_id: &str) -> Result<EngineOrderView, String> {
        let enc = urlencoding_symbol(symbol);
        let url = format!("{}/v1/orders/{enc}/{order_id}", self.base);
        let resp = self
            .http
            .delete(&url)
            .send()
            .await
            .map_err(|e| format!("engine cancel: {e}"))?;
        let status = resp.status();
        let body = resp
            .json::<Value>()
            .await
            .map_err(|e| format!("engine cancel json: {e}"))?;
        if !status.is_success() {
            let err = body
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("cancel failed");
            return Err(err.to_string());
        }
        serde_json::from_value(body).map_err(|e| format!("engine cancel parse: {e}"))
    }

    pub async fn book(&self, symbol: &str, levels: u32) -> Result<Value, String> {
        let enc = urlencoding_symbol(symbol);
        let url = format!("{}/v1/books/{enc}?levels={levels}", self.base);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("engine book: {e}"))?;
        let status = resp.status();
        let body = resp
            .json::<Value>()
            .await
            .map_err(|e| format!("engine book json: {e}"))?;
        if !status.is_success() {
            let err = body
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("book failed");
            return Err(err.to_string());
        }
        Ok(body)
    }
}

fn urlencoding_symbol(symbol: &str) -> String {
    symbol.replace('/', "%2F")
}
