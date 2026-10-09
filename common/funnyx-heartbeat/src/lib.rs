//! **Private** in-system heartbeat (FunnyX service ↔ FunnyX service).
//!
//! - Socket PING/PONG is the core liveness signal ([`probe`], [`listen`], [`socket`]).
//! - Services HTTP-register with Config Server ([`client`]); Config probes via socket.
//! - **Not** for Cloudflare / Game Partners — those use public `GET /health` (`funnyx-health`).
//!
//! Spec: `doc/socket_message.md` §4, `doc/project.md` §11.

pub mod timing;
pub mod socket;
pub mod listen;
pub mod probe;
pub mod http;
#[cfg(feature = "http-client")]
pub mod client;

pub use timing::{HeartbeatTiming, is_stale};
pub use socket::{pong_frame_if_ping, ping_frame, Heartbeat};
pub use listen::spawn_ping_pong_listener;
pub use probe::{parse_tcp_addr, ping_peer};
pub use http::{HttpHeartbeatRequest, RegisterServiceRequest, ServiceInstanceView};

#[cfg(feature = "http-client")]
pub use client::{spawn_registry_heartbeat, spawn_registry_register, RegistryHeartbeatOpts};
