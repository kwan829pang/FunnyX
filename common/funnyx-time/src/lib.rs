//! UTC+0 timestamps stored as BIGINT milliseconds (platform rule).
//!
//! See `doc/project.md` §1.

use funnyx_error::{FunnyxError, Result};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Milliseconds since Unix epoch (UTC+0), matching BIGINT columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TimestampMs(pub i64);

impl TimestampMs {
    /// Current wall-clock time in UTC milliseconds.
    pub fn now() -> Result<Self> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| FunnyxError::Internal(format!("system clock before epoch: {e}")))?;
        Ok(Self(duration.as_millis() as i64))
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }

    pub fn from_i64(value: i64) -> Result<Self> {
        if value < 0 {
            return Err(FunnyxError::Validation(format!(
                "timestamp must be non-negative: {value}"
            )));
        }
        Ok(Self(value))
    }
}

impl From<TimestampMs> for i64 {
    fn from(value: TimestampMs) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_is_positive() {
        let ts = TimestampMs::now().unwrap();
        assert!(ts.as_i64() > 0);
    }
}
