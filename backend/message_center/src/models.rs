use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
}

pub fn valid_source_type(v: &str) -> bool {
    matches!(
        v,
        "marketplace_deal"
            | "marketplace_deal_request"
            | "order"
            | "trade"
            | "shop_order"
            | "deposit_withdrawal_txn"
            | "corp_token_order"
    )
}

pub fn valid_event_type(v: &str) -> bool {
    matches!(
        v,
        "created"
            | "pending_payment"
            | "completed"
            | "cancelled"
            | "rejected"
            | "filled"
            | "partial_filled"
            | "failed"
            | "status_changed"
    )
}

/// Core Engine / internal enqueue body (aligned with message-queue sim).
#[derive(Debug, Clone, Deserialize)]
pub struct EnqueueNoticeRequest {
    pub end_user_id: i64,
    pub source_type: String,
    pub source_id: i64,
    pub event_type: String,
    pub title: String,
    pub body: String,
    #[serde(default = "empty_object")]
    pub payload: Value,
    #[serde(default)]
    pub scheduled_at: i64,
    #[serde(default)]
    pub callback_url: Option<String>,
}

fn empty_object() -> Value {
    Value::Object(Default::default())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundNotice {
    pub id: i64,
    pub end_user_id: i64,
    pub source_type: String,
    pub source_id: i64,
    pub event_type: String,
    pub title: String,
    pub body: String,
    pub payload: Value,
    pub delivery_status: String,
    pub retry_count: i32,
    pub scheduled_at: i64,
    pub sent_at: i64,
    pub notification_id: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NotificationView {
    pub id: i64,
    pub end_user_id: i64,
    pub r#type: String,
    pub payload: Value,
    pub read_at: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoticeStats {
    pub pending: i64,
    pub sending: i64,
    pub sent: i64,
    pub failed: i64,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct MarkReadBody {
    #[serde(default = "default_true")]
    pub read: bool,
}

fn default_true() -> bool {
    true
}
