//! Session place path: Gateway → Client Center → Core Engine.

use std::sync::Arc;

use reqwest::StatusCode;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::models::{BookSnapshot, OrderType, SessionPlaceResult, Side};

#[derive(Clone)]
pub struct SessionClient {
    gateway: String,
    username: String,
    password: String,
    game_account_id: i64,
    http: reqwest::Client,
    token: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Serialize)]
struct LoginBody<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
struct PlaceBody {
    symbol: String,
    side: Side,
    order_type: OrderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<u64>,
    quantity: u64,
    game_account_id: i64,
}

impl SessionClient {
    pub fn new(cfg: &Config) -> anyhow::Result<Option<Self>> {
        let Some(gateway) = cfg.gateway_url.clone() else {
            return Ok(None);
        };
        Ok(Some(Self {
            gateway,
            username: cfg.bot_username.clone(),
            password: cfg.bot_password.clone(),
            game_account_id: cfg.game_account_id,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()?,
            token: Arc::new(Mutex::new(None)),
        }))
    }

    pub fn gateway_url(&self) -> &str {
        &self.gateway
    }

    async fn login(&self) -> anyhow::Result<String> {
        let resp = self
            .http
            .post(format!("{}/v1/client/login", self.gateway))
            .json(&LoginBody {
                username: &self.username,
                password: &self.password,
            })
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("POST /v1/client/login {status}: {body}");
        }
        let v: serde_json::Value = serde_json::from_str(&body)?;
        let token = v
            .get("access_token")
            .and_then(|t| t.as_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow::anyhow!("login missing access_token"))?
            .to_string();
        *self.token.lock().await = Some(token.clone());
        Ok(token)
    }

    async fn bearer(&self) -> anyhow::Result<String> {
        if let Some(t) = self.token.lock().await.clone() {
            return Ok(t);
        }
        self.login().await
    }

    /// `GET /v1/markets` → (market_id, symbol) pairs.
    pub async fn list_markets(&self) -> anyhow::Result<Vec<(i64, String)>> {
        let token = self.bearer().await?;
        let resp = self
            .http
            .get(format!("{}/v1/markets", self.gateway))
            .header("authorization", format!("Bearer {token}"))
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if status == StatusCode::UNAUTHORIZED {
            self.login().await?;
            return Box::pin(self.list_markets()).await;
        }
        if !status.is_success() {
            anyhow::bail!("GET /v1/markets {status}: {body}");
        }
        let v: serde_json::Value = serde_json::from_str(&body)?;
        let mut out = Vec::new();
        if let Some(arr) = v.get("markets").and_then(|m| m.as_array()) {
            for m in arr {
                let id = m.get("id").and_then(|x| x.as_i64()).unwrap_or(0);
                let symbol = m
                    .get("symbol")
                    .or_else(|| m.get("market_name"))
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_ascii_uppercase();
                if id > 0 && !symbol.is_empty() {
                    out.push((id, symbol));
                }
            }
        }
        Ok(out)
    }

    pub async fn book(&self, market_id: i64) -> anyhow::Result<BookSnapshot> {
        let token = self.bearer().await?;
        let resp = self
            .http
            .get(format!(
                "{}/v1/markets/{market_id}/book?levels=5",
                self.gateway
            ))
            .header("authorization", format!("Bearer {token}"))
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if status == StatusCode::UNAUTHORIZED {
            self.login().await?;
            return Box::pin(self.book(market_id)).await;
        }
        if !status.is_success() {
            anyhow::bail!("GET /v1/markets/{market_id}/book {status}: {body}");
        }
        let v: serde_json::Value = serde_json::from_str(&body)?;
        let book_val = v.get("book").cloned().unwrap_or(v);
        Ok(serde_json::from_value(book_val)?)
    }

    pub async fn book_for_symbol(&self, symbol: &str) -> anyhow::Result<BookSnapshot> {
        let markets = self.list_markets().await?;
        let want = symbol.to_ascii_uppercase();
        let id = markets
            .into_iter()
            .find(|(_, s)| s == &want)
            .map(|(id, _)| id)
            .ok_or_else(|| anyhow::anyhow!("no Session market for symbol {want}"))?;
        self.book(id).await
    }

    pub async fn place(
        &self,
        symbol: &str,
        side: Side,
        order_type: OrderType,
        price: Option<u64>,
        quantity: u64,
    ) -> anyhow::Result<Result<SessionPlaceResult, String>> {
        match self
            .place_once(symbol, side, order_type, price, quantity, false)
            .await?
        {
            Ok(r) => Ok(Ok(r)),
            Err(PlaceErr::Unauthorized) => {
                self.login().await?;
                match self
                    .place_once(symbol, side, order_type, price, quantity, true)
                    .await?
                {
                    Ok(r) => Ok(Ok(r)),
                    Err(PlaceErr::Rejected(msg)) => Ok(Err(msg)),
                    Err(PlaceErr::Unauthorized) => Ok(Err("unauthorized after re-login".into())),
                    Err(PlaceErr::Other(e)) => Err(e),
                }
            }
            Err(PlaceErr::Rejected(msg)) => Ok(Err(msg)),
            Err(PlaceErr::Other(e)) => Err(e),
        }
    }

    async fn place_once(
        &self,
        symbol: &str,
        side: Side,
        order_type: OrderType,
        price: Option<u64>,
        quantity: u64,
        after_relogin: bool,
    ) -> anyhow::Result<Result<SessionPlaceResult, PlaceErr>> {
        let token = self.bearer().await?;
        let resp = self
            .http
            .post(format!("{}/v1/orders", self.gateway))
            .header("authorization", format!("Bearer {token}"))
            .json(&PlaceBody {
                symbol: symbol.to_string(),
                side,
                order_type,
                price,
                quantity,
                game_account_id: self.game_account_id,
            })
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if status == StatusCode::UNAUTHORIZED && !after_relogin {
            return Ok(Err(PlaceErr::Unauthorized));
        }
        if status == StatusCode::SERVICE_UNAVAILABLE
            || status == StatusCode::BAD_REQUEST
            || status == StatusCode::FORBIDDEN
            || status == StatusCode::NOT_FOUND
        {
            return Ok(Err(PlaceErr::Rejected(format!("{status}: {body}"))));
        }
        if !status.is_success() {
            return Ok(Err(PlaceErr::Other(anyhow::anyhow!(
                "POST /v1/orders {status}: {body}"
            ))));
        }
        match serde_json::from_str::<SessionPlaceResult>(&body) {
            Ok(parsed) => Ok(Ok(parsed)),
            Err(e) => Ok(Err(PlaceErr::Other(anyhow::anyhow!(
                "decode place response: {e}; body={body}"
            )))),
        }
    }
}

enum PlaceErr {
    Unauthorized,
    Rejected(String),
    Other(anyhow::Error),
}
