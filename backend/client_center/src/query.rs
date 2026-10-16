//! Client Center PostgreSQL SQL constants.
//!
//! All `sqlx` statements used by this service live here as `&str` constants.
//! Store / domain modules call `crate::query::<area>::CONST` — they must not
//! inline SQL.
//!
//! # How to read this file
//! - Nested modules mirror domains (`users`, `wallet`, `shop`, …).
//! - `$1`, `$2`, … are bind parameters in call-site order.
//! - `RETURNING` / `SELECT` columns are what the Rust row types map from.
//! - Numeric amounts are often cast `::float8` for JSON-friendly decoding.
//! - Timestamps are epoch ints (`created_at`, `updated_at`); `0` means unset.
//!
//! # Domains (schemas)
//! | Module     | Primary schemas / tables                         | Typical result |
//! |------------|--------------------------------------------------|----------------|
//! | `users`    | `fx_user.end_users`, `oauth_identities`          | User / OAuth rows |
//! | `bindings` | `fx_game.game_accounts` (+ `games`)              | Game account binds |
//! | `sessions` | `fx_user.user_sessions`                          | Session insert / revoke |
//! | `games`    | `fx_game.games`                                  | Active game catalog |
//! | `corp`     | `fx_corp.corporate_users`                        | Master-corp gate |
//! | `wallet`   | `fx_user.user_wallets`, `fx_money.*`             | Balances & D/W txns |
//! | `shop`     | `fx_shop.*`, wallet credit on settle             | Packages / orders |
//! | `cbt`      | `fx_corp_token.*`                                | Corp tokens & buys |
//! | `market`   | `fx_market.*`, `fx_game.game_balances`           | Pair create + locks |

/// End-user auth, registration, and partner OAuth identity links.
pub mod users {
    pub const DEMO_PASSWORD_HASHES: &str = "SELECT id, username, password_hash FROM fx_user.end_users \
             WHERE username IN ('demo_user', 'alice_plat')";

    pub const UPDATE_PASSWORD_HASH: &str =
        "UPDATE fx_user.end_users SET password_hash = $1, updated_at = $2 WHERE id = $3";

    /// Seeded corp logins (contact_email / master_code); NULL password_hash → fill "demo".
    pub const DEMO_CORP_PASSWORD_HASHES: &str = "SELECT id, master_code, password_hash \
             FROM fx_corp.corporate_users \
             WHERE master_code IN ('DEMO_MASTER_CODE', 'DEMO_CORP_B_CODE')";

    pub const UPDATE_CORP_PASSWORD_HASH: &str =
        "UPDATE fx_corp.corporate_users SET password_hash = $1, updated_at = $2 WHERE id = $3";

    pub const BY_ID: &str = "SELECT id, username, email, password_hash, status \
                     FROM fx_user.end_users WHERE id = $1";

    pub const BY_USERNAME: &str = "SELECT id, username, email, password_hash, status \
                     FROM fx_user.end_users WHERE username = $1";

    pub const INSERT_REGISTER: &str =
        "INSERT INTO fx_user.end_users (username, email, password_hash, status, created_at, updated_at) \
                 VALUES ($1, $2, $3, 'active', $4, 0) \
                 RETURNING id, username, email, password_hash, status";

    pub const UPSERT_FROM_SESSION: &str = "INSERT INTO fx_user.end_users \
                    (id, username, email, password_hash, status, created_at, updated_at) \
                 VALUES ($1, $2, NULL, NULL, 'active', $3, 0) \
                 ON CONFLICT (id) DO UPDATE SET updated_at = EXCLUDED.updated_at \
                 RETURNING id, username, email, password_hash, status";

    pub const UPSERT_OAUTH_IDENTITY: &str = "INSERT INTO fx_user.oauth_identities \
                    (end_user_id, provider, partner_id, partner_user_id, game_id, game_account_id, status, created_at, updated_at) \
                 VALUES ($1, 'partner', $2, $3, $4, $5, 'active', $6, 0) \
                 ON CONFLICT (provider, partner_id, partner_user_id) DO UPDATE SET \
                    end_user_id = EXCLUDED.end_user_id, \
                    game_id = COALESCE(EXCLUDED.game_id, fx_user.oauth_identities.game_id), \
                    game_account_id = COALESCE(EXCLUDED.game_account_id, fx_user.oauth_identities.game_account_id), \
                    updated_at = $6";

    pub const LINKS_FOR_USER: &str =
        "SELECT partner_id, partner_user_id, end_user_id, game_id, game_account_id \
                 FROM fx_user.oauth_identities \
                 WHERE end_user_id = $1 AND status = 'active'";

    pub const OAUTH_IDENTITY_ID: &str = "SELECT id FROM fx_user.oauth_identities \
             WHERE provider = 'partner' AND partner_id = $1 AND partner_user_id = $2";
}

/// Game-account bindings for an end user (join `game_accounts` ↔ `games`).
pub mod bindings {
    pub const LIST_FOR_USER: &str =
        "SELECT a.id, a.end_user_id, a.game_id, g.game_code, a.game_account_id, \
                        COALESCE(g.partner_code, '') AS partner_id, \
                        a.partner_user_id, a.bind_source, a.status \
                 FROM fx_game.game_accounts a \
                 JOIN fx_game.games g ON g.id = a.game_id \
                 WHERE a.end_user_id = $1 \
                 ORDER BY a.id";

    pub const BY_USER_AND_GAME: &str =
        "SELECT a.id, a.end_user_id, a.game_id, g.game_code, a.game_account_id, \
                COALESCE(g.partner_code, '') AS partner_id, \
                a.partner_user_id, a.bind_source, a.status \
         FROM fx_game.game_accounts a \
         JOIN fx_game.games g ON g.id = a.game_id \
         WHERE a.end_user_id = $1 AND a.game_id = $2";

    pub const UPDATE: &str = "UPDATE fx_game.game_accounts \
             SET game_account_id = $1, partner_user_id = $2, bind_source = $3, \
                 status = 'active', updated_at = $4 \
             WHERE id = $5";

    pub const BY_ID: &str =
        "SELECT a.id, a.end_user_id, a.game_id, g.game_code, a.game_account_id, \
                    COALESCE(g.partner_code, '') AS partner_id, \
                    a.partner_user_id, a.bind_source, a.status \
             FROM fx_game.game_accounts a \
             JOIN fx_game.games g ON g.id = a.game_id \
             WHERE a.id = $1";

    pub const INSERT_UPSERT: &str = "INSERT INTO fx_game.game_accounts \
            (game_id, end_user_id, game_account_id, partner_user_id, bind_source, status, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, 'active', $6, 0) \
         ON CONFLICT (game_id, game_account_id) DO UPDATE SET \
            end_user_id = EXCLUDED.end_user_id, \
            partner_user_id = EXCLUDED.partner_user_id, \
            bind_source = EXCLUDED.bind_source, \
            status = 'active', \
            updated_at = EXCLUDED.created_at";

