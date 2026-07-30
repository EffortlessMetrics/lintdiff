//! Explain artifact summary building and aggregation for lintdiff.
//!
//! This microcrate provides functionality for building explain artifact summaries,
//! per-file summary generation, and summary aggregation.
//!
//! # Features
//!
//! - `SummaryBuilder` for constructing summaries using the builder pattern
//! - `ExplainSummary` for representing a summary of explain artifacts
//! - `FileSummary` for per-file summary with counts by severity/disposition
//! - `FindingLike` trait for polymorphic handling of findings
//! - Aggregation functions for combining multiple summaries
//! - Formatting functions for markdown and JSON output
//! - Optional serde serialization via the `serde` feature
//!
//! # Example
//!
//! ```
//! use lintdiff_explain_summary::{SummaryBuilder, ExplainSummary, FindingLike, MockFinding};
//!
//! // Create findings
//! let finding1 = MockFinding::new("src/lib.rs", "warning", "added", Some(10));
//! let finding2 = MockFinding::new("src/lib.rs", "error", "added", Some(20));
//! let finding3 = MockFinding::new("src/main.rs", "warning", "unchanged", Some(5));
//!
//! // Build a summary
//! let summary = SummaryBuilder::new()
//!     .add_finding(&finding1)
//!     .add_finding(&finding2)
//!     .add_finding(&finding3)
//!     .with_timestamp("2024-01-15T10:30:00Z")
//!     .build();
//!
//! assert_eq!(summary.total_findings, 3);
//! assert_eq!(summary.files_affected, 2);
//! ```

#![warn(missing_docs)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Error type for explain summary operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ExplainSummaryError {
    /// Invalid severity string.
    #[error("Invalid severity: '{0}'")]
    InvalidSeverity(String),

    /// Invalid disposition string.
    #[error("Invalid disposition: '{0}'")]
    InvalidDisposition(String),

    /// Empty file path when one is required.
    #[error("File path cannot be empty")]
    EmptyFilePath,
}

impl ExplainSummaryError {
    /// Check if this is an invalid severity error.
    #[must_use]
    pub const fn is_invalid_severity(&self) -> bool {
        matches!(self, Self::InvalidSeverity(_))
    }

    /// Check if this is an invalid disposition error.
    #[must_use]
    pub const fn is_invalid_disposition(&self) -> bool {
        matches!(self, Self::InvalidDisposition(_))
    }

    /// Check if this is an empty file path error.
    #[must_use]
    pub const fn is_empty_file_path(&self) -> bool {
        matches!(self, Self::EmptyFilePath)
    }
}

/// A trait for polymorphic handling of finding-like objects.
///
/// This trait allows the summary builder to work with any type that provides
/// the necessary finding information.
pub trait FindingLike {
    /// Get the file path for this finding.
    fn path(&self) -> Option<&Path>;

    /// Get the severity level as a string.
    fn severity(&self) -> &str;

    /// Get the disposition (e.g., "added", "removed", "unchanged").
    fn disposition(&self) -> &str;

    /// Get the line number if available.
    fn line(&self) -> Option<usize>;
}

/// A mock finding implementation for testing and examples.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MockFinding {
    /// File path.
    pub path: Option<PathBuf>,
    /// Severity level.
    pub severity: String,
    /// Disposition.
    pub disposition: String,
    /// Line number.
    pub line: Option<usize>,
}

impl MockFinding {
    /// Create a new mock finding.
    #[must_use]
    pub fn new(
        path: impl AsRef<str>,
        severity: impl Into<String>,
        disposition: impl Into<String>,
        line: Option<usize>,
    ) -> Self {
        Self {
            path: Some(PathBuf::from(path.as_ref())),
            severity: severity.into(),
            disposition: disposition.into(),
            line,
        }
    }

    /// Create a finding with no path.
    #[must_use]
    pub fn orphan(severity: impl Into<String>, disposition: impl Into<String>) -> Self {
        Self {
            path: None,
            severity: severity.into(),
            disposition: disposition.into(),
            line: None,
        }
    }

