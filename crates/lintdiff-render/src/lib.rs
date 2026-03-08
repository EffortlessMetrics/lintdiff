//! Rendering helpers for lintdiff receipts.

use lintdiff_types::{sort_findings, Finding, Report, Severity, VerdictStatus};

pub const DEFAULT_REPORT_PATH: &str = "artifacts/lintdiff/report.json";

#[derive(Clone, Debug)]
pub struct MarkdownOptions {
    pub max_items: usize,
    pub report_path: String,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            max_items: 20,
            report_path: DEFAULT_REPORT_PATH.to_string(),
        }
    }
}

pub fn render_markdown(report: &Report, opts: MarkdownOptions) -> String {
    let mut findings = report.findings.clone();
    sort_findings(&mut findings);

    let status = match report.verdict.status {
        VerdictStatus::Pass => "PASS",
        VerdictStatus::Warn => "WARN",
        VerdictStatus::Fail => "FAIL",
        VerdictStatus::Skip => "SKIP",
    };

    let mut out = String::new();
    out.push_str("### lintdiff\n\n");
    out.push_str(&format!(
        "**Status:** `{}`  \n**Counts:** error {} · warn {} · info {}\n\n",
        status, report.verdict.counts.error, report.verdict.counts.warn, report.verdict.counts.info
    ));

    // Explain summary line
    if let Some(data) = &report.data {
        if let Some(summary) = data.get("explain_summary") {
            let total = summary.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
            if total > 0 {
                let included = summary
                    .get("included")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let outside = summary
                    .get("dropped_outside_diff")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let no_span = summary
                    .get("dropped_no_span")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let by_path = summary
                    .get("dropped_by_path_filter")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let suppressed = summary
                    .get("suppressed_by_code")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                out.push_str(&format!(
                    "**Diagnostics:** {} total: {} matched",
                    total, included
                ));
                if outside > 0 {
                    out.push_str(&format!(", {} outside diff", outside));
                }
                if no_span > 0 {
                    out.push_str(&format!(", {} no span", no_span));
                }
                if by_path > 0 {
                    out.push_str(&format!(", {} filtered by path", by_path));
                }
                if suppressed > 0 {
                    out.push_str(&format!(", {} suppressed", suppressed));
                }
                out.push_str("\n\n");
            }
        }
    }

    if let Some(data) = &report.data {
        if let Some(trunc) = data.get("truncated").and_then(|v| v.as_bool()) {
            if trunc {
                out.push_str("> Output truncated. See full receipt: `");
                out.push_str(&opts.report_path);
                out.push_str("`.\n\n");
            }
        }
    }

    if report.verdict.status == VerdictStatus::Skip {
        out.push_str("_lintdiff skipped (missing inputs)._\n\n");
        if !report.verdict.reasons.is_empty() {
            out.push_str("Reasons: ");
            out.push_str(&report.verdict.reasons.join(", "));
            out.push_str("\n\n");
        }
        return out;
    }

    if findings.is_empty() {
        out.push_str("_No diagnostics matched changed lines._\n\n");
        out.push_str(&format!("Full receipt: `{}`\n", opts.report_path));
        return out;
    }

    out.push_str("| Sev | Location | Code | Message |\n");
    out.push_str("| --- | --- | --- | --- |\n");

    for f in findings.iter().take(opts.max_items) {
        out.push_str(&format!(
            "| {} | {} | `{}` | {} |\n",
            sev_badge(&f.severity),
            format_location(f),
            f.code,
            escape_table(&f.message)
        ));
    }

    if findings.len() > opts.max_items {
        out.push_str(&format!(
            "\n_And {} more… See full receipt: `{}`_\n",
            findings.len() - opts.max_items,
            opts.report_path
        ));
    } else {
        out.push_str(&format!("\nFull receipt: `{}`\n", opts.report_path));
    }

    out
}

