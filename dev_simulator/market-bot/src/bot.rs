use std::collections::VecDeque;
use std::sync::Arc;

use chrono::Utc;
use rand::Rng;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::config::Config;
use crate::engine::EngineClient;
use crate::models::{
    ActionKind, BookSnapshot, BotAction, BotStatus, OrderType, PlaceOrderRequest, Side,
};

const ACTION_CAP: usize = 200;

#[derive(Clone)]
pub struct Bot {
    cfg: Config,
    engine: EngineClient,
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    paused: bool,
    running: bool,
    ticks: u64,
    actions: u64,
    last_error: Option<String>,
    engine_id: Option<String>,
    log: VecDeque<BotAction>,
}

impl Bot {
    pub fn new(cfg: Config, engine: EngineClient) -> Self {
        Self {
            cfg,
            engine,
            inner: Arc::new(Mutex::new(Inner {
                paused: false,
                running: true,
                ticks: 0,
                actions: 0,
                last_error: None,
                engine_id: None,
                log: VecDeque::new(),
            })),
        }
    }

    pub async fn set_paused(&self, paused: bool) {
        self.inner.lock().await.paused = paused;
    }

    pub async fn status(&self) -> BotStatus {
        let g = self.inner.lock().await;
        BotStatus {
            running: g.running,
            paused: g.paused,
            core_engine_url: self.cfg.core_engine_url.clone(),
            engine_id: g.engine_id.clone(),
            last_error: g.last_error.clone(),
            ticks: g.ticks,
            actions: g.actions,
            last_action: g.log.back().cloned(),
        }
    }

    pub async fn recent_actions(&self, limit: usize) -> Vec<BotAction> {
        let g = self.inner.lock().await;
        g.log.iter().rev().take(limit.max(1)).cloned().collect()
    }

    pub fn spawn_loop(self) {
        tokio::spawn(async move {
            loop {
                let jitter = if self.cfg.jitter_ms == 0 {
                    0
                } else {
                    rand::thread_rng().gen_range(0..=self.cfg.jitter_ms)
                };
                sleep(Duration::from_millis(self.cfg.poll_interval_ms + jitter)).await;
                if let Err(e) = self.tick().await {
                    tracing::warn!(error = %e, "bot tick failed");
                    self.inner.lock().await.last_error = Some(e.to_string());
                }
            }
        });
    }

    async fn tick(&self) -> anyhow::Result<()> {
        {
            let mut g = self.inner.lock().await;
            if g.paused {
                return Ok(());
            }
            g.ticks += 1;
        }

        if !self.engine.health_ok().await {
            self.inner.lock().await.last_error = Some("core engine health failed".into());
            return Ok(());
        }

        let info = self.engine.engine().await?;
        {
            let mut g = self.inner.lock().await;
            g.engine_id = Some(info.engine_id.clone());
        }
        if !info.maintenance.eq_ignore_ascii_case("running") {
            self.inner.lock().await.last_error =
                Some(format!("engine maintenance={}", info.maintenance));
            return Ok(());
        }

        let symbols: Vec<String> = if self.cfg.symbols.is_empty() {
            info.pairs.iter().map(|p| p.symbol.clone()).collect()
        } else {
            self.cfg.symbols.clone()
        };
        if symbols.is_empty() {
            return Ok(());
        }

        let symbol = symbols[rand::thread_rng().gen_range(0..symbols.len())].clone();
        let book = self.engine.book(&symbol).await?;
        let kind = pick_kind(&self.cfg, &book);
        match kind {
            Some(kind) => self.act(kind, &symbol, &book).await?,
            None => {}
        }
        Ok(())
    }

