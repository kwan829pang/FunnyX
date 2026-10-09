# Gateway

Public HTTP/socket ingress for FunnyX. Reverse-proxies Client Web, Company Partner (MasterSigned), and Admin traffic to Client Center / Message Center / Admin API after auth gates.

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
| Client Web (Session) | Bearer STS token | Validate via STS, then forward |
| Company Partner | MasterSigned headers | Require headers present, forward to CC |
| Admin Panel | Bearer admin token | Require Bearer, forward to Admin API |

## Client socket

Connect to `tcp://127.0.0.1:9000`, send a JSON line:

```json
{"access_token":"sts_…"}
```

On success the Gateway replies `{"ok":true,"connection_id":"…"}` and keeps the connection in the in-memory registry (Message Center push fan-out is a follow-up). Config Server PING/PONG frames on the same port are answered with PONG.
