use axum::http::{HeaderMap, StatusCode};
use axum::Json;

use crate::models::ErrorBody;

/// Request header: `X-Sim-Status: {status}` or `X-Sim-Status: {status} {ms}`.
pub const HEADER_NAME: &str = "x-sim-status";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimStatus {
    Success,
    Pending,
    Rejected,
    Cancel,
}

/// Parsed `X-Sim-Status` directive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimDirective {
    pub status: SimStatus,
    /// Delay before applying `status` (milliseconds). Submit responses stay PENDING until then.
    pub delay_ms: u64,
}

impl SimStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "SUCCESS",
            Self::Pending => "PENDING",
            Self::Rejected => "REJECTED",
            Self::Cancel => "CANCEL",
        }
    }

    pub fn parse_token(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_uppercase().as_str() {
            "SUCCESS" => Ok(Self::Success),
            "PENDING" => Ok(Self::Pending),
            "REJECTED" => Ok(Self::Rejected),
            "CANCEL" | "CANCELLED" | "CANCELED" => Ok(Self::Cancel),
            other => Err(format!(
                "status must be SUCCESS, PENDING, REJECTED, or CANCEL (got '{other}')"
            )),
        }
    }

    /// Map to shop payment / callback status.
    pub fn shop_status(self) -> &'static str {
        match self {
            Self::Success => "paid",
            Self::Pending => "pending",
            Self::Rejected => "failed",
            Self::Cancel => "cancelled",
        }
    }

    /// Map to deposit status.
    pub fn deposit_status(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Pending => "pending",
            Self::Rejected => "rejected",
            Self::Cancel => "cancelled",
        }
    }

    pub fn should_fire_callback(self) -> bool {
        !matches!(self, Self::Pending)
    }
}

impl SimDirective {
    pub fn header_value(self) -> String {
        format!("{} {}", self.status.as_str(), self.delay_ms)
    }

    /// Absent header → stay PENDING (no scheduled change).
    pub fn from_headers(
        headers: &HeaderMap,
    ) -> Result<Option<Self>, (StatusCode, Json<ErrorBody>)> {
        let Some(raw) = headers.get(HEADER_NAME) else {
            return Ok(None);
        };
        let value = raw.to_str().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody {
                    error: format!("{HEADER_NAME} must be valid UTF-8"),
                }),
            )
        })?;
        Self::parse(value)
            .map(Some)
            .map_err(|msg| (StatusCode::BAD_REQUEST, Json(ErrorBody { error: msg })))
    }

    /// Formats: `SUCCESS` | `REJECTED 5000` | `CANCEL 0`
    pub fn parse(raw: &str) -> Result<Self, String> {
        let parts: Vec<&str> = raw.split_whitespace().collect();
        match parts.as_slice() {
            [] => Err(format!(
                "{HEADER_NAME} expected '{{status}}' or '{{status}} {{ms}}'"
            )),
            [status] => Ok(Self {
                status: SimStatus::parse_token(status)?,
                delay_ms: 0,
            }),
            [status, ms] => {
                let delay_ms: u64 = ms.parse().map_err(|_| {
                    format!("{HEADER_NAME} delay ms must be an integer (got '{ms}')")
                })?;
                Ok(Self {
                    status: SimStatus::parse_token(status)?,
                    delay_ms,
                })
            }
            _ => Err(format!(
                "{HEADER_NAME} expected '{{status}}' or '{{status}} {{ms}}' (got '{raw}')"
            )),
        }
    }

    pub fn schedules_change(self) -> bool {
        !matches!(self.status, SimStatus::Pending)
    }
}
