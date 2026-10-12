//! Best-effort push to Gateway Session sockets after inbox write.

use reqwest::Client;
use tracing::warn;

use crate::config::Config;
use crate::models::OutboundNotice;

pub async fn push_to_gateway(
    http: &Client,
    config: &Config,
    notice: &OutboundNotice,
    notification_id: i64,
) {
    let Some(base) = config.gateway_url.as_ref() else {
        return;
    };
    if notice.end_user_id <= 0 {
        return;
    }
    let url = format!("{base}/v1/internal/socket/push");
    let body = serde_json::json!({
        "end_user_id": notice.end_user_id,
        "notification_id": notification_id,
        "source_type": notice.source_type,
        "event_type": notice.event_type,
        "title": notice.title,
        "body": notice.body,
        "payload": notice.payload,
    });
    match http
        .post(&url)
        .header("x-internal-key", &config.internal_api_key)
        .json(&body)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {}
        Ok(resp) => {
            warn!(
                notice_id = notice.id,
                status = %resp.status(),
                "gateway socket push non-success"
            );
        }
        Err(e) => {
            warn!(notice_id = notice.id, error = %e, "gateway socket push failed");
        }
    }
}
