//! Determinism snapshot tests for `ingest_on_diff`.
//!
//! Each test constructs fixed inputs, runs the ingest pipeline, serialises the
//! resulting report to pretty-printed JSON, and compares it byte-for-byte against
//! a stored golden file in `tests/snapshots/`.
//!
//! On first run (golden file missing) the test writes the snapshot and skips the
//! assertion so CI will pass once and then guard against regressions thereafter.

use std::io::Cursor;

use lintdiff_engine::{ingest_on_diff, IngestOnDiffParams};
use lintdiff_engine::{parse_cargo_messages, parse_unified_diff};
use lintdiff_types::{LintdiffConfig, NormPath, RunInfo, ToolInfo, TOOL_NAME};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn snapshot_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
}

fn deterministic_tool() -> ToolInfo {
    ToolInfo {
        name: TOOL_NAME.to_string(),
        version: "0.1.0".to_string(),
        commit: None,
    }
}

fn deterministic_run() -> RunInfo {
    RunInfo {
        started_at: "2026-01-01T00:00:00Z".to_string(),
        ended_at: "2026-01-01T00:00:01Z".to_string(),
        duration_ms: None,
        host: None,
        git: None,
    }
}

/// Compare `actual` JSON against the golden file at `name`.
///
/// If the golden file does not exist, write it and print a notice instead of
/// failing, so the very first `cargo test` run bootstraps the snapshots.
fn assert_snapshot(name: &str, actual: &str) {
    let path = snapshot_dir().join(format!("{name}.json"));

    if !path.exists() {
        std::fs::create_dir_all(path.parent().unwrap()).expect("create snapshot dir");
        std::fs::write(&path, actual).expect("write initial snapshot");
        eprintln!(
            "[ snapshot ] wrote new golden file: {}  — re-run to verify",
            path.display()
        );
        return;
    }

    let expected = std::fs::read_to_string(&path).expect("read golden file");

    if actual != expected {
        // Print a useful diff: show first divergent line.
        let mut first_diff_line = None;
        for (i, (a, e)) in actual.lines().zip(expected.lines()).enumerate() {
            if a != e {
                first_diff_line = Some((i + 1, a.to_string(), e.to_string()));
                break;
            }
        }
        if let Some((line, got, want)) = first_diff_line {
            panic!(
                "snapshot mismatch in {name}.json at line {line}:\n  expected: {want}\n       got: {got}\n\n\
                 Full golden file: {}\n\
                 To update, delete the golden file and re-run tests.",
                path.display()
            );
        } else {
            panic!(
                "snapshot mismatch in {name}.json (different number of lines).\n\
                 expected length: {} bytes\n\
                 actual length:   {} bytes\n\n\
                 Full golden file: {}\n\
                 To update, delete the golden file and re-run tests.",
                expected.len(),
                actual.len(),
                path.display()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Test 1: pass verdict — diagnostics exist but none match changed lines
// ---------------------------------------------------------------------------

#[test]
fn snapshot_pass_verdict() {
    let diff = "\
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,1 @@
+fn a() { let x = 1; }
";
    let diff_map = parse_unified_diff(diff).expect("valid diff");

    // Warning on line 999 — well outside the changed range (line 1).
    let diag_jsonl = r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused variable","code":{"code":"clippy::let_unit_value"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":999,"line_end":999,"column_start":10,"column_end":11,"is_primary":true}]}}"#;
    let diagnostics = parse_cargo_messages(Cursor::new(diag_jsonl)).expect("valid diagnostics");

    let cfg = LintdiffConfig::default().effective();

    let report = ingest_on_diff(IngestOnDiffParams {
        tool: deterministic_tool(),
        run: deterministic_run(),
        host: None,
        git: None,
        diff_map: Some(diff_map),
        diagnostics: Some(diagnostics),
        analysis: None,
        repo_root: Some(NormPath::new("/repo")),
        config: cfg,
        repro: None,
    });

    let json = serde_json::to_string_pretty(&report).expect("serialize report");
    assert_snapshot("pass_verdict", &json);
}

// ---------------------------------------------------------------------------
// Test 2: warn verdict — warning matches a changed line
// ---------------------------------------------------------------------------

