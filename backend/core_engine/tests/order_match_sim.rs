//! Order-match simulator for UAT/PLT.
//!
//! Each user starts with **100 PLT** and **1000 UAT** (Testing Game Token).
//! Pair symbol: `UAT/PLT` (base = UAT, quote = PLT). Price ticks = PLT per 1 UAT.
//!
//! Quiet (default) — start + end balances only:
//! `cargo test -p funnyx-core-engine --test order_match_sim`
//!
//! Detail steps:
//! `cargo test -p funnyx-core-engine --test order_match_sim -- --detail`

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use funnyx_core_engine::book::OrderBook;
use funnyx_core_engine::order::Order;
use funnyx_core_engine::types::{OrderId, OrderStatus, OrderType, Price, Quantity, Side, Trade};

/// Serialize multi-line log blocks.
static LOG: Mutex<()> = Mutex::new(());
static DETAIL: AtomicBool = AtomicBool::new(false);

fn detail_enabled() -> bool {
    DETAIL.load(Ordering::Relaxed)
}

fn set_detail(on: bool) {
    DETAIL.store(on, Ordering::Relaxed);
}

macro_rules! detail_println {
    ($($arg:tt)*) => {{
        if detail_enabled() {
            println!($($arg)*);
        }
    }};
}

const START_PLT: u64 = 100;
const START_UAT: u64 = 1000;
const SYMBOL: &str = "UAT/PLT";

#[derive(Debug, Clone)]
struct Wallet {
    plt: u64,
    uat: u64,
    locked_plt: u64,
    locked_uat: u64,
}

impl Wallet {
    fn fresh() -> Self {
        Self {
            plt: START_PLT,
            uat: START_UAT,
            locked_plt: 0,
            locked_uat: 0,
        }
    }

    fn available_plt(&self) -> u64 {
        self.plt.saturating_sub(self.locked_plt)
    }

    fn available_uat(&self) -> u64 {
        self.uat.saturating_sub(self.locked_uat)
    }
}

#[derive(Debug, Clone)]
struct RestingMeta {
    account: String,
    side: Side,
    limit_price: Price,
}

#[derive(Debug, Clone)]
struct PlaceResult {
    order_id: OrderId,
    status: OrderStatus,
    remaining: Quantity,
    trade_count: usize,
}

struct MatchSim {
    book: OrderBook,
    wallets: HashMap<String, Wallet>,
    resting: HashMap<OrderId, RestingMeta>,
    next_id: OrderId,
    step: usize,
}

impl MatchSim {
    fn new(users: &[&str]) -> Self {
        let mut wallets = HashMap::new();
        for user in users {
            wallets.insert((*user).to_string(), Wallet::fresh());
        }
        Self {
            book: OrderBook::new(SYMBOL),
            wallets,
            resting: HashMap::new(),
            next_id: 1,
            step: 0,
        }
    }

    fn print_table(title: &str, headers: &[&str], rows: &[Vec<String>]) {
        let widths: Vec<usize> = headers
            .iter()
            .enumerate()
            .map(|(i, h)| {
                rows.iter()
                    .map(|r| r.get(i).map(|c| c.len()).unwrap_or(0))
                    .max()
                    .unwrap_or(0)
                    .max(h.len())
            })
            .collect();
        let rule = |ch: char| {
            let mut s = String::from("+");
            for w in &widths {
                s.push_str(&ch.to_string().repeat(w + 2));
                s.push('+');
            }
            s
        };
        let fmt_row = |cols: &[&str]| {
            let mut s = String::from("|");
            for (i, col) in cols.iter().enumerate() {
                s.push_str(&format!(" {:width$} |", col, width = widths[i]));
            }
            s
        };
        println!("{title}");
        println!("{}", rule('-'));
        println!("{}", fmt_row(headers));
        println!("{}", rule('='));
        for row in rows {
            let refs: Vec<&str> = row.iter().map(|c| c.as_str()).collect();
            println!("{}", fmt_row(&refs));
        }
        println!("{}", rule('-'));
    }

