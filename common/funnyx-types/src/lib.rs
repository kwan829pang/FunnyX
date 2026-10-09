//! Protocol and domain enum discriminators from `doc/system_type.md`.
//!
//! Runtime numeric/string knobs live in `funnyx-config`, not here.

use funnyx_error::{FunnyxError, Result};
use serde::{Deserialize, Serialize};

/// Message type byte (`0xF1` Normal, `0xF2` Complex).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    NormalCommand = 0xF1,
    ComplexCommand = 0xF2,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0xF1 => Ok(Self::NormalCommand),
            0xF2 => Ok(Self::ComplexCommand),
            other => Err(FunnyxError::Protocol(format!(
                "unknown message type: {other:#04x}"
            ))),
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Server type integers from `doc/system_type.md`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServerType {
    ConfigHeartbeat = 0,
    AdminApi = 1,
    AdminPanel = 2,
    WebhookServer = 3,
    MessageCenter = 4,
    SessionTokenServer = 5,
    Gateway = 6,
    CoreEngine = 7,
    ClientCenter = 8,
    ClientWeb = 9,
    CacheRedis = 10,
    DocumentMongo = 11,
    RelationalPostgres = 12,
}

impl ServerType {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::ConfigHeartbeat),
            1 => Ok(Self::AdminApi),
            2 => Ok(Self::AdminPanel),
            3 => Ok(Self::WebhookServer),
            4 => Ok(Self::MessageCenter),
            5 => Ok(Self::SessionTokenServer),
            6 => Ok(Self::Gateway),
            7 => Ok(Self::CoreEngine),
            8 => Ok(Self::ClientCenter),
            9 => Ok(Self::ClientWeb),
            10 => Ok(Self::CacheRedis),
            11 => Ok(Self::DocumentMongo),
            12 => Ok(Self::RelationalPostgres),
            other => Err(FunnyxError::Validation(format!(
                "unknown server type: {other}"
            ))),
        }
    }
}

/// User role type.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    AdminRoot = 0,
    AdminManager = 1,
    AdminStaffMaker = 2,
    AdminStaffChecker = 3,
    AdminAuthor = 4,
    AdminReadOnly = 5,
    ClientNormal = 6,
    ClientCorp = 7,
    ClientEndUser = 8,
    AiAgent = 9,
    PublicUser = 10,
}

/// Order type (market vs price).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderType {
    Market = 1,
    Price = 2,
}

impl OrderType {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Market),
            2 => Ok(Self::Price),
            other => Err(FunnyxError::Validation(format!(
                "unknown order type: {other}"
            ))),
        }
    }
}

/// Order status.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending = 0,
    Cancel = 1,
    Filled = 2,
    PartialFilled = 3,
}

/// Shop package status.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShopPackageStatus {
    Active = 0,
    Inactive = 1,
    Archived = 2,
}

/// Shop order status.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShopOrderStatus {
    Pending = 0,
    Paid = 1,
    Failed = 2,
    Expired = 3,
    Cancelled = 4,
}

/// Shop payment event status.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShopPaymentEventStatus {
    Received = 0,
    Settled = 1,
    IgnoredDuplicate = 2,
    Rejected = 3,
}

/// Trade side / action.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TradeAction {
    Bid = 1,
    Ask = 2,
}

impl TradeAction {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Bid),
            2 => Ok(Self::Ask),
            other => Err(FunnyxError::Validation(format!(
                "unknown trade action: {other}"
            ))),
        }
    }
}

/// Socket event list (subset; extend as services land).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventCode {
    Heartbeat = 0,
    AdminLogin = 1,
    AdminLogout = 2,
    ClientLogin = 11,
    ClientLogout = 12,
    ShopOrderCreated = 23,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_type_roundtrip() {
        assert_eq!(
            MessageType::from_u8(0xF1).unwrap(),
            MessageType::NormalCommand
        );
        assert!(MessageType::from_u8(0x00).is_err());
    }
}
