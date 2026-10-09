//! Admin API PostgreSQL SQL constants.
//!
//! All `sqlx` statements for admin store modules live here. Handlers call
//! `crate::query::<area>::CONST` — no inline SQL in stores.
//!
//! # How to read this file
//! - Nested modules match admin features (`cbt`, `shop`, `system`, `market`, `game_coin`).
//! - `$n` bind params follow call-site bind order.
//! - List/detail queries return row shapes used by admin DTOs (`::float8` for decimals).
//! - Approve/reject flows use `FOR UPDATE` then mutate locks/pools/pairs in one txn.
//! - Admin FK writes often use `(SELECT id FROM fx_admin.admin_users WHERE id = $n)`
//!   so a missing demo admin id does not violate FK.
//!
//! # Domains
//! | Module      | Logic                                              | Result |
//! |-------------|----------------------------------------------------|--------|
//! | `cbt`       | List/get CBT; approve/reject; partner notice       | CBT row / notice insert |
//! | `shop`      | Corp product list/get/status + action log          | Product row |
//! | `system`    | Platform packages + `system_settings`              | Package / setting value |
//! | `market`    | List markets; approve (transfer lock→pool) / reject (unlock) | Market detail / status |
//! | `game_coin` | CRUD + status for `fx_game.game_coins` + logs      | Coin row |

/// Corp CBT review: list/filter, get, decision update, notify corp partner.
pub mod cbt {
    pub const LIST: &str = "SELECT id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                        status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                        approved_by_admin_id, approved_at, created_at, updated_at \
                 FROM fx_corp_token.company_basic_tokens \
                 WHERE ($1::text IS NULL OR status = $1) \
                 ORDER BY id";

    pub const GET: &str = "SELECT id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                        status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                        approved_by_admin_id, approved_at, created_at, updated_at \
                 FROM fx_corp_token.company_basic_tokens WHERE id = $1";

    pub const UPDATE_DECISION: &str = "UPDATE fx_corp_token.company_basic_tokens SET \
                    status = $1, buyable = $2, approved_by_admin_id = $3, approved_at = $4, updated_at = $4 \
                 WHERE id = $5 AND status IN ('submitted', 'pending', 'rejected') \
                 RETURNING id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                           status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                           approved_by_admin_id, approved_at, created_at, updated_at";

    pub const INSERT_PARTNER_NOTICE: &str = "INSERT INTO fx_corp.corp_partner_notices ( \
                    corporate_user_id, notice_type, title, body, deadline_at, \
                    created_by_admin_id, read_at, created_at \
                 ) VALUES ($1, $2, $3, $4, 0, $5, 0, $6)";
}

/// Corp shop product moderation (status) and admin action audit log.
pub mod shop {
    pub const LIST: &str = "SELECT pr.id, pr.corporate_user_id, pr.game_id, pr.code, pr.name, pr.product_type, \
                        pr.credit_game_coin_id, gc.code AS credit_game_coin, \
                        pr.credit_amount::float8 AS credit_amount, pr.item_code, \
                        pr.fiat_price::float8 AS fiat_price, pr.seller_type, pr.status, \
                        pr.created_at, pr.updated_at \
                 FROM fx_shop.corp_shop_products pr \
                 LEFT JOIN fx_game.game_coins gc ON gc.id = pr.credit_game_coin_id \
                 WHERE ($1::text IS NULL OR pr.status = $1) \
                   AND ($2::bigint IS NULL OR pr.corporate_user_id = $2) \
                 ORDER BY pr.id";

    pub const GET: &str = "SELECT pr.id, pr.corporate_user_id, pr.game_id, pr.code, pr.name, pr.product_type, \
                        pr.credit_game_coin_id, gc.code AS credit_game_coin, \
                        pr.credit_amount::float8 AS credit_amount, pr.item_code, \
                        pr.fiat_price::float8 AS fiat_price, pr.seller_type, pr.status, \
                        pr.created_at, pr.updated_at \
                 FROM fx_shop.corp_shop_products pr \
                 LEFT JOIN fx_game.game_coins gc ON gc.id = pr.credit_game_coin_id \
                 WHERE pr.id = $1";

    pub const UPDATE_STATUS: &str =
        "UPDATE fx_shop.corp_shop_products SET status = $1, updated_at = $2 WHERE id = $3";

    pub const INSERT_ACTION_LOG: &str =
        "INSERT INTO fx_admin.admin_action_logs (admin_id, action, metadata, created_at, updated_at) \
                 VALUES ($1, 'shop.corp_product.status', $2::jsonb, $3, 0)";
}