    fn wallet_row(name: &str, w: &Wallet) -> Vec<String> {
        vec![
            name.to_string(),
            w.plt.to_string(),
            w.available_plt().to_string(),
            w.locked_plt.to_string(),
            w.uat.to_string(),
            w.available_uat().to_string(),
            w.locked_uat.to_string(),
        ]
    }

    fn log_header(&self, title: &str) {
        let _guard = LOG.lock().unwrap_or_else(|e| e.into_inner());
        println!("\n========== {title} ==========");
        self.print_holdings_unlocked("Starting balances");
    }

    fn print_holdings(&self, label: &str) {
        let _guard = LOG.lock().unwrap_or_else(|e| e.into_inner());
        self.print_holdings_unlocked(label);
    }

    fn print_holdings_unlocked(&self, label: &str) {
        let headers = [
            "User",
            "PLT",
            "PLT avail",
            "PLT locked",
            "UAT",
            "UAT avail",
            "UAT locked",
        ];
        let mut names: Vec<_> = self.wallets.keys().cloned().collect();
        names.sort();
        let mut rows: Vec<Vec<String>> = names
            .iter()
            .map(|name| Self::wallet_row(name, &self.wallets[name]))
            .collect();
        let (sum_plt, sum_uat) = self.totals();
        rows.push(vec![
            "TOTAL".into(),
            sum_plt.to_string(),
            String::new(),
            String::new(),
            sum_uat.to_string(),
            String::new(),
            String::new(),
        ]);
        Self::print_table(&format!("--- {label} ---"), &headers, &rows);
    }

    fn print_user(&self, account: &str) {
        if !detail_enabled() {
            return;
        }
        if let Some(w) = self.wallets.get(account) {
            Self::print_table(
                "--- wallet after lock ---",
                &[
                    "User",
                    "PLT",
                    "PLT avail",
                    "PLT locked",
                    "UAT",
                    "UAT avail",
                    "UAT locked",
                ],
                &[Self::wallet_row(account, w)],
            );
        }
    }

    fn totals(&self) -> (u64, u64) {
        self.wallets.values().fold((0, 0), |(p, u), w| (p + w.plt, u + w.uat))
    }

