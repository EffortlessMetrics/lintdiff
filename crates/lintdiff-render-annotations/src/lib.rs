//! GitHub Actions annotations rendering of findings for lintdiff.
//!
//! This microcrate provides a single responsibility: rendering findings as
//! GitHub Actions workflow commands for annotations. These annotations appear
//! in the GitHub Actions UI on PR checks and workflow runs.
//!
//! # GitHub Actions Annotation Format
//!
//! Annotations follow the workflow command format:
//! ```text
//! ::{level} file={file},line={line},title={title}::{message}
//! ```
//!
//! Where `level` is one of: `error`, `warning`, or `notice`.
//!
//! # Example: Rendering Annotations
//!
//! ```
//! use lintdiff_render_annotations::{render_annotations, AnnotationsConfig};
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
//! let config = AnnotationsConfig::default();
//! let output = render_annotations(&findings, &config);
//! assert!(output.contains("::error"));
//! assert!(output.contains("file=src/lib.rs"));
//! assert!(output.contains("line=10"));
//! ```
//!
//! # Example: Single Finding Annotation
//!
//! ```
//! use lintdiff_render_annotations::render_finding_annotation;
//! use lintdiff_types::{Finding, Severity, Location, NormPath};
//!
//! let finding = Finding {
//!     severity: Severity::Warn,
//!     code: "clippy::map_identity".to_string(),
//!     message: "redundant mapping operation".to_string(),
//!     location: Some(Location {
//!         path: NormPath::new("src/main.rs"),
//!         line: Some(42),
//!         col: Some(5),
//!     }),
//!     check_id: None,
//!     help: None,
//!     url: None,
//!     fingerprint: None,
//!     data: None,
//! };
//!
//! let annotation = render_finding_annotation(&finding);
//! assert!(annotation.contains("::warning"));
//! // Note: colons are escaped for GitHub Actions format
//! assert!(annotation.contains("clippy%3A%3Amap_identity"));
//! ```

#![warn(missing_docs)]

use lintdiff_types::{Finding, Severity};

/// Configuration for GitHub annotations rendering.
///
/// Controls which findings are rendered as annotations and limits
/// the total number to avoid overwhelming the GitHub Actions UI.
///
/// # Example
///
/// ```
/// use lintdiff_render_annotations::AnnotationsConfig;
///
/// // Use defaults
/// let config = AnnotationsConfig::default();
/// assert_eq!(config.max_annotations, 50);
/// assert!(config.include_errors);
/// assert!(config.include_warnings);
/// assert!(!config.include_notes);
///
/// // Customize for errors only
/// let errors_only = AnnotationsConfig {
///     max_annotations: 10,
///     include_errors: true,
///     include_warnings: false,
///     include_notes: false,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct AnnotationsConfig {
    /// Maximum number of annotations to render.
    ///
    /// GitHub Actions has limits on annotations, so this prevents
    /// overwhelming the UI. Defaults to 50.
    pub max_annotations: usize,

    /// Whether to include error-level findings.
    ///
    /// When `true`, findings with `Severity::Error` are rendered
    /// as `::error` annotations. Defaults to `true`.
    pub include_errors: bool,

    /// Whether to include warning-level findings.
    ///
    /// When `true`, findings with `Severity::Warn` are rendered
    /// as `::warning` annotations. Defaults to `true`.
    pub include_warnings: bool,

    /// Whether to include note/info-level findings.
    ///
    /// When `true`, findings with `Severity::Info` are rendered
    /// as `::notice` annotations. Defaults to `false` to reduce noise.
    pub include_notes: bool,
}

impl Default for AnnotationsConfig {
    fn default() -> Self {
        Self {
            max_annotations: 50,
            include_errors: true,
            include_warnings: true,
            include_notes: false,
        }
    }
}

impl AnnotationsConfig {
    /// Checks if a finding's severity should be included based on the config.
    const fn should_include(&self, severity: Severity) -> bool {
        match severity {
            Severity::Error => self.include_errors,
            Severity::Warn => self.include_warnings,
            Severity::Info => self.include_notes,
        }
    }
}

