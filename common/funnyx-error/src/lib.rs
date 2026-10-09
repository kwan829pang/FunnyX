//! Shared error and result types for FunnyX backend services.
//!
//! See `doc/project.md` §8 and `api-master.md` for HTTP/socket error conventions.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Convenient result alias used across FunnyX common crates.
pub type Result<T> = std::result::Result<T, FunnyxError>;

/// Cross-cutting error kinds for HTTP and socket paths.
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "message")]
pub enum FunnyxError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("authentication error: {0}")]
    Auth(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("protocol / socket message error: {0}")]
    Protocol(String),

    #[error("network / I/O error: {0}")]
    Network(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl FunnyxError {
    /// Stable machine-readable code for API / socket responses.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Config(_) => "config",
            Self::Auth(_) => "auth",
            Self::Validation(_) => "validation",
            Self::Protocol(_) => "protocol",
            Self::Network(_) => "network",
            Self::NotFound(_) => "not_found",
            Self::Internal(_) => "internal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable() {
        assert_eq!(FunnyxError::Auth("x".into()).code(), "auth");
        assert_eq!(FunnyxError::Protocol("x".into()).code(), "protocol");
    }
}
