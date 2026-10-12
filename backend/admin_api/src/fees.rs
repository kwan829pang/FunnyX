//! C6/C9 platform fees + corp license / api_enabled controls.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use sqlx::Row;

use crate::admins::{role_allowed, AdminUser};
use crate::api::{err_status, extract_bearer, require_admin_session, AppState};
use crate::models::ErrorBody;

/// Suggested catalog amounts (Admin fiat or USDT).
pub const FEE_YEAR1: f64 = 80_000.0;
pub const FEE_RENEWAL: f64 = 10_000.0;
pub const FEE_NEW_PAIR: f64 = 10_000.0;

async fn require_ops_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminUser, (StatusCode, Json<ErrorBody>)> {
    let token = extract_bearer(headers)
        .ok_or_else(|| err_status(StatusCode::UNAUTHORIZED, "Authorization Bearer required"))?;
    let (admin, _) = require_admin_session(state, &token).await?;
    if !role_allowed(&admin.role, &["super_admin", "admin"]) {
        return Err(err_status(StatusCode::FORBIDDEN, "admin role required"));
    }
    Ok(admin)
}

fn row_amount(r: &sqlx::postgres::PgRow) -> f64 {
    if let Ok(v) = r.try_get::<f64, _>("amount") {
        return v;
    }
    if let Ok(v) = r.try_get::<String, _>("amount") {
        return v.parse().unwrap_or(0.0);
    }
    0.0
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub async fn list_fees(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_ops_admin(&state, &headers).await?;
    let Some(pool) = &state.pool else {
        return Ok(Json(serde_json::json!({
            "fees": [],
            "catalog": fee_catalog(),
            "source": "admin_api",
            "note": "POSTGRES_URL required",
        })));
    };
    let rows = sqlx::query(
        "SELECT id, corporate_user_id, fee_type, amount::float8 AS amount, pay_currency, status, \
                license_year, paid_at, created_at \
         FROM fx_corp.platform_fees ORDER BY id DESC LIMIT 200",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let fees: Vec<_> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<i64, _>("id"),
                "corporate_user_id": r.get::<i64, _>("corporate_user_id"),
                "fee_type": r.get::<String, _>("fee_type"),
                "amount": row_amount(&r),
                "pay_currency": r.get::<String, _>("pay_currency"),
                "status": r.get::<String, _>("status"),
                "license_year": r.get::<i32, _>("license_year"),
                "paid_at": r.get::<i64, _>("paid_at"),
                "created_at": r.get::<i64, _>("created_at"),
            })
        })
        .collect();
    Ok(Json(serde_json::json!({
        "fees": fees,
        "catalog": fee_catalog(),
        "source": "admin_api",
    })))
}

fn fee_catalog() -> serde_json::Value {
    serde_json::json!({
        "year1_package": FEE_YEAR1,
        "license_renewal": FEE_RENEWAL,
        "new_pair": FEE_NEW_PAIR,
        "note": "C6/C9 suggested amounts (Admin fiat or USDT)",
    })
}

pub async fn list_corps(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_ops_admin(&state, &headers).await?;
    let Some(pool) = &state.pool else {
        return Ok(Json(serde_json::json!({
            "corps": [],
            "source": "admin_api",
            "note": "POSTGRES_URL required",
        })));
    };
    let rows = sqlx::query(
        "SELECT id, company_name, status, api_enabled, license_deadline_at, created_at \
         FROM fx_corp.corporate_users ORDER BY id DESC LIMIT 200",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let corps: Vec<_> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<i64, _>("id"),
                "company_name": r.get::<String, _>("company_name"),
                "status": r.get::<String, _>("status"),
                "api_enabled": r.get::<bool, _>("api_enabled"),
                "license_deadline_at": r.get::<i64, _>("license_deadline_at"),
                "created_at": r.get::<i64, _>("created_at"),
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "corps": corps, "source": "admin_api" })))
}

#[derive(Debug, Deserialize)]
pub struct CreateFeeBody {
    pub corporate_user_id: i64,
    pub fee_type: String,
    pub amount: f64,
    pub pay_currency: String,
    #[serde(default)]
    pub license_year: Option<i32>,
}

