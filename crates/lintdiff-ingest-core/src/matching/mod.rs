//! Matching helpers for path and span filtering.
//!
//! Internal path/matching utilities used by ingest and policy stages.
//! These helpers remain private to avoid broadening the public API surface.

mod filters;
mod paths;
mod spans;

pub use filters::{compile_filters, path_allowed};
pub use paths::relativize_span_path;
pub use spans::select_spans;
