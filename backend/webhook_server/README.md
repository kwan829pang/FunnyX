# FunnyX Webhook Server

Partner **ingress** for e-shop payment callbacks. Spec: [doc/webhook.md](../../doc/webhook.md) (socket targets + which API to call). Client Web is **not** served here — notices go to `fx_events.outbound_notices` for Message Center / Gateway later. Until then, Client Web polls `GET /v1/shop/orders/{id}` on Client Center.

Default HTTP **8084**, socket ingest **9004**.

## Run

```bash
cd backend/webhook_server
cp .env.sample .env
cargo run
```

`.env` overrides inherited `HTTP_PORT` so this process stays on **8084**. Point Client Center `SHOP_PAYMENT_CALLBACK_URL` at `http://127.0.0.1:8084/v1/webhook/shop/payment`.

Without `POSTGRES_URL`, events and Corp endpoint rows stay in memory. Duplicate `event_id` is still a 200 no-op.

## APIs

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| `GET` | `/health` | Public | Liveness |
| `POST` | `/v1/webhook/shop/payment` | Partner `signature` field (`CALLBACK_SIGNATURE`, default `test-signature`) | Persist + settle via Client Center |
| `GET`/`POST` | `/v1/webhook/endpoints` | MasterSigned Corp | List / create own callback rows |
| `PUT` | `/v1/webhook/endpoints/{id}` | MasterSigned Corp | Update own row |
| `GET` | `/v1/webhook/events/{event_id}` | `X-Internal-Key` | Debug |

Session Bearer on Corp endpoint routes is **401**. Demo Master: `DEMO_MASTER_CODE` / `DEMO_MASTER_ID` / `demo_api_key_do_not_use_live`; `X-Signature: demo` or HMAC-SHA256 hex of `{timestamp}.{master_id}` with `DEMO_MASTER_SECRET`.

```bash
curl -s http://127.0.0.1:8084/health

curl -s http://127.0.0.1:8084/v1/webhook/shop/payment \
  -H "content-type: application/json" \
  -d '{"event_id":"pay_evt_1","partner_order_no":"PAY-1","shop_order_id":1,"status":"paid","fiat_currency":"HKD","fiat_paid":88,"signature":"test-signature"}'

curl -s http://127.0.0.1:8084/v1/webhook/endpoints \
  -H "x-master-account-code: DEMO_MASTER_CODE" \
  -H "x-master-id: DEMO_MASTER_ID" \
  -H "x-api-key: demo_api_key_do_not_use_live" \
  -H "x-signature: demo" \
  -H "x-timestamp: 1" \
  -H "content-type: application/json" \
  -d '{"kind":"shop_payment","callback_url":"https://partner.example.com/cb"}'
```

Socket: length-prefixed frames (`funnyx-socket`). `PING` → `PONG`. Ingest JSON (plain or gzip) with the same shop callback fields, optionally wrapped as `{ "event_type": "shop_payment", "body": { ... } }`.

Failed Client Center settle retries with backoff 5s / 30s / 2m / 10m, then `dead`.
