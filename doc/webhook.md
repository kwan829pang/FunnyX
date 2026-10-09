# Webhook Server

This doc is a detail file for [project.md](project.md) §5 (Webhook Server). HTTP path index: [api-master.md](../api-master.md). E-shop callback JSON: [e-shop.md](e-shop.md) §7.3. Socket framing: [socket_message.md](socket_message.md). Crate: [`backend/webhook_server`](../backend/webhook_server/).

**Detail scope:** Partner ingress (HTTP + S2S socket), Corp endpoint rows, idempotent shop settle into Client Center, retry, and which caller connects where. Not Message Center drain, not Gateway public sockets, not Core Engine.

## 1. Role

Webhook Server is **partner / payment-gate ingress** only.

- Accepts **inbound** shop payment and **Company Basic Token** payment callbacks (HTTP; shop also S2S socket).
- Persists `fx_events.webhook_events` (`event_id` unique, `retry_count`) and shop audit `fx_shop.shop_payment_events`.
- Calls Client Center **internal** settle so orders/wallets update (`/v1/internal/shop/settle`, `/v1/internal/corp-token/settle`).
- Writes `fx_events.outbound_notices` (`shop_order` / `corp_token_order`) for Message Center later; on CBT paid also Corp `corp_partner_notices` and HTTP **balance callback** to Corp (`webhook_endpoints` kind `corp_token` or `partner_endpoints` type `callback`).
- Lets **Corporate Users** create/update **their** callback endpoint rows (MasterSigned). CBT paid settle **does** call the Corp callback URL when configured.

Client Web does **not** open a socket to Webhook Server. Until Message Center + Gateway exist, the browser polls Client Center `GET /v1/shop/orders/{id}` (Session). After those services exist, Client Web uses **Gateway** socket (Session token); Message Center drains notices.

Default local listen: HTTP **8084**, socket **9004** (`funnyx-config`: `DEFAULT_WEBHOOK_HTTP_PORT` / `DEFAULT_WEBHOOK_SOCKET_PORT`). Client Center `SHOP_PAYMENT_CALLBACK_URL` must be `http://127.0.0.1:8084/v1/webhook/shop/payment`.

## 2. Socket connection targets

Who opens a **TCP socket**, to which process, and what to send. Length-prefixed frames (`funnyx-socket`); `PING` → `PONG`. Ingest v1: gzip JSON or UTF-8 JSON (Complex Command envelope). Do not treat Webhook **9004** as a Client Web Session socket.

| Caller | Connects to (socket target) | Dev host:port | What to send | Reply |
| --- | --- | --- | --- | --- |
| payment-gate / Partner payment backend | **Webhook Server ingest** | `127.0.0.1:9004` | `PING`; or ingest JSON (`event_type` + body, same fields as HTTP shop callback) | `PONG`; JSON `{ accepted, duplicate, event_id, delivery_status }` or `{ error }` |
| Config Server registry probe | **Webhook Server ingest** | `127.0.0.1:9004` | `PING` | `PONG` |
| Client Web | **Gateway** (not Webhook) | `127.0.0.1:9000` (Gateway; not started) | Session token + client gzip JSON | Shop order `completed` / `failed` after Message Center drains notices |
| Client Center | — | no socket to Webhook | HTTP only (Webhook → CC settle) | — |
| Message Center | — | no socket to Webhook | HTTP/DB drain of `outbound_notices` (later) | Client Web via Gateway |
| Admin Panel / Admin API | — | do not use `:9004` | — | — |
| Corporate User browser | — | do not use `:9004` | Corp endpoint CRUD is **HTTP MasterSigned**, not socket | — |

**Rule:** Socket **9004** = inbound partner/S2S events into FunnyX. Socket **9000** (Gateway) = Client Web realtime. Mixing them is out of spec.

## 3. Which target to call which API

Pick the **target service** first, then the path. Webhook Server does not replace Client Center shop Session APIs.