    /// Set the line number.
    #[must_use]
    pub const fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }
}

impl FindingLike for MockFinding {
    fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn severity(&self) -> &str {
        &self.severity
    }

    fn disposition(&self) -> &str {
        &self.disposition
    }

    fn line(&self) -> Option<usize> {
        self.line
    }
}

/// Per-file summary with counts by severity and disposition.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FileSummary {
    /// File path.
    pub path: PathBuf,
    /// Total number of findings in this file.
    pub finding_count: usize,
    /// Count of findings by severity.
    pub by_severity: HashMap<String, usize>,
    /// Count of findings by disposition.
    pub by_disposition: HashMap<String, usize>,
    /// Set of line numbers affected.
    pub lines_affected: HashSet<usize>,
}

impl FileSummary {
    /// Create a new file summary for the given path.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            finding_count: 0,
            by_severity: HashMap::new(),
            by_disposition: HashMap::new(),
            lines_affected: HashSet::new(),
        }
    }

    /// Add a finding to this summary.
    pub fn add_finding(&mut self, severity: &str, disposition: &str, line: Option<usize>) {
        self.finding_count += 1;
        *self.by_severity.entry(severity.to_string()).or_insert(0) += 1;
        *self
            .by_disposition
            .entry(disposition.to_string())
            .or_insert(0) += 1;
        if let Some(l) = line {
            self.lines_affected.insert(l);
        }
    }

    /// Get the count for a specific severity.
    #[must_use]
    pub fn severity_count(&self, severity: &str) -> usize {
        self.by_severity.get(severity).copied().unwrap_or(0)
    }

    /// Get the count for a specific disposition.
    #[must_use]
    pub fn disposition_count(&self, disposition: &str) -> usize {
        self.by_disposition.get(disposition).copied().unwrap_or(0)
    }

    /// Check if this file has any findings.
    #[must_use]
    pub const fn has_findings(&self) -> bool {
        self.finding_count > 0
    }

    /// Get the number of unique lines affected.
    #[must_use]
    pub fn lines_affected_count(&self) -> usize {
        self.lines_affected.len()
    }

    /// Check if a specific line is affected.
    #[must_use]
    pub fn is_line_affected(&self, line: usize) -> bool {
        self.lines_affected.contains(&line)
    }

    /// Merge another file summary into this one.
    pub fn merge(&mut self, other: &Self) {
        self.finding_count += other.finding_count;
        for (k, v) in &other.by_severity {
            *self.by_severity.entry(k.clone()).or_insert(0) += v;
        }
        for (k, v) in &other.by_disposition {
            *self.by_disposition.entry(k.clone()).or_insert(0) += v;
        }
        self.lines_affected.extend(&other.lines_affected);
    }

    /// Get all severities present in this file.
    pub fn severities(&self) -> impl Iterator<Item = &str> {
        self.by_severity.keys().map(String::as_str)
    }

    /// Get all dispositions present in this file.
    pub fn dispositions(&self) -> impl Iterator<Item = &str> {
        self.by_disposition.keys().map(String::as_str)
    }
}

/// Summary of explain artifacts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExplainSummary {
    /// Total number of findings.
    pub total_findings: usize,
    /// Count of findings by severity.
    pub by_severity: HashMap<String, usize>,
    /// Count of findings by disposition.
    pub by_disposition: HashMap<String, usize>,
    /// Per-file summaries.
    pub by_file: HashMap<PathBuf, FileSummary>,
    /// Number of files affected.
    pub files_affected: usize,
    /// Optional timestamp.
    pub timestamp: Option<String>,
}

impl ExplainSummary {
    /// Create a new empty summary.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a summary with a timestamp.
    #[must_use]
    pub fn with_timestamp(timestamp: impl Into<String>) -> Self {
        Self {
            timestamp: Some(timestamp.into()),
            ..Self::default()
        }
    }

