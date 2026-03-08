use std::io::Cursor;

use lintdiff_diagnostics::parse_cargo_messages;
use lintdiff_diff::parse_unified_diff;
use lintdiff_ingest_core::{ingest_on_diff, IngestOnDiffParams};
use lintdiff_types::{LintdiffConfig, NormPath, RunInfo, ToolInfo, TOOL_NAME};

#[test]
fn equivalent_message_whitespace_produces_same_fingerprint() {
    let diff = r#"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,1 @@
+fn a() {}
"#;
    let diagnostics = r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused variable","code":{"code":"clippy::let_unit_value"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":1,"line_end":1,"column_start":1,"column_end":10,"is_primary":true}]}}
{"reason":"compiler-message","message":{"level":"warning","message":"  unused   variable\t","code":{"code":"clippy::let_unit_value"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":1,"line_end":1,"column_start":1,"column_end":10,"is_primary":true}]}}"#;

    let diff_map = parse_unified_diff(diff).expect("valid unified diff fixture");
    let diags = parse_cargo_messages(Cursor::new(diagnostics)).expect("valid diagnostics fixture");
    let cfg = LintdiffConfig::default().effective();

    let report = ingest_on_diff(IngestOnDiffParams {
        tool: ToolInfo {
            name: TOOL_NAME.to_string(),
            version: "test".to_string(),
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
        diff_map: Some(diff_map),
        diagnostics: Some(diags),
        repo_root: Some(NormPath::new("/repo")),
        config: cfg,
        repro: None,
    });

    assert_eq!(
        report.findings.len(),
        2,
        "expected both diagnostics to match"
    );
    let a = report.findings[0]
        .fingerprint
        .as_ref()
        .expect("finding 0 fingerprint");
    let b = report.findings[1]
        .fingerprint
        .as_ref()
        .expect("finding 1 fingerprint");
    assert_eq!(a, b);
}
