//! Corp Web wizards: register (BR), login, profile, game submit with documents.

use axum::extract::{Multipart, Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use tokio::fs;

use crate::api::{err_status, extract_bearer, AppState};
use crate::corp::{require_master, CorpIdentity};
use crate::models::ErrorBody;
use crate::uploads;

#[derive(Debug, Deserialize)]
pub struct CorpLoginRequest {
    /// contact_email or master_code
    pub login: String,
    pub password: String,
}

pub async fn require_corp_web(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<CorpIdentity, (StatusCode, Json<ErrorBody>)> {
    if let Some(token) = extract_bearer(headers) {
        if token.starts_with("corp_") {
            let id = state
                .corp_onboard
                .resolve_session(&token)
                .await
                .map_err(|e| err_status(StatusCode::UNAUTHORIZED, e))?;
            return Ok(CorpIdentity {
                corporate_user_id: id,
            });
        }
    }
    require_master(state, headers).await
}

pub async fn register(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let mut company_name = String::new();
    let mut contact_name = String::new();
    let mut contact_email = String::new();
    let mut password = String::new();
    let mut br_name = String::new();
    let mut br_ctype = "application/octet-stream".to_string();
    let mut br_bytes: Vec<u8> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "company_name" => {
                company_name = field
                    .text()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
            }
            "contact_name" => {
                contact_name = field
                    .text()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
            }
            "contact_email" => {
                contact_email = field
                    .text()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
            }
            "password" => {
                password = field
                    .text()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
            }
            "br_file" | "business_registration" => {
                br_name = field
                    .file_name()
                    .unwrap_or("br.bin")
                    .to_string();
                if let Some(ct) = field.content_type() {
                    br_ctype = ct.to_string();
                }
                br_bytes = field
                    .bytes()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
                    .to_vec();
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let result = state
        .corp_onboard
        .register(
            &company_name,
            &contact_name,
            &contact_email,
            &password,
            &br_name,
            &br_ctype,
            &br_bytes,
        )
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "profile": result.profile,
            "access_token": result.access_token,
            "token_type": "Bearer",
            "expires_at": result.expires_at,
            "api_key": result.api_key,
            "api_secret": result.api_secret,
            "message": "Registration submitted. Wait for admin approval of your Business Registration.",
            "source": "client_center",
        })),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<CorpLoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let (profile, token, expires_at) = state
        .corp_onboard
        .login(&req.login, &req.password)
        .await
        .map_err(|e| err_status(StatusCode::UNAUTHORIZED, e))?;
    Ok(Json(json!({
        "profile": profile,
        "access_token": token,
        "token_type": "Bearer",
        "expires_at": expires_at,
        "source": "client_center",
    })))
}

pub async fn profile(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = require_corp_web(&state, &headers).await?;
    let profile = state
        .corp_onboard
        .profile(ident.corporate_user_id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "profile not found"))?;
    Ok(Json(json!({ "profile": profile, "source": "client_center" })))
}

pub async fn list_games(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = require_corp_web(&state, &headers).await?;
    let games = state
        .corp_onboard
        .list_games(ident.corporate_user_id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({ "games": games, "source": "client_center" })))
}

pub async fn get_game(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = require_corp_web(&state, &headers).await?;
    let game = state
        .corp_onboard
        .get_game(ident.corporate_user_id, id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "game not found"))?;
    Ok(Json(game))
}

pub async fn submit_game(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let ident = require_corp_web(&state, &headers).await?;
    let mut game_name = String::new();
    let mut game_code = String::new();
    let mut partner_code = String::new();
    let mut partner_game_id = String::new();
    let mut description = String::new();
    let mut docs: Vec<(String, String, String, Vec<u8>)> = Vec::new();
    let mut pending_doc_type = String::from("other");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "game_name" => {
                game_name = field
                    .text()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
            }
            "game_code" => {
                game_code = field
                    .text()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
            }
            "partner_code" => {
                partner_code = field
                    .text()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
            }
            "partner_game_id" => {
                partner_game_id = field
                    .text()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
            }
            "description" => {
                description = field
                    .text()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
            }
            "doc_type" => {
                pending_doc_type = field
                    .text()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
                    .trim()
                    .to_string();
            }
            "document" | "documents" | "file" => {
                let fname = field.file_name().unwrap_or("doc.bin").to_string();
                let ctype = field
                    .content_type()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "application/octet-stream".into());
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
                    .to_vec();
                let dt = if pending_doc_type.is_empty() {
                    "other".into()
                } else {
                    pending_doc_type.clone()
                };
                docs.push((dt, fname, ctype, bytes));
            }
            "government_approval" => {
                let fname = field.file_name().unwrap_or("gov.bin").to_string();
                let ctype = field
                    .content_type()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "application/octet-stream".into());
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
                    .to_vec();
                docs.push((
                    "game_government_approval".into(),
                    fname,
                    ctype,
                    bytes,
                ));
            }
            "approved_material" => {
                let fname = field.file_name().unwrap_or("material.bin").to_string();
                let ctype = field
                    .content_type()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "application/octet-stream".into());
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?
                    .to_vec();
                docs.push((
                    "game_approved_material".into(),
                    fname,
                    ctype,
                    bytes,
                ));
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let game = state
        .corp_onboard
        .submit_game(
            ident.corporate_user_id,
            &game_name,
            &game_code,
            Some(partner_code.as_str()).filter(|s| !s.is_empty()),
            Some(partner_game_id.as_str()).filter(|s| !s.is_empty()),
            Some(description.as_str()).filter(|s| !s.is_empty()),
            docs,
        )
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "game": game,
            "message": "Game submitted. Wait for admin review of documents.",
            "source": "client_center",
        })),
    ))
}

pub async fn download_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Response, (StatusCode, Json<ErrorBody>)> {
    let ident = require_corp_web(&state, &headers).await?;
    let (doc, stored_path) = state
        .corp_onboard
        .get_document(id)
        .await
        .map_err(|e| err_status(StatusCode::BAD_REQUEST, e))?
        .ok_or_else(|| err_status(StatusCode::NOT_FOUND, "document not found"))?;
    if doc.corporate_user_id != ident.corporate_user_id {
        return Err(err_status(StatusCode::FORBIDDEN, "not your document"));
    }
    let path = uploads::resolve_path(state.corp_onboard.upload_dir(), &stored_path);
    let bytes = fs::read(&path)
        .await
        .map_err(|e| err_status(StatusCode::NOT_FOUND, format!("file missing: {e}")))?;
    Ok((
        [
            (
                header::CONTENT_TYPE,
                doc.content_type.parse().unwrap_or_else(|_| {
                    header::HeaderValue::from_static("application/octet-stream")
                }),
            ),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", doc.file_name)
                    .parse()
                    .unwrap_or_else(|_| header::HeaderValue::from_static("attachment")),
            ),
        ],
        bytes,
    )
        .into_response())
}