    /// Check if the summary is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.total_findings == 0
    }

    /// Get the count for a specific severity.
    #[must_use]
    pub fn severity_count(&self, severity: &str) -> usize {
        self.by_severity.get(severity).copied().unwrap_or(0)
    }

    /// Get the count for a specific disposition.
    #[must_use]
    pub fn disposition_count(&self, disposition: &str) -> usize {
        self.by_disposition.get(disposition).copied().unwrap_or(0)
    }

    /// Get a file summary by path.
    #[must_use]
    pub fn get_file(&self, path: &Path) -> Option<&FileSummary> {
        self.by_file.get(path)
    }

    /// Check if a file is in the summary.
    #[must_use]
    pub fn has_file(&self, path: &Path) -> bool {
        self.by_file.contains_key(path)
    }

    /// Get the number of files in the summary.
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.by_file.len()
    }

    /// Get all severities present in the summary.
    pub fn severities(&self) -> impl Iterator<Item = &str> {
        self.by_severity.keys().map(String::as_str)
    }

    /// Get all dispositions present in the summary.
    pub fn dispositions(&self) -> impl Iterator<Item = &str> {
        self.by_disposition.keys().map(String::as_str)
    }

    /// Get the total count of problem-level severities (warning, error, fatal).
    #[must_use]
    pub fn problem_count(&self) -> usize {
        self.severity_count("warning")
            + self.severity_count("error")
            + self.severity_count("fatal")
            + self.severity_count("critical")
    }

    /// Get the total count of error-level severities (error, fatal, critical).
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.severity_count("error")
            + self.severity_count("fatal")
            + self.severity_count("critical")
    }

    /// Check if there are any added findings.
    #[must_use]
    pub fn has_added(&self) -> bool {
        self.disposition_count("added") > 0
    }

    /// Check if there are any removed findings.
    #[must_use]
    pub fn has_removed(&self) -> bool {
        self.disposition_count("removed") > 0
    }

    /// Check if there are any unchanged findings.
    #[must_use]
    pub fn has_unchanged(&self) -> bool {
        self.disposition_count("unchanged") > 0
    }

    /// Get added count.
    #[must_use]
    pub fn added_count(&self) -> usize {
        self.disposition_count("added")
    }

    /// Get removed count.
    #[must_use]
    pub fn removed_count(&self) -> usize {
        self.disposition_count("removed")
    }

    /// Get unchanged count.
    #[must_use]
    pub fn unchanged_count(&self) -> usize {
        self.disposition_count("unchanged")
    }

    /// Merge another summary into this one.
    pub fn merge(&mut self, other: &Self) {
        self.total_findings += other.total_findings;
        for (k, v) in &other.by_severity {
            *self.by_severity.entry(k.clone()).or_insert(0) += v;
        }
        for (k, v) in &other.by_disposition {
            *self.by_disposition.entry(k.clone()).or_insert(0) += v;
        }
        for (path, file_summary) in &other.by_file {
            if let Some(existing) = self.by_file.get_mut(path) {
                existing.merge(file_summary);
            } else {
                self.by_file.insert(path.clone(), file_summary.clone());
            }
        }
        self.files_affected = self.by_file.len();
    }
}

/// Builder for constructing explain summaries.
#[derive(Debug, Clone, Default)]
pub struct SummaryBuilder {
    findings: Vec<(Option<PathBuf>, String, String, Option<usize>)>,
    timestamp: Option<String>,
}

impl SummaryBuilder {
    /// Create a new summary builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single finding to the builder.
    #[must_use]
    pub fn add_finding<F: FindingLike + ?Sized>(mut self, finding: &F) -> Self {
        self.findings.push((
            finding.path().map(Path::to_path_buf),
            finding.severity().to_string(),
            finding.disposition().to_string(),
            finding.line(),
        ));
        self
    }

