# FunnyX Client Center

Client Web–facing accounts, **login**, **OAuth**, **game-account bind**, **wallet / deposit / withdraw**, **chat** (`chat_messages`), and **e-shop** (catalog + pending orders). Session tokens / OAuth IdP+broker: [Session Token Server](../session_token_server/). Public ingress: [Gateway](../gateway/) on **8080**.

Default port: **8083**. Requires STS on **8082**. Path A bind / money also needs `company-a-server` on **18102**.

Set `POSTGRES_URL` to persist users, OAuth identities, sessions, games, game-account binds, and wallets. Deposit/withdraw require Postgres. Set `MONGO_URL` + `MONGO_DB=funnyx_client` for required `chat_messages` persistence (Session chat APIs return **503** if unset). Without `POSTGRES_URL`, auth/bind APIs run in memory (passwords still Argon2id).

## Run

```bash
cd backend/session_token_server && cargo run
cd backend/client_center && cp .env.sample .env && cargo run
# Path A / Path B partner:
cd dev_simulator/company-a-server && cargo run   # HTTP_PORT=18102
```

## Game bind (after register / OAuth)

1. **Game must exist** on the platform catalog (`GET /v1/client/games` — demo `game_id=1` / `DEMO_GAME`).
2. **Mapping key** = platform `end_user_id` (from session) + `game_id`.
3. Modes:
   - **(A) `partner_fetch`** — Platform register then bind: call Partner `GET /api/players/lookup` to verify the player is on that game, then write mapping.
   - **(B) `direct`** — Game App / Partner OAuth: create mapping from OAuth claims (`game_account_id`, `partner_user_id`); also auto-runs on `/partner/complete`.

```bash
# Login
TOKEN=$(curl -s http://127.0.0.1:8083/v1/client/login \
  -H "content-type: application/json" \
  -d '{"username":"demo_user","password":"demo"}' | jq -r .access_token)

# List games
curl -s http://127.0.0.1:8083/v1/client/games

# Path A — partner fetch bind
curl -s http://127.0.0.1:8083/v1/client/game-accounts/bind \
  -H "authorization: Bearer $TOKEN" -H "content-type: application/json" \
  -d '{"game_id":1,"mode":"partner_fetch","partner_username":"alice_01"}'

# Path B — direct bind (Game App already knows game_account_id)
curl -s http://127.0.0.1:8083/v1/client/game-accounts/bind \
  -H "authorization: Bearer $TOKEN" -H "content-type: application/json" \
  -d '{"game_id":1,"mode":"direct","game_account_id":"player_9001","partner_user_id":"pu_9001"}'

curl -s http://127.0.0.1:8083/v1/client/game-accounts -H "authorization: Bearer $TOKEN"
```

## E-shop (Session)

Catalog + pending orders. Fiat checkout goes to `payment-gate` (`18100`). Point `SHOP_PAYMENT_CALLBACK_URL` at Webhook Server `http://127.0.0.1:8084/v1/webhook/shop/payment`. After partner callback, Webhook Server calls `POST /v1/internal/shop/settle` (`X-Internal-Key`) to mark the order and credit wallets (Postgres). Duplicate `event_id` does not double-credit. Memory-only orders still update status if the settle request hits this process. Pending orders stay payable for **24 hours** (`expires_at`), then are marked `expired`.

```bash
curl -s http://127.0.0.1:8083/v1/shop/packages -H "authorization: Bearer $TOKEN"
curl -s http://127.0.0.1:8083/v1/shop/orders \
  -H "authorization: Bearer $TOKEN" -H "content-type: application/json" \
  -d '{"seller_type":"platform","package_id":1,"game_account_id":1}'
```

`game_account_id` is the platform binding row id from `GET /v1/client/game-accounts`.

## Wallet / deposit / withdraw (Session)

Requires `POSTGRES_URL`. Deposit/withdraw call company-a `/api/deposit` / `/api/withdrawal`, then complete after `sim_delay_ms` (v1; webhook settle later).

```bash
curl -s http://127.0.0.1:8083/v1/client/wallet -H "authorization: Bearer $TOKEN"
curl -s http://127.0.0.1:8083/v1/client/deposit \
  -H "authorization: Bearer $TOKEN" -H "content-type: application/json" \
  -d '{"game_account_id":1,"game_coin":"PLT","amount":10}'
curl -s http://127.0.0.1:8083/v1/client/transactions -H "authorization: Bearer $TOKEN"
```

## Chat (Session, Mongo)

```bash
# MONGO_URL=mongodb://127.0.0.1:27017 MONGO_DB=funnyx_client
curl -s http://127.0.0.1:8083/v1/client/chat/messages \
  -H "authorization: Bearer $TOKEN" -H "content-type: application/json" \
  -d '{"channel":"public","content":"hello"}'
curl -s "http://127.0.0.1:8083/v1/client/chat/messages?channel=public" \
  -H "authorization: Bearer $TOKEN"
```

