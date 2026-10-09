//! System settings + platform packages for Admin setup wizard.

use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

const KEY_BASE_FIAT: &str = "base_fiat_currency";
const KEY_INITIALIZED: &str = "system_initialized";

#[derive(Debug, Clone, Serialize)]
pub struct PackageView {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub game_coin_id: i64,
    pub coin_amount: f64,
    pub fiat_price: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequiredItem {
    pub key: String,
    pub label: String,
    pub ready: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetupStatus {
    pub initialized: bool,
    pub base_fiat_currency: Option<String>,
    pub base_fiat_ready: bool,
    pub packages_ready: bool,
    pub active_package_count: i64,
    pub required: Vec<RequiredItem>,
    pub packages: Vec<PackageView>,
}

struct MemorySys {
    settings: RwLock<HashMap<String, String>>,
    packages: RwLock<HashMap<i64, PackageView>>,
}

#[derive(Clone)]
pub struct SystemStore {
    pool: Option<PgPool>,
    memory: Option<Arc<MemorySys>>,
}

impl SystemStore {
    pub fn new(pool: Option<PgPool>) -> Self {
        if pool.is_some() {
            Self { pool, memory: None }
        } else {
            // Fresh memory mode: not initialized until wizard completes.
            let mut packages = HashMap::new();
            for p in demo_packages() {
                packages.insert(p.id, p);
            }
            Self {
                pool: None,
                memory: Some(Arc::new(MemorySys {
                    settings: RwLock::new(HashMap::new()),
                    packages: RwLock::new(packages),
                })),
            }
        }
    }

    pub async fn setup_status(&self) -> SetupStatus {
        let base = self.get_setting(KEY_BASE_FIAT).await;
        let base_ready = matches!(base.as_deref(), Some("HKD") | Some("USD"));
        let packages = self.list_packages().await;
        let active = packages
            .iter()
            .filter(|p| p.status == "active" && p.fiat_price > 0.0)
            .count() as i64;
        let packages_ready = active >= 1;
        let flag = self.get_setting(KEY_INITIALIZED).await;
        let initialized = flag.as_deref() == Some("true") && base_ready && packages_ready;

        let required = vec![
            RequiredItem {
                key: KEY_BASE_FIAT.into(),
                label: "Base fiat currency".into(),
                ready: base_ready,
                detail: base
                    .clone()
                    .unwrap_or_else(|| "not set (required: HKD or USD)".into()),
            },
            RequiredItem {
                key: "platform_shop_packages".into(),
                label: "Platform PLT package prices".into(),
                ready: packages_ready,
                detail: format!("{active} active package(s) with fiat_price > 0"),
            },
            RequiredItem {
                key: KEY_INITIALIZED.into(),
                label: "Setup wizard completed".into(),
                ready: flag.as_deref() == Some("true"),
                detail: if flag.as_deref() == Some("true") {
                    "confirmed".into()
                } else {
                    "pending confirmation".into()
                },
            },
        ];

        SetupStatus {
            initialized,
            base_fiat_currency: base,
            base_fiat_ready: base_ready,
            packages_ready,
            active_package_count: active,
            required,
            packages,
        }
    }

    pub async fn get_base_currency(&self) -> Option<String> {
        self.get_setting(KEY_BASE_FIAT).await
    }

    pub async fn set_base_currency(
        &self,
        code: &str,
        admin_id: i64,
    ) -> Result<String, String> {
        let c = code.trim().to_uppercase();
        if c != "HKD" && c != "USD" {
            return Err("base_fiat_currency must be HKD or USD".into());
        }
        self.upsert_setting(KEY_BASE_FIAT, &c, admin_id).await?;
        Ok(c)
    }

    pub async fn list_packages(&self) -> Vec<PackageView> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(crate::query::system::LIST_PACKAGES)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
            return rows.into_iter().map(map_package).collect();
        }
        let Some(mem) = &self.memory else {
            return Vec::new();
        };
        let mut v: Vec<_> = mem.packages.read().await.values().cloned().collect();
        v.sort_by_key(|p| p.id);
        v
    }

