//! Annotation format handling and CI detection for lintdiff.
//!
//! This microcrate provides utilities for formatting annotations across different
//! CI platforms. It supports GitHub Actions, GitLab CI, Azure DevOps, `CircleCI`,
//! and plain text output formats.
//!
//! # Example: Detecting CI Platform
//!
//! ```
//! use lintdiff_annotation_format::{detect_ci, CiPlatform};
//!
//! let platform = detect_ci();
//! println!("Running on: {:?}", platform);
//! ```
//!
//! # Example: Formatting Annotations
//!
//! ```
//! use lintdiff_annotation_format::{Annotation, AnnotationFormat, AnnotationSeverity, format_annotation};
//!
//! let annotation = Annotation {
//!     path: "src/lib.rs".to_string(),
//!     line: 42,
//!     column: Some(10),
//!     severity: AnnotationSeverity::Warning,
//!     message: "Unused variable".to_string(),
//! };
//!
//! let output = format_annotation(AnnotationFormat::Github, &annotation);
//! assert!(output.contains("::warning"));
//! assert!(output.contains("file=src/lib.rs"));
//! ```

#![warn(missing_docs)]

use std::env;

/// Supported CI platforms for detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CiPlatform {
    /// GitHub Actions CI
    GithubActions,
    /// GitLab CI
    GitLabCI,
    /// Azure DevOps Pipelines
    AzureDevOps,
    /// `CircleCI`
    CircleCI,
    /// Travis CI
    TravisCI,
    /// Jenkins
    Jenkins,
    /// Unknown or not in CI
    Unknown,
}

impl CiPlatform {
    /// Returns the annotation format for this CI platform.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_annotation_format::{CiPlatform, AnnotationFormat};
    ///
    /// assert_eq!(CiPlatform::GithubActions.annotation_format(), AnnotationFormat::Github);
    /// assert_eq!(CiPlatform::GitLabCI.annotation_format(), AnnotationFormat::Gitlab);
    /// assert_eq!(CiPlatform::Unknown.annotation_format(), AnnotationFormat::Default);
    /// ```
    #[must_use]
    pub const fn annotation_format(self) -> AnnotationFormat {
        match self {
            Self::GithubActions => AnnotationFormat::Github,
            Self::GitLabCI => AnnotationFormat::Gitlab,
            Self::AzureDevOps => AnnotationFormat::Azure,
            Self::CircleCI => AnnotationFormat::CircleCI,
            Self::TravisCI | Self::Jenkins | Self::Unknown => AnnotationFormat::Default,
        }
    }
}

/// Output format types for annotations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnnotationFormat {
    /// GitHub Actions annotations (`::warning::`)
    Github,
    /// GitLab CI annotations
    Gitlab,
    /// Azure DevOps logging commands
    Azure,
    /// `CircleCI` test metadata
    CircleCI,
    /// Plain text output
    #[default]
    Default,
    /// Auto-detect from CI environment
    Auto,
}

impl AnnotationFormat {
    /// Resolves the format, detecting CI if set to Auto.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_annotation_format::AnnotationFormat;
    ///
    /// // Non-auto formats return themselves
    /// assert_eq!(AnnotationFormat::Github.resolve(), AnnotationFormat::Github);
    /// assert_eq!(AnnotationFormat::Default.resolve(), AnnotationFormat::Default);
    ///
    /// // Auto format detects CI environment
    /// let resolved = AnnotationFormat::Auto.resolve();
    /// // Will be one of the CI formats or Default
    /// assert!(matches!(resolved, AnnotationFormat::Github | AnnotationFormat::Gitlab |
    ///                  AnnotationFormat::Azure | AnnotationFormat::CircleCI |
    ///                  AnnotationFormat::Default));
    /// ```
    #[must_use]
    pub fn resolve(self) -> Self {
        match self {
            Self::Auto => detect_ci().annotation_format(),
            other => other,
        }
    }
}

