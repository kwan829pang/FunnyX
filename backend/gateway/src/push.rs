//! Internal HTTP push + Session WebSocket for Client Web.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::api::AppState;
use crate::push_hub::PushEnvelope;
use crate::sts::ErrorBody;

fn require_internal(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorBody>)> {
    let key = headers
        .get("x-internal-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if key != state.config.internal_api_key {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorBody {
                error: "invalid internal key".into(),
            }),
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct PushBody {
    pub end_user_id: i64,
    pub notification_id: i64,
    #[serde(default)]
    pub source_type: String,
    #[serde(default)]
    pub event_type: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

pub async fn internal_socket_push(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PushBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    require_internal(&state, &headers)?;
    if body.end_user_id <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                error: "end_user_id required".into(),
            }),
        ));
    }
    let env = PushEnvelope {
        r#type: "notification",
        notification_id: body.notification_id,
        end_user_id: body.end_user_id,
        source_type: body.source_type,
        event_type: body.event_type,
        title: body.title,
        body: body.body,
        payload: body.payload,
    };
    let delivered = state.push.push_envelope(&env).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody { error: e }),
        )
    })?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "delivered": delivered,
        "source": "gateway",
    })))
}

pub async fn ws_session_upgrade(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_session(socket, state))
}

async fn handle_ws_session(socket: WebSocket, state: AppState) {
    let (mut sink, mut stream) = socket.split();
    let Some(Ok(Message::Text(text))) = stream.next().await else {
        let _ = sink
            .send(Message::Text(
                r#"{"ok":false,"error":"auth required"}"#.into(),
            ))
            .await;
        return;
    };
    let token = match parse_ws_auth(text.trim()) {
        Ok(t) => t,
        Err(e) => {
            let _ = sink
                .send(Message::Text(
                    format!(r#"{{"ok":false,"error":"{}"}}"#, e.replace('"', "'")).into(),
                ))
                .await;
            return;
        }
    };
    let v = match state.sts.validate(&token).await {
        Ok(v) => v,
        Err((_, e)) => {
            let _ = sink
                .send(Message::Text(
                    format!(
                        r#"{{"ok":false,"error":"{}"}}"#,
                        e.error.replace('"', "'")
                    )
                    .into(),
                ))
                .await;
            return;
        }
    };
    if !v.valid {
        let reason = v.reason.unwrap_or_else(|| "invalid session".into());
        let _ = sink
            .send(Message::Text(
                format!(r#"{{"ok":false,"error":"{}"}}"#, reason.replace('"', "'")).into(),
            ))
            .await;
        return;
    }
    let end_user_id = v.end_user_id.unwrap_or(0);
    if end_user_id <= 0 {
        let _ = sink
            .send(Message::Text(
                r#"{"ok":false,"error":"session missing end_user_id"}"#.into(),
            ))
            .await;
        return;
    }
    let connection_id = Uuid::new_v4().to_string();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    state
        .push
        .register(connection_id.clone(), end_user_id, tx)
        .await;
    {
        let mut map = state.memory.write().await;
        map.client_sockets.insert(
            connection_id.clone(),
            crate::memory::ClientSocketConn {
                connection_id: connection_id.clone(),
                end_user_id,
                connected_at_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0),
            },
        );
    }
    let _ = sink
        .send(Message::Text(
            format!(
                r#"{{"ok":true,"connection_id":"{connection_id}","end_user_id":{end_user_id}}}"#
            )
            .into(),
        ))
        .await;

    loop {
        tokio::select! {
            incoming = stream.next() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(p))) => {
                        let _ = sink.send(Message::Pong(p)).await;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
            msg = rx.recv() => {
                match msg {
                    Some(json) => {
                        if sink.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }
    state.push.unregister(&connection_id).await;
    let mut map = state.memory.write().await;
    map.client_sockets.remove(&connection_id);
}

fn parse_ws_auth(text: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    struct Auth {
        access_token: String,
    }
    let a: Auth = serde_json::from_str(text).map_err(|e| format!("auth json: {e}"))?;
    if a.access_token.trim().is_empty() {
        return Err("empty access_token".into());
    }
    Ok(a.access_token)
}
