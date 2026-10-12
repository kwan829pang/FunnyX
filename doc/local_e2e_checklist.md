# Local E2E checklist (§24)

Repeatable verification against **cargo-run** services (not Docker). Ports match service READMEs / `.env.sample`.

## Prerequisites (running)

| Service | Port |
| --- | --- |
| Config Server | `8090` |
| Session Token Server | `8082` |
| Client Center | `8083` |
| Gateway | `8080` |
| Admin API | `18300` |
| Webhook Server | `8084` |
| Message Center | `8085` |
| Core Engine | `18200` |
| Postgres (+ demo seed) | `5432` |

Optional: payment-gate `18100`, market-bot `18103`.

## Run

```powershell
# From repo root
pwsh -File scripts/local_e2e_checklist.ps1
```

Env overrides: `GATEWAY_URL`, `ADMIN_URL`, `CC_URL`, `ENGINE_URL`, `BOT_USER`, `BOT_PASS`.

## Covered paths

1. Public `/health` — Gateway (+ upstreams summary), CC, Admin, Engine
2. Client register/login via Gateway
3. Session wallet read
4. Admin login + markets list
5. Session markets list + place price order (may fail if no pair — reported)
6. Gateway failover readiness — `/health` JSON includes `upstreams`
7. Duplicate shop webhook smoke — skipped unless `SHOP_ORDER_ID` / `EVENT_ID` set

## Pass criteria

Script exits `0` when required steps pass. Soft steps (order place without market, webhook without ids) print `SKIP` / `WARN` and do not fail the run.