    pub const BY_GAME_AND_ACCOUNT: &str =
        "SELECT a.id, a.end_user_id, a.game_id, g.game_code, a.game_account_id, \
                COALESCE(g.partner_code, '') AS partner_id, \
                a.partner_user_id, a.bind_source, a.status \
         FROM fx_game.game_accounts a \
         JOIN fx_game.games g ON g.id = a.game_id \
         WHERE a.game_id = $1 AND a.game_account_id = $2";
}

/// Access/refresh session rows: insert on login, revoke on logout.
pub mod sessions {
    pub const INSERT: &str = "INSERT INTO fx_user.user_sessions ( \
            end_user_id, token, refresh_token, token_type, grant_type, scope, actor_type, \
            partner_id, partner_user_id, game_account_id, oauth_identity_id, \
            expires_at, refresh_expires_at, revoked_at, created_at, updated_at \
         ) VALUES ( \
            $1, $2, $3, $4, $5, $6, 'end_user', \
            $7, $8, $9, $10, \
            $11, $12, 0, $13, 0 \
         ) \
         ON CONFLICT (token) DO UPDATE SET \
            refresh_token = EXCLUDED.refresh_token, \
            grant_type = EXCLUDED.grant_type, \
            partner_id = EXCLUDED.partner_id, \
            partner_user_id = EXCLUDED.partner_user_id, \
            game_account_id = EXCLUDED.game_account_id, \
            oauth_identity_id = EXCLUDED.oauth_identity_id, \
            expires_at = EXCLUDED.expires_at, \
            revoked_at = 0, \
            updated_at = EXCLUDED.created_at";

    pub const REVOKE_BY_TOKEN: &str =
        "UPDATE fx_user.user_sessions SET revoked_at = $1, updated_at = $1 \
             WHERE token = $2 AND revoked_at = 0";

    pub const REVOKE_BY_REFRESH: &str =
        "UPDATE fx_user.user_sessions SET revoked_at = $1, updated_at = $1 \
             WHERE refresh_token = $2 AND revoked_at = 0";
}

/// Game catalog lookups by id, code, or partner identifiers.
pub mod games {
    pub const LIST_ACTIVE: &str = "SELECT id, game_code, game_name, status, \
                        COALESCE(partner_code, '') AS partner_id, \
                        COALESCE(partner_game_id, '') AS partner_game_id \
                 FROM fx_game.games \
                 WHERE status = 'active' \
                 ORDER BY id";

    pub const BY_ID: &str = "SELECT id, game_code, game_name, status, \
                        COALESCE(partner_code, '') AS partner_id, \
                        COALESCE(partner_game_id, '') AS partner_game_id \
                 FROM fx_game.games WHERE id = $1";

    pub const BY_CODE: &str = "SELECT id, game_code, game_name, status, \
                        COALESCE(partner_code, '') AS partner_id, \
                        COALESCE(partner_game_id, '') AS partner_game_id \
                 FROM fx_game.games WHERE lower(game_code) = lower($1)";

    pub const BY_PARTNER_GAME_ID: &str = "SELECT id, game_code, game_name, status, \
                        COALESCE(partner_code, '') AS partner_id, \
                        COALESCE(partner_game_id, '') AS partner_game_id \
                 FROM fx_game.games WHERE partner_game_id = $1";

    pub const BY_PARTNER_CODE: &str = "SELECT id, game_code, game_name, status, \
                        COALESCE(partner_code, '') AS partner_id, \
                        COALESCE(partner_game_id, '') AS partner_game_id \
                 FROM fx_game.games \
                 WHERE partner_code = $1 AND status = 'active' \
                 ORDER BY id LIMIT 1";
}

/// Corporate master-user check before corp-scoped APIs.
pub mod corp {
    pub const REQUIRE_MASTER: &str =
        "SELECT u.id FROM fx_corp.corporate_users u \
             JOIN fx_corp.corp_api_keys k ON k.corporate_user_id = u.id \
             WHERE u.master_code = $1 AND u.master_id = $2 AND k.api_key = $3 \
               AND u.status = 'active' AND k.status = 'active' AND u.api_enabled = TRUE \
               AND (u.license_deadline_at = 0 OR u.license_deadline_at > (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT)";
}

/// Corp onboarding wizards: register (BR), web login/session, game submit + docs.
pub mod corp_onboard {
    pub const INSERT_CORP: &str = "INSERT INTO fx_corp.corporate_users ( \
            company_name, master_code, master_id, status, api_enabled, \
            contact_email, contact_name, password_hash, created_at, updated_at \
         ) VALUES ($1,$2,$3,'pending',FALSE,$4,$5,$6,$7,0) RETURNING id";

    pub const INSERT_API_KEY: &str = "INSERT INTO fx_corp.corp_api_keys ( \
            corporate_user_id, api_key, secret_hash, status, created_at, updated_at \
         ) VALUES ($1,$2,$3,'active',$4,0)";

    pub const INSERT_DOCUMENT: &str = "INSERT INTO fx_corp.corp_documents ( \
            corporate_user_id, game_id, doc_type, file_name, stored_path, content_type, \
            byte_size, status, created_at, updated_at \
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,'submitted',$8,0) RETURNING id";

    pub const INSERT_SESSION: &str = "INSERT INTO fx_corp.corp_web_sessions ( \
            corporate_user_id, token, expires_at, created_at \
         ) VALUES ($1,$2,$3,$4)";

    pub const SESSION_BY_TOKEN: &str =
        "SELECT s.corporate_user_id, u.status \
         FROM fx_corp.corp_web_sessions s \
         JOIN fx_corp.corporate_users u ON u.id = s.corporate_user_id \
         WHERE s.token = $1 AND s.expires_at > $2";

    pub const BY_EMAIL: &str =
        "SELECT id, company_name, master_code, master_id, status, api_enabled, \
                contact_email, contact_name, password_hash, reject_reason, \
                approved_at, created_at, updated_at \
         FROM fx_corp.corporate_users WHERE lower(contact_email) = lower($1)";

    pub const BY_MASTER_CODE: &str =
        "SELECT id, company_name, master_code, master_id, status, api_enabled, \
                contact_email, contact_name, password_hash, reject_reason, \
                approved_at, created_at, updated_at \
         FROM fx_corp.corporate_users WHERE master_code = $1";

    pub const BY_ID: &str =
        "SELECT id, company_name, master_code, master_id, status, api_enabled, \
                contact_email, contact_name, password_hash, reject_reason, \
                approved_at, created_at, updated_at \
         FROM fx_corp.corporate_users WHERE id = $1";

