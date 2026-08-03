//! Rendering helpers for lintdiff receipts.
//!
//! This crate provides rendering functionality for converting lintdiff reports
//! into human-readable formats suitable for different output contexts.
//!
//! # Modules
//!
//! - **Markdown rendering**: Convert reports to GitHub-flavored markdown tables
//! - **GitHub annotations**: Generate GitHub Actions workflow commands
//!
//! # Example
//!
//! ```
//! use lintdiff_render::{render_markdown, render_github_annotations, MarkdownOptions};
//! use lintdiff_types::{Report, Verdict, VerdictStatus, Counts, Finding, Severity, Location, NormPath};
//!
//! // Create a simple report (normally you'd get this from lintdiff-core)
//! let report = Report {
//!     schema: "lintdiff.report.v1".to_string(),
//!     tool: lintdiff_types::ToolInfo {
//!         name: "lintdiff".to_string(),
//!         version: "1.0.0".to_string(),
//!         commit: None,
//!     },
//!     run: lintdiff_types::RunInfo {
//!         started_at: "2026-01-01T00:00:00Z".to_string(),
//!         ended_at: "2026-01-01T00:00:01Z".to_string(),
//!         duration_ms: None,
//!         host: None,
//!         git: None,
//!     },
//!     verdict: lintdiff_types::Verdict {
//!         status: VerdictStatus::Pass,
//!         counts: Counts::default(),
//!         reasons: vec![],
//!     },
//!     findings: vec![],
//!     data: None,
//! };
//!
//! // Render as markdown
//! let md = render_markdown(&report, MarkdownOptions::default());
//! assert!(md.contains("### lintdiff"));
//! assert!(md.contains("PASS"));
//!
//! // Render as GitHub annotations
//! let annotations = render_github_annotations(&report, 50);
//! assert!(annotations.is_empty()); // No findings = no annotations
//! ```
//!
//! # Markdown Options
//!
//! The [`MarkdownOptions`] struct controls markdown output:
//!
//! ```
//! use lintdiff_render::MarkdownOptions;
//!
//! // Default options: 20 items max, default report path
//! let opts = MarkdownOptions::default();
//! assert_eq!(opts.max_items, 20);
//!
//! // Custom options
//! let custom = MarkdownOptions {
//!     max_items: 50,
//!     report_path: "custom/report.json".to_string(),
//! };
//! ```

use lintdiff_types::{sort_findings, Finding, Report, Severity, VerdictStatus};

/// Default path where the lintdiff report is stored.
///
/// This is used as the default location in markdown output when referring
/// users to the full report.
pub const DEFAULT_REPORT_PATH: &str = "artifacts/lintdiff/report.json";

/// Options for controlling markdown output rendering.
///
/// # Example
///
/// ```
/// use lintdiff_render::MarkdownOptions;
///
/// // Use defaults
/// let opts = MarkdownOptions::default();
///
/// // Customize for PR comments with limited space
/// let pr_opts = MarkdownOptions {
///     max_items: 10,
///     report_path: "ci-artifacts/lintdiff.json".to_string(),
/// };
/// ```
#[derive(Clone, Debug)]
pub struct MarkdownOptions {
    /// Maximum number of findings to include in the markdown table.
    /// Additional findings will be summarized as "And N more...".
    pub max_items: usize,

