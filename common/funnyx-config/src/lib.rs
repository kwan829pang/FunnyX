//! FunnyX platform constants and global config with env defaults.
//!
//! - [`constants`] — compile-time / default constant values
//! - [`global`] — [`GlobalConfig`] struct
//! - [`load`] — read `.env`; missing keys filled from constants
//!
//! See `doc/project.md` §8 and `technology.md`.

pub mod constants;
pub mod global;
pub mod load;

pub use constants::*;
pub use global::GlobalConfig;
pub use load::{load, load_default};
