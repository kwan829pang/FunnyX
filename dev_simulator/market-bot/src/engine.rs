use reqwest::StatusCode;

use crate::models::{BookSnapshot, EngineInfo, PlaceOrderRequest, PlaceResult, Trade};

#[derive(Clone)]
pub struct EngineClient {
    base: String,
    http: reqwest::Client,
}

impl EngineClient {
    pub fn new(base: String) -> anyhow::Result<Self> {
        Ok(Self {
            base,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()?,
        })
    }

    pub async fn health_ok(&self) -> bool {
        self.http
            .get(format!("{}/health", self.base))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub async fn engine(&self) -> anyhow::Result<EngineInfo> {
        let resp = self.http.get(format!("{}/v1/engine", self.base)).send().await?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("GET /v1/engine {status}: {body}");
        }
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn book(&self, symbol: &str) -> anyhow::Result<BookSnapshot> {
        let encoded = urlencoding(symbol);
        let resp = self
            .http
            .get(format!("{}/v1/books/{encoded}?levels=5", self.base))
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("GET /v1/books {status}: {body}");
        }
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn last_trade_price(&self, symbol: &str) -> Option<u64> {
        let encoded = urlencoding(symbol);
        let resp = self
            .http
            .get(format!("{}/v1/trades/{encoded}?limit=1", self.base))
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let trades: Vec<Trade> = resp.json().await.ok()?;
        trades.first().map(|t| t.price).filter(|p| *p > 0)
    }

    pub async fn place(&self, req: &PlaceOrderRequest) -> anyhow::Result<Result<PlaceResult, String>> {
        let resp = self
            .http
            .post(format!("{}/v1/orders", self.base))
            .json(req)
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if status == StatusCode::SERVICE_UNAVAILABLE {
            return Ok(Err(body));
        }
        if !status.is_success() {
            anyhow::bail!("POST /v1/orders {status}: {body}");
        }
        Ok(Ok(serde_json::from_str(&body)?))
    }
}

fn urlencoding(symbol: &str) -> String {
    symbol.replace('/', "%2F")
}
