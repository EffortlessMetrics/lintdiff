//! Backward-compatible `lintdiff-domain` facade.
//!
//! The true SRP split now lives in `lintdiff-ingest`, re-exported via
//! `lintdiff-core` for compatibility. This crate re-exports the core API
//! for consumers that still depend on `lintdiff-domain`.

pub use lintdiff_core::*;
