use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use sha2::{Digest, Sha256};

/// SHA-256 hex of UTF-8 `"demo"` — what Flutter sends after [PasswordDigest.sha256Hex].
pub const DEMO_PASSWORD_DIGEST: &str =
    "2a97516c354b68848cdbd8f54a226a0a55b21ed138e207ad6c5cbb9c00aa5aea";

/// Lowercase hex SHA-256 of UTF-8 bytes.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn looks_like_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Normalize a login/register `password` field from Flutter or curl.
///
/// Flutter clients POST SHA-256 hex; legacy/curl plaintext is hashed here so
/// Argon2 at rest always covers the digest (never the raw password alone).
pub fn normalize_client_password(password: &str) -> String {
    let t = password.trim();
    if looks_like_sha256_hex(t) {
        t.to_ascii_lowercase()
    } else {
        sha256_hex(t)
    }
}

pub fn hash_password(client_digest_or_plain: &str) -> Result<String, String> {
    let digest = normalize_client_password(client_digest_or_plain);
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(digest.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify_password(client_digest_or_plain: &str, hash: &str) -> bool {
    let digest = normalize_client_password(client_digest_or_plain);
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(digest.as_bytes(), &parsed)
        .is_ok()
}
