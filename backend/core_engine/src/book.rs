//! Per-symbol order book with price-time priority.
//! Structure inspired by https://github.com/Jkrish1011/order-match-engine-rs

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Arc;

use crate::order::{Order, OrderSnapshot};
use crate::types::{OrderId, OrderType, Price, Quantity, Side, Trade};

#[derive(Debug, Default)]
struct PriceLevel {
    orders: VecDeque<Arc<Order>>,
    total_quantity: Quantity,
}

impl PriceLevel {
    fn push(&mut self, order: Arc<Order>) {
        self.total_quantity = self.total_quantity.saturating_add(order.remaining());
        self.orders.push_back(order);
    }

    fn pop_front_active(&mut self) -> Option<Arc<Order>> {
        while let Some(order) = self.orders.pop_front() {
            let rem = order.remaining();
            if rem == 0 || !order.is_active() {
                continue;
            }
            self.total_quantity = self.total_quantity.saturating_sub(rem);
            // Will re-add remaining after partial fill at caller if needed.
            return Some(order);
        }
        None
    }

    fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
}

#[derive(Debug)]
pub struct OrderBook {
    pub symbol: String,
    /// Bids: highest price first → reverse iteration of BTreeMap
    bids: BTreeMap<Price, PriceLevel>,
    /// Asks: lowest price first
    asks: BTreeMap<Price, PriceLevel>,
    orders: HashMap<OrderId, Arc<Order>>,
    trades: Vec<Trade>,
}