    pub const LIST_DOCS_FOR_CORP: &str =
        "SELECT id, corporate_user_id, game_id, doc_type, file_name, stored_path, \
                content_type, byte_size, status, review_note, created_at \
         FROM fx_corp.corp_documents WHERE corporate_user_id = $1 \
         ORDER BY id";

    pub const DOC_BY_ID: &str =
        "SELECT id, corporate_user_id, game_id, doc_type, file_name, stored_path, \
                content_type, byte_size, status, review_note, created_at \
         FROM fx_corp.corp_documents WHERE id = $1";

    pub const INSERT_GAME: &str = "INSERT INTO fx_game.games ( \
            corporate_user_id, game_name, game_code, partner_code, partner_game_id, \
            status, description, created_at, updated_at \
         ) VALUES ($1,$2,$3,$4,$5,'pending',$6,$7,0) RETURNING id";

    pub const LIST_GAMES_FOR_CORP: &str =
        "SELECT id, corporate_user_id, game_name, game_code, \
                COALESCE(partner_code,'') AS partner_code, \
                COALESCE(partner_game_id,'') AS partner_game_id, \
                status, COALESCE(description,'') AS description, \
                COALESCE(reject_reason,'') AS reject_reason, \
                approved_at, created_at, updated_at \
         FROM fx_game.games WHERE corporate_user_id = $1 ORDER BY id DESC";

    pub const GAME_BY_ID: &str =
        "SELECT id, corporate_user_id, game_name, game_code, \
                COALESCE(partner_code,'') AS partner_code, \
                COALESCE(partner_game_id,'') AS partner_game_id, \
                status, COALESCE(description,'') AS description, \
                COALESCE(reject_reason,'') AS reject_reason, \
                approved_at, created_at, updated_at \
         FROM fx_game.games WHERE id = $1";

    pub const LIST_DOCS_FOR_GAME: &str =
        "SELECT id, corporate_user_id, game_id, doc_type, file_name, stored_path, \
                content_type, byte_size, status, review_note, created_at \
         FROM fx_corp.corp_documents WHERE game_id = $1 ORDER BY id";
}

/// User wallets and deposit/withdrawal lifecycle (pending → completed/failed).
/// Completing a deposit credits `user_wallets`; withdraw debits available balance.
pub mod wallet {
    pub const LIST_FOR_USER: &str =
        "SELECT w.game_account_id AS row_id, a.game_account_id AS ga_str, a.game_id, \
                    g.game_code, w.game_coin_id, c.code AS game_coin, \
                    w.available::float8 AS available, w.locked::float8 AS locked \
             FROM fx_user.user_wallets w \
             JOIN fx_game.game_accounts a ON a.id = w.game_account_id \
             JOIN fx_game.games g ON g.id = a.game_id \
             JOIN fx_game.game_coins c ON c.id = w.game_coin_id \
             WHERE a.end_user_id = $1 \
             ORDER BY w.game_account_id, c.code";

    pub const LIST_TRANSACTIONS: &str =
        "SELECT t.id, t.end_user_id, t.game_account_id, t.transaction_type, \
                    t.amount::float8 AS amount, t.game_coin_id, c.code AS game_coin, \
                    t.status, t.created_at, t.updated_at \
             FROM fx_money.deposit_withdrawal_txns t \
             LEFT JOIN fx_game.game_coins c ON c.id = t.game_coin_id \
             WHERE t.end_user_id = $1 \
             ORDER BY t.id DESC \
             LIMIT $2";

    pub const OWNED_ACCOUNT: &str =
        "SELECT a.id, a.game_id, g.game_code, a.game_account_id, \
                    COALESCE(g.partner_code, '') AS partner_id, \
                    COALESCE(g.partner_game_id, '') AS partner_game_id \
             FROM fx_game.game_accounts a \
             JOIN fx_game.games g ON g.id = a.game_id \
             WHERE a.id = $1 AND a.end_user_id = $2 AND a.status = 'active'";

    pub const RESOLVE_COIN: &str = "SELECT id, code FROM fx_game.game_coins \
             WHERE code = $1 AND status = 'active'";

    pub const AVAILABLE_BALANCE: &str =
        "SELECT available::float8 FROM fx_user.user_wallets \
             WHERE game_account_id = $1 AND game_coin_id = $2";

    pub const INSERT_PENDING_TXN: &str = "INSERT INTO fx_money.deposit_withdrawal_txns ( \
                end_user_id, game_account_id, transaction_type, amount, game_coin_id, \
                status, created_at, updated_at \
             ) VALUES ($1, $2, $3, $4, $5, 'pending', $6, 0) \
             RETURNING id";

    pub const INSERT_STATUS_LOG_CREATED: &str =
        "INSERT INTO fx_money.deposit_withdrawal_status_logs ( \
                deposit_withdrawal_txn_id, old_status, new_status, event_type, note, payload, created_at \
             ) VALUES ($1, NULL, 'pending', 'created', $2, $3::jsonb, $4)";

    pub const MARK_FAILED: &str =
        "UPDATE fx_money.deposit_withdrawal_txns \
             SET status = 'failed', updated_at = $2 WHERE id = $1 AND status = 'pending'";

    pub const INSERT_STATUS_LOG_FAILED: &str =
        "INSERT INTO fx_money.deposit_withdrawal_status_logs ( \
                deposit_withdrawal_txn_id, old_status, new_status, event_type, note, payload, created_at \
             ) VALUES ($1, 'pending', 'failed', 'failed', $2, $3::jsonb, $4)";

    pub const COMPLETED_LOG_EXISTS: &str = "SELECT EXISTS( \
                SELECT 1 FROM fx_money.deposit_withdrawal_status_logs \
                WHERE deposit_withdrawal_txn_id = $1 AND event_type = 'completed' \
             )";

    pub const MARK_COMPLETED: &str = "UPDATE fx_money.deposit_withdrawal_txns \
             SET status = 'completed', updated_at = $2 \
             WHERE id = $1 AND status IN ('pending', 'processing')";

    pub const UPSERT_WALLET_CREDIT: &str =
        "INSERT INTO fx_user.user_wallets (game_account_id, game_coin_id, available, locked, updated_at) \
                 VALUES ($1, $2, $3, 0, $4) \
                 ON CONFLICT (game_account_id, game_coin_id) DO UPDATE SET \
                    available = fx_user.user_wallets.available + EXCLUDED.available, \
                    updated_at = EXCLUDED.updated_at";

    pub const DEBIT_WALLET: &str = "UPDATE fx_user.user_wallets \
                 SET available = available - $3, updated_at = $4 \
                 WHERE game_account_id = $1 AND game_coin_id = $2 AND available >= $3";

    pub const INSERT_STATUS_LOG_COMPLETED: &str =
        "INSERT INTO fx_money.deposit_withdrawal_status_logs ( \
                deposit_withdrawal_txn_id, old_status, new_status, event_type, note, payload, created_at \
             ) VALUES ($1, 'pending', 'completed', 'completed', $2, $3::jsonb, $4)";

