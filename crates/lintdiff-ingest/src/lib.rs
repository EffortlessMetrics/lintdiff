//! Compatibility façade over `lintdiff-ingest-core`.
//!
//! This crate preserves the historical public surface while moving the
//! implementation to its dedicated microcrate.
//!
//! # Deprecation
//!
//! This crate is deprecated. Use `lintdiff_ingest_core` directly instead.

#![deprecated(since = "0.2.0", note = "use lintdiff_ingest_core instead")]

pub use lintdiff_ingest_core::*;
