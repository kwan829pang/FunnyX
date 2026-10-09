-- Indexes across fx_* schemas (ERD relationship notes)

CREATE INDEX IF NOT EXISTS idx_corp_api_keys_corp
    ON fx_corp.corp_api_keys (corporate_user_id, status);

CREATE INDEX IF NOT EXISTS idx_partner_endpoints_corp
    ON fx_corp.partner_endpoints (corporate_user_id, type, status);

CREATE INDEX IF NOT EXISTS idx_platform_fees_corp
    ON fx_corp.platform_fees (corporate_user_id, fee_type, status);

CREATE INDEX IF NOT EXISTS idx_corp_partner_notices_corp
    ON fx_corp.corp_partner_notices (corporate_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_game_coin_supply_requests_corp
    ON fx_corp.game_coin_supply_requests (corporate_user_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_game_coin_supply_requests_game
    ON fx_corp.game_coin_supply_requests (game_id, game_coin_id, status);

CREATE INDEX IF NOT EXISTS idx_corp_payment_records_corp
    ON fx_corp.corp_payment_records (corporate_user_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_corp_payment_records_user
    ON fx_corp.corp_payment_records (end_user_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_corp_payment_records_partner
    ON fx_corp.corp_payment_records (partner_order_no);

CREATE UNIQUE INDEX IF NOT EXISTS uq_user_sessions_token
    ON fx_user.user_sessions (token);

CREATE INDEX IF NOT EXISTS idx_user_sessions_user
    ON fx_user.user_sessions (end_user_id, expires_at);

CREATE INDEX IF NOT EXISTS idx_user_sessions_refresh
    ON fx_user.user_sessions (refresh_token)
    WHERE refresh_token IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_oauth_identities_user
    ON fx_user.oauth_identities (end_user_id, status);

CREATE INDEX IF NOT EXISTS idx_oauth_identities_partner_user
    ON fx_user.oauth_identities (partner_id, partner_user_id);

CREATE INDEX IF NOT EXISTS idx_user_wallets_account
    ON fx_user.user_wallets (game_account_id, game_coin_id);

CREATE INDEX IF NOT EXISTS idx_games_corporate_user
    ON fx_game.games (corporate_user_id, status);

CREATE INDEX IF NOT EXISTS idx_game_accounts_game
    ON fx_game.game_accounts (game_id, status);

CREATE INDEX IF NOT EXISTS idx_game_accounts_end_user
    ON fx_game.game_accounts (end_user_id);

CREATE UNIQUE INDEX IF NOT EXISTS uq_game_accounts_user_game
    ON fx_game.game_accounts (end_user_id, game_id)
    WHERE end_user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_games_partner_code
    ON fx_game.games (partner_code)
    WHERE partner_code IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_game_balances_game
    ON fx_game.game_balances (game_id, game_coin_id);

CREATE INDEX IF NOT EXISTS idx_company_basic_tokens_corp
    ON fx_corp_token.company_basic_tokens (corporate_user_id, status, buyable);

CREATE INDEX IF NOT EXISTS idx_corp_token_orders_user
    ON fx_corp_token.corp_token_orders (end_user_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_corp_token_orders_partner
    ON fx_corp_token.corp_token_orders (partner_order_no);

CREATE INDEX IF NOT EXISTS idx_coin_fee_ledger_token
    ON fx_corp_token.coin_fee_ledger (company_basic_token_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_market_pairs_status
    ON fx_market.market_pairs (status, corporate_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_market_balance_locks_pair
    ON fx_market.market_balance_locks (market_pair_id, status);

CREATE INDEX IF NOT EXISTS idx_orders_market
    ON fx_market.orders (market_pair_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_trades_market
    ON fx_market.trades (market_pair_id, executed_at DESC);

CREATE INDEX IF NOT EXISTS idx_market_ohlcvs_pair_time
    ON fx_market_data.market_ohlcvs (market_pair_id, timeframe, open_time DESC);

CREATE INDEX IF NOT EXISTS idx_marketplace_deals_status
    ON fx_marketplace.marketplace_deals (status, game_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_marketplace_deals_poster
    ON fx_marketplace.marketplace_deals (poster_end_user_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_marketplace_deal_requests_deal
    ON fx_marketplace.marketplace_deal_requests (marketplace_deal_id, status);

CREATE INDEX IF NOT EXISTS idx_marketplace_transfer_events_deal
    ON fx_marketplace.marketplace_transfer_events (marketplace_deal_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_shop_packages_status
    ON fx_shop.shop_packages (status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_corp_shop_products_corp_status
    ON fx_shop.corp_shop_products (corporate_user_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_shop_orders_user_status
    ON fx_shop.shop_orders (end_user_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_shop_orders_partner_order_no
    ON fx_shop.shop_orders (partner_order_no);

CREATE INDEX IF NOT EXISTS idx_shop_payment_events_order
    ON fx_shop.shop_payment_events (shop_order_id, received_at DESC);

CREATE INDEX IF NOT EXISTS idx_deposit_withdrawal_txns_user
    ON fx_money.deposit_withdrawal_txns (end_user_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_deposit_withdrawal_txns_shop
    ON fx_money.deposit_withdrawal_txns (shop_order_id);

CREATE INDEX IF NOT EXISTS idx_deposit_withdrawal_status_logs_txn
    ON fx_money.deposit_withdrawal_status_logs (deposit_withdrawal_txn_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_notifications_user
    ON fx_events.notifications (end_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_outbound_notices_pending
    ON fx_events.outbound_notices (delivery_status, scheduled_at, created_at);

CREATE INDEX IF NOT EXISTS idx_outbound_notices_source
    ON fx_events.outbound_notices (source_type, source_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_outbound_notices_user
    ON fx_events.outbound_notices (end_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_webhook_events_received
    ON fx_events.webhook_events (source, received_at DESC);

CREATE INDEX IF NOT EXISTS idx_webhook_events_retry
    ON fx_events.webhook_events (delivery_status, next_retry_at)
    WHERE delivery_status IN ('failed', 'received', 'processing');

CREATE INDEX IF NOT EXISTS idx_webhook_events_partner_order
    ON fx_events.webhook_events (partner_order_no);

CREATE INDEX IF NOT EXISTS idx_webhook_endpoints_corp
    ON fx_events.webhook_endpoints (corporate_user_id, kind);

CREATE INDEX IF NOT EXISTS idx_admin_action_logs_admin
    ON fx_admin.admin_action_logs (admin_id, created_at DESC);
