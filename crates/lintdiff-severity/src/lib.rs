//! Severity levels for lint diagnostics.
//!
//! Provides a standardized severity classification system with ordering,
//! parsing, and comparison capabilities.
//!
//! # Overview
//!
//! This crate provides a [`Severity`] enum representing diagnostic severity levels
//! and a [`SeverityThreshold`] struct for filtering diagnostics by minimum severity.
//!
//! # Ordering
//!
//! Severity levels are ordered from least to most severe:
//! `Hint < Note < Warning < Error < Fatal`
//!
//! # Examples
//!
//! ```
//! use lintdiff_severity::{Severity, SeverityThreshold};
//!
//! let warning = Severity::Warning;
//! let error = Severity::Error;
//!
//! assert!(warning < error);
//! assert!(error >= warning);
//!
//! let threshold = SeverityThreshold::minimum(Severity::Warning);
//! assert!(threshold.allows(Severity::Error));
//! assert!(!threshold.allows(Severity::Hint));
//! ```

use std::fmt;

/// Severity levels for diagnostics, ordered from least to most severe.
///
/// # Ordering
/// `Hint < Note < Warning < Error < Fatal`
///
/// # Examples
/// ```
/// use lintdiff_severity::Severity;
///
/// let warning = Severity::Warning;
/// let error = Severity::Error;
///
/// assert!(warning < error);
/// assert!(error >= warning);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Severity {
    /// Informational hint, not a problem.
    Hint = 0,
    /// Additional information, not a problem.
    Note = 1,
    /// A warning, potentially problematic but not breaking.
    Warning = 2,
    /// An error, definitely a problem.
    Error = 3,
    /// A fatal error, processing cannot continue.
    Fatal = 4,
}

impl Severity {
    /// Parse a severity from a string (case-insensitive).
    ///
    /// # Supported values
    /// - `"hint"`, `"info"`, `"information"` → `Severity::Hint`
    /// - `"note"`, `"suggestion"` → `Severity::Note`
    /// - `"warning"`, `"warn"` → `Severity::Warning`
    /// - `"error"`, `"err"` → `Severity::Error`
    /// - `"fatal"`, `"critical"`, `"fail"` → `Severity::Fatal`
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::Severity;
    ///
    /// assert_eq!(Severity::parse("warning"), Ok(Severity::Warning));
    /// assert_eq!(Severity::parse("ERROR"), Ok(Severity::Error));
    /// assert!(Severity::parse("unknown").is_err());
    /// ```
    pub fn parse(s: &str) -> Result<Self, SeverityParseError> {
        match s.to_lowercase().as_str() {
            "hint" | "info" | "information" => Ok(Severity::Hint),
            "note" | "suggestion" => Ok(Severity::Note),
            "warning" | "warn" => Ok(Severity::Warning),
            "error" | "err" => Ok(Severity::Error),
            "fatal" | "critical" | "fail" => Ok(Severity::Fatal),
            _ => Err(SeverityParseError(s.to_string())),
        }
    }

    /// Check if this severity is at least as severe as another.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::Severity;
    ///
    /// let error = Severity::Error;
    /// assert!(error.at_least(Severity::Warning));
    /// assert!(!error.at_least(Severity::Fatal));
    /// ```
    pub fn at_least(self, other: Severity) -> bool {
        self >= other
    }

    /// Check if this severity is at most as severe as another.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::Severity;
    ///
    /// let hint = Severity::Hint;
    /// let warning = Severity::Warning;
    /// assert!(hint.at_most(Severity::Note));
    /// assert!(hint.at_most(Severity::Hint));
    /// assert!(hint.at_most(Severity::Warning));
    /// assert!(!warning.at_most(Severity::Hint));
    /// ```
    pub fn at_most(self, other: Severity) -> bool {
        self <= other
    }

    /// Check if this is a problem severity (warning or higher).
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::Severity;
    ///
    /// assert!(!Severity::Hint.is_problem());
    /// assert!(!Severity::Note.is_problem());
    /// assert!(Severity::Warning.is_problem());
    /// assert!(Severity::Error.is_problem());
    /// assert!(Severity::Fatal.is_problem());
    /// ```
    pub fn is_problem(self) -> bool {
        self >= Severity::Warning
    }

    /// Check if this is a blocking severity (error or higher).
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::Severity;
    ///
    /// assert!(!Severity::Warning.is_blocking());
    /// assert!(Severity::Error.is_blocking());
    /// assert!(Severity::Fatal.is_blocking());
    /// ```
    pub fn is_blocking(self) -> bool {
        self >= Severity::Error
    }

    /// Get the numeric level (0-4).
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::Severity;
    ///
    /// assert_eq!(Severity::Hint.level(), 0);
    /// assert_eq!(Severity::Note.level(), 1);
    /// assert_eq!(Severity::Warning.level(), 2);
    /// assert_eq!(Severity::Error.level(), 3);
    /// assert_eq!(Severity::Fatal.level(), 4);
    /// ```
    pub fn level(self) -> u8 {
        self as u8
    }

