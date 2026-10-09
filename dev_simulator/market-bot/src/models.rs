use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Price,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineInfo {
    pub engine_id: String,
    #[allow(dead_code)]
    pub quote_asset: String,
    #[serde(default)]
    pub maintenance: String,
    #[serde(default)]
    pub pairs: Vec<ListedPair>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListedPair {
    pub symbol: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookSnapshot {
    #[allow(dead_code)]
    pub symbol: String,
    pub best_bid: Option<u64>,
    pub best_ask: Option<u64>,
    #[serde(default)]
    pub bids: Vec<(u64, u64)>,
    #[serde(default)]
    pub asks: Vec<(u64, u64)>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    #[serde(default)]
    pub price: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaceOrderRequest {
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<u64>,
    pub quantity: u64,
    pub account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_user_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaceResult {
    pub order: OrderView,
    #[serde(default)]
    pub trades: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderView {
    pub order_id: u64,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: u64,
    pub original_quantity: u64,
    pub remaining_quantity: u64,
    pub status: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    TakePartial,
    TakeFull,
    Make,
}

#[derive(Debug, Clone, Serialize)]
pub struct BotAction {
    pub timestamp_ms: i64,
    pub kind: ActionKind,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: u64,
    pub quantity: u64,
    pub fills: usize,
    pub order_status: String,
    pub remaining_quantity: u64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BotStatus {
    pub running: bool,
    pub paused: bool,
    pub core_engine_url: String,
    pub engine_id: Option<String>,
    pub last_error: Option<String>,
    pub ticks: u64,
    pub actions: u64,
    pub last_action: Option<BotAction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    pub error: String,
}
