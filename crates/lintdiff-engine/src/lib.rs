//! Pure diagnostic analysis and comparison for lintdiff.
//!
//! The engine accepts typed diagnostics and diff data and produces stable
//! receipt projections. External process, filesystem, and renderer concerns
//! remain outside this crate.

mod compare;
mod diagnostics;
mod identity;
mod inventory;
mod matching;
mod policy;
mod receipt;
mod source;

pub use compare::{
    compare_inventories, Comparability, ComparabilityStatus, ContextualChange,
    DiagnosticComparison, DiagnosticRef, MatchBasis, PairingEvidence, ReasonCode,
};
pub use diagnostics::{
    parse_cargo_analysis, parse_cargo_analysis_with_repo_root, parse_cargo_messages,
    parse_cargo_messages_with_status, AnalysisCompletion, AnalysisScope, CargoAnalysis,
    CargoDiagnosticStream, CargoTarget, Diagnostic, DiagnosticChild, DiagnosticLevel,
    DiagnosticObservation, DiagnosticSuggestion, DiagnosticsParseError, ObservationSpan,
    ProducerUnit, Span, UpstreamExecution,
};
pub use identity::fingerprint;
pub use inventory::inventory_from_analysis;
pub use matching::{compile_filters, path_allowed, relativize_span_path, select_spans, Filters};
pub use policy::{
    compute_verdict, counts_from_findings, evaluate_delta_policy, format_level, is_code_allowed,
    map_level_to_severity, normalize_diagnostic_code,
};
pub use receipt::{build_delta_receipt, ingest_on_diff, IngestOnDiffParams};
pub use source::{
    parse_source_change_set, parse_unified_diff, source_diff_id, DiffMap, DiffParseError,
    DiffStats, FileDelta, HunkDelta, LineMapSegment, LineOffset, LocationMapping, SourceChangeSet,
};
