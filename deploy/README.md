# FunnyX platform Docker mesh (Phase 5 §21)

Brings up Config → STS → Client Center / Admin → Webhook / Message Center → Gateway → Core Engine on network `funnyx-platform`.

## Order

```bash
# 1) Stores (Postgres / Redis / Mongo)
cd database
cp .env.sample .env   # optional
docker compose up -d

# Apply seeds once (host psql or exec into postgres):
#   psql ... -f run_all.sql   # first boot already applies schema via init
#   psql ... -f seed/run_seed_dev.sql

# 2) Platform services (repo root)
docker compose -f deploy/docker-compose.yml up --build -d

# 3) Optional simulators
docker compose -f dev_simulator/docker-compose.yml up --build -d
```

Tear down: `docker compose -f deploy/docker-compose.yml down`

## Ports

| Service | Host |
| --- | --- |
| Gateway | `8080` / socket `9000` |
| Config | `8090` |
| STS | `8082` |
| Client Center | `8083` / `9001` |
| Admin API | `18300` / `9003` |
| Webhook | `8084` / `9004` |
| Message Center | `8085` / `9005` |
| Core Engine | `18200` |

DB URLs use `host.docker.internal` so platform containers reach `database/docker-compose.yml` published ports.

## Smoke

```powershell
pwsh -File scripts/db_smoke.ps1          # Phase 1 schema/seed checks (needs psql)
pwsh -File scripts/docker_smoke.ps1      # /health on mesh ports
pwsh -File scripts/local_e2e_checklist.ps1   # same as local cargo checklist against Docker ports
```

Build images are slow on first run (Rust release). Incremental rebuilds reuse Docker layer cache when `common/` and the service crate are unchanged.
