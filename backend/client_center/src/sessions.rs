//! Persist STS-issued sessions into `fx_user.user_sessions`.

use sqlx::PgPool;

use crate::db::now_ms;
use crate::sts::StsSession;

pub struct SessionWrite<'a> {
    pub session: &'a StsSession,
    pub grant_type: &'a str,
    pub partner_id: Option<&'a str>,
    pub partner_user_id: Option<&'a str>,
    pub game_account_id: Option<&'a str>,
    pub oauth_identity_id: Option<i64>,
}

pub async fn record_session(pool: Option<&PgPool>, w: SessionWrite<'_>) {
    let Some(pool) = pool else {
        return;
    };
    let now = now_ms();
    let refresh = if w.session.refresh_token.is_empty() {
        None
    } else {
        Some(w.session.refresh_token.as_str())
    };
    let _ = sqlx::query(
        crate::query::sessions::INSERT,
    )
    .bind(w.session.end_user_id)
    .bind(&w.session.access_token)
    .bind(refresh)
    .bind(&w.session.token_type)
    .bind(w.grant_type)
    .bind(&w.session.scope)
    .bind(w.partner_id)
    .bind(w.partner_user_id)
    .bind(w.game_account_id)
    .bind(w.oauth_identity_id)
    .bind(w.session.expires_at_ms)
    .bind(0_i64)
    .bind(now)
    .execute(pool)
    .await;
}

pub async fn revoke_session(
    pool: Option<&PgPool>,
    access_token: Option<&str>,
    refresh_token: Option<&str>,
) {
    let Some(pool) = pool else {
        return;
    };
    let now = now_ms();
    if let Some(t) = access_token.filter(|s| !s.is_empty()) {
        let _ = sqlx::query(
            crate::query::sessions::REVOKE_BY_TOKEN,
        )
        .bind(now)
        .bind(t)
        .execute(pool)
        .await;
    }
    if let Some(t) = refresh_token.filter(|s| !s.is_empty()) {
        let _ = sqlx::query(
            crate::query::sessions::REVOKE_BY_REFRESH,
        )
        .bind(now)
        .bind(t)
        .execute(pool)
        .await;
    }
}
