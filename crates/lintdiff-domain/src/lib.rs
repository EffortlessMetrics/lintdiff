//! Backward-compatible `lintdiff-domain` facade.
//!
//! **DEPRECATED**: This crate is deprecated. Use `lintdiff-ingest-core` instead.
//!
//! The true SRP split now lives in `lintdiff-ingest-core`. This crate re-exports
//! the core API for backward compatibility only.

#![deprecated(since = "0.2.0", note = "use lintdiff_ingest_core instead")]

#[deprecated(since = "0.2.0", note = "use lintdiff_ingest_core instead")]
#[allow(deprecated)]
pub use lintdiff_core::*;
