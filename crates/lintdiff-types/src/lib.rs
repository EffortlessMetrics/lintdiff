//! Shared DTOs and small utilities.
//!
//! This crate is intentionally "boring": mostly plain data structures with `serde`
//! derives and a few deterministic helpers (path normalization, finding ordering).

mod config;
mod ordering;
mod path;
mod report;

pub use config::*;
pub use ordering::*;
pub use path::*;
pub use report::*;
