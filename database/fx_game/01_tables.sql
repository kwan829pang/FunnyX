-- fx_game tables (4)

CREATE TABLE IF NOT EXISTS fx_game.games (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id),
    game_name VARCHAR(255) NOT NULL,
    game_code VARCHAR(64) NOT NULL UNIQUE,
    -- STS / Client Center partner id (e.g. partner_2001); used for OAuth Path B match
    partner_code VARCHAR(64),
    -- Partner-side game id for lookup APIs (e.g. company-a game_001)
    partner_game_id VARCHAR(128),
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_game.game_accounts (
    id BIGSERIAL PRIMARY KEY,
    game_id BIGINT NOT NULL REFERENCES fx_game.games(id),
    end_user_id BIGINT REFERENCES fx_user.end_users(id),
    game_account_id VARCHAR(128) NOT NULL,
    partner_user_id VARCHAR(128),
    bind_source VARCHAR(32) NOT NULL DEFAULT 'direct'
        CHECK (bind_source IN ('partner_fetch', 'direct')),
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (game_id, game_account_id)
);

CREATE TABLE IF NOT EXISTS fx_game.game_coins (
    id BIGSERIAL PRIMARY KEY,
    code VARCHAR(32) NOT NULL UNIQUE,
    name VARCHAR(128) NOT NULL,
    type VARCHAR(32) NOT NULL DEFAULT 'coin'
        CHECK (type IN ('platform', 'company', 'game', 'internal')),
    asset_kind VARCHAR(32) NOT NULL DEFAULT 'token'
        CHECK (asset_kind IN ('token', 'crypto_token', 'stablecoin')),
    is_platform_token BOOLEAN NOT NULL DEFAULT FALSE,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_game.game_balances (
    id BIGSERIAL PRIMARY KEY,
    game_id BIGINT NOT NULL REFERENCES fx_game.games(id),
    game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    total_supply NUMERIC(28,8) NOT NULL DEFAULT 0,
    available_balance NUMERIC(28,8) NOT NULL DEFAULT 0,
    locked_balance NUMERIC(28,8) NOT NULL DEFAULT 0,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (game_id, game_coin_id)
);
