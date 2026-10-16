# Development Documentation

This doc is a detail guide for the architecture in [project.md](project.md). Matching bot behavior is also described in [gateway_core_engine_logic.md](gateway_core_engine_logic.md).

**Detail scope:** Local simulators and helpers for integration testing—not production deployment or the master platform flow.

Simulators and queues below validate deposits, notices, partner APIs, and matching without live traffic.

Partner setup UML: [partner-uml.md](partner-uml.md).

## 1. Deposit Simulator

The deposit simulator is used to generate simulated deposit events for testing and validation purposes.

It allows developers to trigger synthetic deposit transactions and verify that the downstream processing flow, event propagation, and account updates are functioning as expected.

This simulator is especially useful for validating:

- deposit event ingestion
- webhook processing
- notification flow
- account balance updates
- client-facing status responses

## 2. Notice Message Queue

The notice message queue is a message tracking and monitoring component used to review outbound system notifications.

It provides a centralized view of outgoing messages so developers can confirm whether the sending logic succeeded and whether a message was processed correctly by downstream services.

This queue supports validation of:

- outbound event notifications
- message delivery status
- alert visibility and traceability
- operational debugging for system communication flows

## 3. Auto Order Matching

An auto matching order bot will perform automated order placement and matching activity. It is designed to simulate trading behavior in a controlled development environment and to validate the end-to-end latency, execution flow, and matching logic of the system.

This bot will:

- place buy and sell orders automatically
- route order requests through the Gateway and Client Center path
- send valid internal order commands to the Core Engine
- trigger the matching engine using randomized timing patterns
- generate execution results and trading notices for validation

The bot is intended to operate with random timing intervals so that the platform can be tested under irregular traffic patterns, burst behavior, and non-uniform load. This helps developers validate matching fairness, ring-buffer throughput, and queue stability under realistic but unpredictable conditions.

## 4. Local Development (Docker-based)

**Recommended:** full stack (DB + platform + simulators) → [`deploy/docker-compose.all.yml`](../deploy/docker-compose.all.yml) / [`deploy/README.md`](../deploy/README.md). Deep E2E → [`local_e2e_checklist.md`](local_e2e_checklist.md).

```bash
# From repo root
docker compose -f deploy/docker-compose.all.yml up --build -d
# Or: pwsh -File scripts/docker_up_all.ps1
```

### 4.1 Shared DB stores (split)

PostgreSQL + Redis + MongoDB under [`database/docker-compose.yml`](../database/docker-compose.yml). First Postgres boot applies schema (`run_all.sql`) only — no seed. Apply seeds after boot (canonical: [`database/seed/README.md`](../database/seed/README.md)); local E2E needs `run_seed_dev.sql` (00 + 01). Skip this if you use `docker-compose.all.yml` (it seeds automatically).

```bash
cd database
cp .env.sample .env   # optional
docker compose up -d
# After Postgres is up:
# psql -d funnyx -f seed/run_seed_dev.sql
```

| Container | Port | Notes |
| --- | --- | --- |
| `funnyx-postgres` | `5432` | `funnyx` / `funnyx` / db `funnyx` |
| `funnyx-redis` | `6379` | AOF |
| `funnyx-mongo` | `27017` | Local, no auth |

Networks: `172.33.10.0/24` · `172.33.11.0/24` · `172.33.12.0/24`. Details: [`database/README.md`](../database/README.md).

### 4.2 Dev simulators + Core Engine (split)

Simulators-only compose under [`dev_simulator/docker-compose.yml`](../dev_simulator/docker-compose.yml). Do not run together with `docker-compose.all.yml` (name collisions). Prefer the full stack for partner/payment/market-bot wired to Gateway + Webhook.

```bash
cd dev_simulator
docker compose up --build
```

| Container | Port | Notes |
| --- | --- | --- |
| `funnyx-payment-gate` | `18100` | Fiat / deposit webhook sim |
| `funnyx-message-queue` | `18101` | Notice queue (Core Engine `MESSAGE_CENTER_URL`) |
| `funnyx-company-a-server` | `18102` | Partner APIs + notice callback |
| `funnyx-market-bot` | `18103` | Fill bot; place via Gateway Session when `GATEWAY_URL` set |
| `funnyx-core-engine` | `18200` | Matching; data volume `core_engine_data` |

Network: `funnyx-dev-sim` (`172.28.0.0/24`). Early phase: host services use `127.0.0.1` DB URLs; simulators stay on their own compose network.

## 5. Development Principles

The development environment is designed to support safe validation of platform behavior without affecting live trading or production system integrity.

All simulator-based activities should be isolated from production traffic, and all generated data should be clearly marked as test or simulated data where appropriate.
