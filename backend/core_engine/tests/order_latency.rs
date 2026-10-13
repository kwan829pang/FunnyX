//! Latency tests for place-order and complete-order (full match fill).
//!
//! Measures the Core Engine `OrderBook` path used by `place_order`:
//! - **place** — price/limit order rests on the book (no fill)
//! - **complete** — incoming order fully matches and fills against resting liquidity
//!
//! Run with printed stats:
//! `cargo test -p funnyx-core-engine --test order_latency -- --nocapture`

use std::sync::Arc;
use std::time::{Duration, Instant};

use funnyx_core_engine::book::OrderBook;
use funnyx_core_engine::order::Order;
use funnyx_core_engine::types::{OrderId, OrderStatus, OrderType, Side};

const SYMBOL: &str = "UAT/PLT";
const WARMUP: usize = 2_000;
const ITERS: usize = 20_000;

struct LatencyStats {
    label: String,
    samples_ns: Vec<u64>,
}

impl LatencyStats {
    fn new(label: impl Into<String>, capacity: usize) -> Self {
        Self {
            label: label.into(),
            samples_ns: Vec::with_capacity(capacity),
        }
    }

    fn record(&mut self, elapsed: Duration) {
        self.samples_ns.push(elapsed.as_nanos() as u64);
    }

    fn report(&mut self) {
        assert!(
            !self.samples_ns.is_empty(),
            "{} has no samples",
            self.label
        );
        self.samples_ns.sort_unstable();
        let n = self.samples_ns.len();
        let sum: u128 = self.samples_ns.iter().map(|&v| v as u128).sum();
        let avg = sum / n as u128;
        let min = self.samples_ns[0];
        let max = *self.samples_ns.last().unwrap();
        let p50 = percentile(&self.samples_ns, 50);
        let p95 = percentile(&self.samples_ns, 95);
        let p99 = percentile(&self.samples_ns, 99);

        println!("\n========== latency: {} ==========", self.label);
        println!("  samples : {n}");
        println!("  min     : {}", fmt_ns(min));
        println!("  avg     : {}", fmt_ns(avg as u64));
        println!("  p50     : {}", fmt_ns(p50));
        println!("  p95     : {}", fmt_ns(p95));
        println!("  p99     : {}", fmt_ns(p99));
        println!("  max     : {}", fmt_ns(max));
    }
}

