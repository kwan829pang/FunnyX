//! Game catalog + game-account bind (Path A partner fetch / Path B direct).

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, extract_bearer, map_sts_err, AppState};
use crate::bindings::GameAccountBinding;
use crate::games::GameRecord;
use crate::models::ErrorBody;

#[derive(Debug, Deserialize)]
pub struct BindGameAccountRequest {
    /// Platform `fx_game.games.id` (preferred).
    #[serde(default)]
    pub game_id: Option<i64>,
    /// Or platform `game_code` (e.g. DEMO_GAME).
    #[serde(default)]
    pub game_code: Option<String>,
    /// `partner_fetch` = Path A (call Partner API). `direct` = Path B / Game App.
    pub mode: String,
    /// Path A: partner login name to verify on Partner (e.g. alice_01).
    #[serde(default)]
    pub partner_username: Option<String>,
    /// Path A/B: partner user id when known.
    #[serde(default)]
    pub partner_user_id: Option<String>,
    /// Path B: game account id from Game App / OAuth userinfo.
    #[serde(default)]
    pub game_account_id: Option<String>,
}

pub async fn list_games(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.games.list_active().await)
}

pub async fn list_game_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    let rows = state.bindings.list_for_user(end_user_id).await;
    Ok(Json(rows))
}

pub async fn bind_game_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<BindGameAccountRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let end_user_id = require_end_user(&state, &headers).await?;
    let game = resolve_game(&state, req.game_id, req.game_code.as_deref()).await?;
    if game.status != "active" {
        return Err(err_status(StatusCode::BAD_REQUEST, "game is not active"));
    }

    let mode = req.mode.to_ascii_lowercase();
    let binding = match mode.as_str() {
        "partner_fetch" | "platform" | "a" => {
            bind_via_partner_fetch(&state, end_user_id, &game, &req).await?
        }
        "direct" | "game_app" | "b" => bind_direct(&state, end_user_id, &game, &req).await?,
        other => {
            return Err(err_status(
                StatusCode::BAD_REQUEST,
                format!("unsupported mode: {other} (use partner_fetch|direct)"),
            ))
        }
    };

    Ok((StatusCode::CREATED, Json(binding)))
}

async fn bind_via_partner_fetch(
    state: &AppState,
    end_user_id: i64,
    game: &GameRecord,
    req: &BindGameAccountRequest,
) -> Result<GameAccountBinding, (StatusCode, Json<ErrorBody>)> {
    if req.partner_username.as_deref().unwrap_or("").is_empty()
        && req.partner_user_id.as_deref().unwrap_or("").is_empty()
    {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            "partner_username or partner_user_id required for partner_fetch",
        ));
    }
    let player = state
        .partner
        .lookup_player(
            &game.partner_game_id,
            end_user_id,
            req.partner_username.as_deref(),
            req.partner_user_id.as_deref(),
        )
        .await
        .map_err(map_sts_err)?;
    if !player.playing {
        return Err(err_status(
            StatusCode::BAD_REQUEST,
            "partner reports user is not playing this game",
        ));
    }
    state
        .bindings
        .upsert(
            end_user_id,
            game.game_id,
            game.game_code.clone(),
            player.game_account_id,
            game.partner_id.clone(),
            Some(player.partner_user_id),
            "partner_fetch".into(),
        )
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))
}

async fn bind_direct(
    state: &AppState,
    end_user_id: i64,
    game: &GameRecord,
    req: &BindGameAccountRequest,
) -> Result<GameAccountBinding, (StatusCode, Json<ErrorBody>)> {
    let game_account_id = req
        .game_account_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| err_status(StatusCode::BAD_REQUEST, "game_account_id required for direct"))?
        .to_string();
    state
        .bindings
        .upsert(
            end_user_id,
            game.game_id,
            game.game_code.clone(),
            game_account_id,
            game.partner_id.clone(),
            req.partner_user_id.clone(),
            "direct".into(),
        )
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))
}

/// Used by Partner OAuth complete (Path B) to create mapping without a second round-trip.
pub async fn bind_direct_from_oauth(
    state: &AppState,
    end_user_id: i64,
    partner_game_id: Option<&str>,
    game_account_id: Option<&str>,
    partner_user_id: Option<&str>,
    partner_id: Option<&str>,
) -> Option<GameAccountBinding> {
    let game_account_id = game_account_id.filter(|s| !s.is_empty())?;
    let game = if let Some(gid) = partner_game_id.filter(|s| !s.is_empty()) {
        state.games.get_by_partner_game_id(gid).await
    } else {
        None
    }
    .or(match partner_id.filter(|s| !s.is_empty()) {
        Some(pid) => state.games.get_by_partner_code(pid).await,
        None => None,
    })?;
    state
        .bindings
        .upsert(
            end_user_id,
            game.game_id,
            game.game_code,
            game_account_id.to_string(),
            game.partner_id,
            partner_user_id.map(|s| s.to_string()),
            "direct".into(),
        )
        .await
        .ok()
}

async fn resolve_game(
    state: &AppState,
    game_id: Option<i64>,
    game_code: Option<&str>,
) -> Result<GameRecord, (StatusCode, Json<ErrorBody>)> {
    if let Some(id) = game_id {
        return state
            .games
            .get(id)
            .await
            .ok_or_else(|| err_status(StatusCode::NOT_FOUND, format!("unknown game_id: {id}")));
    }
    if let Some(code) = game_code.filter(|s| !s.is_empty()) {
        return state.games.get_by_code(code).await.ok_or_else(|| {
            err_status(StatusCode::NOT_FOUND, format!("unknown game_code: {code}"))
        });
    }
    Err(err_status(
        StatusCode::BAD_REQUEST,
        "game_id or game_code required (game must exist on platform)",
    ))
}

pub(crate) async fn require_end_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<i64, (StatusCode, Json<ErrorBody>)> {
    let token = extract_bearer(headers).ok_or_else(|| {
        err_status(StatusCode::UNAUTHORIZED, "Authorization Bearer required")
    })?;
    let validated = state.sts.validate(&token).await.map_err(map_sts_err)?;
    if !validated.valid {
        return Err(err_status(
            StatusCode::UNAUTHORIZED,
            validated
                .reason
                .unwrap_or_else(|| "invalid_or_expired".into()),
        ));
    }
    validated
        .end_user_id
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "session missing end_user_id"))
}