/// Severity level for annotations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnnotationSeverity {
    /// Informational notice
    Notice,
    /// Warning level
    #[default]
    Warning,
    /// Error level
    Error,
    /// Fatal error
    Fatal,
}

impl AnnotationSeverity {
    /// Converts the severity to a GitHub annotation level string.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_annotation_format::AnnotationSeverity;
    ///
    /// assert_eq!(AnnotationSeverity::Notice.as_github_level(), "notice");
    /// assert_eq!(AnnotationSeverity::Warning.as_github_level(), "warning");
    /// assert_eq!(AnnotationSeverity::Error.as_github_level(), "error");
    /// assert_eq!(AnnotationSeverity::Fatal.as_github_level(), "error");
    /// ```
    #[must_use]
    pub const fn as_github_level(self) -> &'static str {
        match self {
            Self::Notice => "notice",
            Self::Warning => "warning",
            Self::Error | Self::Fatal => "error",
        }
    }

    /// Converts the severity to a GitLab severity string.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_annotation_format::AnnotationSeverity;
    ///
    /// assert_eq!(AnnotationSeverity::Notice.as_gitlab_severity(), "info");
    /// assert_eq!(AnnotationSeverity::Warning.as_gitlab_severity(), "warning");
    /// assert_eq!(AnnotationSeverity::Error.as_gitlab_severity(), "error");
    /// assert_eq!(AnnotationSeverity::Fatal.as_gitlab_severity(), "critical");
    /// ```
    #[must_use]
    pub const fn as_gitlab_severity(self) -> &'static str {
        match self {
            Self::Notice => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Fatal => "critical",
        }
    }

    /// Converts the severity to an Azure DevOps log level string.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_annotation_format::AnnotationSeverity;
    ///
    /// assert_eq!(AnnotationSeverity::Notice.as_azure_level(), "information");
    /// assert_eq!(AnnotationSeverity::Warning.as_azure_level(), "warning");
    /// assert_eq!(AnnotationSeverity::Error.as_azure_level(), "error");
    /// assert_eq!(AnnotationSeverity::Fatal.as_azure_level(), "error");
    /// ```
    #[must_use]
    pub const fn as_azure_level(self) -> &'static str {
        match self {
            Self::Notice => "information",
            Self::Warning => "warning",
            Self::Error | Self::Fatal => "error",
        }
    }
}

/// Annotation data for CI output.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Annotation {
    /// File path for the annotation
    pub path: String,
    /// Line number (1-based)
    pub line: usize,
    /// Optional column number (1-based)
    pub column: Option<usize>,
    /// Severity level
    pub severity: AnnotationSeverity,
    /// Annotation message
    pub message: String,
}

impl Annotation {
    /// Creates a new annotation with the given parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_annotation_format::{Annotation, AnnotationSeverity};
    ///
    /// let annotation = Annotation::new(
    ///     "src/lib.rs",
    ///     42,
    ///     Some(10),
    ///     AnnotationSeverity::Warning,
    ///     "Unused variable"
    /// );
    ///
    /// assert_eq!(annotation.path, "src/lib.rs");
    /// assert_eq!(annotation.line, 42);
    /// assert_eq!(annotation.column, Some(10));
    /// ```
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        line: usize,
        column: Option<usize>,
        severity: AnnotationSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            line,
            column,
            severity,
            message: message.into(),
        }
    }

    /// Creates a simple annotation without column information.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_annotation_format::{Annotation, AnnotationSeverity};
    ///
    /// let annotation = Annotation::simple("src/lib.rs", 10, AnnotationSeverity::Error, "Error message");
    /// assert_eq!(annotation.column, None);
    /// ```
    #[must_use]
    pub fn simple(
        path: impl Into<String>,
        line: usize,
        severity: AnnotationSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            line,
            column: None,
            severity,
            message: message.into(),
        }
    }
}