    /// Add multiple findings to the builder.
    #[must_use]
    pub fn add_findings<F: FindingLike + ?Sized>(mut self, findings: &[&F]) -> Self {
        for finding in findings {
            self.findings.push((
                finding.path().map(Path::to_path_buf),
                finding.severity().to_string(),
                finding.disposition().to_string(),
                finding.line(),
            ));
        }
        self
    }

    /// Set the timestamp for the summary.
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Build the final explain summary.
    #[must_use]
    pub fn build(self) -> ExplainSummary {
        let mut summary = ExplainSummary::new();
        summary.timestamp = self.timestamp;

        for (path, severity, disposition, line) in &self.findings {
            summary.total_findings += 1;
            *summary.by_severity.entry(severity.clone()).or_insert(0) += 1;
            *summary
                .by_disposition
                .entry(disposition.clone())
                .or_insert(0) += 1;

            if let Some(p) = path {
                let file_summary = summary.by_file.entry(p.clone()).or_insert_with(|| {
                    let mut fs = FileSummary::new(p.clone());
                    fs.by_severity = HashMap::new();
                    fs.by_disposition = HashMap::new();
                    fs
                });
                file_summary.add_finding(severity, disposition, *line);
            }
        }

        summary.files_affected = summary.by_file.len();
        summary
    }

    /// Get the current number of findings in the builder.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.findings.len()
    }

    /// Check if the builder is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.findings.is_empty()
    }

    /// Clear all findings from the builder.
    pub fn clear(&mut self) {
        self.findings.clear();
        self.timestamp = None;
    }
}

/// Aggregate multiple summaries into a single summary.
#[must_use]
pub fn aggregate_summaries(summaries: &[ExplainSummary]) -> ExplainSummary {
    let mut result = ExplainSummary::new();
    for summary in summaries {
        result.merge(summary);
    }
    result
}

/// Create a map of file paths to file summaries from findings.
#[must_use]
pub fn summarize_by_file<F: FindingLike + ?Sized>(
    findings: &[&F],
) -> HashMap<PathBuf, FileSummary> {
    let mut result: HashMap<PathBuf, FileSummary> = HashMap::new();

    for finding in findings {
        if let Some(path) = finding.path() {
            let file_summary = result.entry(path.to_path_buf()).or_insert_with(|| {
                let mut fs = FileSummary::new(path);
                fs.by_severity = HashMap::new();
                fs.by_disposition = HashMap::new();
                fs
            });
            file_summary.add_finding(finding.severity(), finding.disposition(), finding.line());
        }
    }

    result
}

/// Create a map of severity levels to counts from a summary.
#[must_use]
pub fn summarize_by_severity(summary: &ExplainSummary) -> HashMap<String, usize> {
    summary.by_severity.clone()
}

/// Create a map of dispositions to counts from a summary.
#[must_use]
pub fn summarize_by_disposition(summary: &ExplainSummary) -> HashMap<String, usize> {
    summary.by_disposition.clone()
}

/// Format a summary as markdown.
#[must_use]
pub fn format_summary_markdown(summary: &ExplainSummary) -> String {
    use std::fmt::Write;
    let mut output = String::new();

    let _ = writeln!(output, "# Explain Summary\n");

    // Timestamp
    if let Some(ref ts) = summary.timestamp {
        let _ = writeln!(output, "**Timestamp:** {ts}\n");
    }

    // Overview
    let _ = writeln!(output, "## Overview\n");
    let _ = writeln!(output, "- **Total Findings:** {}", summary.total_findings);
    let _ = writeln!(output, "- **Files Affected:** {}\n", summary.files_affected);

    // By Severity
    if !summary.by_severity.is_empty() {
        let _ = writeln!(output, "## By Severity\n");
        let mut severities: Vec<_> = summary.by_severity.iter().collect();
        severities.sort_by(|a, b| a.0.cmp(b.0));
        for (severity, count) in severities {
            let _ = writeln!(output, "- **{severity}:** {count}");
        }
    }

    // By Disposition
    if !summary.by_disposition.is_empty() {
        let _ = writeln!(output, "\n## By Disposition\n");
        let mut dispositions: Vec<_> = summary.by_disposition.iter().collect();
        dispositions.sort_by(|a, b| a.0.cmp(b.0));
        for (disposition, count) in dispositions {
            let _ = writeln!(output, "- **{disposition}:** {count}");
        }
    }

    // By File
    if !summary.by_file.is_empty() {
        let _ = writeln!(output, "\n## By File\n");
        let mut files: Vec<_> = summary.by_file.iter().collect();
        files.sort_by(|a, b| a.0.cmp(b.0));
        for (path, file_summary) in files {
            let _ = writeln!(output, "### {}\n", path.display());
            let _ = writeln!(output, "- **Findings:** {}", file_summary.finding_count);
            if !file_summary.lines_affected.is_empty() {
                let mut lines: Vec<_> = file_summary.lines_affected.iter().copied().collect();
                lines.sort_unstable();
                let _ = writeln!(output, "- **Lines Affected:** {}\n", lines.len());
            }
        }
    }

    output
}