    /// Path to the full report file, included in markdown output so users
    /// can find the complete results.
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

/// Render a lintdiff report as GitHub-flavored markdown.
///
/// The output includes:
/// - A header with the lintdiff status
/// - Summary counts by severity
/// - Optional explain summary (if present in report data)
/// - A table of findings (up to `max_items`)
/// - Links to the full report
///
/// # Arguments
///
/// * `report` - The lintdiff report to render
/// * `opts` - Options controlling output format
///
/// # Returns
///
/// A markdown-formatted string suitable for GitHub PR comments or issues.
///
/// # Example
///
/// ```
/// use lintdiff_render::{render_markdown, MarkdownOptions};
/// use lintdiff_types::{Report, Verdict, VerdictStatus, Counts};
///
/// let report = Report {
///     schema: "lintdiff.report.v1".to_string(),
///     tool: lintdiff_types::ToolInfo {
///         name: "lintdiff".to_string(),
///         version: "1.0.0".to_string(),
///         commit: None,
///     },
///     run: lintdiff_types::RunInfo {
///         started_at: "2026-01-01T00:00:00Z".to_string(),
///         ended_at: "2026-01-01T00:00:01Z".to_string(),
///         duration_ms: None,
///         host: None,
///         git: None,
///     },
///     verdict: lintdiff_types::Verdict {
///         status: VerdictStatus::Warn,
///         counts: Counts { error: 0, warn: 2, info: 0 },
///         reasons: vec![],
///     },
///     findings: vec![],
///     data: None,
/// };
///
/// let md = render_markdown(&report, MarkdownOptions::default());
/// assert!(md.contains("**Status:** `WARN`"));
/// assert!(md.contains("warn 2"));
/// ```
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

/// Render a lintdiff report as GitHub Actions workflow commands (annotations).
///
/// The output uses GitHub's workflow command syntax to create annotations
/// that appear in the "Files changed" tab of pull requests and in the
/// Actions run summary.
///
/// # Annotation Format
///
/// Each annotation follows the format:
/// ```text
/// ::<severity> file=<path>,line=<line>::[<code>] <message>
/// ```
///
/// # Severity Mapping
///
/// - [`Severity::Error`] → `error` (red icon)
/// - [`Severity::Warn`] → `warning` (yellow icon)
/// - [`Severity::Info`] → `notice` (blue icon)
///
/// # Arguments
///
/// * `report` - The lintdiff report to render
/// * `max` - Maximum number of annotations to generate (GitHub has limits)
///
/// # Returns
///
/// A string containing GitHub workflow commands, one per line.
/// Findings without locations are filtered out (GitHub requires a file path).
///
/// # Special Character Escaping
///
/// Messages are escaped according to GitHub's requirements:
/// - `%` → `%25`
/// - `\r` → `%0D`
/// - `\n` → `%0A`
///
/// # Example
///
/// ```
/// use lintdiff_render::render_github_annotations;
/// use lintdiff_types::{Report, Finding, Severity, Location, NormPath, Verdict, VerdictStatus, Counts};
///
/// let finding = Finding {
///     severity: Severity::Warn,
///     check_id: Some("diagnostics.on_diff".to_string()),
///     code: "UNUSED_VAR".to_string(),
///     message: "Variable `x` is unused".to_string(),
///     location: Some(Location {
///         path: NormPath::new("src/lib.rs"),
///         line: Some(42),
///         col: Some(8),
///     }),
///     help: None,
///     url: None,
///     fingerprint: None,
///     data: None,
/// };
///
/// let report = Report {
///     schema: "lintdiff.report.v1".to_string(),
///     tool: lintdiff_types::ToolInfo {
///         name: "lintdiff".to_string(),
///         version: "1.0.0".to_string(),
///         commit: None,
///     },
///     run: lintdiff_types::RunInfo {
///         started_at: "2026-01-01T00:00:00Z".to_string(),
///         ended_at: "2026-01-01T00:00:01Z".to_string(),
///         duration_ms: None,
///         host: None,
///         git: None,
///     },
///     verdict: lintdiff_types::Verdict {
///         status: VerdictStatus::Warn,
///         counts: Counts { error: 0, warn: 1, info: 0 },
///         reasons: vec![],
///     },
///     findings: vec![finding],
///     data: None,
/// };
///
/// let annotations = render_github_annotations(&report, 50);
/// assert!(annotations.contains("::warning file=src/lib.rs,line=42,col=8::[UNUSED_VAR]"));
/// ```
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

/// Options for the bounded diagnostic-delta Markdown projection.
#[derive(Clone, Debug)]
pub struct DeltaMarkdownOptions {
    pub max_items: usize,
    pub receipt_path: String,
}

/// Render a lossy human projection while pointing to the complete delta artifact.
pub fn render_delta_markdown(
    receipt: &lintdiff_types::delta::DeltaReceipt,
    opts: DeltaMarkdownOptions,
) -> String {
    use lintdiff_types::delta::{DeltaLabel, DeltaVerdictStatus};

    let status = match receipt.verdict.status {
        DeltaVerdictStatus::Accepted => "ACCEPTED",
        DeltaVerdictStatus::Rejected => "REJECTED",
        DeltaVerdictStatus::Incomparable => "INCOMPARABLE",
    };
    let mut out = format!(
        "### lintdiff diagnostic delta\n\n**Status:** `{status}`  \n**Summary:** {} total · {} new · {} resolved · {} modified · {} ambiguous\n\n",
        receipt.summary.total,
        receipt.summary.new,
        receipt.summary.resolved,
        receipt.summary.modified,
        receipt.summary.ambiguous,
    );
    if receipt.provenance.comparability.status
        == lintdiff_types::delta::ComparabilityStatus::Incomparable
    {
        out.push_str("> Comparison is incomparable; no confident delta claim was made.\n\n");
    }
    if !receipt.verdict.reasons.is_empty() {
        out.push_str("Reasons: ");
        out.push_str(&receipt.verdict.reasons.join(", "));
        out.push_str("\n\n");
    }
    out.push_str(&format!(
        "Full receipt: `{}`\n\n| Result | Scope | Basis | Movement | Diagnostic |\n| --- | --- | --- | --- | --- |\n",
        opts.receipt_path
    ));
    for item in receipt
        .items
        .iter()
        .filter(|item| !matches!(item.label, Some(DeltaLabel::ExistingUntouched)))
        .take(opts.max_items)
    {
        let label = item
            .label
            .map(delta_label)
            .unwrap_or("ambiguous/incomparable");
        let diagnostic = delta_message(item);
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            label,
            delta_scope(item.diff_scope),
            delta_basis(item.match_basis),
            delta_movement(item.movement),
            escape_table(&diagnostic),
        ));
    }
    let visible = receipt
        .items
        .iter()
        .filter(|item| !matches!(item.label, Some(DeltaLabel::ExistingUntouched)))
        .count();
    if visible > opts.max_items {
        out.push_str(&format!(
            "\n_And {} more… See full receipt: `{}`_\n",
            visible - opts.max_items,
            opts.receipt_path
        ));
    }
    out
}