    fn place(
        &mut self,
        account: &str,
        side: Side,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
    ) -> Result<PlaceResult, String> {
        let _guard = LOG.lock().unwrap_or_else(|e| e.into_inner());

        self.step += 1;
        let order_id = self.next_id;
        self.next_id += 1;

        let kind = match order_type {
            OrderType::Market => "MARKET",
            OrderType::Price => "PRICE/LIMIT",
        };
        let display_price = if matches!(order_type, OrderType::Market) {
            0
        } else {
            price
        };
        if detail_enabled() {
            Self::print_table(
                &format!("\n--- step {} place ---", self.step),
                &["Step", "User", "Type", "Side", "Qty", "Price", "OrderId"],
                &[vec![
                    self.step.to_string(),
                    account.to_string(),
                    kind.to_string(),
                    format!("{side:?}"),
                    quantity.to_string(),
                    display_price.to_string(),
                    order_id.to_string(),
                ]],
            );
        }

        match (order_type, side) {
            (OrderType::Price, Side::Buy) => {
                let cost = price
                    .checked_mul(quantity)
                    .ok_or_else(|| "buy cost overflow".to_string())?;
                let w = self.wallets.get_mut(account).ok_or("unknown user")?;
                if w.available_plt() < cost {
                    return Err(format!(
                        "{account} cannot lock {cost} PLT (avail {})",
                        w.available_plt()
                    ));
                }
                w.locked_plt += cost;
                detail_println!("  lock {cost} PLT on {account}");
            }
            (OrderType::Price, Side::Sell) => {
                let w = self.wallets.get_mut(account).ok_or("unknown user")?;
                if w.available_uat() < quantity {
                    return Err(format!(
                        "{account} cannot lock {quantity} UAT (avail {})",
                        w.available_uat()
                    ));
                }
                w.locked_uat += quantity;
                detail_println!("  lock {quantity} UAT on {account}");
            }
            (OrderType::Market, Side::Buy) => {
                let avail = self.wallets[account].available_plt();
                detail_println!("  market buy — {account} has {avail} PLT available to spend");
            }
            (OrderType::Market, Side::Sell) => {
                let w = self.wallets.get_mut(account).ok_or("unknown user")?;
                if w.available_uat() < quantity {
                    return Err(format!(
                        "{account} cannot sell {quantity} UAT (avail {})",
                        w.available_uat()
                    ));
                }
                w.locked_uat += quantity;
                detail_println!("  lock {quantity} UAT on {account} for market sell");
            }
        }
        self.print_user(account);

        let order = Arc::new(Order::new(
            order_id,
            SYMBOL.to_string(),
            side,
            order_type,
            if matches!(order_type, OrderType::Market) {
                0
            } else {
                price
            },
            quantity,
            account.to_string(),
            None,
            None,
        ));

        self.resting.insert(
            order_id,
            RestingMeta {
                account: account.to_string(),
                side,
                limit_price: price,
            },
        );

        let trades = self.book.place(Arc::clone(&order));
        let mut match_rows: Vec<Vec<String>> = Vec::new();
        if trades.is_empty() {
            detail_println!("  no match");
        } else {
            for (i, trade) in trades.iter().enumerate() {
                let row = self.apply_trade(trade, i + 1)?;
                if let Some(r) = row {
                    match_rows.push(r);
                }
            }
        }
        if detail_enabled() && !match_rows.is_empty() {
            Self::print_table(
                "--- matches ---",
                &[
                    "#",
                    "Qty",
                    "Price",
                    "Cost PLT",
                    "Buyer",
                    "Seller",
                    "TakerOid",
                    "MakerOid",
                ],
                &match_rows,
            );
        }

        // Release unused locks for finished / rejected / cancelled-style remainders.
        self.release_unused_lock(&order)?;

        if detail_enabled() {
            Self::print_table(
                "--- order result ---",
                &["OrderId", "Status", "Remaining", "OnBook"],
                &[vec![
                    order_id.to_string(),
                    format!("{:?}", order.status()),
                    order.remaining().to_string(),
                    if order.is_active() && order.remaining() > 0 {
                        "yes".into()
                    } else {
                        "no".into()
                    },
                ]],
            );
        }
        if order.is_active() && order.remaining() > 0 {
            // keep resting meta
        } else {
            self.resting.remove(&order_id);
        }
        if detail_enabled() {
            if let Some(w) = self.wallets.get(account) {
                Self::print_table(
                    "--- wallet after settle ---",
                    &[
                        "User",
                        "PLT",
                        "PLT avail",
                        "PLT locked",
                        "UAT",
                        "UAT avail",
                        "UAT locked",
                    ],
                    &[Self::wallet_row(account, w)],
                );
            }
        }

        Ok(PlaceResult {
            order_id,
            status: order.status(),
            remaining: order.remaining(),
            trade_count: trades.len(),
        })
    }

    fn order_view(&self, order_id: OrderId) -> Option<(OrderStatus, Quantity)> {
        self.book
            .get_order(order_id)
            .map(|o| (o.status(), o.remaining()))
    }

