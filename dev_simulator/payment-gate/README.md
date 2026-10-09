# payment-gate

Local Rust API that simulates a partner **fiat payment** endpoint and **deposit** callbacks for FunnyX development. All records are marked `source=test` and kept in memory (no database).

Aligned with [doc/e-shop.md](../../doc/e-shop.md) §7 and [doc/partner.md](../../doc/partner.md) §3 / §9.

## Run

```bash
cd dev_simulator/payment-gate
cp .env.sample .env
cargo run
```

Default listen: `http://127.0.0.1:18100`

## `X-Sim-Status` header

Every create/submit **always returns `status=pending`**. Use the header to schedule the final outcome:

```
X-Sim-Status: {STATUS} {ms}
```

| Example | Behavior |
| --- | --- |
| _(omitted)_ or `PENDING` | Stay pending; no webhook |
| `SUCCESS 0` | Immediately (async) → shop `paid` / deposit `success` + webhook |
| `SUCCESS 3000` | After 3s → paid/success + webhook |
| `REJECTED 5000` | After 5s → shop `failed` / deposit `rejected` + webhook |
| `CANCEL 2000` | After 2s → `cancelled` + webhook |

Response includes `sim_status`, `sim_delay_ms`, and echoes the header. Poll `GET /v1/shop/payment/{partner_order_no}` after the delay to see the final status.

```bash
curl -s http://127.0.0.1:18100/v1/shop/payment \
  -H "content-type: application/json" \
  -H "X-Sim-Status: REJECTED 5000" \
  -d '{ ... }'
# → status: pending now; callback fires after 5000ms
```

## Endpoints

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/health` | Liveness |
| POST | `/v1/shop/payment` | Create shop fiat payment (always pending) |
| GET | `/v1/shop/payment/{partner_order_no}` | Inspect payment |
| POST | `/v1/shop/payment/{partner_order_no}/complete` | Schedule/manual complete + webhook |
| POST | `/v1/deposit/payment` | Create deposit (always pending) |
| GET | `/v1/deposit/payment/{partner_order_no}` | Inspect deposit |
| POST | `/v1/deposit/payment/{partner_order_no}/complete` | Schedule/manual complete + webhook |
| GET | `/v1/payments` | List all in-memory payments |
| GET | `/pay/{partner_order_no}` | Tiny HTML checkout helper |

### Create shop payment

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
  "package_code": "PLT_1000",
  "credit_game_coin": "PLT",
  "credit_amount": 1000,
  "fiat_currency": "HKD",
  "fiat_price": 88.00,
  "callback_url": "http://127.0.0.1:8080/webhook/shop/payment",
  "return_url": "http://127.0.0.1:3000/shop/orders/5001"
}'
```

Webhook body matches e-shop callback JSON (`event_id`, `partner_order_no`, `fiat_paid`, `signature`, `source=test`).

## Env

See [`.env.sample`](.env.sample). `CALLBACK_SIGNATURE` is echoed on webhook payloads for local auth stubs.
