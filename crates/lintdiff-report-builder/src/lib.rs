//! Builder for constructing lintdiff reports.
//!
//! This microcrate provides a single responsibility: building structured reports
//! for lintdiff analysis results using the builder pattern.
//!
//! # Features
//!
//! - Builder pattern for constructing reports
//! - Per-file result tracking with added/removed/unchanged findings
//! - Summary calculation with totals
//! - Validation before building
//! - Optional serde serialization
//!
//! # Example
//!
//! ```
//! use lintdiff_report_builder::{ReportBuilder, FileResult, Finding, Severity};
//!
//! // Create a simple report
//! let report = ReportBuilder::new()
//!     .with_tool_info("lintdiff", "1.0.0")
//!     .with_timestamp("2024-01-15T10:30:00Z")
//!     .add_file_result("src/lib.rs", FileResult {
//!         path: "src/lib.rs".to_string(),
//!         added: vec![Finding::new("unused variable", Severity::Warning)],
//!         removed: vec![],
//!         unchanged: vec![],
//!     })
//!     .build()
//!     .expect("Failed to build report");
//!
//! assert_eq!(report.tool.name, "lintdiff");
//! assert_eq!(report.summary.total_added, 1);
//! ```

#![warn(missing_docs)]

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Error type for report builder operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ReportBuilderError {
    /// Tool name is missing.
    #[error("Tool name is required")]
    MissingToolName,

    /// Tool version is missing.
    #[error("Tool version is required")]
    MissingToolVersion,

    /// Timestamp is missing.
    #[error("Timestamp is required")]
    MissingTimestamp,

    /// Timestamp format is invalid.
    #[error("Invalid timestamp format: '{0}'")]
    InvalidTimestampFormat(String),

    /// File path is empty.
    #[error("File path cannot be empty")]
    EmptyFilePath,

    /// Duplicate file path.
    #[error("Duplicate file path: '{0}'")]
    DuplicateFilePath(String),
}

impl ReportBuilderError {
    /// Check if this is a missing tool name error.
    #[must_use]
    pub const fn is_missing_tool_name(&self) -> bool {
        matches!(self, Self::MissingToolName)
    }

    /// Check if this is a missing tool version error.
    #[must_use]
    pub const fn is_missing_tool_version(&self) -> bool {
        matches!(self, Self::MissingToolVersion)
    }

    /// Check if this is a missing timestamp error.
    #[must_use]
    pub const fn is_missing_timestamp(&self) -> bool {
        matches!(self, Self::MissingTimestamp)
    }

    /// Check if this is an invalid timestamp format error.
    #[must_use]
    pub const fn is_invalid_timestamp(&self) -> bool {
        matches!(self, Self::InvalidTimestampFormat(_))
    }
}

/// Tool information for a report.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ToolInfo {
    /// Tool name.
    pub name: String,
    /// Tool version.
    pub version: String,
}

impl ToolInfo {
    /// Create new tool info.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

/// Git information for a report.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GitInfo {
    /// Commit SHA.
    pub sha: String,
    /// Reference name (branch or tag).
    pub ref_name: Option<String>,
}

impl GitInfo {
    /// Create new git info with just a SHA.
    #[must_use]
    pub fn from_sha(sha: impl Into<String>) -> Self {
        Self {
            sha: sha.into(),
            ref_name: None,
        }
    }

    /// Create new git info with SHA and ref name.
    #[must_use]
    pub fn new(sha: impl Into<String>, ref_name: Option<&str>) -> Self {
        Self {
            sha: sha.into(),
            ref_name: ref_name.map(String::from),
        }
    }
}

/// Severity level for a finding.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Severity {
    /// Informational hint.
    Hint = 0,
    /// Note/suggestion.
    Note = 1,
    /// Warning.
    #[default]
    Warning = 2,
    /// Error.
    Error = 3,
    /// Fatal error.
    Fatal = 4,
}

