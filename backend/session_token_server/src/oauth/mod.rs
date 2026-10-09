mod partner;
mod platform;

use axum::routing::{get, post};
use axum::Router;

use crate::api::AppState;

pub use partner::{partner_callback, partner_start};
pub use platform::{
    oauth_authorize, oauth_login_page, oauth_login_submit, oauth_token, oauth_userinfo,
};

pub fn oauth_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/v1/oauth/authorize", get(oauth_authorize))
        .route(
            "/v1/oauth/login",
            get(oauth_login_page).post(oauth_login_submit),
        )
        .route("/v1/oauth/token", post(oauth_token))
        .route("/v1/oauth/userinfo", get(oauth_userinfo))
        .route(
            "/v1/client/oauth/partner/{partner_id}/start",
            get(partner_start),
        )
        .route(
            "/v1/client/oauth/partner/callback",
            get(partner_callback),
        )
}
