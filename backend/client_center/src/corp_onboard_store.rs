//! Corp account + game onboarding (BR / government docs, pending approval).

use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db::now_ms;
use crate::password::{hash_password, verify_password};
use crate::query::corp_onboard as q;
use crate::uploads;

const SESSION_TTL_MS: i64 = 7 * 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, Serialize)]
pub struct CorpProfile {
    pub id: i64,
    pub company_name: String,
    pub master_code: String,
    pub master_id: String,
    pub status: String,
    pub api_enabled: bool,
    pub contact_email: Option<String>,
    pub contact_name: Option<String>,
    pub reject_reason: Option<String>,
    pub approved_at: i64,
    pub created_at: i64,
    pub documents: Vec<CorpDocumentView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorpDocumentView {
    pub id: i64,
    pub corporate_user_id: i64,
    pub game_id: Option<i64>,
    pub doc_type: String,
    pub file_name: String,
    pub content_type: String,
    pub byte_size: i64,
    pub status: String,
    pub review_note: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorpGameView {
    pub id: i64,
    pub corporate_user_id: i64,
    pub game_name: String,
    pub game_code: String,
    pub partner_code: String,
    pub partner_game_id: String,
    pub status: String,
    pub description: String,
    pub reject_reason: String,
    pub approved_at: i64,
    pub created_at: i64,
    pub documents: Vec<CorpDocumentView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisterResult {
    pub profile: CorpProfile,
    pub access_token: String,
    pub expires_at: i64,
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Clone)]
pub struct CorpOnboardStore {
    pool: Option<PgPool>,
    upload_dir: String,
}

impl CorpOnboardStore {
    pub fn new(pool: Option<PgPool>, upload_dir: String) -> Self {
        Self { pool, upload_dir }
    }

    fn pool(&self) -> Result<&PgPool, String> {
        self.pool
            .as_ref()
            .ok_or_else(|| "POSTGRES_URL required for corp onboarding".into())
    }

    pub async fn register(
        &self,
        company_name: &str,
        contact_name: &str,
        contact_email: &str,
        password: &str,
        br_file_name: &str,
        br_content_type: &str,
        br_bytes: &[u8],
    ) -> Result<RegisterResult, String> {
        let pool = self.pool()?;
        if company_name.trim().is_empty() || contact_email.trim().is_empty() || password.len() < 6 {
            return Err("company_name, contact_email, and password (min 6) required".into());
        }
        if br_bytes.is_empty() {
            return Err("business registration (BR) file required".into());
        }
        let exists: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM fx_corp.corporate_users WHERE lower(contact_email) = lower($1)",
        )
        .bind(contact_email.trim())
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        if exists.is_some() {
            return Err("contact_email already registered".into());
        }

        let pw_hash = hash_password(password)?;
        let master_code = format!("MC_{}", Uuid::new_v4().simple());
        let master_id = format!("MID_{}", Uuid::new_v4().simple());
        let api_key = format!("ak_{}", Uuid::new_v4().simple());
        let api_secret = format!("sk_{}", Uuid::new_v4().simple());
        let secret_hash = hash_password(&api_secret)?;
        let now = now_ms();

        let (stored_path, safe_name) = uploads::save_upload(
            &self.upload_dir,
            "br",
            br_file_name,
            br_bytes,
        )
        .await?;

        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
        let corp_id: i64 = sqlx::query_scalar(q::INSERT_CORP)
            .bind(company_name.trim())
            .bind(&master_code)
            .bind(&master_id)
            .bind(contact_email.trim())
            .bind(contact_name.trim())
            .bind(&pw_hash)
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query(q::INSERT_API_KEY)
            .bind(corp_id)
            .bind(&api_key)
            .bind(&secret_hash)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query_scalar::<_, i64>(q::INSERT_DOCUMENT)
            .bind(corp_id)
            .bind(None::<i64>)
            .bind("business_registration")
            .bind(&safe_name)
            .bind(&stored_path)
            .bind(br_content_type)
            .bind(br_bytes.len() as i64)
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        let token = format!("corp_{}", Uuid::new_v4().simple());
        let expires_at = now + SESSION_TTL_MS;
        sqlx::query(q::INSERT_SESSION)
            .bind(corp_id)
            .bind(&token)
            .bind(expires_at)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        tx.commit().await.map_err(|e| e.to_string())?;

        let profile = self
            .profile(corp_id)
            .await?
            .ok_or_else(|| "profile missing after register".to_string())?;
        Ok(RegisterResult {
            profile,
            access_token: token,
            expires_at,
            api_key,
            api_secret,
        })
    }

    pub async fn login(
        &self,
        login: &str,
        password: &str,
    ) -> Result<(CorpProfile, String, i64), String> {
        let pool = self.pool()?;
        let row = if login.contains('@') {
            sqlx::query(q::BY_EMAIL).bind(login.trim()).fetch_optional(pool).await
        } else {
            sqlx::query(q::BY_MASTER_CODE)
                .bind(login.trim())
                .fetch_optional(pool)
                .await
        }
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "invalid login or password".to_string())?;

        let hash: Option<String> = row.get("password_hash");
        let Some(hash) = hash.filter(|s| !s.is_empty()) else {
            return Err("password login not configured for this corp".into());
        };
        if !verify_password(password, &hash) {
            return Err("invalid login or password".into());
        }
        let corp_id: i64 = row.get("id");
        let now = now_ms();
        let token = format!("corp_{}", Uuid::new_v4().simple());
        let expires_at = now + SESSION_TTL_MS;
        sqlx::query(q::INSERT_SESSION)
            .bind(corp_id)
            .bind(&token)
            .bind(expires_at)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        let profile = self
            .profile(corp_id)
            .await?
            .ok_or_else(|| "profile missing".to_string())?;
        Ok((profile, token, expires_at))
    }

