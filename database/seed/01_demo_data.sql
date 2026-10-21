-- FunnyX DEMO data — DEVELOPMENT ONLY
-- Do NOT apply to production / live.
-- Apply after 00_seed_data.sql:
--   psql -d funnyx -f seed/00_seed_data.sql
--   psql -d funnyx -f seed/01_demo_data.sql
-- Idempotent: safe to re-run (ON CONFLICT).
--
-- Contents (smoke / local demo):
--   - 2 corporate users + API keys (+ corp A transfer/item_list endpoints)
--   - 2 platform end users (demo_user, alice_plat) — total 4 demo identities
--   - demo game + company coin + game balance
--   - demo corp e-shop product
--   - demo game accounts + wallets
--   - demo market pair DEMO_COIN/PLT (pending_locked) for Admin approve + Session exchange E2E
--   - matching order history (fx_market.orders / trades) + sample OHLCV + trade notices
-- password_hash: Argon2id of SHA-256("demo") hex (Flutter posts SHA-256; server Argon2-verifies)
--
-- Order status: 0 Pending | 1 Cancel | 2 Filled | 3 Partial Filled
-- Order type:   1 Market | 2 Price

\set ON_ERROR_STOP on

-- 2024-01-02T00:00:00.000Z
\set demo_ts 1704153600000

\echo '=== demo: corporate_users (2) ==='
INSERT INTO fx_corp.corporate_users (
    id, company_name, master_code, master_id, status, api_enabled,
    license_deadline_at, contact_email, contact_name, password_hash,
    created_at, updated_at
)
VALUES
(
    1, 'Demo Game Partner Co.', 'DEMO_MASTER_CODE', 'DEMO_MASTER_ID',
    'active', TRUE, 0,
    'demo_corp@example.local', 'Demo Corp Admin',
    '$argon2id$v=19$m=19456,t=2,p=1$2hf7+kFan1D1FRBvaj6Z/g$qDo/e/JPv0zDLjAL3R17lOqh6PMFVyKNRd5NjZ+E/uU',
    :demo_ts, 0
),
(
    2, 'Second Studio Ltd.', 'DEMO_CORP_B_CODE', 'DEMO_CORP_B_ID',
    'active', TRUE, 0,
    'corp_b@example.local', 'Corp B Admin',
    '$argon2id$v=19$m=19456,t=2,p=1$2hf7+kFan1D1FRBvaj6Z/g$qDo/e/JPv0zDLjAL3R17lOqh6PMFVyKNRd5NjZ+E/uU',
    :demo_ts, 0
)
ON CONFLICT (master_code) DO UPDATE
SET company_name = EXCLUDED.company_name,
    master_id = EXCLUDED.master_id,
    status = EXCLUDED.status,
    api_enabled = EXCLUDED.api_enabled,
    contact_email = EXCLUDED.contact_email,
    contact_name = EXCLUDED.contact_name,
    password_hash = EXCLUDED.password_hash,
    updated_at = :demo_ts;

SELECT setval(
    pg_get_serial_sequence('fx_corp.corporate_users', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_corp.corporate_users), 2)
);

\echo '=== demo: corp_api_keys (2) ==='
INSERT INTO fx_corp.corp_api_keys (
    corporate_user_id, api_key, secret_hash, status, created_at, updated_at
)
VALUES
(
    1, 'demo_api_key_do_not_use_live',
    -- placeholder hash only (not a real secret)
    'demo_secret_hash_placeholder',
    'active', :demo_ts, 0
),
(
    2, 'demo_api_key_corp_b',
    'demo_secret_hash_corp_b_placeholder',
    'active', :demo_ts, 0
)
ON CONFLICT (api_key) DO UPDATE
SET secret_hash = EXCLUDED.secret_hash,
    status = EXCLUDED.status,
    corporate_user_id = EXCLUDED.corporate_user_id,
    updated_at = :demo_ts;

\echo '=== demo: allow partner_endpoints type item_list (idempotent) ==='
DO $$
BEGIN
  ALTER TABLE fx_corp.partner_endpoints DROP CONSTRAINT IF EXISTS partner_endpoints_type_check;