/// Detects the current CI platform from environment variables.
///
/// # Example
///
/// ```
/// use lintdiff_annotation_format::{detect_ci, CiPlatform};
///
/// let platform = detect_ci();
/// println!("Detected CI platform: {:?}", platform);
/// ```
#[must_use]
pub fn detect_ci() -> CiPlatform {
    // Check for GitHub Actions
    if env::var("GITHUB_ACTIONS").is_ok_and(|v| v == "true") {
        return CiPlatform::GithubActions;
    }

    // Check for GitLab CI
    if env::var("GITLAB_CI").is_ok_and(|v| v == "true") {
        return CiPlatform::GitLabCI;
    }

    // Check for Azure DevOps (Azure Pipelines)
    if env::var("TF_BUILD").is_ok_and(|v| v == "True") {
        return CiPlatform::AzureDevOps;
    }

    // Check for CircleCI
    if env::var("CIRCLECI").is_ok_and(|v| v == "true") {
        return CiPlatform::CircleCI;
    }

    // Check for Travis CI
    if env::var("TRAVIS").is_ok_and(|v| v == "true") {
        return CiPlatform::TravisCI;
    }

    // Check for Jenkins
    if env::var("JENKINS_URL").is_ok() {
        return CiPlatform::Jenkins;
    }

    CiPlatform::Unknown
}

/// Checks if running in GitHub Actions.
///
/// # Example
///
/// ```
/// use lintdiff_annotation_format::is_github_actions;
///
/// if is_github_actions() {
///     println!("Running in GitHub Actions");
/// }
/// ```
#[must_use]
pub fn is_github_actions() -> bool {
    env::var("GITHUB_ACTIONS").is_ok_and(|v| v == "true")
}

/// Checks if running in GitLab CI.
///
/// # Example
///
/// ```
/// use lintdiff_annotation_format::is_gitlab_ci;
///
/// if is_gitlab_ci() {
///     println!("Running in GitLab CI");
/// }
/// ```
#[must_use]
pub fn is_gitlab_ci() -> bool {
    env::var("GITLAB_CI").is_ok_and(|v| v == "true")
}

/// Checks if running in Azure DevOps.
///
/// # Example
///
/// ```
/// use lintdiff_annotation_format::is_azure_devops;
///
/// if is_azure_devops() {
///     println!("Running in Azure DevOps");
/// }
/// ```
#[must_use]
pub fn is_azure_devops() -> bool {
    env::var("TF_BUILD").is_ok_and(|v| v == "True")
}

/// Checks if running in `CircleCI`.
///
/// # Example
///
/// ```
/// use lintdiff_annotation_format::is_circleci;
///
/// if is_circleci() {
///     println!("Running in CircleCI");
/// }
/// ```
#[must_use]
pub fn is_circleci() -> bool {
    env::var("CIRCLECI").is_ok_and(|v| v == "true")
}

/// Formats an annotation for the specified format.
///
/// This function resolves `AnnotationFormat::Auto` to the detected CI platform.
///
/// # Example
///
/// ```
/// use lintdiff_annotation_format::{Annotation, AnnotationFormat, AnnotationSeverity, format_annotation};
///
/// let annotation = Annotation::new(
///     "src/lib.rs",
///     42,
///     Some(10),
///     AnnotationSeverity::Warning,
///     "Unused variable"
/// );
///
/// let github_output = format_annotation(AnnotationFormat::Github, &annotation);
/// assert!(github_output.starts_with("::warning"));
/// ```
#[must_use]
pub fn format_annotation(format: AnnotationFormat, annotation: &Annotation) -> String {
    let resolved = format.resolve();
    match resolved {
        AnnotationFormat::Github => format_github_annotation(annotation),
        AnnotationFormat::Gitlab => format_gitlab_annotation(annotation),
        AnnotationFormat::Azure => format_azure_annotation(annotation),
        AnnotationFormat::CircleCI => format_circleci_annotation(annotation),
        AnnotationFormat::Default => format_default_annotation(annotation),
        AnnotationFormat::Auto => unreachable!("Auto should be resolved"),
    }
}

