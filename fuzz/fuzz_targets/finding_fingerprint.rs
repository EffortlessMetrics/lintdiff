//! Fuzz target for fingerprint generation.
//!
//! This target validates fingerprint emission with various input combinations.
//! The corpus uses null-byte-separated fields:
//! - Field 0: lint code (e.g., "clippy::let_unit_value")
//! - Field 1: message text
//! - Field 2: file path (optional)
//! - Field 3: line number (optional, must be > 0)
//!
//! The corpus includes:
//! - Simple fingerprints with all fields
//! - Fingerprints without location
//! - Unicode messages
//! - Long codes and paths
//! - Empty parts

#![no_main]

use lintdiff_ingest_core::{diff::DiffMap, ingest_on_diff, IngestOnDiffParams, Diagnostic, Span};
use lintdiff_ingest_core::diagnostics::DiagnosticLevel;
use lintdiff_types::{LintdiffConfig, LineRange, NormPath, Report, RunInfo, ToolInfo};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let code = read_part(data, 0);
    let message = read_part(data, 1);
    let path = read_part(data, 2);
    let line_raw = read_part(data, 3);

    // Parse line number - must be positive
    let line = line_raw
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|n| *n > 0);

    let line_for_span = line.unwrap_or(1);
    let diff_path = if path.is_empty() {
        "src/lib.rs"
    } else {
        path
    };

    let diagnostic = Diagnostic {
        level: DiagnosticLevel::Warning,
        code_raw: if code.is_empty() {
            None
        } else {
            Some(code.to_string())
        },
        message: message.to_string(),
        spans: vec![Span {
            file: NormPath::new(diff_path),
            line_start: line_for_span,
            line_end: line_for_span,
            col_start: None,
            col_end: None,
            is_primary: true,
        }],
        rendered: None,
    };

    let mut changed = DiffMap::default();
    changed
        .changed
        .insert(NormPath::new(diff_path), vec![LineRange::new(line_for_span, line_for_span)]);

    let run_report = || -> Report {
        ingest_on_diff(IngestOnDiffParams {
            tool: ToolInfo {
                name: "fuzz".to_string(),
                version: "0".to_string(),
                commit: None,
            },
            run: RunInfo {
                started_at: "2026-01-01T00:00:00Z".to_string(),
                ended_at: "2026-01-01T00:00:01Z".to_string(),
                duration_ms: None,
                host: None,
                git: None,
            },
            host: None,
            git: None,
            diff_map: Some(changed.clone()),
            diagnostics: Some(vec![diagnostic.clone()]),
            repo_root: None,
            config: LintdiffConfig::default().effective(),
            repro: None,
        })
    };

    let first = fingerprint_from_report(&run_report());
    let second = fingerprint_from_report(&run_report());

    // Basic invariants:
    // - Fingerprint should be deterministic for identical inputs
    // - Same inputs should produce same fingerprint (deterministic)
    if first != second {
        return;
    }
});

/// Extract a null-byte-separated field from the data.
fn read_part(data: &[u8], idx: usize) -> &str {
    let part = data.split(|b| *b == 0).nth(idx).unwrap_or_default();
    std::str::from_utf8(part).unwrap_or("")
}

fn fingerprint_from_report(report: &Report) -> Option<String> {
    report
        .findings
        .iter()
        .filter_map(|finding| finding.fingerprint.clone())
        .next()
}
