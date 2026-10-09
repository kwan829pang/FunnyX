# FunnyX Config Server

Operational config: JSON **IP whitelist** load/reload, in-memory **service registry**, HTTP heartbeat, `/health`.

Default port: **8090** (`funnyx-config::DEFAULT_CONFIG_HTTP_PORT`). Single instance; no DB.

## Run

From `backend/config_server` so the sample whitelist path resolves:

```bash
cd backend/config_server
cp .env.sample .env
cargo run
```

`WHITELIST_PATH` defaults to `../../config/whitelist.sample.json`. `.env` overrides inherited `HTTP_PORT` so this process stays on **8090**.

## APIs

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| `GET` | `/health` | Public | Liveness |
| `POST` | `/reload` | `X-Internal-Key` or allowlisted IP | Reload whitelist JSON |
| `GET` | `/v1/config/whitelist` | Internal / allowlist | Current allowlist snapshot |
| `GET` | `/v1/config/services` | Internal / allowlist | Registered instances |
| `POST` | `/v1/config/services` | Internal / allowlist | Register Gateway / Core Engine / … |
| `POST` | `/v1/config/services/heartbeat` | Internal / allowlist | Refresh last-seen |
| `GET` | `/v1/config/device-status` | Internal / allowlist | Instance health vs heartbeat timeout |

```bash
KEY=demo-internal-key

curl -s http://127.0.0.1:8090/health

curl -s http://127.0.0.1:8090/v1/config/whitelist -H "x-internal-key: $KEY"

curl -s http://127.0.0.1:8090/v1/config/services \
  -H "x-internal-key: $KEY" -H "content-type: application/json" \
  -d '{"service_name":"gateway","http_url":"http://127.0.0.1:8080","group":"ingress"}'

curl -s http://127.0.0.1:8090/v1/config/services/heartbeat \
  -H "x-internal-key: $KEY" -H "content-type: application/json" \
  -d '{"instance_id":"<id from register>"}'

curl -s http://127.0.0.1:8090/v1/config/device-status -H "x-internal-key: $KEY"

curl -s -X POST http://127.0.0.1:8090/reload -H "x-internal-key: $KEY"
```

Socket heartbeat (PING/PONG) is deferred; HTTP register/heartbeat is the registry path for Gateway snapshots.
