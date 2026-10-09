//! Persist inbound webhook events, shop audit rows, Corp endpoints, notices.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub id: i64,
    pub event_id: String,
    pub source: String,
    pub event_type: String,
    pub payload: Value,
    pub delivery_status: String,
    pub retry_count: i32,
    pub last_error: Option<String>,
    pub next_retry_at: i64,
    pub shop_order_id: Option<i64>,
    pub partner_order_no: Option<String>,
    pub received_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: i64,
    pub corporate_user_id: i64,
    pub kind: String,
    pub callback_url: String,
    pub auth_type: String,
    pub secret_hint: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

struct Memory {
    events: RwLock<HashMap<String, WebhookEvent>>,
    endpoints: RwLock<HashMap<i64, WebhookEndpoint>>,
    next_event: AtomicI64,
    next_endpoint: AtomicI64,
}

#[derive(Clone)]
pub struct EventStore {
    pool: Option<PgPool>,
    memory: Option<Arc<Memory>>,
}

impl EventStore {
    pub fn new(pool: Option<PgPool>) -> Self {
        if pool.is_some() {
            Self {
                pool,
                memory: None,
            }
        } else {
            Self {
                pool: None,
                memory: Some(Arc::new(Memory {
                    events: RwLock::new(HashMap::new()),
                    endpoints: RwLock::new(HashMap::new()),
                    next_event: AtomicI64::new(1),
                    next_endpoint: AtomicI64::new(1),
                })),
            }
        }
    }