/// Platform shop packages and key/value system settings (e.g. FX rates).
pub mod system {
    pub const LIST_PACKAGES: &str =
        "SELECT id, code, name, game_coin_id, coin_amount::float8 AS coin_amount, \
                        fiat_price::float8 AS fiat_price, status \
                 FROM fx_shop.shop_packages ORDER BY id";

    pub const INSERT_PACKAGE: &str = "INSERT INTO fx_shop.shop_packages ( \
                    code, name, game_coin_id, coin_amount, fiat_price, seller_type, status, \
                    created_by_admin_id, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, 'platform', $6, $7, $8, 0) \
                 RETURNING id, code, name, game_coin_id, coin_amount::float8 AS coin_amount, \
                           fiat_price::float8 AS fiat_price, status";

    pub const UPDATE_PACKAGE_STATUS: &str = "UPDATE fx_shop.shop_packages SET status = $1, updated_at = $2 \
                 WHERE id = $3 \
                 RETURNING id, code, name, game_coin_id, coin_amount::float8 AS coin_amount, \
                           fiat_price::float8 AS fiat_price, status";

    pub const SELECT_PLATFORM_TOKEN_COIN_ID: &str = "SELECT id FROM fx_game.game_coins \
                 WHERE is_platform_token = TRUE AND status = 'active' \
                 ORDER BY id LIMIT 1";

    pub const UPDATE_PACKAGE_PRICE: &str = "UPDATE fx_shop.shop_packages SET \
                    fiat_price = $1, \
                    name = COALESCE($2, name), \
                    status = COALESCE($3, status), \
                    updated_at = $4 \
                 WHERE id = $5 \
                 RETURNING id, code, name, game_coin_id, coin_amount::float8 AS coin_amount, \
                           fiat_price::float8 AS fiat_price, status";

    pub const SELECT_SETTING: &str =
        "SELECT setting_value FROM fx_config.system_settings WHERE setting_key = $1";

    pub const UPSERT_SETTING: &str = "INSERT INTO fx_config.system_settings \
                    (setting_key, setting_value, updated_by_admin_id, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $4) \
                 ON CONFLICT (setting_key) DO UPDATE SET \
                    setting_value = EXCLUDED.setting_value, \
                    updated_by_admin_id = EXCLUDED.updated_by_admin_id, \
                    updated_at = EXCLUDED.updated_at";
}

/// Market pair ops: list (+ pool/lock join), approve (consume lock → pool wallet), reject (unlock).
pub mod market {
    pub const LIST: &str = "SELECT p.id, p.corporate_user_id, p.game_id, p.base_game_coin_id, p.quote_game_coin_id, \
        bc.code AS base_game_coin, qc.code AS quote_game_coin, \
        p.market_name, p.funding_source, p.status, p.created_at, p.updated_at, \
        po.id AS pool_id, po.pool_depth::float8 AS pool_depth, \
        po.initial_price::float8 AS initial_price, po.base_amount::float8 AS base_amount, \
        po.quote_amount::float8 AS quote_amount, po.status AS pool_status, \
        l.id AS lock_id, l.game_coin_id AS lock_game_coin_id, l.locked_amount::float8 AS lock_amount \
 FROM fx_market.market_pairs p \
 LEFT JOIN fx_game.game_coins bc ON bc.id = p.base_game_coin_id \
 LEFT JOIN fx_game.game_coins qc ON qc.id = p.quote_game_coin_id \
 LEFT JOIN fx_market.market_pools po ON po.market_pair_id = p.id \
 LEFT JOIN fx_market.market_balance_locks l ON l.market_pair_id = p.id \
   AND l.id = (SELECT MAX(id) FROM fx_market.market_balance_locks WHERE market_pair_id = p.id) \
 ORDER BY p.id DESC";

    pub const LIST_FILTER: &str = "SELECT p.id, p.corporate_user_id, p.game_id, p.base_game_coin_id, p.quote_game_coin_id, \
        bc.code AS base_game_coin, qc.code AS quote_game_coin, \
        p.market_name, p.funding_source, p.status, p.created_at, p.updated_at, \
        po.id AS pool_id, po.pool_depth::float8 AS pool_depth, \
        po.initial_price::float8 AS initial_price, po.base_amount::float8 AS base_amount, \
        po.quote_amount::float8 AS quote_amount, po.status AS pool_status, \
        l.id AS lock_id, l.game_coin_id AS lock_game_coin_id, l.locked_amount::float8 AS lock_amount \
 FROM fx_market.market_pairs p \
 LEFT JOIN fx_game.game_coins bc ON bc.id = p.base_game_coin_id \
 LEFT JOIN fx_game.game_coins qc ON qc.id = p.quote_game_coin_id \
 LEFT JOIN fx_market.market_pools po ON po.market_pair_id = p.id \
 LEFT JOIN fx_market.market_balance_locks l ON l.market_pair_id = p.id \
   AND l.id = (SELECT MAX(id) FROM fx_market.market_balance_locks WHERE market_pair_id = p.id) \
 WHERE p.status = $1 \
 ORDER BY p.id DESC";