    fn apply_trade(&mut self, trade: &Trade, n: usize) -> Result<Option<Vec<String>>, String> {
        let taker = self
            .resting
            .get(&trade.taker_order_id)
            .cloned()
            .ok_or_else(|| format!("missing taker meta {}", trade.taker_order_id))?;
        let maker = self
            .resting
            .get(&trade.maker_order_id)
            .cloned()
            .ok_or_else(|| format!("missing maker meta {}", trade.maker_order_id))?;

        let (buyer, seller) = match trade.taker_side {
            Side::Buy => (taker.account.clone(), maker.account.clone()),
            Side::Sell => (maker.account.clone(), taker.account.clone()),
        };
        let cost = trade
            .price
            .checked_mul(trade.quantity)
            .ok_or_else(|| "trade cost overflow".to_string())?;

        // Unlock seller base.
        {
            let w = self.wallets.get_mut(&seller).ok_or("seller missing")?;
            if w.locked_uat < trade.quantity {
                return Err(format!(
                    "seller {seller} locked_uat {} < {}",
                    w.locked_uat, trade.quantity
                ));
            }
            w.locked_uat -= trade.quantity;
            w.uat = w
                .uat
                .checked_sub(trade.quantity)
                .ok_or("seller UAT underflow")?;
            w.plt = w.plt.checked_add(cost).ok_or("seller PLT overflow")?;
        }

        // Buyer pays quote; unlock limit reserve when the buy order was a price order.
        {
            let buyer_meta = if trade.taker_side == Side::Buy {
                &taker
            } else {
                &maker
            };
            let w = self.wallets.get_mut(&buyer).ok_or("buyer missing")?;
            if buyer_meta.limit_price > 0 {
                let reserved = buyer_meta
                    .limit_price
                    .checked_mul(trade.quantity)
                    .ok_or("buy unlock overflow")?;
                if w.locked_plt < reserved {
                    return Err(format!(
                        "buyer {buyer} locked_plt {} < reserved {reserved}",
                        w.locked_plt
                    ));
                }
                w.locked_plt -= reserved;
            }
            if w.plt < cost {
                return Err(format!("buyer {buyer} PLT {} < cost {cost}", w.plt));
            }
            w.plt -= cost;
            w.uat = w.uat.checked_add(trade.quantity).ok_or("buyer UAT overflow")?;
        }

        Ok(Some(vec![
            n.to_string(),
            trade.quantity.to_string(),
            trade.price.to_string(),
            cost.to_string(),
            buyer,
            seller,
            trade.taker_order_id.to_string(),
            trade.maker_order_id.to_string(),
        ]))
    }

    fn release_unused_lock(&mut self, order: &Order) -> Result<(), String> {
        let rem = order.remaining();
        let meta = match self.resting.get(&order.order_id) {
            Some(m) => m.clone(),
            None => return Ok(()),
        };
        // Still resting with remainder — keep locks.
        if order.is_active() && rem > 0 {
            return Ok(());
        }
        // Finished or rejected remainder: free leftover locks.
        let w = self.wallets.get_mut(&meta.account).ok_or("user missing")?;
        match meta.side {
            Side::Buy if meta.limit_price > 0 && rem > 0 => {
                let unlock = meta.limit_price.saturating_mul(rem);
                let unlock = unlock.min(w.locked_plt);
                w.locked_plt -= unlock;
                detail_println!(
                    "  unlock leftover {unlock} PLT on {} (status={:?})",
                    meta.account,
                    order.status()
                );
            }
            Side::Sell if rem > 0 => {
                let unlock = rem.min(w.locked_uat);
                w.locked_uat -= unlock;
                detail_println!(
                    "  unlock leftover {unlock} UAT on {} (status={:?})",
                    meta.account,
                    order.status()
                );
            }
            _ => {}
        }
        if matches!(
            order.status(),
            OrderStatus::Filled | OrderStatus::Rejected | OrderStatus::Cancelled
        ) {
            // Ensure no stale lock for fully done orders with rem==0.
            if rem == 0 && matches!(meta.side, Side::Buy) && meta.limit_price > 0 {
                // nothing
            }
        }
        Ok(())
    }

