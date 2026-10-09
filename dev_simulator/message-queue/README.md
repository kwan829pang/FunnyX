# message-queue

Local Rust API for inspecting and draining outbound notices during development. Mimics `fx_events.outbound_notices` (Message Center outbox) without PostgreSQL — in-memory only, `source=test` on views.

Aligned with [database/fx_events/01_tables.sql](../../database/fx_events/01_tables.sql) and [doc/dev.md](../../doc/dev.md) §2.

## Run

```bash
cd dev_simulator/message-queue
cp .env.sample .env
cargo run
```

Default listen: `http://127.0.0.1:18101`

## `X-Sim-Status` header

Every enqueue **always returns `delivery_status=pending`**. Use the header to schedule the final outcome + webhook:

```
X-Sim-Status: {STATUS} {ms}
```

| Example | After delay |
| --- | --- |
| _(omitted)_ or `PENDING` | Stay pending; no webhook |
| `SUCCESS 0` | → `sent` + webhook |
| `SUCCESS 3000` | after 3s → `sent` + webhook |
| `REJECTED 5000` | after 5s → `failed` + webhook |
| `CANCEL 2000` | after 2s → `failed` + webhook |

Webhook URL: request field `callback_url`, or env `NOTICE_CALLBACK_URL`.

```bash
curl -s http://127.0.0.1:18101/v1/notices \
  -H "content-type: application/json" \
  -H "X-Sim-Status: REJECTED 5000" \
  -d '{
  "end_user_id": 1,
  "source_type": "shop_order",
  "source_id": 5001,
  "event_type": "pending_payment",
  "title": "Shop order pending payment",
  "body": "Order 5001 awaiting partner fiat payment",
  "payload": {"shop_order_id": 5001},
  "callback_url": "http://127.0.0.1:8080/webhook/notice"
}'
```

## Endpoints

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/health` | Liveness |
| POST | `/v1/notices` | Enqueue (always pending; optional delayed sim + webhook) |
| GET | `/v1/notices` | List / filter |
| GET | `/v1/notices/stats` | Counts by delivery status |
| POST | `/v1/notices/drain` | Claim next pending batch → `sending` |
| GET | `/v1/notices/{id}` | Inspect one notice |
| POST | `/v1/notices/{id}/ack` | Mark `sent` or `failed` |
| POST | `/v1/notices/{id}/retry` | Re-queue a `failed` notice as `pending` |
| DELETE | `/v1/notices` | Clear all (test reset) |

`source_type` / `event_type` / `delivery_status` enums match the PostgreSQL CHECKs on `outbound_notices`.

## Env

See [`.env.sample`](.env.sample).
