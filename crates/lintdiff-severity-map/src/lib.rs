//! Severity mapping utilities for converting between linter-specific and canonical severity levels.
//!
//! This crate provides a standardized way to map severity levels from different
//! linters to a canonical representation, enabling consistent handling across
//! multiple linting tools.
//!
//! # Overview
//!
//! - [`CanonicalSeverity`]: Standard severity levels used internally
//! - [`SeverityMapper`]: Maps linter-specific severities to canonical
//! - [`SeverityMapBuilder`]: Builder pattern for custom mappers
//! - [`map_severity`]: Convenience function using default mapper
//! - [`is_error_level`]: Check if severity is error or above
//!
//! # Supported Linters
//!
//! Built-in mappings are provided for:
//! - **eslint**: "error", "warn", "info", "off", "2", "1", "0"
//! - **rustc**: "error", "warning", "note", "help"
//! - **pylint**: "fatal", "error", "warning", "convention", "refactor", "info"
//! - **golint**: standard Go linter severity output
//! - **shellcheck**: "error", "warning", "info", "style"
//!
//! # Example
//!
//! ```
//! use lintdiff_severity_map::{CanonicalSeverity, SeverityMapper, map_severity, is_error_level};
//!
//! // Use the convenience function with defaults
//! let severity = map_severity("eslint", "error");
//! assert_eq!(severity, CanonicalSeverity::Error);
//!
//! // Check if it's an error level
//! assert!(is_error_level(&severity));
//!
//! // Create a custom mapper
//! let mut mapper = SeverityMapper::new();
//! mapper.add_mapping("my-linter", "critical", CanonicalSeverity::Error);
//! assert_eq!(mapper.map("my-linter", "critical"), CanonicalSeverity::Error);
//! ```

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Canonical severity levels for unified severity representation.
///
/// These levels provide a standardized way to represent severity across
/// different linters and analysis tools.
///
/// # Ordering
///
/// Severity levels are ordered from least to most severe:
/// `Unknown < Hint < Info < Warning < Error`
///
/// # Examples
///
/// ```
/// use lintdiff_severity_map::CanonicalSeverity;
///
/// let error = CanonicalSeverity::Error;
/// let warning = CanonicalSeverity::Warning;
///
/// assert!(warning < error);
/// assert!(error >= warning);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum CanonicalSeverity {
    /// Unknown or unrecognized severity level.
    #[default]
    Unknown = 0,
    /// Hint or suggestion for improvement.
    Hint = 1,
    /// Informational message.
    Info = 2,
    /// Warning - should be reviewed and potentially fixed.
    Warning = 3,
    /// Error - must be fixed.
    Error = 4,
}

impl CanonicalSeverity {
    /// Parse a canonical severity from a string (case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns a `SeverityParseError` if the input string is not recognized
    /// as a valid severity level.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::CanonicalSeverity;
    ///
    /// assert_eq!(CanonicalSeverity::parse("error"), Ok(CanonicalSeverity::Error));
    /// assert_eq!(CanonicalSeverity::parse("WARNING"), Ok(CanonicalSeverity::Warning));
    /// assert_eq!(CanonicalSeverity::parse("info"), Ok(CanonicalSeverity::Info));
    /// assert_eq!(CanonicalSeverity::parse("hint"), Ok(CanonicalSeverity::Hint));
    /// assert!(CanonicalSeverity::parse("unknown-value").is_err());
    /// ```
    pub fn parse(s: &str) -> Result<Self, SeverityParseError> {
        match s.to_lowercase().as_str() {
            "error" | "err" | "fatal" | "critical" | "fail" | "2" => Ok(Self::Error),
            "warning" | "warn" | "1" => Ok(Self::Warning),
            "info" | "information" | "note" | "convention" | "refactor" => Ok(Self::Info),
            "hint" | "suggestion" | "help" | "style" => Ok(Self::Hint),
            "unknown" | "off" | "0" => Ok(Self::Unknown),
            _ => Err(SeverityParseError(s.to_string())),
        }
    }

    /// Get a lowercase string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::CanonicalSeverity;
    ///
    /// assert_eq!(CanonicalSeverity::Error.as_str(), "error");
    /// assert_eq!(CanonicalSeverity::Warning.as_str(), "warning");
    /// assert_eq!(CanonicalSeverity::Info.as_str(), "info");
    /// assert_eq!(CanonicalSeverity::Hint.as_str(), "hint");
    /// assert_eq!(CanonicalSeverity::Unknown.as_str(), "unknown");
    /// ```
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Hint => "hint",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    /// Get the numeric level (0-4).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::CanonicalSeverity;
    ///
    /// assert_eq!(CanonicalSeverity::Unknown.level(), 0);
    /// assert_eq!(CanonicalSeverity::Hint.level(), 1);
    /// assert_eq!(CanonicalSeverity::Info.level(), 2);
    /// assert_eq!(CanonicalSeverity::Warning.level(), 3);
    /// assert_eq!(CanonicalSeverity::Error.level(), 4);
    /// ```
    #[must_use]
    pub const fn level(self) -> u8 {
        self as u8
    }