    fn assert_conserved(&self, start_users: usize) {
        let (plt, uat) = self.totals();
        assert_eq!(plt, START_PLT * start_users as u64, "PLT must be conserved");
        assert_eq!(uat, START_UAT * start_users as u64, "UAT must be conserved");
        for (name, w) in &self.wallets {
            assert!(w.locked_plt <= w.plt, "{name} locked_plt > plt");
            assert!(w.locked_uat <= w.uat, "{name} locked_uat > uat");
        }
    }
}

fn two_users_limit_then_limit_match() {
    let mut sim = MatchSim::new(&["Alice", "Bob"]);
    sim.log_header("TWO USERS — Alice sells limit, Bob buys limit");

    sim.place("Alice", Side::Sell, OrderType::Price, 2, 40)
        .expect("Alice sell");
    sim.place("Bob", Side::Buy, OrderType::Price, 2, 40)
        .expect("Bob buy");

    sim.print_holdings("final holdings");
    sim.assert_conserved(2);

    assert_eq!(sim.wallets["Alice"].uat, START_UAT - 40);
    assert_eq!(sim.wallets["Alice"].plt, START_PLT + 80);
    assert_eq!(sim.wallets["Bob"].uat, START_UAT + 40);
    assert_eq!(sim.wallets["Bob"].plt, START_PLT - 80);
}

fn two_users_limit_then_market_buy() {
    let mut sim = MatchSim::new(&["Alice", "Bob"]);
    sim.log_header("TWO USERS — Alice sells limit, Bob market-buys");

    sim.place("Alice", Side::Sell, OrderType::Price, 2, 30)
        .expect("Alice sell");
    sim.place("Bob", Side::Buy, OrderType::Market, 0, 30)
        .expect("Bob market buy");

    sim.print_holdings("final holdings");
    sim.assert_conserved(2);

    assert_eq!(sim.wallets["Bob"].plt, START_PLT - 60);
    assert_eq!(sim.wallets["Bob"].uat, START_UAT + 30);
    assert_eq!(sim.wallets["Alice"].plt, START_PLT + 60);
    assert_eq!(sim.wallets["Alice"].uat, START_UAT - 30);
}

fn two_users_maker_partial_fill_rests() {
    let mut sim = MatchSim::new(&["Alice", "Bob"]);
    sim.log_header("TWO USERS — maker partial fill, remainder stays PartiallyFilled");

    let maker = sim
        .place("Alice", Side::Sell, OrderType::Price, 2, 50)
        .expect("Alice sell 50");
    assert_eq!(maker.status, OrderStatus::Active);
    assert_eq!(maker.remaining, 50);

    // Bob only takes 20 — Alice must stay PartiallyFilled with 30 resting.
    let taker = sim
        .place("Bob", Side::Buy, OrderType::Price, 2, 20)
        .expect("Bob buy 20");
    assert_eq!(taker.status, OrderStatus::Filled);
    assert_eq!(taker.remaining, 0);
    assert_eq!(taker.trade_count, 1);

    let (maker_status, maker_rem) = sim.order_view(maker.order_id).expect("maker still on book");
    assert_eq!(maker_status, OrderStatus::PartiallyFilled);
    assert_eq!(maker_rem, 30);
    assert_eq!(sim.wallets["Alice"].locked_uat, 30);
    assert_eq!(sim.wallets["Alice"].uat, START_UAT - 20);
    assert_eq!(sim.wallets["Alice"].plt, START_PLT + 40);
    assert_eq!(sim.wallets["Bob"].uat, START_UAT + 20);
    assert_eq!(sim.wallets["Bob"].plt, START_PLT - 40);
    assert_eq!(sim.book.best_ask(), Some(2));

    sim.print_holdings("final holdings — Alice still partially filled");
    sim.assert_conserved(2);
}