    pub const GET_TXN: &str =
        "SELECT t.id, t.end_user_id, t.game_account_id, t.transaction_type, \
                    t.amount::float8 AS amount, t.game_coin_id, c.code AS game_coin, \
                    t.status, t.created_at, t.updated_at \
             FROM fx_money.deposit_withdrawal_txns t \
             LEFT JOIN fx_game.game_coins c ON c.id = t.game_coin_id \
             WHERE t.id = $1";

    /// Lookup by request_id or partner_txn_id stored in status-log payloads.
    /// Only deposit / withdrawal rows (never shop_topup / corp_shop_purchase).
    pub const FIND_BY_PARTNER_REF: &str =
        "SELECT t.id, t.end_user_id, t.game_account_id, t.transaction_type, \
                    t.amount::float8 AS amount, t.game_coin_id, c.code AS game_coin, \
                    t.status, t.created_at, t.updated_at \
             FROM fx_money.deposit_withdrawal_txns t \
             LEFT JOIN fx_game.game_coins c ON c.id = t.game_coin_id \
             WHERE t.transaction_type IN ('deposit', 'withdrawal') \
               AND EXISTS ( \
                SELECT 1 FROM fx_money.deposit_withdrawal_status_logs l \
                WHERE l.deposit_withdrawal_txn_id = t.id \
                  AND ( \
                    ($1 <> '' AND l.payload->>'request_id' = $1) \
                    OR ($2 <> '' AND l.payload->>'partner_txn_id' = $2) \
                  ) \
             ) \
             ORDER BY t.id DESC \
             LIMIT 1";

    pub const INSERT_STATUS_LOG_PARTNER_REF: &str =
        "INSERT INTO fx_money.deposit_withdrawal_status_logs ( \
                deposit_withdrawal_txn_id, old_status, new_status, event_type, note, payload, created_at \
             ) VALUES ($1, 'pending', 'pending', 'status_changed', $2, $3::jsonb, $4)";

    pub const MARK_TERMINAL: &str =
        "UPDATE fx_money.deposit_withdrawal_txns \
             SET status = $2, updated_at = $3 \
             WHERE id = $1 AND status IN ('pending', 'processing')";

    pub const INSERT_STATUS_LOG_TERMINAL: &str =
        "INSERT INTO fx_money.deposit_withdrawal_status_logs ( \
                deposit_withdrawal_txn_id, old_status, new_status, event_type, note, payload, created_at \
             ) VALUES ($1, 'pending', $2, $3, $4, $5::jsonb, $6)";

    pub const INSERT_OUTBOUND_NOTICE: &str = "INSERT INTO fx_events.outbound_notices ( \
            end_user_id, source_type, source_id, event_type, title, body, payload, \
            delivery_status, retry_count, scheduled_at, sent_at, created_at, updated_at \
         ) VALUES ($1,'deposit_withdrawal_txn',$2,$3,$4,$5,$6::jsonb,'pending',0,0,0,$7,0)";
}

/// Shop catalog (packages / corp products), orders, payment settle, and wallet credit.
pub mod shop {
    pub const BASE_FIAT: &str =
        "SELECT setting_value FROM fx_config.system_settings \
                 WHERE setting_key = 'base_fiat_currency'";

    pub const LIST_PACKAGES: &str =
        "SELECT p.id, p.code, p.name, p.game_coin_id, c.code AS credit_game_coin, \
                        p.coin_amount::float8 AS coin_amount, p.fiat_price::float8 AS fiat_price, \
                        p.seller_type, p.status \
                 FROM fx_shop.shop_packages p \
                 JOIN fx_game.game_coins c ON c.id = p.game_coin_id \
                 WHERE p.status = 'active' \
                 ORDER BY p.id";

    pub const LIST_CORP_PRODUCTS_ACTIVE: &str =
        "SELECT pr.id, pr.corporate_user_id, pr.game_id, pr.code, pr.name, pr.product_type, \
                        pr.credit_game_coin_id, gc.code AS credit_game_coin, \
                        pr.credit_amount::float8 AS credit_amount, pr.item_code, \
                        pr.fiat_price::float8 AS fiat_price, pr.seller_type, pr.status \
                 FROM fx_shop.corp_shop_products pr \
                 LEFT JOIN fx_game.game_coins gc ON gc.id = pr.credit_game_coin_id \
                 WHERE pr.status = 'active' \
                 ORDER BY pr.id";

    pub const LIST_CORP_PRODUCTS_OWNED: &str =
        "SELECT pr.id, pr.corporate_user_id, pr.game_id, pr.code, pr.name, pr.product_type, \
                        pr.credit_game_coin_id, gc.code AS credit_game_coin, \
                        pr.credit_amount::float8 AS credit_amount, pr.item_code, \
                        pr.fiat_price::float8 AS fiat_price, pr.seller_type, pr.status \
                 FROM fx_shop.corp_shop_products pr \
                 LEFT JOIN fx_game.game_coins gc ON gc.id = pr.credit_game_coin_id \
                 WHERE pr.corporate_user_id = $1 \
                 ORDER BY pr.id";

    pub const INSERT_CORP_PRODUCT: &str = "INSERT INTO fx_shop.corp_shop_products ( \
                    corporate_user_id, game_id, code, name, product_type, credit_game_coin_id, \
                    credit_amount, item_code, fiat_price, seller_type, status, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'corp', $10, $11, 0) \
                 RETURNING id";

    pub const UPDATE_CORP_PRODUCT: &str =
        "UPDATE fx_shop.corp_shop_products SET game_id = $1, code = $2, name = $3, product_type = $4, \
                    credit_game_coin_id = $5, credit_amount = $6, item_code = $7, fiat_price = $8, updated_at = $9 \
                 WHERE id = $10 AND corporate_user_id = $11";

    pub const SET_CORP_PRODUCT_STATUS: &str =
        "UPDATE fx_shop.corp_shop_products SET status = $1, updated_at = $2 \
                 WHERE id = $3 AND corporate_user_id = $4";

    pub const ORDER_SELECT: &str =
        "SELECT o.id, o.end_user_id, o.game_account_id, o.seller_type, \
    o.package_id, o.corp_product_id, p.code AS package_code, c.code AS product_code, \
    gc.code AS credit_game_coin, o.credit_amount::float8 AS credit_amount, o.item_code, \
    o.fiat_currency, o.fiat_price::float8 AS fiat_price, o.partner_order_no, \
    o.status, o.expires_at, o.paid_at, o.created_at \
 FROM fx_shop.shop_orders o \
 LEFT JOIN fx_shop.shop_packages p ON p.id = o.package_id \
 LEFT JOIN fx_shop.corp_shop_products c ON c.id = o.corp_product_id \
 LEFT JOIN fx_game.game_coins gc ON gc.id = o.game_coin_id \
 WHERE o.end_user_id = $1";