/// Formats an annotation in GitHub Actions format.
///
/// GitHub Actions format: `::{level} file={path},line={line},col={column}::{message}`
///
/// # Example
///
/// ```
/// use lintdiff_annotation_format::{Annotation, AnnotationSeverity, format_github_annotation};
///
/// let annotation = Annotation::new(
///     "src/lib.rs",
///     42,
///     Some(10),
///     AnnotationSeverity::Error,
///     "Missing return"
/// );
///
/// let output = format_github_annotation(&annotation);
/// assert!(output.starts_with("::error file=src/lib.rs"));
/// assert!(output.contains("line=42"));
/// assert!(output.contains("col=10"));
/// ```
#[must_use]
pub fn format_github_annotation(annotation: &Annotation) -> String {
    let level = annotation.severity.as_github_level();
    let message = escape_github_message(&annotation.message);

    annotation.column.map_or_else(
        || {
            format!(
                "::{} file={},line={}::{}",
                level, annotation.path, annotation.line, message
            )
        },
        |col| {
            format!(
                "::{} file={},line={},col={}::{}",
                level, annotation.path, annotation.line, col, message
            )
        },
    )
}

/// Formats an annotation in GitLab CI format.
///
/// GitLab CI format: JSON object with `file_path`, line, severity, and message.
///
/// # Example
///
/// ```
/// use lintdiff_annotation_format::{Annotation, AnnotationSeverity, format_gitlab_annotation};
///
/// let annotation = Annotation::new(
///     "src/lib.rs",
///     42,
///     None,
///     AnnotationSeverity::Warning,
///     "Unused variable"
/// );
///
/// let output = format_gitlab_annotation(&annotation);
/// assert!(output.contains("src/lib.rs"));
/// assert!(output.contains("warning"));
/// ```
#[must_use]
pub fn format_gitlab_annotation(annotation: &Annotation) -> String {
    let severity = annotation.severity.as_gitlab_severity();
    let message = escape_gitlab_message(&annotation.message);

    format!(
        "{}:{}: {}: {}",
        annotation.path, annotation.line, severity, message
    )
}

/// Formats an annotation in Azure DevOps format.
///
/// Azure DevOps format: `##vso[task.logissue type={level};sourcepath={path};linenumber={line}]{message}`
///
/// # Example
///
/// ```
/// use lintdiff_annotation_format::{Annotation, AnnotationSeverity, format_azure_annotation};
///
/// let annotation = Annotation::new(
///     "src/lib.rs",
///     42,
///     None,
///     AnnotationSeverity::Error,
///     "Build error"
/// );
///
/// let output = format_azure_annotation(&annotation);
/// assert!(output.starts_with("##vso[task.logissue"));
/// assert!(output.contains("type=error"));
/// ```
#[must_use]
pub fn format_azure_annotation(annotation: &Annotation) -> String {
    let level = annotation.severity.as_azure_level();
    let message = escape_azure_message(&annotation.message);

    format!(
        "##vso[task.logissue type={};sourcepath={};linenumber={}]{}",
        level, annotation.path, annotation.line, message
    )
}

/// Formats an annotation in `CircleCI` format.
///
/// `CircleCI` uses a simple text format similar to compiler warnings.
///
/// # Example
///
/// ```
/// use lintdiff_annotation_format::{Annotation, AnnotationSeverity, format_circleci_annotation};
///
/// let annotation = Annotation::new(
///     "src/lib.rs",
///     42,
///     Some(10),
///     AnnotationSeverity::Warning,
///     "Unused variable"
/// );
///
/// let output = format_circleci_annotation(&annotation);
/// assert!(output.starts_with("src/lib.rs:42:10:"));
/// ```
#[must_use]
pub fn format_circleci_annotation(annotation: &Annotation) -> String {
    let level = annotation.severity.as_github_level(); // CircleCI uses similar level names
    let message = &annotation.message;

    annotation.column.map_or_else(
        || {
            format!(
                "{}:{}: {}: {}",
                annotation.path, annotation.line, level, message
            )
        },
        |col| {
            format!(
                "{}:{}:{}: {}: {}",
                annotation.path, annotation.line, col, level, message
            )
        },
    )
}

