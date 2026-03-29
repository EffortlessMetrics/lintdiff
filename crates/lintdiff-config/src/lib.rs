//! Configuration types and validation for lintdiff.
//!
//! This crate provides the main configuration types used by lintdiff,
//! including the [`LintdiffConfig`] struct, [`FailOn`] policy, and
//! [`OutputFormat`] settings.
//!
//! # Example
//!
//! ```rust
//! use lintdiff_config::{LintdiffConfig, FailOn, OutputFormat};
//!
//! // Create a default configuration
//! let config = LintdiffConfig::new();
//! assert_eq!(config.fail_on, FailOn::Error);
//! assert_eq!(config.output, OutputFormat::Json);
//!
//! // Parse from TOML
//! let toml = r#"
//! fail_on = "warning"
//! suppress = ["unused_variables"]
//! workspace_only = true
//! output = "markdown"
//! "#;
//! let config = LintdiffConfig::from_toml(toml).unwrap();
//! assert_eq!(config.fail_on, FailOn::Warning);
//! ```

#![warn(missing_docs)]

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to read the config file.
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),

    /// Failed to parse TOML content.
    #[error("Failed to parse TOML: {0}")]
    ParseError(String),

    /// Invalid configuration values.
    #[error("Invalid configuration: {0}")]
    ValidationError(String),
}

/// Policy for failing on diagnostics.
///
/// This enum controls the severity level at which lintdiff will
/// return a non-zero exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FailOn {
    /// Fail on errors only.
    #[default]
    Error,

    /// Fail on errors or warnings.
    Warning,

    /// Fail on any diagnostic (including notes).
    Note,
}

impl std::fmt::Display for FailOn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Note => write!(f, "note"),
        }
    }
}

impl std::str::FromStr for FailOn {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Self::Error),
            "warning" | "warn" => Ok(Self::Warning),
            "note" => Ok(Self::Note),
            other => Err(ConfigError::ValidationError(format!(
                "invalid fail_on value: {other} (expected error, warning, or note)"
            ))),
        }
    }
}

/// Output format configuration.
///
/// Controls how lintdiff presents its findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// JSON report format.
    #[default]
    Json,

    /// Markdown summary format.
    Markdown,

    /// GitHub Actions annotations format.
    Annotations,

    /// Human-readable text format.
    Text,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Markdown => write!(f, "markdown"),
            Self::Annotations => write!(f, "annotations"),
            Self::Text => write!(f, "text"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "markdown" | "md" => Ok(Self::Markdown),
            "annotations" | "github" => Ok(Self::Annotations),
            "text" => Ok(Self::Text),
            other => Err(ConfigError::ValidationError(format!(
                "invalid output format: {other} (expected json, markdown, annotations, or text)"
            ))),
        }
    }
}

/// Main configuration for lintdiff.
///
/// This struct holds all configuration options that control how
/// lintdiff processes diagnostics and generates output.
///
/// # Example
///
/// ```rust
/// use lintdiff_config::{LintdiffConfig, FailOn, OutputFormat};
///
/// let config = LintdiffConfig {
///     fail_on: FailOn::Warning,
///     suppress: vec!["unused_variables".to_string()],
///     deny: vec!["unsafe_code".to_string()],
///     workspace_only: true,
///     output: OutputFormat::Markdown,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LintdiffConfig {
    /// Policy for which diagnostics fail the build.
    pub fail_on: FailOn,

    /// Codes to suppress (never fail).
    pub suppress: Vec<String>,

    /// Codes that always fail (deny list).
    pub deny: Vec<String>,

    /// Whether to only check workspace files.
    pub workspace_only: bool,

    /// Output format.
    pub output: OutputFormat,
}

