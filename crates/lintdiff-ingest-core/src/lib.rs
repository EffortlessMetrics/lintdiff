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
    sort_findings, Counts, EffectiveConfig, Finding, GitInfo, HostInfo, LineRange, Location,
    NormPath, Report, RunInfo, Severity, ToolInfo, Verdict, VerdictStatus,
    CHECK_DIAGNOSTICS_ON_DIFF, SCHEMA_ID, TOOL_NAME,
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
    let mut suppressed: u32 = 0;
    let mut denied: u32 = 0;
    let mut filtered_out_by_path: u32 = 0;
    let mut diagnostics_with_spans: u32 = 0;
    let mut diagnostics_with_path_in_diff: u32 = 0;

    // Precompute rename inverse for best-effort matching.
    let mut rename_inverse: BTreeMap<NormPath, NormPath> = BTreeMap::new();
    for (old, new) in diff_map.renames.iter() {
        rename_inverse.insert(new.clone(), old.clone());
    }

    for d in diagnostics.iter() {
        if d.spans.is_empty() {
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

        for sp in spans_to_check {
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

        let Some(sp) = matched_span else {
            continue;
        };
        let path = matched_path.expect("matched_path set when matched_span is set");

        let (code, url) = normalize_diagnostic_code(d.code_raw.as_deref());

        // Code policy
        if !is_code_allowed(&params.config, &code) {
            suppressed += 1;
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

        let fingerprint = Some(fingerprint(&code, location.as_ref(), &d.message));

        matched.push(Finding {
            severity,
            check_id: Some(CHECK_DIAGNOSTICS_ON_DIFF.to_string()),
            code,
            message: d.message.clone(),
            location,
            help: None,
            url,
            fingerprint,
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

    // Deterministic ordering
    sort_findings(&mut matched);

    let total_findings = matched.len() as u32;

    // Stable truncation (receipt)
    let mut truncated = false;
    let cap = params.config.max_findings;
    if matched.len() > cap {
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
    }
}
