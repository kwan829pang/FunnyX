use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use chrono::{Duration, Utc};
use serde_json::json;
use tokio::sync::RwLock;

use crate::models::{
    CreateDepositRequest, CreateShopPaymentRequest, DepositCallbackPayload, PaymentView,
    ShopCallbackPayload,
};

#[derive(Debug, Clone)]
pub enum PaymentKind {
    Shop,
    Deposit,
}

#[derive(Debug, Clone)]
pub struct PaymentRecord {
    pub kind: PaymentKind,
    pub partner_order_no: String,
    pub status: String,
    pub source: String,
    pub callback_url: Option<String>,
    pub callback_attempts: u32,
    pub last_callback_status: Option<u16>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub shop: Option<CreateShopPaymentRequest>,
    pub deposit: Option<CreateDepositRequest>,
    pub last_event_id: Option<String>,
}

impl PaymentRecord {
    pub fn to_view(&self) -> PaymentView {
        let detail = match self.kind {
            PaymentKind::Shop => serde_json::to_value(self.shop.as_ref()).unwrap_or(json!({})),
            PaymentKind::Deposit => {
                serde_json::to_value(self.deposit.as_ref()).unwrap_or(json!({}))
            }
        };
        PaymentView {
            kind: match self.kind {
                PaymentKind::Shop => "shop".into(),
                PaymentKind::Deposit => "deposit".into(),
            },
            partner_order_no: self.partner_order_no.clone(),
            status: self.status.clone(),
            sim_status: None,
            sim_delay_ms: None,
            source: self.source.clone(),
            callback_url: self.callback_url.clone(),
            callback_attempts: self.callback_attempts,
            last_callback_status: self.last_callback_status,
            created_at: self.created_at,
            updated_at: self.updated_at,
            expires_at: self.expires_at,
            detail,
        }
    }

    pub fn to_view_with_sim(&self, sim_status: &str, sim_delay_ms: u64) -> PaymentView {
        let mut view = self.to_view();
        view.sim_status = Some(sim_status.to_string());
        view.sim_delay_ms = Some(sim_delay_ms);
        view
    }
}

#[derive(Clone)]
pub struct AppState {
    pub public_base_url: String,
    pub callback_signature: String,
    pub payment_ttl_secs: i64,
    pub http: reqwest::Client,
    seq: Arc<AtomicU64>,
    payments: Arc<RwLock<HashMap<String, PaymentRecord>>>,
}

