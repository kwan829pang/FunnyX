//! Local snapshot persistence for maintenance stop / restart.
//! Atomic write: `engine_snapshot.json.tmp` → rename → `engine_snapshot.json`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::book::BookPersist;
use crate::types::{MaintenanceStatus, MarketPool, TradingPair};

pub const SNAPSHOT_FILE: &str = "engine_snapshot.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSnapshot {
    pub version: u32,
    pub engine_id: String,
    pub quote_asset: String,
    pub next_order_id: u64,
    pub maintenance: MaintenanceStatus,
    pub saved_at_ms: i64,
    pub pairs: Vec<TradingPair>,
    pub pools: Vec<MarketPool>,
    pub books: Vec<BookPersist>,
}

impl EngineSnapshot {
    pub fn path(data_dir: &Path) -> PathBuf {
        data_dir.join(SNAPSHOT_FILE)
    }

    pub fn exists(data_dir: &Path) -> bool {
        Self::path(data_dir).is_file()
    }

    pub fn load(data_dir: &Path) -> Result<Self, String> {
        let path = Self::path(data_dir);
        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        serde_json::from_str(&raw).map_err(|e| format!("parse snapshot: {e}"))
    }

    pub fn save(&self, data_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(data_dir).map_err(|e| format!("create data dir: {e}"))?;
        let final_path = Self::path(data_dir);
        let tmp_path = data_dir.join(format!("{SNAPSHOT_FILE}.tmp"));
        let body = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialize snapshot: {e}"))?;
        fs::write(&tmp_path, body).map_err(|e| format!("write tmp snapshot: {e}"))?;
        // On Windows, rename over existing may fail — remove first.
        if final_path.exists() {
            fs::remove_file(&final_path).map_err(|e| format!("remove old snapshot: {e}"))?;
        }
        fs::rename(&tmp_path, &final_path).map_err(|e| format!("rename snapshot: {e}"))?;
        Ok(final_path)
    }
}