EXCEPTION WHEN undefined_object THEN NULL;
END $$;
ALTER TABLE fx_corp.partner_endpoints DROP CONSTRAINT IF EXISTS partner_endpoints_type_check;
ALTER TABLE fx_corp.partner_endpoints
  ADD CONSTRAINT partner_endpoints_type_check
  CHECK (type IN (
      'oauth_authorize',
      'oauth_token',
      'oauth_userinfo',
      'deposit',
      'withdrawal',
      'transfer',
      'item_list',
      'payment',
      'callback'
  ));

\echo '=== demo: partner_endpoints (transfer + item_list) ==='
INSERT INTO fx_corp.partner_endpoints (
    corporate_user_id, type, endpoint, auth_type, status, created_at, updated_at
)
VALUES (
    1, 'transfer', 'http://127.0.0.1:18102/api/transfer',
    'signature', 'active', :demo_ts, 0
),
(
    1, 'item_list', 'http://127.0.0.1:18102/api/items',
    'signature', 'active', :demo_ts, 0
)
ON CONFLICT (corporate_user_id, type) DO UPDATE
SET endpoint = EXCLUDED.endpoint,
    auth_type = EXCLUDED.auth_type,
    status = EXCLUDED.status,
    updated_at = :demo_ts;

\echo '=== demo: allow webhook kind marketplace (idempotent) ==='
DO $$
BEGIN
  ALTER TABLE fx_events.webhook_endpoints DROP CONSTRAINT IF EXISTS webhook_endpoints_kind_check;
EXCEPTION WHEN undefined_object THEN NULL;
END $$;
ALTER TABLE fx_events.webhook_endpoints DROP CONSTRAINT IF EXISTS webhook_endpoints_kind_check;
DO $$
BEGIN
  ALTER TABLE fx_events.webhook_endpoints
    ADD CONSTRAINT webhook_endpoints_kind_check
    CHECK (kind IN ('shop_payment', 'deposit', 'withdrawal', 'corp_token', 'marketplace'));
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

\echo '=== demo: webhook_endpoints (marketplace) ==='
INSERT INTO fx_events.webhook_endpoints (
    corporate_user_id, kind, callback_url, auth_type, secret_hint, status, created_at, updated_at
)
VALUES (
    1, 'marketplace', 'http://127.0.0.1:18102/api/callback',
    'signature', 'demo', 'active', :demo_ts, 0
)
ON CONFLICT (corporate_user_id, kind) DO UPDATE
SET callback_url = EXCLUDED.callback_url,
    auth_type = EXCLUDED.auth_type,
    status = EXCLUDED.status,
    updated_at = :demo_ts;

\echo '=== demo: webhook_post_auth (Post_Auth inbound) ==='
INSERT INTO fx_events.webhook_post_auth (
    source_code, domain_pattern, auth_key_suffix, ppk, allowed_ips, status, created_at, updated_at
)
VALUES
    ('payment_gate', 'payment-gate.local', 'demo_key', 'ppk_payment_gate', '127.0.0.1,::1', 'active', :demo_ts, 0),
    ('partner_2001', '*.company-a.local', 'demo_key', 'ppk_partner_2001', '127.0.0.1,::1', 'active', :demo_ts, 0)
ON CONFLICT (source_code) DO UPDATE
SET domain_pattern = EXCLUDED.domain_pattern,
    auth_key_suffix = EXCLUDED.auth_key_suffix,
    ppk = EXCLUDED.ppk,
    allowed_ips = EXCLUDED.allowed_ips,
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

\echo '=== demo: end_users (2) ==='
INSERT INTO fx_user.end_users (
    id, username, email, password_hash, status, created_at, updated_at
)
VALUES
    (1, 'demo_user', 'demo_user@example.local',
     '$argon2id$v=19$m=19456,t=2,p=1$2hf7+kFan1D1FRBvaj6Z/g$qDo/e/JPv0zDLjAL3R17lOqh6PMFVyKNRd5NjZ+E/uU',
     'active', :demo_ts, 0),
    (2, 'alice_plat', 'alice_plat@example.local',
     '$argon2id$v=19$m=19456,t=2,p=1$2hf7+kFan1D1FRBvaj6Z/g$qDo/e/JPv0zDLjAL3R17lOqh6PMFVyKNRd5NjZ+E/uU',
     'active', :demo_ts, 0)
