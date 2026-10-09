# Dev simulators

Local helpers for FunnyX integration testing (see [doc/dev.md](../doc/dev.md)). Not production services.


| Folder | Status | Port (default) | Role |
| --- | --- | --- | --- |
| [payment-gate](payment-gate/) | Ready | `18100` | Partner fiat pay + deposit webhook simulator |
| [message-queue](message-queue/) | Ready | `18101` | `outbound_notices` drain / inspection queue |
| [company-a-server](company-a-server/) | Ready | `18102` | Company Partner APIs (OAuth, Transfer, Item List, balance/deposit/withdraw) |
| [market-bot](market-bot/) | Ready | `18103` | Watch Core Engine books; place partial/full fills + resting orders |


Each ready simulator is a small standalone Rust crate. Copy `.env.sample` → `.env`, then `cargo run` in its folder.

## Docker (all tools)

Compose file: [`docker-compose.yml`](docker-compose.yml). Brings up payment-gate, message-queue, company-a-server, **core-engine**, and market-bot on network `funnyx-dev-sim`.

```bash
cd dev_simulator
docker compose up --build
# detach:
docker compose up --build -d
docker compose ps
docker compose logs -f market-bot
docker compose down
```

| Service | Host URL |
| --- | --- |
| payment-gate | http://127.0.0.1:18100 |
| message-queue | http://127.0.0.1:18101 |
| company-a-server | http://127.0.0.1:18102 |
| market-bot | http://127.0.0.1:18103 |
| core-engine | http://127.0.0.1:18200 |

Core Engine snapshots persist in Docker volume `core_engine_data`. Internal DNS: `http://core-engine:18200`, `http://message-queue:18101`, etc.

---



## Shared header: `X-Sim-Status`

All **submit/create** calls always start as **PENDING**. Schedule the final outcome with:

```http
X-Sim-Status: {STATUS} {ms}
```


| Header example        | Meaning                               |
| --------------------- | ------------------------------------- |
| *(omit)* or `PENDING` | Stay pending; no webhook              |
| `SUCCESS 0`           | ASAP → success path + webhook         |
| `SUCCESS 3000`        | After 3s → success path + webhook     |
| `REJECTED 5000`       | After 5s → rejected/failed + webhook  |
| `CANCEL 2000`         | After 2s → cancelled/failed + webhook |


---



## payment-gate (`http://127.0.0.1:18100`)

Detail: [payment-gate/README.md](payment-gate/README.md)

### APIs


| Method | Path                                              | Body                            |
| ------ | ------------------------------------------------- | ------------------------------- |
| `GET`  | `/health`                                         | —                               |
| `POST` | `/v1/shop/payment`                                | Shop create JSON                |
| `GET`  | `/v1/shop/payment/{partner_order_no}`             | —                               |
| `POST` | `/v1/shop/payment/{partner_order_no}/complete`    | Optional complete JSON / header |
| `POST` | `/v1/deposit/payment`                             | Deposit create JSON             |
| `GET`  | `/v1/deposit/payment/{partner_order_no}`          | —                               |
| `POST` | `/v1/deposit/payment/{partner_order_no}/complete` | Optional complete JSON / header |
| `GET`  | `/v1/payments`                                    | —                               |
| `GET`  | `/pay/{partner_order_no}`                         | HTML checkout helper            |




### Example: create shop payment

```bash
curl -s http://127.0.0.1:18100/v1/shop/payment \
  -H "content-type: application/json" \
  -H "X-Sim-Status: SUCCESS 3000" \
  -d '{
  "request_id": "req_shop_1001",
  "shop_order_id": 5001,
  "seller_type": "platform",
  "partner_id": "partner_2001",
  "user_id": "user_1001",
  "game_account_id": "player_9001",
  "package_code": "PLT_1000",
  "product_code": null,
  "credit_game_coin": "PLT",
  "credit_amount": 1000,
  "fiat_currency": "HKD",
  "fiat_price": 88.00,
  "callback_url": "http://127.0.0.1:8080/webhook/shop/payment",
  "return_url": "http://127.0.0.1:3000/shop/orders/5001"
}'
```

**Immediate response (always pending):**

```json
{
  "partner_order_no": "PAY-1000",
  "checkout_url": "http://127.0.0.1:18100/pay/PAY-1000",
  "status": "pending",
  "sim_status": "SUCCESS",
  "sim_delay_ms": 3000,
  "expires_at": "2026-10-02T03:30:00Z",
  "source": "test"
}
```

**Webhook callback** (after `sim_delay_ms`, POSTed to `callback_url`):

```json
{
  "event_id": "evt_…",
  "partner_order_no": "PAY-1000",
  "shop_order_id": 5001,
  "seller_type": "platform",
  "status": "paid",
  "fiat_currency": "HKD",
  "fiat_paid": 88.0,
  "credit_game_coin": "PLT",
  "credit_amount": 1000,
  "paid_at": "2026-10-02T03:00:03Z",
  "signature": "test-signature",
  "source": "test"
}
```

