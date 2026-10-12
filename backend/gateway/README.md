# Gateway

Public HTTP/socket ingress for FunnyX. Reverse-proxies Client Web and Company Partner (MasterSigned) traffic to Client Center / Message Center after auth gates. Session Bearer status is checked on **Session Token Server** directly. **Admin Panel** calls Admin API directly (Gateway `/v1/admin/*` proxy is optional, not required).

Spec: [doc/gateway_core_engine_logic.md](../../doc/gateway_core_engine_logic.md) §2. API index: [api-master.md](../../api-master.md).

## Run

```bash
cd backend/gateway
cp .env.sample .env
cargo run
```

- HTTP: `8080`
- Heartbeat + client socket: `9000`
- Public: `GET /health`
- Proxies: `/v1/client/*`, `/v1/shop/*`, `/v1/corp/*`, `/v1/notifications*`, `/v1/admin/*`, …
- Blocked: `/v1/internal/*`, `/v1/config/*`, `/v1/webhook/*`, `/v1/notices*`, `/v1/session/validate`, `/v1/engine*`

## Auth

| Caller | Credential | Gateway action |
| --- | --- | --- |
| Client Web (Public) | none | Forward register/login/OAuth |
| Client Web (Session) | Bearer STS token | Validate via Session Token Server, then forward |
| Company Partner | MasterSigned headers | Require headers present, forward to CC |
| Admin Panel | Bearer admin token | Prefer **direct Admin API**; optional Gateway proxy still validates Bearer then forwards |

## Client socket

Connect to `tcp://127.0.0.1:9000`, send a JSON line:

```json
{"access_token":"sts_…"}
```

On success the Gateway replies `{"ok":true,"connection_id":"…"}` and keeps the connection for push fan-out.

**Browser Client Web:** `ws://127.0.0.1:8080/v1/ws` — first text frame `{"access_token":"sts_…"}`, then JSON notification pushes.

**Internal push (Message Center):** `POST /v1/internal/socket/push` with `X-Internal-Key` and body `{ end_user_id, notification_id, source_type, event_type, title?, body?, payload? }`.

Config Server PING/PONG frames on socket `9000` are answered with PONG.