fn two_users_taker_partial_fill_rests() {
    let mut sim = MatchSim::new(&["Alice", "Bob"]);
    sim.log_header("TWO USERS — taker limit partial fill, remainder rests as bid");

    sim.place("Alice", Side::Sell, OrderType::Price, 2, 20)
        .expect("Alice sell 20");

    // Bob wants 50; only 20 available → fill 20, rest 30 as PartiallyFilled bid.
    let taker = sim
        .place("Bob", Side::Buy, OrderType::Price, 2, 50)
        .expect("Bob buy 50");
    assert_eq!(taker.status, OrderStatus::PartiallyFilled);
    assert_eq!(taker.remaining, 30);
    assert_eq!(taker.trade_count, 1);

    let (taker_status, taker_rem) = sim.order_view(taker.order_id).expect("taker resting");
    assert_eq!(taker_status, OrderStatus::PartiallyFilled);
    assert_eq!(taker_rem, 30);
    // Locked for the unfilled 30 @ 2.
    assert_eq!(sim.wallets["Bob"].locked_plt, 60);
    assert_eq!(sim.wallets["Bob"].plt, START_PLT - 40);
    assert_eq!(sim.wallets["Bob"].uat, START_UAT + 20);
    assert_eq!(sim.wallets["Alice"].uat, START_UAT - 20);
    assert_eq!(sim.wallets["Alice"].plt, START_PLT + 40);
    assert_eq!(sim.book.best_bid(), Some(2));
    assert!(sim.book.best_ask().is_none());

    sim.print_holdings("final holdings — Bob still partially filled");
    sim.assert_conserved(2);
}

fn two_users_market_partial_then_reject() {
    let mut sim = MatchSim::new(&["Alice", "Bob"]);
    sim.log_header("TWO USERS — market buys thin book, remainder Rejected");

    sim.place("Alice", Side::Sell, OrderType::Price, 2, 15)
        .expect("Alice sell 15");

    // Market wants 40; only 15 on book → fill 15, reject leftover 25.
    let taker = sim
        .place("Bob", Side::Buy, OrderType::Market, 0, 40)
        .expect("Bob market buy 40");
    assert_eq!(taker.status, OrderStatus::Rejected);
    assert_eq!(taker.remaining, 25);
    assert_eq!(taker.trade_count, 1);
    assert!(sim.order_view(taker.order_id).is_some());
    assert_eq!(sim.wallets["Bob"].uat, START_UAT + 15);
    assert_eq!(sim.wallets["Bob"].plt, START_PLT - 30);
    assert_eq!(sim.wallets["Alice"].uat, START_UAT - 15);
    assert_eq!(sim.wallets["Alice"].plt, START_PLT + 30);
    assert!(sim.book.best_ask().is_none());

    sim.print_holdings("final holdings — market remainder rejected");
    sim.assert_conserved(2);
}

fn three_users_partial_limit_and_market() {
    let mut sim = MatchSim::new(&["Alice", "Bob", "Carol"]);
    sim.log_header("THREE USERS — maker partial after Bob, then Carol completes");

    // Alice rests a sell of 50 UAT @ 2 PLT (100 PLT notional).
    let maker = sim
        .place("Alice", Side::Sell, OrderType::Price, 2, 50)
        .expect("Alice sell");
    // Bob takes 20 on a limit buy (40 PLT) → Alice PartiallyFilled.
    let bob = sim
        .place("Bob", Side::Buy, OrderType::Price, 2, 20)
        .expect("Bob buy");
    assert_eq!(bob.status, OrderStatus::Filled);
    let (mid_status, mid_rem) = sim.order_view(maker.order_id).expect("maker mid-state");
    assert_eq!(mid_status, OrderStatus::PartiallyFilled);
    assert_eq!(mid_rem, 30);
    assert_eq!(sim.wallets["Alice"].locked_uat, 30);

    // Carol market-buys the remaining 30 (60 PLT) → Alice Filled.
    let carol = sim
        .place("Carol", Side::Buy, OrderType::Market, 0, 30)
        .expect("Carol market buy");
    assert_eq!(carol.status, OrderStatus::Filled);
    let (end_status, end_rem) = sim.order_view(maker.order_id).expect("maker end-state");
    assert_eq!(end_status, OrderStatus::Filled);
    assert_eq!(end_rem, 0);

    sim.print_holdings("final holdings");
    sim.assert_conserved(3);

    assert_eq!(sim.wallets["Alice"].uat, START_UAT - 50);
    assert_eq!(sim.wallets["Alice"].plt, START_PLT + 100);
    assert_eq!(sim.wallets["Alice"].locked_uat, 0);
    assert_eq!(sim.wallets["Bob"].uat, START_UAT + 20);
    assert_eq!(sim.wallets["Bob"].plt, START_PLT - 40);
    assert_eq!(sim.wallets["Carol"].uat, START_UAT + 30);
    assert_eq!(sim.wallets["Carol"].plt, START_PLT - 60);
}