| Intent | Call this target | Transport | Method / message | Path or envelope | Auth |
| --- | --- | --- | --- | --- | --- |
| Partner: shop fiat paid/failed/cancelled | **Webhook Server** | HTTP | `POST` | `/v1/webhook/shop/payment` | Body `signature` = `CALLBACK_SIGNATURE` (dev `test-signature`) |
| Partner / payment-gate: same shop event over socket | **Webhook Server** | Socket `:9004` | Ingest frame | `{ "event_type": "shop_payment", "body": { …same as HTTP… } }` or flattened shop JSON | Same `signature` field in JSON |
| Partner: liveness | **Webhook Server** | HTTP | `GET` | `/health` | Public |
| Partner: socket liveness | **Webhook Server** | Socket `:9004` | `PING` | — | None |
| Corp: list own callback URLs | **Webhook Server** | HTTP | `GET` | `/v1/webhook/endpoints` | MasterSigned (`X-Master-Account-Code`, `X-Master-Id`, `X-Api-Key`, `X-Signature`, `X-Timestamp`). **No** Session Bearer |
| Corp: create/upsert own callback URL | **Webhook Server** | HTTP | `POST` | `/v1/webhook/endpoints` | MasterSigned |
| Corp: update own callback URL | **Webhook Server** | HTTP | `PUT` | `/v1/webhook/endpoints/{id}` | MasterSigned |
| Ops/debug: one inbound event | **Webhook Server** | HTTP | `GET` | `/v1/webhook/events/{event_id}` | `X-Internal-Key` |
| FunnyX: apply wallet/order after accept | **Client Center** | HTTP (server-to-server) | `POST` | `/v1/internal/shop/settle` | `X-Internal-Key` (Webhook Server calls this; Partners do not) |
| FunnyX: apply CBT order after accept | **Client Center** | HTTP (server-to-server) | `POST` | `/v1/internal/corp-token/settle` | `X-Internal-Key` |
| Partner: Company Basic Token buy paid/failed/cancelled | **Webhook Server** | HTTP | `POST` | `/v1/webhook/corp-token/payment` | Body `signature` = `CALLBACK_SIGNATURE` |
| FunnyX → Corp: user balance updated after CBT paid | **Corp User server** | HTTP | `POST` | Corp `callback_url` / `partner_endpoints.callback` | Outbound from Webhook Server |
| Client Web: catalog / checkout / order status | **Client Center** (via Gateway when live) | HTTP | `GET`/`POST` | `/v1/shop/*` | Session Bearer |
| Client Web: CBT buyable list / orders | **Client Center** | HTTP | `GET`/`POST` | `/v1/corp-tokens*` | Session Bearer |
| Client Web: “payment done” until MC exists | **Client Center** | HTTP | `GET` | `/v1/shop/orders/{id}` or `/v1/corp-tokens/orders/{order_id}` | Session Bearer |
| Client Web: “payment done” after MC + Gateway | **Gateway** | Socket `:9000` | Session messages | shop_order / corp_token_order `completed` / `failed` | Session token |
| payment-gate: create checkout (not webhook) | **payment-gate** | HTTP | `POST` | payment-gate create APIs (`18100`) | Simulator contract; `callback_url` must point at Webhook `/v1/webhook/shop/payment` |
| Reserved: deposit / withdrawal / generic | **Webhook Server** (later) | HTTP | `POST` | `/v1/webhook/deposit`, `/v1/webhook/withdrawal`, `/v1/webhook/partner/callback` | Partner; not registered yet |

Duplicate `event_id` on shop ingest: HTTP **200** no-op (no second settle). Socket ingest uses the same persist path.

## 4. Socket ingest payload

v1 envelope (recommended):

```json
{
  "event_type": "shop_payment",
  "body": {
    "event_id": "pay_evt_8899",
    "partner_order_no": "PAY-7788",
    "shop_order_id": 5001,
    "seller_type": "platform",
    "status": "paid",
    "fiat_currency": "HKD",
    "fiat_paid": 88.00,
    "credit_game_coin": "PLT",
    "credit_amount": 1000,
    "paid_at": "2026-09-28T01:05:00Z",
    "signature": "test-signature"
  }
}
```

`status`: `paid` | `failed` | `cancelled`. Full field rules: [e-shop.md](e-shop.md) §7.3–§8.

| `event_type` | Handled now | Maps to HTTP |
| --- | --- | --- |
| `shop_payment` | Yes | `POST /v1/webhook/shop/payment` |
| `deposit` / `withdrawal` / `corp_token` / other | No (error on socket) | Reserved HTTP paths |

## 5. Settlement path

```text
Partner/payment-gate  --HTTP POST or socket ingest-->  Webhook Server :8084 / :9004
Webhook Server        --INSERT webhook_events (+ shop_payment_events for shop)-->  PostgreSQL
Webhook Server        --POST /v1/internal/shop/settle | /v1/internal/corp-token/settle-->  Client Center :8083
Client Center         --update orders, wallets (CBT: 99.9% credit + fee ledger)-->  PostgreSQL
Webhook Server        --INSERT outbound_notices (pending)-->  PostgreSQL
Webhook Server (CBT)  --POST balance payload-->  Corp callback_url (kind corp_token / partner callback)
Message Center (:8085)--drain notices-->  notifications + partner POST; Gateway socket later
```

### CBT payment body (HTTP)

```json
{
  "event_id": "cbt_evt_1001",
  "partner_order_no": "CBT-7788",
  "corp_token_order_id": 7001,
  "status": "paid",
  "coin_amount": 1000,
  "token_code": "GAMECOIN",
  "signature": "test-signature"
}
```

Retry on Client Center failure: `retry_count++`, `next_retry_at` backoff **5s / 30s / 2m / 10m**, then `dead`. Memory-only Client Center still updates in-process order status; wallet rows need Postgres.

## 6. Corp endpoint rows

`fx_events.webhook_endpoints`: `UNIQUE (corporate_user_id, kind)`. `kind`: `shop_payment` | `deposit` | `withdrawal` | `corp_token`. Store `secret_hint` only, never raw Secret.

Demo Master (no Postgres): `DEMO_MASTER_CODE` / `DEMO_MASTER_ID` / `demo_api_key_do_not_use_live`; `X-Signature: demo` or HMAC-SHA256 hex of `{timestamp}.{master_id}` with `DEMO_MASTER_SECRET`. With `POSTGRES_URL`, credentials must match `fx_corp.corporate_users` + `corp_api_keys`.

## Related

| Doc | Use |
| --- | --- |
| [e-shop.md](e-shop.md) | Catalog, checkout JSON, settlement rules |
| [client_connect.md](client_connect.md) | Session vs MasterSigned |
| [socket_message.md](socket_message.md) | Frame / gzip / PING |
| [api-master.md](../api-master.md) | HTTP tables |
| [system_type.md](system_type.md) | Server type enum (Webhook = group 3.2) |
