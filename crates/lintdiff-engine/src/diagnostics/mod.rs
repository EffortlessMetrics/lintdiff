//! Parse cargo `--message-format=json` output into normalized diagnostics.
//!
//! This crate provides functionality to parse JSON output from cargo's
//! `--message-format=json` flag and extract compiler diagnostics.
//!
//! # Overview
//!
//! When running `cargo check --message-format=json`, cargo outputs JSON lines
//! for various events. This crate filters for `compiler-message` events and
//! extracts structured diagnostic information.
//!
//! # Example
//!
//! ```rust
//! use std::io::Cursor;
//! use lintdiff_engine::{parse_cargo_messages, DiagnosticLevel};
//!
//! let json_output = r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused variable","code":{"code":"unused_variables"},"spans":[{"file_name":"src/lib.rs","line_start":10,"is_primary":true}]}}"#;
//!
//! let diagnostics = parse_cargo_messages(Cursor::new(json_output)).unwrap();
//! assert_eq!(diagnostics.len(), 1);
//! assert_eq!(diagnostics[0].level, DiagnosticLevel::Warning);
//! assert_eq!(diagnostics[0].code_raw.as_deref(), Some("unused_variables"));
//! ```
//!
//! # Data Structures
//!
//! - [`Diagnostic`]: A single compiler diagnostic with level, message, code, and spans
//! - [`DiagnosticLevel`]: The severity level (error, warning, note, help, or other)
//! - [`Span`]: A source code location referenced by a diagnostic
//! - [`DiagnosticsParseError`]: Errors that can occur during parsing

use std::{io::BufRead, path::Path};

use serde_json::Value;
use thiserror::Error;

use lintdiff_types::NormPath;

/// A single compiler diagnostic message.
///
/// Contains all the relevant information about a diagnostic including
/// its severity level, message text, optional error code, source spans,
/// and optionally the rendered output.
///
/// # Example
///
/// ```rust
/// use lintdiff_engine::{Diagnostic, DiagnosticLevel, Span};
/// use lintdiff_types::NormPath;
///
/// // Diagnostics are typically created via parse_cargo_messages,
/// // but here's what the structure looks like:
/// let diag = Diagnostic {
///     level: DiagnosticLevel::Error,
///     code_raw: Some("E0425".to_string()),
///     message: "cannot find value `x` in this scope".to_string(),
///     spans: vec![],
///     rendered: None,
/// };
///
/// assert_eq!(diag.level, DiagnosticLevel::Error);
/// assert_eq!(diag.code_raw.as_deref(), Some("E0425"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code_raw: Option<String>,
    pub message: String,
    pub spans: Vec<Span>,
    pub rendered: Option<String>,
}

/// The severity level of a diagnostic message.
///
/// Maps to the standard rustc diagnostic levels, with an `Other` variant
/// for any custom or unknown levels.
///
/// # Example
///
/// ```rust
/// use lintdiff_engine::DiagnosticLevel;
///
/// let level = DiagnosticLevel::Warning;
/// assert_eq!(level, DiagnosticLevel::Warning);
///
/// let custom = DiagnosticLevel::Other("custom-level".to_string());
/// match custom {
///     DiagnosticLevel::Other(name) => assert_eq!(name, "custom-level"),
///     _ => panic!("Expected Other variant"),
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiagnosticLevel {
    /// An error - compilation will fail.
    Error,
    /// A warning - compilation will succeed but should be reviewed.
    Warning,
    /// A note - additional information about a diagnostic.
    Note,
    /// A help message - suggests a fix or improvement.
    Help,
    /// Any other diagnostic level not recognized.
    Other(String),
}

