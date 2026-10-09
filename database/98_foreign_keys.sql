-- Deferred cross-schema foreign keys (run after all tables exist)
-- Safe to re-run: drops constraint if present then adds (or use IF NOT EXISTS pattern)

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_platform_fees_related_game'
    ) THEN
        ALTER TABLE fx_corp.platform_fees
            ADD CONSTRAINT fk_platform_fees_related_game
            FOREIGN KEY (related_game_id) REFERENCES fx_game.games(id);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_platform_fees_related_market_pair'
    ) THEN
        ALTER TABLE fx_corp.platform_fees
            ADD CONSTRAINT fk_platform_fees_related_market_pair
            FOREIGN KEY (related_market_pair_id) REFERENCES fx_market.market_pairs(id);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_corp_partner_notices_admin'
    ) THEN
        ALTER TABLE fx_corp.corp_partner_notices
            ADD CONSTRAINT fk_corp_partner_notices_admin
            FOREIGN KEY (created_by_admin_id) REFERENCES fx_admin.admin_users(id);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_marketplace_deals_matched_request'
    ) THEN
        ALTER TABLE fx_marketplace.marketplace_deals
            ADD CONSTRAINT fk_marketplace_deals_matched_request
            FOREIGN KEY (matched_request_id)
            REFERENCES fx_marketplace.marketplace_deal_requests(id);
    END IF;
END $$;
