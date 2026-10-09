//! Message Center PostgreSQL SQL constants.
//!
//! Delivers outbound notices (claim → send → mark), user in-app notifications,
//! and records delivery against webhook endpoints/events.
//! Stores use `crate::query::<area>::CONST` only.
//!
//! # How to read this file
//! - `$n` bind params match store bind order.
//! - Worker claim uses `FOR UPDATE SKIP LOCKED` so multiple workers don't double-send.
//! - Status flow: `pending` → `sending` → `sent` | retry (`pending`/`failed`) .
//! - `MARK_RETRY_OR_FAILED` merges error info into `payload` via `|| jsonb`.
//!
//! # Domains
//! | Module             | Logic                                   | Result |
//! |--------------------|-----------------------------------------|--------|
//! | `outbound_notices` | Queue, claim batch, mark sent/retry, list/stats | Notice rows / counts |
//! | `notifications`    | In-app inbox insert / list / mark read  | Notification row / id |
//! | `webhook_endpoints`| Resolve active callback URL by corp+kind | `callback_url` |
//! | `webhook_events`   | Upsert delivery status for MC sends     | Upsert only |

/// Outbound notice queue: insert, claim pending batch, mark outcomes, list & stats.
pub mod outbound_notices {
    pub const INSERT: &str = "INSERT INTO fx_events.outbound_notices ( \
                    end_user_id, source_type, source_id, event_type, title, body, payload, \
                    delivery_status, retry_count, scheduled_at, sent_at, created_at, updated_at \
                 ) VALUES ($1,$2,$3,$4,$5,$6,$7::jsonb,'pending',0,$8,0,$9,0) \
                 RETURNING id, end_user_id, source_type, source_id, event_type, title, body, \
                    payload, delivery_status, retry_count, scheduled_at, sent_at, notification_id, \
                    created_at, updated_at";

    pub const CLAIM_PENDING: &str = "UPDATE fx_events.outbound_notices n SET \
                    delivery_status = 'sending', updated_at = $1 \
                 WHERE n.id IN ( \
                    SELECT id FROM fx_events.outbound_notices \
                    WHERE delivery_status = 'pending' \
                      AND (scheduled_at = 0 OR scheduled_at <= $1) \
                    ORDER BY scheduled_at ASC, created_at ASC \
                    FOR UPDATE SKIP LOCKED \
                    LIMIT $2 \
                 ) \
                 RETURNING id, end_user_id, source_type, source_id, event_type, title, body, \
                    payload, delivery_status, retry_count, scheduled_at, sent_at, notification_id, \
                    created_at, updated_at";

    pub const MARK_SENT: &str = "UPDATE fx_events.outbound_notices SET \
                    delivery_status = 'sent', notification_id = $2, sent_at = $3, updated_at = $3 \
                 WHERE id = $1";

    pub const MARK_RETRY_OR_FAILED: &str = "UPDATE fx_events.outbound_notices SET \
                    delivery_status = $2, retry_count = $3, scheduled_at = $4, updated_at = $5, \
                    payload = payload || $6::jsonb \
                 WHERE id = $1";

    pub const LIST_BY_STATUS: &str = "SELECT id, end_user_id, source_type, source_id, event_type, title, body, \
                        payload, delivery_status, retry_count, scheduled_at, sent_at, notification_id, \
                        created_at, updated_at \
                     FROM fx_events.outbound_notices \
                     WHERE delivery_status = $1 \
                     ORDER BY created_at DESC LIMIT $2";

    pub const LIST_ALL: &str = "SELECT id, end_user_id, source_type, source_id, event_type, title, body, \
                        payload, delivery_status, retry_count, scheduled_at, sent_at, notification_id, \
                        created_at, updated_at \
                     FROM fx_events.outbound_notices \
                     ORDER BY created_at DESC LIMIT $1";

    pub const GET: &str = "SELECT id, end_user_id, source_type, source_id, event_type, title, body, \
                    payload, delivery_status, retry_count, scheduled_at, sent_at, notification_id, \
                    created_at, updated_at \
                 FROM fx_events.outbound_notices WHERE id = $1";

    pub const STATS_BY_STATUS: &str =
        "SELECT delivery_status, COUNT(*)::bigint FROM fx_events.outbound_notices \
                 GROUP BY delivery_status";
}

/// Per-user in-app notifications (create, list newest, mark read).
pub mod notifications {
    pub const INSERT: &str = "INSERT INTO fx_events.notifications ( \
                    end_user_id, type, payload, read_at, created_at, updated_at \
                 ) VALUES ($1, $2, $3::jsonb, 0, $4, 0) RETURNING id";

    pub const LIST_FOR_USER: &str = "SELECT id, end_user_id, type, payload, read_at, created_at \
                 FROM fx_events.notifications \
                 WHERE end_user_id = $1 \
                 ORDER BY created_at DESC LIMIT $2";

    pub const MARK_READ: &str = "UPDATE fx_events.notifications SET read_at = $3, updated_at = $3 \
                 WHERE id = $1 AND end_user_id = $2 \
                 RETURNING id, end_user_id, type, payload, read_at, created_at";
}

/// Look up active corp callback URL for a webhook kind.
pub mod webhook_endpoints {
    pub const RESOLVE_CALLBACK: &str =
        "SELECT callback_url FROM fx_events.webhook_endpoints \
             WHERE corporate_user_id = $1 AND kind = $2 AND status = 'active' LIMIT 1";
}

/// Record Message Center delivery outcome on `webhook_events` (upsert by event_id).
pub mod webhook_events {
    pub const UPSERT_DELIVERY: &str = "INSERT INTO fx_events.webhook_events ( \
                event_id, source, event_type, payload, delivery_status, retry_count, \
                last_error, next_retry_at, received_at, updated_at \
             ) VALUES ($1, 'message_center', $2, $3::jsonb, $4, 0, $5, 0, $6, 0) \
             ON CONFLICT (event_id) DO UPDATE SET \
                delivery_status = EXCLUDED.delivery_status, \
                last_error = EXCLUDED.last_error, \
                updated_at = EXCLUDED.received_at";
}
