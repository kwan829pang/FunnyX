//! Password login / register / logout / profile — tokens from Session Token Server.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, extract_bearer, map_sts_err, AppState};
use crate::models::{ErrorBody, PartnerLinkView, ProfileResponse, SessionResponse};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

pub async fn client_login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let user = state
        .users
        .authenticate(&req.username, &req.password)
        .await
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "invalid username or password"))?;

    let session = state
        .sts
        .issue_session(
            "login",
            user.end_user_id,
            Some(&user.username),
            Some(&user.username),
        )
        .await
        .map_err(map_sts_err)?;

    crate::sessions::record_session(
        state.pool.as_ref(),
        crate::sessions::SessionWrite {
            session: &session,
            grant_type: "login",
            partner_id: None,
            partner_user_id: None,
            game_account_id: None,
            oauth_identity_id: None,
        },
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(to_session_response(session, "login")),
    ))
}

pub async fn client_register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let user = state
        .users
        .register(req.username, req.password, req.email)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;

    let session = state
        .sts
        .issue_session(
            "login",
            user.end_user_id,
            Some(&user.username),
            Some(&user.username),
        )
        .await
        .map_err(map_sts_err)?;

    crate::sessions::record_session(
        state.pool.as_ref(),
        crate::sessions::SessionWrite {
            session: &session,
            grant_type: "login",
            partner_id: None,
            partner_user_id: None,
            game_account_id: None,
            oauth_identity_id: None,
        },
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(to_session_response(session, "login")),
    ))
}

pub async fn client_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LogoutRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let access = req
        .access_token
        .filter(|s| !s.is_empty())
        .or_else(|| extract_bearer(&headers));
    let refresh = req.refresh_token.filter(|s| !s.is_empty());
    if access.is_none() && refresh.is_none() {
        return Err(err_status(
            StatusCode::UNAUTHORIZED,
            "access_token / Bearer or refresh_token required",
        ));
    }
    let body = state
        .sts
        .revoke(access.as_deref(), refresh.as_deref())
        .await
        .map_err(map_sts_err)?;
    crate::sessions::revoke_session(state.pool.as_ref(), access.as_deref(), refresh.as_deref())
        .await;
    Ok(Json(serde_json::json!({
        "revoked": body.get("revoked").and_then(|v| v.as_bool()).unwrap_or(true),
        "source": "client_center",
    })))
}

pub async fn client_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let token = extract_bearer(&headers).ok_or_else(|| {
        err_status(StatusCode::UNAUTHORIZED, "Authorization Bearer required")
    })?;
    let validated = state.sts.validate(&token).await.map_err(map_sts_err)?;
    if !validated.valid {
        return Err(err_status(
            StatusCode::UNAUTHORIZED,
            validated.reason.unwrap_or_else(|| "invalid_or_expired".into()),
        ));
    }
    let end_user_id = validated
        .end_user_id
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "session missing end_user_id"))?;

    let user = match state.users.get_by_id(end_user_id).await {
        Some(u) => u,
        None => {
            state
                .users
                .upsert_from_session(end_user_id, validated.username.clone(), None, None, None, None)
                .await
        }
    };
    let links = state.users.links_for_user(end_user_id).await;
    Ok(Json(ProfileResponse {
        end_user_id: user.end_user_id,
        username: user.username,
        email: user.email,
        status: user.status,
        partner_links: links
            .into_iter()
            .map(|l| PartnerLinkView {
                partner_id: l.partner_id,
                partner_user_id: l.partner_user_id,
                game_id: l.game_id,
                game_account_id: l.game_account_id,
            })
            .collect(),
        source: "client_center".into(),
    }))
}

fn to_session_response(s: crate::sts::StsSession, grant_type: &str) -> SessionResponse {
    SessionResponse {
        access_token: s.access_token,
        refresh_token: s.refresh_token,
        token_type: s.token_type,
        expires_in: s.expires_in,
        expires_at_ms: s.expires_at_ms,
        end_user_id: s.end_user_id,
        username: s.username,
        account_id: s.account_id,
        scope: s.scope,
        grant_type: grant_type.into(),
        source: "client_center".into(),
    }
}
