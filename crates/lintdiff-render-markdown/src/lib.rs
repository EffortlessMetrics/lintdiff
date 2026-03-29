//! Markdown rendering of findings for lintdiff.
//!
//! This microcrate provides a single responsibility: rendering findings and statistics
//! as GitHub-flavored markdown for use in PR comments, issues, and documentation.
//!
//! # Example: Rendering Findings
//!
//! ```
//! use lintdiff_render_markdown::{render_markdown, MarkdownConfig};
//! use lintdiff_types::{Finding, Severity, Location, NormPath};
//!
//! let findings = vec![
//!     Finding {
//!         severity: Severity::Error,
//!         code: "clippy::unwrap_used".to_string(),
//!         message: "used unwrap()".to_string(),
//!         location: Some(Location {
//!             path: NormPath::new("src/lib.rs"),
//!             line: Some(10),
//!             col: None,
//!         }),
//!         check_id: None,
//!         help: None,
//!         url: None,
//!         fingerprint: None,
//!         data: None,
//!     },
//! ];
//!
//! let config = MarkdownConfig::default();
//! let markdown = render_markdown(&findings, &config);
//! assert!(markdown.contains("src/lib.rs:10"));
//! assert!(markdown.contains("clippy::unwrap_used"));
//! ```
//!
//! # Example: Rendering a Summary
//!
//! ```
//! use lintdiff_render_markdown::{render_summary, MarkdownConfig};
//! use lintdiff_stats::Stats;
//!
//! let stats = Stats::from_findings(&[]);
//! let config = MarkdownConfig::default();
//! let summary = render_summary(&stats, &config);
//! assert!(summary.contains("Diagnostics"));
//! ```

#![warn(missing_docs)]

use lintdiff_stats::Stats;
use lintdiff_types::{sort_findings, Finding, Severity};

/// Configuration for markdown rendering.
///
/// Controls various aspects of how findings are rendered to markdown,
/// including line length limits and formatting options.
///
/// # Example
///
/// ```
/// use lintdiff_render_markdown::MarkdownConfig;
///
/// // Use defaults
/// let config = MarkdownConfig::default();
/// assert_eq!(config.max_line_length, 120);
/// assert!(config.include_snippets);
/// assert!(config.gfm);
///
/// // Customize for compact output
/// let compact = MarkdownConfig {
///     max_line_length: 80,
///     include_snippets: false,
///     gfm: false,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct MarkdownConfig {
    /// Maximum line length before truncation.
    ///
    /// Messages longer than this will be truncated with an ellipsis.
    pub max_line_length: usize,

    /// Whether to include code snippets in the output.
    ///
    /// When enabled, code is wrapped in backticks for proper formatting.
    pub include_snippets: bool,

    /// Whether to use GitHub-flavored markdown.
    ///
    /// When enabled, uses GFM-specific features like task lists and
    /// strikethrough. Currently reserved for future use.
    pub gfm: bool,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            max_line_length: 120,
            include_snippets: true,
            gfm: true,
        }
    }
}

/// Render findings as markdown.
///
/// Creates a markdown table containing all findings, sorted by severity
/// (errors first, then warnings, then info).
///
/// # Arguments
///
/// * `findings` - Slice of findings to render
/// * `config` - Configuration controlling output format
///
/// # Returns
///
/// A markdown-formatted string containing a table of findings.
///
/// # Example
///
/// ```
/// use lintdiff_render_markdown::{render_markdown, MarkdownConfig};
/// use lintdiff_types::{Finding, Severity, Location, NormPath};
///
/// let findings = vec![
///     Finding {
///         severity: Severity::Warn,
///         code: "WARN001".to_string(),
///         message: "This is a warning".to_string(),
///         location: Some(Location {
///             path: NormPath::new("src/main.rs"),
///             line: Some(42),
///             col: None,
///         }),
///         check_id: None,
///         help: None,
///         url: None,
///         fingerprint: None,
///         data: None,
///     },
/// ];
///
/// let md = render_markdown(&findings, &MarkdownConfig::default());
/// assert!(md.contains("| Sev | Location | Code | Message |"));
/// assert!(md.contains("src/main.rs:42"));
/// ```
#[must_use]
pub fn render_markdown(findings: &[Finding], config: &MarkdownConfig) -> String {
    if findings.is_empty() {
        return "_No findings to display._\n".to_string();
    }

    let mut sorted_findings = findings.to_vec();
    sort_findings(&mut sorted_findings);

    let mut out = String::new();

    // Table header
    out.push_str("| Sev | Location | Code | Message |\n");
    out.push_str("| --- | --- | --- | --- |\n");

    // Table rows
    for finding in &sorted_findings {
        let row = render_finding_row(finding, config);
        out.push_str(&row);
        out.push('\n');
    }

    out
}

