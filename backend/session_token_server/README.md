# FunnyX Session Token Server

Issues **Client Web** session tokens and hosts **OAuth 2.0** for all parties:

1. **Platform OAuth IdP** — Partners login users already registered on the platform  
2. **Partner OAuth broker** — Client Web verifies users via Partner OAuth, then STS issues a platform session  
3. **Password login / register** — local demo identity until Client Center owns accounts  

Design: [doc/client_connect.md](../../doc/client_connect.md), [api-master.md](../../api-master.md), UML [doc/partner-uml.md](../../doc/partner-uml.md).

Default port: **8082**.

## Run

```bash
cd backend/session_token_server
cp .env.sample .env
cargo run
```

With Partner Path B, also run `dev_simulator/company-a-server` on `18102`.

## APIs

| Method | Path | Auth | Purpose |
| --- | --- | --- | --- |
| `GET` | `/health` | — | Liveness |
| `POST` | `/v1/client/register` | Public | Register demo end user + session |
| `POST` | `/v1/client/login` | Public | Password login → session |
| `POST` | `/v1/session/token` | Public | Issue / refresh session |
| `POST` | `/v1/session/validate` | Internal | Gateway validate |
| `POST` | `/v1/session/revoke` | Bearer / Internal | Logout |
| `GET` | `/v1/oauth/authorize` | Public | Platform OAuth authorize (Partner → platform) |
| `GET/POST` | `/v1/oauth/login` | Public | Platform user consent form |
| `POST` | `/v1/oauth/token` | OAuth client | Exchange code → OAuth access token |
| `GET` | `/v1/oauth/userinfo` | Bearer OAuth | Platform userinfo |
| `GET` | `/v1/client/oauth/partner/{partner_id}/start` | Public | Start Partner OAuth |
| `GET` | `/v1/client/oauth/partner/callback` | Public | Partner callback → platform session |

Demo users: `demo_user` / `alice_plat` (password `demo`).

### Password login

```bash
curl -s http://127.0.0.1:8082/v1/client/login \
  -H "content-type: application/json" \
  -d '{"username":"demo_user","password":"demo"}'
```

### Platform OAuth (Partner logs in platform user)

1. Partner redirects browser to:

```text
GET /v1/oauth/authorize?response_type=code&client_id=platform_partner_client&redirect_uri=http://127.0.0.1:3000/oauth/callback&state=xyz
```

2. User signs in on `/v1/oauth/login` → redirect with `code`.
3. Partner exchanges:

```bash
curl -s http://127.0.0.1:8082/v1/oauth/token \
  -H "content-type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code&code=ocode_...&client_id=platform_partner_client&client_secret=platform_partner_secret&redirect_uri=http://127.0.0.1:3000/oauth/callback"
```

4. `GET /v1/oauth/userinfo` with `Authorization: Bearer oat_…`.

### Partner OAuth (Client Web Path B)

```bash
# Browser: start (omit redirect_uri to get JSON on callback for smoke)
open "http://127.0.0.1:8082/v1/client/oauth/partner/partner_2001/start"

# Or with Client Web return URL:
open "http://127.0.0.1:8082/v1/client/oauth/partner/partner_2001/start?redirect_uri=http://127.0.0.1:3000/oauth/callback"
```

Flow: STS → company-a `/oauth/authorize` → login (`alice_01` / `demo`) → STS callback exchanges code + userinfo → links/creates end user → issues `sts_…` session (JSON or redirect).

## Env

See [`.env.sample`](.env.sample) for `PLATFORM_OAUTH_*`, `PARTNER_A_*`, `CLIENT_WEB_REDIRECT`, and Redis options.
