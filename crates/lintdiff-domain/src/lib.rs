//! Pure(ish) core: matching diagnostics to diff ranges and producing a receipt.

use std::collections::{BTreeMap, BTreeSet};

use globset::{Glob, GlobSet, GlobSetBuilder};
use lintdiff_diagnostics::{Diagnostic, DiagnosticLevel, Span};
use lintdiff_diff::DiffMap;
use lintdiff_types::{
    sort_findings, Counts, EffectiveConfig, Finding, GitInfo, HostInfo, LineRange, Location, NormPath,
    Report, RunInfo, Severity, ToolInfo, Verdict, VerdictStatus, CHECK_DIAGNOSTICS_ON_DIFF, SCHEMA_ID,
    TOOL_NAME,
};
use serde_json::json;
use sha2::{Digest, Sha256};

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
/// so the director can reason about "skip" explicitly.
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
        report.verdict.reasons.push("missing_diagnostics".to_string());
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

        let spans_to_check = select_spans(&d.spans);

        // Find first matching span.
        let mut matched_span: Option<Span> = None;
        let mut matched_path: Option<NormPath> = None;

        for sp in spans_to_check {
            let rel = relativize_span_path(&sp.file, params.repo_root.as_ref(), params.config.workspace_only);

            let Some(rel_file) = rel else {
                continue;
            };

            if !path_allowed(&filters, rel_file.as_str()) {
                filtered_out_by_path += 1;
                continue;
            }

            // match against diff map using new-path; also try rename inverse.
            let ranges = diff_map
                .changed
                .get(&rel_file)
                .or_else(|| rename_inverse.get(&rel_file).and_then(|old| diff_map.changed.get(old)));

            let Some(ranges) = ranges else { continue; };

            let span_range = LineRange::new(sp.line_start, sp.line_end.max(sp.line_start));
            if ranges.iter().any(|r| r.intersects(&span_range)) {
                matched_span = Some(sp.clone());
                matched_path = Some(rel_file);
                break;
            }
        }

        let Some(sp) = matched_span else {
            continue;
        };
        let path = matched_path.expect("matched_path set when matched_span is set");

        let (code, url) = normalize_diagnostic_code(d.code_raw.as_deref());

        // Code policy
        if !code_allowed(&params.config, &code) {
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

    // Path mismatch hint: diagnostics present, diff present, but nothing matched
    if total_findings == 0 && !diagnostics.is_empty() && !diff_map.changed.is_empty() {
        report.verdict.status = VerdictStatus::Warn;
        report.verdict.reasons.push("path_mismatch_or_no_matches".to_string());
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

fn counts_from_findings(findings: &[Finding]) -> Counts {
    let mut c = Counts::default();
    for f in findings {
        match f.severity {
            Severity::Info => c.info += 1,
            Severity::Warn => c.warn += 1,
            Severity::Error => c.error += 1,
        }
    }
    c
}

fn compute_verdict(cfg: &EffectiveConfig, findings: &[Finding], suppressed: u32, denied: u32) -> Verdict {
    let counts = counts_from_findings(findings);

    let has_error = counts.error > 0;
    let has_warn = counts.warn > 0;

    let mut reasons: Vec<String> = Vec::new();
    if suppressed > 0 {
        reasons.push("suppressed".to_string());
    }
    if denied > 0 {
        reasons.push("deny_list".to_string());
    }

    let status = match cfg.fail_on {
        lintdiff_types::FailOn::Error => {
            if has_error {
                VerdictStatus::Fail
            } else if has_warn {
                VerdictStatus::Warn
            } else {
                VerdictStatus::Pass
            }
        }
        lintdiff_types::FailOn::Warn => {
            if has_error || has_warn {
                VerdictStatus::Fail
            } else {
                VerdictStatus::Pass
            }
        }
        lintdiff_types::FailOn::Never => {
            if has_error || has_warn {
                VerdictStatus::Warn
            } else {
                VerdictStatus::Pass
            }
        }
    };

    Verdict { status, counts, reasons }
}

fn map_level_to_severity(level: &DiagnosticLevel) -> Severity {
    match level {
        DiagnosticLevel::Error => Severity::Error,
        DiagnosticLevel::Warning => Severity::Warn,
        DiagnosticLevel::Note | DiagnosticLevel::Help => Severity::Info,
        DiagnosticLevel::Other(_) => Severity::Info,
    }
}

fn format_level(level: &DiagnosticLevel) -> String {
    match level {
        DiagnosticLevel::Error => "error".to_string(),
        DiagnosticLevel::Warning => "warning".to_string(),
        DiagnosticLevel::Note => "note".to_string(),
        DiagnosticLevel::Help => "help".to_string(),
        DiagnosticLevel::Other(s) => s.clone(),
    }
}

fn normalize_diagnostic_code(raw: Option<&str>) -> (String, Option<String>) {
    let Some(raw) = raw else {
        return ("lintdiff.diagnostic.unknown".to_string(), None);
    };

    if raw.starts_with("clippy::") {
        let name = raw.trim_start_matches("clippy::");
        let slug = slugify(name);
        let url = Some(format!("https://rust-lang.github.io/rust-clippy/master/index.html#{}", slug));
        return (format!("lintdiff.diagnostic.clippy.{slug}"), url);
    }

    if is_rustc_error_code(raw) {
        let url = Some(format!("https://doc.rust-lang.org/error_codes/{raw}.html"));
        return (format!("lintdiff.diagnostic.rustc.{raw}"), url);
    }

    // rustc warnings are often lint names (e.g. dead_code)
    let slug = slugify(raw);
    (format!("lintdiff.diagnostic.rustc_lint.{slug}"), None)
}

fn is_rustc_error_code(raw: &str) -> bool {
    let b = raw.as_bytes();
    if b.len() != 5 || b[0] != b'E' {
        return false;
    }
    b[1..].iter().all(|c| c.is_ascii_digit())
}

fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else if ch == ':' {
            // clippy::foo => foo; already stripped, but keep safe.
            out.push('.');
        } else {
            out.push('_');
        }
    }
    while out.contains("..") {
        out = out.replace("..", ".");
    }
    out.trim_matches('.').to_string()
}

