//! Settle into Client Center internal endpoints; notify users and Corp callback.

use serde_json::Value;

use crate::config::Config;
use crate::ingest::{CorpTokenPaymentCallback, ShopPaymentCallback};
use crate::store::{backoff_ms, now_ms, EventStore, InsertOutcome};

pub async fn accept_shop_payment(
    store: &EventStore,
    config: &Config,
    http: &reqwest::Client,
    body: ShopPaymentCallback,
    source: &str,
) -> Result<Value, String> {
    if body.event_id.trim().is_empty() {
        return Err("event_id required".into());
    }
    if body.signature != config.callback_signature {
        return Err("invalid callback signature".into());
    }
    let payload = serde_json::to_value(&body).unwrap_or(Value::Null);
    let outcome = store
        .insert_received(
            &body.event_id,
            source,
            "shop_payment",
            &payload,
            Some(body.shop_order_id),
            Some(body.partner_order_no.as_str()),
        )
        .await?;
    match outcome {
        InsertOutcome::Duplicate(ev) => Ok(serde_json::json!({
            "accepted": true,
            "duplicate": true,
            "event_id": ev.event_id,
            "delivery_status": ev.delivery_status,
        })),
        InsertOutcome::Inserted(ev) => {
            settle_shop_event(store, config, http, &ev.event_id, &body).await?;
            let latest = store.get_by_event_id(&ev.event_id).await.unwrap_or(ev);
            Ok(serde_json::json!({
                "accepted": true,
                "duplicate": false,
                "event_id": latest.event_id,
                "delivery_status": latest.delivery_status,
                "last_error": latest.last_error,
            }))
        }
    }
}

pub async fn accept_corp_token_payment(
    store: &EventStore,
    config: &Config,
    http: &reqwest::Client,
    body: CorpTokenPaymentCallback,
    source: &str,
) -> Result<Value, String> {
    if body.event_id.trim().is_empty() {
        return Err("event_id required".into());
    }
    if body.signature != config.callback_signature {
        return Err("invalid callback signature".into());
    }
    let payload = serde_json::to_value(&body).unwrap_or(Value::Null);
    let outcome = store
        .insert_received(
            &body.event_id,
            source,
            "corp_token",
            &payload,
            Some(body.corp_token_order_id),
            Some(body.partner_order_no.as_str()),
        )
        .await?;
    match outcome {
        InsertOutcome::Duplicate(ev) => Ok(serde_json::json!({
            "accepted": true,
            "duplicate": true,
            "event_id": ev.event_id,
            "delivery_status": ev.delivery_status,
        })),
        InsertOutcome::Inserted(ev) => {
            settle_cbt_event(store, config, http, &ev.event_id, &body).await?;
            let latest = store.get_by_event_id(&ev.event_id).await.unwrap_or(ev);
            Ok(serde_json::json!({
                "accepted": true,
                "duplicate": false,
                "event_id": latest.event_id,
                "delivery_status": latest.delivery_status,
                "last_error": latest.last_error,
            }))
        }
    }
}

pub async fn settle_shop_event(
    store: &EventStore,
    config: &Config,
    http: &reqwest::Client,
    event_id: &str,
    body: &ShopPaymentCallback,
) -> Result<(), String> {
    let _ = store.mark_processing(event_id).await;
    let url = format!(
        "{}{}",
        config.client_center_url,
        funnyx_net_api::paths::INTERNAL_SHOP_SETTLE
    );
    let req = serde_json::json!({
        "shop_order_id": body.shop_order_id,
        "partner_order_no": body.partner_order_no,
        "status": body.status,
        "event_id": body.event_id,
        "fiat_currency": body.fiat_currency,
        "fiat_paid": body.fiat_paid,
        "seller_type": body.seller_type,
    });
    let res = http
        .post(&url)
        .header("x-internal-key", &config.internal_api_key)
        .json(&req)
        .send()
        .await;
    match res {
        Ok(resp) => {
            let status = resp.status();
            let json: Value = resp.json().await.unwrap_or(Value::Null);
            if status.is_success() {
                store.mark_settled(event_id).await?;
                if let Some(order) = json.get("order") {
                    if let Some(uid) = order.get("end_user_id").and_then(|v| v.as_i64()) {
                        let notice_type = match body.status.as_str() {
                            "paid" => "completed",
                            "failed" => "failed",
                            "cancelled" => "cancelled",
                            _ => "status_changed",
                        };
                        let _ = store
                            .insert_outbound_notice(
                                uid,
                                body.shop_order_id,
                                notice_type,
                                "Shop order update",
                                &format!("shop_order {} {}", body.shop_order_id, body.status),
                                &json,
                            )
                            .await;
                    }
                }
                Ok(())
            } else {
                fail_retry(store, event_id, &format!("cc {status}: {json}")).await
            }
        }
        Err(e) => fail_retry(store, event_id, &e.to_string()).await,
    }
}

