//! Platform OAuth 2.0 IdP — Partners login platform-registered users.

use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect};
use axum::Form;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::api::{bad_request, err_status, extract_bearer, AppState};
use crate::models::ErrorBody;

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    response_type: Option<String>,
    client_id: Option<String>,
    redirect_uri: String,
    state: Option<String>,
    scope: Option<String>,
}

pub async fn oauth_authorize(
    State(state): State<AppState>,
    Query(q): Query<AuthorizeQuery>,
) -> Result<Redirect, (StatusCode, Json<ErrorBody>)> {
    if q.response_type.as_deref().unwrap_or("code") != "code" {
        return Err(bad_request("response_type must be code"));
    }
    let client_id = q
        .client_id
        .ok_or_else(|| bad_request("client_id required"))?;
    if client_id != state.config.platform_oauth_client.client_id {
        return Err(err_status(
            StatusCode::UNAUTHORIZED,
            "invalid client_id",
        ));
    }
    if !state
        .config
        .platform_oauth_client
        .redirect_uris
        .iter()
        .any(|u| u == &q.redirect_uri)
    {
        return Err(bad_request("redirect_uri not registered"));
    }

    let mut params = format!(
        "redirect_uri={}&client_id={}",
        urlencoding(&q.redirect_uri),
        urlencoding(&client_id)
    );
    if let Some(s) = &q.state {
        params.push_str(&format!("&state={}", urlencoding(s)));
    }
    if let Some(scope) = &q.scope {
        params.push_str(&format!("&scope={}", urlencoding(scope)));
    }
    Ok(Redirect::to(&format!("/v1/oauth/login?{params}")))
}

