-- FunnyX seed data — LIVE + DEMO (required baseline)
-- Apply after database/run_all.sql
--   psql -d funnyx -f seed/00_seed_data.sql
-- Idempotent: safe to re-run (ON CONFLICT).
--
-- Contents:
--   - system admin (seed operator)
--   - system_settings.base_fiat_currency = HKD
--   - Platform Token game_coin (PLT)
--   - Fixed e-shop packages PLT_1000 / 1500 / 3000 / 10000

\set ON_ERROR_STOP on

-- Fixed seed epoch (UTC ms) for reproducible created_at
-- 2024-01-01T00:00:00.000Z
\set seed_ts 1704067200000

\echo '=== seed: admin_users ==='
INSERT INTO fx_admin.admin_users (id, username, role, status, created_at, updated_at)
VALUES (1, 'seed_admin', 'super_admin', 'active', :seed_ts, 0)
ON CONFLICT (username) DO UPDATE
SET role = EXCLUDED.role,
    status = EXCLUDED.status,
    updated_at = :seed_ts;

SELECT setval(
    pg_get_serial_sequence('fx_admin.admin_users', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_admin.admin_users), 1)
);

\echo '=== seed: system_settings ==='
INSERT INTO fx_config.system_settings (
    setting_key, setting_value, updated_by_admin_id, created_at, updated_at
)
VALUES ('base_fiat_currency', 'HKD', 1, :seed_ts, 0)
ON CONFLICT (setting_key) DO UPDATE
SET setting_value = EXCLUDED.setting_value,
    updated_by_admin_id = EXCLUDED.updated_by_admin_id,
    updated_at = :seed_ts;

\echo '=== seed: game_coins (Platform Token) ==='
INSERT INTO fx_game.game_coins (
    id, code, name, type, asset_kind, is_platform_token, status, created_at, updated_at
)
VALUES (
    1, 'PLT', 'Platform Token', 'platform', 'token', TRUE, 'active', :seed_ts, 0
)
ON CONFLICT (code) DO UPDATE
SET name = EXCLUDED.name,
    type = EXCLUDED.type,
    asset_kind = EXCLUDED.asset_kind,
    is_platform_token = EXCLUDED.is_platform_token,
    status = EXCLUDED.status,
    updated_at = :seed_ts;

SELECT setval(
    pg_get_serial_sequence('fx_game.game_coins', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_game.game_coins), 1)
);

\echo '=== seed: shop_packages (PLT_*) ==='
INSERT INTO fx_shop.shop_packages (
    code, name, game_coin_id, coin_amount, fiat_price, seller_type, status,
    created_by_admin_id, created_at, updated_at
)
VALUES
    ('PLT_1000',  'Platform Token 1000',  1, 1000,    8.8,  'platform', 'active', 1, :seed_ts, 0),
    ('PLT_1500',  'Platform Token 1500',  1, 1500,   13.0,  'platform', 'active', 1, :seed_ts, 0),
    ('PLT_3000',  'Platform Token 3000',  1, 3000,   27.0,  'platform', 'active', 1, :seed_ts, 0),
    ('PLT_10000', 'Platform Token 10000', 1, 10000,  75.0,  'platform', 'active', 1, :seed_ts, 0)
ON CONFLICT (code) DO UPDATE
SET name = EXCLUDED.name,
    game_coin_id = EXCLUDED.game_coin_id,
    coin_amount = EXCLUDED.coin_amount,
    fiat_price = EXCLUDED.fiat_price,
    seller_type = EXCLUDED.seller_type,
    status = EXCLUDED.status,
    created_by_admin_id = EXCLUDED.created_by_admin_id,
    updated_at = :seed_ts;

\echo '=== seed: 00_seed_data done ==='
