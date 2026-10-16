# FunnyX demo accounts (local / Docker)

**Do not use these credentials in production.** They come from [`database/seed/`](database/seed/) (`00_seed_data.sql` + `01_demo_data.sql`) and the local simulators under [`dev_simulator/`](dev_simulator/).

Load seeds: full stack (`deploy/docker-compose.all.yml`) runs `seed/run_seed_dev.sql` automatically; otherwise see [`database/seed/README.md`](database/seed/README.md).

Default password for seeded logins with a null hash is **`demo`** (first login may persist an Argon2id hash).

---

## Admin Panel / Admin API

Login: Admin Panel → Admin API `POST /v1/admin/login` (default `http://127.0.0.1:18300`), not Gateway.

| Username | Password | Role | Notes |
| --- | --- | --- | --- |
| `seed_admin` | `demo` | `super_admin` | Full ops (markets, fees, Post_Auth, …) |
| `admin` | `demo` | `admin` | Ops writes allowed |
| `viewer` | `demo` | `viewer` | Read-limited; ops writes rejected |

Source: [`database/seed/00_seed_data.sql`](database/seed/00_seed_data.sql).

---

## Client Web / Session (platform end users) — 2 of 4 demo identities

Login via Gateway Session: `POST http://127.0.0.1:8080/v1/client/login` (or Client Center `8083`).

| Username | Password | Email | Notes |
| --- | --- | --- | --- |
| `demo_user` | `demo` | `demo_user@example.local` | Primary E2E user; market-bot login |
| `alice_plat` | `demo` | `alice_plat@example.local` | Second platform user |

### Bound game accounts

| Username | `game_accounts.id` | `game_account_id` | PLT | DEMO_COIN |
| --- | --- | --- | --- | --- |
| `demo_user` | `1` | `demo_player_9001` (`pu_9001`) | `100` | `50` |
| `alice_plat` | `2` | `demo_player_9002` (`pu_9002`) | `80` | `40` |

Game: `DEMO_GAME` / Demo Adventure (`partner_2001`).

Source: [`database/seed/01_demo_data.sql`](database/seed/01_demo_data.sql).

---

## Partner OAuth players (company-a-server)

Simulator: `http://127.0.0.1:18102` (OAuth provider for Path B / partner_fetch).

| Username | Password | Partner player id | Demo items |
| --- | --- | --- | --- |
| `alice_01` | `demo` | `player_9001` | `sword_01`, `shield_02` |
| `bob_02` | `demo` | `player_9002` | `potion_03` |

| Env / field | Value |
| --- | --- |
| `PARTNER_ID` | `partner_2001` |
| `API_KEY` (`X-Api-Key`) | `demo-api-key` |
| `OAUTH_CLIENT_ID` | `company_a_client` |
| `OAUTH_CLIENT_SECRET` | `company_a_secret` |
| Game id (partner) | `game_001` |

Source: [`dev_simulator/company-a-server`](dev_simulator/company-a-server).

---

## Corporate users (Client Web `/corp/login`) — 2 of 4 demo identities

Password `demo` (Client Center hashes NULL `password_hash` on boot). Login with **email** or **master_code**.

| Company | Email / master_code | Password | Master Id | API key |
| --- | --- | --- | --- | --- |
| Demo Game Partner Co. | `demo_corp@example.local` / `DEMO_MASTER_CODE` | `demo` | `DEMO_MASTER_ID` | `demo_api_key_do_not_use_live` |
| Second Studio Ltd. | `corp_b@example.local` / `DEMO_CORP_B_CODE` | `demo` | `DEMO_CORP_B_ID` | `demo_api_key_corp_b` |

## Corporate Master (Corp Server / Webhook Corp routes)

Headers for Master-authenticated Corp APIs (Client Center / Webhook). Session Bearer is **401** on these routes.

| Field | Corp A (default) | Corp B |
| --- | --- | --- |
| Company | Demo Game Partner Co. | Second Studio Ltd. |
| `X-Master-Account-Code` | `DEMO_MASTER_CODE` | `DEMO_CORP_B_CODE` |
| `X-Master-Id` | `DEMO_MASTER_ID` | `DEMO_CORP_B_ID` |
| `X-Api-Key` | `demo_api_key_do_not_use_live` | `demo_api_key_corp_b` |
| `X-Signature` (demo) | `demo` | `demo` |
| HMAC secret (env fallback) | `demo_secret_do_not_use_live` (`DEMO_MASTER_SECRET`) | same fallback for local |

HMAC-SHA256 hex of `{timestamp}.{master_id}` with `DEMO_MASTER_SECRET` is also accepted when not using the literal `demo` signature.

---

## Internal service key

| Env | Value | Used by |
| --- | --- | --- |
| `INTERNAL_API_KEY` / `SESSION_INTERNAL_API_KEY` | `demo-internal-key` | Config Server, STS, Gateway mesh, Admin/CC internal calls |

---

## Webhook Post_Auth / payment-gate

| Source | Domain / host | Auth key suffix | PPK (`X-Callback-Source`) | Body `signature` |
| --- | --- | --- | --- | --- |
| Payment gate | `payment-gate.local` | `demo_key` | `ppk_payment_gate` | `test-signature` |
| Partner 2001 | `api.company-a.local` / `*.company-a.local` | `demo_key` | `ppk_partner_2001` | `test-signature` |

`X-Post-Auth` = SHA-256 hex of `Post_Auth_{demo_key}`.

Docker full stack widens `allowed_ips` to `*` for mesh callbacks. Host-only seed keeps `127.0.0.1,::1`.

Payment gate URL: `http://127.0.0.1:18100`.

---

## Market bot

| Env | Value |
| --- | --- |
| `BOT_USERNAME` | `demo_user` |
| `BOT_PASSWORD` | `demo` |
| `GAME_ACCOUNT_ID` | `1` |
| Symbol (full stack) | `DEMO_COIN/PLT` |

Health: `http://127.0.0.1:18103/health`. Session place needs the market **Admin-approved** (`pending_locked` after seed).

---

## Demo catalog (reference)

| Item | Value |
| --- | --- |
| Partner id | `partner_2001` |
| Platform token | `PLT` |
| Company coin | `DEMO_COIN` |
| Shop packages | `PLT_1000`, `PLT_1500`, `PLT_3000`, `PLT_10000` |
| Corp shop product | `DEMO_PACK_100` |
| Market pair | `DEMO_COIN/PLT` (seed status `pending_locked`) |
| Core Engine quote (full stack) | `PLT` |

---

## Quick login curls

```bash
# Admin
curl -s http://127.0.0.1:18300/v1/admin/login \
  -H 'content-type: application/json' \
  -d '{"username":"seed_admin","password":"demo"}'

# Client Session via Gateway
curl -s http://127.0.0.1:8080/v1/client/login \
  -H 'content-type: application/json' \
  -d '{"username":"demo_user","password":"demo"}'

# Corp portal (Client Center / Gateway)
curl -s http://127.0.0.1:8080/v1/corp/login \
  -H 'content-type: application/json' \
  -d '{"login":"demo_corp@example.local","password":"demo"}'
```