/// A source code location referenced by a diagnostic.
///
/// Spans identify where in the source code a diagnostic applies.
/// They include file path, line and column ranges, and whether
/// this is the primary span for the diagnostic.
///
/// # Example
///
/// ```rust
/// use lintdiff_engine::Span;
/// use lintdiff_types::NormPath;
///
/// let span = Span {
///     file: NormPath::new("src/lib.rs"),
///     line_start: 42,
///     line_end: 44,
///     col_start: Some(5),
///     col_end: Some(10),
///     is_primary: true,
/// };
///
/// assert_eq!(span.file.as_str(), "src/lib.rs");
/// assert_eq!(span.line_start, 42);
/// assert!(span.is_primary);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    /// The file path where the span is located.
    pub file: NormPath,
    /// The starting line number (1-based).
    pub line_start: u32,
    /// The ending line number (1-based, inclusive).
    pub line_end: u32,
    /// The starting column number (1-based), if available.
    pub col_start: Option<u32>,
    /// The ending column number (1-based, exclusive), if available.
    pub col_end: Option<u32>,
    /// Whether this is the primary span for the diagnostic.
    pub is_primary: bool,
}

/// Errors that can occur while parsing cargo diagnostic messages.
///
/// # Example
///
/// ```rust
/// use lintdiff_engine::{parse_cargo_messages, DiagnosticsParseError};
/// use std::io::Cursor;
///
/// let bad_json = "not valid json";
/// let result = parse_cargo_messages(Cursor::new(bad_json));
///
/// match result {
///     Err(DiagnosticsParseError::InvalidJson { line, .. }) => {
///         assert_eq!(line, 1);
///     }
///     _ => panic!("Expected InvalidJson error"),
/// }
/// ```
#[derive(Debug, Error)]
pub enum DiagnosticsParseError {
    /// The JSON syntax was invalid.
    #[error("invalid json at line {line}: {source}")]
    InvalidJson {
        /// The 1-based line number where the error occurred.
        line: usize,
        /// The underlying JSON parsing error.
        #[source]
        source: serde_json::Error,
    },
    /// The JSON was valid but didn't have the expected structure.
    #[error("unexpected json shape at line {line}: {msg}")]
    InvalidShape {
        /// The 1-based line number where the error occurred.
        line: usize,
        /// A description of what was wrong with the shape.
        msg: String,
    },
}

/// The Cargo producer identity attached to one compiler-message emission.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProducerUnit {
    pub package_id: Option<String>,
    pub manifest_path: Option<String>,
    pub target: Option<CargoTarget>,
    pub profile: Option<String>,
}

/// The package target that produced a compiler message.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CargoTarget {
    pub name: Option<String>,
    pub kind: Vec<String>,
    pub crate_types: Vec<String>,
    pub src_path: Option<String>,
    pub edition: Option<String>,
}

/// Hard and contextual inputs that determine whether two analyses are comparable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AnalysisScope {
    pub repository: Option<String>,
    pub revision: Option<String>,
    pub toolchain: Option<String>,
    pub target: Option<String>,
    pub features: Vec<String>,
    pub package_selection: Vec<String>,
    pub target_selection: Vec<String>,
    pub lint_config_hash: Option<String>,
}

/// A source span retaining raw Cargo values alongside the current normalized span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationSpan {
    pub raw_file_name: Option<String>,
    pub raw_line_start: Option<u32>,
    pub raw_line_end: Option<u32>,
    pub raw_column_start: Option<u32>,
    pub raw_column_end: Option<u32>,
    pub normalized: Option<Span>,
    pub is_primary: bool,
}

/// A suggestion emitted as part of a compiler child diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticSuggestion {
    pub file_name: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub replacement: Option<String>,
    pub applicability: Option<String>,
}

/// A child note, help message, or nested diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticChild {
    pub raw_level: String,
    pub level: DiagnosticLevel,
    pub message: String,
    pub rendered: Option<String>,
    pub spans: Vec<ObservationSpan>,
    pub suggestions: Vec<DiagnosticSuggestion>,
}

/// One Cargo compiler-message observation, before filtering or policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticObservation {
    pub producer: ProducerUnit,
    pub raw_level: String,
    pub raw_code: Option<String>,
    pub message: String,
    pub rendered: Option<String>,
    pub spans: Vec<ObservationSpan>,
    pub children: Vec<DiagnosticChild>,
    pub diagnostic: Diagnostic,
}