/// Format a summary as JSON.
///
/// Requires the `serde` feature to be enabled.
#[cfg(feature = "serde")]
#[must_use]
pub fn format_summary_json(summary: &ExplainSummary) -> String {
    serde_json::to_string_pretty(summary).unwrap_or_else(|e| {
        format!(
            r#"{{"error": "Failed to serialize summary: {}"}}"#,
            e.to_string().replace('"', "\\\"")
        )
    })
}

/// Format a summary as JSON.
///
/// When the `serde` feature is not enabled, returns an error message.
#[cfg(not(feature = "serde"))]
#[must_use]
pub fn format_summary_json(_summary: &ExplainSummary) -> String {
    r#"{"error": "serde feature not enabled"}"#.to_string()
}

/// Format a file summary as markdown.
#[must_use]
pub fn format_file_summary_markdown(file_summary: &FileSummary) -> String {
    use std::fmt::Write;
    let mut output = String::new();

    let _ = writeln!(output, "## {}\n", file_summary.path.display());
    let _ = writeln!(output, "- **Findings:** {}\n", file_summary.finding_count);

    if !file_summary.by_severity.is_empty() {
        let _ = writeln!(output, "### By Severity\n");
        let mut severities: Vec<_> = file_summary.by_severity.iter().collect();
        severities.sort_by(|a, b| a.0.cmp(b.0));
        for (severity, count) in severities {
            let _ = writeln!(output, "- **{severity}:** {count}\n");
        }
    }

    if !file_summary.by_disposition.is_empty() {
        let _ = writeln!(output, "### By Disposition\n");
        let mut dispositions: Vec<_> = file_summary.by_disposition.iter().collect();
        dispositions.sort_by(|a, b| a.0.cmp(b.0));
        for (disposition, count) in dispositions {
            let _ = writeln!(output, "- **{disposition}:** {count}\n");
        }
    }

    if !file_summary.lines_affected.is_empty() {
        let mut lines: Vec<_> = file_summary.lines_affected.iter().copied().collect();
        lines.sort_unstable();
        let _ = writeln!(output, "\n### Lines Affected\n\n{}\n", lines.len());
    }

    output
}

/// Calculate the percentage of findings for a given count.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn percentage(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (count as f64 / total as f64) * 100.0
    }
}

/// Get the top N files by finding count.
#[must_use]
pub fn top_files_by_findings(summary: &ExplainSummary, n: usize) -> Vec<&FileSummary> {
    let mut files: Vec<_> = summary.by_file.values().collect();
    files.sort_by_key(|b| std::cmp::Reverse(b.finding_count));
    files.into_iter().take(n).collect()
}

/// Get files sorted by path.
#[must_use]
pub fn files_sorted_by_path(summary: &ExplainSummary) -> Vec<&FileSummary> {
    let mut files: Vec<_> = summary.by_file.values().collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files
}