/// Render findings as GitHub Actions annotations.
///
/// This function filters findings based on the configuration and renders
/// them as GitHub Actions workflow commands. The output can be written
/// directly to stdout for GitHub Actions to process.
///
/// # Arguments
///
/// * `findings` - Slice of findings to render
/// * `config` - Configuration controlling which findings to include
///
/// # Returns
///
/// A string containing zero or more annotation lines, each ending with a newline.
/// If no findings match the filter criteria, returns an empty string.
///
/// # Example
///
/// ```
/// use lintdiff_render_annotations::{render_annotations, AnnotationsConfig};
/// use lintdiff_types::{Finding, Severity, Location, NormPath};
///
/// let findings = vec![
///     Finding {
///         severity: Severity::Error,
///         code: "E001".to_string(),
///         message: "an error".to_string(),
///         location: Some(Location {
///             path: NormPath::new("src/lib.rs"),
///             line: Some(1),
///             col: None,
///         }),
///         check_id: None,
///         help: None,
///         url: None,
///         fingerprint: None,
///         data: None,
///     },
///     Finding {
///         severity: Severity::Info,
///         code: "I001".to_string(),
///         message: "a note".to_string(),
///         location: None,
///         check_id: None,
///         help: None,
///         url: None,
///         fingerprint: None,
///         data: None,
///     },
/// ];
///
/// // Default config excludes notes
/// let config = AnnotationsConfig::default();
/// let output = render_annotations(&findings, &config);
/// assert!(output.contains("E001"));
/// assert!(!output.contains("I001"));
/// ```
#[must_use]
pub fn render_annotations(findings: &[Finding], config: &AnnotationsConfig) -> String {
    let mut result = String::new();

    for finding in findings
        .iter()
        .filter(|f| config.should_include(f.severity))
        .take(config.max_annotations)
    {
        result.push_str(&render_finding_annotation(finding));
        result.push('\n');
    }

    result
}

/// Render a single finding as a GitHub annotation.
///
/// This function renders one finding as a GitHub Actions workflow command.
/// The annotation level is determined by the finding's severity:
/// - `Severity::Error` → `::error`
/// - `Severity::Warn` → `::warning`
/// - `Severity::Info` → `::notice`
///
/// # Arguments
///
/// * `finding` - The finding to render
///
/// # Returns
///
/// A single annotation line in the GitHub Actions workflow command format.
/// The line does NOT include a trailing newline.
///
/// # Annotation Format
///
/// ```text
/// ::{level} file={path},line={line},title={code}::{message}
/// ```
///
/// If the finding has no location, the file and line parameters are omitted.
///
/// # Example
///
/// ```
/// use lintdiff_render_annotations::render_finding_annotation;
/// use lintdiff_types::{Finding, Severity, Location, NormPath};
///
/// let finding = Finding {
///     severity: Severity::Error,
///     code: "clippy::unwrap_used".to_string(),
///     message: "called `.unwrap()`".to_string(),
///     location: Some(Location {
///         path: NormPath::new("src/lib.rs"),
///         line: Some(42),
///         col: None,
///     }),
///     check_id: None,
///     help: None,
///     url: None,
///     fingerprint: None,
///     data: None,
/// };
///
/// let annotation = render_finding_annotation(&finding);
/// assert!(annotation.starts_with("::error"));
/// assert!(annotation.contains("file=src/lib.rs"));
/// assert!(annotation.contains("line=42"));
/// // Note: colons are escaped for GitHub Actions format
/// assert!(annotation.contains("title=clippy%3A%3Aunwrap_used"));
/// assert!(annotation.ends_with("called `.unwrap()`"));
/// ```
#[must_use]
pub fn render_finding_annotation(finding: &Finding) -> String {
    let level = severity_to_level(finding.severity);

    finding.location.as_ref().map_or_else(
        || {
            let message = escape_message(&finding.message);
            let title = escape_message(&finding.code);

            format!("::{level} title={title}::{message}")
        },
        |location| {
            let file = location.path.as_str();
            let line = location.line.unwrap_or(1);

            // Escape special characters in message for GitHub Actions
            let message = escape_message(&finding.message);
            let title = escape_message(&finding.code);

            format!("::{level} file={file},line={line},title={title}::{message}")
        },
    )
}

/// Convert a severity level to a GitHub annotation level.
const fn severity_to_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warn => "warning",
        Severity::Info => "notice",
    }
}

