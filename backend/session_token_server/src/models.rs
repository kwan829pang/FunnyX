use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub token_id: String,
    pub access_token: String,
    pub refresh_token: String,
    /// Subject id: end_user_id or admin_user_id depending on `actor_type`.
    pub end_user_id: i64,
    pub username: Option<String>,
    pub account_id: Option<String>,
    pub scope: String,
    pub grant_type: String,
    /// `end_user` | `admin`
    #[serde(default = "default_actor_type")]
    pub actor_type: String,
    pub issued_at_ms: i64,
    pub expires_at_ms: i64,
    pub refresh_expires_at_ms: i64,
}

fn default_actor_type() -> String {
    "end_user".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueTokenRequest {
    /// `login` | `oauth` | `refresh` | `admin_login`
    pub grant_type: String,
    #[serde(default)]
    pub end_user_id: Option<i64>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default = "default_scope")]
    pub scope: String,
    /// `end_user` (default) | `admin`
    #[serde(default = "default_actor_type")]
    pub actor_type: String,
    /// Required when grant_type = refresh
    #[serde(default)]
    pub refresh_token: Option<String>,
}

fn default_scope() -> String {
    "http,socket".into()
}

#[derive(Debug, Clone, Serialize)]
pub struct IssueTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub expires_at_ms: i64,
    pub end_user_id: i64,
    pub username: Option<String>,
    pub account_id: Option<String>,
    pub scope: String,
    pub actor_type: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidateTokenRequest {
    /// Access token. Also accepted via `Authorization: Bearer`.
    #[serde(default)]
    pub access_token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidateTokenResponse {
    pub valid: bool,
    pub end_user_id: Option<i64>,
    pub username: Option<String>,
    pub account_id: Option<String>,
    pub scope: Option<String>,
    pub actor_type: Option<String>,
    pub expires_at_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RevokeTokenRequest {
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RevokeTokenResponse {
    pub revoked: bool,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    pub error: String,
}
