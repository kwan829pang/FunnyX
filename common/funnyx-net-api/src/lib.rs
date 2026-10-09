//! Shared HTTP / network API contracts for Gateway, Client Center, Admin, Webhook.
//!
//! Paths align with `api-master.md`. Auth extractors are type-level helpers only
//! (wire into axum in each service).

use funnyx_auth::{AuthClass, MasterSignedCreds, SessionToken};
use funnyx_error::{FunnyxError, Result};
use funnyx_time::TimestampMs;
use serde::{Deserialize, Serialize};

/// Common JSON API envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiErrorBody>,
    pub timestamp_ms: i64,
}

/// Error body inside [`ApiResponse`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Result<Self> {
        Ok(Self {
            ok: true,
            data: Some(data),
            error: None,
            timestamp_ms: TimestampMs::now()?.as_i64(),
        })
    }

    pub fn failure(err: &FunnyxError) -> Result<ApiResponse<()>> {
        Ok(ApiResponse {
            ok: false,
            data: None,
            error: Some(ApiErrorBody {
                code: err.code().to_string(),
                message: err.to_string(),
            }),
            timestamp_ms: TimestampMs::now()?.as_i64(),
        })
    }
}

/// HTTP path constants (`api-master.md` subset; extend as APIs land).
pub mod paths {
    pub const HEALTH: &str = "/health";
    pub const V1: &str = "/v1";

    pub const CLIENT_REGISTER: &str = "/v1/client/register";
    pub const CLIENT_LOGIN: &str = "/v1/client/login";
    pub const CLIENT_LOGOUT: &str = "/v1/client/logout";
    pub const CLIENT_PROFILE: &str = "/v1/client/profile";
    pub const CLIENT_GAMES: &str = "/v1/client/games";
    pub const CLIENT_GAME_ACCOUNTS: &str = "/v1/client/game-accounts";
    pub const CLIENT_GAME_ACCOUNTS_BIND: &str = "/v1/client/game-accounts/bind";
    pub const CLIENT_WALLET: &str = "/v1/client/wallet";
    pub const CLIENT_DEPOSIT: &str = "/v1/client/deposit";
    pub const CLIENT_WITHDRAW: &str = "/v1/client/withdraw";
    pub const CLIENT_TRANSACTIONS: &str = "/v1/client/transactions";
    pub const CLIENT_CHAT_MESSAGES: &str = "/v1/client/chat/messages";
    pub const SESSION_TOKEN: &str = "/v1/session/token";
    pub const SESSION_VALIDATE: &str = "/v1/session/validate";
    pub const SESSION_REVOKE: &str = "/v1/session/revoke";

    pub const OAUTH_AUTHORIZE: &str = "/v1/oauth/authorize";
    pub const OAUTH_LOGIN: &str = "/v1/oauth/login";
    pub const OAUTH_TOKEN: &str = "/v1/oauth/token";
    pub const OAUTH_USERINFO: &str = "/v1/oauth/userinfo";
    pub const OAUTH_PARTNER_START: &str = "/v1/client/oauth/partner/{partner_id}/start";
    pub const OAUTH_PARTNER_CALLBACK: &str = "/v1/client/oauth/partner/callback";
    pub const OAUTH_PARTNER_COMPLETE: &str = "/v1/client/oauth/partner/complete";