    /// Check if this severity is at least as severe as another.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::CanonicalSeverity;
    ///
    /// let error = CanonicalSeverity::Error;
    /// assert!(error.at_least(CanonicalSeverity::Warning));
    /// assert!(error.at_least(CanonicalSeverity::Error));
    ///
    /// let hint = CanonicalSeverity::Hint;
    /// assert!(hint.at_least(CanonicalSeverity::Unknown));
    /// assert!(!hint.at_least(CanonicalSeverity::Warning));
    /// ```
    #[must_use]
    pub const fn at_least(self, other: Self) -> bool {
        self as u8 >= other as u8
    }

    /// Check if this severity is a problem level (warning or error).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::CanonicalSeverity;
    ///
    /// assert!(!CanonicalSeverity::Hint.is_problem());
    /// assert!(!CanonicalSeverity::Info.is_problem());
    /// assert!(CanonicalSeverity::Warning.is_problem());
    /// assert!(CanonicalSeverity::Error.is_problem());
    /// ```
    #[must_use]
    pub const fn is_problem(self) -> bool {
        matches!(self, Self::Warning | Self::Error)
    }

    /// Check if this severity is blocking (error level).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::CanonicalSeverity;
    ///
    /// assert!(!CanonicalSeverity::Warning.is_blocking());
    /// assert!(CanonicalSeverity::Error.is_blocking());
    /// ```
    #[must_use]
    pub const fn is_blocking(self) -> bool {
        matches!(self, Self::Error)
    }
}

impl std::fmt::Display for CanonicalSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error returned when parsing a severity string fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeverityParseError(pub String);

impl std::fmt::Display for SeverityParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown severity: {}", self.0)
    }
}

impl std::error::Error for SeverityParseError {}

/// Mapper for converting linter-specific severities to canonical severity levels.
///
/// This struct maintains mappings from (linter, severity) pairs to
/// [`CanonicalSeverity`] values, with built-in defaults for common linters.
///
/// # Examples
///
/// ```
/// use lintdiff_severity_map::{CanonicalSeverity, SeverityMapper};
///
/// // Create with default mappings
/// let mapper = SeverityMapper::from_defaults();
/// assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
/// assert_eq!(mapper.map("rustc", "warning"), CanonicalSeverity::Warning);
///
/// // Create empty and add custom mappings
/// let mut custom = SeverityMapper::new();
/// custom.add_mapping("mytool", "bad", CanonicalSeverity::Error);
/// assert_eq!(custom.map("mytool", "bad"), CanonicalSeverity::Error);
/// ```
#[derive(Debug, Clone, Default)]
pub struct SeverityMapper {
    /// Maps (linter, severity) -> canonical severity
    mappings: HashMap<(String, String), CanonicalSeverity>,
}

impl SeverityMapper {
    /// Create a new empty mapper.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::SeverityMapper;
    ///
    /// let mapper = SeverityMapper::new();
    /// // No mappings, will return Unknown for any input
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Create a mapper with built-in default linter mappings.
    ///
    /// This includes mappings for:
    /// - **eslint**: error→Error, warn→Warning, info→Info, off→Unknown, 2→Error, 1→Warning, 0→Unknown
    /// - **rustc**: error→Error, warning→Warning, note→Info, help→Hint
    /// - **pylint**: fatal→Error, error→Error, warning→Warning, convention→Info, refactor→Info, info→Info
    /// - **golint**: error→Error, warning→Warning, info→Info
    /// - **shellcheck**: error→Error, warning→Warning, info→Info, style→Hint
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::{CanonicalSeverity, SeverityMapper};
    ///
    /// let mapper = SeverityMapper::from_defaults();
    /// assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
    /// assert_eq!(mapper.map("rustc", "warning"), CanonicalSeverity::Warning);
    /// assert_eq!(mapper.map("pylint", "fatal"), CanonicalSeverity::Error);
    /// ```
    #[must_use]
    pub fn from_defaults() -> Self {
        let mut mapper = Self::new();

        // ESLint mappings
        mapper.add_mapping("eslint", "error", CanonicalSeverity::Error);
        mapper.add_mapping("eslint", "warn", CanonicalSeverity::Warning);
        mapper.add_mapping("eslint", "warning", CanonicalSeverity::Warning);
        mapper.add_mapping("eslint", "info", CanonicalSeverity::Info);
        mapper.add_mapping("eslint", "off", CanonicalSeverity::Unknown);
        mapper.add_mapping("eslint", "2", CanonicalSeverity::Error);
        mapper.add_mapping("eslint", "1", CanonicalSeverity::Warning);
        mapper.add_mapping("eslint", "0", CanonicalSeverity::Unknown);

        // Rustc mappings
        mapper.add_mapping("rustc", "error", CanonicalSeverity::Error);
        mapper.add_mapping("rustc", "warning", CanonicalSeverity::Warning);
        mapper.add_mapping("rustc", "note", CanonicalSeverity::Info);
        mapper.add_mapping("rustc", "help", CanonicalSeverity::Hint);

        // Clippy (uses rustc levels)
        mapper.add_mapping("clippy", "error", CanonicalSeverity::Error);
        mapper.add_mapping("clippy", "warning", CanonicalSeverity::Warning);
        mapper.add_mapping("clippy", "note", CanonicalSeverity::Info);
        mapper.add_mapping("clippy", "help", CanonicalSeverity::Hint);

        // Pylint mappings
        mapper.add_mapping("pylint", "fatal", CanonicalSeverity::Error);
        mapper.add_mapping("pylint", "error", CanonicalSeverity::Error);
        mapper.add_mapping("pylint", "warning", CanonicalSeverity::Warning);
        mapper.add_mapping("pylint", "convention", CanonicalSeverity::Info);
        mapper.add_mapping("pylint", "refactor", CanonicalSeverity::Info);
        mapper.add_mapping("pylint", "info", CanonicalSeverity::Info);

        // Golint mappings
        mapper.add_mapping("golint", "error", CanonicalSeverity::Error);
        mapper.add_mapping("golint", "warning", CanonicalSeverity::Warning);
        mapper.add_mapping("golint", "warn", CanonicalSeverity::Warning);
        mapper.add_mapping("golint", "info", CanonicalSeverity::Info);

        // ShellCheck mappings
        mapper.add_mapping("shellcheck", "error", CanonicalSeverity::Error);
        mapper.add_mapping("shellcheck", "warning", CanonicalSeverity::Warning);
        mapper.add_mapping("shellcheck", "info", CanonicalSeverity::Info);
        mapper.add_mapping("shellcheck", "style", CanonicalSeverity::Hint);

        mapper
    }

