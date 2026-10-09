//! OAuth entry on Client Center — Platform IdP + Partner broker live on Session Token Server.

use axum::body::Body;
use axum::extract::{Path, Query, Request, State};
use axum::http::{header, Method, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::api::{err_status, map_sts_err, AppState};
use crate::models::{ErrorBody, SessionResponse};

#[derive(Debug, Deserialize)]
pub struct PartnerStartQuery {
    /// Final Client Web return URL after CC completes OAuth (optional).
    #[serde(default)]
    pub redirect_uri: Option<String>,
    /// When `true`, skip CC complete hop and return STS JSON (API smoke).
    #[serde(default)]
    pub json: Option<bool>,
}

/// Redirect browser into STS Partner OAuth broker.
/// Default: STS returns to CC `/complete`, which upserts the user then sends Client Web the session.
pub async fn partner_oauth_start(
    State(state): State<AppState>,
    Path(partner_id): Path<String>,
    Query(q): Query<PartnerStartQuery>,
) -> Result<Redirect, (StatusCode, Json<ErrorBody>)> {
    let want_json = q.json.unwrap_or(false);
    let mut url = format!(
        "{}/v1/client/oauth/partner/{}/start",
        state.sts.base_url(),
        urlencoding(&partner_id)
    );

    if want_json {
        // Omit redirect_uri so STS callback returns JSON directly.
        return Ok(Redirect::to(&url));
    }

    // Route STS session back through Client Center complete.
    let mut complete = state.config.partner_oauth_complete_url();
    let next = q
        .redirect_uri
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| state.config.client_web_redirect.clone());
    if !next.is_empty() {
        complete.push_str(&format!("?next={}", urlencoding(&next)));
    }
    url.push_str(&format!("?redirect_uri={}", urlencoding(&complete)));
    Ok(Redirect::to(&url))
}

#[derive(Debug, Deserialize)]
pub struct PartnerCompleteQuery {
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub end_user_id: Option<i64>,
    #[serde(default)]
    pub partner_id: Option<String>,
    #[serde(default)]
    pub partner_user_id: Option<String>,
    #[serde(default)]
    pub game_account_id: Option<String>,
    #[serde(default)]
    pub game_id: Option<String>,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

/// STS Partner callback redirects here with session tokens; CC records the user.
pub async fn partner_oauth_complete(
    State(state): State<AppState>,
    Query(q): Query<PartnerCompleteQuery>,
) -> Result<Response, (StatusCode, Json<ErrorBody>)> {
    if let Some(err) = q.error {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            format!("partner oauth error: {err}"),
        ));
    }
    let access = q
        .access_token
        .filter(|s| !s.is_empty())
        .ok_or_else(|| err_status(StatusCode::BAD_REQUEST, "access_token required"))?;
    let refresh = q.refresh_token.unwrap_or_default();

    let validated = state.sts.validate(&access).await.map_err(map_sts_err)?;
    if !validated.valid {
        return Err(err_status(
            StatusCode::UNAUTHORIZED,
            validated.reason.unwrap_or_else(|| "invalid session from sts".into()),
        ));
    }
    let end_user_id = q
        .end_user_id
        .or(validated.end_user_id)
        .ok_or_else(|| err_status(StatusCode::BAD_REQUEST, "end_user_id missing"))?;

    let platform_game_id = match q.game_id.as_deref() {
        Some(s) if s.parse::<i64>().is_ok() => s.parse::<i64>().ok(),
        Some(_) => state
            .games
            .get_by_partner_game_id(q.game_id.as_deref().unwrap_or(""))
            .await
            .map(|g| g.game_id),
        None => None,
    };

    let user = state
        .users
        .upsert_from_session(
            end_user_id,
            validated.username.clone().or(q.partner_user_id.clone()),
            q.partner_id.clone(),
            q.partner_user_id.clone(),
            q.game_account_id.clone(),
            platform_game_id,
        )
        .await;

    // Path B: Game App OAuth → direct mapping row (game must exist on platform catalog).
    let _binding = crate::game_accounts::bind_direct_from_oauth(
        &state,
        user.end_user_id,
        q.game_id.as_deref(),
        q.game_account_id.as_deref(),
        q.partner_user_id.as_deref(),
        q.partner_id.as_deref(),
    )
    .await;

    let username = user.username.clone();
    let payload = SessionResponse {
        access_token: access.clone(),
        refresh_token: refresh.clone(),
        token_type: "Bearer".into(),
        expires_in: validated
            .expires_at_ms
            .map(|exp| ((exp - now_ms()).max(0) / 1000) as u64)
            .unwrap_or(3600),
        expires_at_ms: validated.expires_at_ms.unwrap_or(0),
        end_user_id: user.end_user_id,
        username: Some(username.clone()),
        account_id: Some(username.clone()),
        scope: validated.scope.unwrap_or_else(|| "http,socket".into()),
        grant_type: "oauth".into(),
        source: "client_center".into(),
    };

    let oauth_identity_id = match (q.partner_id.as_deref(), q.partner_user_id.as_deref()) {
        (Some(pid), Some(puid)) => state.users.oauth_identity_id(pid, puid).await,
        _ => None,
    };
    let sts_session = crate::sts::StsSession {
        access_token: access.clone(),
        refresh_token: refresh.clone(),
        token_type: "Bearer".into(),
        expires_in: payload.expires_in,
        expires_at_ms: payload.expires_at_ms,
        end_user_id: user.end_user_id,
        username: Some(username.clone()),
        account_id: Some(username),
        scope: payload.scope.clone(),
        actor_type: Some("end_user".into()),
    };
    crate::sessions::record_session(
        state.pool.as_ref(),
        crate::sessions::SessionWrite {
            session: &sts_session,
            grant_type: "oauth",
            partner_id: q.partner_id.as_deref(),
            partner_user_id: q.partner_user_id.as_deref(),
            game_account_id: q.game_account_id.as_deref(),
            oauth_identity_id,
        },
    )
    .await;

    if let Some(next) = q.next.filter(|s| !s.is_empty()) {
        let loc = format!(
            "{}{}access_token={}&refresh_token={}&end_user_id={}&token_type=Bearer",
            next,
            if next.contains('?') { "&" } else { "?" },
            urlencoding(&payload.access_token),
            urlencoding(&payload.refresh_token),
            payload.end_user_id,
        );
        return Ok(Redirect::to(&loc).into_response());
    }

    Ok((StatusCode::CREATED, Json(payload)).into_response())
}