/// Render a single finding as markdown.
///
/// Creates a markdown table row for a single finding.
///
/// # Arguments
///
/// * `finding` - The finding to render
/// * `config` - Configuration controlling output format
///
/// # Returns
///
/// A markdown-formatted string containing the finding as a table row.
///
/// # Example
///
/// ```
/// use lintdiff_render_markdown::{render_finding_markdown, MarkdownConfig};
/// use lintdiff_types::{Finding, Severity, Location, NormPath};
///
/// let finding = Finding {
///     severity: Severity::Error,
///     code: "E001".to_string(),
///     message: "Critical error found".to_string(),
///     location: Some(Location {
///         path: NormPath::new("src/error.rs"),
///         line: Some(100),
///         col: Some(5),
///     }),
///     check_id: None,
///     help: None,
///     url: None,
///     fingerprint: None,
///     data: None,
/// };
///
/// let md = render_finding_markdown(&finding, &MarkdownConfig::default());
/// assert!(md.contains("error"));
/// assert!(md.contains("src/error.rs:100"));
/// assert!(md.contains("E001"));
/// ```
#[must_use]
pub fn render_finding_markdown(finding: &Finding, config: &MarkdownConfig) -> String {
    let row = render_finding_row(finding, config);
    format!("{row}\n")
}

/// Render a summary section.
///
/// Creates a markdown summary of statistics including total diagnostics,
/// matched diagnostics, files affected, and breakdowns by severity and code.
///
/// # Arguments
///
/// * `stats` - Statistics to render
/// * `config` - Configuration controlling output format
///
/// # Returns
///
/// A markdown-formatted string containing the summary.
///
/// # Example
///
/// ```
/// use lintdiff_render_markdown::{render_summary, MarkdownConfig};
/// use lintdiff_stats::Stats;
///
/// let mut stats = Stats::new();
/// stats.total_diagnostics = 100;
/// stats.matched_diagnostics = 25;
/// stats.files_affected = 10;
///
/// let summary = render_summary(&stats, &MarkdownConfig::default());
/// assert!(summary.contains("Total: 100"));
/// assert!(summary.contains("Matched: 25"));
/// assert!(summary.contains("Files: 10"));
/// ```
#[must_use]
pub fn render_summary(stats: &Stats, config: &MarkdownConfig) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    out.push_str("### Summary\n\n");

    // Basic counts
    let _ = writeln!(
        out,
        "**Diagnostics:** Total: {} · Matched: {} · Files: {}\n",
        stats.total_diagnostics, stats.matched_diagnostics, stats.files_affected
    );

    // Severity breakdown
    if !stats.by_severity.is_empty() {
        out.push_str("**By Severity:** ");
        let severities: Vec<String> = stats
            .by_severity
            .iter()
            .map(|(k, v)| format!("{v} {k}"))
            .collect();
        out.push_str(&severities.join(" · "));
        out.push_str("\n\n");
    }

    // Code breakdown (top codes only to avoid very long output)
    if !stats.by_code.is_empty() {
        let mut codes: Vec<(&String, &usize)> = stats.by_code.iter().collect();
        codes.sort_by(|a, b| b.1.cmp(a.1));

        let max_codes = if config.max_line_length > 100 { 10 } else { 5 };
        out.push_str("**Top Codes:**\n");
        for (code, count) in codes.iter().take(max_codes) {
            let code_str = if config.include_snippets {
                format!("`{code}`")
            } else {
                (*code).clone()
            };
            let _ = writeln!(out, "- {code_str}: {count}");
        }
        if codes.len() > max_codes {
            let _ = writeln!(out, "- _... and {} more_", codes.len() - max_codes);
        }
        out.push('\n');
    }

    out
}

/// Render a finding as a table row.
fn render_finding_row(finding: &Finding, config: &MarkdownConfig) -> String {
    let severity = severity_badge(finding.severity);
    let location = format_location(finding);
    let code = if config.include_snippets {
        format!("`{}`", finding.code)
    } else {
        finding.code.clone()
    };
    let message = truncate_and_escape(&finding.message, config.max_line_length);

    format!("| {severity} | {location} | {code} | {message} |")
}

/// Get the severity badge text.
const fn severity_badge(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warn => "warn",
        Severity::Info => "info",
    }
}