/// Formats an annotation in default/plain text format.
///
/// # Example
///
/// ```
/// use lintdiff_annotation_format::{Annotation, AnnotationSeverity, format_default_annotation};
///
/// let annotation = Annotation::new(
///     "src/lib.rs",
///     42,
///     None,
///     AnnotationSeverity::Warning,
///     "Unused variable"
/// );
///
/// let output = format_default_annotation(&annotation);
/// assert!(output.contains("src/lib.rs"));
/// assert!(output.contains(":42:"));  // Format is path:line: severity: message
/// assert!(output.contains("Warning"));
/// ```
#[must_use]
pub fn format_default_annotation(annotation: &Annotation) -> String {
    let severity = format!("{:?}", annotation.severity);
    let col_str = annotation.column.map_or(String::new(), |c| format!(":{c}"));

    format!(
        "{}:{}{}: {}: {}",
        annotation.path, annotation.line, col_str, severity, annotation.message
    )
}

/// Escapes special characters for GitHub Actions workflow commands.
///
/// Characters that need escaping: `%`, `\r`, `\n`, `:`, `,`
fn escape_github_message(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '%' => vec!['%', '2', '5'],
            '\r' => vec!['%', '0', 'D'],
            '\n' => vec!['%', '0', 'A'],
            ':' => vec!['%', '3', 'A'],
            ',' => vec!['%', '2', 'C'],
            other => vec![other],
        })
        .collect()
}

/// Escapes special characters for GitLab CI output.
///
/// GitLab uses GCC-style output, so we escape newlines.
fn escape_gitlab_message(s: &str) -> String {
    s.replace('\r', "\\r").replace('\n', "\\n")
}