#[test]
fn snapshot_warn_verdict() {
    let diff = "\
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,1 @@
+fn a() { let x = 1; }
";
    let diff_map = parse_unified_diff(diff).expect("valid diff");

    // Warning on line 1 — inside the changed range.
    let diag_jsonl = r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused variable","code":{"code":"clippy::let_unit_value"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":1,"line_end":1,"column_start":10,"column_end":11,"is_primary":true}]}}"#;
    let diagnostics = parse_cargo_messages(Cursor::new(diag_jsonl)).expect("valid diagnostics");

    let cfg = LintdiffConfig::default().effective();

    let report = ingest_on_diff(IngestOnDiffParams {
        tool: deterministic_tool(),
        run: deterministic_run(),
        host: None,
        git: None,
        diff_map: Some(diff_map),
        diagnostics: Some(diagnostics),
        analysis: None,
        repo_root: Some(NormPath::new("/repo")),
        config: cfg,
        repro: None,
    });

    let json = serde_json::to_string_pretty(&report).expect("serialize report");
    assert_snapshot("warn_verdict", &json);
}

// ---------------------------------------------------------------------------
// Test 3: fail verdict — warning on changed line + deny code upgrades to error
// ---------------------------------------------------------------------------

#[test]
fn snapshot_fail_verdict() {
    let diff = "\
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,1 @@
+fn a() { let x = 1; }
";
    let diff_map = parse_unified_diff(diff).expect("valid diff");

    // Warning on line 1 with code clippy::let_unit_value.
    let diag_jsonl = r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused variable","code":{"code":"clippy::let_unit_value"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":1,"line_end":1,"column_start":10,"column_end":11,"is_primary":true}]}}"#;
    let diagnostics = parse_cargo_messages(Cursor::new(diag_jsonl)).expect("valid diagnostics");

    // Deny the normalized code so it upgrades severity to error → fail verdict.
    let mut user_cfg = LintdiffConfig::default();
    user_cfg.filter.deny_codes = vec!["lintdiff.diagnostic.clippy.let_unit_value".to_string()];
    let cfg = user_cfg.effective();

    let report = ingest_on_diff(IngestOnDiffParams {
        tool: deterministic_tool(),
        run: deterministic_run(),
        host: None,
        git: None,
        diff_map: Some(diff_map),
        diagnostics: Some(diagnostics),
        analysis: None,
        repo_root: Some(NormPath::new("/repo")),
        config: cfg,
        repro: None,
    });

    let json = serde_json::to_string_pretty(&report).expect("serialize report");
    assert_snapshot("fail_verdict", &json);
}

#[test]
fn permuted_diagnostic_input_produces_identical_report_bytes() {
    let diff = "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,0 +1,2 @@\n+fn a() { let x = 1; }\n+fn b() { let y = 2; }\n";
    let diff_map = parse_unified_diff(diff).expect("valid diff");
    let jsonl = r#"{"reason":"compiler-message","package_id":"pkg-a","target":{"name":"a"},"message":{"level":"warning","message":"first","code":{"code":"clippy::first"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":1,"line_end":1,"is_primary":true}]}}
{"reason":"compiler-message","package_id":"pkg-b","target":{"name":"b"},"message":{"level":"warning","message":"second","code":{"code":"clippy::second"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":2,"line_end":2,"is_primary":true}]}}"#;
    let diagnostics = parse_cargo_messages(Cursor::new(jsonl)).expect("valid diagnostics");
    let mut reversed = diagnostics.clone();
    reversed.reverse();
    let cfg = LintdiffConfig::default().effective();

    let report = |diagnostics| {
        ingest_on_diff(IngestOnDiffParams {
            tool: deterministic_tool(),
            run: deterministic_run(),
            host: None,
            git: None,
            diff_map: Some(diff_map.clone()),
            diagnostics: Some(diagnostics),
            analysis: None,
            repo_root: Some(NormPath::new("/repo")),
            config: cfg.clone(),
            repro: None,
        })
    };

    let first = serde_json::to_vec(&report(diagnostics)).expect("serialize first report");
    let second = serde_json::to_vec(&report(reversed)).expect("serialize second report");
    assert_eq!(first, second);
}
