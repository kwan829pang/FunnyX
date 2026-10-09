//! Postgres outbox + notifications (memory fallback for local smoke).

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::RwLock;

use crate::models::{NotificationView, NoticeStats, OutboundNotice};

#[derive(Clone)]
pub struct NoticeStore {
    pool: Option<sqlx::PgPool>,
    memory: Arc<RwLock<MemoryInner>>,
    next_id: Arc<AtomicI64>,
    next_notif: Arc<AtomicI64>,
}

struct MemoryInner {
    notices: HashMap<i64, OutboundNotice>,
    notifications: HashMap<i64, NotificationView>,
}

impl NoticeStore {
    pub fn new(pool: Option<sqlx::PgPool>) -> Self {
        Self {
            pool,
            memory: Arc::new(RwLock::new(MemoryInner {
                notices: HashMap::new(),
                notifications: HashMap::new(),
            })),
            next_id: Arc::new(AtomicI64::new(1)),
            next_notif: Arc::new(AtomicI64::new(1)),
        }
    }

    pub async fn enqueue(
        &self,
        end_user_id: i64,
        source_type: &str,
        source_id: i64,
        event_type: &str,
        title: &str,
        body: &str,
        mut payload: Value,
        scheduled_at: i64,
        callback_url: Option<String>,
    ) -> Result<OutboundNotice, String> {
        if let Some(url) = callback_url.filter(|s| !s.is_empty()) {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("callback_url".into(), Value::String(url));
            }
        }
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let row = sqlx::query_as::<_, NoticeRow>(
                "INSERT INTO fx_events.outbound_notices ( \
                    end_user_id, source_type, source_id, event_type, title, body, payload, \
                    delivery_status, retry_count, scheduled_at, sent_at, created_at, updated_at \
                 ) VALUES ($1,$2,$3,$4,$5,$6,$7::jsonb,'pending',0,$8,0,$9,0) \
                 RETURNING id, end_user_id, source_type, source_id, event_type, title, body, \
                    payload, delivery_status, retry_count, scheduled_at, sent_at, notification_id, \
                    created_at, updated_at",
            )
            .bind(end_user_id)
            .bind(source_type)
            .bind(source_id)
            .bind(event_type)
            .bind(title)
            .bind(body)
            .bind(payload.to_string())
            .bind(scheduled_at)
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(row.into());
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let notice = OutboundNotice {
            id,
            end_user_id,
            source_type: source_type.into(),
            source_id,
            event_type: event_type.into(),
            title: title.into(),
            body: body.into(),
            payload,
            delivery_status: "pending".into(),
            retry_count: 0,
            scheduled_at,
            sent_at: 0,
            notification_id: None,
            created_at: now,
            updated_at: 0,
        };
        self.memory.write().await.notices.insert(id, notice.clone());
        Ok(notice)
    }

    pub async fn claim_pending(&self, limit: i64, now: i64) -> Result<Vec<OutboundNotice>, String> {
        if let Some(pool) = &self.pool {
            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
            let rows = sqlx::query_as::<_, NoticeRow>(
                "UPDATE fx_events.outbound_notices n SET \
                    delivery_status = 'sending', updated_at = $1 \
                 WHERE n.id IN ( \
                    SELECT id FROM fx_events.outbound_notices \
                    WHERE delivery_status = 'pending' \
                      AND (scheduled_at = 0 OR scheduled_at <= $1) \
                    ORDER BY scheduled_at ASC, created_at ASC \
                    FOR UPDATE SKIP LOCKED \
                    LIMIT $2 \
                 ) \
                 RETURNING id, end_user_id, source_type, source_id, event_type, title, body, \
                    payload, delivery_status, retry_count, scheduled_at, sent_at, notification_id, \
                    created_at, updated_at",
            )
            .bind(now)
            .bind(limit)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            tx.commit().await.map_err(|e| e.to_string())?;
            return Ok(rows.into_iter().map(Into::into).collect());
        }

        let mut mem = self.memory.write().await;
        let mut ids: Vec<i64> = mem
            .notices
            .values()
            .filter(|n| {
                n.delivery_status == "pending" && (n.scheduled_at == 0 || n.scheduled_at <= now)
            })
            .map(|n| n.id)
            .collect();
        ids.sort_unstable();
        ids.truncate(limit as usize);
        let mut out = Vec::new();
        for id in ids {
            if let Some(n) = mem.notices.get_mut(&id) {
                n.delivery_status = "sending".into();
                n.updated_at = now;
                out.push(n.clone());
            }
        }
        Ok(out)
    }

    pub async fn mark_sent(
        &self,
        notice_id: i64,
        notification_id: i64,
        now: i64,
    ) -> Result<(), String> {
        if let Some(pool) = &self.pool {
            sqlx::query(
                "UPDATE fx_events.outbound_notices SET \
                    delivery_status = 'sent', notification_id = $2, sent_at = $3, updated_at = $3 \
                 WHERE id = $1",
            )
            .bind(notice_id)
            .bind(notification_id)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(());
        }
        let mut mem = self.memory.write().await;
        if let Some(n) = mem.notices.get_mut(&notice_id) {
            n.delivery_status = "sent".into();
            n.notification_id = Some(notification_id);
            n.sent_at = now;
            n.updated_at = now;
        }
        Ok(())
    }

    pub async fn mark_retry_or_failed(
        &self,
        notice_id: i64,
        retry_count: i32,
        max_retries: i32,
        now: i64,
        backoff_ms: i64,
        err: &str,
    ) -> Result<(), String> {
        let failed = retry_count >= max_retries;
        let status = if failed { "failed" } else { "pending" };
        let scheduled = if failed { 0 } else { now + backoff_ms };
        if let Some(pool) = &self.pool {
            sqlx::query(
                "UPDATE fx_events.outbound_notices SET \
                    delivery_status = $2, retry_count = $3, scheduled_at = $4, updated_at = $5, \
                    payload = payload || $6::jsonb \
                 WHERE id = $1",
            )
            .bind(notice_id)
            .bind(status)
            .bind(retry_count)
            .bind(scheduled)
            .bind(now)
            .bind(
                serde_json::json!({
                    "last_delivery_error": err,
                    "last_delivery_at_ms": now,
                })
                .to_string(),
            )
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(());
        }
        let mut mem = self.memory.write().await;
        if let Some(n) = mem.notices.get_mut(&notice_id) {
            n.delivery_status = status.into();
            n.retry_count = retry_count;
            n.scheduled_at = scheduled;
            n.updated_at = now;
            if let Some(obj) = n.payload.as_object_mut() {
                obj.insert("last_delivery_error".into(), Value::String(err.into()));
                obj.insert("last_delivery_at_ms".into(), Value::from(now));
            }
        }
        Ok(())
    }

    pub async fn insert_notification(
        &self,
        end_user_id: i64,
        type_str: &str,
        payload: &Value,
        now: i64,
    ) -> Result<i64, String> {
        if let Some(pool) = &self.pool {
            let id: i64 = sqlx::query_scalar(
                "INSERT INTO fx_events.notifications ( \
                    end_user_id, type, payload, read_at, created_at, updated_at \
                 ) VALUES ($1, $2, $3::jsonb, 0, $4, 0) RETURNING id",
            )
            .bind(end_user_id)
            .bind(type_str)
            .bind(payload.to_string())
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(id);
        }
        let id = self.next_notif.fetch_add(1, Ordering::SeqCst);
        let view = NotificationView {
            id,
            end_user_id,
            r#type: type_str.into(),
            payload: payload.clone(),
            read_at: 0,
            created_at: now,
        };
        self.memory.write().await.notifications.insert(id, view);
        Ok(id)
    }

    pub async fn list_notifications(
        &self,
        end_user_id: i64,
        limit: i64,
    ) -> Result<Vec<NotificationView>, String> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query_as::<_, NotifRow>(
                "SELECT id, end_user_id, type, payload, read_at, created_at \
                 FROM fx_events.notifications \
                 WHERE end_user_id = $1 \
                 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(end_user_id)
            .bind(limit.max(1).min(200))
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(rows.into_iter().map(Into::into).collect());
        }
        let mem = self.memory.read().await;
        let mut v: Vec<_> = mem
            .notifications
            .values()
            .filter(|n| n.end_user_id == end_user_id)
            .cloned()
            .collect();
        v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        v.truncate(limit.max(1).min(200) as usize);
        Ok(v)
    }

    pub async fn mark_notification_read(
        &self,
        end_user_id: i64,
        id: i64,
        read: bool,
        now: i64,
    ) -> Result<Option<NotificationView>, String> {
        let read_at = if read { now } else { 0 };
        if let Some(pool) = &self.pool {
            let row = sqlx::query_as::<_, NotifRow>(
                "UPDATE fx_events.notifications SET read_at = $3, updated_at = $3 \
                 WHERE id = $1 AND end_user_id = $2 \
                 RETURNING id, end_user_id, type, payload, read_at, created_at",
            )
            .bind(id)
            .bind(end_user_id)
            .bind(read_at)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(row.map(Into::into));
        }
        let mut mem = self.memory.write().await;
        if let Some(n) = mem.notifications.get_mut(&id) {
            if n.end_user_id != end_user_id {
                return Ok(None);
            }
            n.read_at = read_at;
            return Ok(Some(n.clone()));
        }
        Ok(None)
    }

    pub async fn list_notices(
        &self,
        delivery_status: Option<&str>,
        limit: i64,
    ) -> Result<Vec<OutboundNotice>, String> {
        if let Some(pool) = &self.pool {
            let rows = if let Some(st) = delivery_status.filter(|s| !s.is_empty()) {
                sqlx::query_as::<_, NoticeRow>(
                    "SELECT id, end_user_id, source_type, source_id, event_type, title, body, \
                        payload, delivery_status, retry_count, scheduled_at, sent_at, notification_id, \
                        created_at, updated_at \
                     FROM fx_events.outbound_notices \
                     WHERE delivery_status = $1 \
                     ORDER BY created_at DESC LIMIT $2",
                )
                .bind(st)
                .bind(limit.max(1).min(500))
                .fetch_all(pool)
                .await
            } else {
                sqlx::query_as::<_, NoticeRow>(
                    "SELECT id, end_user_id, source_type, source_id, event_type, title, body, \
                        payload, delivery_status, retry_count, scheduled_at, sent_at, notification_id, \
                        created_at, updated_at \
                     FROM fx_events.outbound_notices \
                     ORDER BY created_at DESC LIMIT $1",
                )
                .bind(limit.max(1).min(500))
                .fetch_all(pool)
                .await
            }
            .map_err(|e| e.to_string())?;
            return Ok(rows.into_iter().map(Into::into).collect());
        }
        let mem = self.memory.read().await;
        let mut v: Vec<_> = mem
            .notices
            .values()
            .filter(|n| {
                delivery_status
                    .map(|s| n.delivery_status == s)
                    .unwrap_or(true)
            })
            .cloned()
            .collect();
        v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        v.truncate(limit.max(1).min(500) as usize);
        Ok(v)
    }

    pub async fn get_notice(&self, id: i64) -> Result<Option<OutboundNotice>, String> {
        if let Some(pool) = &self.pool {
            let row = sqlx::query_as::<_, NoticeRow>(
                "SELECT id, end_user_id, source_type, source_id, event_type, title, body, \
                    payload, delivery_status, retry_count, scheduled_at, sent_at, notification_id, \
                    created_at, updated_at \
                 FROM fx_events.outbound_notices WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(row.map(Into::into));
        }
        Ok(self.memory.read().await.notices.get(&id).cloned())
    }

    pub async fn stats(&self) -> Result<NoticeStats, String> {
        if let Some(pool) = &self.pool {
            let rows: Vec<(String, i64)> = sqlx::query_as(
                "SELECT delivery_status, COUNT(*)::bigint FROM fx_events.outbound_notices \
                 GROUP BY delivery_status",
            )
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            let mut s = NoticeStats {
                pending: 0,
                sending: 0,
                sent: 0,
                failed: 0,
                source: "postgres".into(),
            };
            for (st, c) in rows {
                match st.as_str() {
                    "pending" => s.pending = c,
                    "sending" => s.sending = c,
                    "sent" => s.sent = c,
                    "failed" => s.failed = c,
                    _ => {}
                }
            }
            return Ok(s);
        }
        let mem = self.memory.read().await;
        let mut s = NoticeStats {
            pending: 0,
            sending: 0,
            sent: 0,
            failed: 0,
            source: "memory".into(),
        };
        for n in mem.notices.values() {
            match n.delivery_status.as_str() {
                "pending" => s.pending += 1,
                "sending" => s.sending += 1,
                "sent" => s.sent += 1,
                "failed" => s.failed += 1,
                _ => {}
            }
        }
        Ok(s)
    }

    pub async fn resolve_webhook_endpoint(
        &self,
        corporate_user_id: i64,
        kind: &str,
    ) -> Option<String> {
        let pool = self.pool.as_ref()?;
        sqlx::query_scalar::<_, String>(
            "SELECT callback_url FROM fx_events.webhook_endpoints \
             WHERE corporate_user_id = $1 AND kind = $2 AND status = 'active' LIMIT 1",
        )
        .bind(corporate_user_id)
        .bind(kind)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
    }

    pub async fn log_delivery_event(
        &self,
        event_id: &str,
        event_type: &str,
        payload: &Value,
        delivery_status: &str,
        last_error: Option<&str>,
    ) {
        let Some(pool) = &self.pool else {
            return;
        };
        let now = now_ms();
        let _ = sqlx::query(
            "INSERT INTO fx_events.webhook_events ( \
                event_id, source, event_type, payload, delivery_status, retry_count, \
                last_error, next_retry_at, received_at, updated_at \
             ) VALUES ($1, 'message_center', $2, $3::jsonb, $4, 0, $5, 0, $6, 0) \
             ON CONFLICT (event_id) DO UPDATE SET \
                delivery_status = EXCLUDED.delivery_status, \
                last_error = EXCLUDED.last_error, \
                updated_at = EXCLUDED.received_at",
        )
        .bind(event_id)
        .bind(event_type)
        .bind(payload.to_string())
        .bind(delivery_status)
        .bind(last_error)
        .bind(now)
        .execute(pool)
        .await;
    }
}

