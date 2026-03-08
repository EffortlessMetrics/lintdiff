//! Core ingest pipeline for transforming normalized diagnostics and a parsed diff
//! into a stable lintdiff report.

use std::collections::BTreeMap;

use lintdiff_diagnostics::{Diagnostic, Span};
use lintdiff_diff::DiffMap;
use lintdiff_match::{compile_filters, path_allowed, relativize_span_path, select_spans};
use lintdiff_policy::{
    compute_verdict, counts_from_findings, fingerprint, format_level, is_code_allowed,
    map_level_to_severity, normalize_diagnostic_code,
};
use lintdiff_types::{
    sort_findings, sort_findings_cmp, Counts, DiagnosticDisposition, Disposition, EffectiveConfig,
    ExplainSummary, Finding, GitInfo, HostInfo, LineRange, Location, NormPath, Report, RunInfo,
    Severity, ToolInfo, Verdict, VerdictStatus, CHECK_DIAGNOSTICS_ON_DIFF, SCHEMA_ID, TOOL_NAME,
};
use serde_json::json;

#[derive(Clone, Debug)]
pub struct IngestOnDiffParams {
    pub tool: ToolInfo,
    pub run: RunInfo,
    pub host: Option<HostInfo>,
    pub git: Option<GitInfo>,

    pub diff_map: Option<DiffMap>,
    pub diagnostics: Option<Vec<Diagnostic>>,

    pub repo_root: Option<NormPath>,
    pub config: EffectiveConfig,

    /// Optional command line used to generate the inputs (rendered into markdown/data).
    pub repro: Option<String>,
}

