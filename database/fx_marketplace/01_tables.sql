-- fx_marketplace tables (3)
-- C7 user-to-user deals (not exchange ORDER/TRADE; settle via partner Transfer API)
-- Source: doc/marketplace.md, doc/partner.md §1.4
--
-- Game items:
--   Partner Item List API returns opaque item ids; store as offer_item_ref_id / item_ref_id
--   (VARCHAR). No platform item catalog FK and no item escrow table in first draft.
--   Ownership / settle: partner Transfer API + deal status.
-- App invariants (not enforced by CHECK):
--   partner_endpoint_id must reference partner_endpoints.type = 'transfer' (enabled).
--   want_asset_type = platform_token → want_game_coin is platform token;
--   want_asset_type = company_coin → want_game_coin belongs to deal corporate_user_id only.

CREATE TABLE IF NOT EXISTS fx_marketplace.marketplace_deals (
    id BIGSERIAL PRIMARY KEY,
    poster_end_user_id BIGINT NOT NULL REFERENCES fx_user.end_users(id),
    poster_game_account_id BIGINT NOT NULL REFERENCES fx_game.game_accounts(id),
    game_id BIGINT NOT NULL REFERENCES fx_game.games(id),
    corporate_user_id BIGINT NOT NULL REFERENCES fx_corp.corporate_users(id),
    -- Must be partner_endpoints.type = 'transfer' (app check)
    partner_endpoint_id BIGINT NOT NULL REFERENCES fx_corp.partner_endpoints(id),
    -- What the poster offers (sell side)
    offer_asset_type VARCHAR(32) NOT NULL
        CHECK (offer_asset_type IN ('game_coin', 'company_coin', 'game_item')),
    offer_game_coin_id BIGINT REFERENCES fx_game.game_coins(id),
    -- Partner Item List API id when offer_asset_type = game_item (opaque VARCHAR refer)
    offer_item_ref_id VARCHAR(128),
    offer_amount NUMERIC(28,8) NOT NULL,
    -- What the poster wants in return (Platform Token or parent Company Coin only)
    want_asset_type VARCHAR(32) NOT NULL
        CHECK (want_asset_type IN ('platform_token', 'company_coin')),
    want_game_coin_id BIGINT NOT NULL REFERENCES fx_game.game_coins(id),
    want_amount NUMERIC(28,8) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'open'
        CHECK (status IN (
            'open',
            'requested',
            'matched',
            'settling',
            'completed',
            'cancelled',
            'failed'
        )),
    matched_request_id BIGINT,
    cancelled_at BIGINT NOT NULL DEFAULT 0,
    completed_at BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT chk_marketplace_deals_offer_shape CHECK (
        (offer_asset_type = 'game_item' AND offer_item_ref_id IS NOT NULL)
        OR (offer_asset_type IN ('game_coin', 'company_coin') AND offer_game_coin_id IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS fx_marketplace.marketplace_deal_requests (
    id BIGSERIAL PRIMARY KEY,
    marketplace_deal_id BIGINT NOT NULL
        REFERENCES fx_marketplace.marketplace_deals(id),
    requester_end_user_id BIGINT NOT NULL REFERENCES fx_user.end_users(id),
    requester_game_account_id BIGINT NOT NULL REFERENCES fx_game.game_accounts(id),
    status VARCHAR(32) NOT NULL DEFAULT 'pending'
        CHECK (status IN (
            'pending',
            'accepted',
            'rejected',
            'cancelled',
            'completed',
            'failed'
        )),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (marketplace_deal_id, requester_end_user_id)
);

-- Deferred self-FK: matched_request_id → marketplace_deal_requests (see 98_foreign_keys.sql)

CREATE TABLE IF NOT EXISTS fx_marketplace.marketplace_transfer_events (
    id BIGSERIAL PRIMARY KEY,
    marketplace_deal_id BIGINT NOT NULL
        REFERENCES fx_marketplace.marketplace_deals(id),
    marketplace_deal_request_id BIGINT
        REFERENCES fx_marketplace.marketplace_deal_requests(id),
    -- Must be partner_endpoints.type = 'transfer' (app check)
    partner_endpoint_id BIGINT NOT NULL REFERENCES fx_corp.partner_endpoints(id),
    request_id VARCHAR(128) NOT NULL,
    from_game_account_id BIGINT NOT NULL REFERENCES fx_game.game_accounts(id),
    to_game_account_id BIGINT NOT NULL REFERENCES fx_game.game_accounts(id),
    asset_type VARCHAR(32) NOT NULL
        CHECK (asset_type IN ('game_coin', 'company_coin', 'game_item', 'platform_token')),
    game_coin_id BIGINT REFERENCES fx_game.game_coins(id),
    -- Same partner Item List refer id used on Transfer settle when asset_type = game_item
    item_ref_id VARCHAR(128),
    amount NUMERIC(28,8) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'sent', 'succeeded', 'failed')),
    payload JSONB,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (request_id)
);