/// Completion state for one parsed Cargo analysis stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisCompletion {
    SuccessfulComplete,
    FailedComplete,
    IncompleteStream,
    RuntimeFailure,
}

/// Process and Cargo completion evidence for one analysis.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpstreamExecution {
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub build_finished_seen: bool,
    pub build_success: Option<bool>,
    pub completion: Option<AnalysisCompletion>,
}

/// Complete Cargo observations and terminal stream evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoAnalysis {
    pub scope: AnalysisScope,
    pub observations: Vec<DiagnosticObservation>,
    pub execution: UpstreamExecution,
}

impl CargoAnalysis {
    /// Construct analysis evidence for a process that could not produce a Cargo stream.
    pub fn runtime_failure(command: Vec<String>, duration_ms: Option<u64>) -> Self {
        Self {
            scope: AnalysisScope::default(),
            observations: Vec::new(),
            execution: UpstreamExecution {
                command,
                duration_ms,
                completion: Some(AnalysisCompletion::RuntimeFailure),
                ..UpstreamExecution::default()
            },
        }
    }

    /// Attach process metadata acquired by the application shell.
    pub fn with_process_evidence(
        mut self,
        command: Vec<String>,
        exit_code: Option<i32>,
        duration_ms: Option<u64>,
    ) -> Self {
        self.execution.command = command;
        self.execution.exit_code = exit_code;
        self.execution.duration_ms = duration_ms;
        self
    }
}

/// The normalized diagnostics and completion evidence from one Cargo stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoDiagnosticStream {
    pub diagnostics: Vec<Diagnostic>,
    pub observations: Vec<DiagnosticObservation>,
    pub scope: AnalysisScope,
    pub build_finished: bool,
    pub build_success: Option<bool>,
    pub completion: AnalysisCompletion,
    pub execution: UpstreamExecution,
}

/// Parse a cargo JSON-lines stream, returning only compiler messages.
///
/// This function reads a stream of JSON lines (as produced by
/// `cargo check --message-format=json`) and extracts only the
/// `compiler-message` events, returning them as structured diagnostics.
///
/// # Arguments
///
/// * `reader` - Any reader implementing `BufRead` that contains JSON lines.
///
/// # Returns
///
/// A `Result` containing either:
/// - A `Vec<Diagnostic>` of all compiler messages found
/// - A `DiagnosticsParseError` if parsing failed
///
/// # Example
///
/// ```rust
/// use std::io::Cursor;
/// use lintdiff_engine::{parse_cargo_messages, DiagnosticLevel};
///
/// // Simulated cargo output with mixed message types
/// let cargo_output = r#"{"reason":"compiler-artifact","package_id":"test"}
/// {"reason":"compiler-message","message":{"level":"error","message":"cannot find value","code":{"code":"E0425"},"spans":[{"file_name":"src/lib.rs","line_start":10,"is_primary":true}]}}
/// {"reason":"build-finished","success":false}"#;
///
/// let diagnostics = parse_cargo_messages(Cursor::new(cargo_output)).unwrap();
///
/// // Only compiler-message events are returned
/// assert_eq!(diagnostics.len(), 1);
/// assert_eq!(diagnostics[0].level, DiagnosticLevel::Error);
/// assert_eq!(diagnostics[0].code_raw.as_deref(), Some("E0425"));
/// ```
pub fn parse_cargo_messages<R: BufRead>(
    reader: R,
) -> Result<Vec<Diagnostic>, DiagnosticsParseError> {
    Ok(parse_cargo_messages_with_status(reader)?.diagnostics)
}

/// Parse a Cargo JSONL stream into complete, one-emission observations.
pub fn parse_cargo_analysis<R: BufRead>(reader: R) -> Result<CargoAnalysis, DiagnosticsParseError> {
    parse_cargo_analysis_with_repo_root(reader, None)
}

