//! Diagnostic finding representation for lintdiff.
//!
//! Provides the core `Finding` type that represents a single
//! diagnostic issue found during linting.

use std::fmt;

/// A diagnostic finding.
///
/// # Examples
/// ```
/// use lintdiff_finding::Finding;
///
/// let finding = Finding::new("src/lib.rs", "unused variable `x`")
///     .with_line(42)
///     .with_code("unused_variables");
///
/// assert_eq!(finding.path(), "src/lib.rs");
/// assert_eq!(finding.line(), Some(42));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// File path.
    path: String,
    /// Diagnostic message.
    message: String,
    /// Line number (1-based).
    line: Option<u32>,
    /// Column number (1-based).
    column: Option<u32>,
    /// End line for spans.
    end_line: Option<u32>,
    /// End column for spans.
    end_column: Option<u32>,
    /// Diagnostic code/lint name.
    code: Option<String>,
    /// Severity level.
    severity: Severity,
    /// Source tool name.
    source: Option<String>,
    /// Suggested fix.
    suggestion: Option<String>,
}

impl Finding {
    /// Create a new finding with path and message.
    #[must_use]
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            line: None,
            column: None,
            end_line: None,
            end_column: None,
            code: None,
            severity: Severity::Warning,
            source: None,
            suggestion: None,
        }
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

    /// Set the line range.
    #[must_use]
    pub const fn with_line_range(mut self, start: u32, end: u32) -> Self {
        self.line = Some(start);
        self.end_line = Some(end);
        self
    }

    /// Set the column range.
    #[must_use]
    pub const fn with_column_range(mut self, start: u32, end: u32) -> Self {
        self.column = Some(start);
        self.end_column = Some(end);
        self
    }

    /// Set the diagnostic code.
    #[must_use]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set the severity.
    #[must_use]
    pub const fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the source tool.
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set a suggested fix.
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Get the file path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the line number.
    #[must_use]
    pub const fn line(&self) -> Option<u32> {
        self.line
    }

    /// Get the column number.
    #[must_use]
    pub const fn column(&self) -> Option<u32> {
        self.column
    }

    /// Get the end line.
    #[must_use]
    pub const fn end_line(&self) -> Option<u32> {
        self.end_line
    }

    /// Get the end column.
    #[must_use]
    pub const fn end_column(&self) -> Option<u32> {
        self.end_column
    }

    /// Get the diagnostic code.
    #[must_use]
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    /// Get the severity.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// Get the source tool.
    #[must_use]
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    /// Get the suggestion.
    #[must_use]
    pub fn suggestion(&self) -> Option<&str> {
        self.suggestion.as_deref()
    }

    /// Check if this is a multi-line finding.
    #[must_use]
    pub fn is_multiline(&self) -> bool {
        self.end_line.is_some_and(|end| end != self.line.unwrap_or(0))
    }

    /// Check if this finding has a span (line and column).
    #[must_use]
    pub const fn has_span(&self) -> bool {
        self.line.is_some()
    }

    /// Check if this is an error-level finding.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error | Severity::Fatal)
    }

    /// Check if this is a warning-level finding.
    #[must_use]
    pub const fn is_warning(&self) -> bool {
        matches!(self.severity, Severity::Warning)
    }

    /// Create a location string (path:line:column).
    #[must_use]
    pub fn location(&self) -> String {
        match (self.line, self.column) {
            (Some(line), Some(col)) => format!("{}:{}:{}", self.path, line, col),
            (Some(line), None) => format!("{}:{}", self.path, line),
            (None, _) => self.path.clone(),
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.location(), self.message)?;
        if let Some(code) = &self.code {
            write!(f, " [{code}]")?;
        }
        Ok(())
    }
}

/// Severity level for a finding.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
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
    /// Parse from a string.
    ///
    /// # Errors
    /// Returns an error if the string doesn't match a known severity.
    pub fn parse(s: &str) -> Result<Self, SeverityParseError> {
        match s.to_lowercase().as_str() {
            "hint" | "info" | "information" => Ok(Self::Hint),
            "note" | "suggestion" => Ok(Self::Note),
            "warning" | "warn" => Ok(Self::Warning),
            "error" | "err" => Ok(Self::Error),
            "fatal" | "critical" => Ok(Self::Fatal),
            _ => Err(SeverityParseError::new(s)),
        }
    }

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

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error when parsing severity.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Unknown severity: '{0}'")]
pub struct SeverityParseError(String);

impl SeverityParseError {
    /// Create a new severity parse error.
    #[must_use]
    fn new(s: &str) -> Self {
        Self(s.to_string())
    }

    /// Get the unknown severity string.
    #[must_use]
    pub fn unknown(&self) -> &str {
        &self.0
    }
}

/// A collection of findings.
#[derive(Debug, Clone, Default)]
pub struct Findings {
    findings: Vec<Finding>,
}

impl Findings {
    /// Create an empty collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from a vector of findings.
    #[must_use]
    pub const fn from_vec(findings: Vec<Finding>) -> Self {
        Self { findings }
    }

    /// Add a finding.
    pub fn push(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    /// Get the number of findings.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.findings.len()
    }

    /// Check if empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.findings.is_empty()
    }

    /// Get an iterator over findings.
    pub fn iter(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter()
    }

    /// Filter by severity.
    #[must_use]
    pub fn filter_by_severity(&self, severity: Severity) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| f.severity() == severity)
            .collect()
    }

    /// Filter by path prefix.
    #[must_use]
    pub fn filter_by_path(&self, prefix: &str) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| f.path().starts_with(prefix))
            .collect()
    }

    /// Count by severity.
    #[must_use]
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity() == severity)
            .count()
    }

    /// Count errors.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.count_by_severity(Severity::Error) + self.count_by_severity(Severity::Fatal)
    }

    /// Count warnings.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.count_by_severity(Severity::Warning)
    }
}

impl FromIterator<Finding> for Findings {
    fn from_iter<T: IntoIterator<Item = Finding>>(iter: T) -> Self {
        Self {
            findings: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_new() {
        let finding = Finding::new("src/lib.rs", "unused variable");
        assert_eq!(finding.path(), "src/lib.rs");
        assert_eq!(finding.message(), "unused variable");
    }

    #[test]
    fn test_severity_default() {
        let finding = Finding::new("test.rs", "msg");
        assert_eq!(finding.severity(), Severity::Warning);
    }
}
