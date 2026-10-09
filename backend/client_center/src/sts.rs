//! HTTP client for Session Token Server (token issue / validate / revoke + OAuth URLs).

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::Config;
use crate::models::ErrorBody;

#[derive(Clone)]
pub struct StsClient {
    http: Client,
    base: String,
    internal_key: String,
}

#[derive(Debug, Deserialize)]
pub struct StsSession {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub expires_at_ms: i64,
    pub end_user_id: i64,
    pub username: Option<String>,
    pub account_id: Option<String>,
    pub scope: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub actor_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StsValidate {
    pub valid: bool,
    pub end_user_id: Option<i64>,
    pub username: Option<String>,
    #[allow(dead_code)]
    pub account_id: Option<String>,
    pub scope: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub actor_type: Option<String>,
    pub expires_at_ms: Option<i64>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct IssueBody<'a> {
    grant_type: &'a str,
    end_user_id: i64,
    username: Option<&'a str>,
    account_id: Option<&'a str>,
    scope: &'a str,
}

impl StsClient {
    pub fn new(config: &Config, http: Client) -> Self {
        Self {
            http,
            base: config.session_token_server_url.clone(),
            internal_key: config.session_internal_api_key.clone(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base
    }

    pub async fn issue_session(
        &self,
        grant_type: &str,
        end_user_id: i64,
        username: Option<&str>,
        account_id: Option<&str>,
    ) -> Result<StsSession, (reqwest::StatusCode, ErrorBody)> {
        let body = IssueBody {
            grant_type,
            end_user_id,
            username,
            account_id,
            scope: "http,socket",
        };
        let resp = self
            .http
            .post(format!("{}/v1/session/token", self.base))
            .json(&body)
            .send()
            .await
            .map_err(|e| gateway_err(format!("sts token: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let err = resp
                .json::<ErrorBody>()
                .await
                .unwrap_or(ErrorBody {
                    error: format!("sts token failed ({status})"),
                });
            return Err((status, err));
        }
        resp.json::<StsSession>()
            .await
            .map_err(|e| gateway_err(format!("sts token json: {e}")))
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
            .map_err(|e| gateway_err(format!("sts validate: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let err = resp
                .json::<ErrorBody>()
                .await
                .unwrap_or(ErrorBody {
                    error: format!("sts validate failed ({status})"),
                });
            return Err((status, err));
        }
        resp.json::<StsValidate>()
            .await
            .map_err(|e| gateway_err(format!("sts validate json: {e}")))
    }

    pub async fn revoke(
        &self,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
    ) -> Result<Value, (reqwest::StatusCode, ErrorBody)> {
        let mut req = self
            .http
            .post(format!("{}/v1/session/revoke", self.base))
            .header("x-internal-key", &self.internal_key);
        if let Some(t) = access_token {
            req = req.header(reqwest::header::AUTHORIZATION, format!("Bearer {t}"));
        }
        let resp = req
            .json(&serde_json::json!({
                "access_token": access_token,
                "refresh_token": refresh_token,
            }))
            .send()
            .await
            .map_err(|e| gateway_err(format!("sts revoke: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let err = resp
                .json::<ErrorBody>()
                .await
                .unwrap_or(ErrorBody {
                    error: format!("sts revoke failed ({status})"),
                });
            return Err((status, err));
        }
        Ok(resp.json().await.unwrap_or(Value::Null))
    }

    /// Forward raw HTTP to STS (platform OAuth token / userinfo / login HTML).
    pub async fn forward(
        &self,
        method: reqwest::Method,
        path_and_query: &str,
        headers: &axum::http::HeaderMap,
        body: Option<Vec<u8>>,
    ) -> Result<reqwest::Response, (reqwest::StatusCode, ErrorBody)> {
        let url = format!("{}{}", self.base, path_and_query);
        let mut builder = self.http.request(method, url);
        if let Some(ct) = headers.get(axum::http::header::CONTENT_TYPE) {
            builder = builder.header(reqwest::header::CONTENT_TYPE, ct.as_bytes());
        }
        if let Some(auth) = headers.get(axum::http::header::AUTHORIZATION) {
            builder = builder.header(reqwest::header::AUTHORIZATION, auth.as_bytes());
        }
        if let Some(b) = body {
            builder = builder.body(b);
        }
        builder
            .send()
            .await
            .map_err(|e| gateway_err(format!("sts forward: {e}")))
    }
}

fn gateway_err(msg: String) -> (reqwest::StatusCode, ErrorBody) {
    (
        reqwest::StatusCode::BAD_GATEWAY,
        ErrorBody { error: msg },
    )
}