fn three_users_two_asks_price_time_priority() {
    let mut sim = MatchSim::new(&["Alice", "Bob", "Carol"]);
    sim.log_header("THREE USERS — better ask fills first, then same-price time priority");

    // Alice asks 40 @ 1 (better price).
    sim.place("Alice", Side::Sell, OrderType::Price, 1, 40)
        .expect("Alice sell @1");
    // Bob asks 40 @ 2.
    sim.place("Bob", Side::Sell, OrderType::Price, 2, 40)
        .expect("Bob sell @2");
    // Carol limit-buys 50 @ 2 (locks 100 PLT) → Alice 40 @1 then Bob 10 @2 = 60 spent, 40 unlocked.
    sim.place("Carol", Side::Buy, OrderType::Price, 2, 50)
        .expect("Carol buy");

    sim.print_holdings("final holdings");
    sim.assert_conserved(3);

    assert_eq!(sim.wallets["Alice"].uat, START_UAT - 40);
    assert_eq!(sim.wallets["Alice"].plt, START_PLT + 40); // 40 * 1
    assert_eq!(sim.wallets["Bob"].uat, START_UAT - 10);
    assert_eq!(sim.wallets["Bob"].plt, START_PLT + 20); // 10 * 2
    assert_eq!(sim.wallets["Carol"].uat, START_UAT + 50);
    assert_eq!(sim.wallets["Carol"].plt, START_PLT - 60); // 40 + 20
    // Bob still rests 30 UAT on the book.
    assert_eq!(sim.book.best_ask(), Some(2));
}

fn four_users_mixed_limit_and_market() {
    let mut sim = MatchSim::new(&["Alice", "Bob", "Carol", "Dave"]);
    sim.log_header("FOUR USERS — mixed price/limit book + market take");

    // Carol rests a bid below the asks so it does not cross.
    sim.place("Carol", Side::Buy, OrderType::Price, 1, 20)
        .expect("Carol bid");
    // Two sellers rest above Carol's bid.
    sim.place("Alice", Side::Sell, OrderType::Price, 2, 40)
        .expect("Alice sell");
    sim.place("Bob", Side::Sell, OrderType::Price, 3, 60)
        .expect("Bob sell");
    // Dave market-buys 50 → Alice 40 @2 + Bob 10 @3 = 80+30 = 110... too much.
    // Use 40: Alice fills fully @2 = 80 PLT.
    sim.place("Dave", Side::Buy, OrderType::Market, 0, 40)
        .expect("Dave market buy");

    sim.print_holdings("final holdings");
    sim.assert_conserved(4);

    assert_eq!(sim.wallets["Alice"].uat, START_UAT - 40);
    assert_eq!(sim.wallets["Alice"].plt, START_PLT + 80);
    assert_eq!(sim.wallets["Bob"].uat, START_UAT);
    assert_eq!(sim.wallets["Bob"].locked_uat, 60);
    // Carol still resting — PLT locked for 20 @ 1.
    assert_eq!(sim.wallets["Carol"].locked_plt, 20);
    assert_eq!(sim.wallets["Carol"].uat, START_UAT);
    assert_eq!(sim.wallets["Dave"].uat, START_UAT + 40);
    assert_eq!(sim.wallets["Dave"].plt, START_PLT - 80);
    assert_eq!(sim.book.best_ask(), Some(3));
    assert_eq!(sim.book.best_bid(), Some(1));
}