    pub const ORDER_BY_ID: &str =
        "SELECT o.id, o.end_user_id, o.game_account_id, o.seller_type, \
                        o.package_id, o.corp_product_id, p.code AS package_code, c.code AS product_code, \
                        gc.code AS credit_game_coin, o.credit_amount::float8 AS credit_amount, o.item_code, \
                        o.fiat_currency, o.fiat_price::float8 AS fiat_price, o.partner_order_no, \
                        o.status, o.expires_at, o.paid_at, o.created_at, o.game_coin_id, o.corporate_user_id \
                 FROM fx_shop.shop_orders o \
                 LEFT JOIN fx_shop.shop_packages p ON p.id = o.package_id \
                 LEFT JOIN fx_shop.corp_shop_products c ON c.id = o.corp_product_id \
                 LEFT JOIN fx_game.game_coins gc ON gc.id = o.game_coin_id \
                 WHERE o.id = $1";

    pub const SETTLE_ORDER: &str =
        "UPDATE fx_shop.shop_orders SET status = $1, paid_at = $2, partner_order_no = COALESCE($3, partner_order_no), updated_at = $4 \
                 WHERE id = $5";

    pub const INSERT_PAYMENT_EVENT: &str = "INSERT INTO fx_shop.shop_payment_events \
                    (shop_order_id, event_id, event_type, partner_order_no, status, payload, received_at, created_at) \
                 VALUES ($1, $2, 'shop_payment', $3, $4, $5::jsonb, $6, $6) \
                 ON CONFLICT (event_id) DO NOTHING";

    pub const UPSERT_WALLET_CREDIT: &str =
        "INSERT INTO fx_user.user_wallets (game_account_id, game_coin_id, available, locked, updated_at) \
                             VALUES ($1, $2, $3, 0, $4) \
                             ON CONFLICT (game_account_id, game_coin_id) DO UPDATE SET \
                                available = fx_user.user_wallets.available + EXCLUDED.available, \
                                updated_at = EXCLUDED.updated_at";

    pub const INSERT_ORDER: &str = "INSERT INTO fx_shop.shop_orders ( \
                    game_account_id, end_user_id, seller_type, package_id, corp_product_id, \
                    corporate_user_id, partner_order_no, fiat_currency, fiat_price, \
                    credit_amount, game_coin_id, item_code, status, expires_at, paid_at, \
                    created_at, updated_at \
                 ) VALUES ( \
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'pending', $13, 0, $14, 0 \
                 ) RETURNING id";

    pub const SET_PAYMENT: &str = "UPDATE fx_shop.shop_orders \
                 SET partner_order_no = $1, updated_at = $2 WHERE id = $3";

    pub const CANCEL_ORDER: &str =
        "UPDATE fx_shop.shop_orders SET status = 'cancelled', updated_at = $1 WHERE id = $2 \
                 AND end_user_id = $3 AND status = 'pending'";

    pub const EXPIRE_PENDING: &str =
        "UPDATE fx_shop.shop_orders SET status = 'expired', updated_at = $1 \
                 WHERE status = 'pending' AND expires_at <= $1";

    pub const TXN_BY_SHOP_ORDER: &str = "SELECT id FROM fx_money.deposit_withdrawal_txns \
         WHERE shop_order_id = $1 AND transaction_type = $2 \
         ORDER BY id LIMIT 1";

    pub const INSERT_COMPLETED_TXN: &str = "INSERT INTO fx_money.deposit_withdrawal_txns ( \
            end_user_id, game_account_id, corporate_user_id, shop_order_id, \
            transaction_type, amount, game_coin_id, status, created_at, updated_at \
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'completed', $8, 0) \
         RETURNING id";

    pub const COMPLETED_LOG_EXISTS: &str = "SELECT EXISTS( \
            SELECT 1 FROM fx_money.deposit_withdrawal_status_logs \
            WHERE deposit_withdrawal_txn_id = $1 AND event_type = 'completed' \
         )";

    pub const INSERT_STATUS_LOG_COMPLETED: &str =
        "INSERT INTO fx_money.deposit_withdrawal_status_logs ( \
            deposit_withdrawal_txn_id, old_status, new_status, event_type, note, payload, created_at \
         ) VALUES ($1, NULL, 'completed', 'completed', $2, $3::jsonb, $4)";
}

/// Company Basic Tokens: corp CRUD, buyable list, orders, settle + fee ledger + callbacks.
pub mod cbt {
    pub const HAS_APPROVED_FOR_COIN: &str =
        "SELECT EXISTS(SELECT 1 FROM fx_corp_token.company_basic_tokens \
                 WHERE corporate_user_id = $1 AND game_coin_id = $2 \
                   AND status = 'approved' AND buyable = TRUE)";

    pub const LIST_FOR_CORP: &str =
        "SELECT id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                        status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                        approved_by_admin_id, approved_at, created_at, updated_at \
                 FROM fx_corp_token.company_basic_tokens \
                 WHERE corporate_user_id = $1 ORDER BY id";

    pub const BY_ID: &str =
        "SELECT id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                        status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                        approved_by_admin_id, approved_at, created_at, updated_at \
                 FROM fx_corp_token.company_basic_tokens WHERE id = $1";

    pub const INSERT_TOKEN: &str = "INSERT INTO fx_corp_token.company_basic_tokens ( \
                    corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                    status, buyable, buy_fee_rate, approved_at, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, $6, FALSE, $7, 0, $8, 0) \
                 RETURNING id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                           status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                           approved_by_admin_id, approved_at, created_at, updated_at";

    pub const UPDATE_TOKEN: &str = "UPDATE fx_corp_token.company_basic_tokens SET \
                    game_id = $1, game_coin_id = $2, token_code = $3, token_name = $4, \
                    status = $5, buyable = FALSE, approved_by_admin_id = NULL, approved_at = 0, \
                    updated_at = $6 \
                 WHERE id = $7 AND corporate_user_id = $8 \
                 RETURNING id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                           status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                           approved_by_admin_id, approved_at, created_at, updated_at";

    pub const LIST_BUYABLE: &str =
        "SELECT id, corporate_user_id, game_id, game_coin_id, token_code, token_name, \
                        status, buyable, buy_fee_rate::float8 AS buy_fee_rate, \
                        approved_by_admin_id, approved_at, created_at, updated_at \
                 FROM fx_corp_token.company_basic_tokens \
                 WHERE status = 'approved' AND buyable = TRUE ORDER BY id";

