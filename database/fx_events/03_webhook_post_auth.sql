-- Inbound webhook Post_Auth mapping: source ↔ domain ↔ IP ↔ auth key / ppk

CREATE TABLE IF NOT EXISTS fx_events.webhook_post_auth (
    id BIGSERIAL PRIMARY KEY,
    source_code VARCHAR(64) NOT NULL UNIQUE,
    domain_pattern VARCHAR(255) NOT NULL,
    auth_key_suffix VARCHAR(128) NOT NULL,
    ppk VARCHAR(128),
    allowed_ips TEXT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'inactive')),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_webhook_post_auth_status
    ON fx_events.webhook_post_auth (status);

CREATE INDEX IF NOT EXISTS idx_webhook_post_auth_auth_key
    ON fx_events.webhook_post_auth (auth_key_suffix)
    WHERE status = 'active';

CREATE INDEX IF NOT EXISTS idx_webhook_post_auth_ppk
    ON fx_events.webhook_post_auth (ppk)
    WHERE status = 'active' AND ppk IS NOT NULL;