#[derive(sqlx::FromRow)]
struct NoticeRow {
    id: i64,
    end_user_id: i64,
    source_type: String,
    source_id: i64,
    event_type: String,
    title: String,
    body: String,
    payload: sqlx::types::Json<Value>,
    delivery_status: String,
    retry_count: i32,
    scheduled_at: i64,
    sent_at: i64,
    notification_id: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

impl From<NoticeRow> for OutboundNotice {
    fn from(r: NoticeRow) -> Self {
        Self {
            id: r.id,
            end_user_id: r.end_user_id,
            source_type: r.source_type,
            source_id: r.source_id,
            event_type: r.event_type,
            title: r.title,
            body: r.body,
            payload: r.payload.0,
            delivery_status: r.delivery_status,
            retry_count: r.retry_count,
            scheduled_at: r.scheduled_at,
            sent_at: r.sent_at,
            notification_id: r.notification_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct NotifRow {
    id: i64,
    end_user_id: i64,
    #[sqlx(rename = "type")]
    notif_type: String,
    payload: sqlx::types::Json<Value>,
    read_at: i64,
    created_at: i64,
}

impl From<NotifRow> for NotificationView {
    fn from(r: NotifRow) -> Self {
        Self {
            id: r.id,
            end_user_id: r.end_user_id,
            r#type: r.notif_type,
            payload: r.payload.0,
            read_at: r.read_at,
            created_at: r.created_at,
        }
    }
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn memory_claim_ack() {
        let store = NoticeStore::new(None);
        let n = store
            .enqueue(
                1,
                "shop_order",
                100,
                "completed",
                "Paid",
                "Order paid",
                json!({}),
                0,
                None,
            )
            .await
            .unwrap();
        assert_eq!(n.delivery_status, "pending");
        let now = now_ms();
        let claimed = store.claim_pending(10, now).await.unwrap();
        assert_eq!(claimed.len(), 1);
        let nid = store
            .insert_notification(1, "shop_order.completed", &json!({"id": n.id}), now)
            .await
            .unwrap();
        store.mark_sent(n.id, nid, now).await.unwrap();
        let again = store.claim_pending(10, now).await.unwrap();
        assert!(again.is_empty());
        let stats = store.stats().await.unwrap();
        assert_eq!(stats.sent, 1);
    }
}