    pub const LIST_ONE: &str = "SELECT p.id, p.corporate_user_id, p.game_id, p.base_game_coin_id, p.quote_game_coin_id, \
        bc.code AS base_game_coin, qc.code AS quote_game_coin, \
        p.market_name, p.funding_source, p.status, p.created_at, p.updated_at, \
        po.id AS pool_id, po.pool_depth::float8 AS pool_depth, \
        po.initial_price::float8 AS initial_price, po.base_amount::float8 AS base_amount, \
        po.quote_amount::float8 AS quote_amount, po.status AS pool_status, \
        l.id AS lock_id, l.game_coin_id AS lock_game_coin_id, l.locked_amount::float8 AS lock_amount \
 FROM fx_market.market_pairs p \
 LEFT JOIN fx_game.game_coins bc ON bc.id = p.base_game_coin_id \
 LEFT JOIN fx_game.game_coins qc ON qc.id = p.quote_game_coin_id \
 LEFT JOIN fx_market.market_pools po ON po.market_pair_id = p.id \
 LEFT JOIN fx_market.market_balance_locks l ON l.market_pair_id = p.id \
   AND l.id = (SELECT MAX(id) FROM fx_market.market_balance_locks WHERE market_pair_id = p.id) \
 WHERE p.id = $1";

    pub const SELECT_PAIR_FOR_APPROVE: &str =
        "SELECT id, corporate_user_id, game_id, base_game_coin_id, quote_game_coin_id, \
                        market_name, funding_source, status \
                 FROM fx_market.market_pairs WHERE id = $1 FOR UPDATE";

    pub const SELECT_LOCK_FOR_UPDATE: &str =
        "SELECT id, game_coin_id, locked_amount::float8 AS locked_amount \
                 FROM fx_market.market_balance_locks \
                 WHERE market_pair_id = $1 AND status = 'locked' \
                 ORDER BY id LIMIT 1 FOR UPDATE";

    pub const SELECT_POOL_FOR_UPDATE: &str =
        "SELECT id, pool_depth::float8 AS pool_depth, initial_price::float8 AS initial_price, \
                        base_amount::float8 AS base_amount, quote_amount::float8 AS quote_amount, status \
                 FROM fx_market.market_pools WHERE market_pair_id = $1 FOR UPDATE";

    pub const UPDATE_GAME_BALANCE_CONSUME_LOCK: &str = "UPDATE fx_game.game_balances \
                 SET locked_balance = locked_balance - $1, updated_at = $2 \
                 WHERE game_id = $3 AND game_coin_id = $4 AND locked_balance >= $1";

    pub const UPSERT_POOL_WALLET: &str =
        "INSERT INTO fx_market.pool_wallets (market_pool_id, game_coin_id, balance, wallet_type, status) \
                     VALUES ($1, $2, $3, 'market_pool', 'active') \
                     ON CONFLICT (market_pool_id, game_coin_id) DO UPDATE SET \
                        balance = fx_market.pool_wallets.balance + EXCLUDED.balance";

    pub const UPDATE_LOCK_TRANSFERRED: &str = "UPDATE fx_market.market_balance_locks \
                 SET status = 'transferred', updated_at = $2 WHERE id = $1";

    pub const UPDATE_POOL_ACTIVE: &str =
        "UPDATE fx_market.market_pools SET status = 'active', updated_at = $2 WHERE id = $1";

    pub const UPDATE_PAIR_ACTIVE: &str = "UPDATE fx_market.market_pairs \
                 SET status = 'active', \
                     approved_by_admin_id = (SELECT id FROM fx_admin.admin_users WHERE id = $2), \
                     updated_at = $3 WHERE id = $1";

    pub const INSERT_POOL_TRANSFER_EVENT: &str = "INSERT INTO fx_market.pool_transfer_events ( \
                    market_pair_id, market_pool_id, market_balance_lock_id, event_type, amount, created_at \
                 ) VALUES ($1, $2, $3, 'transfer_to_pool', $4, $5)";