impl OrderBook {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::new(),
            trades: Vec::new(),
        }
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    pub fn get_order(&self, order_id: OrderId) -> Option<Arc<Order>> {
        self.orders.get(&order_id).cloned()
    }

    pub fn recent_trades(&self, limit: usize) -> Vec<Trade> {
        self.trades.iter().rev().take(limit).cloned().collect()
    }

    pub fn depth(&self, levels: usize) -> (Vec<(Price, Quantity)>, Vec<(Price, Quantity)>) {
        let bids: Vec<_> = self
            .bids
            .iter()
            .rev()
            .take(levels)
            .map(|(p, lvl)| (*p, lvl.total_quantity))
            .collect();
        let asks: Vec<_> = self
            .asks
            .iter()
            .take(levels)
            .map(|(p, lvl)| (*p, lvl.total_quantity))
            .collect();
        (bids, asks)
    }

    /// Match `incoming` against the book; return generated trades.
    pub fn place(&mut self, incoming: Arc<Order>) -> Vec<Trade> {
        self.orders.insert(incoming.order_id, Arc::clone(&incoming));
        let mut trades = Vec::new();

        match incoming.order_type {
            OrderType::Market => {
                self.match_market(&incoming, &mut trades);
            }
            OrderType::Price => {
                self.match_limit(&incoming, &mut trades);
                if incoming.is_active() && incoming.remaining() > 0 {
                    self.rest(Arc::clone(&incoming));
                }
            }
        }

        self.trades.extend(trades.iter().cloned());
        trades
    }

    pub fn cancel(&mut self, order_id: OrderId) -> Result<Arc<Order>, String> {
        let order = self
            .orders
            .get(&order_id)
            .cloned()
            .ok_or_else(|| format!("order {order_id} not found"))?;
        if !order.cancel() {
            return Err(format!("order {order_id} cannot be cancelled"));
        }
        self.remove_from_book(order_id, order.side, order.price);
        Ok(order)
    }

    fn match_market(&mut self, incoming: &Arc<Order>, trades: &mut Vec<Trade>) {
        while incoming.remaining() > 0 {
            let best_price = match incoming.side {
                Side::Buy => self.best_ask(),
                Side::Sell => self.best_bid(),
            };
            let Some(price) = best_price else { break };
            if !self.consume_best(incoming, price, trades) {
                break;
            }
        }
        // Unfilled market remainder is rejected (not rested).
        if incoming.remaining() > 0 && incoming.is_active() {
            incoming.reject_remainder();
        }
    }

    fn match_limit(&mut self, incoming: &Arc<Order>, trades: &mut Vec<Trade>) {
        while incoming.remaining() > 0 {
            let crosses = match incoming.side {
                Side::Buy => self.best_ask().map(|ask| incoming.price >= ask).unwrap_or(false),
                Side::Sell => self
                    .best_bid()
                    .map(|bid| incoming.price <= bid)
                    .unwrap_or(false),
            };
            if !crosses {
                break;
            }
            let price = match incoming.side {
                Side::Buy => self.best_ask().unwrap(),
                Side::Sell => self.best_bid().unwrap(),
            };
            if !self.consume_best(incoming, price, trades) {
                break;
            }
        }
    }

    fn consume_best(
        &mut self,
        incoming: &Arc<Order>,
        price: Price,
        trades: &mut Vec<Trade>,
    ) -> bool {
        let level = match incoming.side {
            Side::Buy => self.asks.get_mut(&price),
            Side::Sell => self.bids.get_mut(&price),
        };
        let Some(level) = level else {
            return false;
        };
        let Some(maker) = level.pop_front_active() else {
            // Clean empty level
            match incoming.side {
                Side::Buy => {
                    if self.asks.get(&price).map(|l| l.is_empty()).unwrap_or(true) {
                        self.asks.remove(&price);
                    }
                }
                Side::Sell => {
                    if self.bids.get(&price).map(|l| l.is_empty()).unwrap_or(true) {
                        self.bids.remove(&price);
                    }
                }
            }
            return true;
        };

        let trade_qty = incoming.remaining().min(maker.remaining());
        let filled_taker = incoming.fill(trade_qty);
        let filled_maker = maker.fill(filled_taker);
        let qty = filled_taker.min(filled_maker);

        if maker.is_active() && maker.remaining() > 0 {
            // Put remainder back at front (time priority preserved)
            level.total_quantity = level.total_quantity.saturating_add(maker.remaining());
            level.orders.push_front(maker.clone());
        }

        if level.is_empty() {
            match incoming.side {
                Side::Buy => {
                    self.asks.remove(&price);
                }
                Side::Sell => {
                    self.bids.remove(&price);
                }
            }
        }

        trades.push(Trade {
            trade_id: format!("trd_{}", uuid::Uuid::new_v4().simple()),
            symbol: self.symbol.clone(),
            price,
            quantity: qty,
            taker_order_id: incoming.order_id,
            maker_order_id: maker.order_id,
            taker_side: incoming.side,
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
        });
        true
    }

    fn rest(&mut self, order: Arc<Order>) {
        let price = order.price;
        match order.side {
            Side::Buy => self.bids.entry(price).or_default().push(order),
            Side::Sell => self.asks.entry(price).or_default().push(order),
        }
    }

    fn remove_from_book(&mut self, order_id: OrderId, side: Side, price: Price) {
        let map = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        if let Some(level) = map.get_mut(&price) {
            let before = level.orders.len();
            level.orders.retain(|o| {
                if o.order_id == order_id {
                    level.total_quantity = level
                        .total_quantity
                        .saturating_sub(o.remaining());
                    false
                } else {
                    true
                }
            });
            if level.orders.len() != before && level.is_empty() {
                map.remove(&price);
            }
        }
    }

    /// Export resting book + recent trades for local persistence.
    pub fn to_persist(&self) -> BookPersist {
        let mut bid_levels = Vec::new();
        for (price, level) in &self.bids {
            let orders: Vec<_> = level
                .orders
                .iter()
                .filter(|o| o.is_active())
                .map(|o| o.to_snapshot())
                .collect();
            if !orders.is_empty() {
                bid_levels.push(LevelPersist {
                    price: *price,
                    orders,
                });
            }
        }
        let mut ask_levels = Vec::new();
        for (price, level) in &self.asks {
            let orders: Vec<_> = level
                .orders
                .iter()
                .filter(|o| o.is_active())
                .map(|o| o.to_snapshot())
                .collect();
            if !orders.is_empty() {
                ask_levels.push(LevelPersist {
                    price: *price,
                    orders,
                });
            }
        }
        BookPersist {
            symbol: self.symbol.clone(),
            bid_levels,
            ask_levels,
            trades: self.trades.clone(),
        }
    }

    pub fn from_persist(snap: BookPersist) -> Self {
        let mut book = Self::new(snap.symbol);
        for level in snap.bid_levels {
            for order_snap in level.orders {
                let order = Order::from_snapshot(order_snap);
                book.orders.insert(order.order_id, Arc::clone(&order));
                book.bids.entry(level.price).or_default().push(order);
            }
        }
        for level in snap.ask_levels {
            for order_snap in level.orders {
                let order = Order::from_snapshot(order_snap);
                book.orders.insert(order.order_id, Arc::clone(&order));
                book.asks.entry(level.price).or_default().push(order);
            }
        }
        book.trades = snap.trades;
        book
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LevelPersist {
    pub price: Price,
    pub orders: Vec<OrderSnapshot>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BookPersist {
    pub symbol: String,
    pub bid_levels: Vec<LevelPersist>,
    pub ask_levels: Vec<LevelPersist>,
    pub trades: Vec<Trade>,
}
