# FunnyX Webhook Server

Partner **ingress** for e-shop, CBT, and deposit/withdrawal callbacks. Spec: [doc/webhook.md](../../doc/webhook.md). Client Web is **not** served here — notices go to `fx_events.outbound_notices` for Message Center / Gateway later. Until then, Client Web polls order/txn status on Client Center.

Default HTTP **8084**, socket ingest **9004**.

## Run

```bash
cd backend/webhook_server
cp .env.sample .env
cargo run
```

`.env` overrides inherited `HTTP_PORT` so this process stays on **8084**. Point Client Center `SHOP_PAYMENT_CALLBACK_URL` at `http://127.0.0.1:8084/v1/webhook/shop/payment`.

Without `POSTGRES_URL`, events / Corp endpoints / Post_Auth rows stay in memory (`POST_AUTH_DEMO_SOURCES`). Duplicate `event_id` is still a 200 no-op.

## HTTP Post_Auth

Inbound shop / CBT / deposit / withdrawal require:

1. `X-Post-Auth` = SHA-256 hex of `Post_Auth_{auth_key_suffix}` (demo suffix `demo_key`)
2. Optional `X-Callback-Source` = real IP, auth-key, or ppk — else `Host` wild-domain match
3. Caller IP in that row’s `allowed_ips`
4. Body `signature` = `CALLBACK_SIGNATURE`

Socket ingest: body `signature` only (no Post_Auth headers in v1).

## APIs

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| `GET` | `/health` | Public | Liveness |
| `POST` | `/v1/webhook/shop/payment` | Post_Auth + `signature` | Persist + settle via Client Center |
| `POST` | `/v1/webhook/corp-token/payment` | Post_Auth + `signature` | CBT settle via Client Center |
| `POST` | `/v1/webhook/deposit` | Post_Auth + `signature` | Money settle (`transaction_type=deposit`) |
| `POST` | `/v1/webhook/withdrawal` | Post_Auth + `signature` | Money settle (`transaction_type=withdrawal`) |
| `GET`/`POST` | `/v1/webhook/endpoints` | MasterSigned Corp | List / create own callback rows |
| `PUT` | `/v1/webhook/endpoints/{id}` | MasterSigned Corp | Update own row |
| `GET` | `/v1/webhook/events/{event_id}` | `X-Internal-Key` | Debug |

Session Bearer on Corp endpoint routes is **401**. Demo Master: `DEMO_MASTER_CODE` / `DEMO_MASTER_ID` / `demo_api_key_do_not_use_live`; `X-Signature: demo` or HMAC-SHA256 hex of `{timestamp}.{master_id}` with `DEMO_MASTER_SECRET`.

```bash
curl -s http://127.0.0.1:8084/health

POST_AUTH=$(printf 'Post_Auth_demo_key' | openssl dgst -sha256 -hex | awk '{print $2}')

curl -s http://127.0.0.1:8084/v1/webhook/shop/payment \
  -H "content-type: application/json" \
  -H "x-post-auth: $POST_AUTH" \
  -H "x-callback-source: ppk_payment_gate" \
  -d '{"event_id":"pay_evt_1","partner_order_no":"PAY-1","shop_order_id":1,"status":"paid","fiat_currency":"HKD","fiat_paid":88,"signature":"test-signature"}'

curl -s http://127.0.0.1:8084/v1/webhook/deposit \
  -H "content-type: application/json" \
  -H "x-post-auth: $POST_AUTH" \
  -H "x-callback-source: ppk_partner_2001" \
  -d '{"event_id":"money_evt_1","request_id":"req_1","partner_txn_id":"PTXN1","transaction_type":"deposit","status":"success","amount":10,"game_coin":"PLT","signature":"test-signature"}'

curl -s http://127.0.0.1:8084/v1/webhook/endpoints \
  -H "x-master-account-code: DEMO_MASTER_CODE" \
  -H "x-master-id: DEMO_MASTER_ID" \
  -H "x-api-key: demo_api_key_do_not_use_live" \
  -H "x-signature: demo" \
  -H "x-timestamp: 1" \
  -H "content-type: application/json" \
  -d '{"kind":"shop_payment","callback_url":"https://partner.example.com/cb"}'
```

Socket: length-prefixed frames (`funnyx-socket`). `PING` → `PONG`. Ingest JSON with `event_type` `shop_payment` | `corp_token` | `deposit` | `withdrawal` and a `body` object.

Failed Client Center settle retries with backoff 5s / 30s / 2m / 10m, then `dead`.
