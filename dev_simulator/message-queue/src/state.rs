use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;

use crate::models::{
    AckRequest, EnqueueNoticeRequest, ListQuery, NoticeCallbackPayload, NoticeView, StatsResponse,
};
use crate::sim_status::SimStatus;

#[derive(Debug, Clone)]
pub struct NoticeRecord {
    pub id: i64,
    pub end_user_id: i64,
    pub source_type: String,
    pub source_id: i64,
    pub event_type: String,
    pub title: String,
    pub body: String,
    pub payload: serde_json::Value,
    pub delivery_status: String,
    pub retry_count: i32,
    pub scheduled_at: i64,
    pub sent_at: i64,
    pub notification_id: Option<i64>,
    pub callback_url: Option<String>,
    pub callback_attempts: u32,
    pub last_callback_status: Option<u16>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl NoticeRecord {
    pub fn to_view(&self) -> NoticeView {
        NoticeView {
            id: self.id,
            end_user_id: self.end_user_id,
            source_type: self.source_type.clone(),
            source_id: self.source_id,
            event_type: self.event_type.clone(),
            title: self.title.clone(),
            body: self.body.clone(),
            payload: self.payload.clone(),
            delivery_status: self.delivery_status.clone(),
            sim_status: None,
            sim_delay_ms: None,
            callback_url: self.callback_url.clone(),
            retry_count: self.retry_count,
            scheduled_at: self.scheduled_at,
            sent_at: self.sent_at,
            notification_id: self.notification_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
            source: "test".into(),
        }
    }

    pub fn to_view_with_sim(&self, sim_status: &str, sim_delay_ms: u64) -> NoticeView {
        let mut view = self.to_view();
        view.sim_status = Some(sim_status.to_string());
        view.sim_delay_ms = Some(sim_delay_ms);
        view
    }
}

#[derive(Clone)]
pub struct AppState {
    pub default_callback_url: Option<String>,
    http: reqwest::Client,
    seq: Arc<AtomicI64>,
    notices: Arc<RwLock<HashMap<i64, NoticeRecord>>>,
}

impl AppState {
    pub fn new(default_callback_url: Option<String>) -> Self {
        Self {
            default_callback_url,
            http: reqwest::Client::new(),
            seq: Arc::new(AtomicI64::new(1)),
            notices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn now_ms() -> i64 {
        Utc::now().timestamp_millis()
    }

    /// Always creates with `delivery_status=pending`.
    pub async fn enqueue(&self, req: EnqueueNoticeRequest) -> NoticeRecord {
        let id = self.seq.fetch_add(1, Ordering::Relaxed);
        let now = Self::now_ms();
        let callback_url = req
            .callback_url
            .clone()
            .or_else(|| self.default_callback_url.clone());
        let record = NoticeRecord {
            id,
            end_user_id: req.end_user_id,
            source_type: req.source_type,
            source_id: req.source_id,
            event_type: req.event_type,
            title: req.title,
            body: req.body,
            payload: req.payload,
            delivery_status: "pending".into(),
            retry_count: 0,
            scheduled_at: req.scheduled_at,
            sent_at: 0,
            notification_id: None,
            callback_url,
            callback_attempts: 0,
            last_callback_status: None,
            created_at: now,
            updated_at: now,
        };
        self.notices.write().await.insert(id, record.clone());
        record
    }

    pub async fn get(&self, id: i64) -> Option<NoticeRecord> {
        self.notices.read().await.get(&id).cloned()
    }

    pub async fn list(&self, q: &ListQuery) -> Vec<NoticeView> {
        let guard = self.notices.read().await;
        let mut rows: Vec<_> = guard
            .values()
            .filter(|n| {
                q.delivery_status
                    .as_ref()
                    .map(|s| &n.delivery_status == s)
                    .unwrap_or(true)
                    && q.end_user_id.map(|u| n.end_user_id == u).unwrap_or(true)
                    && q.source_type
                        .as_ref()
                        .map(|s| &n.source_type == s)
                        .unwrap_or(true)
            })
            .map(NoticeRecord::to_view)
            .collect();
        rows.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        rows
    }

    pub async fn drain(&self, limit: usize) -> Vec<NoticeRecord> {
        let now = Self::now_ms();
        let mut guard = self.notices.write().await;
        let mut candidates: Vec<i64> = guard
            .values()
            .filter(|n| {
                n.delivery_status == "pending" && (n.scheduled_at == 0 || n.scheduled_at <= now)
            })
            .map(|n| n.id)
            .collect();
        candidates.sort_by_key(|id| {
            let n = &guard[id];
            (n.scheduled_at, n.created_at, n.id)
        });
        candidates.truncate(limit.max(1));

        let mut claimed = Vec::with_capacity(candidates.len());
        for id in candidates {
            if let Some(n) = guard.get_mut(&id) {
                n.delivery_status = "sending".into();
                n.updated_at = now;
                claimed.push(n.clone());
            }
        }
        claimed
    }

    pub async fn ack(&self, id: i64, req: AckRequest) -> anyhow::Result<NoticeRecord> {
        let mut guard = self.notices.write().await;
        let n = guard
            .get_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("notice not found"))?;
        if n.delivery_status != "sending" && n.delivery_status != "pending" {
            anyhow::bail!(
                "cannot ack notice in delivery_status '{}'",
                n.delivery_status
            );
        }
        let now = Self::now_ms();
        match req.delivery_status.as_str() {
            "sent" => {
                n.delivery_status = "sent".into();
                n.sent_at = now;
                n.notification_id = req.notification_id;
            }
            "failed" => {
                n.delivery_status = "failed".into();
                n.retry_count = n.retry_count.saturating_add(1);
            }
            other => anyhow::bail!("delivery_status must be sent or failed, got '{other}'"),
        }
        n.updated_at = now;
        Ok(n.clone())
    }

    pub async fn retry(&self, id: i64) -> anyhow::Result<NoticeRecord> {
        let mut guard = self.notices.write().await;
        let n = guard
            .get_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("notice not found"))?;
        if n.delivery_status != "failed" {
            anyhow::bail!("only failed notices can be retried");
        }
        n.delivery_status = "pending".into();
        n.updated_at = Self::now_ms();
        Ok(n.clone())
    }

