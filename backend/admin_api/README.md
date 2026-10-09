# FunnyX Admin API

Admin Panel backend. After Admin API verifies credentials, **Session Token Server** issues an admin session (`actor_type=admin`, scope includes `admin`).

Default port: **18300**. Requires STS on **8082**.

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
| `GET` | `/v1/admin/shop/corp-products` | Bearer (admin) | List Corp e-shop products (any status; query `status`, `corporate_user_id`) |
| `GET` | `/v1/admin/shop/corp-products/{id}` | Bearer (admin) | Corp product detail |
| `PATCH` | `/v1/admin/shop/corp-products/{id}/status` | Bearer (admin) | Set `active` / `inactive` / `archived` (suspend) |

Demo admins: `seed_admin` / `admin` (password `demo`).

```bash
TOKEN=$(curl -s http://127.0.0.1:18300/v1/admin/login \
  -H "content-type: application/json" \
  -d '{"username":"seed_admin","password":"demo"}' | jq -r .access_token)

curl -s http://127.0.0.1:18300/v1/admin/shop/corp-products \
  -H "authorization: Bearer $TOKEN"

curl -s -X PATCH http://127.0.0.1:18300/v1/admin/shop/corp-products/1/status \
  -H "authorization: Bearer $TOKEN" -H "content-type: application/json" \
  -d '{"status":"inactive"}'
```

Without `POSTGRES_URL`, a demo product `DEMO_PACK_100` is in memory. Set `POSTGRES_URL` to list/suspend rows in `fx_shop.corp_shop_products`. Status changes write `fx_admin.admin_action_logs` when Postgres is up.

Corp **create** of products is `/v1/corp/shop/products` (not this service).

## Env

See [`.env.sample`](.env.sample).