    pub async fn insert_received(
        &self,
        event_id: &str,
        source: &str,
        event_type: &str,
        payload: &Value,
        shop_order_id: Option<i64>,
        partner_order_no: Option<&str>,
    ) -> Result<InsertOutcome, String> {
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let existing = sqlx::query(
                "SELECT id, event_id, source, event_type, payload::text AS payload, delivery_status, retry_count, \
                        last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at \
                 FROM fx_events.webhook_events WHERE event_id = $1",
            )
            .bind(event_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
            if let Some(row) = existing {
                return Ok(InsertOutcome::Duplicate(map_event(row)));
            }
            let insert = sqlx::query(
                "INSERT INTO fx_events.webhook_events ( \
                    event_id, source, event_type, payload, delivery_status, retry_count, \
                    last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at \
                 ) VALUES ($1, $2, $3, $4::jsonb, 'received', 0, NULL, 0, $5, $6, $7, $7) \
                 RETURNING id, event_id, source, event_type, payload::text AS payload, delivery_status, retry_count, \
                           last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at",
            )
            .bind(event_id)
            .bind(source)
            .bind(event_type)
            .bind(payload.to_string())
            .bind(shop_order_id)
            .bind(partner_order_no)
            .bind(now);
            let row = match insert.fetch_one(pool).await {
                Ok(r) => r,
                Err(e) => {
                    let msg = e.to_string();
                    if shop_order_id.is_some()
                        && (msg.contains("foreign key") || msg.contains("23503"))
                    {
                        sqlx::query(
                            "INSERT INTO fx_events.webhook_events ( \
                                event_id, source, event_type, payload, delivery_status, retry_count, \
                                last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at \
                             ) VALUES ($1, $2, $3, $4::jsonb, 'received', 0, NULL, 0, NULL, $5, $6, $6) \
                             RETURNING id, event_id, source, event_type, payload::text AS payload, delivery_status, retry_count, \
                                       last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at",
                        )
                        .bind(event_id)
                        .bind(source)
                        .bind(event_type)
                        .bind(payload.to_string())
                        .bind(partner_order_no)
                        .bind(now)
                        .fetch_one(pool)
                        .await
                        .map_err(|e2| e2.to_string())?
                    } else {
                        return Err(msg);
                    }
                }
            };
            if event_type == "shop_payment" {
                if let Some(oid) = shop_order_id {
                    let _ = sqlx::query(
                        "INSERT INTO fx_shop.shop_payment_events \
                            (shop_order_id, event_id, event_type, partner_order_no, status, payload, received_at, created_at) \
                         VALUES ($1, $2, 'shop_payment', $3, $4, $5::jsonb, $6, $6) \
                         ON CONFLICT (event_id) DO NOTHING",
                    )
                    .bind(oid)
                    .bind(event_id)
                    .bind(partner_order_no.unwrap_or(""))
                    .bind(payload.get("status").and_then(|v| v.as_str()).unwrap_or("received"))
                    .bind(payload.to_string())
                    .bind(now)
                    .execute(pool)
                    .await;
                }
            }
            return Ok(InsertOutcome::Inserted(map_event(row)));
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        {
            let g = mem.events.read().await;
            if let Some(ev) = g.get(event_id) {
                return Ok(InsertOutcome::Duplicate(ev.clone()));
            }
        }
        let id = mem.next_event.fetch_add(1, Ordering::Relaxed);
        let rec = WebhookEvent {
            id,
            event_id: event_id.into(),
            source: source.into(),
            event_type: event_type.into(),
            payload: payload.clone(),
            delivery_status: "received".into(),
            retry_count: 0,
            last_error: None,
            next_retry_at: 0,
            shop_order_id,
            partner_order_no: partner_order_no.map(|s| s.to_string()),
            received_at: now,
            updated_at: now,
        };
        mem.events.write().await.insert(event_id.into(), rec.clone());
        Ok(InsertOutcome::Inserted(rec))
    }

    pub async fn get_by_event_id(&self, event_id: &str) -> Option<WebhookEvent> {
        if let Some(pool) = &self.pool {
            return sqlx::query(
                "SELECT id, event_id, source, event_type, payload::text AS payload, delivery_status, retry_count, \
                        last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at \
                 FROM fx_events.webhook_events WHERE event_id = $1",
            )
            .bind(event_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .map(map_event);
        }
        let mem = self.memory.as_ref()?;
        mem.events.read().await.get(event_id).cloned()
    }

    pub async fn mark_processing(&self, event_id: &str) -> Result<(), String> {
        self.set_status(event_id, "processing", None, 0, None).await
    }

    pub async fn mark_settled(&self, event_id: &str) -> Result<(), String> {
        self.set_status(event_id, "settled", None, 0, None).await
    }

    pub async fn mark_failed(&self, event_id: &str, err: &str, retry_count: i32, next: i64) -> Result<(), String> {
        let status = if retry_count >= MAX_RETRIES {
            "dead"
        } else {
            "failed"
        };
        self.set_status(event_id, status, Some(err), retry_count, Some(next))
            .await
    }

    async fn set_status(
        &self,
        event_id: &str,
        status: &str,
        last_error: Option<&str>,
        retry_count: i32,
        next_retry_at: Option<i64>,
    ) -> Result<(), String> {
        let now = now_ms();
        if let Some(pool) = &self.pool {
            sqlx::query(
                "UPDATE fx_events.webhook_events SET delivery_status = $1, last_error = $2, retry_count = $3, \
                    next_retry_at = COALESCE($4, next_retry_at), updated_at = $5 \
                 WHERE event_id = $6",
            )
            .bind(status)
            .bind(last_error)
            .bind(retry_count)
            .bind(next_retry_at)
            .bind(now)
            .bind(event_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        if let Some(ev) = mem.events.write().await.get_mut(event_id) {
            ev.delivery_status = status.into();
            ev.last_error = last_error.map(|s| s.chars().take(512).collect());
            ev.retry_count = retry_count;
            if let Some(n) = next_retry_at {
                ev.next_retry_at = n;
            }
            ev.updated_at = now;
        }
        Ok(())
    }

    pub async fn due_retries(&self) -> Vec<WebhookEvent> {
        let now = now_ms();
        if let Some(pool) = &self.pool {
            return sqlx::query(
                "SELECT id, event_id, source, event_type, payload::text AS payload, delivery_status, retry_count, \
                        last_error, next_retry_at, shop_order_id, partner_order_no, received_at, updated_at \
                 FROM fx_events.webhook_events \
                 WHERE delivery_status = 'failed' AND next_retry_at <= $1 \
                 ORDER BY next_retry_at ASC LIMIT 50",
            )
            .bind(now)
            .fetch_all(pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(map_event)
            .collect();
        }
        let Some(mem) = &self.memory else {
            return Vec::new();
        };
        mem.events
            .read()
            .await
            .values()
            .filter(|e| e.delivery_status == "failed" && e.next_retry_at <= now)
            .cloned()
            .collect()
    }

    pub async fn insert_outbound_notice(
        &self,
        end_user_id: i64,
        shop_order_id: i64,
        event_type: &str,
        title: &str,
        body: &str,
        payload: &Value,
    ) -> Result<(), String> {
        self.insert_outbound_notice_typed(
            end_user_id,
            "shop_order",
            shop_order_id,
            event_type,
            title,
            body,
            payload,
        )
        .await
    }

    pub async fn insert_outbound_notice_typed(
        &self,
        end_user_id: i64,
        source_type: &str,
        source_id: i64,
        event_type: &str,
        title: &str,
        body: &str,
        payload: &Value,
    ) -> Result<(), String> {
        let now = now_ms();
        let Some(pool) = &self.pool else {
            return Ok(());
        };
        sqlx::query(
            "INSERT INTO fx_events.outbound_notices ( \
                end_user_id, source_type, source_id, event_type, title, body, payload, \
                delivery_status, retry_count, scheduled_at, sent_at, created_at, updated_at \
             ) VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, 'pending', 0, 0, 0, $8, 0)",
        )
        .bind(end_user_id)
        .bind(source_type)
        .bind(source_id)
        .bind(event_type)
        .bind(title)
        .bind(body)
        .bind(payload.to_string())
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn get_corp_token_callback_url(
        &self,
        corporate_user_id: i64,
    ) -> Option<String> {
        if corporate_user_id <= 0 {
            return None;
        }
        if let Some(pool) = &self.pool {
            if let Ok(Some(url)) = sqlx::query_scalar::<_, String>(
                "SELECT callback_url FROM fx_events.webhook_endpoints \
                 WHERE corporate_user_id = $1 AND kind = 'corp_token' AND status = 'active' \
                 LIMIT 1",
            )
            .bind(corporate_user_id)
            .fetch_optional(pool)
            .await
            {
                return Some(url);
            }
            if let Ok(Some(url)) = sqlx::query_scalar::<_, String>(
                "SELECT endpoint FROM fx_corp.partner_endpoints \
                 WHERE corporate_user_id = $1 AND type = 'callback' AND status = 'active' \
                 LIMIT 1",
            )
            .bind(corporate_user_id)
            .fetch_optional(pool)
            .await
            {
                return Some(url);
            }
        }
        self.memory.as_ref().and_then(|_| None)
    }

    pub async fn insert_corp_partner_notice(
        &self,
        corporate_user_id: i64,
        title: &str,
        body: &str,
    ) -> Result<(), String> {
        let now = now_ms();
        let Some(pool) = &self.pool else {
            return Ok(());
        };
        sqlx::query(
            "INSERT INTO fx_corp.corp_partner_notices ( \
                corporate_user_id, notice_type, title, body, deadline_at, \
                created_by_admin_id, read_at, created_at \
             ) VALUES ($1, 'other', $2, $3, 0, NULL, 0, $4)",
        )
        .bind(corporate_user_id)
        .bind(title)
        .bind(body)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn list_endpoints(&self, corporate_user_id: i64) -> Result<Vec<WebhookEndpoint>, String> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT id, corporate_user_id, kind, callback_url, auth_type, secret_hint, status, created_at, updated_at \
                 FROM fx_events.webhook_endpoints WHERE corporate_user_id = $1 ORDER BY id",
            )
            .bind(corporate_user_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(rows.into_iter().map(map_endpoint).collect());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        Ok(mem
            .endpoints
            .read()
            .await
            .values()
            .filter(|e| e.corporate_user_id == corporate_user_id)
            .cloned()
            .collect())
    }

    pub async fn upsert_endpoint(
        &self,
        corporate_user_id: i64,
        kind: &str,
        callback_url: &str,
        auth_type: &str,
        secret_hint: Option<&str>,
        status: &str,
    ) -> Result<WebhookEndpoint, String> {
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let row = sqlx::query(
                "INSERT INTO fx_events.webhook_endpoints ( \
                    corporate_user_id, kind, callback_url, auth_type, secret_hint, status, created_at, updated_at \
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, 0) \
                 ON CONFLICT (corporate_user_id, kind) DO UPDATE SET \
                    callback_url = EXCLUDED.callback_url, \
                    auth_type = EXCLUDED.auth_type, \
                    secret_hint = EXCLUDED.secret_hint, \
                    status = EXCLUDED.status, \
                    updated_at = $7 \
                 RETURNING id, corporate_user_id, kind, callback_url, auth_type, secret_hint, status, created_at, updated_at",
            )
            .bind(corporate_user_id)
            .bind(kind)
            .bind(callback_url)
            .bind(auth_type)
            .bind(secret_hint)
            .bind(status)
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(map_endpoint(row));
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut g = mem.endpoints.write().await;
        if let Some(existing) = g.values_mut().find(|e| e.corporate_user_id == corporate_user_id && e.kind == kind)
        {
            existing.callback_url = callback_url.into();
            existing.auth_type = auth_type.into();
            existing.secret_hint = secret_hint.map(|s| s.to_string());
            existing.status = status.into();
            existing.updated_at = now;
            return Ok(existing.clone());
        }
        let id = mem.next_endpoint.fetch_add(1, Ordering::Relaxed);
        let rec = WebhookEndpoint {
            id,
            corporate_user_id,
            kind: kind.into(),
            callback_url: callback_url.into(),
            auth_type: auth_type.into(),
            secret_hint: secret_hint.map(|s| s.to_string()),
            status: status.into(),
            created_at: now,
            updated_at: 0,
        };
        g.insert(id, rec.clone());
        Ok(rec)
    }

    pub async fn update_endpoint(
        &self,
        corporate_user_id: i64,
        id: i64,
        callback_url: Option<&str>,
        auth_type: Option<&str>,
        secret_hint: Option<&str>,
        status: Option<&str>,
    ) -> Result<WebhookEndpoint, String> {
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let row = sqlx::query(
                "UPDATE fx_events.webhook_endpoints SET \
                    callback_url = COALESCE($1, callback_url), \
                    auth_type = COALESCE($2, auth_type), \
                    secret_hint = COALESCE($3, secret_hint), \
                    status = COALESCE($4, status), \
                    updated_at = $5 \
                 WHERE id = $6 AND corporate_user_id = $7 \
                 RETURNING id, corporate_user_id, kind, callback_url, auth_type, secret_hint, status, created_at, updated_at",
            )
            .bind(callback_url)
            .bind(auth_type)
            .bind(secret_hint)
            .bind(status)
            .bind(now)
            .bind(id)
            .bind(corporate_user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "endpoint not found".to_string())?;
            return Ok(map_endpoint(row));
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut g = mem.endpoints.write().await;
        let rec = g.get_mut(&id).ok_or_else(|| "endpoint not found".to_string())?;
        if rec.corporate_user_id != corporate_user_id {
            return Err("endpoint not found".into());
        }
        if let Some(u) = callback_url {
            rec.callback_url = u.into();
        }
        if let Some(a) = auth_type {
            rec.auth_type = a.into();
        }
        if let Some(h) = secret_hint {
            rec.secret_hint = Some(h.into());
        }
        if let Some(s) = status {
            rec.status = s.into();
        }
        rec.updated_at = now;
        Ok(rec.clone())
    }
}

pub enum InsertOutcome {
    Inserted(WebhookEvent),
    Duplicate(WebhookEvent),
}

pub const MAX_RETRIES: i32 = 4;

pub fn backoff_ms(retry_count: i32) -> i64 {
    match retry_count {
        1 => 5_000,
        2 => 30_000,
        3 => 120_000,
        _ => 600_000,
    }
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn map_event(row: sqlx::postgres::PgRow) -> WebhookEvent {
    let payload: Value = row
        .try_get::<String, _>("payload")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null);
    WebhookEvent {
        id: row.get("id"),
        event_id: row.get("event_id"),
        source: row.get("source"),
        event_type: row.get("event_type"),
        payload,
        delivery_status: row.get("delivery_status"),
        retry_count: row.get("retry_count"),
        last_error: row.get("last_error"),
        next_retry_at: row.get("next_retry_at"),
        shop_order_id: row.get("shop_order_id"),
        partner_order_no: row.get("partner_order_no"),
        received_at: row.get("received_at"),
        updated_at: row.get("updated_at"),
    }
}

fn map_endpoint(row: sqlx::postgres::PgRow) -> WebhookEndpoint {
    WebhookEndpoint {
        id: row.get("id"),
        corporate_user_id: row.get("corporate_user_id"),
        kind: row.get("kind"),
        callback_url: row.get("callback_url"),
        auth_type: row.get("auth_type"),
        secret_hint: row.get("secret_hint"),
        status: row.get("status"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