impl Severity {
    /// Get a string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Hint => "hint",
            Self::Note => "note",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Fatal => "fatal",
        }
    }

    /// Check if this is a problem severity (warning or higher).
    #[must_use]
    pub fn is_problem(&self) -> bool {
        *self >= Self::Warning
    }

    /// Check if this is a blocking severity (error or higher).
    #[must_use]
    pub fn is_blocking(&self) -> bool {
        *self >= Self::Error
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A diagnostic finding.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Finding {
    /// Finding message.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
    /// Optional code/lint name.
    pub code: Option<String>,
    /// Optional file path.
    pub path: Option<String>,
    /// Optional line number.
    pub line: Option<u32>,
    /// Optional column number.
    pub column: Option<u32>,
}

impl Finding {
    /// Create a new finding with message and severity.
    #[must_use]
    pub fn new(message: impl Into<String>, severity: Severity) -> Self {
        Self {
            message: message.into(),
            severity,
            code: None,
            path: None,
            line: None,
            column: None,
        }
    }

    /// Create an error finding.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, Severity::Error)
    }

    /// Create a warning finding.
    #[must_use]
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, Severity::Warning)
    }

    /// Create a hint finding.
    #[must_use]
    pub fn hint(message: impl Into<String>) -> Self {
        Self::new(message, Severity::Hint)
    }

    /// Create a note finding.
    #[must_use]
    pub fn note(message: impl Into<String>) -> Self {
        Self::new(message, Severity::Note)
    }

    /// Create a fatal finding.
    #[must_use]
    pub fn fatal(message: impl Into<String>) -> Self {
        Self::new(message, Severity::Fatal)
    }

    /// Set the code.
    #[must_use]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set the path.
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the line number.
    #[must_use]
    pub const fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    /// Set the column number.
    #[must_use]
    pub const fn with_column(mut self, column: u32) -> Self {
        self.column = Some(column);
        self
    }

    /// Set the location (path, line, column).
    #[must_use]
    pub fn with_location(mut self, path: impl Into<String>, line: u32, column: u32) -> Self {
        self.path = Some(path.into());
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

/// Per-file result containing added, removed, and unchanged findings.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FileResult {
    /// File path.
    pub path: String,
    /// Added findings (new issues).
    pub added: Vec<Finding>,
    /// Removed findings (fixed issues).
    pub removed: Vec<Finding>,
    /// Unchanged findings (pre-existing issues).
    pub unchanged: Vec<Finding>,
}

impl FileResult {
    /// Create a new file result for the given path.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            added: Vec::new(),
            removed: Vec::new(),
            unchanged: Vec::new(),
        }
    }

    /// Add a finding to the added list.
    pub fn add_added(&mut self, finding: Finding) {
        self.added.push(finding);
    }

    /// Add a finding to the removed list.
    pub fn add_removed(&mut self, finding: Finding) {
        self.removed.push(finding);
    }

    /// Add a finding to the unchanged list.
    pub fn add_unchanged(&mut self, finding: Finding) {
        self.unchanged.push(finding);
    }

    /// Get the total count of all findings.
    #[must_use]
    pub const fn total_count(&self) -> usize {
        self.added.len() + self.removed.len() + self.unchanged.len()
    }

    /// Check if this file has any changes (added or removed).
    #[must_use]
    pub const fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty()
    }

    /// Check if this file has any added findings.
    #[must_use]
    pub const fn has_added(&self) -> bool {
        !self.added.is_empty()
    }

    /// Check if this file has any removed findings.
    #[must_use]
    pub const fn has_removed(&self) -> bool {
        !self.removed.is_empty()
    }

    /// Count errors in added findings.
    #[must_use]
    pub fn added_errors(&self) -> usize {
        self.added
            .iter()
            .filter(|f| f.severity.is_blocking())
            .count()
    }

    /// Count warnings in added findings.
    #[must_use]
    pub fn added_warnings(&self) -> usize {
        self.added
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .count()
    }
}

/// Summary counts for a report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReportSummary {
    /// Total added findings.
    pub total_added: usize,
    /// Total removed findings.
    pub total_removed: usize,
    /// Total unchanged findings.
    pub total_unchanged: usize,
    /// Number of files affected (files with changes).
    pub files_affected: usize,
}