    /// Add a custom mapping for a linter.
    ///
    /// # Arguments
    ///
    /// * `linter` - The linter name (case-insensitive for lookup)
    /// * `from` - The linter-specific severity string (case-insensitive for lookup)
    /// * `to` - The canonical severity to map to
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::{CanonicalSeverity, SeverityMapper};
    ///
    /// let mut mapper = SeverityMapper::new();
    /// mapper.add_mapping("my-linter", "critical", CanonicalSeverity::Error);
    /// mapper.add_mapping("my-linter", "minor", CanonicalSeverity::Warning);
    ///
    /// assert_eq!(mapper.map("my-linter", "critical"), CanonicalSeverity::Error);
    /// assert_eq!(mapper.map("my-linter", "minor"), CanonicalSeverity::Warning);
    /// ```
    pub fn add_mapping(&mut self, linter: &str, from: &str, to: CanonicalSeverity) {
        let key = (linter.to_lowercase(), from.to_lowercase());
        self.mappings.insert(key, to);
    }

    /// Map a linter-specific severity to a canonical severity.
    ///
    /// The lookup is case-insensitive for both linter name and severity string.
    /// Returns [`CanonicalSeverity::Unknown`] if no mapping is found.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::{CanonicalSeverity, SeverityMapper};
    ///
    /// let mapper = SeverityMapper::from_defaults();
    ///
    /// // Known mappings
    /// assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
    /// assert_eq!(mapper.map("ESLINT", "ERROR"), CanonicalSeverity::Error); // case-insensitive
    ///
    /// // Unknown linter/severity
    /// assert_eq!(mapper.map("unknown-linter", "error"), CanonicalSeverity::Unknown);
    /// ```
    #[must_use]
    pub fn map(&self, linter: &str, severity: &str) -> CanonicalSeverity {
        let key = (linter.to_lowercase(), severity.to_lowercase());
        self.mappings.get(&key).copied().unwrap_or_default()
    }

    /// Check if a mapping exists for the given linter and severity.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::SeverityMapper;
    ///
    /// let mapper = SeverityMapper::from_defaults();
    /// assert!(mapper.has_mapping("eslint", "error"));
    /// assert!(!mapper.has_mapping("unknown", "error"));
    /// ```
    #[must_use]
    pub fn has_mapping(&self, linter: &str, severity: &str) -> bool {
        let key = (linter.to_lowercase(), severity.to_lowercase());
        self.mappings.contains_key(&key)
    }

    /// Get the number of mappings stored.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::SeverityMapper;
    ///
    /// let empty = SeverityMapper::new();
    /// assert_eq!(empty.mapping_count(), 0);
    ///
    /// let defaults = SeverityMapper::from_defaults();
    /// assert!(defaults.mapping_count() > 0);
    /// ```
    #[must_use]
    pub fn mapping_count(&self) -> usize {
        self.mappings.len()
    }

    /// Check if the mapper has no mappings.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::SeverityMapper;
    ///
    /// let empty = SeverityMapper::new();
    /// assert!(empty.is_empty());
    ///
    /// let defaults = SeverityMapper::from_defaults();
    /// assert!(!defaults.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Remove all mappings for a specific linter.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::{CanonicalSeverity, SeverityMapper};
    ///
    /// let mut mapper = SeverityMapper::from_defaults();
    /// mapper.remove_linter("eslint");
    /// assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Unknown);
    /// ```
    pub fn remove_linter(&mut self, linter: &str) {
        let linter_lower = linter.to_lowercase();
        self.mappings.retain(|(l, _), _| l != &linter_lower);
    }