/// Parse Cargo JSONL and earn repository-relative paths from a known root.
pub fn parse_cargo_analysis_with_repo_root<R: BufRead>(
    reader: R,
    repo_root: Option<&Path>,
) -> Result<CargoAnalysis, DiagnosticsParseError> {
    let mut observations = Vec::new();
    let mut build_finished_seen = false;
    let mut build_success = None;
    let mut scope = AnalysisScope::default();

    for (idx, line_res) in reader.lines().enumerate() {
        let line_no = idx + 1;
        let line = line_res.map_err(|e| DiagnosticsParseError::InvalidShape {
            line: line_no,
            msg: format!("io error reading diagnostics stream: {e}"),
        })?;

        if line.trim().is_empty() {
            continue;
        }

        let value: Value =
            serde_json::from_str(&line).map_err(|e| DiagnosticsParseError::InvalidJson {
                line: line_no,
                source: e,
            })?;

        let reason = value.get("reason").and_then(Value::as_str);
        if reason == Some("build-finished") {
            build_finished_seen = true;
            build_success = value.get("success").and_then(Value::as_bool);
            scope.toolchain = value
                .get("toolchain")
                .and_then(Value::as_str)
                .map(str::to_string);
            scope.target = value
                .get("target")
                .and_then(Value::as_str)
                .map(str::to_string);
            scope.features = string_array(value.get("features"));
            continue;
        }
        if reason != Some("compiler-message") {
            continue;
        }

        let message_value =
            value
                .get("message")
                .ok_or_else(|| DiagnosticsParseError::InvalidShape {
                    line: line_no,
                    msg: "missing 'message' field".to_string(),
                })?;

        observations.push(parse_observation(&value, message_value, repo_root));
    }

    let completion = match (build_finished_seen, build_success) {
        (true, Some(true)) => AnalysisCompletion::SuccessfulComplete,
        (true, Some(false)) => AnalysisCompletion::FailedComplete,
        _ => AnalysisCompletion::IncompleteStream,
    };

    Ok(CargoAnalysis {
        scope,
        observations,
        execution: UpstreamExecution {
            build_finished_seen,
            build_success,
            completion: Some(completion),
            ..UpstreamExecution::default()
        },
    })
}

/// Parse Cargo JSONL while retaining the terminal build-finished evidence.
pub fn parse_cargo_messages_with_status<R: BufRead>(
    reader: R,
) -> Result<CargoDiagnosticStream, DiagnosticsParseError> {
    let analysis = parse_cargo_analysis(reader)?;
    let diagnostics = analysis
        .observations
        .iter()
        .map(|observation| observation.diagnostic.clone())
        .collect();
    let execution = analysis.execution.clone();
    let completion = execution
        .completion
        .ok_or_else(|| DiagnosticsParseError::InvalidShape {
            line: 0,
            msg: "parsed Cargo analysis did not contain completion state".to_string(),
        })?;
    Ok(CargoDiagnosticStream {
        diagnostics,
        observations: analysis.observations,
        scope: analysis.scope,
        build_finished: execution.build_finished_seen,
        build_success: execution.build_success,
        completion,
        execution,
    })
}