/// Create a valid lintdiff receipt from the provided inputs.
///
/// This function is designed to always return a report, even on missing inputs,
/// so the caller can reason about `skip` explicitly.
pub fn ingest_on_diff(params: IngestOnDiffParams) -> Report {
    if params.tool.name != TOOL_NAME {
        // Keep tool.name stable; if a caller uses a different name, we still emit lintdiff.
    }

    let mut run = params.run;
    run.host = params.host;
    run.git = params.git;

    let mut report = Report {
        schema: SCHEMA_ID.to_string(),
        tool: ToolInfo {
            name: TOOL_NAME.to_string(),
            version: params.tool.version,
            commit: params.tool.commit,
        },
        run,
        verdict: Verdict {
            status: VerdictStatus::Skip,
            counts: Counts::default(),
            reasons: vec![],
        },
        findings: vec![],
        data: None,
    };

    let Some(diff_map) = params.diff_map else {
        report.verdict.status = VerdictStatus::Fail;
        report.verdict.reasons.push("missing_diff".to_string());
        report.findings.push(tool_error_finding(
            "lintdiff.input.missing_diff",
            "missing diff input: provide --base/--head or --diff-file",
        ));
        report.data = Some(json!({
            "repro": params.repro,
        }));
        return finalize(report, &params.config);
    };

    let Some(diagnostics) = params.diagnostics else {
        report.verdict.status = VerdictStatus::Skip;
        report
            .verdict
            .reasons
            .push("missing_diagnostics".to_string());
        report.data = Some(json!({
            "repro": params.repro,
            "stats": {
                "diff_files": diff_map.stats.files,
                "diff_hunks": diff_map.stats.hunks,
                "diff_added_lines": diff_map.stats.added_lines,
            }
        }));
        return finalize(report, &params.config);
    };

    let filters = compile_filters(&params.config);

    let mut matched: Vec<Finding> = Vec::new();
    let mut matched_explain_indices: Vec<usize> = Vec::new();
    let mut suppressed: u32 = 0;
    let mut denied: u32 = 0;
    let mut filtered_out_by_path: u32 = 0;
    let mut diagnostics_with_spans: u32 = 0;
    let mut diagnostics_with_path_in_diff: u32 = 0;
    let mut explain: Vec<DiagnosticDisposition> = Vec::new();

    // Precompute rename inverse for best-effort matching.
    let mut rename_inverse: BTreeMap<NormPath, NormPath> = BTreeMap::new();
    for (old, new) in diff_map.renames.iter() {
        rename_inverse.insert(new.clone(), old.clone());
    }

    for d in diagnostics.iter() {
        let (code_for_explain, _) = normalize_diagnostic_code(d.code_raw.as_deref());
        let message_preview = truncate_message(&d.message, 120);

        if d.spans.is_empty() {
            explain.push(DiagnosticDisposition {
                code: code_for_explain,
                message_preview,
                file: None,
                line: None,
                disposition: Disposition::DroppedNoSpan,
                fingerprint: None,
            });
            continue;
        }
        diagnostics_with_spans += 1;

        let spans_to_check = if params.config.feature_flags.prefer_primary_spans {
            select_spans(&d.spans)
        } else {
            d.spans.clone()
        };

        // Find first matching span.
        let mut matched_span: Option<Span> = None;
        let mut matched_path: Option<NormPath> = None;
        let mut found_path_in_diff = false;
        let mut was_filtered_by_path = false;

        for sp in &spans_to_check {
            let rel = relativize_span_path(
                &sp.file,
                params.repo_root.as_ref(),
                params.config.workspace_only,
            );

            let Some(rel_file) = rel else {
                continue;
            };

            // Track whether any diagnostic path exists in the diff regardless of
            // path-filter feature behavior. This avoids false path-mismatch warnings
            // when diagnostics are intentionally excluded by filter rules.
            let ranges = diff_map.changed.get(&rel_file).or_else(|| {
                rename_inverse
                    .get(&rel_file)
                    .and_then(|old| diff_map.changed.get(old))
            });
            if ranges.is_some() {
                found_path_in_diff = true;
            }

            if params.config.feature_flags.path_filters
                && !path_allowed(&filters, rel_file.as_str())
            {
                filtered_out_by_path += 1;
                was_filtered_by_path = true;
                continue;
            }

            let Some(ranges) = ranges else {
                continue;
            };

            let span_range = LineRange::new(sp.line_start, sp.line_end.max(sp.line_start));
            if ranges.iter().any(|r| r.intersects(&span_range)) {
                matched_span = Some(sp.clone());
                matched_path = Some(rel_file);
                break;
            }
        }

        if found_path_in_diff {
            diagnostics_with_path_in_diff += 1;
        }

        // Determine explain file/line from first available span.
        let explain_file = spans_to_check.first().and_then(|sp| {
            relativize_span_path(
                &sp.file,
                params.repo_root.as_ref(),
                params.config.workspace_only,
            )
            .map(|p| p.as_str().to_string())
        });
        let explain_line = spans_to_check.first().map(|sp| sp.line_start);

        let Some(sp) = matched_span else {
            let disposition = if was_filtered_by_path {
                Disposition::DroppedByPathFilter
            } else {
                Disposition::DroppedOutsideDiff
            };
            explain.push(DiagnosticDisposition {
                code: code_for_explain,
                message_preview,
                file: explain_file,
                line: explain_line,
                disposition,
                fingerprint: None,
            });
            continue;
        };
        let path = matched_path.expect("matched_path set when matched_span is set");

        let (code, url) = normalize_diagnostic_code(d.code_raw.as_deref());

        // Code policy
        if !is_code_allowed(&params.config, &code) {
            suppressed += 1;
            explain.push(DiagnosticDisposition {
                code: code.clone(),
                message_preview,
                file: Some(path.as_str().to_string()),
                line: Some(sp.line_start),
                disposition: Disposition::SuppressedByCode,
                fingerprint: None,
            });
            continue;
        }
        let mut severity = map_level_to_severity(&d.level);

        if params.config.filter.deny_codes.iter().any(|c| c == &code) {
            // Deny list upgrades to error regardless of original level.
            denied += 1;
            severity = Severity::Error;
        }

        let location = Some(Location {
            path: path.clone(),
            line: Some(sp.line_start),
            col: sp.col_start,
        });

        let fp = Some(fingerprint(&code, location.as_ref(), &d.message));

        explain.push(DiagnosticDisposition {
            code: code.clone(),
            message_preview,
            file: Some(path.as_str().to_string()),
            line: Some(sp.line_start),
            disposition: Disposition::Included,
            fingerprint: fp.clone(),
        });
        matched_explain_indices.push(explain.len() - 1);

        matched.push(Finding {
            severity,
            check_id: Some(CHECK_DIAGNOSTICS_ON_DIFF.to_string()),
            code,
            message: d.message.clone(),
            location,
            help: None,
            url,
            fingerprint: fp,
            data: Some(json!({
                "code_raw": d.code_raw,
                "level": format_level(&d.level),
                "matched_span": {
                    "file": path.as_str(),
                    "line_start": sp.line_start,
                    "line_end": sp.line_end,
                    "col_start": sp.col_start,
                    "col_end": sp.col_end,
                    "is_primary": sp.is_primary,
                }
            })),
        });
    }

    // Deterministic ordering — paired sort keeps explain indices aligned
    {
        let mut paired: Vec<(Finding, usize)> =
            matched.into_iter().zip(matched_explain_indices).collect();
        paired.sort_by(|(a, _), (b, _)| sort_findings_cmp(a, b));
        let (sorted_matched, sorted_indices): (Vec<_>, Vec<_>) = paired.into_iter().unzip();
        matched = sorted_matched;
        matched_explain_indices = sorted_indices;
    }

    let total_findings = matched.len() as u32;

    // Stable truncation (receipt)
    let mut truncated = false;
    let cap = params.config.max_findings;
    if matched.len() > cap {
        for &idx in &matched_explain_indices[cap..] {
            explain[idx].disposition = Disposition::CutByBudget;
        }
        matched.truncate(cap);
        truncated = true;
    }

    report.findings = matched;

    // Counts
    report.verdict.counts = counts_from_findings(&report.findings);

    // Verdict
    report.verdict = compute_verdict(&params.config, &report.findings, suppressed, denied);

    // Path mismatch hint: diagnostics present, diff present, nothing matched, AND
    // none of the diagnostic paths were found in the diff. This suggests a path
    // normalization issue rather than legitimate "no overlap" scenario.
    // If diagnostics had paths in the diff but lines didn't overlap, that's expected
    // behavior (pass), not a warning.
    if total_findings == 0
        && diagnostics_with_spans > 0
        && !diff_map.changed.is_empty()
        && diagnostics_with_path_in_diff == 0
    {
        report.verdict.status = VerdictStatus::Warn;
        report
            .verdict
            .reasons
            .push("path_mismatch_or_no_matches".to_string());
        report.findings.push(Finding {
            severity: Severity::Info,
            check_id: Some("lintdiff.matching".to_string()),
            code: "lintdiff.matching.no_matches".to_string(),
            message: "No diagnostics matched changed lines. If this is unexpected, check path normalization (absolute vs repo-relative) and ensure your diff is base..head for the same workspace.".to_string(),
            location: None,
            help: Some("Try: git diff --unified=0 <base>..<head> and ensure cargo emits spans under the repo root.".to_string()),
            url: None,
            fingerprint: None,
            data: None,
        });
        sort_findings(&mut report.findings);
    }

    // Build explain summary
    let explain_summary = build_explain_summary(&explain);

    // Data payload (tool-specific)
    report.data = Some(json!({
        "repro": params.repro,
        "stats": {
            "diagnostics_total": diagnostics.len(),
            "diagnostics_with_spans": diagnostics_with_spans,
            "diagnostics_with_path_in_diff": diagnostics_with_path_in_diff,
            "matched_findings_total": total_findings,
            "matched_findings_emitted": report.findings.len(),
            "suppressed_by_code": suppressed,
            "denied_by_code": denied,
            "filtered_out_by_path": filtered_out_by_path,
            "diff_files": diff_map.stats.files,
            "diff_hunks": diff_map.stats.hunks,
            "diff_added_lines": diff_map.stats.added_lines,
        },
        "truncated": truncated,
        "truncated_findings": if truncated { total_findings.saturating_sub(report.findings.len() as u32) } else { 0 },
        "explain": explain,
        "explain_summary": explain_summary,
    }));

    finalize(report, &params.config)
}