    pub const ADMIN_LOGIN: &str = "/v1/admin/login";
    pub const ADMIN_LOGOUT: &str = "/v1/admin/logout";
    pub const ADMIN_ME: &str = "/v1/admin/me";
    pub const ADMIN_SHOP_CORP_PRODUCTS: &str = "/v1/admin/shop/corp-products";
    pub const ADMIN_SHOP_CORP_PRODUCT: &str = "/v1/admin/shop/corp-products/{id}";
    pub const ADMIN_SHOP_CORP_PRODUCT_STATUS: &str = "/v1/admin/shop/corp-products/{id}/status";
    pub const ADMIN_BASIC_TOKENS: &str = "/v1/admin/basic-tokens";
    pub const ADMIN_BASIC_TOKEN: &str = "/v1/admin/basic-tokens/{id}";
    pub const ADMIN_BASIC_TOKEN_APPROVE: &str = "/v1/admin/basic-tokens/{id}/approve";
    pub const ADMIN_BASIC_TOKEN_REJECT: &str = "/v1/admin/basic-tokens/{id}/reject";
    pub const ADMIN_SYSTEM_SETUP: &str = "/v1/admin/system/setup";
    pub const ADMIN_SYSTEM_SETUP_COMPLETE: &str = "/v1/admin/system/setup/complete";
    pub const ADMIN_SYSTEM_BASE_CURRENCY: &str = "/v1/admin/system/base-currency";
    pub const ADMIN_SHOP_PACKAGES: &str = "/v1/admin/shop/packages";
    pub const ADMIN_SHOP_PACKAGE: &str = "/v1/admin/shop/packages/{id}";
    pub const ADMIN_SHOP_PACKAGE_STATUS: &str = "/v1/admin/shop/packages/{id}/status";

    pub const ADMIN_GAME_COINS: &str = "/v1/admin/game-coins";
    pub const ADMIN_GAME_COIN: &str = "/v1/admin/game-coins/{id}";
    pub const ADMIN_GAME_COIN_STATUS: &str = "/v1/admin/game-coins/{id}/status";

    pub const ADMIN_MARKETS: &str = "/v1/admin/markets";
    pub const ADMIN_MARKET: &str = "/v1/admin/markets/{id}";
    pub const ADMIN_MARKET_APPROVE: &str = "/v1/admin/markets/{id}/approve";
    pub const ADMIN_MARKET_REJECT: &str = "/v1/admin/markets/{id}/reject";

    pub const CONFIG_RELOAD: &str = "/reload";
    pub const CONFIG_WHITELIST: &str = "/v1/config/whitelist";
    pub const CONFIG_SERVICES: &str = "/v1/config/services";
    pub const CONFIG_SERVICE_HEARTBEAT: &str = "/v1/config/services/heartbeat";
    pub const CONFIG_DEVICE_STATUS: &str = "/v1/config/device-status";

    pub const SHOP_BASE_CURRENCY: &str = "/v1/shop/base-currency";
    pub const SHOP_PACKAGES: &str = "/v1/shop/packages";
    pub const SHOP_PACKAGE: &str = "/v1/shop/packages/{id}";
    pub const SHOP_CORP_PRODUCTS: &str = "/v1/shop/corp-products";
    pub const SHOP_CORP_PRODUCT: &str = "/v1/shop/corp-products/{id}";
    pub const SHOP_ORDERS: &str = "/v1/shop/orders";
    pub const SHOP_ORDER: &str = "/v1/shop/orders/{id}";
    pub const SHOP_ORDER_CANCEL: &str = "/v1/shop/orders/{id}/cancel";
    pub const INTERNAL_SHOP_SETTLE: &str = "/v1/internal/shop/settle";
    pub const INTERNAL_CORP_TOKEN_SETTLE: &str = "/v1/internal/corp-token/settle";

    pub const WEBHOOK_SHOP_PAYMENT: &str = "/v1/webhook/shop/payment";
    pub const WEBHOOK_CORP_TOKEN_PAYMENT: &str = "/v1/webhook/corp-token/payment";
    pub const WEBHOOK_ENDPOINTS: &str = "/v1/webhook/endpoints";
    pub const WEBHOOK_ENDPOINT: &str = "/v1/webhook/endpoints/{id}";
    pub const WEBHOOK_EVENT: &str = "/v1/webhook/events/{event_id}";

    /// Message Center: enqueue (Internal / Core Engine) + ops list.
    pub const NOTICES: &str = "/v1/notices";
    pub const NOTICES_STATS: &str = "/v1/notices/stats";
    pub const NOTICE: &str = "/v1/notices/{id}";
    /// End-user inbox after drain (Session).
    pub const NOTIFICATIONS: &str = "/v1/notifications";
    pub const NOTIFICATION: &str = "/v1/notifications/{id}";