    pub const INSERT_ORDER: &str = "INSERT INTO fx_corp_token.corp_token_orders ( \
                    company_basic_token_id, end_user_id, game_account_id, partner_order_no, \
                    pay_amount, pay_game_coin_id, coin_amount, fee_coin_amount, credited_coin_amount, \
                    game_coin_id, status, expires_at, paid_at, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, $6, $5, $7, $8, $6, 'pending', $9, 0, $10, 0) \
                 RETURNING id, company_basic_token_id, end_user_id, game_account_id, partner_order_no, \
                           pay_amount::float8 AS pay_amount, pay_game_coin_id, \
                           coin_amount::float8 AS coin_amount, fee_coin_amount::float8 AS fee_coin_amount, \
                           credited_coin_amount::float8 AS credited_coin_amount, game_coin_id, \
                           status, expires_at, paid_at, created_at, updated_at";

    pub const LIST_ORDERS_FOR_USER: &str =
        "SELECT o.id, o.company_basic_token_id, o.end_user_id, o.game_account_id, o.partner_order_no, \
                        o.pay_amount::float8 AS pay_amount, o.pay_game_coin_id, \
                        o.coin_amount::float8 AS coin_amount, o.fee_coin_amount::float8 AS fee_coin_amount, \
                        o.credited_coin_amount::float8 AS credited_coin_amount, o.game_coin_id, \
                        o.status, o.expires_at, o.paid_at, o.created_at, o.updated_at, \
                        t.token_code, t.corporate_user_id \
                 FROM fx_corp_token.corp_token_orders o \
                 JOIN fx_corp_token.company_basic_tokens t ON t.id = o.company_basic_token_id \
                 WHERE o.end_user_id = $1 ORDER BY o.id DESC";

    pub const ORDER_BY_ID: &str =
        "SELECT o.id, o.company_basic_token_id, o.end_user_id, o.game_account_id, o.partner_order_no, \
                        o.pay_amount::float8 AS pay_amount, o.pay_game_coin_id, \
                        o.coin_amount::float8 AS coin_amount, o.fee_coin_amount::float8 AS fee_coin_amount, \
                        o.credited_coin_amount::float8 AS credited_coin_amount, o.game_coin_id, \
                        o.status, o.expires_at, o.paid_at, o.created_at, o.updated_at, \
                        t.token_code, t.corporate_user_id \
                 FROM fx_corp_token.corp_token_orders o \
                 JOIN fx_corp_token.company_basic_tokens t ON t.id = o.company_basic_token_id \
                 WHERE o.id = $1";

    pub const CANCEL_ORDER: &str =
        "UPDATE fx_corp_token.corp_token_orders SET status = 'cancelled', updated_at = $1 WHERE id = $2";

    pub const LIST_ORDERS_FOR_TOKEN: &str =
        "SELECT o.id, o.company_basic_token_id, o.end_user_id, o.game_account_id, o.partner_order_no, \
                        o.pay_amount::float8 AS pay_amount, o.pay_game_coin_id, \
                        o.coin_amount::float8 AS coin_amount, o.fee_coin_amount::float8 AS fee_coin_amount, \
                        o.credited_coin_amount::float8 AS credited_coin_amount, o.game_coin_id, \
                        o.status, o.expires_at, o.paid_at, o.created_at, o.updated_at, \
                        t.token_code, t.corporate_user_id \
                 FROM fx_corp_token.corp_token_orders o \
                 JOIN fx_corp_token.company_basic_tokens t ON t.id = o.company_basic_token_id \
                 WHERE o.company_basic_token_id = $1 ORDER BY o.id DESC";

    pub const LIST_FEES_FOR_TOKEN: &str =
        "SELECT id, corporate_user_id, company_basic_token_id, corp_token_order_id, game_coin_id, \
                        fee_rate::float8 AS fee_rate, gross_coin_amount::float8 AS gross_coin_amount, \
                        fee_coin_amount::float8 AS fee_coin_amount, net_coin_amount::float8 AS net_coin_amount, \
                        created_at \
                 FROM fx_corp_token.coin_fee_ledger \
                 WHERE company_basic_token_id = $1 ORDER BY id DESC";

    pub const SETTLE_ORDER: &str =
        "UPDATE fx_corp_token.corp_token_orders SET status = $1, paid_at = $2, \
                    partner_order_no = COALESCE($3, partner_order_no), updated_at = $4 \
                 WHERE id = $5";

    pub const INSERT_PAYMENT_EVENT: &str = "INSERT INTO fx_corp_token.corp_token_payment_events \
                    (corp_token_order_id, event_id, event_type, partner_order_no, status, payload, received_at, created_at) \
                 VALUES ($1, $2, 'corp_token_payment', $3, $4, $5::jsonb, $6, $6) \
                 ON CONFLICT (event_id) DO NOTHING";

    pub const UPSERT_WALLET_CREDIT: &str =
        "INSERT INTO fx_user.user_wallets (game_account_id, game_coin_id, available, locked, updated_at) \
                     VALUES ($1, $2, $3, 0, $4) \
                     ON CONFLICT (game_account_id, game_coin_id) DO UPDATE SET \
                        available = fx_user.user_wallets.available + EXCLUDED.available, \
                        updated_at = EXCLUDED.updated_at";

    pub const INSERT_FEE_LEDGER: &str = "INSERT INTO fx_corp_token.coin_fee_ledger ( \
                        corporate_user_id, company_basic_token_id, corp_token_order_id, game_coin_id, \
                        fee_rate, gross_coin_amount, fee_coin_amount, net_coin_amount, created_at \
                     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)";

    pub const WEBHOOK_CALLBACK_URL: &str =
        "SELECT callback_url FROM fx_events.webhook_endpoints \
                 WHERE corporate_user_id = $1 AND kind = 'corp_token' AND status = 'active' \
                 LIMIT 1";

    pub const PARTNER_CALLBACK_URL: &str =
        "SELECT endpoint FROM fx_corp.partner_endpoints \
                 WHERE corporate_user_id = $1 AND type = 'callback' AND status = 'active' \
                 LIMIT 1";

    pub const EXPIRE_PENDING: &str =
        "UPDATE fx_corp_token.corp_token_orders SET status = 'expired', updated_at = $1 \
                 WHERE status = 'pending' AND expires_at <= $1";
}

/// Corp market-pair create: lock game balance, insert pair/pool/lock rows (pending).
pub mod market {
    pub const LIST_FOR_CORP: &str =
        "SELECT p.id, p.corporate_user_id, p.game_id, p.base_game_coin_id, p.quote_game_coin_id, \
                        p.market_name, p.funding_source, p.status, p.created_at, p.updated_at, \
                        po.id AS pool_id, po.pool_depth::float8 AS pool_depth, \
                        po.initial_price::float8 AS initial_price, po.base_amount::float8 AS base_amount, \
                        po.quote_amount::float8 AS quote_amount, po.status AS pool_status, \
                        l.game_coin_id AS lock_game_coin_id, l.locked_amount::float8 AS lock_amount \
                 FROM fx_market.market_pairs p \
                 LEFT JOIN fx_market.market_pools po ON po.market_pair_id = p.id \
                 LEFT JOIN fx_market.market_balance_locks l ON l.market_pair_id = p.id AND l.status = 'locked' \
                 WHERE p.corporate_user_id = $1 \
                 ORDER BY p.id";