    /// Merge mappings from another mapper into this one.
    ///
    /// Existing mappings are overwritten by the other mapper's values.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::{CanonicalSeverity, SeverityMapper};
    ///
    /// let mut mapper1 = SeverityMapper::new();
    /// mapper1.add_mapping("linter", "error", CanonicalSeverity::Error);
    ///
    /// let mut mapper2 = SeverityMapper::new();
    /// mapper2.add_mapping("linter", "warning", CanonicalSeverity::Warning);
    ///
    /// mapper1.merge(mapper2);
    /// assert_eq!(mapper1.map("linter", "error"), CanonicalSeverity::Error);
    /// assert_eq!(mapper1.map("linter", "warning"), CanonicalSeverity::Warning);
    /// ```
    pub fn merge(&mut self, other: Self) {
        for ((linter, severity), canonical) in other.mappings {
            self.mappings.insert((linter, severity), canonical);
        }
    }
}

/// Builder for creating custom severity mappers.
///
/// # Examples
///
/// ```
/// use lintdiff_severity_map::{CanonicalSeverity, SeverityMapBuilder};
///
/// let mapper = SeverityMapBuilder::new()
///     .with_linter("my-linter", [
///         ("critical", CanonicalSeverity::Error),
///         ("warning", CanonicalSeverity::Warning),
///         ("info", CanonicalSeverity::Info),
///     ])
///     .with_linter("another-linter", [
///         ("bad", CanonicalSeverity::Error),
///     ])
///     .build();
///
/// assert_eq!(mapper.map("my-linter", "critical"), CanonicalSeverity::Error);
/// assert_eq!(mapper.map("another-linter", "bad"), CanonicalSeverity::Error);
/// ```
#[derive(Debug, Default)]
pub struct SeverityMapBuilder {
    mapper: SeverityMapper,
}

impl SeverityMapBuilder {
    /// Create a new builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::SeverityMapBuilder;
    ///
    /// let builder = SeverityMapBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder pre-populated with default mappings.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::{CanonicalSeverity, SeverityMapBuilder};
    ///
    /// let mapper = SeverityMapBuilder::with_defaults()
    ///     .with_linter("custom", [("error", CanonicalSeverity::Error)])
    ///     .build();
    ///
    /// // Has default mappings
    /// assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
    /// // And custom mappings
    /// assert_eq!(mapper.map("custom", "error"), CanonicalSeverity::Error);
    /// ```
    #[must_use]
    pub fn with_defaults() -> Self {
        Self {
            mapper: SeverityMapper::from_defaults(),
        }
    }

    /// Add mappings for a specific linter.
    ///
    /// # Arguments
    ///
    /// * `name` - The linter name
    /// * `mappings` - An iterable of (severity-string, canonical-severity) pairs
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::{CanonicalSeverity, SeverityMapBuilder};
    ///
    /// let mapper = SeverityMapBuilder::new()
    ///     .with_linter("my-linter", [
    ///         ("critical", CanonicalSeverity::Error),
    ///         ("warning", CanonicalSeverity::Warning),
    ///     ])
    ///     .build();
    ///
    /// assert_eq!(mapper.map("my-linter", "critical"), CanonicalSeverity::Error);
    /// ```
    #[must_use]
    pub fn with_linter<I, S>(mut self, name: &str, mappings: I) -> Self
    where
        I: IntoIterator<Item = (S, CanonicalSeverity)>,
        S: AsRef<str>,
    {
        for (severity, canonical) in mappings {
            self.mapper.add_mapping(name, severity.as_ref(), canonical);
        }
        self
    }

    /// Add a single mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::{CanonicalSeverity, SeverityMapBuilder};
    ///
    /// let mapper = SeverityMapBuilder::new()
    ///     .with_mapping("linter", "error", CanonicalSeverity::Error)
    ///     .build();
    ///
    /// assert_eq!(mapper.map("linter", "error"), CanonicalSeverity::Error);
    /// ```
    #[must_use]
    pub fn with_mapping(
        mut self,
        linter: &str,
        severity: &str,
        canonical: CanonicalSeverity,
    ) -> Self {
        self.mapper.add_mapping(linter, severity, canonical);
        self
    }

    /// Build the final mapper.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_severity_map::{CanonicalSeverity, SeverityMapBuilder};
    ///
    /// let mapper = SeverityMapBuilder::new()
    ///     .with_mapping("linter", "error", CanonicalSeverity::Error)
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> SeverityMapper {
        self.mapper
    }
}

/// Map a severity using the default mapper.
///
/// This is a convenience function that creates a default mapper and
/// performs the mapping. For repeated mappings, create a [`SeverityMapper`]
/// instance instead.
///
/// # Examples
///
/// ```
/// use lintdiff_severity_map::{map_severity, CanonicalSeverity};
///
/// assert_eq!(map_severity("eslint", "error"), CanonicalSeverity::Error);
/// assert_eq!(map_severity("rustc", "warning"), CanonicalSeverity::Warning);
/// assert_eq!(map_severity("unknown", "error"), CanonicalSeverity::Unknown);
/// ```
#[must_use]
pub fn map_severity(linter: &str, severity: &str) -> CanonicalSeverity {
    static DEFAULT_MAPPER: std::sync::OnceLock<SeverityMapper> = std::sync::OnceLock::new();
    DEFAULT_MAPPER
        .get_or_init(SeverityMapper::from_defaults)
        .map(linter, severity)
}

