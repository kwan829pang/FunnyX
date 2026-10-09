# FunnyX Admin API

Admin Panel backend. After Admin API verifies credentials, **Session Token Server** issues an admin session (`actor_type=admin`, scope includes `admin`).

Default port: **18300**. Requires STS on **8082**. Public ingress: [Gateway](../gateway/) on **8080**.

## Run

```bash
# Terminal 1
cd backend/session_token_server && cargo run

# Terminal 2
cd backend/admin_api
cp .env.sample .env
cargo run
```

## Auth flow

```text
Admin Panel
  → POST /v1/admin/login  (Admin API verifies username/password)
  → Session Token Server issues sts_… (grant_type=admin_login, actor_type=admin)
  → Admin Panel uses Bearer on Admin APIs
  → GET /v1/admin/me validates token via STS, then loads admin profile
```

## APIs

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| `GET` | `/health` | — | Liveness |
| `POST` | `/v1/admin/login` | Public | Verify admin → STS token |
| `POST` | `/v1/admin/logout` | Bearer / body | Revoke via STS |
| `GET` | `/v1/admin/me` | Bearer | Profile after STS validate |
| `GET`/`POST` | `/v1/admin/game-coins` | Bearer | List / create Game Partner Game Coin |
| `PUT` | `/v1/admin/game-coins/{id}` | Bearer | Update name / asset_kind |
| `PATCH` | `/v1/admin/game-coins/{id}/status` | Bearer | `active` / `inactive` |
| `GET` | `/v1/admin/markets` | Bearer | List market pairs (+ pool/lock); `?status=` |
| `GET` | `/v1/admin/markets/{id}` | Bearer | Market detail |
| `POST` | `/v1/admin/markets/{id}/approve` | Bearer | Transfer lock → pool; activate; Core Engine `POST /v1/admin/pairs` |
| `POST` | `/v1/admin/markets/{id}/reject` | Bearer | Unlock client lock; reject pair |
| `GET` | `/v1/admin/shop/corp-products` | Bearer | List Corp e-shop products |
| `GET` | `/v1/admin/shop/corp-products/{id}` | Bearer | Corp product detail |
| `PATCH` | `/v1/admin/shop/corp-products/{id}/status` | Bearer | Set `active` / `inactive` / `archived` |
| `GET`/`POST`/`PUT`/`PATCH` | `/v1/admin/shop/packages*` | Bearer | PLT packages |
| `GET`/`POST`/`PUT` | `/v1/admin/system/*` | Bearer | Setup wizard + base currency |
| `GET`/`POST` | `/v1/admin/basic-tokens*` | Bearer | CBT approve / reject |

Demo admins: `seed_admin` / `admin` (password `demo`).

```bash
TOKEN=$(curl -s http://127.0.0.1:18300/v1/admin/login \
  -H "content-type: application/json" \
  -d '{"username":"seed_admin","password":"demo"}' | jq -r .access_token)

curl -s http://127.0.0.1:18300/v1/admin/game-coins -H "authorization: Bearer $TOKEN"
curl -s "http://127.0.0.1:18300/v1/admin/markets?status=pending_locked" \
  -H "authorization: Bearer $TOKEN"
curl -s -X POST http://127.0.0.1:18300/v1/admin/markets/1/approve \
  -H "authorization: Bearer $TOKEN"
```

Market approve resolves Core Engine via Config Server registry (`CONFIG_SERVER_URL`) or `CORE_ENGINE_URL` / `CORE_ENGINE_ADMIN_KEY` fallback.

Without `POSTGRES_URL`, game-coins + a demo pending market run in memory. Set `POSTGRES_URL` for shared `fx_game` / `fx_market` with Client Center.

## Env

See [`.env.sample`](.env.sample).
