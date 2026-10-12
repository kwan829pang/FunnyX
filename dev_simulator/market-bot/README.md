# market-bot

Local Rust bot that watches FunnyX **Core Engine** books and places orders to simulate **partial fills**, **full fills**, and resting liquidity.

**Place + book (default):** Gateway → Client Center → Core Engine (`GATEWAY_URL` + Session login: `/v1/orders`, `/v1/markets`, `/v1/markets/{id}/book`).  
**Health / maintenance:** still direct `CORE_ENGINE_URL` (`/health`, `/v1/engine`).

## Run

Prerequisites for Gateway place: STS, Client Center (`POSTGRES_URL` + demo seed), Gateway, Core Engine. Seed user `demo_user` / `demo` and `game_accounts.id=1`.

```bash
# Core Engine (book + matching)
cd backend/core_engine && cp .env.sample .env && cargo run

# STS + Client Center + Gateway (Session place)
# … start session_token_server, client_center, gateway …

cd dev_simulator/market-bot
cp .env.sample .env
cargo run
```

Default listen: `http://127.0.0.1:18103` · book engine `18200` · place Gateway `8080`.

Legacy direct place (no Gateway): set `GATEWAY_URL=` empty in `.env`.

## Behavior

Each tick (interval + random jitter):

1. `GET {CORE_ENGINE_URL}/v1/engine` — skip if maintenance is not `running`
2. Pick a listed pair (or `SYMBOLS`)
3. Book: Gateway `GET /v1/markets/{id}/book` (or direct engine books if `GATEWAY_URL` empty)
4. Roll an action:
   - **take_partial** — hit best bid or ask with quantity **less than** that level
   - **take_full** — take the whole best level (price or market)
   - **make** — rest a price order around mid / last trade / `DEFAULT_PRICE`
   - **idle** — wait for the next tick
5. Place:
   - **Gateway:** `POST /v1/client/login` then `POST /v1/orders` with Bearer (re-login on 401)
   - **Direct:** `POST {CORE_ENGINE_URL}/v1/orders` with `account_id` maker/taker suffixes

## Env

See [`.env.sample`](.env.sample). Important:

| Var | Default | Purpose |
| --- | --- | --- |
| `GATEWAY_URL` | `http://127.0.0.1:8080` | Session place; empty = direct engine |
| `BOT_USERNAME` / `BOT_PASSWORD` | `demo_user` / `demo` | CC login via Gateway |
| `GAME_ACCOUNT_ID` | `1` | Seed `demo_user` game account row |
| `CORE_ENGINE_URL` | `http://127.0.0.1:18200` | Book + health |

## APIs

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Liveness |
| `GET` | `/v1/bot` | Pause flag, `place_via`, tick/action counts, last error |
| `GET` | `/v1/bot/actions?limit=50` | Recent bot orders and fill counts |
| `POST` | `/v1/bot/pause` | Stop placing orders |
| `POST` | `/v1/bot/resume` | Resume |

```bash
curl -s http://127.0.0.1:18103/v1/bot
curl -s http://127.0.0.1:18103/v1/bot/actions
curl -s -X POST http://127.0.0.1:18103/v1/bot/pause
```

Manual Session place via Gateway (same path the bot uses):

```bash
TOKEN=$(curl -s http://127.0.0.1:8080/v1/client/login \
  -H "content-type: application/json" \
  -d '{"username":"demo_user","password":"demo"}' | jq -r .access_token)

curl -s http://127.0.0.1:8080/v1/orders \
  -H "authorization: Bearer $TOKEN" \
  -H "content-type: application/json" \
  -d '{"symbol":"BTC/USDT","side":"buy","order_type":"price","price":50000,"quantity":1,"game_account_id":1}'
```

Seed a resting sell on the engine (liquidity the Session bot can take):

```bash
curl -s http://127.0.0.1:18200/v1/orders -H "content-type: application/json" \
  -d '{"symbol":"BTC/USDT","side":"sell","order_type":"price","price":50000,"quantity":5,"account_id":"seed_seller"}'
```
