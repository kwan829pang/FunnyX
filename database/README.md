# FunnyX PostgreSQL DDL

Runnable create scripts for **12 schemas** and **45 tables**. Design/ERD remains in [`db_config/`](../db_config/).

## Counts

| Layer | Count |
| --- | --- |
| Schemas (`fx_*`) | 12 |
| Tables | 45 |

## Schemas

| Folder / schema | Tables |
| --- | --- |
| `fx_corp` | `corporate_users`, `corp_api_keys`, `partner_endpoints`, `platform_fees`, `corp_partner_notices`, `game_coin_supply_requests`, `corp_payment_records` (7) |
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
| `fx_events` | `notifications`, `outbound_notices`, `webhook_events`, `webhook_endpoints` (4) |

## Apply

From the `database/` directory:

```bash
psql -d funnyx -f run_all.sql
```

Or with path from repo root (adjust `-f` working directory so `\i` resolves):

```bash
cd database && psql -d funnyx -f run_all.sql
```

## Layout

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
