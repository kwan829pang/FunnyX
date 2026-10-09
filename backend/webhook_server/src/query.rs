//! Webhook Server PostgreSQL SQL constants.
//!
//! Ingests partner/payment webhooks, persists events, drives retries, and
//! resolves corp callback endpoints. Stores use `crate::query::<area>::CONST`.
//!
//! # How to read this file
//! - `$n` bind params match store bind order.
//! - Event rows return `payload` as text (`payload::text`) for JSON re-parse.
//! - Delivery statuses: `received` → processing → `failed` (retry) / success.
//! - `DUE_RETRIES` picks failed events whose `next_retry_at <= now` (limit 50).
//!
//! # Domains
//! | Module      | Logic                                      | Result |
//! |-------------|--------------------------------------------|--------|
//! | `events`    | Idempotent insert, status, retry poll      | Webhook event row |
//! | `shop`      | Link payment event to shop order           | Insert (ignore dup event_id) |
//! | `notices`   | Queue end-user / corp partner notices      | Insert only |
//! | `endpoints` | Resolve & CRUD corp webhook callbacks      | URL / endpoint row |
//! | `corp`      | Auth corp via master_code + api_key        | `corporate_users.id` |

/// Inbound webhook_events: lookup by event_id, insert, status updates, due retries.
pub mod events {
    pub const BY_EVENT_ID: &str = "SELECT id, event_id, source, event_type, payload::text AS payload, delivery_status, retry_count, \
                        last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at \
                 FROM fx_events.webhook_events WHERE event_id = $1";

    pub const INSERT: &str = "INSERT INTO fx_events.webhook_events ( \
                    event_id, source, event_type, payload, delivery_status, retry_count, \
                    last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at \
                 ) VALUES ($1, $2, $3, $4::jsonb, 'received', 0, NULL, 0, $5, $6, $7, $7) \
                 RETURNING id, event_id, source, event_type, payload::text AS payload, delivery_status, retry_count, \
                           last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at";

    pub const INSERT_WITHOUT_SHOP_ORDER: &str = "INSERT INTO fx_events.webhook_events ( \
                                event_id, source, event_type, payload, delivery_status, retry_count, \
                                last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at \
                             ) VALUES ($1, $2, $3, $4::jsonb, 'received', 0, NULL, 0, NULL, $5, $6, $6) \
                             RETURNING id, event_id, source, event_type, payload::text AS payload, delivery_status, retry_count, \
                                       last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at";

    pub const SET_STATUS: &str = "UPDATE fx_events.webhook_events SET delivery_status = $1, last_error = $2, retry_count = $3, \
                    next_retry_at = COALESCE($4, next_retry_at), updated_at = $5 \
                 WHERE event_id = $6";

    pub const DUE_RETRIES: &str = "SELECT id, event_id, source, event_type, payload::text AS payload, delivery_status, retry_count, \
                        last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at \
                 FROM fx_events.webhook_events \
                 WHERE delivery_status = 'failed' AND next_retry_at <= $1 \
                 ORDER BY next_retry_at ASC LIMIT 50";
}

/// Persist shop payment events keyed by `event_id` (ON CONFLICT DO NOTHING).
pub mod shop {
    pub const INSERT_PAYMENT_EVENT: &str = "INSERT INTO fx_shop.shop_payment_events \
                            (shop_order_id, event_id, event_type, partner_order_no, status, payload, received_at, created_at) \
                         VALUES ($1, $2, 'shop_payment', $3, $4, $5::jsonb, $6, $6) \
                         ON CONFLICT (event_id) DO NOTHING";
}

/// Fan-out: pending outbound_notices for users, corp_partner_notices for corps.
pub mod notices {
    pub const INSERT_OUTBOUND: &str = "INSERT INTO fx_events.outbound_notices ( \
                end_user_id, source_type, source_id, event_type, title, body, payload, \
                delivery_status, retry_count, scheduled_at, sent_at, created_at, updated_at \
             ) VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, 'pending', 0, 0, 0, $8, 0)";

    pub const INSERT_CORP_PARTNER: &str = "INSERT INTO fx_corp.corp_partner_notices ( \
                corporate_user_id, notice_type, title, body, deadline_at, \
                created_by_admin_id, read_at, created_at \
             ) VALUES ($1, 'other', $2, $3, 0, NULL, 0, $4)";
}

/// Resolve active corp_token / partner callback URLs; list/upsert/update endpoints.
pub mod endpoints {
    pub const CORP_TOKEN_CALLBACK_URL: &str = "SELECT callback_url FROM fx_events.webhook_endpoints \
                 WHERE corporate_user_id = $1 AND kind = 'corp_token' AND status = 'active' \
                 LIMIT 1";

    pub const PARTNER_CALLBACK_ENDPOINT: &str = "SELECT endpoint FROM fx_corp.partner_endpoints \
                 WHERE corporate_user_id = $1 AND type = 'callback' AND status = 'active' \
                 LIMIT 1";

    pub const LIST: &str = "SELECT id, corporate_user_id, kind, callback_url, auth_type, secret_hint, status, created_at, updated_at \
                 FROM fx_events.webhook_endpoints WHERE corporate_user_id = $1 ORDER BY id";

    pub const UPSERT: &str = "INSERT INTO fx_events.webhook_endpoints ( \
                    corporate_user_id, kind, callback_url, auth_type, secret_hint, status, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, 0) \
                 ON CONFLICT (corporate_user_id, kind) DO UPDATE SET \
                    callback_url = EXCLUDED.callback_url, \
                    auth_type = EXCLUDED.auth_type, \
                    secret_hint = EXCLUDED.secret_hint, \
                    status = EXCLUDED.status, \
                    updated_at = $7 \
                 RETURNING id, corporate_user_id, kind, callback_url, auth_type, secret_hint, status, created_at, updated_at";

    pub const UPDATE: &str = "UPDATE fx_events.webhook_endpoints SET \
                    callback_url = COALESCE($1, callback_url), \
                    auth_type = COALESCE($2, auth_type), \
                    secret_hint = COALESCE($3, secret_hint), \
                    status = COALESCE($4, status), \
                    updated_at = $5 \
                 WHERE id = $6 AND corporate_user_id = $7 \
                 RETURNING id, corporate_user_id, kind, callback_url, auth_type, secret_hint, status, created_at, updated_at";
}

/// Validate corp master_code + master_id + api_key → corporate_user id.
pub mod corp {
    pub const SELECT_BY_MASTER: &str = "SELECT u.id FROM fx_corp.corporate_users u \
             JOIN fx_corp.corp_api_keys k ON k.corporate_user_id = u.id \
             WHERE u.master_code = $1 AND u.master_id = $2 AND k.api_key = $3 \
               AND u.status = 'active' AND k.status = 'active' AND u.api_enabled = TRUE";
}
