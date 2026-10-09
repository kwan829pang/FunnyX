//! Outbound partner HTTP POST + delivery logging.

use serde_json::{json, Value};
use tracing::{debug, warn};

use crate::config::Config;
use crate::models::OutboundNotice;
use crate::store::NoticeStore;

/// Resolve partner callback URL for a notice.
pub async fn resolve_callback_url(
    store: &NoticeStore,
    config: &Config,
    notice: &OutboundNotice,
) -> Option<String> {
    if let Some(url) = notice
        .payload
        .get("callback_url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        return Some(url.to_string());
    }
    if let Some(corp_id) = notice
        .payload
        .get("corporate_user_id")
        .and_then(|v| v.as_i64())
        .filter(|&id| id > 0)
    {
        let kind = match notice.source_type.as_str() {
            "shop_order" => "shop_payment",
            "corp_token_order" => "corp_token",
            "deposit_withdrawal_txn" => notice
                .payload
                .get("transaction_type")
                .and_then(|v| v.as_str())
                .unwrap_or("deposit"),
            _ => "shop_payment",
        };
        if let Some(url) = store.resolve_webhook_endpoint(corp_id, kind).await {
            return Some(url);
        }
    }
    config.partner_notice_url.clone()
}

pub async fn push_partner(
    http: &reqwest::Client,
    store: &NoticeStore,
    notice: &OutboundNotice,
    url: &str,
) -> Result<(), String> {
    let event_id = format!("mc_notice_{}", notice.id);
    let body = json!({
        "channel": "message_center",
        "event_id": event_id,
        "notice_id": notice.id,
        "end_user_id": notice.end_user_id,
        "source_type": notice.source_type,
        "source_id": notice.source_id,
        "event_type": notice.event_type,
        "title": notice.title,
        "body": notice.body,
        "payload": notice.payload,
        "delivery_status": "sending",
    });
    match http.post(url).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => {
            debug!(%url, notice_id = notice.id, "partner notice delivered");
            store
                .log_delivery_event(
                    &event_id,
                    &notice.event_type,
                    &body,
                    "settled",
                    None,
                )
                .await;
            Ok(())
        }
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            let err = format!("partner HTTP {status}: {text}");
            warn!(%url, notice_id = notice.id, %err, "partner notice failed");
            store
                .log_delivery_event(
                    &event_id,
                    &notice.event_type,
                    &body,
                    "failed",
                    Some(&err),
                )
                .await;
            Err(err)
        }
        Err(e) => {
            let err = e.to_string();
            warn!(%url, notice_id = notice.id, %err, "partner notice transport failed");
            store
                .log_delivery_event(
                    &event_id,
                    &notice.event_type,
                    &body,
                    "failed",
                    Some(&err),
                )
                .await;
            Err(err)
        }
    }
}

pub fn notification_payload(notice: &OutboundNotice) -> Value {
    json!({
        "outbound_notice_id": notice.id,
        "source_type": notice.source_type,
        "source_id": notice.source_id,
        "event_type": notice.event_type,
        "title": notice.title,
        "body": notice.body,
        "payload": notice.payload,
    })
}

pub fn notification_type(notice: &OutboundNotice) -> String {
    format!("{}.{}", notice.source_type, notice.event_type)
}
