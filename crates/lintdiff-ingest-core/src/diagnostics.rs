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
//! use lintdiff_ingest_core::diagnostics::{parse_cargo_messages, DiagnosticLevel};
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

use std::io::BufRead;

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
/// use lintdiff_ingest_core::diagnostics::{Diagnostic, DiagnosticLevel, Span};
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
#[derive(Clone, Debug)]
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
/// use lintdiff_ingest_core::diagnostics::DiagnosticLevel;
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
/// use lintdiff_ingest_core::Span;
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
#[derive(Clone, Debug)]
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
/// use lintdiff_ingest_core::diagnostics::{parse_cargo_messages, DiagnosticsParseError};
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
/// use lintdiff_ingest_core::diagnostics::{parse_cargo_messages, DiagnosticLevel};
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
    let mut out: Vec<Diagnostic> = Vec::new();

    for (idx, line_res) in reader.lines().enumerate() {
        let line_no = idx + 1;
        let line = line_res.map_err(|e| DiagnosticsParseError::InvalidShape {
            line: line_no,
            msg: format!("io error reading diagnostics stream: {e}"),
        })?;

        if line.trim().is_empty() {
            continue;
        }

        let v: Value =
            serde_json::from_str(&line).map_err(|e| DiagnosticsParseError::InvalidJson {
                line: line_no,
                source: e,
            })?;

        // cargo messages are objects with "reason"
        let reason = v.get("reason").and_then(|x| x.as_str());
        if reason != Some("compiler-message") {
            continue;
        }

        let msg = v
            .get("message")
            .ok_or_else(|| DiagnosticsParseError::InvalidShape {
                line: line_no,
                msg: "missing 'message' field".to_string(),
            })?;

        let level_raw = msg.get("level").and_then(|x| x.as_str()).unwrap_or("other");
        let level = match level_raw {
            "error" => DiagnosticLevel::Error,
            "warning" => DiagnosticLevel::Warning,
            "note" => DiagnosticLevel::Note,
            "help" => DiagnosticLevel::Help,
            other => DiagnosticLevel::Other(other.to_string()),
        };

        let message = msg
            .get("message")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();

        let rendered = msg
            .get("rendered")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());

        let code_raw = msg
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());

        let spans_val = msg
            .get("spans")
            .and_then(|x| x.as_array())
            .cloned()
            .unwrap_or_default();
        let mut spans: Vec<Span> = Vec::new();
        for sp in spans_val {
            let file_name = sp.get("file_name").and_then(|x| x.as_str()).unwrap_or("");
            let line_start = sp.get("line_start").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
            let line_end = sp
                .get("line_end")
                .and_then(|x| x.as_u64())
                .unwrap_or(line_start as u64) as u32;
            let col_start = sp
                .get("column_start")
                .and_then(|x| x.as_u64())
                .map(|n| n as u32);
            let col_end = sp
                .get("column_end")
                .and_then(|x| x.as_u64())
                .map(|n| n as u32);
            let is_primary = sp
                .get("is_primary")
                .and_then(|x| x.as_bool())
                .unwrap_or(false);

            // rustc uses 1-based lines/cols; if missing, keep 0 but avoid underflow.
            let ls = line_start.max(1);
            let le = line_end.max(ls);

            spans.push(Span {
                file: NormPath::new(file_name),
                line_start: ls,
                line_end: le,
                col_start,
                col_end,
                is_primary,
            });
        }

        out.push(Diagnostic {
            level,
            code_raw,
            message,
            spans,
            rendered,
        });
    }

    Ok(out)
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
}