impl AppState {
    pub fn new(public_base_url: String, callback_signature: String, payment_ttl_secs: i64) -> Self {
        Self {
            public_base_url,
            callback_signature,
            payment_ttl_secs,
            http: reqwest::Client::new(),
            seq: Arc::new(AtomicU64::new(1000)),
            payments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn next_order_no(&self, prefix: &str) -> String {
        let n = self.seq.fetch_add(1, Ordering::Relaxed);
        format!("{prefix}-{n}")
    }

    pub async fn create_shop(
        &self,
        req: CreateShopPaymentRequest,
    ) -> anyhow::Result<(PaymentRecord, String)> {
        let now = Utc::now();
        let partner_order_no = self.next_order_no("PAY");
        let expires_at = now + Duration::seconds(self.payment_ttl_secs);
        let checkout_url = format!("{}/pay/{}", self.public_base_url, partner_order_no);
        let record = PaymentRecord {
            kind: PaymentKind::Shop,
            partner_order_no: partner_order_no.clone(),
            status: "pending".into(),
            source: "test".into(),
            callback_url: Some(req.callback_url.clone()),
            callback_attempts: 0,
            last_callback_status: None,
            created_at: req.created_at.unwrap_or(now),
            updated_at: now,
            expires_at: Some(expires_at),
            shop: Some(req),
            deposit: None,
            last_event_id: None,
        };
        self.payments
            .write()
            .await
            .insert(partner_order_no.clone(), record.clone());
        Ok((record, checkout_url))
    }

    pub async fn create_deposit(
        &self,
        req: CreateDepositRequest,
    ) -> anyhow::Result<PaymentRecord> {
        let now = Utc::now();
        let partner_order_no = self.next_order_no("DEP");
        let source = if req.source.is_empty() {
            "test".into()
        } else {
            req.source.clone()
        };
        let record = PaymentRecord {
            kind: PaymentKind::Deposit,
            partner_order_no: partner_order_no.clone(),
            status: "pending".into(),
            source,
            callback_url: req.callback_url.clone(),
            callback_attempts: 0,
            last_callback_status: None,
            created_at: now,
            updated_at: now,
            expires_at: None,
            shop: None,
            deposit: Some(req),
            last_event_id: None,
        };
        self.payments
            .write()
            .await
            .insert(partner_order_no, record.clone());
        Ok(record)
    }

    pub async fn get(&self, partner_order_no: &str) -> Option<PaymentRecord> {
        self.payments.read().await.get(partner_order_no).cloned()
    }

    pub async fn list(&self) -> Vec<PaymentView> {
        let guard = self.payments.read().await;
        let mut rows: Vec<_> = guard.values().map(PaymentRecord::to_view).collect();
        rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        rows
    }

    pub async fn complete(
        &self,
        partner_order_no: &str,
        status: &str,
        fire_callback: bool,
    ) -> anyhow::Result<PaymentRecord> {
        let mut guard = self.payments.write().await;
        let record = guard
            .get_mut(partner_order_no)
            .ok_or_else(|| anyhow::anyhow!("payment not found"))?;

        let allowed = match record.kind {
            PaymentKind::Shop => matches!(status, "paid" | "failed" | "cancelled"),
            PaymentKind::Deposit => {
                matches!(status, "success" | "failed" | "rejected" | "cancelled")
            }
        };
        if !allowed {
            anyhow::bail!("invalid status '{status}' for {:?}", record.kind);
        }

        let now = Utc::now();
        let event_id = format!("evt_{}", uuid::Uuid::new_v4().simple());
        record.status = status.to_string();
        record.updated_at = now;
        record.last_event_id = Some(event_id.clone());

        let snapshot = record.clone();
        drop(guard);

        if fire_callback {
            if let Some(url) = snapshot.callback_url.clone() {
                let (http_status, _) = self.post_callback(&snapshot, &event_id, &url).await?;
                let mut guard = self.payments.write().await;
                if let Some(r) = guard.get_mut(partner_order_no) {
                    r.callback_attempts = r.callback_attempts.saturating_add(1);
                    r.last_callback_status = Some(http_status);
                    r.updated_at = Utc::now();
                    return Ok(r.clone());
                }
            }
        }

        Ok(snapshot)
    }

    async fn post_callback(
        &self,
        record: &PaymentRecord,
        event_id: &str,
        url: &str,
    ) -> anyhow::Result<(u16, String)> {
        let paid_at = Utc::now();
        let body = match record.kind {
            PaymentKind::Shop => {
                let shop = record
                    .shop
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("missing shop payload"))?;
                let fiat_paid = if record.status == "paid" {
                    shop.fiat_price
                } else {
                    0.0
                };
                serde_json::to_value(ShopCallbackPayload {
                    event_id: event_id.to_string(),
                    partner_order_no: record.partner_order_no.clone(),
                    shop_order_id: shop.shop_order_id,
                    seller_type: shop.seller_type.clone(),
                    status: record.status.clone(),
                    fiat_currency: shop.fiat_currency.clone(),
                    fiat_paid,
                    credit_game_coin: shop.credit_game_coin.clone(),
                    credit_amount: shop.credit_amount,
                    paid_at,
                    signature: self.callback_signature.clone(),
                    source: "test".into(),
                })?
            }
            PaymentKind::Deposit => {
                let dep = record
                    .deposit
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("missing deposit payload"))?;
                serde_json::to_value(DepositCallbackPayload {
                    event_id: event_id.to_string(),
                    partner_order_no: record.partner_order_no.clone(),
                    request_id: dep.request_id.clone(),
                    partner_id: dep.partner_id.clone(),
                    user_id: dep.user_id.clone(),
                    game_id: dep.game_id.clone(),
                    game_account_id: dep.game_account_id.clone(),
                    transaction_type: dep.transaction_type.clone(),
                    amount: dep.amount,
                    game_coin: dep.game_coin.clone(),
                    status: record.status.clone(),
                    paid_at,
                    signature: self.callback_signature.clone(),
                    source: record.source.clone(),
                })?
            }
        };

        tracing::info!(%url, order = %record.partner_order_no, "posting webhook callback");
        let resp = self.http.post(url).json(&body).send().await?;
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        tracing::info!(%status, body = %text, "callback response");
        Ok((status, text))
    }
}
