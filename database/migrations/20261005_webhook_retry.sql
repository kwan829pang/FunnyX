-- Widen webhook_events + create webhook_endpoints for existing databases.

\set ON_ERROR_STOP on

ALTER TABLE fx_events.webhook_events
    ADD COLUMN IF NOT EXISTS event_id VARCHAR(128);

ALTER TABLE fx_events.webhook_events
    ADD COLUMN IF NOT EXISTS delivery_status VARCHAR(32);

ALTER TABLE fx_events.webhook_events
    ADD COLUMN IF NOT EXISTS retry_count INT;

ALTER TABLE fx_events.webhook_events
    ADD COLUMN IF NOT EXISTS last_error VARCHAR(512);

ALTER TABLE fx_events.webhook_events
    ADD COLUMN IF NOT EXISTS next_retry_at BIGINT;

ALTER TABLE fx_events.webhook_events
    ADD COLUMN IF NOT EXISTS shop_order_id BIGINT;

ALTER TABLE fx_events.webhook_events
    ADD COLUMN IF NOT EXISTS partner_order_no VARCHAR(128);

ALTER TABLE fx_events.webhook_events
    ADD COLUMN IF NOT EXISTS updated_at BIGINT;

UPDATE fx_events.webhook_events SET delivery_status = 'received' WHERE delivery_status IS NULL;
UPDATE fx_events.webhook_events SET retry_count = 0 WHERE retry_count IS NULL;
UPDATE fx_events.webhook_events SET next_retry_at = 0 WHERE next_retry_at IS NULL;
UPDATE fx_events.webhook_events SET updated_at = 0 WHERE updated_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS uq_webhook_events_event_id
    ON fx_events.webhook_events (event_id)
    WHERE event_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS fx_events.webhook_endpoints (
    id BIGSERIAL PRIMARY KEY,
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id) ON DELETE CASCADE,
    kind VARCHAR(32) NOT NULL,
    callback_url VARCHAR(512) NOT NULL,
    auth_type VARCHAR(32) NOT NULL DEFAULT 'signature',
    secret_hint VARCHAR(64),
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (corporate_user_id, kind)
);