    pub const INSERT_STATUS_LOG_APPROVE: &str = "INSERT INTO fx_market.market_status_logs ( \
                    market_pair_id, old_status, new_status, updated_by_admin_id, updated_at, created_at \
                 ) VALUES ($1, 'pending_locked', 'active', \
                    (SELECT id FROM fx_admin.admin_users WHERE id = $2), $3, $3)";

    pub const INSERT_ACTION_LOG_APPROVE: &str =
        "INSERT INTO fx_admin.admin_action_logs (admin_id, action, metadata, created_at, updated_at) \
                 SELECT id, 'market.approve', $2::jsonb, $3, 0 FROM fx_admin.admin_users WHERE id = $1";

    pub const SELECT_PAIR_FOR_REJECT: &str =
        "SELECT id, game_id, status FROM fx_market.market_pairs WHERE id = $1 FOR UPDATE";

    pub const UPDATE_GAME_BALANCE_UNLOCK: &str = "UPDATE fx_game.game_balances \
                     SET locked_balance = locked_balance - $1, \
                         available_balance = available_balance + $1, \
                         updated_at = $2 \
                     WHERE game_id = $3 AND game_coin_id = $4 AND locked_balance >= $1";

    pub const UPDATE_LOCK_UNLOCKED: &str = "UPDATE fx_market.market_balance_locks \
                     SET status = 'unlocked', updated_at = $2 WHERE id = $1";

    pub const UPDATE_POOL_CANCELLED: &str =
        "UPDATE fx_market.market_pools SET status = 'cancelled', updated_at = $2 \
                 WHERE market_pair_id = $1 AND status = 'pending'";

    pub const UPDATE_PAIR_REJECTED: &str = "UPDATE fx_market.market_pairs \
                 SET status = 'rejected', \
                     approved_by_admin_id = (SELECT id FROM fx_admin.admin_users WHERE id = $2), \
                     updated_at = $3 WHERE id = $1";

    pub const INSERT_STATUS_LOG_REJECT: &str = "INSERT INTO fx_market.market_status_logs ( \
                    market_pair_id, old_status, new_status, updated_by_admin_id, updated_at, created_at \
                 ) VALUES ($1, $2, 'rejected', \
                    (SELECT id FROM fx_admin.admin_users WHERE id = $3), $4, $4)";

    pub const INSERT_ACTION_LOG_REJECT: &str =
        "INSERT INTO fx_admin.admin_action_logs (admin_id, action, metadata, created_at, updated_at) \
                 SELECT id, 'market.reject', $2::jsonb, $3, 0 FROM fx_admin.admin_users WHERE id = $1";
}

/// Game-coin catalog CRUD/status with admin_action_logs.
pub mod game_coin {
    pub const LIST: &str = "SELECT id, code, name, type, asset_kind, is_platform_token, status, \
                        created_at, updated_at \
                 FROM fx_game.game_coins ORDER BY id";

    pub const GET: &str = "SELECT id, code, name, type, asset_kind, is_platform_token, status, \
                        created_at, updated_at \
                 FROM fx_game.game_coins WHERE id = $1";

    pub const INSERT: &str = "INSERT INTO fx_game.game_coins ( \
                    code, name, type, asset_kind, is_platform_token, status, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, 'active', $6, 0) \
                 RETURNING id, code, name, type, asset_kind, is_platform_token, status, created_at, updated_at";

    pub const INSERT_CREATE_ACTION_LOG: &str =
        "INSERT INTO fx_admin.admin_action_logs (admin_id, action, metadata, created_at, updated_at) \
                 SELECT id, 'game_coin.create', $2::jsonb, $3, 0 FROM fx_admin.admin_users WHERE id = $1";

    pub const UPDATE: &str =
        "UPDATE fx_game.game_coins SET name = $2, asset_kind = $3, updated_at = $4 WHERE id = $1";

    pub const INSERT_UPDATE_ACTION_LOG: &str =
        "INSERT INTO fx_admin.admin_action_logs (admin_id, action, metadata, created_at, updated_at) \
                 SELECT id, 'game_coin.update', $2::jsonb, $3, 0 FROM fx_admin.admin_users WHERE id = $1";

    pub const UPDATE_STATUS: &str =
        "UPDATE fx_game.game_coins SET status = $2, updated_at = $3 WHERE id = $1";

    pub const INSERT_STATUS_ACTION_LOG: &str =
        "INSERT INTO fx_admin.admin_action_logs (admin_id, action, metadata, created_at, updated_at) \
                 SELECT id, 'game_coin.status', $2::jsonb, $3, 0 FROM fx_admin.admin_users WHERE id = $1";
}