    pub async fn resolve_session(&self, token: &str) -> Result<i64, String> {
        let pool = self.pool()?;
        let now = now_ms();
        let row = sqlx::query(q::SESSION_BY_TOKEN)
            .bind(token)
            .bind(now)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "invalid or expired corp session".to_string())?;
        Ok(row.get("corporate_user_id"))
    }

    pub async fn profile(&self, corp_id: i64) -> Result<Option<CorpProfile>, String> {
        let pool = self.pool()?;
        let row = sqlx::query(q::BY_ID)
            .bind(corp_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
        let Some(row) = row else {
            return Ok(None);
        };
        let docs = self.list_docs_for_corp(corp_id).await?;
        Ok(Some(profile_from_row(row, docs)))
    }

    pub async fn list_docs_for_corp(&self, corp_id: i64) -> Result<Vec<CorpDocumentView>, String> {
        let pool = self.pool()?;
        let rows = sqlx::query(q::LIST_DOCS_FOR_CORP)
            .bind(corp_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(doc_from_row).collect())
    }

    pub async fn get_document(&self, doc_id: i64) -> Result<Option<(CorpDocumentView, String)>, String> {
        let pool = self.pool()?;
        let row = sqlx::query(q::DOC_BY_ID)
            .bind(doc_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(row.map(|r| {
            let path: String = r.get("stored_path");
            (doc_from_row(r), path)
        }))
    }

    pub async fn submit_game(
        &self,
        corp_id: i64,
        game_name: &str,
        game_code: &str,
        partner_code: Option<&str>,
        partner_game_id: Option<&str>,
        description: Option<&str>,
        docs: Vec<(String, String, String, Vec<u8>)>, // doc_type, file_name, content_type, bytes
    ) -> Result<CorpGameView, String> {
        let pool = self.pool()?;
        let profile = self
            .profile(corp_id)
            .await?
            .ok_or_else(|| "corp not found".to_string())?;
        if profile.status != "active" {
            return Err("company account must be approved before submitting games".into());
        }
        if game_name.trim().is_empty() || game_code.trim().is_empty() {
            return Err("game_name and game_code required".into());
        }
        if docs.is_empty() {
            return Err("at least one game document required (government approval or approved material)".into());
        }
        for (dt, _, _, bytes) in &docs {
            if !matches!(
                dt.as_str(),
                "game_government_approval" | "game_approved_material" | "other"
            ) {
                return Err(format!("invalid doc_type: {dt}"));
            }
            if bytes.is_empty() {
                return Err("empty document upload".into());
            }
        }

        let now = now_ms();
        let mut saved = Vec::new();
        for (dt, fname, ctype, bytes) in &docs {
            let (path, safe) =
                uploads::save_upload(&self.upload_dir, &format!("games/{corp_id}"), fname, bytes)
                    .await?;
            saved.push((dt.clone(), safe, path, ctype.clone(), bytes.len() as i64));
        }

        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
        let game_id: i64 = sqlx::query_scalar(q::INSERT_GAME)
            .bind(corp_id)
            .bind(game_name.trim())
            .bind(game_code.trim())
            .bind(partner_code.filter(|s| !s.is_empty()))
            .bind(partner_game_id.filter(|s| !s.is_empty()))
            .bind(description.unwrap_or(""))
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                if e.to_string().contains("unique") {
                    "game_code already exists".into()
                } else {
                    e.to_string()
                }
            })?;
        for (dt, safe, path, ctype, size) in saved {
            sqlx::query_scalar::<_, i64>(q::INSERT_DOCUMENT)
                .bind(corp_id)
                .bind(game_id)
                .bind(&dt)
                .bind(&safe)
                .bind(&path)
                .bind(&ctype)
                .bind(size)
                .bind(now)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        }
        tx.commit().await.map_err(|e| e.to_string())?;
        self.get_game(corp_id, game_id)
            .await?
            .ok_or_else(|| "game missing after insert".to_string())
    }

    pub async fn list_games(&self, corp_id: i64) -> Result<Vec<CorpGameView>, String> {
        let pool = self.pool()?;
        let rows = sqlx::query(q::LIST_GAMES_FOR_CORP)
            .bind(corp_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            let id: i64 = row.get("id");
            let docs = self.list_docs_for_game(id).await?;
            out.push(game_from_row(row, docs));
        }
        Ok(out)
    }

    pub async fn get_game(&self, corp_id: i64, game_id: i64) -> Result<Option<CorpGameView>, String> {
        let pool = self.pool()?;
        let row = sqlx::query(q::GAME_BY_ID)
            .bind(game_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
        let Some(row) = row else {
            return Ok(None);
        };
        let owner: i64 = row.get("corporate_user_id");
        if owner != corp_id {
            return Ok(None);
        }
        let docs = self.list_docs_for_game(game_id).await?;
        Ok(Some(game_from_row(row, docs)))
    }

    pub async fn list_docs_for_game(&self, game_id: i64) -> Result<Vec<CorpDocumentView>, String> {
        let pool = self.pool()?;
        let rows = sqlx::query(q::LIST_DOCS_FOR_GAME)
            .bind(game_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(doc_from_row).collect())
    }

    pub fn upload_dir(&self) -> &str {
        &self.upload_dir
    }
}

