//! Compatibility façade over `lintdiff-ingest`.
//!
//! This crate is intentionally tiny and re-exports the stable public API used by
//! adapters and integration points while keeping the actual orchestration logic in
//! a focused microcrate.

pub use lintdiff_ingest::*;
