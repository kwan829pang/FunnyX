//! Ephemeral OAuth authorization codes, OAuth access tokens, and partner start state.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::token::{is_expired, now_ms};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCodeRecord {
    pub code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub end_user_id: i64,
    pub username: String,
    pub scope: String,
    pub expires_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OauthAccessRecord {
    pub access_token: String,
    pub end_user_id: i64,
    pub username: String,
    pub client_id: String,
    pub scope: String,
    pub expires_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnerStartState {
    pub partner_id: String,
    pub client_redirect: Option<String>,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
}

#[derive(Clone, Default)]
pub struct OauthStore {
    inner: Arc<Mutex<OauthInner>>,
}

#[derive(Default)]
struct OauthInner {
    codes: HashMap<String, AuthCodeRecord>,
    oauth_tokens: HashMap<String, OauthAccessRecord>,
    partner_states: HashMap<String, PartnerStartState>,
}

impl OauthStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn issue_code(
        &self,
        client_id: String,
        redirect_uri: String,
        end_user_id: i64,
        username: String,
        scope: String,
        ttl_secs: u64,
    ) -> String {
        let code = format!("ocode_{}", Uuid::new_v4().simple());
        let rec = AuthCodeRecord {
            code: code.clone(),
            client_id,
            redirect_uri,
            end_user_id,
            username,
            scope,
            expires_at_ms: now_ms() + (ttl_secs as i64) * 1000,
        };
        self.inner.lock().await.codes.insert(code.clone(), rec);
        code
    }

    pub async fn take_code(&self, code: &str) -> Option<AuthCodeRecord> {
        let mut g = self.inner.lock().await;
        let rec = g.codes.remove(code)?;
        if is_expired(rec.expires_at_ms) {
            return None;
        }
        Some(rec)
    }

    pub async fn issue_oauth_access(
        &self,
        end_user_id: i64,
        username: String,
        client_id: String,
        scope: String,
        ttl_secs: u64,
    ) -> String {
        let access_token = format!("oat_{}", Uuid::new_v4().simple());
        let rec = OauthAccessRecord {
            access_token: access_token.clone(),
            end_user_id,
            username,
            client_id,
            scope,
            expires_at_ms: now_ms() + (ttl_secs as i64) * 1000,
        };
        self.inner
            .lock()
            .await
            .oauth_tokens
            .insert(access_token.clone(), rec);
        access_token
    }

    pub async fn get_oauth_access(&self, token: &str) -> Option<OauthAccessRecord> {
        let mut g = self.inner.lock().await;
        let Some(rec) = g.oauth_tokens.get(token).cloned() else {
            return None;
        };
        if is_expired(rec.expires_at_ms) {
            g.oauth_tokens.remove(token);
            return None;
        }
        Some(rec)
    }

    pub async fn put_partner_state(
        &self,
        partner_id: String,
        client_redirect: Option<String>,
        ttl_secs: u64,
    ) -> String {
        let state = format!("pst_{}", Uuid::new_v4().simple());
        let now = now_ms();
        self.inner.lock().await.partner_states.insert(
            state.clone(),
            PartnerStartState {
                partner_id,
                client_redirect,
                created_at_ms: now,
                expires_at_ms: now + (ttl_secs as i64) * 1000,
            },
        );
        state
    }

    pub async fn take_partner_state(&self, state: &str) -> Option<PartnerStartState> {
        let mut g = self.inner.lock().await;
        let rec = g.partner_states.remove(state)?;
        if is_expired(rec.expires_at_ms) {
            return None;
        }
        Some(rec)
    }
}