    pub const OWNED_GAME: &str =
        "SELECT id FROM fx_game.games WHERE id = $1 AND corporate_user_id = $2";

    pub const ACTIVE_COINS_COUNT: &str =
        "SELECT COUNT(*) FROM fx_game.game_coins WHERE id IN ($1, $2) AND status = 'active'";

    pub const EXISTING_PAIRS_COUNT: &str =
        "SELECT COUNT(*) FROM fx_market.market_pairs WHERE game_id = $1 \
                 AND status NOT IN ('rejected', 'inactive')";

    pub const IS_PLATFORM_TOKEN: &str =
        "SELECT COALESCE(is_platform_token, FALSE) FROM fx_game.game_coins WHERE id = $1";

    pub const LOCK_BALANCE: &str =
        "UPDATE fx_game.game_balances SET available_balance = available_balance - $1, \
                    locked_balance = locked_balance + $1, updated_at = $2 \
                 WHERE game_id = $3 AND game_coin_id = $4 AND available_balance >= $1";

    pub const INSERT_PAIR: &str = "INSERT INTO fx_market.market_pairs ( \
                    corporate_user_id, game_id, base_game_coin_id, quote_game_coin_id, market_name, \
                    funding_source, status, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, $6, 'pending_locked', $7, 0) RETURNING id";

    pub const INSERT_BALANCE_LOCK: &str = "INSERT INTO fx_market.market_balance_locks ( \
                    market_pair_id, game_id, game_coin_id, locked_amount, status, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, 'locked', $5, 0)";

    pub const INSERT_POOL: &str = "INSERT INTO fx_market.market_pools ( \
                    market_pair_id, pool_depth, initial_price, base_amount, quote_amount, status, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, 'pending', $6, 0) RETURNING id";

    pub const INSERT_POOL_WALLET: &str =
        "INSERT INTO fx_market.pool_wallets (market_pool_id, game_coin_id, balance, wallet_type, status) \
                     VALUES ($1, $2, 0, 'market_pool', 'active') \
                     ON CONFLICT (market_pool_id, game_coin_id) DO NOTHING";

    pub const INSERT_STATUS_LOG: &str =
        "INSERT INTO fx_market.market_status_logs (market_pair_id, old_status, new_status, updated_at, created_at) \
                 VALUES ($1, 'submitted', 'pending_locked', $2, $2)";

    /// Preserves the incomplete SQL originally inlined in `ensure_pool`.
    pub const INSERT_POOL_WALLET_ENSURE: &str =
        "INSERT INTO fx_market.pool_wallets (market_pool_id, game_coin_id, balance, wallet_type, status) \
                     ON CONFLICT (market_pool_id, game_coin_id) DO NOTHING";
}

/// C7 Marketplace deals: gates, list/create/take/cancel, wallet fee/want, transfer events, notices.
pub mod marketplace {
    /// Listing fee in Platform Token units (debited from poster's wallet on place).
    pub const LISTING_FEE_PLT: f64 = 1.0;

    pub const PLT_COIN_ID: &str =
        "SELECT id FROM fx_game.game_coins \
         WHERE is_platform_token = TRUE AND status = 'active' ORDER BY id LIMIT 1";

    pub const HAS_PLT_PURCHASE: &str = "SELECT EXISTS( \
            SELECT 1 FROM fx_shop.shop_orders o \
            JOIN fx_shop.shop_packages p ON p.id = o.package_id \
            JOIN fx_game.game_coins c ON c.id = p.game_coin_id \
            WHERE o.end_user_id = $1 AND o.status = 'completed' \
              AND c.is_platform_token = TRUE \
        ) OR EXISTS( \
            SELECT 1 FROM fx_user.user_wallets w \
            JOIN fx_game.game_accounts a ON a.id = w.game_account_id \
            JOIN fx_game.game_coins c ON c.id = w.game_coin_id \
            WHERE a.end_user_id = $1 AND c.is_platform_token = TRUE AND w.available > 0 \
        )";

    pub const HAS_ACTIVE_GAME_BIND: &str = "SELECT EXISTS( \
            SELECT 1 FROM fx_game.game_accounts a \
            JOIN fx_game.games g ON g.id = a.game_id \
            WHERE a.end_user_id = $1 AND a.status = 'active' AND g.status = 'active' \
        )";

    pub const OWNED_GAME_ACCOUNT: &str =
        "SELECT a.id, a.end_user_id, a.game_id, a.game_account_id, \
                COALESCE(a.partner_user_id, '') AS partner_user_id, \
                g.corporate_user_id, COALESCE(g.partner_code, '') AS partner_code, \
                COALESCE(g.partner_game_id, '') AS partner_game_id \
         FROM fx_game.game_accounts a \
         JOIN fx_game.games g ON g.id = a.game_id \
         WHERE a.id = $1 AND a.end_user_id = $2 AND a.status = 'active' AND g.status = 'active'";

    pub const TRANSFER_ENDPOINT: &str =
        "SELECT id, endpoint FROM fx_corp.partner_endpoints \
         WHERE corporate_user_id = $1 AND type = 'transfer' AND status = 'active' \
         ORDER BY id LIMIT 1";

    pub const ITEM_LIST_ENDPOINT: &str =
        "SELECT endpoint FROM fx_corp.partner_endpoints \
         WHERE corporate_user_id = $1 AND type = 'item_list' AND status = 'active' \
         ORDER BY id LIMIT 1";

    pub const HAS_TRANSFER_FOR_ANY_GAME: &str = "SELECT EXISTS( \
            SELECT 1 FROM fx_game.game_accounts a \
            JOIN fx_game.games g ON g.id = a.game_id \
            JOIN fx_corp.partner_endpoints e ON e.corporate_user_id = g.corporate_user_id \
            WHERE a.end_user_id = $1 AND a.status = 'active' AND g.status = 'active' \
              AND e.type = 'transfer' AND e.status = 'active' \
        )";

    pub const COIN_BY_ID: &str =
        "SELECT id, code, COALESCE(is_platform_token, FALSE) AS is_platform_token, type \
         FROM fx_game.game_coins WHERE id = $1 AND status = 'active'";

    pub const COMPANY_COIN_FOR_CORP: &str =
        "SELECT EXISTS( \
            SELECT 1 FROM fx_game.games g \
            JOIN fx_game.game_balances b ON b.game_id = g.id \
            WHERE g.corporate_user_id = $1 AND b.game_coin_id = $2 \
        ) OR EXISTS( \
            SELECT 1 FROM fx_corp_token.company_basic_tokens t \
            WHERE t.corporate_user_id = $1 AND t.game_coin_id = $2 \
              AND t.status IN ('approved', 'active') \
        )";

