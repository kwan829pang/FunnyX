-- FunnyX PostgreSQL DDL runner
-- Apply from repo root or this directory with psql:
--   psql -d funnyx -f database/run_all.sql
-- Or from database/:
--   psql -d funnyx -f run_all.sql
--
-- Order is FK-safe: schemas → corp → user → admin → config → game →
-- wallets → corp_token → market → market_data → marketplace → shop → money → events →
-- deferred FKs → indexes

\echo '=== 00 schemas ==='
\i 00_schemas.sql

\echo '=== fx_corp ==='
\i fx_corp/00_create_schema.sql
\i fx_corp/01_tables.sql

\echo '=== fx_user (identity) ==='
\i fx_user/00_create_schema.sql
\i fx_user/01_tables.sql

\echo '=== fx_admin ==='
\i fx_admin/00_create_schema.sql
\i fx_admin/01_tables.sql

\echo '=== fx_config ==='
\i fx_config/00_create_schema.sql
\i fx_config/01_tables.sql

\echo '=== fx_game ==='
\i fx_game/00_create_schema.sql
\i fx_game/01_tables.sql

\echo '=== fx_user wallets ==='
\i fx_user/02_user_wallets.sql

\echo '=== fx_user oauth identities ==='
\i fx_user/03_oauth.sql

\echo '=== fx_corp supply + payment records ==='
\i fx_corp/02_supply_and_payments.sql

\echo '=== fx_corp_token ==='
\i fx_corp_token/00_create_schema.sql
\i fx_corp_token/01_tables.sql

\echo '=== fx_market ==='
\i fx_market/00_create_schema.sql
\i fx_market/01_tables.sql

\echo '=== fx_market_data ==='
\i fx_market_data/00_create_schema.sql
\i fx_market_data/01_tables.sql

\echo '=== fx_marketplace ==='
\i fx_marketplace/00_create_schema.sql
\i fx_marketplace/01_tables.sql

\echo '=== fx_shop ==='
\i fx_shop/00_create_schema.sql
\i fx_shop/01_tables.sql

\echo '=== fx_money ==='
\i fx_money/00_create_schema.sql
\i fx_money/01_tables.sql

\echo '=== fx_events ==='
\i fx_events/00_create_schema.sql
\i fx_events/01_tables.sql
\i fx_events/02_webhook.sql

\echo '=== 98 deferred FKs ==='
\i 98_foreign_keys.sql

\echo '=== 99 indexes ==='
\i 99_indexes.sql

\echo '=== Done: 12 schemas, 45 tables ==='