    pub async fn create_package(
        &self,
        code: &str,
        name: &str,
        coin_amount: f64,
        fiat_price: f64,
        status: Option<&str>,
        admin_id: i64,
    ) -> Result<PackageView, String> {
        let code = code.trim().to_uppercase();
        let name = name.trim();
        if code.is_empty() || name.is_empty() {
            return Err("code and name required".into());
        }
        if !code.starts_with("PLT_") {
            return Err("platform package code must start with PLT_".into());
        }
        if coin_amount <= 0.0 {
            return Err("coin_amount must be > 0".into());
        }
        if fiat_price <= 0.0 {
            return Err("fiat_price must be > 0".into());
        }
        let status = status.unwrap_or("active");
        if !matches!(status, "active" | "inactive" | "archived") {
            return Err("invalid status".into());
        }
        let now = now_ms();
        let game_coin_id = self.platform_token_coin_id().await?;

        if let Some(pool) = &self.pool {
            let row = sqlx::query(crate::query::system::INSERT_PACKAGE)
            .bind(&code)
            .bind(name)
            .bind(game_coin_id)
            .bind(coin_amount)
            .bind(fiat_price)
            .bind(status)
            .bind(admin_id)
            .bind(now)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("duplicate") || msg.contains("unique") {
                    "package code already exists".into()
                } else {
                    msg
                }
            })?;
            return Ok(map_package(row));
        }

        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        {
            let g = mem.packages.read().await;
            if g.values().any(|p| p.code == code) {
                return Err("package code already exists".into());
            }
        }
        let id = {
            let g = mem.packages.read().await;
            g.keys().max().copied().unwrap_or(0) + 1
        };
        let rec = PackageView {
            id,
            code,
            name: name.into(),
            game_coin_id,
            coin_amount,
            fiat_price,
            status: status.into(),
        };
        mem.packages.write().await.insert(id, rec.clone());
        Ok(rec)
    }

    pub async fn set_package_status(
        &self,
        id: i64,
        status: &str,
    ) -> Result<PackageView, String> {
        let status = status.trim();
        if !matches!(status, "active" | "inactive" | "archived") {
            return Err("status must be active, inactive, or archived".into());
        }
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let row = sqlx::query(crate::query::system::UPDATE_PACKAGE_STATUS)
            .bind(status)
            .bind(now)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "package not found".to_string())?;
            return Ok(map_package(row));
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut g = mem.packages.write().await;
        let rec = g
            .get_mut(&id)
            .ok_or_else(|| "package not found".to_string())?;
        rec.status = status.into();
        Ok(rec.clone())
    }

    async fn platform_token_coin_id(&self) -> Result<i64, String> {
        if let Some(pool) = &self.pool {
            let id: Option<i64> =
                sqlx::query_scalar(crate::query::system::SELECT_PLATFORM_TOKEN_COIN_ID)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
            return id.ok_or_else(|| "platform token game_coin not found".to_string());
        }
        Ok(1)
    }

    pub async fn update_package_price(
        &self,
        id: i64,
        fiat_price: f64,
        name: Option<&str>,
        status: Option<&str>,
    ) -> Result<PackageView, String> {
        if fiat_price <= 0.0 {
            return Err("fiat_price must be > 0".into());
        }
        if let Some(s) = status {
            if !matches!(s, "active" | "inactive" | "archived") {
                return Err("invalid status".into());
            }
        }
        let now = now_ms();
        if let Some(pool) = &self.pool {
            let row = sqlx::query(crate::query::system::UPDATE_PACKAGE_PRICE)
            .bind(fiat_price)
            .bind(name.map(|s| s.trim()).filter(|s| !s.is_empty()))
            .bind(status)
            .bind(now)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "package not found".to_string())?;
            return Ok(map_package(row));
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        let mut g = mem.packages.write().await;
        let rec = g
            .get_mut(&id)
            .ok_or_else(|| "package not found".to_string())?;
        rec.fiat_price = fiat_price;
        if let Some(n) = name.map(|s| s.trim()).filter(|s| !s.is_empty()) {
            rec.name = n.into();
        }
        if let Some(s) = status {
            rec.status = s.into();
        }
        Ok(rec.clone())
    }

    pub async fn complete_setup(&self, admin_id: i64) -> Result<SetupStatus, String> {
        let status = self.setup_status().await;
        if !status.base_fiat_ready {
            return Err("set base_fiat_currency (HKD or USD) first".into());
        }
        if !status.packages_ready {
            return Err("configure at least one active platform package with fiat_price > 0".into());
        }
        self.upsert_setting(KEY_INITIALIZED, "true", admin_id).await?;
        Ok(self.setup_status().await)
    }

    async fn get_setting(&self, key: &str) -> Option<String> {
        if let Some(pool) = &self.pool {
            return sqlx::query_scalar::<_, String>(crate::query::system::SELECT_SETTING)
            .bind(key)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        }
        let mem = self.memory.as_ref()?;
        mem.settings.read().await.get(key).cloned()
    }

    async fn upsert_setting(
        &self,
        key: &str,
        value: &str,
        admin_id: i64,
    ) -> Result<(), String> {
        let now = now_ms();
        if let Some(pool) = &self.pool {
            sqlx::query(crate::query::system::UPSERT_SETTING)
            .bind(key)
            .bind(value)
            .bind(admin_id)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            return Ok(());
        }
        let mem = self.memory.as_ref().ok_or("store unavailable")?;
        mem.settings
            .write()
            .await
            .insert(key.into(), value.into());
        Ok(())
    }
}

fn demo_packages() -> Vec<PackageView> {
    [
        (1, "PLT_1000", "Platform Token 1000", 1000.0, 8.8),
        (2, "PLT_1500", "Platform Token 1500", 1500.0, 13.0),
        (3, "PLT_3000", "Platform Token 3000", 3000.0, 27.0),
        (4, "PLT_10000", "Platform Token 10000", 10000.0, 75.0),
    ]
    .into_iter()
    .map(|(id, code, name, amt, price)| PackageView {
        id,
        code: code.into(),
        name: name.into(),
        game_coin_id: 1,
        coin_amount: amt,
        fiat_price: price,
        status: "active".into(),
    })
    .collect()
}

fn map_package(row: sqlx::postgres::PgRow) -> PackageView {
    PackageView {
        id: row.get("id"),
        code: row.get("code"),
        name: row.get("name"),
        game_coin_id: row.get("game_coin_id"),
        coin_amount: row.get("coin_amount"),
        fiat_price: row.get("fiat_price"),
        status: row.get("status"),
    }
}