fn four_users_market_sell_hits_bids() {
    let mut sim = MatchSim::new(&["Alice", "Bob", "Carol", "Dave"]);
    sim.log_header("FOUR USERS — resting bids, market sell walks the book");

    sim.place("Alice", Side::Buy, OrderType::Price, 3, 10)
        .expect("Alice bid @3");
    sim.place("Bob", Side::Buy, OrderType::Price, 2, 15)
        .expect("Bob bid @2");
    sim.place("Carol", Side::Buy, OrderType::Price, 1, 20)
        .expect("Carol bid @1");
    // Dave market-sells 25 UAT → Alice 10 @3 + Bob 15 @2.
    sim.place("Dave", Side::Sell, OrderType::Market, 0, 25)
        .expect("Dave market sell");

    sim.print_holdings("final holdings");
    sim.assert_conserved(4);

    assert_eq!(sim.wallets["Alice"].uat, START_UAT + 10);
    assert_eq!(sim.wallets["Alice"].plt, START_PLT - 30);
    assert_eq!(sim.wallets["Bob"].uat, START_UAT + 15);
    assert_eq!(sim.wallets["Bob"].plt, START_PLT - 30);
    assert_eq!(sim.wallets["Carol"].locked_plt, 20);
    assert_eq!(sim.wallets["Dave"].uat, START_UAT - 25);
    assert_eq!(sim.wallets["Dave"].plt, START_PLT + 60); // 10*3 + 15*2
    assert_eq!(sim.book.best_bid(), Some(1));
}

fn print_usage() {
    println!(
        "Usage: cargo test -p funnyx-core-engine --test order_match_sim -- [--detail] [--help]\n\
         \n\
         Default: print start and end balances only.\n\
         --detail: print each place, lock, match, and wallet step."
    );
}

fn run_case(name: &str, f: fn()) -> bool {
    print!("test {name} ... ");
    let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_ok();
    if ok {
        println!("ok");
    } else {
        println!("FAILED");
    }
    ok
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return;
    }
    let detail = args.iter().any(|a| a == "--detail")
        || matches!(
            std::env::var("FX_MATCH_DETAIL").as_deref(),
            Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
        );
    set_detail(detail);
    if detail {
        println!("order_match_sim: detail steps ON");
    } else {
        println!("order_match_sim: quiet (start/end balances only). Pass --detail for steps.");
    }

    let cases: &[(&str, fn())] = &[
        ("two_users_limit_then_limit_match", two_users_limit_then_limit_match),
        ("two_users_limit_then_market_buy", two_users_limit_then_market_buy),
        ("two_users_maker_partial_fill_rests", two_users_maker_partial_fill_rests),
        ("two_users_taker_partial_fill_rests", two_users_taker_partial_fill_rests),
        ("two_users_market_partial_then_reject", two_users_market_partial_then_reject),
        ("three_users_partial_limit_and_market", three_users_partial_limit_and_market),
        ("three_users_two_asks_price_time_priority", three_users_two_asks_price_time_priority),
        ("four_users_mixed_limit_and_market", four_users_mixed_limit_and_market),
        ("four_users_market_sell_hits_bids", four_users_market_sell_hits_bids),
    ];

    let mut passed = 0usize;
    let total = cases.len();
    for (name, f) in cases {
        if run_case(name, *f) {
            passed += 1;
        }
    }
    println!("\ntest result: {passed} passed; {} failed; 0 ignored; 0 measured; 0 filtered out", total - passed);
    if passed != total {
        std::process::exit(1);
    }
}