    async fn act(&self, kind: ActionKind, symbol: &str, book: &BookSnapshot) -> anyhow::Result<()> {
        let mid = if matches!(kind, ActionKind::Make) {
            Some(self.mid_price(symbol, book).await)
        } else {
            None
        };
        let (side, order_type, price, quantity, note) = {
            let mut rng = rand::thread_rng();
            match kind {
                ActionKind::TakePartial | ActionKind::TakeFull => {
                    let take_asks = match (book.best_ask, book.best_bid) {
                        (Some(_), None) => true,
                        (None, Some(_)) => false,
                        (Some(_), Some(_)) => rng.gen_bool(0.5),
                        (None, None) => return Ok(()),
                    };
                    let (level_price, level_qty, side) = if take_asks {
                        let (p, q) = book.asks.first().copied().unwrap_or((0, 0));
                        (p, q, Side::Buy)
                    } else {
                        let (p, q) = book.bids.first().copied().unwrap_or((0, 0));
                        (p, q, Side::Sell)
                    };
                    if level_price == 0 || level_qty == 0 {
                        return Ok(());
                    }
                    let qty = if matches!(kind, ActionKind::TakePartial) && level_qty > 1 {
                        rng.gen_range(1..level_qty)
                    } else if matches!(kind, ActionKind::TakeFull) {
                        level_qty
                    } else {
                        1
                    };
                    let use_market = matches!(kind, ActionKind::TakeFull) && rng.gen_bool(0.35);
                    if use_market {
                        (
                            side,
                            OrderType::Market,
                            0,
                            qty,
                            format!("sweep {side:?} qty={qty} vs {level_qty}@{level_price}"),
                        )
                    } else {
                        (
                            side,
                            OrderType::Price,
                            level_price,
                            qty,
                            format!("hit {side:?} {qty}@{level_price} of {level_qty}"),
                        )
                    }
                }
                ActionKind::Make => {
                    let mid = mid.unwrap_or(self.cfg.default_price);
                    let sell = rng.gen_bool(0.5);
                    let offset = rng.gen_range(0..=3);
                    let price = if sell {
                        mid.saturating_add(offset).max(1)
                    } else {
                        mid.saturating_sub(offset).max(1)
                    };
                    let qty = rng.gen_range(self.cfg.min_qty..=self.cfg.max_qty);
                    (
                        if sell { Side::Sell } else { Side::Buy },
                        OrderType::Price,
                        price,
                        qty,
                        format!("rest liquidity {qty}@{price} mid={mid}"),
                    )
                }
            }
        };

        let req = PlaceOrderRequest {
            symbol: symbol.to_string(),
            side,
            order_type,
            price: if matches!(order_type, OrderType::Market) {
                None
            } else {
                Some(price)
            },
            quantity,
            account_id: format!(
                "{}:{}",
                self.cfg.account_id,
                match kind {
                    ActionKind::Make => "maker",
                    _ => "taker",
                }
            ),
            end_user_id: self.cfg.end_user_id,
        };

        match self.engine.place(&req).await? {
            Ok(result) => {
                let action = BotAction {
                    timestamp_ms: Utc::now().timestamp_millis(),
                    kind,
                    symbol: symbol.to_string(),
                    side,
                    order_type,
                    price: result.order.price,
                    quantity,
                    fills: result.trades.len(),
                    order_status: result.order.status.clone(),
                    remaining_quantity: result.order.remaining_quantity,
                    note,
                };
                tracing::info!(
                    kind = ?kind,
                    symbol,
                    side = ?side,
                    qty = quantity,
                    fills = action.fills,
                    status = %action.order_status,
                    remaining = action.remaining_quantity,
                    "{}",
                    action.note
                );
                let mut g = self.inner.lock().await;
                g.actions += 1;
                g.last_error = None;
                g.log.push_back(action);
                while g.log.len() > ACTION_CAP {
                    g.log.pop_front();
                }
            }
            Err(msg) => {
                tracing::warn!(%msg, "engine rejected bot order");
                self.inner.lock().await.last_error = Some(msg);
            }
        }
        Ok(())
    }

    async fn mid_price(&self, symbol: &str, book: &BookSnapshot) -> u64 {
        match (book.best_bid, book.best_ask) {
            (Some(b), Some(a)) => ((b + a) / 2).max(1),
            (Some(b), None) => b,
            (None, Some(a)) => a,
            (None, None) => self
                .engine
                .last_trade_price(symbol)
                .await
                .unwrap_or(self.cfg.default_price),
        }
    }
}

fn pick_kind(cfg: &Config, book: &BookSnapshot) -> Option<ActionKind> {
    let has_book = book.best_bid.is_some() || book.best_ask.is_some();
    let roll = rand::thread_rng().gen_range(0..100u8);
    let take_partial_end = cfg.take_partial_pct;
    let take_full_end = take_partial_end.saturating_add(cfg.take_full_pct);
    let make_end = take_full_end.saturating_add(cfg.make_pct);
    if roll < take_partial_end {
        if has_book {
            Some(ActionKind::TakePartial)
        } else {
            Some(ActionKind::Make)
        }
    } else if roll < take_full_end {
        if has_book {
            Some(ActionKind::TakeFull)
        } else {
            Some(ActionKind::Make)
        }
    } else if roll < make_end {
        Some(ActionKind::Make)
    } else {
        None
    }
}