fn parse_observation(
    value: &Value,
    message_value: &Value,
    repo_root: Option<&Path>,
) -> DiagnosticObservation {
    let raw_level = message_value
        .get("level")
        .and_then(Value::as_str)
        .unwrap_or("other")
        .to_string();
    let level = parse_level(&raw_level);
    let message = message_value
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let rendered = message_value
        .get("rendered")
        .and_then(Value::as_str)
        .map(str::to_string);
    let raw_code = message_value
        .get("code")
        .and_then(|code| code.get("code"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let spans = parse_spans(message_value.get("spans"), repo_root);
    let children = parse_children(message_value.get("children"), repo_root);
    let normalized_spans = spans.iter().map(legacy_span).collect();

    DiagnosticObservation {
        producer: parse_producer(value, repo_root),
        raw_level,
        raw_code: raw_code.clone(),
        message: message.clone(),
        rendered: rendered.clone(),
        spans,
        children,
        diagnostic: Diagnostic {
            level,
            code_raw: raw_code,
            message,
            spans: normalized_spans,
            rendered,
        },
    }
}

fn parse_producer(value: &Value, repo_root: Option<&Path>) -> ProducerUnit {
    ProducerUnit {
        package_id: value
            .get("package_id")
            .and_then(Value::as_str)
            .map(|package_id| canonical_package_id(package_id, repo_root)),
        manifest_path: value
            .get("manifest_path")
            .and_then(Value::as_str)
            .map(|path| canonical_producer_path(path, repo_root)),
        target: parse_target(value.get("target"), repo_root),
        profile: value
            .get("profile")
            .and_then(Value::as_object)
            .and_then(|profile| profile.get("opt_level"))
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

fn parse_target(value: Option<&Value>, repo_root: Option<&Path>) -> Option<CargoTarget> {
    let target = value?.as_object()?;
    Some(CargoTarget {
        name: target
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_string),
        kind: string_array(target.get("kind")),
        crate_types: string_array(target.get("crate_types")),
        src_path: target
            .get("src_path")
            .and_then(Value::as_str)
            .map(|path| canonical_producer_path(path, repo_root)),
        edition: target
            .get("edition")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn canonical_package_id(package_id: &str, repo_root: Option<&Path>) -> String {
    if repo_root.is_some() && package_id.starts_with("path+file://") {
        if let Some((_, package)) = package_id.rsplit_once('#') {
            return format!("path+file://#{package}");
        }
    }
    package_id.to_string()
}

fn canonical_producer_path(raw: &str, repo_root: Option<&Path>) -> String {
    let raw_without_extended_prefix = raw.strip_prefix("\\\\?\\").unwrap_or(raw);
    let raw_normalized = raw_without_extended_prefix.replace('\\', "/");
    if let Some(repo_root) = repo_root {
        let mut root_normalized = repo_root.to_string_lossy().into_owned();
        if let Some(stripped) = root_normalized.strip_prefix("\\\\?\\") {
            root_normalized = stripped.to_string();
        }
        root_normalized = root_normalized.replace('\\', "/");
        let root_prefix = format!("{}/", root_normalized.trim_end_matches('/'));
        let comparable_raw = if root_normalized.as_bytes().get(1) == Some(&b':') {
            raw_normalized.to_ascii_lowercase()
        } else {
            raw_normalized.clone()
        };
        let comparable_root = if root_normalized.as_bytes().get(1) == Some(&b':') {
            root_prefix.to_ascii_lowercase()
        } else {
            root_prefix
        };
        if comparable_raw.starts_with(&comparable_root) {
            let relative = raw_normalized
                .get(comparable_root.len()..)
                .unwrap_or_default();
            return lintdiff_types::NormPath::from_repo_path(relative).to_string();
        }
    }
    raw_normalized
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn parse_level(raw: &str) -> DiagnosticLevel {
    match raw {
        "error" => DiagnosticLevel::Error,
        "warning" => DiagnosticLevel::Warning,
        "note" => DiagnosticLevel::Note,
        "help" => DiagnosticLevel::Help,
        other => DiagnosticLevel::Other(other.to_string()),
    }
}

fn parse_spans(value: Option<&Value>, repo_root: Option<&Path>) -> Vec<ObservationSpan> {
    value
        .and_then(Value::as_array)
        .map(|spans| {
            spans
                .iter()
                .map(|span| parse_span(span, repo_root))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_span(value: &Value, repo_root: Option<&Path>) -> ObservationSpan {
    let raw_file_name = value
        .get("file_name")
        .and_then(Value::as_str)
        .map(str::to_string);
    let raw_line_start = value.get("line_start").and_then(as_u32);
    let raw_line_end = value.get("line_end").and_then(as_u32);
    let raw_column_start = value.get("column_start").and_then(as_u32);
    let raw_column_end = value.get("column_end").and_then(as_u32);
    let is_primary = value
        .get("is_primary")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let normalized = match (
        repository_path(raw_file_name.as_deref(), repo_root),
        raw_line_start,
    ) {
        (Some(file), Some(line_start)) if line_start > 0 => Some(Span {
            file,
            line_start,
            line_end: raw_line_end
                .filter(|line_end| *line_end > 0)
                .unwrap_or(line_start)
                .max(line_start),
            col_start: raw_column_start,
            col_end: raw_column_end,
            is_primary,
        }),
        _ => None,
    };

    ObservationSpan {
        raw_file_name,
        raw_line_start,
        raw_line_end,
        raw_column_start,
        raw_column_end,
        normalized,
        is_primary,
    }
}

fn repository_path(raw_file_name: Option<&str>, repo_root: Option<&Path>) -> Option<NormPath> {
    let raw_file_name = raw_file_name?;
    let repo_root = repo_root?;
    let raw = Path::new(raw_file_name);
    if raw.is_absolute() {
        let relative = raw.strip_prefix(repo_root).ok()?;
        relative.to_str().map(NormPath::from_repo_path)
    } else {
        Some(NormPath::from_repo_path(raw_file_name))
    }
}

fn legacy_span(span: &ObservationSpan) -> Span {
    let line_start = span.raw_line_start.unwrap_or(0).max(1);
    Span {
        file: NormPath::from_repo_path(span.raw_file_name.as_deref().unwrap_or("")),
        line_start,
        line_end: span
            .raw_line_end
            .or(span.raw_line_start)
            .unwrap_or(0)
            .max(line_start),
        col_start: span.raw_column_start,
        col_end: span.raw_column_end,
        is_primary: span.is_primary,
    }
}

fn as_u32(value: &Value) -> Option<u32> {
    value.as_u64().and_then(|number| u32::try_from(number).ok())
}

fn parse_children(value: Option<&Value>, repo_root: Option<&Path>) -> Vec<DiagnosticChild> {
    value
        .and_then(Value::as_array)
        .map(|children| {
            children
                .iter()
                .map(|child| {
                    let raw_level = child
                        .get("level")
                        .and_then(Value::as_str)
                        .unwrap_or("other")
                        .to_string();
                    let spans = parse_spans(child.get("spans"), repo_root);
                    let suggestions = parse_suggestions(child.get("spans"));
                    DiagnosticChild {
                        raw_level: raw_level.clone(),
                        level: parse_level(&raw_level),
                        message: child
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                        rendered: child
                            .get("rendered")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        spans,
                        suggestions,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_suggestions(value: Option<&Value>) -> Vec<DiagnosticSuggestion> {
    value
        .and_then(Value::as_array)
        .map(|spans| {
            spans
                .iter()
                .filter_map(|span| {
                    let replacement = span
                        .get("suggested_replacement")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    let applicability = span
                        .get("suggestion_applicability")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    if replacement.is_none() && applicability.is_none() {
                        return None;
                    }
                    Some(DiagnosticSuggestion {
                        file_name: span
                            .get("file_name")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        line_start: span.get("line_start").and_then(as_u32),
                        line_end: span.get("line_end").and_then(as_u32),
                        replacement,
                        applicability,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_compiler_message_only() {
        let input = r#"{"reason":"build-script-executed","package_id":"x"}
{"reason":"compiler-message","message":{"level":"warning","message":"hi","code":{"code":"clippy::needless_borrow"},"spans":[{"file_name":"src/lib.rs","line_start":3,"line_end":3,"column_start":1,"column_end":2,"is_primary":true}]}}"#;
        let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].message, "hi");
        assert_eq!(
            diags[0].code_raw.as_deref(),
            Some("clippy::needless_borrow")
        );
        assert_eq!(diags[0].spans[0].file.as_str(), "src/lib.rs");
    }

    #[test]
    fn cargo_paths_preserve_repository_directories_named_a_or_b() {
        let input = r#"{"reason":"compiler-message","message":{"level":"warning","message":"hi","spans":[{"file_name":"a/src/lib.rs","line_start":3,"line_end":3,"is_primary":true}]}}"#;

        let diagnostics = parse_cargo_messages(Cursor::new(input)).unwrap();
        assert_eq!(diagnostics[0].spans[0].file.as_str(), "a/src/lib.rs");
    }

    #[test]
    fn parses_build_completion_evidence_without_changing_compatibility_wrapper() {
        let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"failed"}}
{"reason":"build-finished","success":false}"#;

        let stream = parse_cargo_messages_with_status(Cursor::new(input)).unwrap();
        assert_eq!(stream.diagnostics.len(), 1);
        assert!(stream.build_finished);
        assert_eq!(stream.build_success, Some(false));
        assert_eq!(parse_cargo_messages(Cursor::new(input)).unwrap().len(), 1);
    }

    #[test]
    fn incomplete_stream_has_no_build_success_value() {
        let stream = parse_cargo_messages_with_status(Cursor::new(
            r#"{"reason":"compiler-message","message":{"level":"warning","message":"partial"}}"#,
        ))
        .unwrap();

        assert!(!stream.build_finished);
        assert_eq!(stream.build_success, None);
        assert_eq!(stream.completion, AnalysisCompletion::IncompleteStream);
    }

    #[test]
    fn complete_analysis_preserves_producer_children_and_suggestions(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"{"reason":"compiler-message","package_id":"path+file:///repo#pkg","manifest_path":"/repo/Cargo.toml","target":{"name":"demo","kind":["lib"],"crate_types":["lib"],"src_path":"/repo/src/lib.rs","edition":"2024"},"message":{"level":"warning","message":"unused value","code":{"code":"unused_variables"},"rendered":"warning: unused value","spans":[{"file_name":"src/lib.rs","line_start":0,"is_primary":true}],"children":[{"level":"help","message":"remove it","spans":[{"file_name":"src/lib.rs","line_start":3,"line_end":3,"suggested_replacement":"","suggestion_applicability":"MachineApplicable"}]}]}}
{"reason":"build-finished","success":true,"toolchain":"rustc 1.86.0","target":"x86_64-pc-windows-msvc","features":["default","clippy"]}"#;

        let analysis = parse_cargo_analysis(Cursor::new(input))?;
        assert_eq!(analysis.observations.len(), 1);
        assert_eq!(
            analysis.execution.completion,
            Some(AnalysisCompletion::SuccessfulComplete)
        );
        assert_eq!(analysis.scope.toolchain.as_deref(), Some("rustc 1.86.0"));
        assert_eq!(
            analysis.scope.target.as_deref(),
            Some("x86_64-pc-windows-msvc")
        );
        assert_eq!(analysis.scope.features, ["default", "clippy"]);
        let observation = analysis.observations.first().ok_or("missing observation")?;
        assert_eq!(
            observation.producer.package_id.as_deref(),
            Some("path+file:///repo#pkg")
        );
        assert_eq!(
            observation
                .producer
                .target
                .as_ref()
                .and_then(|target| target.name.as_deref()),
            Some("demo")
        );
        let span = observation.spans.first().ok_or("missing span")?;
        assert_eq!(span.raw_line_start, Some(0));
        assert!(span.normalized.is_none());
        assert_eq!(observation.diagnostic.spans[0].line_start, 1);
        let child = observation.children.first().ok_or("missing child")?;
        let suggestion = child.suggestions.first().ok_or("missing suggestion")?;
        assert_eq!(suggestion.replacement.as_deref(), Some(""));
        assert_eq!(
            suggestion.applicability.as_deref(),
            Some("MachineApplicable")
        );
        Ok(())
    }

    #[test]
    fn repository_root_earns_context_correct_absolute_paths(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let repo_root = std::env::current_dir()?;
        let source_path = repo_root.join("src").join("lib.rs");
        let raw_file_name = source_path.to_string_lossy().to_string();
        let raw_file_name_json = serde_json::to_string(&raw_file_name)?;
        let input = format!(
            r#"{{"reason":"compiler-message","message":{{"level":"warning","message":"absolute","spans":[{{"file_name":{raw_file_name_json},"line_start":4,"line_end":4,"is_primary":true}}]}}}}"#
        );
        let analysis = parse_cargo_analysis_with_repo_root(Cursor::new(input), Some(&repo_root))?;
        let observation = analysis.observations.first().ok_or("missing observation")?;
        let span = observation.spans.first().ok_or("missing span")?;
        assert_eq!(span.raw_file_name.as_deref(), Some(raw_file_name.as_str()));
        assert_eq!(
            span.normalized.as_ref().map(|span| span.file.as_str()),
            Some("src/lib.rs")
        );
        Ok(())
    }

    #[test]
    fn producer_identity_is_stable_across_base_and_head_worktrees(
    ) -> Result<(), Box<dyn std::error::Error>> {
        fn stream(root: &str) -> Result<String, serde_json::Error> {
            serde_json::to_string(&serde_json::json!({
                "reason": "compiler-message",
                "package_id": format!("path+file://{root}#demo@0.1.0"),
                "manifest_path": format!("{root}\\Cargo.toml"),
                "target": {
                    "name": "demo",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": format!("{root}\\src\\lib.rs"),
                    "edition": "2024"
                },
                "message": {
                    "level": "warning",
                    "message": "unused value",
                    "spans": [{"file_name": "src/lib.rs", "line_start": 1, "line_end": 1, "is_primary": true}]
                }
            }))
        }

        let base_root = r"C:\analysis\base";
        let head_root = r"C:\analysis\head";
        let base = parse_cargo_analysis_with_repo_root(
            Cursor::new(stream(base_root)?),
            Some(Path::new(base_root)),
        )?;
        let head = parse_cargo_analysis_with_repo_root(
            Cursor::new(stream(head_root)?),
            Some(Path::new(head_root)),
        )?;
        let base_producer = &base
            .observations
            .first()
            .ok_or("missing base observation")?
            .producer;
        let head_producer = &head
            .observations
            .first()
            .ok_or("missing head observation")?
            .producer;

        assert_eq!(base_producer, head_producer);
        assert_eq!(
            base_producer.package_id.as_deref(),
            Some("path+file://#demo@0.1.0")
        );
        assert_eq!(base_producer.manifest_path.as_deref(), Some("Cargo.toml"));
        assert_eq!(
            base_producer
                .target
                .as_ref()
                .and_then(|target| target.src_path.as_deref()),
            Some("src/lib.rs")
        );
        Ok(())
    }

    #[test]
    fn runtime_failure_is_explicit_analysis_evidence() {
        let analysis = CargoAnalysis::runtime_failure(vec!["cargo".to_string()], Some(12));
        assert!(analysis.observations.is_empty());
        assert_eq!(
            analysis.execution.completion,
            Some(AnalysisCompletion::RuntimeFailure)
        );
        assert_eq!(analysis.execution.duration_ms, Some(12));
    }

    #[test]
    fn each_compiler_message_is_one_observation() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"{"reason":"compiler-message","package_id":"pkg-a","target":{"name":"a"},"message":{"level":"warning","message":"first"}}
{"reason":"compiler-artifact","package_id":"pkg-a"}
{"reason":"compiler-message","package_id":"pkg-b","target":{"name":"b"},"message":{"level":"error","message":"second"}}
{"reason":"build-finished","success":false}"#;

        let analysis = parse_cargo_analysis(Cursor::new(input))?;
        assert_eq!(analysis.observations.len(), 2);
        assert_eq!(
            analysis.observations[0].producer.package_id.as_deref(),
            Some("pkg-a")
        );
        assert_eq!(
            analysis.observations[1].producer.package_id.as_deref(),
            Some("pkg-b")
        );
        assert_eq!(
            analysis.execution.completion,
            Some(AnalysisCompletion::FailedComplete)
        );
        Ok(())
    }
}
