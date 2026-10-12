//! Admin review of corporate onboarding and game document submissions.

use serde::Serialize;
use sqlx::{PgPool, Row};

use crate::query::corp_onboard as q;

#[derive(Debug, Clone, Serialize)]
pub struct AdminCorpUser {
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
    pub documents: Vec<AdminCorpDoc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminCorpDoc {
    pub id: i64,
    pub corporate_user_id: i64,
    pub game_id: Option<i64>,
    pub doc_type: String,
    pub file_name: String,
    pub stored_path: String,
    pub content_type: String,
    pub byte_size: i64,
    pub status: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminGame {
    pub id: i64,
    pub corporate_user_id: i64,
    pub company_name: String,
    pub game_name: String,
    pub game_code: String,
    pub partner_code: String,
    pub partner_game_id: String,
    pub status: String,
    pub description: String,
    pub reject_reason: String,
    pub approved_at: i64,
    pub created_at: i64,
    pub documents: Vec<AdminCorpDoc>,
}

#[derive(Clone)]
pub struct CorpUsersStore {
    pool: Option<PgPool>,
    upload_dir: String,
}

impl CorpUsersStore {
    pub fn new(pool: Option<PgPool>, upload_dir: String) -> Self {
        Self { pool, upload_dir }
    }

    fn pool(&self) -> Result<&PgPool, String> {
        self.pool
            .as_ref()
            .ok_or_else(|| "POSTGRES_URL required".into())
    }

    pub fn upload_dir(&self) -> &str {
        &self.upload_dir
    }

    pub async fn list_corps(&self, status: Option<&str>) -> Result<Vec<AdminCorpUser>, String> {
        let pool = self.pool()?;
        let rows = if let Some(s) = status {
            sqlx::query(q::LIST_CORPS_FILTER)
                .bind(s)
                .fetch_all(pool)
                .await
        } else {
            sqlx::query(q::LIST_CORPS).fetch_all(pool).await
        }
        .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            let id: i64 = row.get("id");
            let docs = self.docs_for_corp(id).await?;
            out.push(corp_from_row(row, docs));
        }
        Ok(out)
    }

    pub async fn get_corp(&self, id: i64) -> Result<Option<AdminCorpUser>, String> {
        let pool = self.pool()?;
        let row = sqlx::query(q::CORP_BY_ID)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
        let Some(row) = row else {
            return Ok(None);
        };
        let docs = self.docs_for_corp(id).await?;
        Ok(Some(corp_from_row(row, docs)))
    }

    async fn docs_for_corp(&self, corp_id: i64) -> Result<Vec<AdminCorpDoc>, String> {
        let pool = self.pool()?;
        let rows = sqlx::query(q::DOCS_FOR_CORP)
            .bind(corp_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(doc_from_row).collect())
    }

    pub async fn approve_corp(&self, id: i64, admin_id: i64) -> Result<AdminCorpUser, String> {
        let pool = self.pool()?;
        let now = chrono_now();
        let updated = sqlx::query(q::APPROVE_CORP)
            .bind(id)
            .bind(admin_id)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();
        if updated == 0 {
            return Err("corp not found or not pending".into());
        }
        sqlx::query(q::ACCEPT_BR_DOCS)
            .bind(id)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        self.get_corp(id)
            .await?
            .ok_or_else(|| "missing after approve".into())
    }

    pub async fn reject_corp(
        &self,
        id: i64,
        reason: &str,
    ) -> Result<AdminCorpUser, String> {
        let pool = self.pool()?;
        let now = chrono_now();
        let updated = sqlx::query(q::REJECT_CORP)
            .bind(id)
            .bind(reason)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();
        if updated == 0 {
            return Err("corp not found or not pending".into());
        }
        self.get_corp(id)
            .await?
            .ok_or_else(|| "missing after reject".into())
    }

    pub async fn list_games(&self, status: Option<&str>) -> Result<Vec<AdminGame>, String> {
        let pool = self.pool()?;
        let rows = if let Some(s) = status {
            sqlx::query(q::LIST_GAMES_FILTER)
                .bind(s)
                .fetch_all(pool)
                .await
        } else {
            sqlx::query(q::LIST_GAMES).fetch_all(pool).await
        }
        .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            let id: i64 = row.get("id");
            let docs = self.docs_for_game(id).await?;
            out.push(game_from_row(row, docs));
        }
        Ok(out)
    }

    pub async fn get_game(&self, id: i64) -> Result<Option<AdminGame>, String> {
        let pool = self.pool()?;
        let row = sqlx::query(q::GAME_BY_ID)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
        let Some(row) = row else {
            return Ok(None);
        };
        let docs = self.docs_for_game(id).await?;
        Ok(Some(game_from_row(row, docs)))
    }

    async fn docs_for_game(&self, game_id: i64) -> Result<Vec<AdminCorpDoc>, String> {
        let pool = self.pool()?;
        let rows = sqlx::query(q::DOCS_FOR_GAME)
            .bind(game_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(doc_from_row).collect())
    }

    pub async fn approve_game(&self, id: i64) -> Result<AdminGame, String> {
        let pool = self.pool()?;
        let now = chrono_now();
        let updated = sqlx::query(q::APPROVE_GAME)
            .bind(id)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();
        if updated == 0 {
            return Err("game not found or not pending".into());
        }
        sqlx::query(q::ACCEPT_GAME_DOCS)
            .bind(id)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        self.get_game(id)
            .await?
            .ok_or_else(|| "missing after approve".into())
    }

    pub async fn reject_game(&self, id: i64, reason: &str) -> Result<AdminGame, String> {
        let pool = self.pool()?;
        let now = chrono_now();
        let updated = sqlx::query(q::REJECT_GAME)
            .bind(id)
            .bind(reason)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();
        if updated == 0 {
            return Err("game not found or not pending".into());
        }
        self.get_game(id)
            .await?
            .ok_or_else(|| "missing after reject".into())
    }

    pub async fn get_doc(
        &self,
        id: i64,
    ) -> Result<Option<(AdminCorpDoc, String)>, String> {
        let pool = self.pool()?;
        let row = sqlx::query(q::DOC_BY_ID)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(row.map(|r| {
            let path: String = r.get("stored_path");
            (doc_from_row(r), path)
        }))
    }
}

fn chrono_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn corp_from_row(row: sqlx::postgres::PgRow, documents: Vec<AdminCorpDoc>) -> AdminCorpUser {
    AdminCorpUser {
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

fn doc_from_row(row: sqlx::postgres::PgRow) -> AdminCorpDoc {
    AdminCorpDoc {
        id: row.get("id"),
        corporate_user_id: row.get("corporate_user_id"),
        game_id: row.get("game_id"),
        doc_type: row.get("doc_type"),
        file_name: row.get("file_name"),
        stored_path: row.get("stored_path"),
        content_type: row.get("content_type"),
        byte_size: row.get("byte_size"),
        status: row.get("status"),
        created_at: row.get("created_at"),
    }
}

fn game_from_row(row: sqlx::postgres::PgRow, documents: Vec<AdminCorpDoc>) -> AdminGame {
    AdminGame {
        id: row.get("id"),
        corporate_user_id: row.get("corporate_user_id"),
        company_name: row.get("company_name"),
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
