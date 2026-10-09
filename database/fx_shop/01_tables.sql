-- fx_shop tables (4)
-- Platform PLT_* packages + Corp products; fiat HKD|USD (C2/C4)

CREATE TABLE IF NOT EXISTS fx_shop.shop_packages (
    id BIGSERIAL PRIMARY KEY,
    code VARCHAR(64) NOT NULL UNIQUE,
    name VARCHAR(128) NOT NULL,
    game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    coin_amount NUMERIC(28,8) NOT NULL CHECK (coin_amount > 0),
    fiat_price NUMERIC(28,8) NOT NULL CHECK (fiat_price > 0),
    seller_type VARCHAR(16) NOT NULL DEFAULT 'platform'
        CHECK (seller_type = 'platform'),
    status VARCHAR(32) NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'inactive', 'archived')),
    created_by_admin_id BIGINT REFERENCES fx_admin.admin_users(id),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_shop.corp_shop_products (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id),
    game_id BIGINT REFERENCES fx_game.games(id),
    code VARCHAR(64) NOT NULL,
    name VARCHAR(128) NOT NULL,
    product_type VARCHAR(32) NOT NULL
        CHECK (product_type IN ('game_coin_package', 'company_coin_package', 'game_item')),
    credit_game_coin_id BIGINT REFERENCES fx_game.game_coins(id),
    credit_amount NUMERIC(28,8),
    item_code VARCHAR(128),
    fiat_price NUMERIC(28,8) NOT NULL CHECK (fiat_price > 0),
    seller_type VARCHAR(16) NOT NULL DEFAULT 'corp'
        CHECK (seller_type = 'corp'),
    status VARCHAR(32) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'active', 'inactive', 'archived')),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (corporate_user_id, code)
);

CREATE TABLE IF NOT EXISTS fx_shop.shop_orders (
    id BIGSERIAL PRIMARY KEY,
    game_account_id BIGINT NOT NULL REFERENCES fx_game.game_accounts(id),
    end_user_id BIGINT NOT NULL REFERENCES fx_user.end_users(id),
    seller_type VARCHAR(16) NOT NULL
        CHECK (seller_type IN ('platform', 'corp')),
    package_id BIGINT REFERENCES fx_shop.shop_packages(id),
    corp_product_id BIGINT REFERENCES fx_shop.corp_shop_products(id),
    corporate_user_id BIGINT REFERENCES fx_corp.corporate_users(id),
    partner_endpoint_id BIGINT REFERENCES fx_corp.partner_endpoints(id),
    partner_order_no VARCHAR(128),
    fiat_currency VARCHAR(8) NOT NULL
        CHECK (fiat_currency IN ('HKD', 'USD')),
    fiat_price NUMERIC(28,8) NOT NULL CHECK (fiat_price > 0),
    credit_amount NUMERIC(28,8),
    game_coin_id BIGINT REFERENCES fx_game.game_coins(id),
    item_code VARCHAR(128),
    status VARCHAR(32) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'paid', 'failed', 'expired', 'cancelled')),
    expires_at BIGINT NOT NULL,
    paid_at BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (partner_order_no)
);

CREATE TABLE IF NOT EXISTS fx_shop.shop_payment_events (
    id BIGSERIAL PRIMARY KEY,
    shop_order_id BIGINT NOT NULL REFERENCES fx_shop.shop_orders(id) ON DELETE CASCADE,
    event_id VARCHAR(128) NOT NULL,
    event_type VARCHAR(64) NOT NULL,
    partner_order_no VARCHAR(128) NOT NULL,
    status VARCHAR(32) NOT NULL,
    payload JSONB NOT NULL,
    received_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (event_id)
);