pub async fn settle_cbt_event(
    store: &EventStore,
    config: &Config,
    http: &reqwest::Client,
    event_id: &str,
    body: &CorpTokenPaymentCallback,
) -> Result<(), String> {
    let _ = store.mark_processing(event_id).await;
    let url = format!(
        "{}{}",
        config.client_center_url,
        funnyx_net_api::paths::INTERNAL_CORP_TOKEN_SETTLE
    );
    let req = serde_json::json!({
        "corp_token_order_id": body.corp_token_order_id,
        "partner_order_no": body.partner_order_no,
        "status": body.status,
        "event_id": body.event_id,
        "coin_amount": body.coin_amount,
    });
    let res = http
        .post(&url)
        .header("x-internal-key", &config.internal_api_key)
        .json(&req)
        .send()
        .await;
    match res {
        Ok(resp) => {
            let status = resp.status();
            let json: Value = resp.json().await.unwrap_or(Value::Null);
            if status.is_success() {
                store.mark_settled(event_id).await?;
                let order = json.get("order");
                if let Some(uid) = order.and_then(|o| o.get("end_user_id")).and_then(|v| v.as_i64())
                {
                    let notice_type = match body.status.as_str() {
                        "paid" => "completed",
                        "failed" => "failed",
                        "cancelled" => "cancelled",
                        _ => "status_changed",
                    };
                    let _ = store
                        .insert_outbound_notice_typed(
                            uid,
                            "corp_token_order",
                            body.corp_token_order_id,
                            notice_type,
                            "Company Basic Token order update",
                            &format!(
                                "corp_token_order {} {}",
                                body.corp_token_order_id, body.status
                            ),
                            &json,
                        )
                        .await;
                }

                let corp_id = json
                    .get("corporate_user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                if corp_id > 0 {
                    let _ = store
                        .insert_corp_partner_notice(
                            corp_id,
                            "Company Basic Token payment",
                            &format!(
                                "Order {} status={} — end-user balance credited when paid.",
                                body.corp_token_order_id, body.status
                            ),
                        )
                        .await;
                }

                if body.status == "paid" {
                    let callback = json
                        .get("partner_callback_url")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty());
                    let callback = match callback {
                        Some(u) => Some(u),
                        None if corp_id > 0 => {
                            store.get_corp_token_callback_url(corp_id).await
                        }
                        None => None,
                    };
                    if let Some(cb_url) = callback {
                        let credited = order
                            .and_then(|o| o.get("credited_coin_amount"))
                            .cloned()
                            .unwrap_or(Value::Null);
                        let fee = order
                            .and_then(|o| o.get("fee_coin_amount"))
                            .cloned()
                            .unwrap_or(Value::Null);
                        let balance_payload = serde_json::json!({
                            "event_type": "corp_token_balance_updated",
                            "event_id": body.event_id,
                            "corp_token_order_id": body.corp_token_order_id,
                            "partner_order_no": body.partner_order_no,
                            "corporate_user_id": corp_id,
                            "end_user_id": order.and_then(|o| o.get("end_user_id")).cloned(),
                            "game_account_id": order.and_then(|o| o.get("game_account_id")).cloned(),
                            "game_coin_id": order.and_then(|o| o.get("game_coin_id")).cloned(),
                            "coin_amount": order.and_then(|o| o.get("coin_amount")).cloned(),
                            "credited_coin_amount": credited,
                            "fee_coin_amount": fee,
                            "status": "paid",
                        });
                        let cb_res = http.post(&cb_url).json(&balance_payload).send().await;
                        match cb_res {
                            Ok(r) if r.status().is_success() => {}
                            Ok(r) => {
                                tracing::warn!(
                                    "corp balance callback {} returned {}",
                                    cb_url,
                                    r.status()
                                );
                            }
                            Err(e) => {
                                tracing::warn!("corp balance callback {} failed: {e}", cb_url);
                            }
                        }
                    }
                }
                Ok(())
            } else {
                fail_retry(store, event_id, &format!("cc {status}: {json}")).await
            }
        }
        Err(e) => fail_retry(store, event_id, &e.to_string()).await,
    }
}

async fn fail_retry(store: &EventStore, event_id: &str, err: &str) -> Result<(), String> {
    let ev = store.get_by_event_id(event_id).await;
    let retry_count = ev.as_ref().map(|e| e.retry_count + 1).unwrap_or(1);
    let next = now_ms() + backoff_ms(retry_count);
    store
        .mark_failed(
            event_id,
            &err.chars().take(512).collect::<String>(),
            retry_count,
            next,
        )
        .await?;
    Ok(())
}

pub fn spawn_retry_worker(store: EventStore, config: Config, http: reqwest::Client) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            ticker.tick().await;
            let due = store.due_retries().await;
            for ev in due {
                match ev.event_type.as_str() {
                    "shop_payment" => {
                        let Ok(body) =
                            serde_json::from_value::<ShopPaymentCallback>(ev.payload.clone())
                        else {
                            let _ = store
                                .mark_failed(
                                    &ev.event_id,
                                    "payload not shop_payment",
                                    ev.retry_count + 1,
                                    now_ms(),
                                )
                                .await;
                            continue;
                        };
                        let _ =
                            settle_shop_event(&store, &config, &http, &ev.event_id, &body).await;
                    }
                    "corp_token" => {
                        let Ok(body) =
                            serde_json::from_value::<CorpTokenPaymentCallback>(ev.payload.clone())
                        else {
                            let _ = store
                                .mark_failed(
                                    &ev.event_id,
                                    "payload not corp_token",
                                    ev.retry_count + 1,
                                    now_ms(),
                                )
                                .await;
                            continue;
                        };
                        let _ = settle_cbt_event(&store, &config, &http, &ev.event_id, &body).await;
                    }
                    _ => {}
                }
            }
        }
    });
}