    /// Get a lowercase string representation.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::Severity;
    ///
    /// assert_eq!(Severity::Warning.as_str(), "warning");
    /// assert_eq!(Severity::Error.as_str(), "error");
    /// ```
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Hint => "hint",
            Severity::Note => "note",
            Severity::Warning => "warning",
            Severity::Error => "error",
            Severity::Fatal => "fatal",
        }
    }

    /// Get an icon for display (Unicode symbols).
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::Severity;
    ///
    /// assert_eq!(Severity::Hint.icon(), "💡");
    /// assert_eq!(Severity::Warning.icon(), "⚠️");
    /// assert_eq!(Severity::Error.icon(), "❌");
    /// ```
    pub fn icon(self) -> &'static str {
        match self {
            Severity::Hint => "💡",
            Severity::Note => "📝",
            Severity::Warning => "⚠️",
            Severity::Error => "❌",
            Severity::Fatal => "🔥",
        }
    }

    /// Get the ANSI color code for this severity.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::Severity;
    ///
    /// #[cfg(feature = "colors")]
    /// {
    ///     assert_eq!(Severity::Error.ansi_color(), "\x1b[31m");
    /// }
    /// ```
    #[cfg(feature = "colors")]
    pub fn ansi_color(self) -> &'static str {
        match self {
            Severity::Hint => "\x1b[36m",    // Cyan
            Severity::Note => "\x1b[34m",    // Blue
            Severity::Warning => "\x1b[33m", // Yellow
            Severity::Error => "\x1b[31m",   // Red
            Severity::Fatal => "\x1b[35m",   // Magenta
        }
    }
}

/// Error returned when parsing a severity fails.
///
/// # Examples
/// ```
/// use lintdiff_severity::Severity;
///
/// let result = Severity::parse("unknown");
/// assert!(result.is_err());
/// let err = result.unwrap_err();
/// assert!(err.to_string().contains("unknown"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Unknown severity level: '{0}'")]
pub struct SeverityParseError(String);

impl SeverityParseError {
    /// Get the invalid input that caused this error.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::Severity;
    ///
    /// let err = Severity::parse("invalid").unwrap_err();
    /// assert_eq!(err.input(), "invalid");
    /// ```
    pub fn input(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Severity {
    type Err = SeverityParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Severity::parse(s)
    }
}

/// A threshold for filtering diagnostics by severity.
///
/// # Examples
/// ```
/// use lintdiff_severity::{Severity, SeverityThreshold};
///
/// let threshold = SeverityThreshold::minimum(Severity::Warning);
/// assert!(threshold.allows(Severity::Warning));
/// assert!(threshold.allows(Severity::Error));
/// assert!(!threshold.allows(Severity::Hint));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeverityThreshold {
    minimum: Severity,
}

impl SeverityThreshold {
    /// Create a new threshold with the given minimum severity.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::{Severity, SeverityThreshold};
    ///
    /// let threshold = SeverityThreshold::minimum(Severity::Error);
    /// assert!(threshold.allows(Severity::Error));
    /// assert!(threshold.allows(Severity::Fatal));
    /// assert!(!threshold.allows(Severity::Warning));
    /// ```
    pub fn minimum(min: Severity) -> Self {
        Self { minimum: min }
    }

    /// Check if a severity passes the threshold.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::{Severity, SeverityThreshold};
    ///
    /// let threshold = SeverityThreshold::minimum(Severity::Note);
    /// assert!(threshold.allows(Severity::Note));
    /// assert!(threshold.allows(Severity::Warning));
    /// assert!(!threshold.allows(Severity::Hint));
    /// ```
    pub fn allows(self, severity: Severity) -> bool {
        severity >= self.minimum
    }

    /// Get the minimum severity.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_severity::{Severity, SeverityThreshold};
    ///
    /// let threshold = SeverityThreshold::minimum(Severity::Warning);
    /// assert_eq!(threshold.min_severity(), Severity::Warning);
    /// ```
    pub fn min_severity(self) -> Severity {
        self.minimum
    }
}

impl Default for SeverityThreshold {
    fn default() -> Self {
        Self::minimum(Severity::Hint) // Allow all by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Hint < Severity::Note);
        assert!(Severity::Note < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Fatal);
    }

    #[test]
    fn test_severity_from_str() {
        assert_eq!(Severity::parse("hint").unwrap(), Severity::Hint);
        assert_eq!(Severity::parse("WARNING").unwrap(), Severity::Warning);
        assert!(Severity::parse("unknown").is_err());
    }

    #[test]
    fn test_threshold_default() {
        let threshold = SeverityThreshold::default();
        assert!(threshold.allows(Severity::Hint));
        assert!(threshold.allows(Severity::Fatal));
    }
}
