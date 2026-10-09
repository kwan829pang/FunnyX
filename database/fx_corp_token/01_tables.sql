-- fx_corp_token tables (4)

CREATE TABLE IF NOT EXISTS fx_corp_token.company_basic_tokens (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id),
    game_id BIGINT NOT NULL REFERENCES fx_game.games(id),
    game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    token_code VARCHAR(32) NOT NULL,
    token_name VARCHAR(128) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'submitted'
        CHECK (status IN ('submitted', 'pending', 'approved', 'rejected')),
    buyable BOOLEAN NOT NULL DEFAULT FALSE,
    buy_fee_rate NUMERIC(10,6) NOT NULL DEFAULT 0.001,
    approved_by_admin_id BIGINT REFERENCES fx_admin.admin_users(id),
    approved_at BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (corporate_user_id, token_code)
);

CREATE TABLE IF NOT EXISTS fx_corp_token.corp_token_orders (
    id BIGSERIAL PRIMARY KEY,
    company_basic_token_id BIGINT NOT NULL REFERENCES fx_corp_token.company_basic_tokens(id),
    end_user_id BIGINT NOT NULL REFERENCES fx_user.end_users(id),
    game_account_id BIGINT NOT NULL REFERENCES fx_game.game_accounts(id),
    partner_endpoint_id BIGINT REFERENCES fx_corp.partner_endpoints(id),
    partner_order_no VARCHAR(128),
    pay_amount NUMERIC(28,8) NOT NULL CHECK (pay_amount > 0),
    pay_game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    coin_amount NUMERIC(28,8) NOT NULL CHECK (coin_amount > 0),
    fee_coin_amount NUMERIC(28,8) NOT NULL DEFAULT 0,
    credited_coin_amount NUMERIC(28,8) NOT NULL DEFAULT 0,
    game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    status VARCHAR(32) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'paid', 'failed', 'expired', 'cancelled')),
    expires_at BIGINT NOT NULL,
    paid_at BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (partner_order_no)
);

CREATE TABLE IF NOT EXISTS fx_corp_token.corp_token_payment_events (
    id BIGSERIAL PRIMARY KEY,
    corp_token_order_id BIGINT NOT NULL REFERENCES fx_corp_token.corp_token_orders(id) ON DELETE CASCADE,
    event_id VARCHAR(128) NOT NULL,
    event_type VARCHAR(64) NOT NULL,
    partner_order_no VARCHAR(128) NOT NULL,
    status VARCHAR(32) NOT NULL,
    payload JSONB NOT NULL,
    received_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (event_id)
);

CREATE TABLE IF NOT EXISTS fx_corp_token.coin_fee_ledger (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id),
    company_basic_token_id BIGINT NOT NULL REFERENCES fx_corp_token.company_basic_tokens(id),
    corp_token_order_id BIGINT REFERENCES fx_corp_token.corp_token_orders(id),
    game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    fee_rate NUMERIC(10,6) NOT NULL DEFAULT 0.001,
    gross_coin_amount NUMERIC(28,8) NOT NULL,
    fee_coin_amount NUMERIC(28,8) NOT NULL,
    net_coin_amount NUMERIC(28,8) NOT NULL,
    created_at BIGINT NOT NULL
);
