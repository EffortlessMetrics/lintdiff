//! Parse cargo `--message-format=json` output into normalized diagnostics.

use std::io::BufRead;

use serde_json::Value;
use thiserror::Error;

use lintdiff_types::NormPath;

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code_raw: Option<String>,
    pub message: String,
    pub spans: Vec<Span>,
    pub rendered: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
    Other(String),
}

#[derive(Clone, Debug)]
pub struct Span {
    pub file: NormPath,
    pub line_start: u32,
    pub line_end: u32,
    pub col_start: Option<u32>,
    pub col_end: Option<u32>,
    pub is_primary: bool,
}

#[derive(Debug, Error)]
pub enum DiagnosticsParseError {
    #[error("invalid json at line {line}: {source}")]
    InvalidJson {
        line: usize,
        #[source]
        source: serde_json::Error,
    },
    #[error("unexpected json shape at line {line}: {msg}")]
    InvalidShape { line: usize, msg: String },
}

/// Parse a cargo JSON-lines stream, returning only compiler messages.
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