impl ReportSummary {
    /// Create a new empty summary.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a summary with specific values.
    #[must_use]
    pub const fn from_counts(
        total_added: usize,
        total_removed: usize,
        total_unchanged: usize,
        files_affected: usize,
    ) -> Self {
        Self {
            total_added,
            total_removed,
            total_unchanged,
            files_affected,
        }
    }

    /// Get total findings count.
    #[must_use]
    pub const fn total_findings(&self) -> usize {
        self.total_added + self.total_removed + self.total_unchanged
    }

    /// Check if there are any changes.
    #[must_use]
    pub const fn has_changes(&self) -> bool {
        self.total_added > 0 || self.total_removed > 0
    }

    /// Get the net change (added - removed).
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub const fn net_change(&self) -> isize {
        self.total_added as isize - self.total_removed as isize
    }

    /// Calculate summary from file results.
    #[must_use]
    pub fn from_file_results(file_results: &[FileResult]) -> Self {
        let mut summary = Self::new();
        for result in file_results {
            summary.total_added += result.added.len();
            summary.total_removed += result.removed.len();
            summary.total_unchanged += result.unchanged.len();
            if result.has_changes() {
                summary.files_affected += 1;
            }
        }
        summary
    }
}

/// A complete lintdiff report.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Report {
    /// Tool information.
    pub tool: ToolInfo,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Git information (optional).
    pub git: Option<GitInfo>,
    /// Per-file results.
    pub files: Vec<FileResult>,
    /// Summary counts.
    pub summary: ReportSummary,
}

impl Report {
    /// Get the number of files in the report.
    #[must_use]
    pub const fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Check if the report is empty (no files).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Get a file result by path.
    #[must_use]
    pub fn get_file(&self, path: &str) -> Option<&FileResult> {
        self.files.iter().find(|f| f.path == path)
    }

    /// Check if the report has any added findings.
    #[must_use]
    pub const fn has_added(&self) -> bool {
        self.summary.total_added > 0
    }

    /// Check if the report has any removed findings.
    #[must_use]
    pub const fn has_removed(&self) -> bool {
        self.summary.total_removed > 0
    }
}

/// Builder for constructing reports.
#[derive(Debug, Clone, Default)]
pub struct ReportBuilder {
    tool_name: Option<String>,
    tool_version: Option<String>,
    timestamp: Option<String>,
    git: Option<GitInfo>,
    file_results: HashMap<String, FileResult>,
    custom_summary: Option<ReportSummary>,
}

impl ReportBuilder {
    /// Create a new report builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set tool information.
    #[must_use]
    pub fn with_tool_info(mut self, name: &str, version: &str) -> Self {
        self.tool_name = Some(name.to_string());
        self.tool_version = Some(version.to_string());
        self
    }

    /// Set the timestamp.
    #[must_use]
    pub fn with_timestamp(mut self, ts: &str) -> Self {
        self.timestamp = Some(ts.to_string());
        self
    }

    /// Set git information.
    #[must_use]
    pub fn with_git_info(mut self, sha: &str, ref_name: Option<&str>) -> Self {
        self.git = Some(GitInfo::new(sha, ref_name));
        self
    }

    /// Add a file result.
    #[must_use]
    pub fn add_file_result(mut self, path: &str, result: FileResult) -> Self {
        self.file_results.insert(path.to_string(), result);
        self
    }

    /// Add a custom summary (overrides automatic calculation).
    #[must_use]
    pub const fn add_summary(mut self, summary: ReportSummary) -> Self {
        self.custom_summary = Some(summary);
        self
    }

    /// Add a finding to a file's added list (creates file if needed).
    #[must_use]
    pub fn add_finding(mut self, path: &str, finding: Finding) -> Self {
        let file_result = self
            .file_results
            .entry(path.to_string())
            .or_insert_with(|| FileResult::new(path));
        file_result.add_added(finding);
        self
    }