/// Escape special characters in a message for GitHub Actions workflow commands.
///
/// GitHub Actions workflow commands require certain characters to be escaped:
/// - `%` → `%25`
/// - `\r` → `%0D`
/// - `\n` → `%0A`
/// - `:` → `%3A`
/// - `,` → `%2C`
fn escape_message(s: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use lintdiff_types::{Location, NormPath};

    fn create_test_finding(severity: Severity, code: &str, message: &str) -> Finding {
        Finding {
            severity,
            code: code.to_string(),
            message: message.to_string(),
            location: Some(Location {
                path: NormPath::new("src/test.rs"),
                line: Some(10),
                col: None,
            }),
            check_id: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        }
    }

    fn create_test_finding_no_location(severity: Severity, code: &str, message: &str) -> Finding {
        Finding {
            severity,
            code: code.to_string(),
            message: message.to_string(),
            location: None,
            check_id: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        }
    }

    #[test]
    fn test_empty_findings() {
        let findings: Vec<Finding> = vec![];
        let config = AnnotationsConfig::default();
        let result = render_annotations(&findings, &config);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_error_finding() {
        let findings = vec![create_test_finding(
            Severity::Error,
            "E001",
            "test error message",
        )];
        let config = AnnotationsConfig::default();
        let result = render_annotations(&findings, &config);

        assert!(result.contains("::error"));
        assert!(result.contains("file=src/test.rs"));
        assert!(result.contains("line=10"));
        assert!(result.contains("title=E001"));
        assert!(result.contains("test error message"));
    }

    #[test]
    fn test_single_warning_finding() {
        let findings = vec![create_test_finding(
            Severity::Warn,
            "W001",
            "test warning message",
        )];
        let config = AnnotationsConfig::default();
        let result = render_annotations(&findings, &config);

        assert!(result.contains("::warning"));
        assert!(result.contains("title=W001"));
    }

    #[test]
    fn test_single_info_finding() {
        let findings = vec![create_test_finding(
            Severity::Info,
            "I001",
            "test info message",
        )];
        let config = AnnotationsConfig::default();
        let result = render_annotations(&findings, &config);

        // Default config excludes notes
        assert!(!result.contains("I001"));
        assert!(result.is_empty());
    }

    #[test]
    fn test_info_finding_included_when_enabled() {
        let findings = vec![create_test_finding(
            Severity::Info,
            "I001",
            "test info message",
        )];
        let config = AnnotationsConfig {
            include_notes: true,
            ..Default::default()
        };
        let result = render_annotations(&findings, &config);

        assert!(result.contains("::notice"));
        assert!(result.contains("I001"));
    }

    #[test]
    fn test_multiple_findings() {
        let findings = vec![
            create_test_finding(Severity::Error, "E001", "error 1"),
            create_test_finding(Severity::Warn, "W001", "warning 1"),
            create_test_finding(Severity::Error, "E002", "error 2"),
        ];
        let config = AnnotationsConfig::default();
        let result = render_annotations(&findings, &config);

        assert!(result.contains("E001"));
        assert!(result.contains("W001"));
        assert!(result.contains("E002"));
        assert!(result.lines().count() == 3);
    }

    #[test]
    fn test_max_annotations_limit() {
        let findings: Vec<Finding> = (0..10)
            .map(|i| create_test_finding(Severity::Error, &format!("E{i:03}"), "error"))
            .collect();

        let config = AnnotationsConfig {
            max_annotations: 3,
            ..Default::default()
        };
        let result = render_annotations(&findings, &config);

        assert_eq!(result.lines().count(), 3);
        assert!(result.contains("E000"));
        assert!(result.contains("E001"));
        assert!(result.contains("E002"));
        assert!(!result.contains("E003"));
    }

    #[test]
    fn test_severity_filtering() {
        let findings = vec![
            create_test_finding(Severity::Error, "E001", "error"),
            create_test_finding(Severity::Warn, "W001", "warning"),
            create_test_finding(Severity::Info, "I001", "info"),
        ];

        let config = AnnotationsConfig {
            include_errors: true,
            include_warnings: false,
            include_notes: false,
            ..Default::default()
        };
        let result = render_annotations(&findings, &config);

        assert!(result.contains("E001"));
        assert!(!result.contains("W001"));
        assert!(!result.contains("I001"));
    }

    #[test]
    fn test_finding_without_location() {
        let finding = create_test_finding_no_location(Severity::Error, "E001", "no location error");
        let annotation = render_finding_annotation(&finding);

        assert!(annotation.starts_with("::error"));
        assert!(annotation.contains("title=E001"));
        assert!(annotation.contains("no location error"));
        assert!(!annotation.contains("file="));
        assert!(!annotation.contains("line="));
    }

    #[test]
    fn test_escape_special_characters() {
        assert_eq!(escape_message("hello:world"), "hello%3Aworld");
        assert_eq!(escape_message("a,b,c"), "a%2Cb%2Cc");
        assert_eq!(escape_message("100%"), "100%25");
        assert_eq!(escape_message("line1\nline2"), "line1%0Aline2");
        assert_eq!(escape_message("line1\r\nline2"), "line1%0D%0Aline2");
    }

    #[test]
    fn test_annotation_format_compliance() {
        let finding = create_test_finding(Severity::Error, "E001", "test message");
        let annotation = render_finding_annotation(&finding);

        // Verify format: ::{level} file={file},line={line},title={title}::{message}
        assert!(annotation.starts_with("::error "));
        assert!(annotation.contains("file=src/test.rs"));
        assert!(annotation.contains(",line=10"));
        assert!(annotation.contains(",title=E001"));
        assert!(annotation.contains("::test message"));
    }

    #[test]
    fn test_config_default_values() {
        let config = AnnotationsConfig::default();
        assert_eq!(config.max_annotations, 50);
        assert!(config.include_errors);
        assert!(config.include_warnings);
        assert!(!config.include_notes);
    }

    #[test]
    fn test_severity_to_level() {
        assert_eq!(severity_to_level(Severity::Error), "error");
        assert_eq!(severity_to_level(Severity::Warn), "warning");
        assert_eq!(severity_to_level(Severity::Info), "notice");
    }
}
