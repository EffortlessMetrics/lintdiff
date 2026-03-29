//! Fluent builder for constructing `Finding` objects.
//!
//! This crate provides a builder pattern for creating `Finding` instances
//! from `lintdiff-types` with a clean, chainable API.
//!
//! # Example
//!
//! ```
//! use lintdiff_finding_builder::FindingBuilder;
//! use lintdiff_types::Severity;
//!
//! let finding = FindingBuilder::new()
//!     .with_code("unused_variables")
//!     .with_message("unused variable `x`")
//!     .with_severity(Severity::Warn)
//!     .with_path("src/lib.rs")
//!     .with_line(42)
//!     .build()
//!     .unwrap();
//!
//! assert_eq!(finding.code, "unused_variables");
//! assert_eq!(finding.message, "unused variable `x`");
//! ```

use lintdiff_types::{Finding, Location, NormPath, Severity};
use serde_json::Value;

/// Builder for constructing `Finding` objects with a fluent API.
///
/// Provides methods to set all fields of a `Finding`, with validation
/// performed during the `build()` step.
///
/// # Required Fields
///
/// The following fields are required before calling `build()`:
/// - `code` - The diagnostic code (e.g., "unused_variables")
/// - `message` - The diagnostic message
///
/// # Optional Fields
///
/// All other fields have sensible defaults:
/// - `severity` - Defaults to `Severity::Warn`
/// - `location` - Defaults to `None`
/// - `check_id` - Defaults to `None`
/// - `help` - Defaults to `None`
/// - `url` - Defaults to `None`
/// - `fingerprint` - Defaults to `None`
/// - `data` - Defaults to `None`
#[derive(Debug, Clone, Default)]
pub struct FindingBuilder {
    /// Diagnostic code (required).
    code: Option<String>,
    /// Diagnostic message (required).
    message: Option<String>,
    /// Severity level.
    severity: Option<Severity>,
    /// File path for location.
    path: Option<String>,
    /// Line number for location.
    line: Option<u32>,
    /// Column number for location.
    col: Option<u32>,
    /// Check ID for categorization.
    check_id: Option<String>,
    /// Help text for fixing the issue.
    help: Option<String>,
    /// URL for more information.
    url: Option<String>,
    /// Fingerprint for deduplication.
    fingerprint: Option<String>,
    /// Additional data.
    data: Option<Value>,
}

impl FindingBuilder {
    /// Create a new `FindingBuilder` with default values.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    ///
    /// let builder = FindingBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the diagnostic code.
    ///
    /// This field is required before calling `build()`.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    ///
    /// let builder = FindingBuilder::new()
    ///     .with_code("clippy::let_unit_value");
    /// ```
    #[must_use]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set the diagnostic message.
    ///
    /// This field is required before calling `build()`.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    ///
    /// let builder = FindingBuilder::new()
    ///     .with_message("unused variable `x`");
    /// ```
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set the severity level.
    ///
    /// If not set, defaults to `Severity::Warn`.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    /// use lintdiff_types::Severity;
    ///
    /// let builder = FindingBuilder::new()
    ///     .with_severity(Severity::Error);
    /// ```
    #[must_use]
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Set the file path for the location.
    ///
    /// When set, a `Location` will be created for the finding.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    ///
    /// let builder = FindingBuilder::new()
    ///     .with_path("src/lib.rs");
    /// ```
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the line number for the location.
    ///
    /// Only used if `with_path` is also set.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    ///
    /// let builder = FindingBuilder::new()
    ///     .with_path("src/lib.rs")
    ///     .with_line(42);
    /// ```
    #[must_use]
    pub fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    /// Set the column number for the location.
    ///
    /// Only used if `with_path` is also set.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    ///
    /// let builder = FindingBuilder::new()
    ///     .with_path("src/lib.rs")
    ///     .with_line(42)
    ///     .with_col(10);
    /// ```
    #[must_use]
    pub fn with_col(mut self, col: u32) -> Self {
        self.col = Some(col);
        self
    }

    /// Set the check ID for categorization.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    ///
    /// let builder = FindingBuilder::new()
    ///     .with_check_id("lintdiff.runtime");
    /// ```
    #[must_use]
    pub fn with_check_id(mut self, check_id: impl Into<String>) -> Self {
        self.check_id = Some(check_id.into());
        self
    }

    /// Set the help text for fixing the issue.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    ///
    /// let builder = FindingBuilder::new()
    ///     .with_help("Consider removing the unused variable");
    /// ```
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Set the URL for more information.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    ///
    /// let builder = FindingBuilder::new()
    ///     .with_url("https://rust-lang.github.io/rust-clippy/master/index.html#let_unit_value");
    /// ```
    #[must_use]
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the fingerprint for deduplication.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    ///
    /// let builder = FindingBuilder::new()
    ///     .with_fingerprint("abc123");
    /// ```
    #[must_use]
    pub fn with_fingerprint(mut self, fingerprint: impl Into<String>) -> Self {
        self.fingerprint = Some(fingerprint.into());
        self
    }

