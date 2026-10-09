//! Partner fiat payment create (payment-gate simulator / partner pay endpoint).

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::models::ErrorBody;
use crate::shop_store::ShopOrderView;

#[derive(Clone)]
pub struct PaymentGateClient {
    http: Client,
    base: String,
}

#[derive(Debug, Serialize)]
struct CreateShopPaymentRequest<'a> {
    request_id: String,
    shop_order_id: i64,
    seller_type: &'a str,
    partner_id: &'a str,
    user_id: String,
    game_account_id: Option<String>,
    package_code: Option<&'a str>,
    product_code: Option<&'a str>,
    credit_game_coin: Option<&'a str>,
    credit_amount: Option<f64>,
    fiat_currency: &'a str,
    fiat_price: f64,
    callback_url: &'a str,
    return_url: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShopPaymentResponse {
    pub partner_order_no: String,
    pub checkout_url: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub status: String,
}

impl PaymentGateClient {
    pub fn new(config: &Config, http: Client) -> Self {
        Self {
            http,
            base: config.payment_gate_url.clone(),
        }
    }

    pub async fn create_shop_payment(
        &self,
        config: &Config,
        order: &ShopOrderView,
        return_url: Option<&str>,
        game_account_ext: Option<&str>,
    ) -> Result<CreateShopPaymentResponse, (reqwest::StatusCode, ErrorBody)> {
        let body = CreateShopPaymentRequest {
            request_id: format!("req_shop_{}", order.id),
            shop_order_id: order.id,
            seller_type: &order.seller_type,
            partner_id: &config.default_partner_id,
            user_id: order.end_user_id.to_string(),
            game_account_id: game_account_ext.map(|s| s.to_string()),
            package_code: order.package_code.as_deref(),
            product_code: order.product_code.as_deref(),
            credit_game_coin: order.credit_game_coin.as_deref(),
            credit_amount: order.credit_amount,
            fiat_currency: &order.fiat_currency,
            fiat_price: order.fiat_price,
            callback_url: &config.shop_payment_callback_url,
            return_url,
        };
        let resp = self
            .http
            .post(format!("{}/v1/shop/payment", self.base))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                (
                    reqwest::StatusCode::BAD_GATEWAY,
                    ErrorBody {
                        error: format!("payment-gate: {e}"),
                    },
                )
            })?;
        let status = resp.status();
        if !status.is_success() {
            let err = resp.json::<ErrorBody>().await.unwrap_or(ErrorBody {
                error: format!("payment-gate failed ({status})"),
            });
            return Err((status, err));
        }
        resp.json()
            .await
            .map_err(|e| {
                (
                    reqwest::StatusCode::BAD_GATEWAY,
                    ErrorBody {
                        error: format!("payment-gate json: {e}"),
                    },
                )
            })
    }
}