`X-Sim-Status` → callback `status`: `SUCCESS`→`paid`, `REJECTED`→`failed`, `CANCEL`→`cancelled`.

### Example: create deposit

```bash
curl -s http://127.0.0.1:18100/v1/deposit/payment \
  -H "content-type: application/json" \
  -H "X-Sim-Status: REJECTED 5000" \
  -d '{
  "request_id": "req_778899",
  "partner_id": "partner_2001",
  "user_id": "user_1001",
  "game_id": "game_001",
  "game_account_id": "player_9001",
  "transaction_type": "deposit",
  "amount": 100.0,
  "game_coin": "PLT",
  "channel": "partner_token",
  "source": "test",
  "callback_url": "http://127.0.0.1:8080/webhook/deposit",
  "metadata": { "order_no": "ORD-1001", "remark": "top up game wallet" }
}'
```

**Immediate response:**

```json
{
  "partner_order_no": "DEP-1001",
  "status": "pending",
  "sim_status": "REJECTED",
  "sim_delay_ms": 5000,
  "source": "test"
}
```

**Webhook callback** (after 5000ms):

```json
{
  "event_id": "evt_…",
  "partner_order_no": "DEP-1001",
  "request_id": "req_778899",
  "partner_id": "partner_2001",
  "user_id": "user_1001",
  "game_id": "game_001",
  "game_account_id": "player_9001",
  "transaction_type": "deposit",
  "amount": 100.0,
  "game_coin": "PLT",
  "status": "rejected",
  "paid_at": "2026-10-02T03:00:05Z",
  "signature": "test-signature",
  "source": "test"
}
```

`X-Sim-Status` → deposit callback `status`: `SUCCESS`→`success`, `REJECTED`→`rejected`, `CANCEL`→`cancelled`.

### Example: schedule complete on existing payment

```bash
curl -s http://127.0.0.1:18100/v1/shop/payment/PAY-1000/complete \
  -H "content-type: application/json" \
  -H "X-Sim-Status: CANCEL 2000" \
  -d '{}'
```

Or apply immediately without the header:

```bash
curl -s http://127.0.0.1:18100/v1/shop/payment/PAY-1000/complete \
  -H "content-type: application/json" \
  -d '{ "status": "paid", "fire_callback": true }'
```

---



## message-queue (`http://127.0.0.1:18101`)

Detail: [message-queue/README.md](message-queue/README.md)

### APIs


| Method   | Path                                  | Body                       |
| -------- | ------------------------------------- | -------------------------- |
| `GET`    | `/health`                             | —                          |
| `POST`   | `/v1/notices`                         | Enqueue JSON               |
| `GET`    | `/v1/notices?delivery_status=pending` | —                          |
| `GET`    | `/v1/notices/stats`                   | —                          |
| `POST`   | `/v1/notices/drain`                   | Optional `{ "limit": 10 }` |
| `GET`    | `/v1/notices/{id}`                    | —                          |
| `POST`   | `/v1/notices/{id}/ack`                | Ack JSON                   |
| `POST`   | `/v1/notices/{id}/retry`              | —                          |
| `DELETE` | `/v1/notices`                         | —                          |




### Example: enqueue notice

```bash
curl -s http://127.0.0.1:18101/v1/notices \
  -H "content-type: application/json" \
  -H "X-Sim-Status: SUCCESS 3000" \
  -d '{
  "end_user_id": 1,
  "source_type": "shop_order",
  "source_id": 5001,
  "event_type": "pending_payment",
  "title": "Shop order pending payment",
  "body": "Order 5001 awaiting partner fiat payment",
  "payload": { "shop_order_id": 5001, "status": "pending" },
  "scheduled_at": 0,
  "callback_url": "http://127.0.0.1:8080/webhook/notice"
}'
```

**Immediate response (always pending):**

```json
{
  "id": 1,
  "end_user_id": 1,
  "source_type": "shop_order",
  "source_id": 5001,
  "event_type": "pending_payment",
  "title": "Shop order pending payment",
  "body": "Order 5001 awaiting partner fiat payment",
  "payload": { "shop_order_id": 5001, "status": "pending" },
  "delivery_status": "pending",
  "sim_status": "SUCCESS",
  "sim_delay_ms": 3000,
  "callback_url": "http://127.0.0.1:8080/webhook/notice",
  "retry_count": 0,
  "scheduled_at": 0,
  "sent_at": 0,
  "notification_id": null,
  "created_at": 1790910000000,
  "updated_at": 1790910000000,
  "source": "test"
}
```

**Webhook callback** (after `sim_delay_ms`, POSTed to `callback_url` or `NOTICE_CALLBACK_URL`):

```json
{
  "event_id": "evt_…",
  "notice_id": 1,
  "end_user_id": 1,
  "source_type": "shop_order",
  "source_id": 5001,
  "event_type": "pending_payment",
  "delivery_status": "sent",
  "sim_status": "SUCCESS",
  "title": "Shop order pending payment",
  "body": "Order 5001 awaiting partner fiat payment",
  "payload": { "shop_order_id": 5001, "status": "pending" },
  "source": "test"
}
```

