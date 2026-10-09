//! Core domain types aligned with FunnyX Market (01) / Price (02) orders.
//! Inspired by https://github.com/Jkrish1011/order-match-engine-rs (price-time book model).

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type OrderId = u64;
pub type Price = u64;
pub type Quantity = u64;

/// Max trading pairs per Core Engine instance (quote-asset shard).
pub const MAX_PAIRS_PER_ENGINE: usize = 10;

/// 01 = Market Order, 02 = Price Order (`doc/gateway_core_engine_logic.md`).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market = 1,
    Price = 2,
}

impl OrderType {
    #[allow(dead_code)]
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    #[allow(dead_code)]
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Market),
            2 => Some(Self::Price),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Buy,
    Sell,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Active = 0,
    PartiallyFilled = 1,
    Filled = 2,
    Cancelled = 3,
    Rejected = 4,
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("pair limit reached: max {MAX_PAIRS_PER_ENGINE} pairs for quote {quote}")]
    PairLimit { quote: String },
    #[error("pair quote mismatch: engine quote is {engine_quote}, pair quote is {pair_quote}")]
    QuoteMismatch {
        engine_quote: String,
        pair_quote: String,
    },
    #[error("pair already listed: {0}")]
    PairExists(String),
    #[error("pair not listed: {0}")]
    PairNotFound(String),
    #[error("market pool not found for pair: {0}")]
    PoolNotFound(String),
    #[error("order not found: {0}")]
    OrderNotFound(OrderId),
    #[error("invalid order: {0}")]
    InvalidOrder(String),
    #[error("invalid pool: {0}")]
    InvalidPool(String),
    #[error("engine not accepting orders: {0}")]
    Maintenance(String),
    #[error("persist error: {0}")]
    Persist(String),
    #[error("order cannot be cancelled: {0}")]
    CancelRejected(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceStatus {
    /// Accepting orders and admin mutations.
    Running,
    /// Persisting / rejecting new orders; cancels still allowed.
    Draining,
    /// Snapshot on disk; memory may be empty or frozen; no trading until start.
    Stopped,
}

/// In-engine market pool (mirrors `fx_market.market_pools` + pool wallets).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPool {
    pub pool_id: String,
    pub symbol: String,
    pub market_id: String,
    pub pool_depth: f64,
    pub initial_price: f64,
    pub base_amount: f64,
    pub quote_amount: f64,
    pub base_wallet_balance: f64,
    pub quote_wallet_balance: f64,
    pub funding_source: String,
    pub status: PoolStatus,
    pub approved_by_admin_id: Option<String>,
    pub corporate_user_id: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PoolStatus {
    Pending,
    Active,
    Suspended,
}

/// Admin create-pair + init-pool request body fields (engine layer).
#[derive(Debug, Clone)]
pub struct AdminCreatePairInput {
    pub symbol: String,
    pub market_id: String,
    pub tick_size: Price,
    pub lot_size: Quantity,
    pub pool_depth: f64,
    pub initial_price: f64,
    pub base_amount: f64,
    pub quote_amount: f64,
    pub funding_source: String,
    pub admin_id: String,
    pub corporate_user_id: Option<i64>,
    pub seed_book: bool,
}


/// Trading pair symbol, e.g. `BTC/USDT` (base/quote).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradingPair {
    pub symbol: String,
    pub base: String,
    pub quote: String,
    pub market_id: String,
    pub tick_size: Price,
    pub lot_size: Quantity,
    pub status: PairStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairStatus {
    Listed,
    Suspended,
}

impl TradingPair {
    pub fn parse(symbol: &str, market_id: impl Into<String>) -> Result<Self, EngineError> {
        let parts: Vec<_> = symbol.split('/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(EngineError::InvalidOrder(format!(
                "symbol must be BASE/QUOTE, got '{symbol}'"
            )));
        }
        Ok(Self {
            symbol: symbol.to_ascii_uppercase(),
            base: parts[0].to_ascii_uppercase(),
            quote: parts[1].to_ascii_uppercase(),
            market_id: market_id.into(),
            tick_size: 1,
            lot_size: 1,
            status: PairStatus::Listed,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub symbol: String,
    pub price: Price,
    pub quantity: Quantity,
    pub taker_order_id: OrderId,
    pub maker_order_id: OrderId,
    pub taker_side: Side,
    pub timestamp_ms: i64,
}
