//! Auth credential types and signature helpers (not a full Token Server / IdP).
//!
//! See `doc/client_connect.md`, `api-master.md` Auth legend, `doc/project.md` §5.1.

use funnyx_error::{FunnyxError, Result};
use funnyx_time::TimestampMs;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Auth class for an HTTP or socket call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthClass {
    Public,
    Session,
    Corp,
    Admin,
    Internal,
    Partner,
    Oauth,
}

/// Client Web session token issued by Session Token Server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionToken {
    pub token: String,
}

impl SessionToken {
    pub fn new(token: impl Into<String>) -> Result<Self> {
        let token = token.into();
        if token.trim().is_empty() {
            return Err(FunnyxError::Auth("empty session token".into()));
        }
        Ok(Self { token })
    }

    pub fn as_str(&self) -> &str {
        &self.token
    }
}

/// Company Partner MasterSigned credentials (C8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MasterSignedCreds {
    pub master_account_code: String,
    pub master_id: String,
    pub api_key: String,
    /// Raw secret used for HMAC; never log this field.
    pub secret: String,
    pub timestamp_ms: i64,
}

impl MasterSignedCreds {
    pub fn validate_present(&self) -> Result<()> {
        for (name, value) in [
            ("master_account_code", &self.master_account_code),
            ("master_id", &self.master_id),
            ("api_key", &self.api_key),
            ("secret", &self.secret),
        ] {
            if value.trim().is_empty() {
                return Err(FunnyxError::Auth(format!("missing {name}")));
            }
        }
        Ok(())
    }

    /// Canonical string to sign: code|id|api_key|timestamp|body
    pub fn signing_payload(&self, body: &str) -> String {
        format!(
            "{}|{}|{}|{}|{}",
            self.master_account_code, self.master_id, self.api_key, self.timestamp_ms, body
        )
    }

    /// HMAC-SHA256 hex signature over the signing payload.
    pub fn sign(&self, body: &str) -> Result<String> {
        self.validate_present()?;
        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())
            .map_err(|e| FunnyxError::Auth(format!("hmac key: {e}")))?;
        mac.update(self.signing_payload(body).as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    /// Verify a provided signature.
    pub fn verify(&self, body: &str, signature_hex: &str) -> Result<()> {
        let expected = self.sign(body)?;
        if expected.eq_ignore_ascii_case(signature_hex) {
            Ok(())
        } else {
            Err(FunnyxError::Auth("signature mismatch".into()))
        }
    }
}

/// OAuth 2.0 token bundle (partner or platform).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OauthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at_ms: Option<i64>,
}

impl OauthTokens {
    pub fn bearer(access_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            refresh_token: None,
            token_type: "Bearer".into(),
            expires_at_ms: None,
        }
    }
}

/// Stamp current UTC ms onto MasterSigned creds.
pub fn stamp_now(mut creds: MasterSignedCreds) -> Result<MasterSignedCreds> {
    creds.timestamp_ms = TimestampMs::now()?.as_i64();
    Ok(creds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn master_sign_verify() {
        let creds = MasterSignedCreds {
            master_account_code: "AC".into(),
            master_id: "MID".into(),
            api_key: "KEY".into(),
            secret: "sekrit".into(),
            timestamp_ms: 1,
        };
        let sig = creds.sign("body").unwrap();
        assert!(creds.verify("body", &sig).is_ok());
        assert!(creds.verify("other", &sig).is_err());
    }
}
