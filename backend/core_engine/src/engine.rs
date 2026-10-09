//! Quote-asset Core Engine: one instance owns pairs that share a quote (e.g. USDT), max 10.
//! Supports local snapshot persist for maintenance stop / restart.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::Serialize;
use tokio::sync::Mutex;

use crate::book::OrderBook;
use crate::notice::NoticePublisher;
use crate::order::{Order, OrderView};
use crate::persist::EngineSnapshot;
use crate::types::{
    AdminCreatePairInput, EngineError, MaintenanceStatus, MarketPool, OrderId, OrderStatus,
    OrderType, PairStatus, PoolStatus, Price, Quantity, Side, Trade, TradingPair,
    MAX_PAIRS_PER_ENGINE,
};

#[derive(Clone)]
pub struct CoreEngine {
    pub engine_id: String,
    pub quote_asset: String,
    pub max_pairs: usize,
    pub data_dir: PathBuf,
    order_seq: Arc<AtomicU64>,
    maintenance: Arc<Mutex<MaintenanceStatus>>,
    inner: Arc<Mutex<EngineInner>>,
    notices: NoticePublisher,
}

struct EngineInner {
    pairs: HashMap<String, TradingPair>,
    books: HashMap<String, OrderBook>,
    pools: HashMap<String, MarketPool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineInfo {
    pub engine_id: String,
    pub quote_asset: String,
    pub max_pairs: usize,
    pub listed_pairs: usize,
    pub pairs: Vec<TradingPair>,
    pub pools: Vec<MarketPool>,
    pub maintenance: MaintenanceStatus,
    pub snapshot_path: String,
    pub status: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminCreatePairResult {
    pub pair: TradingPair,
    pub pool: MarketPool,
    pub seed_orders: Vec<OrderView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaceResult {
    pub order: OrderView,
    pub trades: Vec<Trade>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MaintenanceInfo {
    pub maintenance: MaintenanceStatus,
    pub snapshot_path: String,
    pub snapshot_exists: bool,
    pub listed_pairs: usize,
    pub open_orders: usize,
    pub pools: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BookSnapshot {
    pub symbol: String,
    pub best_bid: Option<Price>,
    pub best_ask: Option<Price>,
    pub bids: Vec<(Price, Quantity)>,
    pub asks: Vec<(Price, Quantity)>,
}

impl CoreEngine {
    pub fn new(
        engine_id: String,
        quote_asset: String,
        data_dir: PathBuf,
        notices: NoticePublisher,
    ) -> Self {
        Self {
            engine_id,
            quote_asset: quote_asset.to_ascii_uppercase(),
            max_pairs: MAX_PAIRS_PER_ENGINE,
            data_dir,
            order_seq: Arc::new(AtomicU64::new(1)),
            maintenance: Arc::new(Mutex::new(MaintenanceStatus::Running)),
            inner: Arc::new(Mutex::new(EngineInner {
                pairs: HashMap::new(),
                books: HashMap::new(),
                pools: HashMap::new(),
            })),
            notices,
        }
    }

    pub async fn maintenance_status(&self) -> MaintenanceStatus {
        *self.maintenance.lock().await
    }

    async fn ensure_running(&self) -> Result<(), EngineError> {
        let status = *self.maintenance.lock().await;
        if status != MaintenanceStatus::Running {
            return Err(EngineError::Maintenance(format!("{status:?}")));
        }
        Ok(())
    }

    pub async fn info(&self) -> EngineInfo {
        let guard = self.inner.lock().await;
        let mut pairs: Vec<_> = guard.pairs.values().cloned().collect();
        pairs.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        let mut pools: Vec<_> = guard.pools.values().cloned().collect();
        pools.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        let maintenance = *self.maintenance.lock().await;
        EngineInfo {
            engine_id: self.engine_id.clone(),
            quote_asset: self.quote_asset.clone(),
            max_pairs: self.max_pairs,
            listed_pairs: pairs.len(),
            pairs,
            pools,
            maintenance,
            snapshot_path: EngineSnapshot::path(&self.data_dir).display().to_string(),
            status: format!("{maintenance:?}").to_ascii_lowercase(),
            source: "core_engine".into(),
        }
    }

    pub async fn maintenance_info(&self) -> MaintenanceInfo {
        let guard = self.inner.lock().await;
        let open_orders: usize = guard
            .books
            .values()
            .map(|b| {
                let snap = b.to_persist();
                snap.bid_levels.iter().map(|l| l.orders.len()).sum::<usize>()
                    + snap.ask_levels.iter().map(|l| l.orders.len()).sum::<usize>()
            })
            .sum();
        MaintenanceInfo {
            maintenance: *self.maintenance.lock().await,
            snapshot_path: EngineSnapshot::path(&self.data_dir).display().to_string(),
            snapshot_exists: EngineSnapshot::exists(&self.data_dir),
            listed_pairs: guard.pairs.len(),
            open_orders,
            pools: guard.pools.len(),
        }
    }

    pub async fn export_snapshot(&self) -> EngineSnapshot {
        let guard = self.inner.lock().await;
        let maintenance = *self.maintenance.lock().await;
        let mut pairs: Vec<_> = guard.pairs.values().cloned().collect();
        pairs.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        let mut pools: Vec<_> = guard.pools.values().cloned().collect();
        pools.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        let mut books: Vec<_> = guard.books.values().map(|b| b.to_persist()).collect();
        books.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        EngineSnapshot {
            version: 1,
            engine_id: self.engine_id.clone(),
            quote_asset: self.quote_asset.clone(),
            next_order_id: self.order_seq.load(Ordering::Relaxed),
            maintenance,
            saved_at_ms: chrono::Utc::now().timestamp_millis(),
            pairs,
            pools,
            books,
        }
    }

    pub async fn save_snapshot(&self) -> Result<PathBuf, EngineError> {
        let snap = self.export_snapshot().await;
        snap.save(&self.data_dir).map_err(EngineError::Persist)
    }

    pub async fn load_snapshot_from_disk(&self) -> Result<(), EngineError> {
        if !EngineSnapshot::exists(&self.data_dir) {
            return Err(EngineError::Persist("no snapshot file found".into()));
        }
        let snap = EngineSnapshot::load(&self.data_dir).map_err(EngineError::Persist)?;
        self.import_snapshot(snap).await
    }

    pub async fn import_snapshot(&self, snap: EngineSnapshot) -> Result<(), EngineError> {
        if snap.quote_asset.to_ascii_uppercase() != self.quote_asset {
            return Err(EngineError::Persist(format!(
                "snapshot quote {} != engine quote {}",
                snap.quote_asset, self.quote_asset
            )));
        }
        let mut pairs = HashMap::new();
        let mut pools = HashMap::new();
        let mut books = HashMap::new();
        for pair in snap.pairs {
            pairs.insert(pair.symbol.clone(), pair);
        }
        for pool in snap.pools {
            pools.insert(pool.symbol.clone(), pool);
        }
        for book_snap in snap.books {
            let symbol = book_snap.symbol.clone();
            books.insert(symbol, OrderBook::from_persist(book_snap));
        }
        for symbol in pairs.keys() {
            books
                .entry(symbol.clone())
                .or_insert_with(|| OrderBook::new(symbol.clone()));
        }

        {
            let mut guard = self.inner.lock().await;
            guard.pairs = pairs;
            guard.pools = pools;
            guard.books = books;
        }
        self.order_seq
            .store(snap.next_order_id.max(1), Ordering::Relaxed);
        *self.maintenance.lock().await = MaintenanceStatus::Running;
        tracing::info!(
            path = %EngineSnapshot::path(&self.data_dir).display(),
            "restored engine snapshot into memory"
        );
        Ok(())
    }

    /// Persist to local disk, mark stopped, clear memory (state remains on disk).
    pub async fn maintenance_stop(&self) -> Result<MaintenanceInfo, EngineError> {
        *self.maintenance.lock().await = MaintenanceStatus::Draining;
        let path = self.save_snapshot().await?;
        {
            let mut guard = self.inner.lock().await;
            guard.pairs.clear();
            guard.pools.clear();
            guard.books.clear();
        }
        *self.maintenance.lock().await = MaintenanceStatus::Stopped;
        tracing::info!(path = %path.display(), "maintenance stop: snapshot saved, memory cleared");
        Ok(self.maintenance_info().await)
    }

    /// Load local snapshot into memory and resume trading.
    pub async fn maintenance_start(&self) -> Result<MaintenanceInfo, EngineError> {
        self.load_snapshot_from_disk().await?;
        *self.maintenance.lock().await = MaintenanceStatus::Running;
        tracing::info!("maintenance start: engine running from local snapshot");
        Ok(self.maintenance_info().await)
    }

    pub async fn list_pairs(&self) -> Vec<TradingPair> {
        let guard = self.inner.lock().await;
        let mut pairs: Vec<_> = guard.pairs.values().cloned().collect();
        pairs.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        pairs
    }

    pub async fn add_pair(&self, mut pair: TradingPair) -> Result<TradingPair, EngineError> {
        self.ensure_running().await?;
        if pair.quote != self.quote_asset {
            return Err(EngineError::QuoteMismatch {
                engine_quote: self.quote_asset.clone(),
                pair_quote: pair.quote,
            });
        }
        let mut guard = self.inner.lock().await;
        if guard.pairs.contains_key(&pair.symbol) {
            return Err(EngineError::PairExists(pair.symbol));
        }
        if guard.pairs.len() >= self.max_pairs {
            return Err(EngineError::PairLimit {
                quote: self.quote_asset.clone(),
            });
        }
        pair.status = PairStatus::Listed;
        guard
            .books
            .insert(pair.symbol.clone(), OrderBook::new(pair.symbol.clone()));
        guard.pairs.insert(pair.symbol.clone(), pair.clone());
        Ok(pair)
    }

    pub async fn admin_create_pair_with_pool(
        &self,
        input: AdminCreatePairInput,
    ) -> Result<AdminCreatePairResult, EngineError> {
        self.ensure_running().await?;
        if input.pool_depth <= 0.0
            || input.initial_price <= 0.0
            || input.base_amount <= 0.0
            || input.quote_amount <= 0.0
        {
            return Err(EngineError::InvalidPool(
                "pool_depth, initial_price, base_amount, quote_amount must be > 0".into(),
            ));
        }
        let funding = input.funding_source.to_ascii_lowercase();
        if funding != "gamecoin_lockup" && funding != "platform_token_deposit" {
            return Err(EngineError::InvalidPool(
                "funding_source must be gamecoin_lockup or platform_token_deposit".into(),
            ));
        }

        let mut pair = TradingPair::parse(&input.symbol, input.market_id.clone())?;
        pair.tick_size = input.tick_size.max(1);
        pair.lot_size = input.lot_size.max(1);
        let pair = self.add_pair(pair).await?;

        let now = chrono::Utc::now().timestamp_millis();
        let pool = MarketPool {
            pool_id: format!("pool_{}", uuid::Uuid::new_v4().simple()),
            symbol: pair.symbol.clone(),
            market_id: pair.market_id.clone(),
            pool_depth: input.pool_depth,
            initial_price: input.initial_price,
            base_amount: input.base_amount,
            quote_amount: input.quote_amount,
            base_wallet_balance: input.base_amount,
            quote_wallet_balance: input.quote_amount,
            funding_source: funding,
            status: PoolStatus::Active,
            approved_by_admin_id: Some(input.admin_id.clone()),
            corporate_user_id: input.corporate_user_id,
            created_at_ms: now,
            updated_at_ms: now,
        };

        {
            let mut guard = self.inner.lock().await;
            guard.pools.insert(pair.symbol.clone(), pool.clone());
        }

        let mut seed_orders = Vec::new();
        if input.seed_book {
            let price_ticks = price_to_ticks(input.initial_price, pair.tick_size);
            let qty = (input.base_amount.floor() as Quantity).max(pair.lot_size);
            if qty > 0 && price_ticks > 0 {
                let placed = self
                    .place_order(
                        &pair.symbol,
                        Side::Sell,
                        OrderType::Price,
                        price_ticks,
                        qty,
                        format!("market_pool:{}", pool.pool_id),
                        None,
                        input.corporate_user_id,
                    )
                    .await?;
                seed_orders.push(placed.order);
            }
        }

        self.notices.publish_pool_created(&pair, &pool).await;
        let _ = self.save_snapshot().await;

        Ok(AdminCreatePairResult {
            pair,
            pool,
            seed_orders,
        })
    }

    pub async fn get_pool(&self, symbol: &str) -> Result<MarketPool, EngineError> {
        let symbol = symbol.to_ascii_uppercase();
        let guard = self.inner.lock().await;
        guard
            .pools
            .get(&symbol)
            .cloned()
            .ok_or(EngineError::PoolNotFound(symbol))
    }

    pub async fn list_pools(&self) -> Vec<MarketPool> {
        let guard = self.inner.lock().await;
        let mut pools: Vec<_> = guard.pools.values().cloned().collect();
        pools.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        pools
    }

    pub async fn remove_pair(&self, symbol: &str) -> Result<(), EngineError> {
        self.ensure_running().await?;
        let symbol = symbol.to_ascii_uppercase();
        let mut guard = self.inner.lock().await;
        if guard.pairs.remove(&symbol).is_none() {
            return Err(EngineError::PairNotFound(symbol));
        }
        guard.books.remove(&symbol);
        guard.pools.remove(&symbol);
        Ok(())
    }

    pub async fn place_order(
        &self,
        symbol: &str,
        side: Side,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
        account_id: String,
        end_user_id: Option<i64>,
        corporate_user_id: Option<i64>,
    ) -> Result<PlaceResult, EngineError> {
        self.ensure_running().await?;
        if quantity == 0 {
            return Err(EngineError::InvalidOrder("quantity must be > 0".into()));
        }
        if matches!(order_type, OrderType::Price) && price == 0 {
            return Err(EngineError::InvalidOrder(
                "price order requires price > 0".into(),
            ));
        }

        let symbol = symbol.to_ascii_uppercase();
        let order_id = self.order_seq.fetch_add(1, Ordering::Relaxed);
        let order = Arc::new(Order::new(
            order_id,
            symbol.clone(),
            side,
            order_type,
            price,
            quantity,
            account_id,
            end_user_id,
            corporate_user_id,
        ));

        let trades = {
            let mut guard = self.inner.lock().await;
            if !guard.pairs.contains_key(&symbol) {
                return Err(EngineError::PairNotFound(symbol));
            }
            let pair = guard.pairs.get(&symbol).unwrap();
            if pair.status != PairStatus::Listed {
                return Err(EngineError::InvalidOrder(format!(
                    "pair {symbol} is not listed"
                )));
            }
            let book = guard.books.get_mut(&symbol).unwrap();
            book.place(Arc::clone(&order))
        };

        let view = order.to_view();
        if trades.is_empty() {
            self.notices
                .publish_order_event(
                    if matches!(view.status, OrderStatus::Rejected) {
                        "rejected"
                    } else {
                        "created"
                    },
                    &view,
                )
                .await;
        } else {
            self.notices.publish_trades(&trades, &view, None).await;
        }

        Ok(PlaceResult {
            order: view,
            trades,
        })
    }

    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: OrderId,
    ) -> Result<OrderView, EngineError> {
        // Cancels allowed during draining; blocked only when stopped with empty memory.
        let status = *self.maintenance.lock().await;
        if status == MaintenanceStatus::Stopped {
            return Err(EngineError::Maintenance("Stopped".into()));
        }
        let symbol = symbol.to_ascii_uppercase();
        let order = {
            let mut guard = self.inner.lock().await;
            let book = guard
                .books
                .get_mut(&symbol)
                .ok_or_else(|| EngineError::PairNotFound(symbol.clone()))?;
            book.cancel(order_id)
                .map_err(EngineError::CancelRejected)?
        };
        let view = order.to_view();
        self.notices.publish_order_event("cancelled", &view).await;
        Ok(view)
    }

    pub async fn get_order(
        &self,
        symbol: &str,
        order_id: OrderId,
    ) -> Result<OrderView, EngineError> {
        let symbol = symbol.to_ascii_uppercase();
        let guard = self.inner.lock().await;
        let book = guard
            .books
            .get(&symbol)
            .ok_or_else(|| EngineError::PairNotFound(symbol))?;
        book.get_order(order_id)
            .map(|o| o.to_view())
            .ok_or(EngineError::OrderNotFound(order_id))
    }

    pub async fn book_snapshot(
        &self,
        symbol: &str,
        levels: usize,
    ) -> Result<BookSnapshot, EngineError> {
        let symbol = symbol.to_ascii_uppercase();
        let guard = self.inner.lock().await;
        let book = guard
            .books
            .get(&symbol)
            .ok_or_else(|| EngineError::PairNotFound(symbol.clone()))?;
        let (bids, asks) = book.depth(levels.max(1));
        Ok(BookSnapshot {
            symbol,
            best_bid: book.best_bid(),
            best_ask: book.best_ask(),
            bids,
            asks,
        })
    }

    pub async fn recent_trades(
        &self,
        symbol: &str,
        limit: usize,
    ) -> Result<Vec<Trade>, EngineError> {
        let symbol = symbol.to_ascii_uppercase();
        let guard = self.inner.lock().await;
        let book = guard
            .books
            .get(&symbol)
            .ok_or_else(|| EngineError::PairNotFound(symbol))?;
        Ok(book.recent_trades(limit.max(1)))
    }

    pub fn notices(&self) -> &NoticePublisher {
        &self.notices
    }
}

fn price_to_ticks(price: f64, tick_size: Price) -> Price {
    let tick = tick_size.max(1) as f64;
    let scaled = (price / tick).round().max(1.0) as Price;
    scaled.saturating_mul(tick_size.max(1))
}