pub async fn create_fee(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateFeeBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let _ = require_ops_admin(&state, &headers).await?;
    let Some(pool) = &state.pool else {
        return Err(err_status(
            StatusCode::SERVICE_UNAVAILABLE,
            "POSTGRES_URL required",
        ));
    };
    let fee_type = body.fee_type.trim();
    match fee_type {
        "year1_package" | "license_renewal" | "new_pair" | "supply_increase" => {}
        _ => {
            return Err(err_status(StatusCode::BAD_REQUEST, "invalid fee_type"));
        }
    }
    let cur = body.pay_currency.trim().to_ascii_uppercase();
    if !matches!(cur.as_str(), "HKD" | "USD" | "USDT") {
        return Err(err_status(StatusCode::BAD_REQUEST, "invalid pay_currency"));
    }
    if body.amount <= 0.0 {
        return Err(err_status(StatusCode::BAD_REQUEST, "amount must be > 0"));
    }
    let now = now_ms();
    let license_year = body.license_year.unwrap_or(0);
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO fx_corp.platform_fees ( \
            corporate_user_id, fee_type, amount, pay_currency, license_year, \
            status, created_at, updated_at \
         ) VALUES ($1, $2, $3, $4, $5, 'pending', $6, 0) RETURNING id",
    )
    .bind(body.corporate_user_id)
    .bind(fee_type)
    .bind(body.amount)
    .bind(&cur)
    .bind(license_year)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| err_status(StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "fee": { "id": id, "status": "pending" },
            "source": "admin_api",
        })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct CorpAccessBody {
    #[serde(default)]
    pub api_enabled: Option<bool>,
    #[serde(default)]
    pub license_deadline_at: Option<i64>,
}

pub async fn patch_corp_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<CorpAccessBody>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let admin = require_ops_admin(&state, &headers).await?;
    let Some(pool) = &state.pool else {
        return Err(err_status(
            StatusCode::SERVICE_UNAVAILABLE,
            "POSTGRES_URL required",
        ));
    };
    if body.api_enabled.is_none() && body.license_deadline_at.is_none() {
        return Err(err_status(StatusCode::BAD_REQUEST, "nothing to update"));
    }
    let now = now_ms();
    let prev: Option<(bool,)> = sqlx::query_as(
        "SELECT api_enabled FROM fx_corp.corporate_users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let Some((was_enabled,)) = prev else {
        return Err(err_status(StatusCode::NOT_FOUND, "corporate user not found"));
    };

    let row = sqlx::query(
        "UPDATE fx_corp.corporate_users SET \
            api_enabled = COALESCE($2, api_enabled), \
            license_deadline_at = COALESCE($3, license_deadline_at), \
            updated_at = $4 \
         WHERE id = $1 \
         RETURNING id, company_name, api_enabled, license_deadline_at, status",
    )
    .bind(id)
    .bind(body.api_enabled)
    .bind(body.license_deadline_at)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| err_status(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let now_enabled: bool = row.get("api_enabled");
    if was_enabled && !now_enabled {
        let _ = sqlx::query(
            "INSERT INTO fx_corp.corp_partner_notices ( \
                corporate_user_id, notice_type, title, body, deadline_at, \
                created_by_admin_id, created_at \
             ) VALUES ($1, 'api_disabled', $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind("MasterSigned APIs disabled")
        .bind("Platform APIs are disabled (unpaid license or admin action). New market submissions are blocked until re-enabled.")
        .bind(row.get::<i64, _>("license_deadline_at"))
        .bind(admin.admin_user_id)
        .bind(now)
        .execute(pool)
        .await;
    }

    Ok(Json(serde_json::json!({
        "corporate_user": {
            "id": row.get::<i64, _>("id"),
            "company_name": row.get::<String, _>("company_name"),
            "api_enabled": now_enabled,
            "license_deadline_at": row.get::<i64, _>("license_deadline_at"),
            "status": row.get::<String, _>("status"),
        },
        "source": "admin_api",
    })))
}
