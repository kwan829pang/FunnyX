# market-bot

Local Rust bot that watches FunnyX **Core Engine** books and places orders to simulate **partial fills**, **full fills**, and resting liquidity.

Gateway / Client Center are not required: the bot calls Core Engine HTTP directly (`CORE_ENGINE_URL`). Production path remains Gateway → Client Center → Core Engine.

## Run

```bash
# terminal 1
cd backend/core_engine
cp .env.sample .env
cargo run

# terminal 2
cd dev_simulator/market-bot
cp .env.sample .env
cargo run
```

Default listen: `http://127.0.0.1:18103` · engine `http://127.0.0.1:18200`.

## Behavior

Each tick (interval + random jitter):

1. `GET /v1/engine` — skip if maintenance is not `running`
2. Pick a listed pair (or `SYMBOLS`)
3. `GET /v1/books/{symbol}` — inspect BBO / depth
4. Roll an action:
   - **take_partial** — hit best bid or ask with quantity **less than** that level
   - **take_full** — take the whole best level (price or market)
   - **make** — rest a price order around mid / last trade / `DEFAULT_PRICE`
   - **idle** — wait for the next tick

Maker and taker use different `account_id` suffixes (`market_bot:maker` / `market_bot:taker`).

## APIs

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Liveness |
| `GET` | `/v1/bot` | Pause flag, tick/action counts, last error |
| `GET` | `/v1/bot/actions?limit=50` | Recent bot orders and fill counts |
| `POST` | `/v1/bot/pause` | Stop placing orders |
| `POST` | `/v1/bot/resume` | Resume |

```bash
curl -s http://127.0.0.1:18103/v1/bot
curl -s http://127.0.0.1:18103/v1/bot/actions
curl -s -X POST http://127.0.0.1:18103/v1/bot/pause
```

Seed a resting sell on the engine, then let the bot take it:

```bash
curl -s http://127.0.0.1:18200/v1/orders -H "content-type: application/json" -d "{\"symbol\":\"BTC/USDT\",\"side\":\"sell\",\"order_type\":\"price\",\"price\":50000,\"quantity\":5,\"account_id\":\"seed_seller\"}"
```
