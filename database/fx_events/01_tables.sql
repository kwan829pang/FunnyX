-- fx_events tables (4 of 4) — notifications, outbound_notices, webhook_events
-- webhook_endpoints in 02_webhook.sql
-- notifications = user-facing inbox after delivery
-- outbound_notices = Message Center outbox (wait-to-send) for domain status changes
-- webhook_events = inbound partner/ops copies
--
-- outbound_notices uses polymorphic (source_type, source_id) with no FKs to
-- marketplace / order / trade / shop_order / deposit_withdrawal / corp_token tables.
-- Logical association only; Message Center drains delivery_status = pending.

CREATE TABLE IF NOT EXISTS fx_events.notifications (
    id BIGSERIAL PRIMARY KEY,
    end_user_id BIGINT NOT NULL REFERENCES fx_user.end_users(id),
    type VARCHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    read_at BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

-- Outbox: create on marketplace / exchange order-trade / shop (platform|corp) /
-- deposit-withdrawal status changes; Message Center drains delivery_status=pending
CREATE TABLE IF NOT EXISTS fx_events.outbound_notices (
    id BIGSERIAL PRIMARY KEY,
    end_user_id BIGINT NOT NULL REFERENCES fx_user.end_users(id),
    source_type VARCHAR(64) NOT NULL
        CHECK (source_type IN (
            'marketplace_deal',
            'marketplace_deal_request',
            'order',
            'trade',
            'shop_order',
            'deposit_withdrawal_txn',
            'corp_token_order'
        )),
    source_id BIGINT NOT NULL,
    event_type VARCHAR(64) NOT NULL
        CHECK (event_type IN (
            'created',
            'pending_payment',
            'completed',
            'cancelled',
            'rejected',
            'filled',
            'partial_filled',
            'failed',
            'status_changed'
        )),
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    delivery_status VARCHAR(32) NOT NULL DEFAULT 'pending'
        CHECK (delivery_status IN ('pending', 'sending', 'sent', 'failed')),
    retry_count INT NOT NULL DEFAULT 0,
    scheduled_at BIGINT NOT NULL DEFAULT 0,
    sent_at BIGINT NOT NULL DEFAULT 0,
    notification_id BIGINT REFERENCES fx_events.notifications(id),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_events.webhook_events (
    id BIGSERIAL PRIMARY KEY,
    event_id VARCHAR(128) NOT NULL UNIQUE,
    source VARCHAR(64) NOT NULL,
    event_type VARCHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    delivery_status VARCHAR(32) NOT NULL DEFAULT 'received'
        CHECK (delivery_status IN ('received', 'processing', 'settled', 'failed', 'dead')),
    retry_count INT NOT NULL DEFAULT 0,
    last_error VARCHAR(512),
    next_retry_at BIGINT NOT NULL DEFAULT 0,
    shop_order_id BIGINT REFERENCES fx_shop.shop_orders(id) ON DELETE SET NULL,
    partner_order_no VARCHAR(128),
    received_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);
