//! Session Token Server validate client.

use reqwest::Client;
use serde::Deserialize;

use crate::config::Config;
use crate::models::ErrorBody;

#[derive(Clone)]
pub struct StsClient {
    http: Client,
    base: String,
    internal_key: String,
}

#[derive(Debug, Deserialize)]
pub struct StsValidate {
    pub valid: bool,
    pub end_user_id: Option<i64>,
    pub reason: Option<String>,
}

impl StsClient {
    pub fn new(config: &Config, http: Client) -> Self {
        Self {
            http,
            base: config.session_token_server_url.clone(),
            internal_key: config.session_internal_api_key.clone(),
        }
    }

    pub async fn validate(
        &self,
        access_token: &str,
    ) -> Result<StsValidate, (reqwest::StatusCode, ErrorBody)> {
        let resp = self
            .http
            .post(format!("{}/v1/session/validate", self.base))
            .header("x-internal-key", &self.internal_key)
            .json(&serde_json::json!({ "access_token": access_token }))
            .send()
            .await
            .map_err(|e| {
                (
                    reqwest::StatusCode::BAD_GATEWAY,
                    ErrorBody {
                        error: format!("sts validate: {e}"),
                    },
                )
            })?;
        let status = resp.status();
        if !status.is_success() {
            let err = resp.json::<ErrorBody>().await.unwrap_or(ErrorBody {
                error: format!("sts validate failed ({status})"),
            });
            return Err((status, err));
        }
        resp.json::<StsValidate>().await.map_err(|e| {
            (
                reqwest::StatusCode::BAD_GATEWAY,
                ErrorBody {
                    error: format!("sts validate json: {e}"),
                },
            )
        })
    }
}
