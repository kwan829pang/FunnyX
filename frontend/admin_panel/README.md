# FunnyX Admin Panel

Flutter Web admin UI for the platform control plane. Shared code lives under [`../common`](../common) and the shared setup steps are kept in [`../setup-reference.md`](../setup-reference.md).

## Auth

Admin Panel calls **Admin API directly** (default `http://127.0.0.1:18300`) — not via Gateway. Gateway remains for Client Web / Partner public ingress and validates session tokens on Session Token Server.

| UI | API |
| --- | --- |
| Login | `POST /v1/admin/login` |
| Logout | `POST /v1/admin/logout` |

Session Token Server issues `sts_…` with `actor_type=admin` after Admin API verifies credentials.

```bash
cd backend/session_token_server && cargo run
cd backend/admin_api && cargo run
```

Demo: `seed_admin` / `demo` (or `admin` / `demo`; `viewer` / `demo` is read-limited on ops writes).

## Corp onboarding

`/corp-onboard` — approve/reject pending companies (BR) and games (government + materials). APIs: `GET/POST /v1/admin/corporate-users…`, `GET/POST /v1/admin/games…`. See [doc/corp_onboarding.md](../../doc/corp_onboarding.md).

## Webhook Post_Auth

`/webhook-auth` — list/create/update/activate inbound partner auth mapping (`fx_events.webhook_post_auth`). APIs: `GET/POST/PUT/PATCH /v1/admin/webhook-post-auth*`. See [doc/webhook.md](../../doc/webhook.md) §3.1.

## Fees & license

`/fees` — C6/C9 platform fees (Year-1 80K / renewal 10K / extra pair 10K), corp `license_deadline_at` + `api_enabled` toggle. APIs: `GET/POST /v1/admin/fees`, `GET /v1/admin/fees/corps`, `PATCH /v1/admin/fees/corps/{id}/access`.

## App notes

- Public route: `/login`
- Session routes: `/dashboard`, `/markets`, `/corp-onboard`, `/webhook-auth`, `/fees`, `/setup`
- State and routing use **GetX** (`AdminApiAuthProvider`)
- Locales: `en`, `zh_TW`, `zh_CN`

For the common setup command flow, see [`../setup-reference.md`](../setup-reference.md).