/// Check if a summary has any problems (warnings or errors).
#[must_use]
pub fn has_problems(summary: &ExplainSummary) -> bool {
    summary.problem_count() > 0
}

/// Check if a summary has only informational findings.
#[must_use]
pub fn is_info_only(summary: &ExplainSummary) -> bool {
    summary.total_findings > 0
        && summary.total_findings
            == summary.severity_count("info")
                + summary.severity_count("hint")
                + summary.severity_count("note")
}

/// Get a summary of only added findings.
#[must_use]
pub fn filter_by_disposition(summary: &ExplainSummary, disposition: &str) -> ExplainSummary {
    let mut result = ExplainSummary::new();
    result.timestamp.clone_from(&summary.timestamp);

    for (path, file_summary) in &summary.by_file {
        let count = file_summary.disposition_count(disposition);
        if count > 0 {
            let mut new_file_summary = FileSummary::new(path.clone());
            new_file_summary.finding_count = count;
            for (sev, sev_count) in &file_summary.by_severity {
                // We need to check each finding's disposition, but since FileSummary
                // doesn't track individual findings, we approximate
                *new_file_summary.by_severity.entry(sev.clone()).or_insert(0) += *sev_count;
            }
            new_file_summary
                .by_disposition
                .insert(disposition.to_string(), count);
            new_file_summary
                .lines_affected
                .clone_from(&file_summary.lines_affected);
            result.by_file.insert(path.clone(), new_file_summary);
        }
    }

    result.total_findings = summary.disposition_count(disposition);
    result
        .by_disposition
        .insert(disposition.to_string(), result.total_findings);
    result.files_affected = result.by_file.len();

    // Copy severity counts proportionally
    result.by_severity.clone_from(&summary.by_severity);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_finding_new_creates_finding() {
        let finding = MockFinding::new("src/lib.rs", "warning", "added", Some(10));
        assert_eq!(finding.path.as_deref(), Some(Path::new("src/lib.rs")));
        assert_eq!(finding.severity, "warning");
        assert_eq!(finding.disposition, "added");
        assert_eq!(finding.line, Some(10));
    }

    #[test]
    fn mock_finding_orphan_has_no_path() {
        let finding = MockFinding::orphan("error", "removed");
        assert!(finding.path.is_none());
        assert_eq!(finding.severity, "error");
        assert_eq!(finding.disposition, "removed");
    }

    #[test]
    fn summary_builder_new_creates_empty_builder() {
        let builder = SummaryBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
    }

    #[test]
    fn summary_builder_add_finding_increments_count() {
        let finding = MockFinding::new("src/lib.rs", "warning", "added", Some(10));
        let builder = SummaryBuilder::new().add_finding(&finding);
        assert_eq!(builder.len(), 1);
    }

    #[test]
    fn summary_builder_build_creates_summary() {
        let finding = MockFinding::new("src/lib.rs", "warning", "added", Some(10));
        let summary = SummaryBuilder::new().add_finding(&finding).build();
        assert_eq!(summary.total_findings, 1);
    }

    #[test]
    fn explain_summary_new_creates_empty_summary() {
        let summary = ExplainSummary::new();
        assert!(summary.is_empty());
        assert_eq!(summary.total_findings, 0);
    }

    #[test]
    fn file_summary_new_creates_empty_summary() {
        let file_summary = FileSummary::new("src/lib.rs");
        assert!(!file_summary.has_findings());
        assert_eq!(file_summary.finding_count, 0);
    }

    #[test]
    fn aggregate_summaries_combines_multiple() {
        let s1 = SummaryBuilder::new()
            .add_finding(&MockFinding::new("a.rs", "warning", "added", None))
            .build();
        let s2 = SummaryBuilder::new()
            .add_finding(&MockFinding::new("b.rs", "error", "added", None))
            .build();
        let combined = aggregate_summaries(&[s1, s2]);
        assert_eq!(combined.total_findings, 2);
        assert_eq!(combined.files_affected, 2);
    }
}