impl Default for LintdiffConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl LintdiffConfig {
    /// Create a new config with defaults.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lintdiff_config::{LintdiffConfig, FailOn, OutputFormat};
    ///
    /// let config = LintdiffConfig::new();
    /// assert_eq!(config.fail_on, FailOn::Error);
    /// assert_eq!(config.output, OutputFormat::Json);
    /// assert!(config.suppress.is_empty());
    /// assert!(config.deny.is_empty());
    /// assert!(config.workspace_only);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            fail_on: FailOn::default(),
            suppress: Vec::new(),
            deny: Vec::new(),
            workspace_only: true,
            output: OutputFormat::default(),
        }
    }

    /// Load config from a file.
    ///
    /// The file must be valid TOML containing a lintdiff configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::IoError`] if the file cannot be read.
    /// Returns [`ConfigError::ParseError`] if the TOML is invalid.
    /// Returns [`ConfigError::ValidationError`] if the configuration is invalid.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use lintdiff_config::LintdiffConfig;
    ///
    /// let config = LintdiffConfig::from_file(Path::new("lintdiff.toml")).unwrap();
    /// ```
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }

    /// Load config from a TOML string.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::ParseError`] if the TOML is invalid.
    /// Returns [`ConfigError::ValidationError`] if the configuration is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lintdiff_config::{LintdiffConfig, FailOn};
    ///
    /// let toml = r#"
    /// fail_on = "warning"
    /// suppress = ["unused_variables"]
    /// "#;
    /// let config = LintdiffConfig::from_toml(toml).unwrap();
    /// assert_eq!(config.fail_on, FailOn::Warning);
    /// ```
    pub fn from_toml(s: &str) -> Result<Self, ConfigError> {
        let config: Self = toml::from_str(s).map_err(|e| ConfigError::ParseError(e.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration.
    ///
    /// Checks for conflicting settings and invalid values.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::ValidationError`] if the configuration is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lintdiff_config::LintdiffConfig;
    ///
    /// let config = LintdiffConfig::new();
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Check for codes that are both suppressed and denied
        let conflicts: Vec<&String> = self
            .suppress
            .iter()
            .filter(|code| self.deny.contains(code))
            .collect();

        if !conflicts.is_empty() {
            let conflict_list: Vec<&str> = conflicts.iter().map(|s| s.as_str()).collect();
            return Err(ConfigError::ValidationError(format!(
                "codes cannot be both suppressed and denied: {}",
                conflict_list.join(", ")
            )));
        }

        // Check for empty codes
        let empty_suppress: bool = self.suppress.iter().any(|s| s.trim().is_empty());
        let empty_deny: bool = self.deny.iter().any(|s| s.trim().is_empty());

        if empty_suppress || empty_deny {
            return Err(ConfigError::ValidationError(
                "code lists cannot contain empty strings".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if a code is suppressed.
    ///
    /// Suppressed codes will never cause a build failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lintdiff_config::LintdiffConfig;
    ///
    /// let config = LintdiffConfig {
    ///     suppress: vec!["unused_variables".to_string()],
    ///     ..LintdiffConfig::new()
    /// };
    /// assert!(config.is_suppressed("unused_variables"));
    /// assert!(!config.is_suppressed("dead_code"));
    /// ```
    #[must_use]
    pub fn is_suppressed(&self, code: &str) -> bool {
        self.suppress.iter().any(|c| c == code)
    }

    /// Check if a code is denied.
    ///
    /// Denied codes will always cause a build failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lintdiff_config::LintdiffConfig;
    ///
    /// let config = LintdiffConfig {
    ///     deny: vec!["unsafe_code".to_string()],
    ///     ..LintdiffConfig::new()
    /// };
    /// assert!(config.is_denied("unsafe_code"));
    /// assert!(!config.is_denied("unused_variables"));
    /// ```
    #[must_use]
    pub fn is_denied(&self, code: &str) -> bool {
        self.deny.iter().any(|c| c == code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LintdiffConfig::new();
        assert_eq!(config.fail_on, FailOn::Error);
        assert_eq!(config.output, OutputFormat::Json);
        assert!(config.suppress.is_empty());
        assert!(config.deny.is_empty());
        assert!(config.workspace_only);
    }

    #[test]
    fn test_fail_on_default() {
        assert_eq!(FailOn::default(), FailOn::Error);
    }

    #[test]
    fn test_output_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Json);
    }

    #[test]
    fn test_fail_on_from_str() {
        assert_eq!("error".parse::<FailOn>().unwrap(), FailOn::Error);
        assert_eq!("warning".parse::<FailOn>().unwrap(), FailOn::Warning);
        assert_eq!("warn".parse::<FailOn>().unwrap(), FailOn::Warning);
        assert_eq!("note".parse::<FailOn>().unwrap(), FailOn::Note);
        assert!("invalid".parse::<FailOn>().is_err());
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!(
            "markdown".parse::<OutputFormat>().unwrap(),
            OutputFormat::Markdown
        );
        assert_eq!(
            "md".parse::<OutputFormat>().unwrap(),
            OutputFormat::Markdown
        );
        assert_eq!(
            "annotations".parse::<OutputFormat>().unwrap(),
            OutputFormat::Annotations
        );
        assert_eq!(
            "github".parse::<OutputFormat>().unwrap(),
            OutputFormat::Annotations
        );
        assert_eq!("text".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
        assert!("invalid".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_from_toml() {
        let toml = r#"
            fail_on = "warning"
            suppress = ["unused_variables", "dead_code"]
            deny = ["unsafe_code"]
            workspace_only = false
            output = "markdown"
        "#;
        let config = LintdiffConfig::from_toml(toml).unwrap();
        assert_eq!(config.fail_on, FailOn::Warning);
        assert_eq!(config.suppress, vec!["unused_variables", "dead_code"]);
        assert_eq!(config.deny, vec!["unsafe_code"]);
        assert!(!config.workspace_only);
        assert_eq!(config.output, OutputFormat::Markdown);
    }

    #[test]
    fn test_from_toml_partial() {
        let toml = r#"
            fail_on = "note"
        "#;
        let config = LintdiffConfig::from_toml(toml).unwrap();
        assert_eq!(config.fail_on, FailOn::Note);
        assert!(config.suppress.is_empty());
        assert!(config.deny.is_empty());
        assert!(config.workspace_only);
        assert_eq!(config.output, OutputFormat::Json);
    }

    #[test]
    fn test_from_toml_invalid() {
        let toml = r#"
            fail_on = "invalid_value"
        "#;
        assert!(LintdiffConfig::from_toml(toml).is_err());
    }

    #[test]
    fn test_validate_conflict() {
        let config = LintdiffConfig {
            suppress: vec!["unused".to_string()],
            deny: vec!["unused".to_string()],
            ..LintdiffConfig::new()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_codes() {
        let config = LintdiffConfig {
            suppress: vec![String::new()],
            ..LintdiffConfig::new()
        };
        assert!(config.validate().is_err());

        let config = LintdiffConfig {
            deny: vec!["   ".to_string()],
            ..LintdiffConfig::new()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_is_suppressed() {
        let config = LintdiffConfig {
            suppress: vec!["unused_variables".to_string(), "dead_code".to_string()],
            ..LintdiffConfig::new()
        };
        assert!(config.is_suppressed("unused_variables"));
        assert!(config.is_suppressed("dead_code"));
        assert!(!config.is_suppressed("unsafe_code"));
    }

    #[test]
    fn test_is_denied() {
        let config = LintdiffConfig {
            deny: vec!["unsafe_code".to_string(), "deprecated".to_string()],
            ..LintdiffConfig::new()
        };
        assert!(config.is_denied("unsafe_code"));
        assert!(config.is_denied("deprecated"));
        assert!(!config.is_denied("unused_variables"));
    }

    #[test]
    fn test_display_implementations() {
        assert_eq!(FailOn::Error.to_string(), "error");
        assert_eq!(FailOn::Warning.to_string(), "warning");
        assert_eq!(FailOn::Note.to_string(), "note");

        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Markdown.to_string(), "markdown");
        assert_eq!(OutputFormat::Annotations.to_string(), "annotations");
        assert_eq!(OutputFormat::Text.to_string(), "text");
    }
}
