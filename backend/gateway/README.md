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
- Public: `GET /health` — includes `upstreams` role summary; `status=degraded` when CC/STS peers are all down
- Proxies: `/v1/client/*`, `/v1/shop/*`, `/v1/corp/*`, `/v1/notifications*`, `/v1/admin/*`, …
- Blocked: `/v1/internal/*`, `/v1/config/*`, `/v1/webhook/*`, `/v1/notices*`, `/v1/session/validate`, `/v1/engine*`

### Upstream failover (§20) — socket heartbeat, not HTTP `/health`

Routing health comes from **Config Server private socket PING/PONG** (`GET /v1/config/device-status` after Config probes each registered `socket_url`). Gateway does **not** call upstream `GET /health` for failover.

1. Each service (including this Gateway) registers with Config Server with `socket_url`.
2. Config Server probes sockets; Gateway imports that status into its in-memory map.
3. Proxy RR: prefer `Healthy`, then `Degraded`; **503** if a role has only Unhealthy peers.
4. Same-role `failover_map` steers to backup when primary is down.
5. **Gateway mesh:** other Gateways from Config listing are pinged over their sockets; peers appear on `GET /health` as `gateway_peers` (reporting only).

Env: `INSTANCE_ID`, `SOCKET_ADVERTISE_URL` (reachable `tcp://host:port` for peers/Config), `PEER_PING_TIMEOUT_MS`, `REGISTRY_REFRESH_MS`. Admin proxy still checks Config whitelist IP (unless `*` / empty).

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