/// Check if a severity is at error level or above.
///
/// Returns `true` for [`CanonicalSeverity::Error`], `false` otherwise.
///
/// # Examples
///
/// ```
/// use lintdiff_severity_map::{is_error_level, CanonicalSeverity};
///
/// assert!(is_error_level(&CanonicalSeverity::Error));
/// assert!(!is_error_level(&CanonicalSeverity::Warning));
/// assert!(!is_error_level(&CanonicalSeverity::Info));
/// assert!(!is_error_level(&CanonicalSeverity::Hint));
/// assert!(!is_error_level(&CanonicalSeverity::Unknown));
/// ```
#[must_use]
pub const fn is_error_level(severity: &CanonicalSeverity) -> bool {
    matches!(severity, CanonicalSeverity::Error)
}

/// Check if a severity is at warning level or above.
///
/// Returns `true` for [`CanonicalSeverity::Warning`] or [`CanonicalSeverity::Error`].
///
/// # Examples
///
/// ```
/// use lintdiff_severity_map::{is_warning_level, CanonicalSeverity};
///
/// assert!(is_warning_level(&CanonicalSeverity::Error));
/// assert!(is_warning_level(&CanonicalSeverity::Warning));
/// assert!(!is_warning_level(&CanonicalSeverity::Info));
/// assert!(!is_warning_level(&CanonicalSeverity::Hint));
/// ```
#[must_use]
pub const fn is_warning_level(severity: &CanonicalSeverity) -> bool {
    matches!(
        severity,
        CanonicalSeverity::Warning | CanonicalSeverity::Error
    )
}