    /// Add a removed finding to a file (creates file if needed).
    #[must_use]
    pub fn add_removed_finding(mut self, path: &str, finding: Finding) -> Self {
        let file_result = self
            .file_results
            .entry(path.to_string())
            .or_insert_with(|| FileResult::new(path));
        file_result.add_removed(finding);
        self
    }

    /// Add an unchanged finding to a file (creates file if needed).
    #[must_use]
    pub fn add_unchanged_finding(mut self, path: &str, finding: Finding) -> Self {
        let file_result = self
            .file_results
            .entry(path.to_string())
            .or_insert_with(|| FileResult::new(path));
        file_result.add_unchanged(finding);
        self
    }

    /// Validate the builder state.
    ///
    /// # Errors
    /// Returns an error if required fields are missing or invalid.
    pub fn validate(&self) -> Result<(), ReportBuilderError> {
        if self.tool_name.is_none() || self.tool_name.as_ref().is_some_and(String::is_empty) {
            return Err(ReportBuilderError::MissingToolName);
        }

        if self.tool_version.is_none() || self.tool_version.as_ref().is_some_and(String::is_empty) {
            return Err(ReportBuilderError::MissingToolVersion);
        }

        if self.timestamp.is_none() || self.timestamp.as_ref().is_some_and(String::is_empty) {
            return Err(ReportBuilderError::MissingTimestamp);
        }

        // Validate timestamp format (basic ISO 8601 check)
        if let Some(ts) = &self.timestamp {
            // Basic check: should contain a date component
            if !ts.contains('-') || ts.len() < 10 {
                return Err(ReportBuilderError::InvalidTimestampFormat(ts.clone()));
            }
        }

        // Validate file paths
        for path in self.file_results.keys() {
            if path.is_empty() {
                return Err(ReportBuilderError::EmptyFilePath);
            }
        }

        Ok(())
    }

    /// Build the final report.
    ///
    /// # Errors
    /// Returns an error if validation fails.
    #[allow(clippy::missing_panics_doc)]
    pub fn build(self) -> Result<Report, ReportBuilderError> {
        self.validate()?;

        // SAFETY: validate() ensures these are Some and non-empty
        #[allow(clippy::unwrap_used)]
        let tool = ToolInfo::new(self.tool_name.unwrap(), self.tool_version.unwrap());

        let mut files: Vec<FileResult> = self.file_results.into_values().collect();
        // Sort files by path for deterministic output
        files.sort_by(|a, b| a.path.cmp(&b.path));

        let summary = self
            .custom_summary
            .unwrap_or_else(|| ReportSummary::from_file_results(&files));

        // SAFETY: validate() ensures timestamp is Some and non-empty
        #[allow(clippy::unwrap_used)]
        let timestamp = self.timestamp.unwrap();

        Ok(Report {
            tool,
            timestamp,
            git: self.git,
            files,
            summary,
        })
    }
}

