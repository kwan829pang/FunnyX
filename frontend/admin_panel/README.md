# FunnyX Admin Panel

Flutter Web admin UI for the platform control plane. Shared code lives under [`../common`](../common) and the shared setup steps are kept in [`../setup-reference.md`](../setup-reference.md).

## Auth

Admin Panel calls **Gateway** (default `http://127.0.0.1:8080`), which proxies `/v1/admin/*` to Admin API:

| UI | API |
| --- | --- |
| Login | `POST /v1/admin/login` |
| Logout | `POST /v1/admin/logout` |

Session Token Server issues `sts_…` with `actor_type=admin` after Admin API verifies credentials.

```bash
cd backend/session_token_server && cargo run
cd backend/admin_api && cargo run
cd backend/gateway && cargo run
```

Demo: `seed_admin` / `demo` (or `admin` / `demo`).

## App notes

- Public route: `/login`
- Session routes: `/dashboard`, `/markets`
- State and routing use **GetX** (`AdminApiAuthProvider`)
- Locales: `en`, `zh_TW`, `zh_CN`

For the common setup command flow, see [`../setup-reference.md`](../setup-reference.md).
