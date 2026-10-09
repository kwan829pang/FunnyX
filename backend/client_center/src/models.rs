use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub expires_at_ms: i64,
    pub end_user_id: i64,
    pub username: Option<String>,
    pub account_id: Option<String>,
    pub scope: String,
    pub grant_type: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileResponse {
    pub end_user_id: i64,
    pub username: String,
    pub email: Option<String>,
    pub status: String,
    pub partner_links: Vec<PartnerLinkView>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PartnerLinkView {
    pub partner_id: String,
    pub partner_user_id: String,
    pub game_id: Option<String>,
    pub game_account_id: Option<String>,
}
