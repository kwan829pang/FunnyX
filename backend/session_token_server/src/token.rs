use chrono::Utc;
use uuid::Uuid;

use crate::models::SessionRecord;

pub fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn new_access_token() -> String {
    format!("sts_{}", Uuid::new_v4().simple())
}

pub fn new_refresh_token() -> String {
    format!("str_{}", Uuid::new_v4().simple())
}

pub fn build_session(
    end_user_id: i64,
    username: Option<String>,
    account_id: Option<String>,
    scope: String,
    grant_type: String,
    actor_type: String,
    token_ttl_secs: u64,
    refresh_ttl_secs: u64,
) -> SessionRecord {
    let issued_at_ms = now_ms();
    let access = new_access_token();
    let refresh = new_refresh_token();
    SessionRecord {
        token_id: access.clone(),
        access_token: access,
        refresh_token: refresh,
        end_user_id,
        username,
        account_id,
        scope,
        grant_type,
        actor_type,
        issued_at_ms,
        expires_at_ms: issued_at_ms + (token_ttl_secs as i64) * 1000,
        refresh_expires_at_ms: issued_at_ms + (refresh_ttl_secs as i64) * 1000,
    }
}

pub fn is_expired(expires_at_ms: i64) -> bool {
    now_ms() >= expires_at_ms
}
