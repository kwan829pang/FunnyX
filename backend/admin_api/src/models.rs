use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminSessionResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub expires_at_ms: i64,
    pub admin_user_id: i64,
    pub username: String,
    pub role: String,
    pub scope: String,
    pub actor_type: String,
    pub grant_type: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminMeResponse {
    pub admin_user_id: i64,
    pub username: String,
    pub role: String,
    pub status: String,
    pub scope: Option<String>,
    pub source: String,
}
