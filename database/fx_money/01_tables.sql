-- fx_money tables (2)
-- transaction_type: deposit | withdrawal | shop_topup | corp_shop_purchase
-- deposit_withdrawal_status_logs: status-change audit for money movements
-- First draft: only money has a dedicated status-log table; marketplace / exchange
-- orders / shop rely on row status + fx_events.outbound_notices.

CREATE TABLE IF NOT EXISTS fx_money.deposit_withdrawal_txns (
    id BIGSERIAL PRIMARY KEY,
    end_user_id BIGINT REFERENCES fx_user.end_users(id),
    game_account_id BIGINT REFERENCES fx_game.game_accounts(id),
    corporate_user_id BIGINT REFERENCES fx_corp.corporate_users(id),
    partner_endpoint_id BIGINT REFERENCES fx_corp.partner_endpoints(id),
    shop_order_id BIGINT REFERENCES fx_shop.shop_orders(id),
    transaction_type VARCHAR(32) NOT NULL
        CHECK (transaction_type IN ('deposit', 'withdrawal', 'shop_topup', 'corp_shop_purchase')),
    amount NUMERIC(28,8) NOT NULL,
    game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    status VARCHAR(32) NOT NULL DEFAULT 'pending'
        CHECK (status IN (
            'pending',
            'pending_payment',
            'processing',
            'completed',
            'cancelled',
            'rejected',
            'failed'
        )),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fx_money.deposit_withdrawal_status_logs (
    id BIGSERIAL PRIMARY KEY,
    deposit_withdrawal_txn_id BIGINT NOT NULL
        REFERENCES fx_money.deposit_withdrawal_txns(id),
    old_status VARCHAR(32),
    new_status VARCHAR(32) NOT NULL
        CHECK (new_status IN (
            'pending',
            'pending_payment',
            'processing',
            'completed',
            'cancelled',
            'rejected',
            'failed'
        )),
    event_type VARCHAR(64) NOT NULL
        CHECK (event_type IN (
            'created',
            'pending_payment',
            'completed',
            'cancelled',
            'rejected',
            'failed',
            'status_changed'
        )),
    note TEXT,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at BIGINT NOT NULL
);
