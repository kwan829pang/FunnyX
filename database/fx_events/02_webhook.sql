-- Corp-owned webhook callback endpoints (MasterSigned CRUD on Webhook Server)

CREATE TABLE IF NOT EXISTS fx_events.webhook_endpoints (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id) ON DELETE CASCADE,
    kind VARCHAR(32) NOT NULL
        CHECK (kind IN ('shop_payment', 'deposit', 'withdrawal', 'corp_token')),
    callback_url VARCHAR(512) NOT NULL,
    auth_type VARCHAR(32) NOT NULL DEFAULT 'signature',
    secret_hint VARCHAR(64),
    status VARCHAR(32) NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'inactive')),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (corporate_user_id, kind)
);
