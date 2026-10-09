-- FunnyX DEMO data — DEVELOPMENT ONLY
-- Do NOT apply to production / live.
-- Apply after 00_seed_data.sql:
--   psql -d funnyx -f seed/00_seed_data.sql
--   psql -d funnyx -f seed/01_demo_data.sql
-- Idempotent: safe to re-run (ON CONFLICT).
--
-- Contents (smoke / local demo):
--   - demo corporate user + API key + transfer endpoint
--   - demo game + company coin + game balance
--   - demo corp e-shop product
--   - demo end user + game account + wallet

\set ON_ERROR_STOP on

-- 2024-01-02T00:00:00.000Z
\set demo_ts 1704153600000

\echo '=== demo: corporate_users ==='
INSERT INTO fx_corp.corporate_users (
    id, company_name, master_code, master_id, status, api_enabled,
    license_deadline_at, created_at, updated_at
)
VALUES (
    1, 'Demo Game Partner Co.', 'DEMO_MASTER_CODE', 'DEMO_MASTER_ID',
    'active', TRUE, 0, :demo_ts, 0
)
ON CONFLICT (master_code) DO UPDATE
SET company_name = EXCLUDED.company_name,
    master_id = EXCLUDED.master_id,
    status = EXCLUDED.status,
    api_enabled = EXCLUDED.api_enabled,
    updated_at = :demo_ts;

SELECT setval(
    pg_get_serial_sequence('fx_corp.corporate_users', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_corp.corporate_users), 1)
);

\echo '=== demo: corp_api_keys ==='
INSERT INTO fx_corp.corp_api_keys (
    corporate_user_id, api_key, secret_hash, status, created_at, updated_at
)
VALUES (
    1, 'demo_api_key_do_not_use_live',
    -- placeholder hash only (not a real secret)
    'demo_secret_hash_placeholder',
    'active', :demo_ts, 0
)
ON CONFLICT (api_key) DO UPDATE
SET secret_hash = EXCLUDED.secret_hash,
    status = EXCLUDED.status,
    updated_at = :demo_ts;

\echo '=== demo: partner_endpoints (transfer) ==='
INSERT INTO fx_corp.partner_endpoints (
    corporate_user_id, type, endpoint, auth_type, status, created_at, updated_at
)
VALUES (
    1, 'transfer', 'https://partner.demo.local/api/transfer',
    'signature', 'active', :demo_ts, 0
)
ON CONFLICT (corporate_user_id, type) DO UPDATE
SET endpoint = EXCLUDED.endpoint,
    auth_type = EXCLUDED.auth_type,
    status = EXCLUDED.status,
    updated_at = :demo_ts;

\echo '=== demo: game_coins (company) ==='
INSERT INTO fx_game.game_coins (
    id, code, name, type, asset_kind, is_platform_token, status, created_at, updated_at
)
VALUES (
    2, 'DEMO_COIN', 'Demo Company Coin', 'company', 'token', FALSE, 'active', :demo_ts, 0
)
ON CONFLICT (code) DO UPDATE
SET name = EXCLUDED.name,
    type = EXCLUDED.type,
    status = EXCLUDED.status,
    updated_at = :demo_ts;

SELECT setval(
    pg_get_serial_sequence('fx_game.game_coins', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 2) FROM fx_game.game_coins), 2)
);

\echo '=== demo: games ==='
INSERT INTO fx_game.games (
    id, corporate_user_id, game_name, game_code, partner_code, partner_game_id,
    status, created_at, updated_at
)
VALUES (
    1, 1, 'Demo Adventure', 'DEMO_GAME', 'partner_2001', 'game_001',
    'active', :demo_ts, 0
)
ON CONFLICT (game_code) DO UPDATE
SET game_name = EXCLUDED.game_name,
    corporate_user_id = EXCLUDED.corporate_user_id,
    partner_code = EXCLUDED.partner_code,
    partner_game_id = EXCLUDED.partner_game_id,
    status = EXCLUDED.status,
    updated_at = :demo_ts;

SELECT setval(
    pg_get_serial_sequence('fx_game.games', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_game.games), 1)
);

\echo '=== demo: game_balances ==='
INSERT INTO fx_game.game_balances (
    game_id, game_coin_id, total_supply, available_balance, locked_balance, updated_at
)
VALUES (1, 2, 1000000, 1000000, 0, :demo_ts)
ON CONFLICT (game_id, game_coin_id) DO UPDATE
SET total_supply = EXCLUDED.total_supply,
    available_balance = EXCLUDED.available_balance,
    locked_balance = EXCLUDED.locked_balance,
    updated_at = :demo_ts;

\echo '=== demo: corp_shop_products ==='
INSERT INTO fx_shop.corp_shop_products (
    corporate_user_id, game_id, code, name, product_type,
    credit_game_coin_id, credit_amount, item_code, fiat_price,
    seller_type, status, created_at, updated_at
)
VALUES (
    1, 1, 'DEMO_PACK_100', 'Demo Coin Pack 100', 'company_coin_package',
    2, 100, NULL, 9.9, 'corp', 'active', :demo_ts, 0
)
ON CONFLICT (corporate_user_id, code) DO UPDATE
SET name = EXCLUDED.name,
    product_type = EXCLUDED.product_type,
    credit_game_coin_id = EXCLUDED.credit_game_coin_id,
    credit_amount = EXCLUDED.credit_amount,
    fiat_price = EXCLUDED.fiat_price,
    status = EXCLUDED.status,
    updated_at = :demo_ts;

\echo '=== demo: end_users ==='
INSERT INTO fx_user.end_users (
    id, username, email, password_hash, status, created_at, updated_at
)
VALUES (
    1, 'demo_user', 'demo_user@example.local', NULL, 'active', :demo_ts, 0
),
    (
    2, 'alice_plat', 'alice_plat@example.local', NULL, 'active', :demo_ts, 0
)
ON CONFLICT (username) DO UPDATE
SET email = EXCLUDED.email,
    status = EXCLUDED.status,
    updated_at = :demo_ts;

SELECT setval(
    pg_get_serial_sequence('fx_user.end_users', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_user.end_users), 2)
);

\echo '=== demo: game_accounts ==='
INSERT INTO fx_game.game_accounts (
    id, game_id, end_user_id, game_account_id, partner_user_id, bind_source,
    status, created_at, updated_at
)
VALUES (
    1, 1, 1, 'demo_player_9001', 'pu_9001', 'direct', 'active', :demo_ts, 0
)
ON CONFLICT (game_id, game_account_id) DO UPDATE
SET end_user_id = EXCLUDED.end_user_id,
    partner_user_id = EXCLUDED.partner_user_id,
    bind_source = EXCLUDED.bind_source,
    status = EXCLUDED.status,
    updated_at = :demo_ts;

SELECT setval(
    pg_get_serial_sequence('fx_game.game_accounts', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_game.game_accounts), 1)
);

\echo '=== demo: user_wallets (PLT + DEMO_COIN) ==='
INSERT INTO fx_user.user_wallets (
    game_account_id, game_coin_id, available, locked, updated_at
)
VALUES
    (1, 1, 0, 0, :demo_ts),
    (1, 2, 0, 0, :demo_ts)
ON CONFLICT (game_account_id, game_coin_id) DO UPDATE
SET available = EXCLUDED.available,
    locked = EXCLUDED.locked,
    updated_at = :demo_ts;

\echo '=== demo: 01_demo_data done (development only) ==='