fn fingerprint(code: &str, loc: Option<&Location>, msg: &str) -> String {
    let mut h = Sha256::new();
    h.update(code.as_bytes());
    h.update(b"|");
    if let Some(loc) = loc {
        h.update(loc.path.as_str().as_bytes());
        h.update(b":");
        if let Some(line) = loc.line {
            h.update(line.to_string().as_bytes());
        }
        h.update(b":");
    }
    h.update(normalize_message(msg).as_bytes());
    hex::encode(h.finalize())
}

fn normalize_message(msg: &str) -> String {
    // Deterministic-ish: trim and collapse whitespace.
    let mut out = String::new();
    let mut prev_ws = false;
    for ch in msg.trim().chars() {
        let ws = ch.is_whitespace();
        if ws {
            if !prev_ws {
                out.push(' ');
            }
            prev_ws = true;
        } else {
            out.push(ch);
            prev_ws = false;
        }
    }
    out
}

fn select_spans(spans: &[Span]) -> Vec<Span> {
    let prim: Vec<Span> = spans.iter().filter(|s| s.is_primary).cloned().collect();
    if !prim.is_empty() {
        prim
    } else {
        spans.to_vec()
    }
}

fn relativize_span_path(file: &NormPath, repo_root: Option<&NormPath>, workspace_only: bool) -> Option<NormPath> {
    let s = file.as_str();

    // Already repo-relative (best effort): doesn't look absolute.
    if !looks_absolute(s) {
        return Some(NormPath::new(s));
    }

    let Some(root) = repo_root else {
        return if workspace_only { None } else { Some(NormPath::new(s)) };
    };

    let root_s = root.as_str().trim_end_matches('/');
    if let Some(stripped) = s.strip_prefix(root_s) {
        let stripped = stripped.trim_start_matches('/');
        if stripped.is_empty() {
            return if workspace_only { None } else { Some(NormPath::new(s)) };
        }
        return Some(NormPath::new(stripped));
    }

    if workspace_only {
        None
    } else {
        Some(NormPath::new(s))
    }
}

fn looks_absolute(s: &str) -> bool {
    s.starts_with('/') || (s.len() >= 3 && s.as_bytes()[1] == b':' && s.as_bytes()[2] == b'/')
}

struct Filters {
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
}

fn compile_filters(cfg: &EffectiveConfig) -> Filters {
    Filters {
        include: build_globset(&cfg.filter.include_paths),
        exclude: build_globset(&cfg.filter.exclude_paths),
    }
}

fn build_globset(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        if let Ok(g) = Glob::new(p) {
            b.add(g);
        }
    }
    b.build().ok()
}

fn path_allowed(filters: &Filters, path: &str) -> bool {
    if let Some(ex) = &filters.exclude {
        if ex.is_match(path) {
            return false;
        }
    }
    if let Some(inc) = &filters.include {
        return inc.is_match(path);
    }
    true
}

fn code_allowed(cfg: &EffectiveConfig, code: &str) -> bool {
    if !cfg.filter.allow_codes.is_empty() {
        return cfg.filter.allow_codes.iter().any(|c| c == code);
    }
    if cfg.filter.suppress_codes.iter().any(|c| c == code) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintdiff_diff::parse_unified_diff;
    use lintdiff_diagnostics::parse_cargo_messages;
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
            tool: ToolInfo { name: TOOL_NAME.to_string(), version: "0.1.0".to_string(), commit: None },
            run: RunInfo { started_at: "2026-01-01T00:00:00Z".to_string(), ended_at: "2026-01-01T00:00:01Z".to_string(), duration_ms: None, host: None, git: None },
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
        assert_eq!(report.findings[0].location.as_ref().unwrap().path.as_str(), "src/lib.rs");
    }

    #[test]
    fn rustc_error_code_maps_to_doc_url() {
        let (code, url) = normalize_diagnostic_code(Some("E0502"));
        assert_eq!(code, "lintdiff.diagnostic.rustc.E0502");
        assert!(url.unwrap().contains("error_codes/E0502.html"));
    }
}
