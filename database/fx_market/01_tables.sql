-- fx_market tables (9)
-- market_pair.status: submitted | pending_locked | approved | rejected | active | inactive
-- order_type: 1 Market | 2 Price; order status: 0 Pending | 1 Cancel | 2 Filled | 3 Partial Filled

CREATE TABLE IF NOT EXISTS fx_market.core_engines (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    host VARCHAR(255) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_market.market_pairs (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id),
    game_id BIGINT NOT NULL REFERENCES fx_game.games(id),
    base_game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    quote_game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    market_name VARCHAR(64) NOT NULL,
    funding_source VARCHAR(32) NOT NULL
        CHECK (funding_source IN ('gamecoin_lockup', 'platform_token_deposit')),
    platform_fee_id BIGINT REFERENCES fx_corp.platform_fees(id),
    engine_id BIGINT REFERENCES fx_market.core_engines(id),
    status VARCHAR(32) NOT NULL DEFAULT 'submitted',
    approved_by_admin_id BIGINT REFERENCES fx_admin.admin_users(id),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_market.market_balance_locks (
    id BIGSERIAL PRIMARY KEY,
    market_pair_id BIGINT NOT NULL REFERENCES fx_market.market_pairs(id),
    game_id BIGINT NOT NULL REFERENCES fx_game.games(id),
    game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    locked_amount NUMERIC(28,8) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'locked'
        CHECK (status IN ('locked', 'transferred', 'unlocked')),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_market.market_pools (
    id BIGSERIAL PRIMARY KEY,
    market_pair_id BIGINT NOT NULL UNIQUE REFERENCES fx_market.market_pairs(id),
    pool_depth NUMERIC(28,8) NOT NULL DEFAULT 0,
    initial_price NUMERIC(28,8) NOT NULL DEFAULT 0,
    base_amount NUMERIC(28,8) NOT NULL DEFAULT 0,
    quote_amount NUMERIC(28,8) NOT NULL DEFAULT 0,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_market.pool_wallets (
    id BIGSERIAL PRIMARY KEY,
    market_pool_id BIGINT NOT NULL REFERENCES fx_market.market_pools(id),
    game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    balance NUMERIC(28,8) NOT NULL DEFAULT 0,
    wallet_type VARCHAR(32) NOT NULL DEFAULT 'market_pool',
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    UNIQUE (market_pool_id, game_coin_id)
);

CREATE TABLE IF NOT EXISTS fx_market.pool_transfer_events (
    id BIGSERIAL PRIMARY KEY,
    market_pair_id BIGINT NOT NULL REFERENCES fx_market.market_pairs(id),
    market_pool_id BIGINT NOT NULL REFERENCES fx_market.market_pools(id),
    market_balance_lock_id BIGINT REFERENCES fx_market.market_balance_locks(id),
    event_type VARCHAR(32) NOT NULL
        CHECK (event_type IN ('lock', 'unlock', 'transfer_to_pool')),
    amount NUMERIC(28,8) NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS fx_market.market_status_logs (
    id BIGSERIAL PRIMARY KEY,
    market_pair_id BIGINT NOT NULL REFERENCES fx_market.market_pairs(id),
    old_status VARCHAR(32),
    new_status VARCHAR(32) NOT NULL,
    updated_by_admin_id BIGINT REFERENCES fx_admin.admin_users(id),
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_market.orders (
    id BIGSERIAL PRIMARY KEY,
    game_account_id BIGINT NOT NULL REFERENCES fx_game.game_accounts(id),
    market_pair_id BIGINT NOT NULL REFERENCES fx_market.market_pairs(id),
    order_type SMALLINT NOT NULL CHECK (order_type IN (1, 2)),
    side VARCHAR(8) NOT NULL CHECK (side IN ('buy', 'sell')),
    price NUMERIC(28,8) NOT NULL DEFAULT 0,
    qty NUMERIC(28,8) NOT NULL,
    status SMALLINT NOT NULL CHECK (status IN (0, 1, 2, 3)),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_market.trades (
    id BIGSERIAL PRIMARY KEY,
    market_pair_id BIGINT NOT NULL REFERENCES fx_market.market_pairs(id),
    buyer_order_id BIGINT NOT NULL REFERENCES fx_market.orders(id),
    seller_order_id BIGINT NOT NULL REFERENCES fx_market.orders(id),
    executed_price NUMERIC(28,8) NOT NULL,
    executed_qty NUMERIC(28,8) NOT NULL,
    executed_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT 0,
    updated_at BIGINT NOT NULL DEFAULT 0
);
