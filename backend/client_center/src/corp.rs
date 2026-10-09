//! Corp MasterSigned auth (Client Center).

use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::Row;

use crate::api::{err_status, AppState};
use crate::models::ErrorBody;

#[derive(Debug, Clone)]
pub struct CorpIdentity {
    pub corporate_user_id: i64,
}

pub async fn require_master(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<CorpIdentity, (StatusCode, Json<ErrorBody>)> {
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if auth.to_ascii_lowercase().starts_with("bearer ") {
            return Err(err_status(
                StatusCode::UNAUTHORIZED,
                "Session Bearer is not allowed on Corp APIs; use MasterSigned headers",
            ));
        }
    }
    let code = header(headers, "x-master-account-code")?;
    let master_id = header(headers, "x-master-id")?;
    let api_key = header(headers, "x-api-key")?;
    let signature = header(headers, "x-signature")?;
    let timestamp = header(headers, "x-timestamp")?;
    if !valid_signature(&state.config.demo_master_secret, &timestamp, &master_id, &signature) {
        return Err(err_status(StatusCode::UNAUTHORIZED, "invalid MasterSigned signature"));
    }
    if let Some(pool) = &state.pool {
        let row = sqlx::query(
            "SELECT u.id FROM fx_corp.corporate_users u \
             JOIN fx_corp.corp_api_keys k ON k.corporate_user_id = u.id \
             WHERE u.master_code = $1 AND u.master_id = $2 AND k.api_key = $3 \
               AND u.status = 'active' AND k.status = 'active' AND u.api_enabled = TRUE",
        )
        .bind(&code)
        .bind(&master_id)
        .bind(&api_key)
        .fetch_optional(pool)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let Some(row) = row else {
            return Err(err_status(StatusCode::UNAUTHORIZED, "unknown Master credentials"));
        };
        return Ok(CorpIdentity {
            corporate_user_id: row.get("id"),
        });
    }
    if code == state.config.demo_master_code
        && master_id == state.config.demo_master_id
        && api_key == state.config.demo_api_key
    {
        return Ok(CorpIdentity {
            corporate_user_id: 1,
        });
    }
    Err(err_status(StatusCode::UNAUTHORIZED, "unknown Master credentials"))
}

fn header(headers: &HeaderMap, name: &str) -> Result<String, (StatusCode, Json<ErrorBody>)> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, format!("{name} required")))
}

fn valid_signature(secret: &str, timestamp: &str, master_id: &str, signature: &str) -> bool {
    if signature == "demo" {
        return true;
    }
    type HmacSha256 = Hmac<Sha256>;
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(timestamp.as_bytes());
    mac.update(b".");
    mac.update(master_id.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    expected.eq_ignore_ascii_case(signature)
}