/// Escapes special characters for Azure DevOps logging commands.
///
/// Azure DevOps requires escaping `]`, `;`, and newlines.
fn escape_azure_message(s: &str) -> String {
    s.replace(']', "%5D")
        .replace(';', "%3B")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

#[cfg(test)]
mod tests {
    use super::*;

    mod ci_platform {
        use super::*;

        #[test]
        fn annotation_format_returns_correct_format() {
            assert_eq!(
                CiPlatform::GithubActions.annotation_format(),
                AnnotationFormat::Github
            );
            assert_eq!(
                CiPlatform::GitLabCI.annotation_format(),
                AnnotationFormat::Gitlab
            );
            assert_eq!(
                CiPlatform::AzureDevOps.annotation_format(),
                AnnotationFormat::Azure
            );
            assert_eq!(
                CiPlatform::CircleCI.annotation_format(),
                AnnotationFormat::CircleCI
            );
            assert_eq!(
                CiPlatform::TravisCI.annotation_format(),
                AnnotationFormat::Default
            );
            assert_eq!(
                CiPlatform::Jenkins.annotation_format(),
                AnnotationFormat::Default
            );
            assert_eq!(
                CiPlatform::Unknown.annotation_format(),
                AnnotationFormat::Default
            );
        }
    }

    mod annotation_format {
        use super::*;

        #[test]
        fn resolve_returns_self_for_non_auto() {
            assert_eq!(AnnotationFormat::Github.resolve(), AnnotationFormat::Github);
            assert_eq!(AnnotationFormat::Gitlab.resolve(), AnnotationFormat::Gitlab);
            assert_eq!(AnnotationFormat::Azure.resolve(), AnnotationFormat::Azure);
            assert_eq!(
                AnnotationFormat::CircleCI.resolve(),
                AnnotationFormat::CircleCI
            );
            assert_eq!(
                AnnotationFormat::Default.resolve(),
                AnnotationFormat::Default
            );
        }

        #[test]
        fn default_is_default() {
            assert_eq!(AnnotationFormat::default(), AnnotationFormat::Default);
        }
    }

    mod annotation_severity {
        use super::*;

        #[test]
        fn as_github_level_returns_correct_strings() {
            assert_eq!(AnnotationSeverity::Notice.as_github_level(), "notice");
            assert_eq!(AnnotationSeverity::Warning.as_github_level(), "warning");
            assert_eq!(AnnotationSeverity::Error.as_github_level(), "error");
            assert_eq!(AnnotationSeverity::Fatal.as_github_level(), "error");
        }

        #[test]
        fn as_gitlab_severity_returns_correct_strings() {
            assert_eq!(AnnotationSeverity::Notice.as_gitlab_severity(), "info");
            assert_eq!(AnnotationSeverity::Warning.as_gitlab_severity(), "warning");
            assert_eq!(AnnotationSeverity::Error.as_gitlab_severity(), "error");
            assert_eq!(AnnotationSeverity::Fatal.as_gitlab_severity(), "critical");
        }

        #[test]
        fn as_azure_level_returns_correct_strings() {
            assert_eq!(AnnotationSeverity::Notice.as_azure_level(), "information");
            assert_eq!(AnnotationSeverity::Warning.as_azure_level(), "warning");
            assert_eq!(AnnotationSeverity::Error.as_azure_level(), "error");
            assert_eq!(AnnotationSeverity::Fatal.as_azure_level(), "error");
        }

        #[test]
        fn default_is_warning() {
            assert_eq!(AnnotationSeverity::default(), AnnotationSeverity::Warning);
        }
    }

    mod annotation_struct {
        use super::*;

        #[test]
        fn new_creates_annotation_with_all_fields() {
            let annotation = Annotation::new(
                "src/lib.rs",
                42,
                Some(10),
                AnnotationSeverity::Error,
                "Test error",
            );

            assert_eq!(annotation.path, "src/lib.rs");
            assert_eq!(annotation.line, 42);
            assert_eq!(annotation.column, Some(10));
            assert_eq!(annotation.severity, AnnotationSeverity::Error);
            assert_eq!(annotation.message, "Test error");
        }

        #[test]
        fn simple_creates_annotation_without_column() {
            let annotation =
                Annotation::simple("main.rs", 1, AnnotationSeverity::Notice, "Info message");

            assert_eq!(annotation.path, "main.rs");
            assert_eq!(annotation.line, 1);
            assert_eq!(annotation.column, None);
            assert_eq!(annotation.severity, AnnotationSeverity::Notice);
            assert_eq!(annotation.message, "Info message");
        }
    }

    mod format_annotation_function {
        use super::*;

        #[test]
        fn format_annotation_github_with_column() {
            let annotation = Annotation::new(
                "src/lib.rs",
                42,
                Some(10),
                AnnotationSeverity::Warning,
                "Unused variable",
            );

            let output = format_annotation(AnnotationFormat::Github, &annotation);
            assert!(output.starts_with("::warning"));
            assert!(output.contains("file=src/lib.rs"));
            assert!(output.contains("line=42"));
            assert!(output.contains("col=10"));
        }

        #[test]
        fn format_annotation_github_without_column() {
            let annotation = Annotation::new(
                "src/main.rs",
                10,
                None,
                AnnotationSeverity::Error,
                "Build failed",
            );

            let output = format_annotation(AnnotationFormat::Github, &annotation);
            assert!(output.starts_with("::error"));
            assert!(output.contains("file=src/main.rs"));
            assert!(output.contains("line=10"));
            assert!(!output.contains("col="));
        }

        #[test]
        fn format_annotation_gitlab() {
            let annotation = Annotation::new(
                "src/lib.rs",
                42,
                None,
                AnnotationSeverity::Warning,
                "Unused variable",
            );

            let output = format_annotation(AnnotationFormat::Gitlab, &annotation);
            assert!(output.contains("src/lib.rs:42"));
            assert!(output.contains("warning"));
        }

        #[test]
        fn format_annotation_azure() {
            let annotation =
                Annotation::new("build.rs", 5, None, AnnotationSeverity::Error, "Error");

            let output = format_annotation(AnnotationFormat::Azure, &annotation);
            assert!(output.starts_with("##vso[task.logissue"));
            assert!(output.contains("type=error"));
            assert!(output.contains("sourcepath=build.rs"));
            assert!(output.contains("linenumber=5"));
        }

        #[test]
        fn format_annotation_circleci_with_column() {
            let annotation =
                Annotation::new("test.rs", 100, Some(5), AnnotationSeverity::Notice, "Note");

            let output = format_annotation(AnnotationFormat::CircleCI, &annotation);
            assert!(output.starts_with("test.rs:100:5:"));
            assert!(output.contains("notice"));
        }

        #[test]
        fn format_annotation_default() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Fatal, "Fatal error");

            let output = format_annotation(AnnotationFormat::Default, &annotation);
            assert!(output.contains("file.rs:1"));
            assert!(output.contains("Fatal"));
            assert!(output.contains("Fatal error"));
        }
    }

    mod github_format {
        use super::*;

        #[test]
        fn format_github_escapes_special_characters() {
            let annotation = Annotation::new(
                "src/lib.rs",
                1,
                None,
                AnnotationSeverity::Error,
                "Error: test, value%done",
            );

            let output = format_github_annotation(&annotation);
            // Colon should be escaped
            assert!(output.contains("%3A"));
            // Comma should be escaped
            assert!(output.contains("%2C"));
            // Percent should be escaped
            assert!(output.contains("%25"));
        }

        #[test]
        fn format_github_escapes_newlines() {
            let annotation = Annotation::new(
                "src/lib.rs",
                1,
                None,
                AnnotationSeverity::Error,
                "Line1\nLine2\rLine3",
            );

            let output = format_github_annotation(&annotation);
            assert!(output.contains("%0A")); // \n
            assert!(output.contains("%0D")); // \r
        }
    }

    mod gitlab_format {
        use super::*;

        #[test]
        fn format_gitlab_escapes_newlines() {
            let annotation = Annotation::new(
                "src/lib.rs",
                1,
                None,
                AnnotationSeverity::Warning,
                "Line1\nLine2\rLine3",
            );

            let output = format_gitlab_annotation(&annotation);
            assert!(output.contains("\\n"));
            assert!(output.contains("\\r"));
        }

        #[test]
        fn format_gitlab_uses_correct_severity() {
            let fatal = Annotation::simple("f.rs", 1, AnnotationSeverity::Fatal, "fatal");
            let error = Annotation::simple("e.rs", 1, AnnotationSeverity::Error, "error");
            let warning = Annotation::simple("w.rs", 1, AnnotationSeverity::Warning, "warning");
            let notice = Annotation::simple("n.rs", 1, AnnotationSeverity::Notice, "notice");

            assert!(format_gitlab_annotation(&fatal).contains("critical"));
            assert!(format_gitlab_annotation(&error).contains("error"));
            assert!(format_gitlab_annotation(&warning).contains("warning"));
            assert!(format_gitlab_annotation(&notice).contains("info"));
        }
    }

    mod azure_format {
        use super::*;

        #[test]
        fn format_azure_escapes_special_characters() {
            let annotation = Annotation::new(
                "src/lib.rs",
                1,
                None,
                AnnotationSeverity::Warning,
                "Error]; check; value",
            );

            let output = format_azure_annotation(&annotation);
            assert!(output.contains("%5D")); // ]
            assert!(output.contains("%3B")); // ;
        }

        #[test]
        fn format_azure_escapes_newlines() {
            let annotation = Annotation::new(
                "src/lib.rs",
                1,
                None,
                AnnotationSeverity::Error,
                "Line1\nLine2\rLine3",
            );

            let output = format_azure_annotation(&annotation);
            assert!(output.contains("%0A"));
            assert!(output.contains("%0D"));
        }
    }

    mod circleci_format {
        use super::*;

        #[test]
        fn format_circleci_includes_column_when_present() {
            let annotation =
                Annotation::new("test.rs", 10, Some(5), AnnotationSeverity::Error, "E");

            let output = format_circleci_annotation(&annotation);
            assert!(output.starts_with("test.rs:10:5:"));
        }

        #[test]
        fn format_circleci_omits_column_when_absent() {
            let annotation = Annotation::simple("test.rs", 10, AnnotationSeverity::Warning, "W");

            let output = format_circleci_annotation(&annotation);
            assert!(output.starts_with("test.rs:10:"));
            // Format is: path:line: level: message
            assert!(output.contains("warning"));
            assert!(output.ends_with("W"));
        }
    }

    mod default_format {
        use super::*;

        #[test]
        fn format_default_includes_all_info() {
            let annotation = Annotation::new(
                "src/main.rs",
                42,
                Some(10),
                AnnotationSeverity::Error,
                "Test error",
            );

            let output = format_default_annotation(&annotation);
            assert!(output.contains("src/main.rs"));
            assert!(output.contains("42:10"));
            assert!(output.contains("Error"));
            assert!(output.contains("Test error"));
        }

        #[test]
        fn format_default_without_column() {
            let annotation = Annotation::simple("lib.rs", 5, AnnotationSeverity::Warning, "Warn");

            let output = format_default_annotation(&annotation);
            assert!(output.starts_with("lib.rs:5:"));
            assert!(output.contains("Warning"));
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn empty_message() {
            let annotation = Annotation::simple("empty.rs", 1, AnnotationSeverity::Notice, "");

            let github = format_github_annotation(&annotation);
            assert!(github.ends_with("::")); // Empty message after ::

            let gitlab = format_gitlab_annotation(&annotation);
            assert!(gitlab.ends_with(": ")); // Empty message after colon space

            let azure = format_azure_annotation(&annotation);
            assert!(azure.ends_with(']')); // Empty message in brackets
        }

        #[test]
        fn special_characters_in_path() {
            let annotation = Annotation::new(
                "src/special-file_name.rs",
                1,
                None,
                AnnotationSeverity::Warning,
                "Test",
            );

            let output = format_github_annotation(&annotation);
            assert!(output.contains("special-file_name.rs"));
        }

        #[test]
        fn unicode_in_message() {
            let annotation = Annotation::simple(
                "unicode.rs",
                1,
                AnnotationSeverity::Error,
                "Error: 你好世界 🌍",
            );

            let output = format_default_annotation(&annotation);
            assert!(output.contains("你好世界"));
            assert!(output.contains("🌍"));
        }

        #[test]
        fn very_long_message() {
            let long_message = "x".repeat(1000);
            let annotation =
                Annotation::simple("long.rs", 1, AnnotationSeverity::Warning, &long_message);

            let output = format_github_annotation(&annotation);
            assert!(output.contains(&long_message));
        }

        #[test]
        fn line_number_one() {
            let annotation =
                Annotation::simple("first.rs", 1, AnnotationSeverity::Error, "First line");

            let output = format_github_annotation(&annotation);
            assert!(output.contains("line=1"));
        }

        #[test]
        fn column_number_one() {
            let annotation =
                Annotation::new("first.rs", 1, Some(1), AnnotationSeverity::Error, "E");

            let output = format_github_annotation(&annotation);
            assert!(output.contains("col=1"));
        }
    }
}
