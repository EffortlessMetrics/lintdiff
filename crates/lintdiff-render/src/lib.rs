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
