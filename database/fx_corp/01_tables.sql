-- fx_corp tables (5 base; supply/payment tables in 02_supply_and_payments.sql after fx_game)
-- Cross-schema FKs deferred: platform_fees.related_game_id, corp_partner_notices.created_by_admin_id
-- See database/98_foreign_keys.sql

CREATE TABLE IF NOT EXISTS fx_corp.corporate_users (
    id BIGSERIAL PRIMARY KEY,
    company_name VARCHAR(255) NOT NULL,
    master_code VARCHAR(64) NOT NULL UNIQUE,
    master_id VARCHAR(64) NOT NULL UNIQUE,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    api_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    license_deadline_at BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_corp.corp_api_keys (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id),
    api_key VARCHAR(128) NOT NULL UNIQUE,
    secret_hash VARCHAR(255) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_corp.partner_endpoints (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id),
    type VARCHAR(32) NOT NULL
        CHECK (type IN (
            'oauth_authorize',
            'oauth_token',
            'oauth_userinfo',
            'deposit',
            'withdrawal',
            'transfer',
            'payment',
            'callback'
        )),
    endpoint VARCHAR(512) NOT NULL,
    auth_type VARCHAR(32) NOT NULL DEFAULT 'signature',
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (corporate_user_id, type)
);

CREATE TABLE IF NOT EXISTS fx_corp.platform_fees (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id),
    fee_type VARCHAR(32) NOT NULL
        CHECK (fee_type IN ('year1_package', 'license_renewal', 'new_pair', 'supply_increase')),
    amount NUMERIC(28,8) NOT NULL,
    pay_currency VARCHAR(16) NOT NULL
        CHECK (pay_currency IN ('HKD', 'USD', 'USDT')),
    license_year INT NOT NULL DEFAULT 0,
    related_market_pair_id BIGINT,
    related_game_id BIGINT,
    claim_next_year_license BOOLEAN NOT NULL DEFAULT FALSE,
    claimed_against_fee_id BIGINT REFERENCES fx_corp.platform_fees(id),
    status VARCHAR(32) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'paid', 'failed', 'refunded', 'claimed')),
    paid_at BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_corp.corp_partner_notices (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id),
    notice_type VARCHAR(32) NOT NULL
        CHECK (notice_type IN ('first_payment_due', 'renew_due', 'api_disable_warning', 'api_disabled', 'other')),
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    deadline_at BIGINT NOT NULL DEFAULT 0,
    related_platform_fee_id BIGINT REFERENCES fx_corp.platform_fees(id),
    created_by_admin_id BIGINT,
    read_at BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL
);
