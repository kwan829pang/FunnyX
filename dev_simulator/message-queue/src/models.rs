use serde::{Deserialize, Serialize};
use serde_json::Value;

const SOURCE_TYPES: &[&str] = &[
    "marketplace_deal",
    "marketplace_deal_request",
    "order",
    "trade",
    "shop_order",
    "deposit_withdrawal_txn",
    "corp_token_order",
];

const EVENT_TYPES: &[&str] = &[
    "created",
    "pending_payment",
    "completed",
    "cancelled",
    "rejected",
    "filled",
    "partial_filled",
    "failed",
    "status_changed",
];

const DELIVERY_STATUSES: &[&str] = &["pending", "sending", "sent", "failed"];

pub fn valid_source_type(v: &str) -> bool {
    SOURCE_TYPES.contains(&v)
}

pub fn valid_event_type(v: &str) -> bool {
    EVENT_TYPES.contains(&v)
}

pub fn valid_delivery_status(v: &str) -> bool {
    DELIVERY_STATUSES.contains(&v)
}

/// Enqueue body aligned with `fx_events.outbound_notices`.
#[derive(Debug, Clone, Deserialize)]
pub struct EnqueueNoticeRequest {
    pub end_user_id: i64,
    pub source_type: String,
    pub source_id: i64,
    pub event_type: String,
    pub title: String,
    pub body: String,
    #[serde(default = "default_payload")]
    pub payload: Value,
    /// UTC ms; 0 means immediate.
    #[serde(default)]
    pub scheduled_at: i64,
    /// Optional webhook URL invoked when scheduled `X-Sim-Status` is applied.
    #[serde(default)]
    pub callback_url: Option<String>,
}

fn default_payload() -> Value {
    Value::Object(Default::default())
}

#[derive(Debug, Clone, Serialize)]
pub struct NoticeView {
    pub id: i64,
    pub end_user_id: i64,
    pub source_type: String,
    pub source_id: i64,
    pub event_type: String,
    pub title: String,
    pub body: String,
    pub payload: Value,
    pub delivery_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sim_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sim_delay_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    pub retry_count: i32,
    pub scheduled_at: i64,
    pub sent_at: i64,
    pub notification_id: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoticeCallbackPayload {
    pub event_id: String,
    pub notice_id: i64,
    pub end_user_id: i64,
    pub source_type: String,
    pub source_id: i64,
    pub event_type: String,
    pub delivery_status: String,
    pub sim_status: String,
    pub title: String,
    pub body: String,
    pub payload: Value,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DrainRequest {
    /// Max notices to claim (default 10).
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Clone, Deserialize)]
pub struct AckRequest {
    /// `sent` or `failed`.
    pub delivery_status: String,
    #[serde(default)]
    pub notification_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsResponse {
    pub total: usize,
    pub pending: usize,
    pub sending: usize,
    pub sent: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    pub error: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ListQuery {
    pub delivery_status: Option<String>,
    pub end_user_id: Option<i64>,
    pub source_type: Option<String>,
}