    pub const CORP_BASIC_TOKENS: &str = "/v1/corp/basic-tokens";
    pub const CORP_BASIC_TOKEN: &str = "/v1/corp/basic-tokens/{id}";
    pub const CORP_BASIC_TOKEN_BUY_ORDERS: &str = "/v1/corp/basic-tokens/{id}/buy-orders";
    pub const CORP_BASIC_TOKEN_FEES: &str = "/v1/corp/basic-tokens/{id}/fees";

    pub const CORP_TOKENS: &str = "/v1/corp-tokens";
    pub const CORP_TOKEN: &str = "/v1/corp-tokens/{id}";
    pub const CORP_TOKEN_CREATE_ORDER: &str = "/v1/corp-tokens/{id}/orders";
    pub const CORP_TOKEN_ORDERS: &str = "/v1/corp-tokens/orders";
    pub const CORP_TOKEN_ORDER: &str = "/v1/corp-tokens/orders/{order_id}";
    pub const CORP_TOKEN_ORDER_CANCEL: &str = "/v1/corp-tokens/orders/{order_id}/cancel";

    pub const CORP_SHOP_PRODUCTS: &str = "/v1/corp/shop/products";
    pub const CORP_SHOP_PRODUCT: &str = "/v1/corp/shop/products/{id}";
    pub const CORP_SHOP_PRODUCT_STATUS: &str = "/v1/corp/shop/products/{id}/status";
    pub const CORP_MARKETS: &str = "/v1/corp/markets";
    pub const CORP_MARKET: &str = "/v1/corp/markets/{id}";
    pub const CORP_MARKET_POOL: &str = "/v1/corp/markets/{id}/pool";
    pub const CORP_ENDPOINTS: &str = "/v1/corp/endpoints";
    pub const CORP_API_KEYS: &str = "/v1/corp/api-keys";
}

/// Header names for Session and MasterSigned (also in `funnyx-config::constants`).
pub mod headers {
    pub const AUTHORIZATION: &str = "Authorization";
    pub const MASTER_ACCOUNT_CODE: &str = "X-Master-Account-Code";
    pub const MASTER_ID: &str = "X-Master-Id";
    pub const API_KEY: &str = "X-Api-Key";
    pub const SIGNATURE: &str = "X-Signature";
    pub const TIMESTAMP: &str = "X-Timestamp";
}

/// Parsed auth material from headers (service middleware fills this).
#[derive(Debug, Clone)]
pub enum RequestAuth {
    Public,
    Session(SessionToken),
    Corp {
        creds: MasterSignedCreds,
        signature: String,
    },
}

impl RequestAuth {
    pub fn class(&self) -> AuthClass {
        match self {
            Self::Public => AuthClass::Public,
            Self::Session(_) => AuthClass::Session,
            Self::Corp { .. } => AuthClass::Corp,
        }
    }

    /// Parse `Authorization: Bearer <token>`.
    pub fn from_bearer(header: Option<&str>) -> Result<Self> {
        let Some(raw) = header else {
            return Err(FunnyxError::Auth("missing Authorization".into()));
        };
        let token = raw
            .strip_prefix("Bearer ")
            .or_else(|| raw.strip_prefix("bearer "))
            .ok_or_else(|| FunnyxError::Auth("expected Bearer token".into()))?;
        Ok(Self::Session(SessionToken::new(token)?))
    }
}

/// Require a given auth class for a route (helper for service middleware).
pub fn require_class(actual: AuthClass, expected: AuthClass) -> Result<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(FunnyxError::Auth(format!(
            "expected {expected:?}, got {actual:?}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_parse() {
        let auth = RequestAuth::from_bearer(Some("Bearer abc123")).unwrap();
        assert_eq!(auth.class(), AuthClass::Session);
    }

    #[test]
    fn success_envelope() {
        let r = ApiResponse::success("ping").unwrap();
        assert!(r.ok);
        assert_eq!(r.data.as_deref(), Some("ping"));
    }
}
