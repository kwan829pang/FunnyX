//! Local multipart file storage for corp onboarding documents.

use std::path::{Path, PathBuf};

use tokio::fs;
use uuid::Uuid;

pub async fn ensure_upload_dir(root: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(root);
    fs::create_dir_all(&path)
        .await
        .map_err(|e| format!("create upload dir: {e}"))?;
    Ok(path)
}

pub async fn save_upload(
    root: &str,
    subdir: &str,
    original_name: &str,
    bytes: &[u8],
) -> Result<(String, String), String> {
    let root = ensure_upload_dir(root).await?;
    let dir = root.join(subdir);
    fs::create_dir_all(&dir)
        .await
        .map_err(|e| format!("create subdir: {e}"))?;
    let safe = sanitize_filename(original_name);
    let stored_name = format!("{}_{safe}", Uuid::new_v4().simple());
    let full = dir.join(&stored_name);
    fs::write(&full, bytes)
        .await
        .map_err(|e| format!("write upload: {e}"))?;
    let rel = format!("{subdir}/{stored_name}");
    Ok((rel, safe))
}

pub fn resolve_path(root: &str, stored_path: &str) -> PathBuf {
    Path::new(root).join(stored_path)
}

fn sanitize_filename(name: &str) -> String {
    let base = Path::new(name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file.bin");
    let cleaned: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "file.bin".into()
    } else {
        cleaned
    }
}