/// Format the location of a finding.
fn format_location(finding: &Finding) -> String {
    if let Some(loc) = &finding.location {
        if let Some(line) = loc.line {
            return format!("`{}:{}`", loc.path.as_str(), line);
        }
        return format!("`{}`", loc.path.as_str());
    }
    "`-`".to_string()
}

/// Truncate a message and escape markdown table characters.
fn truncate_and_escape(s: &str, max_length: usize) -> String {
    let escaped = s
        .replace('|', "\\|")
        .replace('\r', "")
        .replace('\n', " ");

    if escaped.len() > max_length {
        format!("{}...", &escaped[..max_length.saturating_sub(3)])
    } else {
        escaped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintdiff_types::{Location, NormPath};

    fn test_finding(severity: Severity, path: &str, line: u32, code: &str, message: &str) -> Finding {
        Finding {
            severity,
            code: code.to_string(),
            message: message.to_string(),
            location: Some(Location {
                path: NormPath::new(path),
                line: Some(line),
                col: None,
            }),
            check_id: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        }
    }

    #[test]
    fn empty_findings_returns_message() {
        let findings: Vec<Finding> = vec![];
        let md = render_markdown(&findings, &MarkdownConfig::default());
        assert!(md.contains("No findings"));
    }

    #[test]
    fn single_finding_renders_correctly() {
        let findings = vec![test_finding(
            Severity::Error,
            "src/lib.rs",
            10,
            "E001",
            "Test error",
        )];
        let md = render_markdown(&findings, &MarkdownConfig::default());
        assert!(md.contains("| Sev | Location | Code | Message |"));
        assert!(md.contains("error"));
        assert!(md.contains("src/lib.rs:10"));
        assert!(md.contains("E001"));
        assert!(md.contains("Test error"));
    }

    #[test]
    fn multiple_findings_sorted_by_severity() {
        let findings = vec![
            test_finding(Severity::Info, "src/a.rs", 1, "I001", "info"),
            test_finding(Severity::Error, "src/b.rs", 2, "E001", "error"),
            test_finding(Severity::Warn, "src/c.rs", 3, "W001", "warn"),
        ];
        let md = render_markdown(&findings, &MarkdownConfig::default());
        let lines: Vec<&str> = md.lines().collect();
        // Error should come first (after header)
        assert!(lines[2].contains("error"));
        assert!(lines[3].contains("warn"));
        assert!(lines[4].contains("info"));
    }

    #[test]
    fn pipe_escaped_in_message() {
        let findings = vec![test_finding(
            Severity::Warn,
            "src/lib.rs",
            1,
            "W001",
            "has | pipe",
        )];
        let md = render_markdown(&findings, &MarkdownConfig::default());
        assert!(md.contains("has \\| pipe"));
    }

    #[test]
    fn newline_replaced_in_message() {
        let findings = vec![test_finding(
            Severity::Warn,
            "src/lib.rs",
            1,
            "W001",
            "line1\nline2",
        )];
        let md = render_markdown(&findings, &MarkdownConfig::default());
        // The newline in the message should be replaced with a space
        assert!(md.contains("line1 line2"));
    }

    #[test]
    fn message_truncated_when_long() {
        let long_message = "x".repeat(150);
        let findings = vec![test_finding(
            Severity::Warn,
            "src/lib.rs",
            1,
            "W001",
            &long_message,
        )];
        let config = MarkdownConfig {
            max_line_length: 50,
            ..Default::default()
        };
        let md = render_markdown(&findings, &config);
        assert!(md.contains("..."));
    }

    #[test]
    fn summary_renders_stats() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 100;
        stats.matched_diagnostics = 50;
        stats.files_affected = 10;

        let summary = render_summary(&stats, &MarkdownConfig::default());
        assert!(summary.contains("Total: 100"));
        assert!(summary.contains("Matched: 50"));
        assert!(summary.contains("Files: 10"));
    }

    #[test]
    fn single_finding_markdown_function() {
        let finding = test_finding(Severity::Error, "src/test.rs", 42, "E001", "Error message");
        let md = render_finding_markdown(&finding, &MarkdownConfig::default());
        assert!(md.starts_with('|'));
        assert!(md.contains("error"));
        assert!(md.contains("src/test.rs:42"));
    }

    #[test]
    fn no_snippets_config() {
        let findings = vec![test_finding(
            Severity::Warn,
            "src/lib.rs",
            1,
            "CODE001",
            "message",
        )];
        let config = MarkdownConfig {
            include_snippets: false,
            ..Default::default()
        };
        let md = render_markdown(&findings, &config);
        // Code should not have backticks when include_snippets is false
        assert!(md.contains("CODE001"));
        assert!(!md.contains("`CODE001`"));
    }
}
