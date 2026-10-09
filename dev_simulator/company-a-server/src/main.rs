mod models;
mod sim_status;
mod state;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use chrono::Utc;
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use models::{
    BalanceQuery, ErrorBody, HealthResponse, ItemListQuery, ItemListResponse, MoneyRequest,
    PartnerEndpoint, PartnerSetupResponse, PlayerLookupQuery, PlayerLookupResponse, TokenResponse,
    TransferRequest,
};
use sim_status::{SimDirective, SimStatus, HEADER_NAME};
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let host = std::env::var("HTTP_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("HTTP_PORT")
        .unwrap_or_else(|_| "18102".into())
        .parse()?;
    let public_base_url =
        std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| format!("http://127.0.0.1:{port}"));
    let partner_id = std::env::var("PARTNER_ID").unwrap_or_else(|_| "partner_2001".into());
    let company_name =
        std::env::var("COMPANY_NAME").unwrap_or_else(|_| "Company A Game Partner".into());
    let api_key = std::env::var("API_KEY").unwrap_or_else(|_| "demo-api-key".into());
    let client_id = std::env::var("OAUTH_CLIENT_ID").unwrap_or_else(|_| "company_a_client".into());
    let client_secret =
        std::env::var("OAUTH_CLIENT_SECRET").unwrap_or_else(|_| "company_a_secret".into());

    let state = AppState::new(
        partner_id,
        company_name,
        public_base_url,
        api_key,
        client_id,
        client_secret,
    );

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/partner/setup", get(partner_setup))
        // OAuth 2.0 (partner as provider)
        .route("/oauth/authorize", get(oauth_authorize))
        .route("/oauth/login", get(oauth_login_page).post(oauth_login_submit))
        .route("/oauth/token", post(oauth_token))
        .route("/oauth/userinfo", get(oauth_userinfo))
        // Required partner APIs
        .route("/api/transfer", post(create_transfer))
        .route("/api/transfer/{transfer_id}", get(get_transfer))
        .route("/api/items", get(list_items))
        .route("/api/players/lookup", get(lookup_player))
        // Optional partner wallet APIs
        .route("/api/balance", get(get_balance))
        .route("/api/deposit", post(create_deposit))
        .route("/api/withdrawal", post(create_withdrawal))
        .route("/api/callback", post(partner_callback))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!(
        "company-a-server listening on http://{addr} (header {HEADER_NAME}='STATUS' or 'STATUS ms')"
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        service: "company-a-server".into(),
        partner_id: state.partner_id.clone(),
        timestamp_ms: Utc::now().timestamp_millis(),
    })
}

async fn partner_setup(State(state): State<AppState>) -> Json<PartnerSetupResponse> {
    let base = &state.public_base_url;
    Json(PartnerSetupResponse {
        partner_id: state.partner_id.clone(),
        company_name: state.company_name.clone(),
        server_name: "company-a-server".into(),
        endpoints: vec![
            PartnerEndpoint {
                endpoint_type: "oauth_authorize".into(),
                endpoint: format!("{base}/oauth/authorize"),
            },
            PartnerEndpoint {
                endpoint_type: "oauth_token".into(),
                endpoint: format!("{base}/oauth/token"),
            },
            PartnerEndpoint {
                endpoint_type: "oauth_userinfo".into(),
                endpoint: format!("{base}/oauth/userinfo"),
            },
            PartnerEndpoint {
                endpoint_type: "transfer".into(),
                endpoint: format!("{base}/api/transfer"),
            },
            PartnerEndpoint {
                endpoint_type: "item_list".into(),
                endpoint: format!("{base}/api/items"),
            },
            PartnerEndpoint {
                endpoint_type: "player_lookup".into(),
                endpoint: format!("{base}/api/players/lookup"),
            },
            PartnerEndpoint {
                endpoint_type: "balance".into(),
                endpoint: format!("{base}/api/balance"),
            },
            PartnerEndpoint {
                endpoint_type: "deposit".into(),
                endpoint: format!("{base}/api/deposit"),
            },
            PartnerEndpoint {
                endpoint_type: "withdrawal".into(),
                endpoint: format!("{base}/api/withdrawal"),
            },
            PartnerEndpoint {
                endpoint_type: "callback".into(),
                endpoint: format!("{base}/api/callback"),
            },
        ],
        auth_type: "signature".into(),
        api_key: state.api_key.clone(),
        status: "active".into(),
        source: "test".into(),
        created_at: Utc::now(),
    })
}

#[derive(Debug, Deserialize)]
struct AuthorizeQuery {
    response_type: Option<String>,
    client_id: Option<String>,
    redirect_uri: String,
    state: Option<String>,
    scope: Option<String>,
}