ON CONFLICT (username) DO UPDATE
SET email = EXCLUDED.email,
    status = EXCLUDED.status,
    password_hash = EXCLUDED.password_hash,
    updated_at = :demo_ts;

SELECT setval(
    pg_get_serial_sequence('fx_user.end_users', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_user.end_users), 2)
);

\echo '=== demo: game_accounts (one per end user) ==='
INSERT INTO fx_game.game_accounts (
    id, game_id, end_user_id, game_account_id, partner_user_id, bind_source,
    status, created_at, updated_at
)
VALUES
    (1, 1, 1, 'demo_player_9001', 'pu_9001', 'direct', 'active', :demo_ts, 0),
    (2, 1, 2, 'demo_player_9002', 'pu_9002', 'direct', 'active', :demo_ts, 0)
ON CONFLICT (game_id, game_account_id) DO UPDATE
SET end_user_id = EXCLUDED.end_user_id,
    partner_user_id = EXCLUDED.partner_user_id,
    bind_source = EXCLUDED.bind_source,
    status = EXCLUDED.status,
    updated_at = :demo_ts;

SELECT setval(
    pg_get_serial_sequence('fx_game.game_accounts', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_game.game_accounts), 2)
);

\echo '=== demo: user_wallets (PLT + DEMO_COIN) ==='
INSERT INTO fx_user.user_wallets (
    game_account_id, game_coin_id, available, locked, updated_at
)
VALUES
    (1, 1, 100, 0, :demo_ts),
    (1, 2, 50, 0, :demo_ts),
    (2, 1, 80, 0, :demo_ts),
    (2, 2, 40, 0, :demo_ts)
ON CONFLICT (game_account_id, game_coin_id) DO UPDATE
SET available = EXCLUDED.available,
    locked = EXCLUDED.locked,
    updated_at = :demo_ts;

\echo '=== demo: market pair DEMO_COIN/PLT (pending_locked) ==='
-- Quote = PLT → run Core Engine with QUOTE_ASSET=PLT for local exchange E2E.
INSERT INTO fx_market.core_engines (id, name, host, status, created_at, updated_at)
VALUES (1, 'demo-core-engine', 'http://127.0.0.1:18200', 'active', :demo_ts, 0)
ON CONFLICT (id) DO UPDATE
SET name = EXCLUDED.name,
    host = EXCLUDED.host,
    status = EXCLUDED.status,
    updated_at = :demo_ts;

SELECT setval(
    pg_get_serial_sequence('fx_market.core_engines', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_market.core_engines), 1)
);

INSERT INTO fx_market.market_pairs (
    id, corporate_user_id, game_id, base_game_coin_id, quote_game_coin_id,
    market_name, funding_source, engine_id, status, created_at, updated_at
)
VALUES (
    1, 1, 1, 2, 1, 'DEMO_COIN/PLT', 'gamecoin_lockup', 1,
    'pending_locked', :demo_ts, 0
)
ON CONFLICT (id) DO UPDATE
SET corporate_user_id = EXCLUDED.corporate_user_id,
    game_id = EXCLUDED.game_id,
    base_game_coin_id = EXCLUDED.base_game_coin_id,
    quote_game_coin_id = EXCLUDED.quote_game_coin_id,
    market_name = EXCLUDED.market_name,
    funding_source = EXCLUDED.funding_source,
    engine_id = EXCLUDED.engine_id,
    -- Keep active if already approved in a prior E2E run.
    status = CASE
        WHEN fx_market.market_pairs.status = 'active' THEN 'active'
        ELSE EXCLUDED.status
    END,
    updated_at = :demo_ts;