fn percentile(sorted: &[u64], pct: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((pct as f64 / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn fmt_ns(ns: u64) -> String {
    if ns >= 1_000_000 {
        format!("{:.3} ms", ns as f64 / 1_000_000.0)
    } else if ns >= 1_000 {
        format!("{:.3} µs", ns as f64 / 1_000.0)
    } else {
        format!("{ns} ns")
    }
}

fn next_order(
    id: OrderId,
    side: Side,
    order_type: OrderType,
    price: u64,
    qty: u64,
    account: &str,
) -> Arc<Order> {
    Arc::new(Order::new(
        id,
        SYMBOL.to_string(),
        side,
        order_type,
        price,
        qty,
        account.to_string(),
        None,
        None,
    ))
}

/// Place a resting sell so the book always has liquidity for complete-order runs.
fn seed_ask(book: &mut OrderBook, id: &mut OrderId, price: u64, qty: u64) {
    let order = next_order(*id, Side::Sell, OrderType::Price, price, qty, "maker");
    *id += 1;
    let trades = book.place(order);
    assert!(trades.is_empty(), "seed ask must rest");
}

#[test]
fn latency_place_resting_limit_order() {
    let mut book = OrderBook::new(SYMBOL);
    let mut id: OrderId = 1;
    let mut stats = LatencyStats::new("place resting PRICE/LIMIT order", ITERS);

    for i in 0..(WARMUP + ITERS) {
        // Alternate sides/prices so levels stay shallow and comparable.
        let side = if i % 2 == 0 { Side::Sell } else { Side::Buy };
        let price = 10 + (i as u64 % 50);
        let order = next_order(id, side, OrderType::Price, price, 1, "placer");
        id += 1;

        let start = Instant::now();
        let trades = book.place(Arc::clone(&order));
        let elapsed = start.elapsed();

        // Cancel immediately so the book does not grow without bound.
        if order.is_active() && order.remaining() > 0 {
            let _ = book.cancel(order.order_id);
        }

        if i >= WARMUP {
            assert!(
                trades.is_empty() || order.status() == OrderStatus::Filled,
                "place sample unexpected heavy match"
            );
            stats.record(elapsed);
        }
    }

    stats.report();
    let p99 = percentile(&{
        let mut s = stats.samples_ns.clone();
        s.sort_unstable();
        s
    }, 99);
    // Soft sanity: matching a single resting place should stay well under 5 ms on a normal machine.
    assert!(
        p99 < 5_000_000,
        "place p99 too high: {}",
        fmt_ns(p99)
    );
}

#[test]
fn latency_complete_order_limit_fill() {
    let mut book = OrderBook::new(SYMBOL);
    let mut id: OrderId = 1;
    let mut stats = LatencyStats::new("complete order — PRICE/LIMIT full fill", ITERS);

    for i in 0..(WARMUP + ITERS) {
        seed_ask(&mut book, &mut id, 5, 10);
        let buy = next_order(id, Side::Buy, OrderType::Price, 5, 10, "taker");
        id += 1;

        let start = Instant::now();
        let trades = book.place(Arc::clone(&buy));
        let elapsed = start.elapsed();

        assert_eq!(trades.len(), 1, "expected one completing trade");
        assert_eq!(buy.status(), OrderStatus::Filled);
        assert_eq!(buy.remaining(), 0);
        assert_eq!(trades[0].quantity, 10);
        assert_eq!(trades[0].price, 5);

        if i >= WARMUP {
            stats.record(elapsed);
        }
    }

    stats.report();
    let mut sorted = stats.samples_ns.clone();
    sorted.sort_unstable();
    let p99 = percentile(&sorted, 99);
    assert!(
        p99 < 5_000_000,
        "complete(limit) p99 too high: {}",
        fmt_ns(p99)
    );
}

#[test]
fn latency_complete_order_market_fill() {
    let mut book = OrderBook::new(SYMBOL);
    let mut id: OrderId = 1;
    let mut stats = LatencyStats::new("complete order — MARKET full fill", ITERS);

    for i in 0..(WARMUP + ITERS) {
        seed_ask(&mut book, &mut id, 7, 8);
        let buy = next_order(id, Side::Buy, OrderType::Market, 0, 8, "taker");
        id += 1;

        let start = Instant::now();
        let trades = book.place(Arc::clone(&buy));
        let elapsed = start.elapsed();

        assert_eq!(trades.len(), 1, "expected one market fill");
        assert_eq!(buy.status(), OrderStatus::Filled);
        assert_eq!(trades[0].price, 7);
        assert_eq!(trades[0].quantity, 8);

        if i >= WARMUP {
            stats.record(elapsed);
        }
    }

    stats.report();
    let mut sorted = stats.samples_ns.clone();
    sorted.sort_unstable();
    let p99 = percentile(&sorted, 99);
    assert!(
        p99 < 5_000_000,
        "complete(market) p99 too high: {}",
        fmt_ns(p99)
    );
}

#[test]
fn latency_place_then_complete_round_trip() {
    let mut book = OrderBook::new(SYMBOL);
    let mut id: OrderId = 1;
    let mut place_stats = LatencyStats::new("round-trip place half", ITERS);
    let mut complete_stats = LatencyStats::new("round-trip complete half", ITERS);
    let mut total_stats = LatencyStats::new("round-trip place + complete", ITERS);

    for i in 0..(WARMUP + ITERS) {
        let sell = next_order(id, Side::Sell, OrderType::Price, 4, 5, "maker");
        id += 1;
        let buy = next_order(id, Side::Buy, OrderType::Price, 4, 5, "taker");
        id += 1;

        let total_start = Instant::now();

        let place_start = Instant::now();
        let place_trades = book.place(Arc::clone(&sell));
        let place_elapsed = place_start.elapsed();
        assert!(place_trades.is_empty());
        assert!(sell.is_active());

        let complete_start = Instant::now();
        let complete_trades = book.place(Arc::clone(&buy));
        let complete_elapsed = complete_start.elapsed();
        let total_elapsed = total_start.elapsed();

        assert_eq!(complete_trades.len(), 1);
        assert_eq!(buy.status(), OrderStatus::Filled);
        assert_eq!(sell.status(), OrderStatus::Filled);

        if i >= WARMUP {
            place_stats.record(place_elapsed);
            complete_stats.record(complete_elapsed);
            total_stats.record(total_elapsed);
        }
    }

    place_stats.report();
    complete_stats.report();
    total_stats.report();

    let mut sorted = total_stats.samples_ns.clone();
    sorted.sort_unstable();
    let p99 = percentile(&sorted, 99);
    assert!(
        p99 < 8_000_000,
        "round-trip p99 too high: {}",
        fmt_ns(p99)
    );
}

#[test]
fn latency_complete_order_walks_multiple_levels() {
    let mut book = OrderBook::new(SYMBOL);
    let mut id: OrderId = 1;
    let mut stats = LatencyStats::new("complete order — walk 5 ask levels", ITERS / 2);

    let iters = ITERS / 2;
    for i in 0..(WARMUP + iters) {
        for level in 0..5u64 {
            seed_ask(&mut book, &mut id, 10 + level, 2);
        }
        let buy = next_order(id, Side::Buy, OrderType::Market, 0, 10, "taker");
        id += 1;

        let start = Instant::now();
        let trades = book.place(Arc::clone(&buy));
        let elapsed = start.elapsed();

        assert_eq!(trades.len(), 5, "market should walk five levels");
        assert_eq!(buy.status(), OrderStatus::Filled);

        if i >= WARMUP {
            stats.record(elapsed);
        }
    }

    stats.report();
    let mut sorted = stats.samples_ns.clone();
    sorted.sort_unstable();
    let p99 = percentile(&sorted, 99);
    assert!(
        p99 < 8_000_000,
        "multi-level complete p99 too high: {}",
        fmt_ns(p99)
    );
}