`X-Sim-Status` → `delivery_status`: `SUCCESS`→`sent`, `REJECTED`/`CANCEL`→`failed`.

Allowed `source_type`: `marketplace_deal`, `marketplace_deal_request`, `order`, `trade`, `shop_order`, `deposit_withdrawal_txn`, `corp_token_order`.

Allowed `event_type`: `created`, `pending_payment`, `completed`, `cancelled`, `rejected`, `filled`, `partial_filled`, `failed`, `status_changed`.

### Example: drain + ack

```bash
curl -s http://127.0.0.1:18101/v1/notices/drain \
  -H "content-type: application/json" \
  -d '{ "limit": 10 }'

curl -s http://127.0.0.1:18101/v1/notices/1/ack \
  -H "content-type: application/json" \
  -d '{ "delivery_status": "sent", "notification_id": 42 }'
```



### Example: list / stats

```bash
curl -s "http://127.0.0.1:18101/v1/notices?delivery_status=pending"
curl -s http://127.0.0.1:18101/v1/notices/stats
```

---

## company-a-server (`http://127.0.0.1:18102`)

Detail: [company-a-server/README.md](company-a-server/README.md) · UML: [doc/partner-uml.md](../doc/partner-uml.md)

### APIs

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| `GET` | `/health` | — | Liveness |
| `GET` | `/v1/partner/setup` | — | Endpoint registration snapshot for Corp setup |
| `GET` | `/oauth/authorize` | OAuth client | Start authorize (redirects to login) |
| `POST` | `/oauth/token` | client_id + secret | Exchange code → access_token |
| `GET` | `/oauth/userinfo` | Bearer token | Partner user claims |
| `POST` | `/api/transfer` | `X-Api-Key` | Marketplace / asset transfer |
| `GET` | `/api/transfer/{id}` | — | Inspect transfer |
| `GET` | `/api/items?game_account_id=` | `X-Api-Key` | Item / game-assets list |
| `GET` | `/api/balance?game_account_id=` | `X-Api-Key` | Optional balance |
| `POST` | `/api/deposit` | `X-Api-Key` | Optional deposit |
| `POST` | `/api/withdrawal` | `X-Api-Key` | Optional withdrawal |
| `POST` | `/api/callback` | — | Inbound callback sink |

### Example: partner setup

```bash
curl -s http://127.0.0.1:18102/v1/partner/setup
```

### Example: Item List

```bash
curl -s "http://127.0.0.1:18102/api/items?game_account_id=player_9001" \
  -H "X-Api-Key: demo-api-key"
```

```json
{
  "game_account_id": "player_9001",
  "items": [
    { "item_ref_id": "sword_01", "name": "Iron Sword", "qty": 1, "game_account_id": "player_9001" },
    { "item_ref_id": "shield_02", "name": "Wooden Shield", "qty": 1, "game_account_id": "player_9001" }
  ],
  "source": "test"
}
```

### Example: Transfer (game item)

```bash
curl -s http://127.0.0.1:18102/api/transfer \
  -H "content-type: application/json" \
  -H "X-Api-Key: demo-api-key" \
  -H "X-Sim-Status: SUCCESS 3000" \
  -d '{
  "request_id": "xfer_7001",
  "partner_id": "partner_2001",
  "from_game_account_id": "player_9001",
  "to_game_account_id": "player_9002",
  "asset_type": "game_item",
  "item_ref_id": "sword_01",
  "game_coin": null,
  "amount": 1,
  "deal_id": "deal_4001"
}'
```

**Immediate response:**

```json
{
  "request_id": "xfer_7001",
  "transfer_id": "XFER-1000",
  "status": "pending",
  "sim_status": "SUCCESS",
  "sim_delay_ms": 3000,
  "asset_type": "game_item",
  "item_ref_id": "sword_01",
  "amount": 1.0,
  "source": "test"
}
```

### Example: Deposit

```bash
curl -s http://127.0.0.1:18102/api/deposit \
  -H "content-type: application/json" \
  -H "X-Api-Key: demo-api-key" \
  -H "X-Sim-Status: SUCCESS 2000" \
  -d '{
  "request_id": "req_778899",
  "partner_id": "partner_2001",
  "user_id": "user_1001",
  "game_id": "game_001",
  "game_account_id": "player_9001",
  "transaction_type": "deposit",
  "amount": 100.0,
  "game_coin": "GCA",
  "source": "test"
}'
```

Demo OAuth users: `alice_01` / `bob_02` (password `demo`).

---

## market-bot (`http://127.0.0.1:18103`)

Detail: [market-bot/README.md](market-bot/README.md)

Polls Core Engine books and places maker/taker orders (partial fill, full fill, rest). Talks to `CORE_ENGINE_URL` directly.

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Liveness |
| `GET` | `/v1/bot` | Status |
| `GET` | `/v1/bot/actions` | Recent actions |
| `POST` | `/v1/bot/pause` | Pause |
| `POST` | `/v1/bot/resume` | Resume |

```bash
curl -s http://127.0.0.1:18103/v1/bot
curl -s http://127.0.0.1:18103/v1/bot/actions
```