    /// Set additional data as JSON.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    /// use serde_json::json;
    ///
    /// let builder = FindingBuilder::new()
    ///     .with_data(json!({ "key": "value" }));
    /// ```
    #[must_use]
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Build the `Finding` from the configured values.
    ///
    /// # Errors
    ///
    /// Returns `BuildError` if required fields are missing:
    /// - `code` is required
    /// - `message` is required
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_finding_builder::FindingBuilder;
    /// use lintdiff_types::Severity;
    ///
    /// let finding = FindingBuilder::new()
    ///     .with_code("unused_variables")
    ///     .with_message("unused variable `x`")
    ///     .with_severity(Severity::Error)
    ///     .with_path("src/lib.rs")
    ///     .with_line(42)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(finding.code, "unused_variables");
    /// assert_eq!(finding.message, "unused variable `x`");
    /// assert_eq!(finding.severity, Severity::Error);
    /// ```
    pub fn build(self) -> Result<Finding, BuildError> {
        let code = self.code.ok_or(BuildError::MissingCode)?;
        let message = self.message.ok_or(BuildError::MissingMessage)?;

        let location = self.path.map(|p| Location {
            path: NormPath::new(&p),
            line: self.line,
            col: self.col,
        });

        Ok(Finding {
            severity: self.severity.unwrap_or(Severity::Warn),
            check_id: self.check_id,
            code,
            message,
            location,
            help: self.help,
            url: self.url,
            fingerprint: self.fingerprint,
            data: self.data,
        })
    }
}

/// Error type for `FindingBuilder::build()` failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// The `code` field was not set.
    MissingCode,
    /// The `message` field was not set.
    MissingMessage,
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCode => write!(f, "required field 'code' was not set"),
            Self::MissingMessage => write!(f, "required field 'message' was not set"),
        }
    }
}

impl std::error::Error for BuildError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_builder() {
        let finding = FindingBuilder::new()
            .with_code("test_code")
            .with_message("test message")
            .build()
            .unwrap();

        assert_eq!(finding.code, "test_code");
        assert_eq!(finding.message, "test message");
        assert_eq!(finding.severity, Severity::Warn);
        assert!(finding.location.is_none());
        assert!(finding.check_id.is_none());
        assert!(finding.help.is_none());
        assert!(finding.url.is_none());
        assert!(finding.fingerprint.is_none());
        assert!(finding.data.is_none());
    }

    #[test]
    fn test_full_builder() {
        let finding = FindingBuilder::new()
            .with_code("E001")
            .with_message("error message")
            .with_severity(Severity::Error)
            .with_path("src/main.rs")
            .with_line(10)
            .with_col(5)
            .with_check_id("custom.check")
            .with_help("fix this issue")
            .with_url("https://example.com/E001")
            .with_fingerprint("fp123")
            .with_data(json!({ "extra": "data" }))
            .build()
            .unwrap();

        assert_eq!(finding.code, "E001");
        assert_eq!(finding.message, "error message");
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.check_id, Some("custom.check".to_string()));
        assert_eq!(finding.help, Some("fix this issue".to_string()));
        assert_eq!(finding.url, Some("https://example.com/E001".to_string()));
        assert_eq!(finding.fingerprint, Some("fp123".to_string()));
        assert!(finding.data.is_some());

        let loc = finding.location.unwrap();
        assert_eq!(loc.path.as_str(), "src/main.rs");
        assert_eq!(loc.line, Some(10));
        assert_eq!(loc.col, Some(5));
    }

    #[test]
    fn test_missing_code() {
        let result = FindingBuilder::new().with_message("test message").build();

        assert_eq!(result.unwrap_err(), BuildError::MissingCode);
    }

    #[test]
    fn test_missing_message() {
        let result = FindingBuilder::new().with_code("test_code").build();

        assert_eq!(result.unwrap_err(), BuildError::MissingMessage);
    }

    #[test]
    fn test_missing_both_required() {
        let result = FindingBuilder::new().build();
        // Code is checked first
        assert_eq!(result.unwrap_err(), BuildError::MissingCode);
    }

    #[test]
    fn test_path_without_line_or_col() {
        let finding = FindingBuilder::new()
            .with_code("test")
            .with_message("msg")
            .with_path("lib.rs")
            .build()
            .unwrap();

        let loc = finding.location.unwrap();
        assert_eq!(loc.path.as_str(), "lib.rs");
        assert!(loc.line.is_none());
        assert!(loc.col.is_none());
    }

    #[test]
    fn test_line_without_path_is_ignored() {
        let finding = FindingBuilder::new()
            .with_code("test")
            .with_message("msg")
            .with_line(42)
            .build()
            .unwrap();

        // Line without path should not create a location
        assert!(finding.location.is_none());
    }

    #[test]
    fn test_severity_info() {
        let finding = FindingBuilder::new()
            .with_code("test")
            .with_message("msg")
            .with_severity(Severity::Info)
            .build()
            .unwrap();

        assert_eq!(finding.severity, Severity::Info);
    }

    #[test]
    fn test_path_normalization() {
        let finding = FindingBuilder::new()
            .with_code("test")
            .with_message("msg")
            .with_path("src\\lib.rs")
            .build()
            .unwrap();

        let loc = finding.location.unwrap();
        // NormPath normalizes backslashes to forward slashes
        assert_eq!(loc.path.as_str(), "src/lib.rs");
    }

    #[test]
    fn test_builder_is_chainable() {
        // Ensure all with_* methods return Self for chaining
        let _finding = FindingBuilder::new()
            .with_code("a")
            .with_message("b")
            .with_severity(Severity::Error)
            .with_path("c")
            .with_line(1)
            .with_col(2)
            .with_check_id("d")
            .with_help("e")
            .with_url("f")
            .with_fingerprint("g")
            .with_data(json!({}))
            .build()
            .unwrap();
    }

    #[test]
    fn test_display_error() {
        assert_eq!(
            format!("{}", BuildError::MissingCode),
            "required field 'code' was not set"
        );
        assert_eq!(
            format!("{}", BuildError::MissingMessage),
            "required field 'message' was not set"
        );
    }
}
