//! Pure diagnostic analysis and comparison for lintdiff.
//!
//! The engine accepts typed diagnostics and diff data and produces stable
//! receipt projections. External process, filesystem, and renderer concerns
//! remain outside this crate.

mod diagnostics;
mod identity;
mod matching;
mod policy;
mod receipt;
mod source;

pub use diagnostics::{
    parse_cargo_messages, Diagnostic, DiagnosticLevel, DiagnosticsParseError, Span,
};
pub use identity::fingerprint;
pub use matching::{compile_filters, path_allowed, relativize_span_path, select_spans, Filters};
pub use policy::{
    compute_verdict, counts_from_findings, format_level, is_code_allowed, map_level_to_severity,
    normalize_diagnostic_code,
};
pub use receipt::{ingest_on_diff, IngestOnDiffParams};
pub use source::{parse_unified_diff, DiffMap, DiffParseError, DiffStats};
