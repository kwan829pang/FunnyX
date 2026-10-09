# FunnyX Core Engine

Private matching service for FunnyX. Design: [doc/gateway_core_engine_logic.md](../../doc/gateway_core_engine_logic.md). Market pool: [doc/partner.md](../../doc/partner.md) §8 / `fx_market.market_pools`.

## FunnyX differences

1. **Quote-asset shard** — one Core Engine instance owns a single quote (e.g. `USDT`) and only lists pairs ending in that quote (`BTC/USDT`, `ETH/USDT`, …).
2. **Max 10 pairs** — `MAX_PAIRS_PER_ENGINE = 10`. More pairs with the same quote need another engine instance (e.g. `core-engine-usdt-2`).
3. **Partner / Message Center notices** — on create/fill/cancel/pool-init, publish to Message Center and partner server group.
4. **Admin create pair + init pool** — `POST /v1/admin/pairs` lists the pair and creates an active market pool (wallets funded with base/quote amounts).
5. **HTTP info APIs** — engine identity, listed pairs, pools, book depth, trades/notices.
6. **Maintenance stop / restart** — persist pairs, pools, resting orders, and recent trades to local JSON under `DATA_DIR`, clear memory for shutdown; on start (or process boot) reload the snapshot into memory.

Order types (platform): **01 Market**, **02 Price** (`order_type`: `market` | `price`).

## Run

```bash
cd backend/core_engine
cp .env.sample .env
cargo run
```

Default: `http://127.0.0.1:18200` · quote `USDT` · bootstrap `BTC/USDT`, `ETH/USDT` · admin key `demo-admin-key`.

## APIs

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Liveness |
| `GET` | `/v1/engine` | Engine info (id, quote, pairs, pools, max 10) |
| `GET` | `/v1/engine/pairs` | On-list pairs |
| `POST` | `/v1/engine/pairs` | List a pair only (no pool; internal) |
| `DELETE` | `/v1/engine/pairs/{symbol}` | Delist pair (+ drop pool) |
| `POST` | `/v1/admin/pairs` | **Admin:** create pair **and** init market pool (`X-Admin-Key`) |
| `GET` | `/v1/engine/pools` | All market pools |
| `GET` | `/v1/engine/pools/{symbol}` | Pool for one pair |
| `POST` | `/v1/orders` | Place order |
| `GET` | `/v1/orders/{symbol}/{order_id}` | Inspect order |
| `DELETE` | `/v1/orders/{symbol}/{order_id}` | Cancel |
| `GET` | `/v1/books/{symbol}?levels=10` | BBO + depth |
| `GET` | `/v1/trades/{symbol}` | Recent trades |
| `GET` | `/v1/notices` | Recent engine notices |
| `GET` | `/v1/admin/maintenance` | Maintenance status + snapshot path |
| `POST` | `/v1/admin/maintenance/save` | Checkpoint to disk (`X-Admin-Key`) |
| `POST` | `/v1/admin/maintenance/stop` | Save → clear memory → `Stopped` (`X-Admin-Key`) |
| `POST` | `/v1/admin/maintenance/start` | Load disk → memory → `Running` (`X-Admin-Key`) |

### Maintenance stop / restart

Local file: `{DATA_DIR}/engine_snapshot.json` (atomic write via `.tmp` rename).

```bash
# Checkpoint while running
curl -s -X POST http://127.0.0.1:18200/v1/admin/maintenance/save \
  -H "X-Admin-Key: demo-admin-key"

# Stop for server maintenance (rejects new orders with 503)
curl -s -X POST http://127.0.0.1:18200/v1/admin/maintenance/stop \
  -H "X-Admin-Key: demo-admin-key"

# Resume from local snapshot
curl -s -X POST http://127.0.0.1:18200/v1/admin/maintenance/start \
  -H "X-Admin-Key: demo-admin-key"

curl -s http://127.0.0.1:18200/v1/admin/maintenance
```

On process start: if a snapshot exists it is loaded into memory (bootstrap pairs are skipped). On Ctrl+C / SIGTERM the engine saves a final snapshot. Periodic saves run every `SNAPSHOT_INTERVAL_SECS` (default 30).

### Admin create pair + init market pool

```bash
curl -s http://127.0.0.1:18200/v1/admin/pairs \
  -H "content-type: application/json" \
  -H "X-Admin-Key: demo-admin-key" \
  -d '{
  "symbol": "GAME/USDT",
  "market_id": "pair_3001",
  "admin_id": "admin_01",
  "corporate_user_id": 1,
  "funding_source": "gamecoin_lockup",
  "pool_depth": 5000000,
  "initial_price": 0.025,
  "base_amount": 200000,
  "quote_amount": 5000,
  "seed_book": true
}'
```

Creates:
- listed `TradingPair` on this quote engine
- `MarketPool` with `status=active`, wallet balances = `base_amount` / `quote_amount`
- optional seed sell on the book from pool base at `initial_price`
- notice to Message Center + partner group

### Engine info

```bash
curl -s http://127.0.0.1:18200/v1/engine
curl -s http://127.0.0.1:18200/v1/engine/pools/GAME%2FUSDT
```

### Place price + market orders

```bash
curl -s http://127.0.0.1:18200/v1/orders -H "content-type: application/json" -d '{
  "symbol":"BTC/USDT","side":"sell","order_type":"price","price":50000,"quantity":2,
  "account_id":"acc_seller","end_user_id":1
}'
```

## Scaling note

| Engine instance | Quote | Example pairs (≤10) |
| --- | --- | --- |
| `core-engine-usdt-1` | USDT | BTC/USDT … up to 10 |
| `core-engine-usdt-2` | USDT | next 10 USDT pairs |
| `core-engine-plt-1` | PLT | GAME/PLT … |

Gateway `market_engine_map` should route each `symbol` → `engine_id`.