/// Check if a severity represents a problem (warning or error).
///
/// # Examples
///
/// ```
/// use lintdiff_severity_map::{is_problem_level, CanonicalSeverity};
///
/// assert!(is_problem_level(&CanonicalSeverity::Error));
/// assert!(is_problem_level(&CanonicalSeverity::Warning));
/// assert!(!is_problem_level(&CanonicalSeverity::Info));
/// assert!(!is_problem_level(&CanonicalSeverity::Hint));
/// ```
#[must_use]
pub const fn is_problem_level(severity: &CanonicalSeverity) -> bool {
    severity.is_problem()
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // CanonicalSeverity Tests
    // =========================================================================

    #[test]
    fn canonical_severity_ordering() {
        assert!(CanonicalSeverity::Unknown < CanonicalSeverity::Hint);
        assert!(CanonicalSeverity::Hint < CanonicalSeverity::Info);
        assert!(CanonicalSeverity::Info < CanonicalSeverity::Warning);
        assert!(CanonicalSeverity::Warning < CanonicalSeverity::Error);
    }

    #[test]
    fn canonical_severity_parse_error() {
        assert_eq!(CanonicalSeverity::parse("error"), Ok(CanonicalSeverity::Error));
        assert_eq!(CanonicalSeverity::parse("ERROR"), Ok(CanonicalSeverity::Error));
        assert_eq!(CanonicalSeverity::parse("err"), Ok(CanonicalSeverity::Error));
        assert_eq!(CanonicalSeverity::parse("fatal"), Ok(CanonicalSeverity::Error));
        assert_eq!(CanonicalSeverity::parse("critical"), Ok(CanonicalSeverity::Error));
        assert_eq!(CanonicalSeverity::parse("fail"), Ok(CanonicalSeverity::Error));
        assert_eq!(CanonicalSeverity::parse("2"), Ok(CanonicalSeverity::Error));
    }

    #[test]
    fn canonical_severity_parse_warning() {
        assert_eq!(
            CanonicalSeverity::parse("warning"),
            Ok(CanonicalSeverity::Warning)
        );
        assert_eq!(
            CanonicalSeverity::parse("WARNING"),
            Ok(CanonicalSeverity::Warning)
        );
        assert_eq!(
            CanonicalSeverity::parse("warn"),
            Ok(CanonicalSeverity::Warning)
        );
        assert_eq!(
            CanonicalSeverity::parse("1"),
            Ok(CanonicalSeverity::Warning)
        );
    }

    #[test]
    fn canonical_severity_parse_info() {
        assert_eq!(CanonicalSeverity::parse("info"), Ok(CanonicalSeverity::Info));
        assert_eq!(
            CanonicalSeverity::parse("INFO"),
            Ok(CanonicalSeverity::Info)
        );
        assert_eq!(
            CanonicalSeverity::parse("information"),
            Ok(CanonicalSeverity::Info)
        );
        assert_eq!(CanonicalSeverity::parse("note"), Ok(CanonicalSeverity::Info));
        assert_eq!(
            CanonicalSeverity::parse("convention"),
            Ok(CanonicalSeverity::Info)
        );
        assert_eq!(
            CanonicalSeverity::parse("refactor"),
            Ok(CanonicalSeverity::Info)
        );
    }

    #[test]
    fn canonical_severity_parse_hint() {
        assert_eq!(CanonicalSeverity::parse("hint"), Ok(CanonicalSeverity::Hint));
        assert_eq!(CanonicalSeverity::parse("HINT"), Ok(CanonicalSeverity::Hint));
        assert_eq!(
            CanonicalSeverity::parse("suggestion"),
            Ok(CanonicalSeverity::Hint)
        );
        assert_eq!(CanonicalSeverity::parse("help"), Ok(CanonicalSeverity::Hint));
        assert_eq!(CanonicalSeverity::parse("style"), Ok(CanonicalSeverity::Hint));
    }

    #[test]
    fn canonical_severity_parse_unknown() {
        assert_eq!(
            CanonicalSeverity::parse("unknown"),
            Ok(CanonicalSeverity::Unknown)
        );
        assert_eq!(
            CanonicalSeverity::parse("off"),
            Ok(CanonicalSeverity::Unknown)
        );
        assert_eq!(
            CanonicalSeverity::parse("0"),
            Ok(CanonicalSeverity::Unknown)
        );
    }

    #[test]
    fn canonical_severity_parse_invalid() {
        assert!(CanonicalSeverity::parse("invalid-value").is_err());
        assert!(CanonicalSeverity::parse("").is_err());
        assert!(CanonicalSeverity::parse("xyz").is_err());
    }

    #[test]
    fn canonical_severity_as_str() {
        assert_eq!(CanonicalSeverity::Error.as_str(), "error");
        assert_eq!(CanonicalSeverity::Warning.as_str(), "warning");
        assert_eq!(CanonicalSeverity::Info.as_str(), "info");
        assert_eq!(CanonicalSeverity::Hint.as_str(), "hint");
        assert_eq!(CanonicalSeverity::Unknown.as_str(), "unknown");
    }

    #[test]
    fn canonical_severity_level() {
        assert_eq!(CanonicalSeverity::Unknown.level(), 0);
        assert_eq!(CanonicalSeverity::Hint.level(), 1);
        assert_eq!(CanonicalSeverity::Info.level(), 2);
        assert_eq!(CanonicalSeverity::Warning.level(), 3);
        assert_eq!(CanonicalSeverity::Error.level(), 4);
    }

    #[test]
    fn canonical_severity_at_least() {
        let error = CanonicalSeverity::Error;
        assert!(error.at_least(CanonicalSeverity::Warning));
        assert!(error.at_least(CanonicalSeverity::Info));
        assert!(error.at_least(CanonicalSeverity::Error)); // Error >= Error

        let warning = CanonicalSeverity::Warning;
        assert!(warning.at_least(CanonicalSeverity::Info));
        assert!(warning.at_least(CanonicalSeverity::Warning)); // Warning >= Warning
        assert!(!warning.at_least(CanonicalSeverity::Error));
    }

    #[test]
    fn canonical_severity_is_problem() {
        assert!(CanonicalSeverity::Error.is_problem());
        assert!(CanonicalSeverity::Warning.is_problem());
        assert!(!CanonicalSeverity::Info.is_problem());
        assert!(!CanonicalSeverity::Hint.is_problem());
        assert!(!CanonicalSeverity::Unknown.is_problem());
    }

    #[test]
    fn canonical_severity_is_blocking() {
        assert!(CanonicalSeverity::Error.is_blocking());
        assert!(!CanonicalSeverity::Warning.is_blocking());
        assert!(!CanonicalSeverity::Info.is_blocking());
        assert!(!CanonicalSeverity::Hint.is_blocking());
        assert!(!CanonicalSeverity::Unknown.is_blocking());
    }

    #[test]
    fn canonical_severity_default() {
        assert_eq!(CanonicalSeverity::default(), CanonicalSeverity::Unknown);
    }

    #[test]
    fn canonical_severity_display() {
        assert_eq!(format!("{}", CanonicalSeverity::Error), "error");
        assert_eq!(format!("{}", CanonicalSeverity::Warning), "warning");
        assert_eq!(format!("{}", CanonicalSeverity::Info), "info");
        assert_eq!(format!("{}", CanonicalSeverity::Hint), "hint");
        assert_eq!(format!("{}", CanonicalSeverity::Unknown), "unknown");
    }

    // =========================================================================
    // SeverityMapper Tests
    // =========================================================================

    #[test]
    fn severity_mapper_new_creates_empty() {
        let mapper = SeverityMapper::new();
        assert!(mapper.is_empty());
        assert_eq!(mapper.mapping_count(), 0);
    }

    #[test]
    fn severity_mapper_default_creates_empty() {
        let mapper = SeverityMapper::default();
        assert!(mapper.is_empty());
    }

    #[test]
    fn severity_mapper_add_mapping() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("test", "error", CanonicalSeverity::Error);

        assert_eq!(mapper.map("test", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.mapping_count(), 1);
        assert!(!mapper.is_empty());
    }

    #[test]
    fn severity_mapper_add_mapping_case_insensitive() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("Test", "ERROR", CanonicalSeverity::Error);

        // Lookup should be case-insensitive
        assert_eq!(mapper.map("test", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("TEST", "ERROR"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("TeSt", "ErRoR"), CanonicalSeverity::Error);
    }

    #[test]
    fn severity_mapper_map_unknown_returns_unknown() {
        let mapper = SeverityMapper::new();
        assert_eq!(mapper.map("unknown", "error"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn severity_mapper_has_mapping() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("test", "error", CanonicalSeverity::Error);

        assert!(mapper.has_mapping("test", "error"));
        assert!(!mapper.has_mapping("test", "warning"));
        assert!(!mapper.has_mapping("other", "error"));
    }

    #[test]
    fn severity_mapper_remove_linter() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("linter1", "error", CanonicalSeverity::Error);
        mapper.add_mapping("linter1", "warning", CanonicalSeverity::Warning);
        mapper.add_mapping("linter2", "error", CanonicalSeverity::Error);

        mapper.remove_linter("linter1");

        assert_eq!(mapper.map("linter1", "error"), CanonicalSeverity::Unknown);
        assert_eq!(mapper.map("linter2", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn severity_mapper_merge() {
        let mut mapper1 = SeverityMapper::new();
        mapper1.add_mapping("linter", "error", CanonicalSeverity::Error);

        let mut mapper2 = SeverityMapper::new();
        mapper2.add_mapping("linter", "warning", CanonicalSeverity::Warning);
        mapper2.add_mapping("other", "error", CanonicalSeverity::Error);

        mapper1.merge(mapper2);

        assert_eq!(mapper1.map("linter", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper1.map("linter", "warning"), CanonicalSeverity::Warning);
        assert_eq!(mapper1.map("other", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn severity_mapper_merge_overwrites() {
        let mut mapper1 = SeverityMapper::new();
        mapper1.add_mapping("linter", "error", CanonicalSeverity::Error);

        let mut mapper2 = SeverityMapper::new();
        mapper2.add_mapping("linter", "error", CanonicalSeverity::Warning);

        mapper1.merge(mapper2);

        assert_eq!(
            mapper1.map("linter", "error"),
            CanonicalSeverity::Warning
        );
    }

    // =========================================================================
    // Default Mappings Tests
    // =========================================================================

    #[test]
    fn from_defaults_has_mappings() {
        let mapper = SeverityMapper::from_defaults();
        assert!(!mapper.is_empty());
        assert!(mapper.mapping_count() > 0);
    }

    #[test]
    fn eslint_mappings() {
        let mapper = SeverityMapper::from_defaults();

        assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("eslint", "warn"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("eslint", "warning"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("eslint", "info"), CanonicalSeverity::Info);
        assert_eq!(mapper.map("eslint", "off"), CanonicalSeverity::Unknown);
        assert_eq!(mapper.map("eslint", "2"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("eslint", "1"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("eslint", "0"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn rustc_mappings() {
        let mapper = SeverityMapper::from_defaults();

        assert_eq!(mapper.map("rustc", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("rustc", "warning"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("rustc", "note"), CanonicalSeverity::Info);
        assert_eq!(mapper.map("rustc", "help"), CanonicalSeverity::Hint);
    }

    #[test]
    fn clippy_mappings() {
        let mapper = SeverityMapper::from_defaults();

        assert_eq!(mapper.map("clippy", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("clippy", "warning"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("clippy", "note"), CanonicalSeverity::Info);
        assert_eq!(mapper.map("clippy", "help"), CanonicalSeverity::Hint);
    }

    #[test]
    fn pylint_mappings() {
        let mapper = SeverityMapper::from_defaults();

        assert_eq!(mapper.map("pylint", "fatal"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("pylint", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("pylint", "warning"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("pylint", "convention"), CanonicalSeverity::Info);
        assert_eq!(mapper.map("pylint", "refactor"), CanonicalSeverity::Info);
        assert_eq!(mapper.map("pylint", "info"), CanonicalSeverity::Info);
    }

    #[test]
    fn golint_mappings() {
        let mapper = SeverityMapper::from_defaults();

        assert_eq!(mapper.map("golint", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("golint", "warning"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("golint", "warn"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("golint", "info"), CanonicalSeverity::Info);
    }

    #[test]
    fn shellcheck_mappings() {
        let mapper = SeverityMapper::from_defaults();

        assert_eq!(mapper.map("shellcheck", "error"), CanonicalSeverity::Error);
        assert_eq!(
            mapper.map("shellcheck", "warning"),
            CanonicalSeverity::Warning
        );
        assert_eq!(mapper.map("shellcheck", "info"), CanonicalSeverity::Info);
        assert_eq!(mapper.map("shellcheck", "style"), CanonicalSeverity::Hint);
    }

    #[test]
    fn case_insensitive_linter_lookup() {
        let mapper = SeverityMapper::from_defaults();

        assert_eq!(mapper.map("ESLINT", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("Rustc", "WARNING"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("PYLINT", "Fatal"), CanonicalSeverity::Error);
    }

    // =========================================================================
    // SeverityMapBuilder Tests
    // =========================================================================

    #[test]
    fn builder_new_creates_empty() {
        let mapper = SeverityMapBuilder::new().build();
        assert!(mapper.is_empty());
    }

    #[test]
    fn builder_with_linter() {
        let mapper = SeverityMapBuilder::new()
            .with_linter("custom", [
                ("error", CanonicalSeverity::Error),
                ("warning", CanonicalSeverity::Warning),
            ])
            .build();

        assert_eq!(mapper.map("custom", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("custom", "warning"), CanonicalSeverity::Warning);
    }

    #[test]
    fn builder_with_mapping() {
        let mapper = SeverityMapBuilder::new()
            .with_mapping("linter", "error", CanonicalSeverity::Error)
            .build();

        assert_eq!(mapper.map("linter", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn builder_with_defaults() {
        let mapper = SeverityMapBuilder::with_defaults()
            .with_mapping("custom", "error", CanonicalSeverity::Error)
            .build();

        // Has default mappings
        assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
        // And custom mappings
        assert_eq!(mapper.map("custom", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn builder_chaining() {
        let mapper = SeverityMapBuilder::new()
            .with_linter("linter1", [("error", CanonicalSeverity::Error)])
            .with_linter("linter2", [("error", CanonicalSeverity::Error)])
            .with_mapping("linter3", "error", CanonicalSeverity::Error)
            .build();

        assert_eq!(mapper.map("linter1", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("linter2", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("linter3", "error"), CanonicalSeverity::Error);
    }

    // =========================================================================
    // Convenience Function Tests
    // =========================================================================

    #[test]
    fn map_severity_function() {
        assert_eq!(map_severity("eslint", "error"), CanonicalSeverity::Error);
        assert_eq!(map_severity("rustc", "warning"), CanonicalSeverity::Warning);
        assert_eq!(map_severity("unknown", "error"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn is_error_level_function() {
        assert!(is_error_level(&CanonicalSeverity::Error));
        assert!(!is_error_level(&CanonicalSeverity::Warning));
        assert!(!is_error_level(&CanonicalSeverity::Info));
        assert!(!is_error_level(&CanonicalSeverity::Hint));
        assert!(!is_error_level(&CanonicalSeverity::Unknown));
    }

    #[test]
    fn is_warning_level_function() {
        assert!(is_warning_level(&CanonicalSeverity::Error));
        assert!(is_warning_level(&CanonicalSeverity::Warning));
        assert!(!is_warning_level(&CanonicalSeverity::Info));
        assert!(!is_warning_level(&CanonicalSeverity::Hint));
        assert!(!is_warning_level(&CanonicalSeverity::Unknown));
    }

    #[test]
    fn is_problem_level_function() {
        assert!(is_problem_level(&CanonicalSeverity::Error));
        assert!(is_problem_level(&CanonicalSeverity::Warning));
        assert!(!is_problem_level(&CanonicalSeverity::Info));
        assert!(!is_problem_level(&CanonicalSeverity::Hint));
        assert!(!is_problem_level(&CanonicalSeverity::Unknown));
    }

    // =========================================================================
    // Edge Case Tests
    // =========================================================================

    #[test]
    fn empty_string_severity() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("eslint", ""), CanonicalSeverity::Unknown);
    }

    #[test]
    fn empty_string_linter() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("", "error"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn whitespace_severity() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("eslint", " error "), CanonicalSeverity::Unknown);
    }

    #[test]
    fn numeric_severity_strings() {
        let mapper = SeverityMapper::from_defaults();

        assert_eq!(mapper.map("eslint", "2"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("eslint", "1"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("eslint", "0"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn unicode_severity() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("eslint", "错误"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn very_long_severity_string() {
        let mapper = SeverityMapper::from_defaults();
        let long_severity = "error".repeat(100);
        assert_eq!(mapper.map("eslint", &long_severity), CanonicalSeverity::Unknown);
    }

    // =========================================================================
    // Clone and Debug Tests
    // =========================================================================

    #[test]
    fn severity_mapper_clone() {
        let mapper = SeverityMapper::from_defaults();
        let cloned = mapper.clone();

        assert_eq!(mapper.map("eslint", "error"), cloned.map("eslint", "error"));
    }

    #[test]
    fn canonical_severity_clone() {
        let severity = CanonicalSeverity::Error;
        let cloned = severity.clone();
        assert_eq!(severity, cloned);
    }

    #[test]
    fn canonical_severity_debug() {
        assert_eq!(
            format!("{:?}", CanonicalSeverity::Error),
            "Error"
        );
        assert_eq!(
            format!("{:?}", CanonicalSeverity::Warning),
            "Warning"
        );
    }

    #[test]
    fn severity_mapper_debug() {
        let mapper = SeverityMapper::new();
        let debug = format!("{:?}", mapper);
        assert!(debug.contains("SeverityMapper"));
    }

    // =========================================================================
    // Error Tests
    // =========================================================================

    #[test]
    fn severity_parse_error_display() {
        let error = SeverityParseError("invalid".to_string());
        assert_eq!(format!("{}", error), "Unknown severity: invalid");
    }

    #[test]
    fn severity_parse_error_equality() {
        let error1 = SeverityParseError("invalid".to_string());
        let error2 = SeverityParseError("invalid".to_string());
        let error3 = SeverityParseError("other".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
}
