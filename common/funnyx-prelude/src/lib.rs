//! Thin re-export crate: `use funnyx_prelude::*` in FunnyX services.
//!
//! Workspace layout: `common/` — see `doc/project.md` §8.

pub use funnyx_auth;
pub use funnyx_config;
pub use funnyx_error;
pub use funnyx_health;
pub use funnyx_heartbeat;
pub use funnyx_net_api;
pub use funnyx_socket;
pub use funnyx_socket_msg;
pub use funnyx_time;
pub use funnyx_types;

pub use funnyx_error::{FunnyxError, Result};
pub use funnyx_time::TimestampMs;
pub use funnyx_config::{load_default, GlobalConfig};