async fn oauth_authorize(
    State(state): State<AppState>,
    Query(q): Query<AuthorizeQuery>,
) -> Result<Redirect, (StatusCode, Json<ErrorBody>)> {
    if q.response_type.as_deref().unwrap_or("code") != "code" {
        return Err(err(StatusCode::BAD_REQUEST, "response_type must be code"));
    }
    if let Some(cid) = &q.client_id {
        if cid != &state.client_id {
            return Err(err(StatusCode::UNAUTHORIZED, "invalid client_id"));
        }
    }
    let mut params = format!(
        "redirect_uri={}",
        urlencoding_lite(&q.redirect_uri)
    );
    if let Some(s) = q.state {
        params.push_str(&format!("&state={}", urlencoding_lite(&s)));
    }
    if let Some(scope) = q.scope {
        params.push_str(&format!("&scope={}", urlencoding_lite(&scope)));
    }
    Ok(Redirect::to(&format!("/oauth/login?{params}")))
}

async fn oauth_login_page(Query(q): Query<HashMap<String, String>>) -> Html<String> {
    let redirect_uri = q.get("redirect_uri").cloned().unwrap_or_default();
    let state = q.get("state").cloned().unwrap_or_default();
    Html(format!(
        r#"<!doctype html>
<html><head><meta charset="utf-8"><title>Company A OAuth</title>
<style>body{{font-family:system-ui;max-width:28rem;margin:2rem auto;background:#102a43;color:#f0f4f8}}
input,button{{display:block;width:100%;margin:.5rem 0;padding:.6rem;font:inherit}}
.hint{{opacity:.8;font-size:.9rem}}</style></head><body>
<h1>Company A Login</h1>
<p class="hint">Demo users: <code>alice_01</code> / <code>bob_02</code> (password <code>demo</code>)</p>
<form method="post" action="/oauth/login">
<input type="hidden" name="redirect_uri" value="{redirect_uri}">
<input type="hidden" name="state" value="{state}">
<label>Username</label><input name="username" value="alice_01">
<label>Password</label><input name="password" type="password" value="demo">
<button type="submit">Authorize</button>
</form></body></html>"#
    ))
}

#[derive(Debug, Deserialize)]
struct LoginForm {
    username: String,
    password: String,
    redirect_uri: String,
    #[serde(default)]
    state: String,
}

async fn oauth_login_submit(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Result<Redirect, (StatusCode, Html<String>)> {
    let user = state
        .find_user_by_username(&form.username)
        .await
        .filter(|u| u.password == form.password)
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Html("<h1>Invalid credentials</h1>".into()),
            )
        })?;
    let code = state.issue_auth_code(&user.partner_user_id).await;
    let mut loc = format!(
        "{}{}code={}",
        form.redirect_uri,
        if form.redirect_uri.contains('?') {
            "&"
        } else {
            "?"
        },
        code
    );
    if !form.state.is_empty() {
        loc.push_str(&format!("&state={}", urlencoding_lite(&form.state)));
    }
    Ok(Redirect::to(&loc))
}

#[derive(Debug, Deserialize)]
struct TokenForm {
    grant_type: Option<String>,
    code: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    redirect_uri: Option<String>,
}

async fn oauth_token(
    State(state): State<AppState>,
    Form(form): Form<TokenForm>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<ErrorBody>)> {
    if form.grant_type.as_deref().unwrap_or("authorization_code") != "authorization_code" {
        return Err(err(StatusCode::BAD_REQUEST, "unsupported grant_type"));
    }
    if form.client_id.as_deref() != Some(state.client_id.as_str())
        || form.client_secret.as_deref() != Some(state.client_secret.as_str())
    {
        return Err(err(StatusCode::UNAUTHORIZED, "invalid client credentials"));
    }
    let code = form
        .code
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "code required"))?;
    let (access_token, _) = state
        .exchange_code(&code)
        .await
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "invalid or expired code"))?;
    let _ = form.redirect_uri;
    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer".into(),
        expires_in: 3600,
        refresh_token: None,
        scope: "openid profile game".into(),
    }))
}

async fn oauth_userinfo(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Authorization Bearer required"))?;
    let token = auth
        .strip_prefix("Bearer ")
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Bearer token required"))?;
    let info = state
        .userinfo_from_token(token)
        .await
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "invalid access token"))?;
    Ok(Json(info))
}

async fn create_transfer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<TransferRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_api_key(&headers, &state)?;
    let directive = SimDirective::from_headers(&headers)?;
    if req.partner_id != state.partner_id {
        return Err(err(StatusCode::BAD_REQUEST, "partner_id mismatch"));
    }
    if !matches!(
        req.asset_type.as_str(),
        "game_item" | "game_coin" | "company_coin"
    ) {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "asset_type must be game_item, game_coin, or company_coin",
        ));
    }
    let record = state.create_transfer(req).await;
    let (sim_status, sim_delay_ms, header_echo) =
        schedule_transfer(&state, &record.transfer_id, directive);
    Ok((
        StatusCode::CREATED,
        [(HEADER_NAME, header_echo)],
        Json(AppState::transfer_response(&record, &sim_status, sim_delay_ms)),
    ))
}