SELECT setval(
    pg_get_serial_sequence('fx_market.market_pairs', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_market.market_pairs), 1)
);

-- Ensure enough locked_balance for Admin approve (consumes lock → pool).
UPDATE fx_game.game_balances
SET locked_balance = GREATEST(locked_balance, 1000),
    updated_at = :demo_ts
WHERE game_id = 1 AND game_coin_id = 2;

INSERT INTO fx_market.market_balance_locks (
    id, market_pair_id, game_id, game_coin_id, locked_amount, status, created_at, updated_at
)
VALUES (1, 1, 1, 2, 1000, 'locked', :demo_ts, 0)
ON CONFLICT (id) DO UPDATE
SET locked_amount = EXCLUDED.locked_amount,
    status = CASE
        WHEN fx_market.market_balance_locks.status = 'transferred' THEN 'transferred'
        ELSE EXCLUDED.status
    END,
    updated_at = :demo_ts;

SELECT setval(
    pg_get_serial_sequence('fx_market.market_balance_locks', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_market.market_balance_locks), 1)
);

INSERT INTO fx_market.market_pools (
    id, market_pair_id, pool_depth, initial_price, base_amount, quote_amount,
    status, created_at, updated_at
)
VALUES (1, 1, 1000, 1, 100, 100, 'pending', :demo_ts, 0)
ON CONFLICT (market_pair_id) DO UPDATE
SET pool_depth = EXCLUDED.pool_depth,
    initial_price = EXCLUDED.initial_price,
    base_amount = EXCLUDED.base_amount,
    quote_amount = EXCLUDED.quote_amount,
    status = CASE
        WHEN fx_market.market_pools.status = 'active' THEN 'active'
        ELSE EXCLUDED.status
    END,
    updated_at = :demo_ts;

SELECT setval(
    pg_get_serial_sequence('fx_market.market_pools', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_market.market_pools), 1)
);

-- ---------------------------------------------------------------------------
-- Matching order history (DEMO_COIN/PLT between demo_user ga=1 and alice_plat ga=2)
-- Sample past prints for Admin/history UIs; market pair may still be pending_locked
-- until Admin approve — history rows do not change pair status.
-- ---------------------------------------------------------------------------
\echo '=== demo: fx_market.orders (match history) ==='
-- Timestamps relative to :demo_ts (2024-01-02T00:00:00Z)
INSERT INTO fx_market.orders (
    id, game_account_id, market_pair_id, order_type, side, price, qty,
    status, created_at, updated_at
)
VALUES
    -- Full match @ 1.00: alice sells 10, demo_user buys 10
    (1, 2, 1, 2, 'sell', 1.00000000, 10.00000000, 2, :demo_ts,           :demo_ts + 1000),
    (2, 1, 1, 2, 'buy',  1.00000000, 10.00000000, 2, :demo_ts + 1000,    :demo_ts + 1000),
    -- Cancelled resting sell (demo_user)
    (3, 1, 1, 2, 'sell', 1.20000000,  5.00000000, 1, :demo_ts + 3600000, :demo_ts + 3900000),
    -- Open bid still resting (alice)
    (4, 2, 1, 2, 'buy',  0.95000000, 20.00000000, 0, :demo_ts + 7200000, :demo_ts + 7200000),
    -- Partial then fill: demo_user sells 8 @ 1.05; alice takes 5 → seller partial
    (5, 1, 1, 2, 'sell', 1.05000000,  8.00000000, 3, :demo_ts + 10800000, :demo_ts + 10801000),
    (6, 2, 1, 2, 'buy',  1.05000000,  5.00000000, 2, :demo_ts + 10801000, :demo_ts + 10801000),
    -- Market buy fills against price sell @ 1.10
    (7, 2, 1, 2, 'sell', 1.10000000,  3.00000000, 2, :demo_ts + 14400000, :demo_ts + 14401000),
    (8, 1, 1, 1, 'buy',  0.00000000,  3.00000000, 2, :demo_ts + 14401000, :demo_ts + 14401000)