/// Render native GitHub annotations for head-side diagnostics in the delta.
pub fn render_delta_annotations(
    receipt: &lintdiff_types::delta::DeltaReceipt,
    max: usize,
) -> String {
    use lintdiff_types::delta::{DeltaLabel, PairingEvidence};

    let mut out = String::new();
    for item in receipt
        .items
        .iter()
        .filter(|item| {
            matches!(
                item.label,
                Some(DeltaLabel::NewOnDiff | DeltaLabel::Modified | DeltaLabel::Ambiguous)
            )
        })
        .take(max)
    {
        let (diagnostic, message) = match &item.pairing {
            PairingEvidence::HeadOnly { head } => {
                (head.as_ref(), format!("[{}] {}", head.code, head.message))
            }
            PairingEvidence::Matched { head, .. } => {
                (head.as_ref(), format!("[{}] {}", head.code, head.message))
            }
            PairingEvidence::Ambiguous {
                head_candidates, ..
            } => {
                let Some(head) = head_candidates.first() else {
                    continue;
                };
                (
                    head,
                    format!("ambiguous pairing: [{}] {}", head.code, head.message),
                )
            }
            PairingEvidence::BaseOnly { .. } => continue,
        };
        let Some(span) = diagnostic
            .primary_span
            .and_then(|index| diagnostic.spans.get(index))
            .or_else(|| diagnostic.spans.iter().find(|span| span.path.is_some()))
        else {
            continue;
        };
        let Some(path) = span.path.as_deref() else {
            continue;
        };
        let severity = if diagnostic.level == "error" {
            "error"
        } else if diagnostic.level == "warning" {
            "warning"
        } else {
            "notice"
        };
        let line = span.line_start.unwrap_or(1);
        out.push_str(&format!(
            "::{} file={},line={}::{}\n",
            severity,
            escape_github_command(path),
            line,
            escape_github_command(&message)
        ));
    }
    out
}

fn delta_label(label: lintdiff_types::delta::DeltaLabel) -> &'static str {
    match label {
        lintdiff_types::delta::DeltaLabel::NewOnDiff => "new_on_diff",
        lintdiff_types::delta::DeltaLabel::NewOffDiff => "new_off_diff",
        lintdiff_types::delta::DeltaLabel::ExistingTouched => "existing_touched",
        lintdiff_types::delta::DeltaLabel::ExistingUntouched => "existing_untouched",
        lintdiff_types::delta::DeltaLabel::Resolved => "resolved",
        lintdiff_types::delta::DeltaLabel::Modified => "modified",
        lintdiff_types::delta::DeltaLabel::Ambiguous => "ambiguous",
    }
}

fn delta_scope(scope: lintdiff_types::delta::DiffScope) -> &'static str {
    match scope {
        lintdiff_types::delta::DiffScope::Touched => "touched",
        lintdiff_types::delta::DiffScope::Untouched => "untouched",
        lintdiff_types::delta::DiffScope::NoLocation => "no_location",
        lintdiff_types::delta::DiffScope::Unknown => "unknown",
    }
}

fn delta_basis(basis: lintdiff_types::delta::MatchBasis) -> &'static str {
    match basis {
        lintdiff_types::delta::MatchBasis::Exact => "exact",
        lintdiff_types::delta::MatchBasis::LineMapped => "line_mapped",
        lintdiff_types::delta::MatchBasis::RenameMapped => "rename_mapped",
        lintdiff_types::delta::MatchBasis::Semantic => "semantic",
        lintdiff_types::delta::MatchBasis::Context => "context",
        lintdiff_types::delta::MatchBasis::ModifiedContext => "modified_context",
        lintdiff_types::delta::MatchBasis::None => "none",
        lintdiff_types::delta::MatchBasis::Ambiguous => "ambiguous",
    }
}

fn delta_movement(movement: lintdiff_types::delta::Movement) -> &'static str {
    match movement {
        lintdiff_types::delta::Movement::Same => "same",
        lintdiff_types::delta::Movement::Shifted => "shifted",
        lintdiff_types::delta::Movement::Renamed => "renamed",
        lintdiff_types::delta::Movement::ShiftedAndRenamed => "shifted_and_renamed",
        lintdiff_types::delta::Movement::Unknown => "unknown",
    }
}

fn delta_message(item: &lintdiff_types::delta::DeltaItem) -> String {
    use lintdiff_types::delta::PairingEvidence;
    let diagnostic = match &item.pairing {
        PairingEvidence::Matched { head, .. } => head.as_ref(),
        PairingEvidence::BaseOnly { base } => base.as_ref(),
        PairingEvidence::HeadOnly { head } => head.as_ref(),
        PairingEvidence::Ambiguous {
            head_candidates, ..
        } => {
            if let Some(head) = head_candidates.first() {
                head
            } else {
                return "candidate set has no head diagnostic".to_string();
            }
        }
    };
    format!("{}: {}", diagnostic.code, diagnostic.message)
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
