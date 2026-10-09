//! Execution notice publisher for Message Center + partner server group.
//! Aligns with `doc/gateway_core_engine_logic.md` §4.8 and outbound notice shapes.

use std::sync::Arc;

use serde::Serialize;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::order::OrderView;
use crate::types::Trade;

#[derive(Clone)]
pub struct NoticePublisher {
    http: reqwest::Client,
    message_center_url: Option<String>,
    partner_notice_url: Option<String>,
    recent: Arc<RwLock<Vec<EngineNotice>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineNotice {
    pub notice_id: String,
    pub event_type: String,
    pub source_type: String,
    pub source_id: String,
    pub symbol: String,
    pub title: String,
    pub body: String,
    pub payload: serde_json::Value,
    pub targets: Vec<String>,
    pub created_at_ms: i64,
}

impl NoticePublisher {
    pub fn new(message_center_url: Option<String>, partner_notice_url: Option<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            message_center_url,
            partner_notice_url,
            recent: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn recent(&self, limit: usize) -> Vec<EngineNotice> {
        let guard = self.recent.read().await;
        guard.iter().rev().take(limit).cloned().collect()
    }

    pub async fn publish_trades(&self, trades: &[Trade], taker: &OrderView, maker: Option<&OrderView>) {
        for trade in trades {
            let notice = EngineNotice {
                notice_id: format!("n_{}", uuid::Uuid::new_v4().simple()),
                event_type: if taker.remaining_quantity == 0 {
                    "filled".into()
                } else {
                    "partial_filled".into()
                },
                source_type: "trade".into(),
                source_id: trade.trade_id.clone(),
                symbol: trade.symbol.clone(),
                title: format!("Trade executed on {}", trade.symbol),
                body: format!(
                    "qty {} @ {} (taker {} / maker {})",
                    trade.quantity, trade.price, trade.taker_order_id, trade.maker_order_id
                ),
                payload: serde_json::json!({
                    "trade": trade,
                    "taker_order": taker,
                    "maker_order": maker,
                }),
                targets: vec![
                    "message_center".into(),
                    "partner_server_group".into(),
                    "client_center".into(),
                ],
                created_at_ms: chrono::Utc::now().timestamp_millis(),
            };
            self.dispatch(notice).await;
        }
    }

    pub async fn publish_order_event(&self, event_type: &str, order: &OrderView) {
        let notice = EngineNotice {
            notice_id: format!("n_{}", uuid::Uuid::new_v4().simple()),
            event_type: event_type.into(),
            source_type: "order".into(),
            source_id: order.order_id.to_string(),
            symbol: order.symbol.clone(),
            title: format!("Order {event_type} on {}", order.symbol),
            body: format!(
                "order_id={} side={:?} type={:?} remaining={}",
                order.order_id, order.side, order.order_type, order.remaining_quantity
            ),
            payload: serde_json::json!({ "order": order }),
            targets: vec![
                "message_center".into(),
                "partner_server_group".into(),
                "client_center".into(),
            ],
            created_at_ms: chrono::Utc::now().timestamp_millis(),
        };
        self.dispatch(notice).await;
    }

    pub async fn publish_pool_created(
        &self,
        pair: &crate::types::TradingPair,
        pool: &crate::types::MarketPool,
    ) {
        let notice = EngineNotice {
            notice_id: format!("n_{}", uuid::Uuid::new_v4().simple()),
            event_type: "completed".into(),
            source_type: "order".into(),
            source_id: pool.pool_id.clone(),
            symbol: pair.symbol.clone(),
            title: format!("Market pool initialized for {}", pair.symbol),
            body: format!(
                "pool_depth={} initial_price={} base={} quote={} funding={}",
                pool.pool_depth,
                pool.initial_price,
                pool.base_amount,
                pool.quote_amount,
                pool.funding_source
            ),
            payload: serde_json::json!({
                "pair": pair,
                "pool": pool,
                "event": "admin_create_pair_with_pool"
            }),
            targets: vec![
                "message_center".into(),
                "partner_server_group".into(),
                "client_center".into(),
                "admin_api".into(),
            ],
            created_at_ms: chrono::Utc::now().timestamp_millis(),
        };
        self.dispatch(notice).await;
    }

    async fn dispatch(&self, notice: EngineNotice) {
        {
            let mut guard = self.recent.write().await;
            guard.push(notice.clone());
            if guard.len() > 500 {
                let drain = guard.len() - 500;
                guard.drain(0..drain);
            }
        }

        // Message Center / local message-queue style enqueue
        if let Some(url) = &self.message_center_url {
            let endpoint = format!("{}/v1/notices", url.trim_end_matches('/'));
            let body = serde_json::json!({
                "end_user_id": notice.payload.get("taker_order")
                    .and_then(|o| o.get("end_user_id"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
                "source_type": notice.source_type,
                "source_id": notice.source_id.parse::<i64>().unwrap_or(0),
                "event_type": notice.event_type,
                "title": notice.title,
                "body": notice.body,
                "payload": notice.payload,
                "callback_url": self.partner_notice_url,
            });
            match self.http.post(&endpoint).json(&body).send().await {
                Ok(resp) => info!(%endpoint, status = %resp.status(), "notice → message center"),
                Err(e) => warn!(%endpoint, error = %e, "message center notice failed"),
            }
        }

        // Partner server group callback (company partner notice channel)
        if let Some(url) = &self.partner_notice_url {
            let endpoint = url.clone();
            let body = serde_json::json!({
                "channel": "partner_server_group",
                "notice": notice,
                "source": "core_engine",
            });
            match self.http.post(&endpoint).json(&body).send().await {
                Ok(resp) => info!(%endpoint, status = %resp.status(), "notice → partner group"),
                Err(e) => warn!(%endpoint, error = %e, "partner notice failed"),
            }
        }
    }
}
