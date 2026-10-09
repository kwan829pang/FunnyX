-- fx_corp deferred tables (after fx_game + fx_user exist)
-- game_coin_supply_requests: Corp submit GamerCoin supply increase (POST /v1/corp/game-coins/supply)
-- corp_payment_records: Corp end-user payment accounting (notify/history; NOT e-shop wallet credit)

CREATE TABLE IF NOT EXISTS fx_corp.game_coin_supply_requests (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id),
    game_id BIGINT NOT NULL REFERENCES fx_game.games(id),
    game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    requested_amount NUMERIC(28,8) NOT NULL CHECK (requested_amount > 0),
    current_total_supply NUMERIC(28,8) NOT NULL DEFAULT 0,
    -- Optional link to C6-style fee paid for this raise (e.g. supply_increase / 10K)
    platform_fee_id BIGINT REFERENCES fx_corp.platform_fees(id),
    status VARCHAR(32) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'approved', 'rejected', 'cancelled')),
    approved_by_admin_id BIGINT REFERENCES fx_admin.admin_users(id),
    approved_at BIGINT NOT NULL DEFAULT 0,
    rejected_at BIGINT NOT NULL DEFAULT 0,
    reject_reason TEXT,
    note TEXT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

-- Partner-pushed end-user payment records (corp accounting). Not shop_orders settlement.
CREATE TABLE IF NOT EXISTS fx_corp.corp_payment_records (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id),
    end_user_id BIGINT NOT NULL REFERENCES fx_user.end_users(id),
    game_account_id BIGINT REFERENCES fx_game.game_accounts(id),
    item_code VARCHAR(128),
    package_code VARCHAR(128),
    amount NUMERIC(28,8) NOT NULL CHECK (amount >= 0),
    game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    status VARCHAR(32) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'paid', 'failed', 'refunded')),
    partner_order_no VARCHAR(128),
    paid_at BIGINT NOT NULL DEFAULT 0,
    source VARCHAR(16) NOT NULL DEFAULT 'live'
        CHECK (source IN ('live', 'test')),
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (corporate_user_id, partner_order_no)
);