pub async fn oauth_login_page(Query(q): Query<HashMap<String, String>>) -> Html<String> {
    let redirect_uri = q.get("redirect_uri").cloned().unwrap_or_default();
    let state = q.get("state").cloned().unwrap_or_default();
    let client_id = q.get("client_id").cloned().unwrap_or_default();
    let scope = q
        .get("scope")
        .cloned()
        .unwrap_or_else(|| "openid profile".into());
    Html(format!(
        r#"<!doctype html>
<html><head><meta charset="utf-8"><title>FunnyX Platform OAuth</title>
<style>
body{{font-family:system-ui;max-width:28rem;margin:2rem auto;background:#0f172a;color:#e2e8f0}}
input,button{{display:block;width:100%;margin:.5rem 0;padding:.65rem;font:inherit;border-radius:6px;border:1px solid #334155;background:#1e293b;color:#e2e8f0}}
button{{background:#2563eb;border:none;cursor:pointer;font-weight:600}}
.hint{{opacity:.8;font-size:.9rem}}
code{{background:#1e293b;padding:.1rem .3rem;border-radius:4px}}
</style></head><body>
<h1>FunnyX Login</h1>
<p class="hint">Platform OAuth for Partners. Demo: <code>demo_user</code> / <code>alice_plat</code> (password <code>demo</code>)</p>
<form method="post" action="/v1/oauth/login">
<input type="hidden" name="redirect_uri" value="{redirect_uri}">
<input type="hidden" name="state" value="{state}">
<input type="hidden" name="client_id" value="{client_id}">
<input type="hidden" name="scope" value="{scope}">
<label>Username</label><input name="username" value="demo_user" autocomplete="username">
<label>Password</label><input name="password" type="password" value="demo" autocomplete="current-password">
<button type="submit">Authorize</button>
</form></body></html>"#
    ))
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
    redirect_uri: String,
    client_id: String,
    #[serde(default)]
    state: String,
    #[serde(default = "default_scope")]
    scope: String,
}

fn default_scope() -> String {
    "openid profile".into()
}

pub async fn oauth_login_submit(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Result<Redirect, (StatusCode, Html<String>)> {
    let user = state
        .users
        .authenticate(&form.username, &form.password)
        .await
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Html("<h1>Invalid credentials</h1><a href=\"/v1/oauth/authorize\">Back</a>".into()),
            )
        })?;
    if form.client_id != state.config.platform_oauth_client.client_id {
        return Err((
            StatusCode::UNAUTHORIZED,
            Html("<h1>Invalid client</h1>".into()),
        ));
    }
    if !state
        .config
        .platform_oauth_client
        .redirect_uris
        .iter()
        .any(|u| u == &form.redirect_uri)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Html("<h1>redirect_uri not registered</h1>".into()),
        ));
    }

    let code = state
        .oauth
        .issue_code(
            form.client_id,
            form.redirect_uri.clone(),
            user.end_user_id,
            user.username,
            form.scope,
            state.config.oauth_code_ttl_secs,
        )
        .await;

    let mut loc = append_query(&form.redirect_uri, &format!("code={code}"));
    if !form.state.is_empty() {
        loc.push_str(&format!("&state={}", urlencoding(&form.state)));
    }
    Ok(Redirect::to(&loc))
}

#[derive(Debug, Deserialize)]
pub struct TokenForm {
    grant_type: Option<String>,
    code: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    redirect_uri: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OauthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
}

pub async fn oauth_token(
    State(state): State<AppState>,
    Form(form): Form<TokenForm>,
) -> Result<Json<OauthTokenResponse>, (StatusCode, Json<ErrorBody>)> {
    if form.grant_type.as_deref().unwrap_or("authorization_code") != "authorization_code" {
        return Err(bad_request("unsupported grant_type"));
    }
    let client_id = form
        .client_id
        .ok_or_else(|| bad_request("client_id required"))?;
    let client_secret = form
        .client_secret
        .ok_or_else(|| bad_request("client_secret required"))?;
    if client_id != state.config.platform_oauth_client.client_id
        || client_secret != state.config.platform_oauth_client.client_secret
    {
        return Err(err_status(
            StatusCode::UNAUTHORIZED,
            "invalid client credentials",
        ));
    }
    let code = form.code.ok_or_else(|| bad_request("code required"))?;
    let rec = state
        .oauth
        .take_code(&code)
        .await
        .ok_or_else(|| bad_request("invalid or expired code"))?;
    if rec.client_id != client_id {
        return Err(err_status(StatusCode::UNAUTHORIZED, "client_id mismatch"));
    }
    if let Some(uri) = &form.redirect_uri {
        if uri != &rec.redirect_uri {
            return Err(bad_request("redirect_uri mismatch"));
        }
    }

    let access_token = state
        .oauth
        .issue_oauth_access(
            rec.end_user_id,
            rec.username,
            client_id,
            rec.scope.clone(),
            state.config.token_ttl_secs,
        )
        .await;

    Ok(Json(OauthTokenResponse {
        access_token,
        token_type: "Bearer".into(),
        expires_in: state.config.token_ttl_secs,
        scope: rec.scope,
    }))
}

#[derive(Debug, Serialize)]
pub struct PlatformUserInfo {
    pub sub: String,
    pub end_user_id: i64,
    pub username: String,
    pub email: Option<String>,
    pub status: String,
    pub source: String,
}

pub async fn oauth_userinfo(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let token = extract_bearer(&headers)
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "Authorization Bearer required"))?;
    let oauth = state
        .oauth
        .get_oauth_access(&token)
        .await
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "invalid access token"))?;
    let user = state
        .users
        .get_by_id(oauth.end_user_id)
        .await
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "user not found"))?;
    Ok(Json(PlatformUserInfo {
        sub: user.end_user_id.to_string(),
        end_user_id: user.end_user_id,
        username: user.username,
        email: user.email,
        status: user.status,
        source: "session_token_server".into(),
    }))
}

fn append_query(uri: &str, q: &str) -> String {
    if uri.contains('?') {
        format!("{uri}&{q}")
    } else {
        format!("{uri}?{q}")
    }
}

pub fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