pub fn render_github_annotations(report: &Report, max: usize) -> String {
    let mut findings = report.findings.clone();
    sort_findings(&mut findings);

    let mut out = String::new();

    for f in findings
        .into_iter()
        .filter(|f| f.location.is_some())
        .take(max)
    {
        let sev = match f.severity {
            Severity::Error => "error",
            Severity::Warn => "warning",
            Severity::Info => "notice",
        };

        let loc = f.location.as_ref().unwrap();
        let mut meta = format!("file={}", loc.path.as_str());
        if let Some(line) = loc.line {
            meta.push_str(&format!(",line={}", line));
        }
        if let Some(col) = loc.col {
            meta.push_str(&format!(",col={}", col));
        }

        let msg = format!("[{}] {}", f.code, f.message);
        out.push_str(&format!(
            "::{} {}::{}\n",
            sev,
            meta,
            escape_github_command(&msg)
        ));
    }

    out
}

fn sev_badge(sev: &Severity) -> &'static str {
    match sev {
        Severity::Error => "error",
        Severity::Warn => "warn",
        Severity::Info => "info",
    }
}

fn format_location(f: &Finding) -> String {
    if let Some(loc) = &f.location {
        if let Some(line) = loc.line {
            return format!("`{}:{}`", loc.path.as_str(), line);
        }
        return format!("`{}`", loc.path.as_str());
    }
    "`-`".to_string()
}

fn escape_table(s: &str) -> String {
    // Keep markdown tables from breaking on pipes/newlines.
    s.replace('|', "\\|").replace('\n', " ")
}

fn escape_github_command(s: &str) -> String {
    // GitHub Actions command escaping:
    // https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions
    s.replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintdiff_types::{
        Counts, Finding, Location, NormPath, Report, RunInfo, ToolInfo, Verdict, VerdictStatus,
        SCHEMA_ID, TOOL_NAME,
    };

    fn test_report(status: VerdictStatus, findings: Vec<Finding>) -> Report {
        let counts = counts_from(&findings);
        Report {
            schema: SCHEMA_ID.to_string(),
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
            verdict: Verdict {
                status,
                counts,
                reasons: vec![],
            },
            findings,
            data: None,
        }
    }

    fn counts_from(findings: &[Finding]) -> Counts {
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

    fn warn_finding(path: &str, line: u32, code: &str, msg: &str) -> Finding {
        Finding {
            severity: Severity::Warn,
            check_id: Some("diagnostics.on_diff".to_string()),
            code: code.to_string(),
            message: msg.to_string(),
            location: Some(Location {
                path: NormPath::new(path),
                line: Some(line),
                col: None,
            }),
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        }
    }

    #[test]
    fn markdown_pass_shows_no_findings_message() {
        let r = test_report(VerdictStatus::Pass, vec![]);
        let md = render_markdown(&r, MarkdownOptions::default());
        assert!(md.contains("PASS"));
        assert!(md.contains("No diagnostics matched"));
    }

    #[test]
    fn markdown_warn_shows_table() {
        let f = warn_finding("src/lib.rs", 1, "test.code", "test message");
        let r = test_report(VerdictStatus::Warn, vec![f]);
        let md = render_markdown(&r, MarkdownOptions::default());
        assert!(md.contains("WARN"));
        assert!(md.contains("| Sev | Location | Code | Message |"));
        assert!(md.contains("src/lib.rs:1"));
        assert!(md.contains("test.code"));
    }

    #[test]
    fn markdown_escapes_pipe_in_message() {
        let f = warn_finding("src/lib.rs", 1, "test", "has | pipe");
        let r = test_report(VerdictStatus::Warn, vec![f]);
        let md = render_markdown(&r, MarkdownOptions::default());
        assert!(md.contains("has \\| pipe"));
    }

    #[test]
    fn annotations_format_correct() {
        let f = warn_finding("src/lib.rs", 42, "test.code", "message");
        let r = test_report(VerdictStatus::Warn, vec![f]);
        let out = render_github_annotations(&r, 50);
        assert!(out.contains("::warning file=src/lib.rs,line=42::[test.code] message"));
    }

    #[test]
    fn annotations_escapes_newlines() {
        let f = warn_finding("src/lib.rs", 1, "test", "line1\nline2");
        let r = test_report(VerdictStatus::Warn, vec![f]);
        let out = render_github_annotations(&r, 50);
        assert!(out.contains("line1%0Aline2"));
        assert!(!out.contains('\n') || out.lines().count() <= 2); // only the trailing newline
    }

    #[test]
    fn annotations_empty_for_no_findings() {
        let r = test_report(VerdictStatus::Pass, vec![]);
        let out = render_github_annotations(&r, 50);
        assert!(out.trim().is_empty());
    }
}