ON CONFLICT (id) DO UPDATE
SET game_account_id = EXCLUDED.game_account_id,
    market_pair_id = EXCLUDED.market_pair_id,
    order_type = EXCLUDED.order_type,
    side = EXCLUDED.side,
    price = EXCLUDED.price,
    qty = EXCLUDED.qty,
    status = EXCLUDED.status,
    created_at = EXCLUDED.created_at,
    updated_at = EXCLUDED.updated_at;

SELECT setval(
    pg_get_serial_sequence('fx_market.orders', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_market.orders), 8)
);

\echo '=== demo: fx_market.trades (match prints) ==='
INSERT INTO fx_market.trades (
    id, market_pair_id, buyer_order_id, seller_order_id,
    executed_price, executed_qty, executed_at, created_at, updated_at
)
VALUES
    (1, 1, 2, 1, 1.00000000, 10.00000000, :demo_ts + 1000,     :demo_ts + 1000,     :demo_ts + 1000),
    (2, 1, 6, 5, 1.05000000,  5.00000000, :demo_ts + 10801000, :demo_ts + 10801000, :demo_ts + 10801000),
    (3, 1, 8, 7, 1.10000000,  3.00000000, :demo_ts + 14401000, :demo_ts + 14401000, :demo_ts + 14401000)
ON CONFLICT (id) DO UPDATE
SET market_pair_id = EXCLUDED.market_pair_id,
    buyer_order_id = EXCLUDED.buyer_order_id,
    seller_order_id = EXCLUDED.seller_order_id,
    executed_price = EXCLUDED.executed_price,
    executed_qty = EXCLUDED.executed_qty,
    executed_at = EXCLUDED.executed_at,
    created_at = EXCLUDED.created_at,
    updated_at = EXCLUDED.updated_at;

SELECT setval(
    pg_get_serial_sequence('fx_market.trades', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_market.trades), 3)
);

\echo '=== demo: fx_market_data.market_ohlcvs (from match prints) ==='
INSERT INTO fx_market_data.market_ohlcvs (
    market_pair_id, timeframe, open_time, close_time,
    open, high, low, close, volume, quote_volume, trade_count,
    created_at, updated_at
)
VALUES
    -- 1h bars covering seeded trades (open_time aligned to hour starts from demo_ts)
    (1, '1h', :demo_ts,           :demo_ts + 3600000 - 1,
        1.00000000, 1.00000000, 1.00000000, 1.00000000,
        10.00000000, 10.00000000, 1, :demo_ts + 1000, :demo_ts + 1000),
    (1, '1h', :demo_ts + 10800000, :demo_ts + 14400000 - 1,
        1.05000000, 1.05000000, 1.05000000, 1.05000000,
        5.00000000, 5.25000000, 1, :demo_ts + 10801000, :demo_ts + 10801000),
    (1, '1h', :demo_ts + 14400000, :demo_ts + 18000000 - 1,
        1.10000000, 1.10000000, 1.10000000, 1.10000000,
        3.00000000, 3.30000000, 1, :demo_ts + 14401000, :demo_ts + 14401000),
    (1, '15m', :demo_ts, :demo_ts + 900000 - 1,
        1.00000000, 1.00000000, 1.00000000, 1.00000000,
        10.00000000, 10.00000000, 1, :demo_ts + 1000, :demo_ts + 1000)
ON CONFLICT (market_pair_id, timeframe, open_time) DO UPDATE
SET close_time = EXCLUDED.close_time,
    open = EXCLUDED.open,
    high = EXCLUDED.high,
    low = EXCLUDED.low,
    close = EXCLUDED.close,
    volume = EXCLUDED.volume,
    quote_volume = EXCLUDED.quote_volume,
    trade_count = EXCLUDED.trade_count,
    updated_at = EXCLUDED.updated_at;

