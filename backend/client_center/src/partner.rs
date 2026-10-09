//! Partner HTTP helpers (company-a demo player lookup).

use reqwest::Client;
use serde::Deserialize;

use crate::config::Config;
use crate::models::ErrorBody;

#[derive(Clone)]
pub struct PartnerClient {
    http: Client,
    base: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct PartnerPlayer {
    pub partner_user_id: String,
    #[allow(dead_code)]
    pub game_id: String,
    pub game_account_id: String,
    #[allow(dead_code)]
    pub username: String,
    pub playing: bool,
}

impl PartnerClient {
    pub fn new(config: &Config, http: Client) -> Self {
        Self {
            http,
            base: config.partner_a_base_url.clone(),
            api_key: config.partner_a_api_key.clone(),
        }
    }

    pub async fn lookup_player(
        &self,
        partner_game_id: &str,
        platform_end_user_id: i64,
        partner_username: Option<&str>,
        partner_user_id: Option<&str>,
    ) -> Result<PartnerPlayer, (reqwest::StatusCode, ErrorBody)> {
        let mut url = reqwest::Url::parse(&format!("{}/api/players/lookup", self.base))
            .map_err(|e| gateway(format!("partner url: {e}")))?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("game_id", partner_game_id);
            q.append_pair("platform_end_user_id", &platform_end_user_id.to_string());
            if let Some(u) = partner_username.filter(|s| !s.is_empty()) {
                q.append_pair("username", u);
            }
            if let Some(p) = partner_user_id.filter(|s| !s.is_empty()) {
                q.append_pair("partner_user_id", p);
            }
        }
        let resp = self
            .http
            .get(url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| gateway(format!("partner lookup: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let err = resp.json::<ErrorBody>().await.unwrap_or(ErrorBody {
                error: format!("partner lookup failed ({status})"),
            });
            return Err((status, err));
        }
        resp.json::<PartnerPlayer>()
            .await
            .map_err(|e| gateway(format!("partner lookup json: {e}")))
    }
}

fn gateway(msg: String) -> (reqwest::StatusCode, ErrorBody) {
    (
        reqwest::StatusCode::BAD_GATEWAY,
        ErrorBody { error: msg },
    )
}
