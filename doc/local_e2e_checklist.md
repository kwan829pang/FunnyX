# Local E2E checklist (§24 / Phase 6)

Repeatable deep verification against **cargo-run** services (not Docker). Ports match service READMEs / `.env.sample`.

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
| Core Engine | `18200` (`QUOTE_ASSET=PLT` for demo market) |
| Postgres (+ `00_seed_data` + `01_demo_data`) | `5432` |

Optional simulators (steps **SKIP** when down):

| Simulator | Port | Used by |
| --- | --- | --- |
| payment-gate | `18100` | Slice A e-shop settle |
| company-a-server | `18102` | Slice B deposit/withdraw |

## Run

```powershell
# From repo root
pwsh -File scripts/local_e2e_checklist.ps1
# or
powershell -File scripts/local_e2e_checklist.ps1
```

Env overrides: `GATEWAY_URL`, `ADMIN_URL`, `ENGINE_URL`, `WEBHOOK_URL`, `PAYMENT_GATE_URL`, `COMPANY_A_URL`, `BOT_USER`, `BOT_PASS`, `ADMIN_USER`, `ADMIN_PASS`, `GAME_ACCOUNT_ID`, `CALLBACK_SIGNATURE`, `POST_AUTH_KEY`, `ENGINE_ADMIN_KEY`.

Modules live under [`scripts/e2e/`](../scripts/e2e/).

## Covered paths

### Baseline

1. Public `/health` — Gateway (+ upstreams), Engine, Admin
2. Client login via Gateway (`demo_user` / `demo`)
3. Session wallet read
4. Admin login + markets list

### Slice A — E-shop + webhook

1. `GET /v1/shop/packages` → create platform order for `PLT_1000`
2. payment-gate `complete` with `fire_callback=false`
3. Webhook `POST /v1/webhook/shop/payment` with Post_Auth → order `paid`, PLT credited once
4. Replay **same `event_id`** → no double-credit

### Slice B — Deposit / withdrawal

1. `POST /v1/client/deposit` DEMO_COIN → company-a → webhook → wallet up
2. `POST /v1/client/withdraw` → wallet down + completed txn in history

### Slice C — Market + exchange

1. Admin approve `pending_locked` demo market (`DEMO_COIN/PLT`) if needed
2. Session markets → place price order → cancel
3. Engine pair ensured via Admin approve and/or `POST /v1/admin/pairs`

### Slice D — Marketplace gates

1. `demo_user` eligibility `can_place=true` (PLT + listed game + transfer)
2. Place deal (1 PLT fee) then cancel
3. `alice_plat` eligibility gate fail + place rejected

## Pass criteria

Script exits `0` when required steps pass. Optional simulator steps print `SKIP` and do not fail the run. Exchange requires demo market seed + Core Engine `QUOTE_ASSET=PLT`.