async fn get_transfer(
    State(state): State<AppState>,
    Path(transfer_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let record = state
        .get_transfer(&transfer_id)
        .await
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "transfer not found"))?;
    Ok(Json(record.to_view()))
}

async fn list_items(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ItemListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_api_key(&headers, &state)?;
    let items = state.list_items(&q.game_account_id).await;
    Ok(Json(ItemListResponse {
        game_account_id: q.game_account_id,
        items,
        source: "test".into(),
    }))
}

/// Verify a partner player is active on `game_id` (used by Client Center bind Path A).
async fn lookup_player(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<PlayerLookupQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_api_key(&headers, &state)?;
    if q.game_id.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "game_id required"));
    }
    let user = state
        .lookup_player(
            &q.game_id,
            q.username.as_deref(),
            q.partner_user_id.as_deref(),
            q.platform_end_user_id,
        )
        .await
        .ok_or_else(|| {
            err(
                StatusCode::NOT_FOUND,
                "player not found for game_id (need username or partner_user_id)",
            )
        })?;
    Ok(Json(PlayerLookupResponse {
        partner_user_id: user.partner_user_id,
        game_id: user.game_id,
        game_account_id: user.game_account_id,
        username: user.username,
        status: "active".into(),
        playing: true,
        source: "test".into(),
    }))
}

async fn get_balance(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<BalanceQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_api_key(&headers, &state)?;
    let coin = q.game_coin.unwrap_or_else(|| "GCA".into());
    Ok(Json(state.balance(&q.game_account_id, &coin).await))
}

async fn create_deposit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut req): Json<MoneyRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    req.transaction_type = "deposit".into();
    create_money(state, headers, req).await
}

async fn create_withdrawal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut req): Json<MoneyRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    req.transaction_type = "withdrawal".into();
    create_money(state, headers, req).await
}

async fn create_money(
    state: AppState,
    headers: HeaderMap,
    req: MoneyRequest,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_api_key(&headers, &state)?;
    let directive = SimDirective::from_headers(&headers)?;
    if req.amount <= 0.0 {
        return Err(err(StatusCode::BAD_REQUEST, "amount must be > 0"));
    }
    let record = state.create_money(req).await;
    let (sim_status, sim_delay_ms, header_echo) =
        schedule_money(&state, &record.partner_txn_id, directive);
    Ok((
        StatusCode::CREATED,
        [(HEADER_NAME, header_echo)],
        Json(AppState::money_response(&record, &sim_status, sim_delay_ms)),
    ))
}

async fn partner_callback(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    tracing::info!(payload = %body, "received partner callback");
    (StatusCode::OK, Json(serde_json::json!({ "ok": true, "source": "test" })))
}

fn schedule_transfer(
    state: &AppState,
    transfer_id: &str,
    directive: Option<SimDirective>,
) -> (String, u64, String) {
    let Some(directive) = directive.filter(|d| d.schedules_change()) else {
        return (
            SimStatus::Pending.as_str().into(),
            0,
            format!("{} 0", SimStatus::Pending.as_str()),
        );
    };
    let state = state.clone();
    let id = transfer_id.to_string();
    let delay_ms = directive.delay_ms;
    let target = directive.status;
    tokio::spawn(async move {
        if delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
        match state.apply_transfer_status(&id, target).await {
            Ok(r) => tracing::info!(transfer_id = %id, status = %r.status, "transfer finalized"),
            Err(e) => tracing::warn!(transfer_id = %id, error = %e, "transfer schedule failed"),
        }
    });
    (target.as_str().into(), delay_ms, directive.header_value())
}

fn schedule_money(
    state: &AppState,
    partner_txn_id: &str,
    directive: Option<SimDirective>,
) -> (String, u64, String) {
    let Some(directive) = directive.filter(|d| d.schedules_change()) else {
        return (
            SimStatus::Pending.as_str().into(),
            0,
            format!("{} 0", SimStatus::Pending.as_str()),
        );
    };
    let state = state.clone();
    let id = partner_txn_id.to_string();
    let delay_ms = directive.delay_ms;
    let target = directive.status;
    tokio::spawn(async move {
        if delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
        match state.apply_money_status(&id, target).await {
            Ok(r) => tracing::info!(partner_txn_id = %id, status = %r.status, "money txn finalized"),
            Err(e) => tracing::warn!(partner_txn_id = %id, error = %e, "money schedule failed"),
        }
    });
    (target.as_str().into(), delay_ms, directive.header_value())
}

fn require_api_key(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    let key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("ApiKey "))
        });
    match key {
        Some(k) if k == state.api_key => Ok(()),
        _ => Err(err(
            StatusCode::UNAUTHORIZED,
            "X-Api-Key (or Authorization: ApiKey …) required",
        )),
    }
}

fn urlencoding_lite(s: &str) -> String {
    s.replace('%', "%25")
        .replace(' ', "%20")
        .replace('&', "%26")
        .replace('?', "%3F")
        .replace('=', "%3D")
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.into() }))
}
