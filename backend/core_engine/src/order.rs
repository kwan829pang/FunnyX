use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::types::{OrderId, OrderStatus, OrderType, Price, Quantity, Side};

#[derive(Debug)]
pub struct Order {
    pub order_id: OrderId,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Price,
    pub original_quantity: Quantity,
    remaining_quantity: AtomicU64,
    status: AtomicU8,
    pub timestamp_ms: u64,
    pub account_id: String,
    pub end_user_id: Option<i64>,
    pub corporate_user_id: Option<i64>,
}

impl Order {
    pub fn new(
        order_id: OrderId,
        symbol: String,
        side: Side,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
        account_id: String,
        end_user_id: Option<i64>,
        corporate_user_id: Option<i64>,
    ) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self {
            order_id,
            symbol,
            side,
            order_type,
            price,
            original_quantity: quantity,
            remaining_quantity: AtomicU64::new(quantity),
            status: AtomicU8::new(OrderStatus::Active as u8),
            timestamp_ms,
            account_id,
            end_user_id,
            corporate_user_id,
        }
    }

    pub fn remaining(&self) -> Quantity {
        self.remaining_quantity.load(Ordering::Acquire)
    }

    pub fn status(&self) -> OrderStatus {
        match self.status.load(Ordering::Acquire) {
            0 => OrderStatus::Active,
            1 => OrderStatus::PartiallyFilled,
            2 => OrderStatus::Filled,
            3 => OrderStatus::Cancelled,
            _ => OrderStatus::Rejected,
        }
    }

    pub fn reject_remainder(&self) {
        if self.remaining() > 0 && self.is_active() {
            self.status
                .store(OrderStatus::Rejected as u8, Ordering::Release);
        }
    }

    pub fn fill(&self, qty: Quantity) -> Quantity {
        let mut current = self.remaining_quantity.load(Ordering::Acquire);
        loop {
            if current == 0 {
                return 0;
            }
            let fill_amount = current.min(qty);
            let new_remaining = current - fill_amount;
            match self.remaining_quantity.compare_exchange_weak(
                current,
                new_remaining,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    if new_remaining == 0 {
                        self.status
                            .store(OrderStatus::Filled as u8, Ordering::Release);
                    } else {
                        self.status
                            .store(OrderStatus::PartiallyFilled as u8, Ordering::Release);
                    }
                    return fill_amount;
                }
                Err(actual) => current = actual,
            }
        }
    }

    pub fn cancel(&self) -> bool {
        let mut current = self.status.load(Ordering::Acquire);
        loop {
            if current == OrderStatus::Filled as u8 || current == OrderStatus::Cancelled as u8 {
                return false;
            }
            match self.status.compare_exchange_weak(
                current,
                OrderStatus::Cancelled as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status(),
            OrderStatus::Active | OrderStatus::PartiallyFilled
        )
    }

    pub fn to_view(self: &Arc<Self>) -> OrderView {
        OrderView {
            order_id: self.order_id,
            symbol: self.symbol.clone(),
            side: self.side,
            order_type: self.order_type,
            price: self.price,
            original_quantity: self.original_quantity,
            remaining_quantity: self.remaining(),
            status: self.status(),
            timestamp_ms: self.timestamp_ms,
            account_id: self.account_id.clone(),
            end_user_id: self.end_user_id,
            corporate_user_id: self.corporate_user_id,
        }
    }

    pub fn to_snapshot(self: &Arc<Self>) -> OrderSnapshot {
        OrderSnapshot {
            order_id: self.order_id,
            symbol: self.symbol.clone(),
            side: self.side,
            order_type: self.order_type,
            price: self.price,
            original_quantity: self.original_quantity,
            remaining_quantity: self.remaining(),
            status: self.status(),
            timestamp_ms: self.timestamp_ms,
            account_id: self.account_id.clone(),
            end_user_id: self.end_user_id,
            corporate_user_id: self.corporate_user_id,
        }
    }

    pub fn from_snapshot(snap: OrderSnapshot) -> Arc<Self> {
        Arc::new(Self {
            order_id: snap.order_id,
            symbol: snap.symbol,
            side: snap.side,
            order_type: snap.order_type,
            price: snap.price,
            original_quantity: snap.original_quantity,
            remaining_quantity: AtomicU64::new(snap.remaining_quantity),
            status: AtomicU8::new(snap.status as u8),
            timestamp_ms: snap.timestamp_ms,
            account_id: snap.account_id,
            end_user_id: snap.end_user_id,
            corporate_user_id: snap.corporate_user_id,
        })
    }
}

/// Resting-order fields for local snapshot JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderSnapshot {
    pub order_id: OrderId,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Price,
    pub original_quantity: Quantity,
    pub remaining_quantity: Quantity,
    pub status: OrderStatus,
    pub timestamp_ms: u64,
    pub account_id: String,
    pub end_user_id: Option<i64>,
    pub corporate_user_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderView {
    pub order_id: OrderId,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Price,
    pub original_quantity: Quantity,
    pub remaining_quantity: Quantity,
    pub status: OrderStatus,
    pub timestamp_ms: u64,
    pub account_id: String,
    pub end_user_id: Option<i64>,
    pub corporate_user_id: Option<i64>,
}
