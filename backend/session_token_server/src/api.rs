use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use funnyx_health::HealthPayload;
use funnyx_net_api::paths;
use reqwest::Client;
use serde::Deserialize;

use crate::config::Config;
use crate::models::{
    ErrorBody, IssueTokenRequest, IssueTokenResponse, RevokeTokenRequest, RevokeTokenResponse,
    SessionRecord, ValidateTokenRequest, ValidateTokenResponse,
};
use crate::oauth::oauth_routes;
use crate::oauth_store::OauthStore;
use crate::store::SessionStore;
use crate::token::{build_session, is_expired};
use crate::users::UserDirectory;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub store: SessionStore,
    pub oauth: OauthStore,
    pub users: UserDirectory,
    pub http: Client,
}

pub fn router(state: AppState) -> Router {
    let app = Router::new()
        .route(paths::HEALTH, get(health))
        .route(paths::SESSION_TOKEN, post(issue_token))
        .route(paths::SESSION_VALIDATE, post(validate_token))
        .route(paths::SESSION_REVOKE, post(revoke_token))
        .route(paths::CLIENT_LOGIN, post(client_login))
        .route(paths::CLIENT_REGISTER, post(client_register));
    oauth_routes(app).with_state(state)
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match HealthPayload::ok(state.config.service_name.clone()) {
        Ok(payload) => Json(payload).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct ClientLoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct ClientRegisterRequest {
    username: String,
    password: String,
    #[serde(default)]
    email: Option<String>,
}

async fn client_login(
    State(state): State<AppState>,
    Json(req): Json<ClientLoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let user = state
        .users
        .authenticate(&req.username, &req.password)
        .await
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "invalid username or password"))?;
    let session = issue_session_for_user(
        &state,
        user.end_user_id,
        Some(user.username.clone()),
        Some(user.username),
        "login".into(),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(to_issue_response(&state, session))))
}

async fn client_register(
    State(state): State<AppState>,
    Json(req): Json<ClientRegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let user = state
        .users
        .register(req.username, req.password, req.email)
        .await
        .map_err(bad_request)?;
    let session = issue_session_for_user(
        &state,
        user.end_user_id,
        Some(user.username.clone()),
        Some(user.username),
        "login".into(),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(to_issue_response(&state, session))))
}

async fn issue_token(
    State(state): State<AppState>,
    Json(req): Json<IssueTokenRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let grant = req.grant_type.to_ascii_lowercase();
    let actor_type = normalize_actor_type(&req.actor_type);
    let record = match grant.as_str() {
        "login" | "oauth" | "admin_login" => {
            let end_user_id = req
                .end_user_id
                .ok_or_else(|| bad_request("end_user_id required for login/oauth/admin_login grant"))?;
            if end_user_id <= 0 {
                return Err(bad_request("end_user_id must be > 0"));
            }
            let scope = if grant == "admin_login" || actor_type == "admin" {
                if req.scope.contains("admin") {
                    req.scope
                } else {
                    format!("admin,{}", req.scope.trim_matches(','))
                        .trim_matches(',')
                        .to_string()
                }
            } else {
                req.scope
            };
            let actor = if grant == "admin_login" {
                "admin".into()
            } else {
                actor_type
            };
            build_session(
                end_user_id,
                req.username,
                req.account_id,
                scope,
                grant,
                actor,
                state.config.token_ttl_secs,
                state.config.refresh_ttl_secs,
            )
        }
        "refresh" => {
            let refresh = req
                .refresh_token
                .as_deref()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| bad_request("refresh_token required for refresh grant"))?;
            let existing = state
                .store
                .get_by_refresh(refresh)
                .await
                .map_err(internal)?;
            let Some(existing) = existing else {
                return Err(err_status(
                    StatusCode::UNAUTHORIZED,
                    "invalid or expired refresh_token",
                ));
            };
            if is_expired(existing.refresh_expires_at_ms) {
                let _ = state.store.revoke_refresh(refresh).await;
                return Err(err_status(StatusCode::UNAUTHORIZED, "refresh_token expired"));
            }
            let _ = state.store.revoke_access(&existing.access_token).await;
            build_session(
                existing.end_user_id,
                existing.username,
                existing.account_id,
                existing.scope,
                "refresh".into(),
                existing.actor_type,
                state.config.token_ttl_secs,
                state.config.refresh_ttl_secs,
            )
        }
        other => {
            return Err(bad_request(format!(
                "unsupported grant_type: {other} (use login|oauth|admin_login|refresh)"
            )));
        }
    };

    state.store.put(record.clone()).await.map_err(internal)?;
    Ok((StatusCode::CREATED, Json(to_issue_response(&state, record))))
}