## APIs

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| `GET` | `/health` | — | Liveness |
| `POST` | `/v1/client/register` | Public | Register + STS session |
| `POST` | `/v1/client/login` | Public | Login + STS session |
| `POST` | `/v1/client/logout` | Bearer | Revoke via STS |
| `GET` | `/v1/client/profile` | Bearer | Profile |
| `GET` | `/v1/client/games` | Public | Active games (`game_id` / `game_code`) |
| `GET` | `/v1/client/game-accounts` | Bearer | Bindings for current user |
| `POST` | `/v1/client/game-accounts/bind` | Bearer | `partner_fetch` or `direct` |
| `GET` | `/v1/client/wallet` | Bearer | Platform wallets (`user_wallets`) for bound accounts |
| `POST` | `/v1/client/deposit` | Bearer | Deposit via partner → pending txn → async complete |
| `POST` | `/v1/client/withdraw` | Bearer | Withdraw via partner (requires available balance) |
| `GET` | `/v1/client/transactions` | Bearer | Money txn history |
| `GET`/`POST` | `/v1/client/chat/messages` | Bearer | List / send Mongo `chat_messages` |
| `GET` | `/v1/client/oauth/partner/{partner_id}/start` | Public | Partner OAuth start |
| `GET` | `/v1/client/oauth/partner/complete` | Public | OAuth complete + **direct bind** |
| `GET` | `/v1/oauth/authorize` | Public | Platform OAuth → STS |
| `GET` | `/v1/shop/base-currency` | Bearer | System base fiat (`HKD`/`USD`) |
| `GET` | `/v1/shop/packages` | Bearer | Active `PLT_*` catalog |
| `GET` | `/v1/shop/packages/{id}` | Bearer | One platform package |
| `GET` | `/v1/shop/corp-products` | Bearer | Active Corp products |
| `GET` | `/v1/shop/corp-products/{id}` | Bearer | One Corp product |
| `POST` | `/v1/shop/orders` | Bearer | Create pending order + partner checkout |
| `GET` | `/v1/shop/orders` | Bearer | Current user orders |
| `GET` | `/v1/shop/orders/{id}` | Bearer | Order status |
| `POST` | `/v1/shop/orders/{id}/cancel` | Bearer | Cancel pending |
| `POST` | `/v1/internal/shop/settle` | `X-Internal-Key` | Webhook Server settle |
| `GET`/`POST` | `/v1/corp/shop/products` | MasterSigned | List / create Corp e-shop products |
| `GET`/`PUT` | `/v1/corp/shop/products/{id}` | MasterSigned | Get / update draft or inactive product |
| `PATCH` | `/v1/corp/shop/products/{id}/status` | MasterSigned | `draft` / `active` / `inactive` / `archived` |
| `GET`/`POST` | `/v1/corp/markets` | MasterSigned | List / submit market pair + lock + pending trade pool |
| `GET` | `/v1/corp/markets/{id}` | MasterSigned | Market pair + pool |
| `POST` | `/v1/corp/markets/{id}/pool` | MasterSigned | Create pending trade pool if missing |

## Corp (MasterSigned)

Headers: `X-Master-Account-Code`, `X-Master-Id`, `X-Api-Key`, `X-Signature`, `X-Timestamp`. Demo: `DEMO_MASTER_*` and `X-Signature: demo`. Session Bearer is **401**.

First market for a game must include Platform Token (`PLT`). Submit locks **client** `fx_game.game_balances` (`pending_locked`); pool row is `pending` until Admin approval transfers into the pool. Demo memory lock uses 1_000_000 of coin id `2` on `game_id=1`.

```bash
H=(-H "x-master-account-code: DEMO_MASTER_CODE" -H "x-master-id: DEMO_MASTER_ID" \
   -H "x-api-key: demo_api_key_do_not_use_live" -H "x-signature: demo" -H "x-timestamp: 1")

curl -s http://127.0.0.1:8083/v1/corp/shop/products "${H[@]}" \
  -H "content-type: application/json" \
  -d '{"code":"PACK_50","name":"Pack 50","product_type":"company_coin_package","credit_game_coin_id":2,"credit_amount":50,"fiat_price":4.9}'

curl -s -X PATCH http://127.0.0.1:8083/v1/corp/shop/products/2/status "${H[@]}" \
  -H "content-type: application/json" -d '{"status":"active"}'

curl -s http://127.0.0.1:8083/v1/corp/markets "${H[@]}" \
  -H "content-type: application/json" \
  -d '{"game_id":1,"base_game_coin_id":2,"quote_game_coin_id":1,"market_name":"DEMO_COIN/PLT","pool_depth":5000000,"initial_price":0.025,"base_amount":200000,"quote_amount":5000}'
```

Demo users: `demo_user` / `alice_plat` (password `demo`, stored as Argon2id). Partner players: `alice_01` / `bob_02` (`demo`).

## Env

See [`.env.sample`](.env.sample).