/// Quick function to create a minimal valid report.
///
/// # Panics
/// Panics if the report cannot be built (should never happen with valid inputs).
#[must_use]
#[allow(clippy::expect_used)]
pub fn quick_report(tool_name: &str, tool_version: &str, timestamp: &str) -> Report {
    ReportBuilder::new()
        .with_tool_info(tool_name, tool_version)
        .with_timestamp(timestamp)
        .build()
        .expect("Quick report should always be valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_info_creation() {
        let info = ToolInfo::new("lintdiff", "1.0.0");
        assert_eq!(info.name, "lintdiff");
        assert_eq!(info.version, "1.0.0");
    }

    #[test]
    fn test_git_info_creation() {
        let git = GitInfo::from_sha("abc123");
        assert_eq!(git.sha, "abc123");
        assert_eq!(git.ref_name, None);

        let git_with_ref = GitInfo::new("def456", Some("main"));
        assert_eq!(git_with_ref.sha, "def456");
        assert_eq!(git_with_ref.ref_name, Some("main".to_string()));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Note);
        assert!(Severity::Note > Severity::Hint);
        assert!(Severity::Fatal > Severity::Error);
    }

    #[test]
    fn test_severity_is_problem() {
        assert!(!Severity::Hint.is_problem());
        assert!(!Severity::Note.is_problem());
        assert!(Severity::Warning.is_problem());
        assert!(Severity::Error.is_problem());
        assert!(Severity::Fatal.is_problem());
    }

    #[test]
    fn test_severity_is_blocking() {
        assert!(!Severity::Hint.is_blocking());
        assert!(!Severity::Note.is_blocking());
        assert!(!Severity::Warning.is_blocking());
        assert!(Severity::Error.is_blocking());
        assert!(Severity::Fatal.is_blocking());
    }

    #[test]
    fn test_finding_creation() {
        let finding = Finding::new("unused variable", Severity::Warning);
        assert_eq!(finding.message, "unused variable");
        assert_eq!(finding.severity, Severity::Warning);
        assert_eq!(finding.code, None);
        assert_eq!(finding.path, None);
    }

    #[test]
    fn test_finding_convenience_methods() {
        let error = Finding::error("test error");
        assert_eq!(error.severity, Severity::Error);

        let warning = Finding::warning("test warning");
        assert_eq!(warning.severity, Severity::Warning);

        let hint = Finding::hint("test hint");
        assert_eq!(hint.severity, Severity::Hint);
    }

    #[test]
    fn test_finding_builders() {
        let finding = Finding::error("test")
            .with_code("E001")
            .with_path("src/lib.rs")
            .with_line(42)
            .with_column(10);

        assert_eq!(finding.code, Some("E001".to_string()));
        assert_eq!(finding.path, Some("src/lib.rs".to_string()));
        assert_eq!(finding.line, Some(42));
        assert_eq!(finding.column, Some(10));
    }

    #[test]
    fn test_finding_with_location() {
        let finding = Finding::warning("test").with_location("src/main.rs", 10, 5);

        assert_eq!(finding.path, Some("src/main.rs".to_string()));
        assert_eq!(finding.line, Some(10));
        assert_eq!(finding.column, Some(5));
    }

    #[test]
    fn test_file_result_creation() {
        let result = FileResult::new("src/lib.rs");
        assert_eq!(result.path, "src/lib.rs");
        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
        assert!(result.unchanged.is_empty());
    }

    #[test]
    fn test_file_result_add_findings() {
        let mut result = FileResult::new("src/lib.rs");
        result.add_added(Finding::error("error1"));
        result.add_removed(Finding::warning("warning1"));
        result.add_unchanged(Finding::hint("hint1"));

        assert_eq!(result.added.len(), 1);
        assert_eq!(result.removed.len(), 1);
        assert_eq!(result.unchanged.len(), 1);
    }

    #[test]
    fn test_file_result_total_count() {
        let mut result = FileResult::new("src/lib.rs");
        result.add_added(Finding::error("e1"));
        result.add_added(Finding::error("e2"));
        result.add_removed(Finding::warning("w1"));
        result.add_unchanged(Finding::hint("h1"));
        result.add_unchanged(Finding::hint("h2"));

        assert_eq!(result.total_count(), 5);
    }

    #[test]
    fn test_file_result_has_changes() {
        let mut result = FileResult::new("src/lib.rs");
        assert!(!result.has_changes());

        result.add_unchanged(Finding::hint("hint"));
        assert!(!result.has_changes());

        result.add_added(Finding::error("error"));
        assert!(result.has_changes());
    }

    #[test]
    fn test_file_result_has_added_removed() {
        let mut result = FileResult::new("src/lib.rs");
        assert!(!result.has_added());
        assert!(!result.has_removed());

        result.add_added(Finding::error("e"));
        assert!(result.has_added());
        assert!(!result.has_removed());

        result.add_removed(Finding::warning("w"));
        assert!(result.has_added());
        assert!(result.has_removed());
    }

    #[test]
    fn test_file_result_added_errors_warnings() {
        let mut result = FileResult::new("src/lib.rs");
        result.add_added(Finding::error("e1"));
        result.add_added(Finding::error("e2"));
        result.add_added(Finding::warning("w1"));
        result.add_added(Finding::hint("h1"));

        assert_eq!(result.added_errors(), 2);
        assert_eq!(result.added_warnings(), 1);
    }

    #[test]
    fn test_report_summary_creation() {
        let summary = ReportSummary::new();
        assert_eq!(summary.total_added, 0);
        assert_eq!(summary.total_removed, 0);
        assert_eq!(summary.total_unchanged, 0);
        assert_eq!(summary.files_affected, 0);
    }

    #[test]
    fn test_report_summary_from_counts() {
        let summary = ReportSummary::from_counts(10, 5, 20, 3);
        assert_eq!(summary.total_added, 10);
        assert_eq!(summary.total_removed, 5);
        assert_eq!(summary.total_unchanged, 20);
        assert_eq!(summary.files_affected, 3);
    }

    #[test]
    fn test_report_summary_total_findings() {
        let summary = ReportSummary::from_counts(10, 5, 20, 3);
        assert_eq!(summary.total_findings(), 35);
    }

    #[test]
    fn test_report_summary_has_changes() {
        let no_changes = ReportSummary::from_counts(0, 0, 10, 0);
        assert!(!no_changes.has_changes());

        let with_added = ReportSummary::from_counts(1, 0, 10, 1);
        assert!(with_added.has_changes());

        let with_removed = ReportSummary::from_counts(0, 1, 10, 1);
        assert!(with_removed.has_changes());
    }

    #[test]
    fn test_report_summary_net_change() {
        let added = ReportSummary::from_counts(10, 3, 0, 1);
        assert_eq!(added.net_change(), 7);

        let removed = ReportSummary::from_counts(2, 5, 0, 1);
        assert_eq!(removed.net_change(), -3);

        let balanced = ReportSummary::from_counts(5, 5, 0, 1);
        assert_eq!(balanced.net_change(), 0);
    }

    #[test]
    fn test_report_summary_from_file_results() {
        let mut file1 = FileResult::new("src/a.rs");
        file1.add_added(Finding::error("e1"));
        file1.add_added(Finding::warning("w1"));
        file1.add_unchanged(Finding::hint("h1"));

        let mut file2 = FileResult::new("src/b.rs");
        file2.add_removed(Finding::error("e2"));

        let file3 = FileResult::new("src/c.rs"); // No changes

        let summary = ReportSummary::from_file_results(&[file1, file2, file3]);
        assert_eq!(summary.total_added, 2);
        assert_eq!(summary.total_removed, 1);
        assert_eq!(summary.total_unchanged, 1);
        assert_eq!(summary.files_affected, 2);
    }

    #[test]
    fn test_report_builder_new() {
        let builder = ReportBuilder::new();
        assert!(builder.tool_name.is_none());
        assert!(builder.tool_version.is_none());
        assert!(builder.timestamp.is_none());
        assert!(builder.git.is_none());
        assert!(builder.file_results.is_empty());
    }

    #[test]
    fn test_report_builder_with_tool_info() {
        let builder = ReportBuilder::new().with_tool_info("lintdiff", "1.0.0");
        assert_eq!(builder.tool_name, Some("lintdiff".to_string()));
        assert_eq!(builder.tool_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_report_builder_with_timestamp() {
        let builder = ReportBuilder::new().with_timestamp("2024-01-15T10:30:00Z");
        assert_eq!(builder.timestamp, Some("2024-01-15T10:30:00Z".to_string()));
    }

    #[test]
    fn test_report_builder_with_git_info() {
        let builder = ReportBuilder::new().with_git_info("abc123", Some("main"));
        assert!(builder.git.is_some());
        let git = builder.git.unwrap();
        assert_eq!(git.sha, "abc123");
        assert_eq!(git.ref_name, Some("main".to_string()));
    }

    #[test]
    fn test_report_builder_add_file_result() {
        let result = FileResult::new("src/lib.rs");
        let builder = ReportBuilder::new().add_file_result("src/lib.rs", result);
        assert_eq!(builder.file_results.len(), 1);
    }

    #[test]
    fn test_report_builder_add_finding() {
        let builder = ReportBuilder::new().add_finding("src/lib.rs", Finding::error("test error"));

        assert_eq!(builder.file_results.len(), 1);
        let result = builder.file_results.get("src/lib.rs").unwrap();
        assert_eq!(result.added.len(), 1);
    }

    #[test]
    fn test_report_builder_add_removed_finding() {
        let builder =
            ReportBuilder::new().add_removed_finding("src/lib.rs", Finding::warning("fixed"));

        let result = builder.file_results.get("src/lib.rs").unwrap();
        assert_eq!(result.removed.len(), 1);
        assert!(result.added.is_empty());
    }

    #[test]
    fn test_report_builder_add_unchanged_finding() {
        let builder =
            ReportBuilder::new().add_unchanged_finding("src/lib.rs", Finding::hint("unchanged"));

        let result = builder.file_results.get("src/lib.rs").unwrap();
        assert_eq!(result.unchanged.len(), 1);
    }

    #[test]
    fn test_report_builder_validate_missing_tool_name() {
        let builder = ReportBuilder::new()
            .with_tool_info("", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z");

        let err = builder.validate().unwrap_err();
        assert!(err.is_missing_tool_name());
    }

    #[test]
    fn test_report_builder_validate_missing_tool_version() {
        let builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "")
            .with_timestamp("2024-01-15T10:30:00Z");

        let err = builder.validate().unwrap_err();
        assert!(err.is_missing_tool_version());
    }

    #[test]
    fn test_report_builder_validate_missing_timestamp() {
        let builder = ReportBuilder::new().with_tool_info("lintdiff", "1.0.0");

        let err = builder.validate().unwrap_err();
        assert!(err.is_missing_timestamp());
    }

    #[test]
    fn test_report_builder_validate_invalid_timestamp() {
        let builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("invalid");

        let err = builder.validate().unwrap_err();
        assert!(err.is_invalid_timestamp());
    }

    #[test]
    fn test_report_builder_validate_success() {
        let builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z");

        assert!(builder.validate().is_ok());
    }

    #[test]
    fn test_report_builder_build_success() {
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("src/lib.rs", Finding::error("test"))
            .build()
            .unwrap();

        assert_eq!(report.tool.name, "lintdiff");
        assert_eq!(report.tool.version, "1.0.0");
        assert_eq!(report.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(report.files.len(), 1);
        assert_eq!(report.summary.total_added, 1);
    }

    #[test]
    fn test_report_builder_build_with_git() {
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .with_git_info("abc123", Some("main"))
            .build()
            .unwrap();

        assert!(report.git.is_some());
        let git = report.git.unwrap();
        assert_eq!(git.sha, "abc123");
        assert_eq!(git.ref_name, Some("main".to_string()));
    }

    #[test]
    fn test_report_builder_build_with_custom_summary() {
        let custom = ReportSummary::from_counts(100, 50, 200, 10);
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("src/lib.rs", Finding::error("test"))
            .add_summary(custom.clone())
            .build()
            .unwrap();

        // Custom summary should override calculated one
        assert_eq!(report.summary, custom);
    }

    #[test]
    fn test_report_builder_files_sorted() {
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("z.rs", Finding::error("z"))
            .add_finding("a.rs", Finding::error("a"))
            .add_finding("m.rs", Finding::error("m"))
            .build()
            .unwrap();

        // Files should be sorted by path
        assert_eq!(report.files[0].path, "a.rs");
        assert_eq!(report.files[1].path, "m.rs");
        assert_eq!(report.files[2].path, "z.rs");
    }

    #[test]
    fn test_report_file_count() {
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("a.rs", Finding::error("a"))
            .add_finding("b.rs", Finding::error("b"))
            .build()
            .unwrap();

        assert_eq!(report.file_count(), 2);
    }

    #[test]
    fn test_report_is_empty() {
        let empty = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .build()
            .unwrap();
        assert!(empty.is_empty());

        let not_empty = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("a.rs", Finding::error("a"))
            .build()
            .unwrap();
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_report_get_file() {
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("src/lib.rs", Finding::error("test"))
            .build()
            .unwrap();

        let file = report.get_file("src/lib.rs");
        assert!(file.is_some());

        let missing = report.get_file("nonexistent.rs");
        assert!(missing.is_none());
    }

    #[test]
    fn test_report_has_added_removed() {
        let with_added = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("a.rs", Finding::error("a"))
            .build()
            .unwrap();
        assert!(with_added.has_added());
        assert!(!with_added.has_removed());

        let with_removed = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_removed_finding("a.rs", Finding::error("a"))
            .build()
            .unwrap();
        assert!(!with_removed.has_added());
        assert!(with_removed.has_removed());
    }

    #[test]
    fn test_quick_report() {
        let report = quick_report("mytool", "2.0.0", "2024-01-15T10:30:00Z");
        assert_eq!(report.tool.name, "mytool");
        assert_eq!(report.tool.version, "2.0.0");
        assert_eq!(report.timestamp, "2024-01-15T10:30:00Z");
        assert!(report.git.is_none());
        assert!(report.is_empty());
    }

    #[test]
    fn test_error_types() {
        let err = ReportBuilderError::MissingToolName;
        assert!(err.is_missing_tool_name());
        assert!(!err.is_missing_tool_version());

        let err = ReportBuilderError::MissingToolVersion;
        assert!(err.is_missing_tool_version());
        assert!(!err.is_missing_timestamp());

        let err = ReportBuilderError::MissingTimestamp;
        assert!(err.is_missing_timestamp());
        assert!(!err.is_invalid_timestamp());

        let err = ReportBuilderError::InvalidTimestampFormat("bad".to_string());
        assert!(err.is_invalid_timestamp());
        assert!(!err.is_missing_timestamp());
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Hint), "hint");
        assert_eq!(format!("{}", Severity::Note), "note");
        assert_eq!(format!("{}", Severity::Warning), "warning");
        assert_eq!(format!("{}", Severity::Error), "error");
        assert_eq!(format!("{}", Severity::Fatal), "fatal");
    }

    #[test]
    fn test_builder_chaining() {
        // Test that all builder methods can be chained
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .with_git_info("abc123", Some("main"))
            .add_finding("src/a.rs", Finding::error("error1"))
            .add_removed_finding("src/a.rs", Finding::warning("fixed1"))
            .add_unchanged_finding("src/b.rs", Finding::hint("hint1"))
            .add_file_result("src/c.rs", FileResult::new("src/c.rs"))
            .build()
            .unwrap();

        assert_eq!(report.files.len(), 3);
    }

    #[test]
    fn test_empty_file_result_has_no_changes() {
        let result = FileResult::new("empty.rs");
        assert!(!result.has_changes());
        assert!(!result.has_added());
        assert!(!result.has_removed());
        assert_eq!(result.total_count(), 0);
    }

    #[test]
    fn test_report_summary_empty() {
        let summary = ReportSummary::from_file_results(&[]);
        assert_eq!(summary.total_added, 0);
        assert_eq!(summary.total_removed, 0);
        assert_eq!(summary.total_unchanged, 0);
        assert_eq!(summary.files_affected, 0);
    }

    #[test]
    fn test_multiple_findings_same_file() {
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("src/lib.rs", Finding::error("e1"))
            .add_finding("src/lib.rs", Finding::error("e2"))
            .add_finding("src/lib.rs", Finding::warning("w1"))
            .build()
            .unwrap();

        assert_eq!(report.files.len(), 1);
        assert_eq!(report.summary.total_added, 3);
    }

    #[test]
    fn test_file_result_equality() {
        let mut result1 = FileResult::new("src/lib.rs");
        result1.add_added(Finding::error("test"));

        let mut result2 = FileResult::new("src/lib.rs");
        result2.add_added(Finding::error("test"));

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_report_equality() {
        let report1 = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("a.rs", Finding::error("test"))
            .build()
            .unwrap();

        let report2 = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("a.rs", Finding::error("test"))
            .build()
            .unwrap();

        assert_eq!(report1, report2);
    }
}