fn profile_from_row(row: sqlx::postgres::PgRow, documents: Vec<CorpDocumentView>) -> CorpProfile {
    CorpProfile {
        id: row.get("id"),
        company_name: row.get("company_name"),
        master_code: row.get("master_code"),
        master_id: row.get("master_id"),
        status: row.get("status"),
        api_enabled: row.get("api_enabled"),
        contact_email: row.get("contact_email"),
        contact_name: row.get("contact_name"),
        reject_reason: row.get("reject_reason"),
        approved_at: row.get("approved_at"),
        created_at: row.get("created_at"),
        documents,
    }
}

fn doc_from_row(row: sqlx::postgres::PgRow) -> CorpDocumentView {
    CorpDocumentView {
        id: row.get("id"),
        corporate_user_id: row.get("corporate_user_id"),
        game_id: row.get("game_id"),
        doc_type: row.get("doc_type"),
        file_name: row.get("file_name"),
        content_type: row.get("content_type"),
        byte_size: row.get("byte_size"),
        status: row.get("status"),
        review_note: row.get("review_note"),
        created_at: row.get("created_at"),
    }
}

fn game_from_row(row: sqlx::postgres::PgRow, documents: Vec<CorpDocumentView>) -> CorpGameView {
    CorpGameView {
        id: row.get("id"),
        corporate_user_id: row.get("corporate_user_id"),
        game_name: row.get("game_name"),
        game_code: row.get("game_code"),
        partner_code: row.get("partner_code"),
        partner_game_id: row.get("partner_game_id"),
        status: row.get("status"),
        description: row.get("description"),
        reject_reason: row.get("reject_reason"),
        approved_at: row.get("approved_at"),
        created_at: row.get("created_at"),
        documents,
    }
}
