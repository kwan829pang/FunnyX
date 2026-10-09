//! Partner OAuth broker — Client Web verifies users via Partner OAuth, then STS issues session.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::api::{bad_request, err_status, issue_session_for_user, AppState};
use crate::models::ErrorBody;
use crate::oauth::platform::urlencoding;

#[derive(Debug, Deserialize)]
pub struct PartnerStartQuery {
    /// Optional Client Web return URL after successful session issue.
    /// Omit for JSON callback response (API / smoke).
    #[serde(default)]
    pub redirect_uri: Option<String>,
}

pub async fn partner_start(
    State(state): State<AppState>,
    Path(partner_id): Path<String>,
    Query(q): Query<PartnerStartQuery>,
) -> Result<Redirect, (StatusCode, Json<ErrorBody>)> {
    let partner = state
        .config
        .partner(&partner_id)
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "unknown partner_id"))?
        .clone();

    let client_redirect = q.redirect_uri.filter(|s| !s.is_empty());

    let state_token = state
        .oauth
        .put_partner_state(
            partner.partner_id.clone(),
            client_redirect,
            state.config.oauth_code_ttl_secs,
        )
        .await;

    let loc = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&state={}&scope={}",
        partner.authorize_url,
        urlencoding(&partner.client_id),
        urlencoding(&partner.redirect_uri),
        urlencoding(&state_token),
        urlencoding("openid profile game"),
    );
    Ok(Redirect::to(&loc))
}

#[derive(Debug, Deserialize)]
pub struct PartnerCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PartnerCallbackResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub expires_at_ms: i64,
    pub end_user_id: i64,
    pub username: Option<String>,
    pub partner_id: String,
    pub partner_user_id: String,
    pub game_account_id: Option<String>,
    pub game_id: Option<String>,
    pub grant_type: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
struct PartnerTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct PartnerUserInfo {
    partner_user_id: String,
    #[serde(default)]
    game_id: Option<String>,
    #[serde(default)]
    game_account_id: Option<String>,
    #[serde(default)]
    username: Option<String>,
}

pub async fn partner_callback(
    State(state): State<AppState>,
    Query(q): Query<PartnerCallbackQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    if let Some(err) = q.error {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            format!("partner oauth error: {err}"),
        ));
    }
    let code = q
        .code
        .ok_or_else(|| bad_request("code required from partner"))?;
    let state_token = q
        .state
        .ok_or_else(|| bad_request("state required"))?;
    let pending = state
        .oauth
        .take_partner_state(&state_token)
        .await
        .ok_or_else(|| bad_request("invalid or expired state"))?;
    let partner = state
        .config
        .partner(&pending.partner_id)
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "unknown partner_id"))?
        .clone();

    let client = state.http.clone();
    let token_form = [
        ("grant_type", "authorization_code"),
        ("code", code.as_str()),
        ("client_id", partner.client_id.as_str()),
        ("client_secret", partner.client_secret.as_str()),
        ("redirect_uri", partner.redirect_uri.as_str()),
    ];
    let token_resp = client
        .post(&partner.token_url)
        .form(&token_form)
        .send()
        .await
        .map_err(|e| err_status(StatusCode::BAD_GATEWAY, format!("partner token: {e}")))?;
    if !token_resp.status().is_success() {
        let body = token_resp.text().await.unwrap_or_default();
        return Err(err_status(
            StatusCode::BAD_GATEWAY,
            format!("partner token failed: {body}"),
        ));
    }
    let token_body: PartnerTokenResponse = token_resp
        .json()
        .await
        .map_err(|e| err_status(StatusCode::BAD_GATEWAY, format!("partner token json: {e}")))?;

    let userinfo_resp = client
        .get(&partner.userinfo_url)
        .bearer_auth(&token_body.access_token)
        .send()
        .await
        .map_err(|e| err_status(StatusCode::BAD_GATEWAY, format!("partner userinfo: {e}")))?;
    if !userinfo_resp.status().is_success() {
        let body = userinfo_resp.text().await.unwrap_or_default();
        return Err(err_status(
            StatusCode::BAD_GATEWAY,
            format!("partner userinfo failed: {body}"),
        ));
    }
    let info: PartnerUserInfo = userinfo_resp
        .json()
        .await
        .map_err(|e| err_status(StatusCode::BAD_GATEWAY, format!("partner userinfo json: {e}")))?;

    let user = state
        .users
        .link_or_create_from_partner(
            &partner.partner_id,
            &info.partner_user_id,
            info.username.clone(),
            info.game_id.clone(),
            info.game_account_id.clone(),
        )
        .await;

    let session = issue_session_for_user(
        &state,
        user.end_user_id,
        Some(user.username.clone()),
        Some(user.username.clone()),
        "oauth".into(),
    )
    .await?;

    let payload = PartnerCallbackResponse {
        access_token: session.access_token.clone(),
        refresh_token: session.refresh_token.clone(),
        token_type: "Bearer".into(),
        expires_in: state.config.token_ttl_secs,
        expires_at_ms: session.expires_at_ms,
        end_user_id: user.end_user_id,
        username: Some(user.username),
        partner_id: partner.partner_id,
        partner_user_id: info.partner_user_id.clone(),
        game_account_id: info.game_account_id.clone(),
        game_id: info.game_id.clone(),
        grant_type: "oauth".into(),
        source: "session_token_server".into(),
    };

    // Browser Client Web flow: start with ?redirect_uri=… → redirect with tokens.
    // API / smoke: omit redirect_uri → JSON body.
    if let Some(redirect) = pending.client_redirect.filter(|s| !s.is_empty()) {
        let loc = format!(
            "{}{}access_token={}&refresh_token={}&end_user_id={}&token_type=Bearer&partner_id={}&partner_user_id={}&game_account_id={}&game_id={}",
            redirect,
            if redirect.contains('?') { "&" } else { "?" },
            urlencoding(&payload.access_token),
            urlencoding(&payload.refresh_token),
            payload.end_user_id,
            urlencoding(&payload.partner_id),
            urlencoding(&payload.partner_user_id),
            urlencoding(payload.game_account_id.as_deref().unwrap_or("")),
            urlencoding(payload.game_id.as_deref().unwrap_or("")),
        );
        return Ok(Redirect::to(&loc).into_response());
    }

    Ok((StatusCode::CREATED, Json(payload)).into_response())
}