fn finalize(mut report: Report, _cfg: &EffectiveConfig) -> Report {
    // Ensure the receipt is always schema-valid.
    if report.schema.is_empty() {
        report.schema = SCHEMA_ID.to_string();
    }
    if report.tool.name.is_empty() {
        report.tool.name = TOOL_NAME.to_string();
    }
    report
}

fn build_explain_summary(explain: &[DiagnosticDisposition]) -> ExplainSummary {
    let mut summary = ExplainSummary {
        total: explain.len() as u32,
        ..Default::default()
    };
    for entry in explain {
        match entry.disposition {
            Disposition::Included => summary.included += 1,
            Disposition::DroppedNoSpan => summary.dropped_no_span += 1,
            Disposition::DroppedOutsideDiff => summary.dropped_outside_diff += 1,
            Disposition::DroppedByPathFilter => summary.dropped_by_path_filter += 1,
            Disposition::SuppressedByCode => summary.suppressed_by_code += 1,
            Disposition::CutByBudget => summary.cut_by_budget += 1,
        }
    }
    summary
}

pub fn truncate_message(msg: &str, max_len: usize) -> String {
    if msg.len() <= max_len {
        msg.to_string()
    } else {
        let end = msg
            .char_indices()
            .map(|(i, _)| i)
            .take_while(|&i| i <= max_len)
            .last()
            .unwrap_or(0);
        let mut s = msg[..end].to_string();
        s.push_str("...");
        s
    }
}

