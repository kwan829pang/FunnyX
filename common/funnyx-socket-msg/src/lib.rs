//! Socket message models and encode/decode from `doc/socket_message.md`.
//!
//! Normal Command fixed size = 512 bytes. Transport/compression is in `funnyx-socket`.

use bytes::{BufMut, Bytes, BytesMut};
use funnyx_error::{FunnyxError, Result};
use funnyx_types::{MessageType, OrderType, TradeAction};
use serde::{Deserialize, Serialize};

/// Normal Command total size.
pub const NORMAL_COMMAND_SIZE: usize = 512;
/// Header bytes before content.
pub const NORMAL_HEADER_SIZE: usize = 5;
/// Content capacity.
pub const NORMAL_CONTENT_SIZE: usize = NORMAL_COMMAND_SIZE - NORMAL_HEADER_SIZE;

/// Decoded Normal Command (fixed 512-byte wire form).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalCommand {
    pub message_type: MessageType,
    pub event_name: u8,
    pub action_type: u8,
    pub content: Vec<u8>,
}

impl NormalCommand {
    pub fn new(event_name: u8, action_type: u8, content: Vec<u8>) -> Result<Self> {
        if content.len() > NORMAL_CONTENT_SIZE {
            return Err(FunnyxError::Protocol(format!(
                "content length {} exceeds {}",
                content.len(),
                NORMAL_CONTENT_SIZE
            )));
        }
        Ok(Self {
            message_type: MessageType::NormalCommand,
            event_name,
            action_type,
            content,
        })
    }

    /// Encode to a 512-byte buffer.
    pub fn encode(&self) -> Result<Bytes> {
        if self.content.len() > NORMAL_CONTENT_SIZE {
            return Err(FunnyxError::Protocol("content too large".into()));
        }
        let mut buf = BytesMut::with_capacity(NORMAL_COMMAND_SIZE);
        buf.put_u8(self.message_type.as_u8());
        buf.put_u8(self.event_name);
        buf.put_u8(self.action_type);
        buf.put_u16(self.content.len() as u16);
        buf.extend_from_slice(&self.content);
        buf.resize(NORMAL_COMMAND_SIZE, 0);
        Ok(buf.freeze())
    }

    /// Decode from at least 512 bytes (extra ignored).
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < NORMAL_HEADER_SIZE {
            return Err(FunnyxError::Protocol(format!(
                "normal command too short: {}",
                data.len()
            )));
        }
        let message_type = MessageType::from_u8(data[0])?;
        if message_type != MessageType::NormalCommand {
            return Err(FunnyxError::Protocol(
                "expected NormalCommand message type".into(),
            ));
        }
        let event_name = data[1];
        let action_type = data[2];
        let content_size = u16::from_be_bytes([data[3], data[4]]) as usize;
        if content_size > NORMAL_CONTENT_SIZE {
            return Err(FunnyxError::Protocol(format!(
                "invalid content size: {content_size}"
            )));
        }
        if data.len() < NORMAL_HEADER_SIZE + content_size {
            return Err(FunnyxError::Protocol("truncated content".into()));
        }
        let content = data[NORMAL_HEADER_SIZE..NORMAL_HEADER_SIZE + content_size].to_vec();
        Ok(Self {
            message_type,
            event_name,
            action_type,
            content,
        })
    }
}

/// Order payload inside Normal Command content (see socket_message.md §2.4.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderPayload {
    pub order_type: OrderType,
    pub trade_side: TradeAction,
    pub client_id: i64,
    pub market_id: i64,
    pub price: i64,
    pub quantity: i64,
}

impl OrderPayload {
    pub const WIRE_SIZE: usize = 1 + 1 + 8 + 8 + 8 + 8;

    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(Self::WIRE_SIZE);
        buf.put_u8(self.order_type as u8);
        buf.put_u8(self.trade_side as u8);
        buf.put_i64(self.client_id);
        buf.put_i64(self.market_id);
        buf.put_i64(self.price);
        buf.put_i64(self.quantity);
        buf.freeze()
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < Self::WIRE_SIZE {
            return Err(FunnyxError::Protocol("order payload too short".into()));
        }
        let client_id = i64::from_be_bytes(
            data[2..10]
                .try_into()
                .map_err(|_| FunnyxError::Protocol("client_id bytes".into()))?,
        );
        let market_id = i64::from_be_bytes(
            data[10..18]
                .try_into()
                .map_err(|_| FunnyxError::Protocol("market_id bytes".into()))?,
        );
        let price = i64::from_be_bytes(
            data[18..26]
                .try_into()
                .map_err(|_| FunnyxError::Protocol("price bytes".into()))?,
        );
        let quantity = i64::from_be_bytes(
            data[26..34]
                .try_into()
                .map_err(|_| FunnyxError::Protocol("quantity bytes".into()))?,
        );
        Ok(Self {
            order_type: OrderType::from_u8(data[0])?,
            trade_side: TradeAction::from_u8(data[1])?,
            client_id,
            market_id,
            price,
            quantity,
        })
    }
}

/// Complex Command envelope (variable body; type byte `0xF2`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplexCommand {
    pub event_name: u8,
    pub action_type: u8,
    pub payload: Vec<u8>,
}

impl ComplexCommand {
    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(1 + 1 + 1 + 4 + self.payload.len());
        buf.put_u8(MessageType::ComplexCommand.as_u8());
        buf.put_u8(self.event_name);
        buf.put_u8(self.action_type);
        buf.put_u32(self.payload.len() as u32);
        buf.extend_from_slice(&self.payload);
        buf.freeze()
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 7 {
            return Err(FunnyxError::Protocol("complex command too short".into()));
        }
        let mt = MessageType::from_u8(data[0])?;
        if mt != MessageType::ComplexCommand {
            return Err(FunnyxError::Protocol(
                "expected ComplexCommand message type".into(),
            ));
        }
        let event_name = data[1];
        let action_type = data[2];
        let len = u32::from_be_bytes([data[3], data[4], data[5], data[6]]) as usize;
        if data.len() < 7 + len {
            return Err(FunnyxError::Protocol("complex payload truncated".into()));
        }
        Ok(Self {
            event_name,
            action_type,
            payload: data[7..7 + len].to_vec(),
        })
    }
}

/// Peek first byte and dispatch.
pub fn peek_message_type(data: &[u8]) -> Result<MessageType> {
    data.first()
        .copied()
        .ok_or_else(|| FunnyxError::Protocol("empty buffer".into()))
        .and_then(MessageType::from_u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_roundtrip() {
        let cmd = NormalCommand::new(0, 1, b"hi".to_vec()).unwrap();
        let encoded = cmd.encode().unwrap();
        assert_eq!(encoded.len(), NORMAL_COMMAND_SIZE);
        let decoded = NormalCommand::decode(&encoded).unwrap();
        assert_eq!(decoded.content, b"hi");
    }

    #[test]
    fn order_payload_roundtrip() {
        let o = OrderPayload {
            order_type: OrderType::Market,
            trade_side: TradeAction::Bid,
            client_id: 1,
            market_id: 2,
            price: 100,
            quantity: 5,
        };
        let b = o.encode();
        let d = OrderPayload::decode(&b).unwrap();
        assert_eq!(o, d);
    }
}
