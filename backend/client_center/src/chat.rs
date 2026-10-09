//! Session chat persistence APIs (Mongo `chat_messages`).

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::api::{err_status, AppState};
use crate::game_accounts::require_end_user;
use crate::models::ErrorBody;

#[derive(Debug, Deserialize)]
pub struct SendChatRequest {
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(default)]
    pub receiver: Option<String>,
    pub content: String,
}

fn default_channel() -> String {
    "public".into()
}

#[derive(Debug, Deserialize)]
pub struct ListChatQuery {
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn send_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SendChatRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    if !state.chat.is_ready() {
        return Err(err_status(
            StatusCode::SERVICE_UNAVAILABLE,
            "MONGO_URL required for chat_messages",
        ));
    }
    let end_user_id = require_end_user(&state, &headers).await?;
    let content = body.content.trim();
    if content.is_empty() {
        return Err(err_status(StatusCode::BAD_REQUEST, "content required"));
    }
    let channel = body.channel.trim();
    if channel.is_empty() {
        return Err(err_status(StatusCode::BAD_REQUEST, "channel required"));
    }
    let sender = format!("user_{end_user_id}");
    let receiver = body
        .receiver
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("")
        .to_string();
    let msg = state
        .chat
        .insert(end_user_id, channel, &sender, &receiver, content)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok((StatusCode::CREATED, Json(msg)))
}

pub async fn list_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListChatQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    if !state.chat.is_ready() {
        return Err(err_status(
            StatusCode::SERVICE_UNAVAILABLE,
            "MONGO_URL required for chat_messages",
        ));
    }
    let end_user_id = require_end_user(&state, &headers).await?;
    let rows = state
        .chat
        .list_for_client(end_user_id, q.channel.as_deref(), q.limit)
        .await
        .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({ "messages": rows })))
}