\echo '=== demo: trade notifications + outbound_notices ==='
INSERT INTO fx_events.notifications (
    id, end_user_id, type, payload, read_at, created_at, updated_at
)
VALUES
    (1, 1, 'trade_filled',
     '{"symbol":"DEMO_COIN/PLT","trade_id":1,"side":"buy","price":"1","qty":"10"}'::jsonb,
     0, :demo_ts + 1000, 0),
    (2, 2, 'trade_filled',
     '{"symbol":"DEMO_COIN/PLT","trade_id":1,"side":"sell","price":"1","qty":"10"}'::jsonb,
     0, :demo_ts + 1000, 0),
    (3, 1, 'trade_partial_filled',
     '{"symbol":"DEMO_COIN/PLT","trade_id":2,"side":"sell","price":"1.05","qty":"5"}'::jsonb,
     0, :demo_ts + 10801000, 0),
    (4, 2, 'trade_filled',
     '{"symbol":"DEMO_COIN/PLT","trade_id":2,"side":"buy","price":"1.05","qty":"5"}'::jsonb,
     0, :demo_ts + 10801000, 0)
ON CONFLICT (id) DO UPDATE
SET end_user_id = EXCLUDED.end_user_id,
    type = EXCLUDED.type,
    payload = EXCLUDED.payload,
    updated_at = EXCLUDED.updated_at;

SELECT setval(
    pg_get_serial_sequence('fx_events.notifications', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_events.notifications), 4)
);

INSERT INTO fx_events.outbound_notices (
    id, end_user_id, source_type, source_id, event_type, title, body, payload,
    delivery_status, retry_count, scheduled_at, sent_at, notification_id,
    created_at, updated_at
)
VALUES
    (1, 1, 'trade', 1, 'filled', 'Trade filled',
     'Bought 10 DEMO_COIN @ 1 PLT',
     '{"symbol":"DEMO_COIN/PLT","trade_id":1}'::jsonb,
     'sent', 0, 0, :demo_ts + 1000, 1, :demo_ts + 1000, :demo_ts + 1000),
    (2, 2, 'trade', 1, 'filled', 'Trade filled',
     'Sold 10 DEMO_COIN @ 1 PLT',
     '{"symbol":"DEMO_COIN/PLT","trade_id":1}'::jsonb,
     'sent', 0, 0, :demo_ts + 1000, 2, :demo_ts + 1000, :demo_ts + 1000),
    (3, 1, 'trade', 2, 'partial_filled', 'Trade partial fill',
     'Sold 5 of 8 DEMO_COIN @ 1.05 PLT',
     '{"symbol":"DEMO_COIN/PLT","trade_id":2,"order_id":5}'::jsonb,
     'sent', 0, 0, :demo_ts + 10801000, 3, :demo_ts + 10801000, :demo_ts + 10801000),
    (4, 2, 'trade', 2, 'filled', 'Trade filled',
     'Bought 5 DEMO_COIN @ 1.05 PLT',
     '{"symbol":"DEMO_COIN/PLT","trade_id":2}'::jsonb,
     'sent', 0, 0, :demo_ts + 10801000, 4, :demo_ts + 10801000, :demo_ts + 10801000),
    (5, 1, 'order', 3, 'cancelled', 'Order cancelled',
     'Sell 5 DEMO_COIN @ 1.20 cancelled',
     '{"symbol":"DEMO_COIN/PLT","order_id":3}'::jsonb,
     'sent', 0, 0, :demo_ts + 3900000, NULL, :demo_ts + 3900000, :demo_ts + 3900000)
ON CONFLICT (id) DO UPDATE
SET end_user_id = EXCLUDED.end_user_id,
    source_type = EXCLUDED.source_type,
    source_id = EXCLUDED.source_id,
    event_type = EXCLUDED.event_type,
    title = EXCLUDED.title,
    body = EXCLUDED.body,
    payload = EXCLUDED.payload,
    delivery_status = EXCLUDED.delivery_status,
    sent_at = EXCLUDED.sent_at,
    notification_id = EXCLUDED.notification_id,
    updated_at = EXCLUDED.updated_at;

SELECT setval(
    pg_get_serial_sequence('fx_events.outbound_notices', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 1) FROM fx_events.outbound_notices), 5)
);

\echo '=== demo: 01_demo_data done (development only) ==='