    pub const LIST_OPEN: &str =
        "SELECT d.id, d.poster_end_user_id, d.poster_game_account_id, d.game_id, d.corporate_user_id, \
                d.partner_endpoint_id, d.offer_asset_type, d.offer_game_coin_id, \
                d.offer_item_ref_id, d.offer_amount::float8 AS offer_amount, \
                d.want_asset_type, d.want_game_coin_id, d.want_amount::float8 AS want_amount, \
                d.status, d.matched_request_id, d.cancelled_at, d.completed_at, \
                d.created_at, d.updated_at, \
                COALESCE(oc.code, '') AS offer_game_coin, \
                COALESCE(wc.code, '') AS want_game_coin, \
                COALESCE(g.game_code, '') AS game_code \
         FROM fx_marketplace.marketplace_deals d \
         LEFT JOIN fx_game.game_coins oc ON oc.id = d.offer_game_coin_id \
         LEFT JOIN fx_game.game_coins wc ON wc.id = d.want_game_coin_id \
         LEFT JOIN fx_game.games g ON g.id = d.game_id \
         WHERE d.status = 'open' ORDER BY d.id DESC LIMIT $1";

    pub const GET_BY_ID: &str =
        "SELECT d.id, d.poster_end_user_id, d.poster_game_account_id, d.game_id, d.corporate_user_id, \
                d.partner_endpoint_id, d.offer_asset_type, d.offer_game_coin_id, \
                d.offer_item_ref_id, d.offer_amount::float8 AS offer_amount, \
                d.want_asset_type, d.want_game_coin_id, d.want_amount::float8 AS want_amount, \
                d.status, d.matched_request_id, d.cancelled_at, d.completed_at, \
                d.created_at, d.updated_at, \
                COALESCE(oc.code, '') AS offer_game_coin, \
                COALESCE(wc.code, '') AS want_game_coin, \
                COALESCE(g.game_code, '') AS game_code \
         FROM fx_marketplace.marketplace_deals d \
         LEFT JOIN fx_game.game_coins oc ON oc.id = d.offer_game_coin_id \
         LEFT JOIN fx_game.game_coins wc ON wc.id = d.want_game_coin_id \
         LEFT JOIN fx_game.games g ON g.id = d.game_id \
         WHERE d.id = $1";

    pub const LOCK_OPEN: &str =
        "SELECT id, poster_end_user_id, poster_game_account_id, game_id, corporate_user_id, \
                partner_endpoint_id, offer_asset_type, offer_game_coin_id, offer_item_ref_id, \
                offer_amount::float8 AS offer_amount, want_asset_type, want_game_coin_id, \
                want_amount::float8 AS want_amount, status \
         FROM fx_marketplace.marketplace_deals WHERE id = $1 FOR UPDATE";

    pub const INSERT_DEAL: &str = "INSERT INTO fx_marketplace.marketplace_deals ( \
            poster_end_user_id, poster_game_account_id, game_id, corporate_user_id, \
            partner_endpoint_id, offer_asset_type, offer_game_coin_id, offer_item_ref_id, \
            offer_amount, want_asset_type, want_game_coin_id, want_amount, status, \
            created_at, updated_at \
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,'open',$13,0) \
         RETURNING id";

    pub const CANCEL_OPEN: &str =
        "UPDATE fx_marketplace.marketplace_deals \
         SET status = 'cancelled', cancelled_at = $2, updated_at = $2 \
         WHERE id = $1 AND poster_end_user_id = $3 AND status = 'open'";

    pub const SET_SETTLING: &str =
        "UPDATE fx_marketplace.marketplace_deals \
         SET status = 'settling', matched_request_id = $2, updated_at = $3 \
         WHERE id = $1 AND status = 'open'";

    pub const SET_COMPLETED: &str =
        "UPDATE fx_marketplace.marketplace_deals \
         SET status = 'completed', completed_at = $2, updated_at = $2 WHERE id = $1";

    pub const SET_FAILED: &str =
        "UPDATE fx_marketplace.marketplace_deals \
         SET status = 'failed', updated_at = $2 WHERE id = $1";

    pub const INSERT_REQUEST: &str = "INSERT INTO fx_marketplace.marketplace_deal_requests ( \
            marketplace_deal_id, requester_end_user_id, requester_game_account_id, \
            status, created_at, updated_at \
         ) VALUES ($1, $2, $3, 'accepted', $4, 0) RETURNING id";

    pub const COMPLETE_REQUEST: &str =
        "UPDATE fx_marketplace.marketplace_deal_requests \
         SET status = 'completed', updated_at = $2 WHERE id = $1";

    pub const FAIL_REQUEST: &str =
        "UPDATE fx_marketplace.marketplace_deal_requests \
         SET status = 'failed', updated_at = $2 WHERE id = $1";

    pub const INSERT_TRANSFER_EVENT: &str = "INSERT INTO fx_marketplace.marketplace_transfer_events ( \
            marketplace_deal_id, marketplace_deal_request_id, partner_endpoint_id, request_id, \
            from_game_account_id, to_game_account_id, asset_type, game_coin_id, item_ref_id, \
            amount, status, payload, created_at, updated_at \
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'pending',$11::jsonb,$12,0) \
         RETURNING id";

    pub const SET_TRANSFER_STATUS: &str =
        "UPDATE fx_marketplace.marketplace_transfer_events \
         SET status = $2, payload = COALESCE($3::jsonb, payload), updated_at = $4 \
         WHERE id = $1";

    pub const DEBIT_WALLET: &str = "UPDATE fx_user.user_wallets \
         SET available = available - $3, updated_at = $4 \
         WHERE game_account_id = $1 AND game_coin_id = $2 AND available >= $3";

    pub const CREDIT_WALLET: &str =
        "INSERT INTO fx_user.user_wallets (game_account_id, game_coin_id, available, locked, updated_at) \
         VALUES ($1, $2, $3, 0, $4) \
         ON CONFLICT (game_account_id, game_coin_id) DO UPDATE SET \
            available = fx_user.user_wallets.available + EXCLUDED.available, \
            updated_at = EXCLUDED.updated_at";

    pub const INSERT_OUTBOUND_NOTICE: &str = "INSERT INTO fx_events.outbound_notices ( \
            end_user_id, source_type, source_id, event_type, title, body, payload, \
            delivery_status, retry_count, scheduled_at, sent_at, created_at, updated_at \
         ) VALUES ($1,$2,$3,$4,$5,$6,$7::jsonb,'pending',0,0,0,$8,0)";

    pub const ENDPOINT_URL_BY_ID: &str =
        "SELECT endpoint FROM fx_corp.partner_endpoints WHERE id = $1";

    pub const GAME_ACCOUNT_PARTNER_ID: &str =
        "SELECT game_account_id FROM fx_game.game_accounts WHERE id = $1";
}
