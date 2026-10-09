//! Core Engine client: resolve URL via Config Server / env, create pair on approve.

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::config::Config;
use crate::models::ErrorBody;

#[derive(Clone)]
pub struct EngineClient {
    http: Client,
    config: Config,
}

#[derive(Debug, Deserialize)]
struct ServiceListResponse {
    services: Vec<RemoteService>,
}

#[derive(Debug, Deserialize)]
struct RemoteService {
    service_name: String,
    http_url: String,
    #[serde(default)]
    status: String,
}

impl EngineClient {
    pub fn new(config: &Config, http: Client) -> Self {
        Self {
            http,
            config: config.clone(),
        }
    }

    pub async fn resolve_url(&self) -> String {
        if let Ok(Some(url)) = self.lookup_config_server().await {
            return url;
        }
        self.config.core_engine_url.clone()
    }

    async fn lookup_config_server(&self) -> Result<Option<String>, String> {
        let base = self.config.config_server_url.trim();
        if base.is_empty() {
            return Ok(None);
        }
        let resp = self
            .http
            .get(format!("{}/v1/config/services", base))
            .header("x-internal-key", &self.config.internal_api_key)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Ok(None);
        }
        let list: ServiceListResponse = resp.json().await.map_err(|e| e.to_string())?;
        let found = list.services.into_iter().find(|s| {
            let n = s.service_name.to_ascii_lowercase();
            (n.contains("core") || n.contains("engine"))
                && (s.status.is_empty()
                    || s.status.eq_ignore_ascii_case("healthy")
                    || s.status.eq_ignore_ascii_case("ok")
                    || s.status.eq_ignore_ascii_case("active"))
        });
        Ok(found.map(|s| s.http_url.trim_end_matches('/').to_string()))
    }

    pub async fn create_pair(
        &self,
        symbol: &str,
        market_id: &str,
        pool_depth: f64,
        initial_price: f64,
        base_amount: f64,
        quote_amount: f64,
        funding_source: &str,
        admin_id: i64,
        corporate_user_id: i64,
    ) -> Result<serde_json::Value, (reqwest::StatusCode, ErrorBody)> {
        let base = self.resolve_url().await;
        let url = format!("{}/v1/admin/pairs", base.trim_end_matches('/'));
        let body = json!({
            "symbol": symbol,
            "market_id": market_id,
            "pool_depth": pool_depth,
            "initial_price": initial_price,
            "base_amount": base_amount,
            "quote_amount": quote_amount,
            "funding_source": funding_source,
            "admin_id": admin_id.to_string(),
            "corporate_user_id": corporate_user_id,
            "seed_book": false,
        });
        let resp = self
            .http
            .post(&url)
            .header("x-admin-key", &self.config.core_engine_admin_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                (
                    reqwest::StatusCode::BAD_GATEWAY,
                    ErrorBody {
                        error: format!("core engine: {e}"),
                    },
                )
            })?;
        let status = resp.status();
        if !status.is_success() {
            let err = resp.json::<ErrorBody>().await.unwrap_or(ErrorBody {
                error: format!("core engine create pair failed ({status})"),
            });
            return Err((status, err));
        }
        resp.json::<serde_json::Value>().await.map_err(|e| {
            (
                reqwest::StatusCode::BAD_GATEWAY,
                ErrorBody {
                    error: format!("core engine json: {e}"),
                },
            )
        })
    }
}