fn tool_error_finding(code: &str, msg: &str) -> Finding {
    Finding {
        severity: Severity::Error,
        check_id: Some("lintdiff.runtime".to_string()),
        code: code.to_string(),
        message: msg.to_string(),
        location: None,
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintdiff_diagnostics::parse_cargo_messages;
    use lintdiff_diff::parse_unified_diff;
    use std::io::Cursor;

    #[test]
    fn end_to_end_match_on_changed_line() {
        let diff = r#"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,1 @@
+fn a() { let x = 1; }
"#;
        let map = parse_unified_diff(diff).unwrap();

        let diag = r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused variable","code":{"code":"clippy::let_unit_value"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":1,"line_end":1,"column_start":10,"column_end":11,"is_primary":true}]}}"#;
        let diags = parse_cargo_messages(Cursor::new(diag)).unwrap();

        let cfg = lintdiff_types::LintdiffConfig::default().effective();

        let report = ingest_on_diff(IngestOnDiffParams {
            tool: ToolInfo {
                name: TOOL_NAME.to_string(),
                version: "0.1.0".to_string(),
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
            diff_map: Some(map),
            diagnostics: Some(diags),
            repo_root: Some(NormPath::new("/repo")),
            config: cfg,
            repro: None,
        });

        assert_eq!(report.verdict.counts.warn, 1);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].location.as_ref().unwrap().path.as_str(),
            "src/lib.rs"
        );

        // Verify explain artifact
        let data = report.data.as_ref().unwrap();
        let explain = data.get("explain").unwrap().as_array().unwrap();
        assert_eq!(explain.len(), 1);
        assert_eq!(explain[0]["disposition"], "included");
        assert!(explain[0]["fingerprint"].is_string());
    }

    #[test]
    fn explain_tracks_no_span_diagnostics() {
        let diff = "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,0 +1,1 @@\n+fn a() {}\n";
        let map = parse_unified_diff(diff).unwrap();

        let jsonl = r#"{"reason":"compiler-message","message":{"level":"warning","message":"5 warnings emitted","code":null,"spans":[]}}
{"reason":"compiler-message","message":{"level":"warning","message":"unused","code":{"code":"clippy::let_unit_value"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":1,"line_end":1,"column_start":1,"column_end":10,"is_primary":true}]}}"#;
        let diags = parse_cargo_messages(Cursor::new(jsonl)).unwrap();
        let cfg = lintdiff_types::LintdiffConfig::default().effective();

        let report = ingest_on_diff(IngestOnDiffParams {
            tool: ToolInfo {
                name: TOOL_NAME.to_string(),
                version: "test".to_string(),
                commit: None,
            },
            run: RunInfo {
                started_at: "t0".to_string(),
                ended_at: "t1".to_string(),
                duration_ms: None,
                host: None,
                git: None,
            },
            host: None,
            git: None,
            diff_map: Some(map),
            diagnostics: Some(diags),
            repo_root: Some(NormPath::new("/repo")),
            config: cfg,
            repro: None,
        });

        let data = report.data.as_ref().unwrap();
        let explain = data.get("explain").unwrap().as_array().unwrap();
        assert_eq!(explain.len(), 2);

        let no_span = explain
            .iter()
            .filter(|e| e["disposition"] == "dropped_no_span")
            .count();
        let included = explain
            .iter()
            .filter(|e| e["disposition"] == "included")
            .count();
        assert_eq!(no_span, 1);
        assert_eq!(included, 1);

        // Verify summary
        let summary = data.get("explain_summary").unwrap();
        assert_eq!(summary["total"], 2);
        assert_eq!(summary["included"], 1);
        assert_eq!(summary["dropped_no_span"], 1);
    }

    #[test]
    fn explain_summary_matches_explain_entries() {
        let diff = "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,0 +1,1 @@\n+fn a() {}\n";
        let map = parse_unified_diff(diff).unwrap();

        let jsonl = r#"{"reason":"compiler-message","message":{"level":"warning","message":"outside diff","code":{"code":"clippy::let_unit_value"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":999,"line_end":999,"column_start":1,"column_end":10,"is_primary":true}]}}"#;
        let diags = parse_cargo_messages(Cursor::new(jsonl)).unwrap();
        let cfg = lintdiff_types::LintdiffConfig::default().effective();

        let report = ingest_on_diff(IngestOnDiffParams {
            tool: ToolInfo {
                name: TOOL_NAME.to_string(),
                version: "test".to_string(),
                commit: None,
            },
            run: RunInfo {
                started_at: "t0".to_string(),
                ended_at: "t1".to_string(),
                duration_ms: None,
                host: None,
                git: None,
            },
            host: None,
            git: None,
            diff_map: Some(map),
            diagnostics: Some(diags),
            repo_root: Some(NormPath::new("/repo")),
            config: cfg,
            repro: None,
        });

        let data = report.data.as_ref().unwrap();
        let summary = data.get("explain_summary").unwrap();
        assert_eq!(summary["total"], 1);
        assert_eq!(summary["dropped_outside_diff"], 1);
        assert_eq!(summary["included"], 0);
    }

    #[test]
    fn truncate_message_ascii_within_limit() {
        let msg = "hello world";
        assert_eq!(truncate_message(msg, 20), "hello world");
    }

    #[test]
    fn truncate_message_ascii_at_exact_boundary() {
        let msg = "hello";
        assert_eq!(truncate_message(msg, 5), "hello");
    }

    #[test]
    fn truncate_message_ascii_over_limit() {
        let msg = "hello world";
        assert_eq!(truncate_message(msg, 5), "hello...");
    }

    #[test]
    fn truncate_message_unicode_at_multibyte_boundary() {
        // "aé" = 'a' (1 byte) + 'é' (2 bytes) = 3 bytes total
        // With max_len=2, slicing at byte 2 would be inside 'é'.
        // Should truncate to "a" (byte index 1) instead of panicking.
        let msg = "aéb";
        let result = truncate_message(msg, 2);
        assert_eq!(result, "a...");
    }

    #[test]
    fn truncate_message_emoji_boundary() {
        // '🦀' is 4 bytes. "x🦀y" = 1 + 4 + 1 = 6 bytes.
        // max_len=3: last char boundary <= 3 is index 1 ('x' ends at 1, '🦀' starts at 1 and ends at 5).
        // So we should get "x..." because the emoji starts at byte 1 but extends to byte 5.
        let msg = "x🦀y";
        let result = truncate_message(msg, 3);
        assert_eq!(result, "x...");
    }

    #[test]
    fn truncate_message_all_multibyte() {
        // "ééé" = 6 bytes, max_len=4 → last boundary <= 4 is byte 4 ("éé")
        let msg = "ééé";
        let result = truncate_message(msg, 4);
        assert_eq!(result, "éé...");
    }

    #[test]
    fn truncate_message_zero_max_len() {
        let msg = "hello";
        let result = truncate_message(msg, 0);
        assert_eq!(result, "...");
    }

    #[test]
    fn budget_cut_with_duplicate_fingerprints() {
        // Create 3 diagnostics on different lines but same code+message (same fingerprint).
        // Budget = 1. Verify correct explain entries are marked CutByBudget.
        let diff = r#"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,5 @@
+fn a() { let x = 1; }
+fn b() { let y = 2; }
+fn c() { let z = 3; }
+fn d() { let w = 4; }
+fn e() { let v = 5; }
"#;
        let map = parse_unified_diff(diff).unwrap();

        // 3 warnings on lines 1, 2, 3 with same code and message
        let jsonl = r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused variable","code":{"code":"clippy::let_unit_value"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":1,"line_end":1,"column_start":10,"column_end":11,"is_primary":true}]}}
{"reason":"compiler-message","message":{"level":"warning","message":"unused variable","code":{"code":"clippy::let_unit_value"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":2,"line_end":2,"column_start":10,"column_end":11,"is_primary":true}]}}
{"reason":"compiler-message","message":{"level":"warning","message":"unused variable","code":{"code":"clippy::let_unit_value"},"spans":[{"file_name":"/repo/src/lib.rs","line_start":3,"line_end":3,"column_start":10,"column_end":11,"is_primary":true}]}}"#;
        let diags = parse_cargo_messages(Cursor::new(jsonl)).unwrap();

        let mut cfg = lintdiff_types::LintdiffConfig::default();
        cfg.max_findings = Some(1);
        let eff = cfg.effective();

        let report = ingest_on_diff(IngestOnDiffParams {
            tool: ToolInfo {
                name: TOOL_NAME.to_string(),
                version: "test".to_string(),
                commit: None,
            },
            run: RunInfo {
                started_at: "t0".to_string(),
                ended_at: "t1".to_string(),
                duration_ms: None,
                host: None,
                git: None,
            },
            host: None,
            git: None,
            diff_map: Some(map),
            diagnostics: Some(diags),
            repo_root: Some(NormPath::new("/repo")),
            config: eff,
            repro: None,
        });

        // Should have 1 finding (budget=1)
        assert_eq!(report.findings.len(), 1);

        let data = report.data.as_ref().unwrap();
        let explain = data.get("explain").unwrap().as_array().unwrap();
        assert_eq!(explain.len(), 3);

        let included = explain
            .iter()
            .filter(|e| e["disposition"] == "included")
            .count();
        let cut = explain
            .iter()
            .filter(|e| e["disposition"] == "cut_by_budget")
            .count();
        // Exactly 1 included, 2 cut
        assert_eq!(included, 1, "expected 1 included, got {included}");
        assert_eq!(cut, 2, "expected 2 cut_by_budget, got {cut}");

        // Summary should match
        let summary = data.get("explain_summary").unwrap();
        assert_eq!(summary["included"], 1);
        assert_eq!(summary["cut_by_budget"], 2);
    }
}