/// Alias: Partners / docs may call `/callback` on Client Center; redirect into STS callback
/// only when STS already issued — prefer `/complete` after STS broker.
pub async fn partner_oauth_callback_alias(
    State(state): State<AppState>,
    Query(q): Query<serde_json::Map<String, serde_json::Value>>,
) -> Redirect {
    // If STS already finished and bounced here with tokens, treat as complete.
    if q.contains_key("access_token") {
        let mut loc = format!(
            "{}/v1/client/oauth/partner/complete?",
            state.config.public_base_url
        );
        let mut first = true;
        for (k, v) in q {
            if !first {
                loc.push('&');
            }
            first = false;
            let val = match v {
                serde_json::Value::String(s) => s,
                other => other.to_string(),
            };
            loc.push_str(&format!("{}={}", urlencoding(&k), urlencoding(&val)));
        }
        return Redirect::to(&loc);
    }
    // Otherwise forward authorize code exchange to STS callback.
    let mut loc = format!(
        "{}/v1/client/oauth/partner/callback?",
        state.sts.base_url()
    );
    let mut first = true;
    for (k, v) in q {
        if !first {
            loc.push('&');
        }
        first = false;
        let val = match v {
            serde_json::Value::String(s) => s,
            other => other.to_string(),
        };
        loc.push_str(&format!("{}={}", urlencoding(&k), urlencoding(&val)));
    }
    Redirect::to(&loc)
}

/// Platform OAuth authorize → STS IdP.
pub async fn platform_authorize_redirect(
    State(state): State<AppState>,
    req: Request,
) -> Redirect {
    let q = req
        .uri()
        .query()
        .map(|q| format!("?{q}"))
        .unwrap_or_default();
    Redirect::to(&format!(
        "{}/v1/oauth/authorize{q}",
        state.sts.base_url()
    ))
}

/// Proxy STS OAuth login page / submit, token, userinfo.
pub async fn proxy_sts_oauth(
    State(state): State<AppState>,
    req: Request,
) -> Result<Response, (StatusCode, Json<ErrorBody>)> {
    let method = match *req.method() {
        Method::GET => reqwest::Method::GET,
        Method::POST => reqwest::Method::POST,
        _ => {
            return Err(err_status(
                StatusCode::METHOD_NOT_ALLOWED,
                "method not allowed",
            ))
        }
    };
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    let headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), 1024 * 1024)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?;
    let body = if body.is_empty() {
        None
    } else {
        Some(body.to_vec())
    };

    let upstream = state
        .sts
        .forward(method, &path_and_query, &headers, body)
        .await
        .map_err(map_sts_err)?;

    let status = StatusCode::from_u16(upstream.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let mut builder = Response::builder().status(status);
    if let Some(ct) = upstream.headers().get(reqwest::header::CONTENT_TYPE) {
        if let Ok(v) = ct.to_str() {
            builder = builder.header(header::CONTENT_TYPE, v);
        }
    }
    if let Some(loc) = upstream.headers().get(reqwest::header::LOCATION) {
        if let Ok(v) = loc.to_str() {
            // Keep relative STS login redirects working when proxied via CC.
            let rewritten = if v.starts_with('/') {
                format!("{}{v}", state.config.public_base_url)
            } else if v.contains("/v1/oauth/") {
                v.replacen(
                    state.sts.base_url(),
                    &state.config.public_base_url,
                    1,
                )
            } else {
                v.to_string()
            };
            builder = builder.header(header::LOCATION, rewritten);
        }
    }
    let bytes = upstream
        .bytes()
        .await
        .map_err(|e| err_status(StatusCode::BAD_GATEWAY, e.to_string()))?;
    builder
        .body(Body::from(bytes))
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn oauth_info_json(state: &AppState) -> serde_json::Value {
    json!({
        "platform_authorize": format!("{}/v1/oauth/authorize", state.config.public_base_url),
        "partner_start": format!(
            "{}/v1/client/oauth/partner/{{partner_id}}/start",
            state.config.public_base_url
        ),
        "sts": state.sts.base_url(),
        "default_partner_id": state.config.default_partner_id,
    })
}
