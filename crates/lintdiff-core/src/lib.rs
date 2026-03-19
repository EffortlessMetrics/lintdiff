//! Compatibility façade over `lintdiff-ingest`.
//!
//! **DEPRECATED**: This crate is deprecated. Use `lintdiff_ingest_core` instead.
//!
//! This crate is intentionally tiny and re-exports the stable public API used by
//! adapters and integration points while keeping the actual orchestration logic in
//! a focused microcrate.

#![deprecated(since = "0.2.0", note = "use lintdiff_ingest_core instead")]

#[allow(deprecated)]
pub use lintdiff_ingest::*;
