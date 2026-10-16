# FunnyX PostgreSQL DDL

Runnable create scripts for **12 schemas** and **48 tables**. Design/ERD remains in [`db_config/`](../db_config/).

## Counts

| Layer | Count |
| --- | --- |
| Schemas (`fx_*`) | 12 |
| Tables | 48 |

## Schemas

| Folder / schema | Tables |
| --- | --- |
| `fx_corp` | `corporate_users`, `corp_api_keys`, `partner_endpoints`, `platform_fees`, `corp_partner_notices`, `game_coin_supply_requests`, `corp_payment_records`, `corp_documents`, `corp_web_sessions` (9) |
| `fx_game` | `games`, `game_accounts`, `game_coins`, `game_balances` (4) |
| `fx_user` | `end_users`, `user_sessions`, `user_wallets`, `oauth_identities` (4) |
| `fx_market` | `core_engines`, `market_pairs`, `market_balance_locks`, `market_pools`, `pool_wallets`, `pool_transfer_events`, `market_status_logs`, `orders`, `trades` (9) |
| `fx_market_data` | `market_ohlcvs` (1) |
| `fx_marketplace` | `marketplace_deals`, `marketplace_deal_requests`, `marketplace_transfer_events` (3) |
| `fx_config` | `system_settings` (1) |
| `fx_shop` | `shop_packages`, `corp_shop_products`, `shop_orders`, `shop_payment_events` (4) |
| `fx_corp_token` | `company_basic_tokens`, `corp_token_orders`, `corp_token_payment_events`, `coin_fee_ledger` (4) |
| `fx_money` | `deposit_withdrawal_txns`, `deposit_withdrawal_status_logs` (2) |
| `fx_admin` | `admin_users`, `admin_action_logs` (2) |
| `fx_events` | `notifications`, `outbound_notices`, `webhook_events`, `webhook_endpoints`, `webhook_post_auth` (5) |

## Apply

From the `database/` directory:

```bash
psql -d funnyx -f run_all.sql
```

Or with path from repo root (adjust `-f` working directory so `\i` resolves):

```bash
cd database && psql -d funnyx -f run_all.sql
```

## Local Docker (Postgres + Redis + Mongo)

Shared Phase-1 stores for host-run FunnyX services. First Postgres boot applies `run_all.sql` only (**no seed**).

```bash
cd database
cp .env.sample .env   # optional
docker compose up -d
```

| Container | Host port | Notes |
| --- | --- | --- |
| `funnyx-postgres` | `5432` | User/pass/db `funnyx`; schema via `docker/postgres-init.sh` |
| `funnyx-redis` | `6379` | AOF enabled |
| `funnyx-mongo` | `27017` | No auth (local dev) |

Host connection URLs:

```text
POSTGRES_URL=postgres://funnyx:funnyx@127.0.0.1:5432/funnyx
REDIS_URL=redis://127.0.0.1:6379
MONGO_URL=mongodb://127.0.0.1:27017
```

Networks: `funnyx-db-postgres` `172.33.10.0/24`, `funnyx-db-redis` `172.33.11.0/24`, `funnyx-db-mongo` `172.33.12.0/24` (local compose; avoids overlap with other Docker stacks on `172.20.0.0/16`). Seed apply: [seed/README.md](seed/README.md) (`run_seed_dev.sql` for local E2E). Smoke: [`scripts/db_smoke.ps1`](../scripts/db_smoke.ps1).

## Layout

- `docker-compose.yml` — local Postgres / Redis / Mongo
- `docker/postgres-init.sh` — first-boot DDL (`run_all.sql`)
- `.env.sample` — compose credentials / ports
- `00_schemas.sql` — create all schemas
- `fx_*/00_create_schema.sql` — per-schema create
- `fx_*/01_tables.sql` — tables for that schema
- `fx_user/02_user_wallets.sql` — wallets after `fx_game` (FK order)
- `fx_user/03_oauth.sql` — `oauth_identities` after `fx_game` (optional `game_id` FK)
- `fx_events/02_webhook.sql` — Corp `webhook_endpoints`
- `fx_corp/02_supply_and_payments.sql` — supply requests + corp payment records after game/user
- `98_foreign_keys.sql` — deferred cross-schema FKs
- `99_indexes.sql` — indexes
- `migrations/` — idempotent ALTERs for databases already created from an earlier `run_all.sql`
- `run_all.sql` — ordered schema runner
- `seed/` — baseline + development demo data (`00_seed_data.sql`, `01_demo_data.sql`)

## Source

Ported alongside [`db_config/database_relationships.md`](../db_config/database_relationships.md).

Timestamps: **BIGINT UTC**. Seeds: [`seed/`](seed/) — `00_seed_data.sql` (live+demo), `01_demo_data.sql` (development only).
