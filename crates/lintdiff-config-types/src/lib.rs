//! Configuration types for lintdiff.
//!
//! Provides shared configuration types used across multiple
//! crates for consistent configuration handling.

use std::fmt;
use std::path::PathBuf;

/// Output format options.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OutputFormat {
    /// Human-readable text output.
    #[default]
    Text = 0,
    /// JSON output.
    Json = 1,
    /// GitHub Actions annotations.
    GitHub = 2,
    /// Markdown format.
    Markdown = 3,
}

impl OutputFormat {
    /// Parse from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the format string is not recognized.
    pub fn parse(s: &str) -> Result<Self, ConfigParseError> {
        match s.to_lowercase().as_str() {
            "text" | "txt" | "plain" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "github" | "gh" | "actions" => Ok(Self::GitHub),
            "markdown" | "md" => Ok(Self::Markdown),
            _ => Err(ConfigParseError::InvalidFormat(s.to_string())),
        }
    }

    /// Get file extension for this format.
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Text | Self::GitHub => "txt",
            Self::Markdown => "md",
        }
    }

    /// Check if this is a machine-readable format.
    #[must_use]
    pub const fn is_machine_readable(&self) -> bool {
        matches!(self, Self::Json)
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Json => write!(f, "json"),
            Self::GitHub => write!(f, "github"),
            Self::Markdown => write!(f, "markdown"),
        }
    }
}

/// Failure mode configuration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FailOn {
    /// Never fail (always exit 0).
    Never = 0,
    /// Fail on any new error.
    #[default]
    Error = 1,
    /// Fail on any new warning or error.
    Warning = 2,
    /// Fail on any new issue (hint and above).
    Any = 3,
}

impl FailOn {
    /// Parse from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the fail-on string is not recognized.
    pub fn parse(s: &str) -> Result<Self, ConfigParseError> {
        match s.to_lowercase().as_str() {
            "never" | "none" | "off" => Ok(Self::Never),
            "error" | "errors" => Ok(Self::Error),
            "warning" | "warnings" | "warn" => Ok(Self::Warning),
            "any" | "all" => Ok(Self::Any),
            _ => Err(ConfigParseError::InvalidFailOn(s.to_string())),
        }
    }

    /// Get the minimum severity level that triggers failure.
    #[must_use]
    pub const fn min_severity(&self) -> u8 {
        match self {
            Self::Never => 255, // Never triggers
            Self::Error => 3,   // Error level
            Self::Warning => 2, // Warning level
            Self::Any => 0,     // Any level
        }
    }
}

impl fmt::Display for FailOn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Never => write!(f, "never"),
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Any => write!(f, "any"),
        }
    }
}

/// File source configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum FileSource {
    /// Read from stdin.
    #[default]
    Stdin,
    /// Read from a file path.
    Path(PathBuf),
    /// Read from inline content.
    Inline(String),
}

impl FileSource {
    /// Create a path source.
    #[must_use]
    pub fn path(p: impl Into<PathBuf>) -> Self {
        Self::Path(p.into())
    }

    /// Create an inline source.
    #[must_use]
    pub fn inline(content: impl Into<String>) -> Self {
        Self::Inline(content.into())
    }

    /// Check if this is stdin.
    #[must_use]
    pub const fn is_stdin(&self) -> bool {
        matches!(self, Self::Stdin)
    }

    /// Check if this is a file path.
    #[must_use]
    pub const fn is_path(&self) -> bool {
        matches!(self, Self::Path(_))
    }

    /// Get the path if this is a file path.
    #[must_use]
    pub const fn as_path(&self) -> Option<&PathBuf> {
        match self {
            Self::Path(p) => Some(p),
            _ => None,
        }
    }
}

impl From<PathBuf> for FileSource {
    fn from(path: PathBuf) -> Self {
        Self::Path(path)
    }
}

impl From<&str> for FileSource {
    fn from(s: &str) -> Self {
        Self::Path(PathBuf::from(s))
    }
}

/// Suppression rule for ignoring diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuppressRule {
    /// Rule name/identifier.
    pub name: String,
    /// Code pattern to suppress (supports glob).
    pub code: Option<String>,
    /// File pattern to suppress (supports glob).
    pub path: Option<String>,
    /// Reason for suppression.
    pub reason: Option<String>,
}

impl SuppressRule {
    /// Create a new suppression rule.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            code: None,
            path: None,
            reason: None,
        }
    }

    /// Set the code pattern.
    #[must_use]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set the path pattern.
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the reason.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

/// Error when parsing configuration.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigParseError {
    /// Invalid output format.
    #[error("Invalid output format: '{0}'")]
    InvalidFormat(String),
    /// Invalid fail-on value.
    #[error("Invalid fail-on value: '{0}'")]
    InvalidFailOn(String),
    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_format_default_is_text() {
        assert_eq!(OutputFormat::default(), OutputFormat::Text);
    }

    #[test]
    fn fail_on_default_is_error() {
        assert_eq!(FailOn::default(), FailOn::Error);
    }

    #[test]
    fn file_source_default_is_stdin() {
        assert_eq!(FileSource::default(), FileSource::Stdin);
    }
}