    /// Apply scheduled sim outcome while still pending; optionally POST webhook.
    pub async fn apply_sim_status(
        &self,
        id: i64,
        sim: SimStatus,
        fire_callback: bool,
    ) -> anyhow::Result<NoticeRecord> {
        let delivery_status = sim.delivery_status();
        let mut guard = self.notices.write().await;
        let n = guard
            .get_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("notice not found"))?;
        if n.delivery_status != "pending" {
            anyhow::bail!(
                "notice already finalized as '{}'",
                n.delivery_status
            );
        }
        let now = Self::now_ms();
        n.delivery_status = delivery_status.into();
        if delivery_status == "sent" {
            n.sent_at = now;
        }
        if delivery_status == "failed" {
            n.retry_count = n.retry_count.saturating_add(1);
        }
        n.updated_at = now;
        let snapshot = n.clone();
        drop(guard);

        if fire_callback {
            if let Some(url) = snapshot.callback_url.clone() {
                let http_status = self
                    .post_callback(&snapshot, sim.as_str(), &url)
                    .await
                    .unwrap_or(0);
                let mut guard = self.notices.write().await;
                if let Some(n) = guard.get_mut(&id) {
                    n.callback_attempts = n.callback_attempts.saturating_add(1);
                    n.last_callback_status = Some(http_status);
                    n.updated_at = Self::now_ms();
                    return Ok(n.clone());
                }
            }
        }
        Ok(snapshot)
    }

    async fn post_callback(
        &self,
        record: &NoticeRecord,
        sim_status: &str,
        url: &str,
    ) -> anyhow::Result<u16> {
        let body = NoticeCallbackPayload {
            event_id: format!("evt_{}", uuid::Uuid::new_v4().simple()),
            notice_id: record.id,
            end_user_id: record.end_user_id,
            source_type: record.source_type.clone(),
            source_id: record.source_id,
            event_type: record.event_type.clone(),
            delivery_status: record.delivery_status.clone(),
            sim_status: sim_status.into(),
            title: record.title.clone(),
            body: record.body.clone(),
            payload: record.payload.clone(),
            source: "test".into(),
        };
        tracing::info!(%url, notice_id = record.id, sim_status, "posting notice webhook callback");
        let resp = self.http.post(url).json(&body).send().await?;
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        tracing::info!(%status, body = %text, "notice callback response");
        Ok(status)
    }

    pub async fn clear(&self) {
        self.notices.write().await.clear();
    }

    pub async fn stats(&self) -> StatsResponse {
        let guard = self.notices.read().await;
        let mut pending = 0;
        let mut sending = 0;
        let mut sent = 0;
        let mut failed = 0;
        for n in guard.values() {
            match n.delivery_status.as_str() {
                "pending" => pending += 1,
                "sending" => sending += 1,
                "sent" => sent += 1,
                "failed" => failed += 1,
                _ => {}
            }
        }
        StatsResponse {
            total: guard.len(),
            pending,
            sending,
            sent,
            failed,
        }
    }
}
