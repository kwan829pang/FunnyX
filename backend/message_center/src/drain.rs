//! Background drain of pending outbound_notices.

use std::time::Duration;

use tokio::time::sleep;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::mongo::NoticeHistory;
use crate::partner::{self, notification_payload, notification_type};
use crate::store::{now_ms, NoticeStore};

pub fn spawn_drain_worker(
    store: NoticeStore,
    config: Config,
    http: reqwest::Client,
    history: NoticeHistory,
) {
    tokio::spawn(async move {
        let interval = Duration::from_millis(config.drain_interval_ms.max(500));
        info!(
            interval_ms = config.drain_interval_ms,
            batch = config.drain_batch_size,
            "message center drain worker started"
        );
        loop {
            if let Err(e) = drain_once(&store, &config, &http, &history).await {
                warn!(error = %e, "drain cycle failed");
            }
            sleep(interval).await;
        }
    });
}

async fn drain_once(
    store: &NoticeStore,
    config: &Config,
    http: &reqwest::Client,
    history: &NoticeHistory,
) -> Result<(), String> {
    let now = now_ms();
    let batch = store.claim_pending(config.drain_batch_size, now).await?;
    if batch.is_empty() {
        return Ok(());
    }
    debug!(count = batch.len(), "claimed outbound notices");
    for notice in batch {
        process_one(store, config, http, history, notice).await;
    }
    Ok(())
}

async fn process_one(
    store: &NoticeStore,
    config: &Config,
    http: &reqwest::Client,
    history: &NoticeHistory,
    notice: crate::models::OutboundNotice,
) {
    let now = now_ms();
    let type_str = notification_type(&notice);
    let payload = notification_payload(&notice);

    let notif_id = match store
        .insert_notification(notice.end_user_id, &type_str, &payload, now)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            warn!(notice_id = notice.id, error = %e, "insert notification failed");
            let retry = notice.retry_count.saturating_add(1);
            let backoff = backoff_ms(retry);
            let _ = store
                .mark_retry_or_failed(
                    notice.id,
                    retry,
                    config.max_delivery_retries,
                    now,
                    backoff,
                    &e,
                )
                .await;
            return;
        }
    };

    let mut partner_err: Option<String> = None;
    if let Some(url) = partner::resolve_callback_url(store, config, &notice).await {
        if let Err(e) = partner::push_partner(http, store, &notice, &url).await {
            partner_err = Some(e);
        }
    }

    // User inbox is source of truth for "sent"; partner failure schedules retry
    // only when we intentionally want re-push — for v1, mark sent after inbox write,
    // and log partner failure on the notice payload via a soft retry if partner URL set.
    if let Some(err) = partner_err {
        // Still mark sent for user notification; partner retries are best-effort via payload flag.
        // If we never wrote notification before, we already did — mark sent.
        let _ = store.mark_sent(notice.id, notif_id, now).await;
        let mut n = notice.clone();
        n.sent_at = now;
        n.notification_id = Some(notif_id);
        history.mirror_sent(&n, notif_id).await;
        warn!(
            notice_id = notice.id,
            error = %err,
            "notice sent to inbox; partner push failed (logged)"
        );
        return;
    }

    if let Err(e) = store.mark_sent(notice.id, notif_id, now).await {
        warn!(notice_id = notice.id, error = %e, "mark sent failed");
        return;
    }
    let mut n = notice;
    n.sent_at = now;
    n.notification_id = Some(notif_id);
    history.mirror_sent(&n, notif_id).await;
}

fn backoff_ms(retry: i32) -> i64 {
    let base = 5_000i64;
    base.saturating_mul(1 << (retry.min(6) as u32))
}
