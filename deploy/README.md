# FunnyX platform Docker mesh (Phase 5 §21)

## Full stack (recommended)

One compose file starts **Postgres + Redis + Mongo**, all platform services, and local simulators on network `funnyx-all`. Schema + demo seed run automatically via `db-seed`.

```bash
# From repo root
docker compose -f deploy/docker-compose.all.yml up --build -d

# Or (Windows)
pwsh -File scripts/docker_up_all.ps1
```

Tear down:

```bash
docker compose -f deploy/docker-compose.all.yml down
# Wipe DB / engine volumes:
docker compose -f deploy/docker-compose.all.yml down -v
```

### Ports

| Service | Host |
| --- | --- |
| Gateway | `8080` / socket `9000` |
| Config | `8090` |
| STS | `8082` |
| Client Center | `8083` / `9001` |
| Admin API | `18300` / `9003` |
| Webhook | `8084` / `9004` |
| Message Center | `8085` / `9005` |
| Core Engine | `18200` (`QUOTE_ASSET=PLT`, `DEMO_COIN/PLT`) — **CPU-only** (no GPU; default 2 CPUs / 1G; `CPU_ONLY=1`) |
| Postgres | `5432` |
| Redis | `6379` |
| Mongo | `27017` |
| Payment gate (sim) | `18100` |
| Message queue (sim) | `18101` |
| Company A (sim) | `18102` |
| Market bot (sim) | `18103` |

Internal wiring (Compose DNS): Client Center → `company-a-server` / `payment-gate`; simulators → `webhook-server`; market-bot → Gateway Session + Core Engine; Core Engine notices → Message Center + Company A.

Demo login for market-bot / Session: `demo_user` / `demo` (from `seed/01_demo_data.sql`).

## Split stacks (optional)

Use when you want stores or simulators managed separately:

```bash
# 1) Stores only
docker compose -f database/docker-compose.yml up -d
# Apply seeds: see database/seed/README.md

# 2) Platform only (DB via host.docker.internal)
docker compose -f deploy/docker-compose.yml up --build -d

# 3) Simulators only (includes its own Core Engine — do not combine with full stack)
docker compose -f dev_simulator/docker-compose.yml up --build -d
```

Do **not** run `docker-compose.all.yml` together with the split stacks — container names collide. Stop them first:

```bash
docker compose -f database/docker-compose.yml down
docker compose -f dev_simulator/docker-compose.yml down
```

`PUBLIC_BASE_URL` values are Compose DNS names so Gateway registry upstreams work inside the mesh. Host ports stay published for curl/smoke from the machine.

Demo market `DEMO_COIN/PLT` seeds as `pending_locked` — Admin approve is required before Session/market-bot can trade it.

## Smoke

```powershell
pwsh -File scripts/db_smoke.ps1          # Phase 1 schema/seed checks (needs psql)
pwsh -File scripts/docker_smoke.ps1      # /health on mesh ports
pwsh -File scripts/local_e2e_checklist.ps1   # deep E2E against Docker ports
```

First image builds are slow (Rust release). Later rebuilds reuse Docker layer cache when `common/` and the service crate are unchanged.