async fn validate_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ValidateTokenRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal(&headers, &state.config.internal_api_key)?;
    let token = extract_token(&headers, req.access_token.as_deref())
        .ok_or_else(|| bad_request("access_token or Authorization Bearer required"))?;

    match state.store.get_by_access(&token).await.map_err(internal)? {
        Some(rec) => Ok(Json(ValidateTokenResponse {
            valid: true,
            end_user_id: Some(rec.end_user_id),
            username: rec.username,
            account_id: rec.account_id,
            scope: Some(rec.scope),
            actor_type: Some(rec.actor_type),
            expires_at_ms: Some(rec.expires_at_ms),
            reason: None,
        })),
        None => Ok(Json(ValidateTokenResponse {
            valid: false,
            end_user_id: None,
            username: None,
            account_id: None,
            scope: None,
            actor_type: None,
            expires_at_ms: None,
            reason: Some("invalid_or_expired".into()),
        })),
    }
}

async fn revoke_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RevokeTokenRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let has_internal = header_eq(&headers, "x-internal-key", &state.config.internal_api_key);
    let access = extract_token(&headers, req.access_token.as_deref());
    let refresh = req.refresh_token.as_deref().filter(|s| !s.is_empty());

    if !has_internal && access.is_none() && refresh.is_none() {
        return Err(err_status(
            StatusCode::UNAUTHORIZED,
            "Bearer token, refresh_token, or X-Internal-Key required",
        ));
    }

    let mut revoked = false;
    if let Some(access) = access {
        revoked |= state.store.revoke_access(&access).await.map_err(internal)?;
    }
    if let Some(refresh) = refresh {
        revoked |= state.store.revoke_refresh(refresh).await.map_err(internal)?;
    }

    Ok(Json(RevokeTokenResponse {
        revoked,
        source: "session_token_server".into(),
    }))
}

pub async fn issue_session_for_user(
    state: &AppState,
    end_user_id: i64,
    username: Option<String>,
    account_id: Option<String>,
    grant_type: String,
) -> Result<SessionRecord, (StatusCode, Json<ErrorBody>)> {
    let record = build_session(
        end_user_id,
        username,
        account_id,
        "http,socket".into(),
        grant_type,
        "end_user".into(),
        state.config.token_ttl_secs,
        state.config.refresh_ttl_secs,
    );
    state.store.put(record.clone()).await.map_err(internal)?;
    Ok(record)
}

fn to_issue_response(state: &AppState, record: SessionRecord) -> IssueTokenResponse {
    IssueTokenResponse {
        access_token: record.access_token,
        refresh_token: record.refresh_token,
        token_type: "Bearer".into(),
        expires_in: state.config.token_ttl_secs,
        expires_at_ms: record.expires_at_ms,
        end_user_id: record.end_user_id,
        username: record.username,
        account_id: record.account_id,
        scope: record.scope,
        actor_type: record.actor_type,
        source: "session_token_server".into(),
    }
}

fn normalize_actor_type(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "admin" => "admin".into(),
        _ => "end_user".into(),
    }
}

pub fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    extract_token(headers, None)
}

fn extract_token(headers: &HeaderMap, body_token: Option<&str>) -> Option<String> {
    if let Some(t) = body_token.map(str::trim).filter(|s| !s.is_empty()) {
        return Some(t.to_string());
    }
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            let v = v.trim();
            v.strip_prefix("Bearer ").map(|rest| rest.trim().to_string())
        })
        .filter(|t| !t.is_empty())
}

fn require_internal(
    headers: &HeaderMap,
    expected: &str,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    if header_eq(headers, "x-internal-key", expected) {
        Ok(())
    } else {
        Err(err_status(StatusCode::UNAUTHORIZED, "X-Internal-Key required"))
    }
}

fn header_eq(headers: &HeaderMap, name: &str, expected: &str) -> bool {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|v| v == expected)
        .unwrap_or(false)
}

pub fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    err_status(StatusCode::BAD_REQUEST, msg)
}

pub fn err_status(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (
        status,
        Json(ErrorBody {
            error: msg.into(),
        }),
    )
}

fn internal(e: impl ToString) -> (StatusCode, Json<ErrorBody>) {
    err_status(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
