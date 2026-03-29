//! Diagnostic level parsing and conversion utilities for lintdiff.
//!
//! This microcrate provides a single responsibility: parsing and converting
//! diagnostic severity levels from various linters to a canonical representation.
//!
//! # Overview
//!
//! - [`DiagnosticLevel`]: Enum representing diagnostic severity levels
//! - [`DiagnosticLevelParser`]: Configurable parser with custom mappings
//! - Parsing functions: [`parse_level`], [`parse_level_rustc`], [`parse_level_eslint`], [`from_number`]
//! - Conversion functions: [`to_canonical`], [`is_problem`], [`is_error`], [`is_warning`]
//!
//! # Diagnostic Levels
//!
//! | Level | Meaning | Numeric |
//! |-------|---------|---------|
//! | `Fatal` | Compiler/linter fatal error | 4 |
//! | `Error` | Error-level diagnostic | 3 |
//! | `Warning` | Warning-level diagnostic | 2 |
//! | `Note` | Note/informational diagnostic | 1 |
//! | `Hint` | Hint/suggestion diagnostic | 0 |
//! | `Unknown` | Unrecognized level | 255 |
//!
//! # Example
//!
//! ```
//! use lintdiff_diagnostic_level::{DiagnosticLevel, parse_level, is_error};
//!
//! // Parse from various formats
//! let level = parse_level("error");
//! assert_eq!(level, DiagnosticLevel::Error);
//!
//! // Check if it's an error level
//! assert!(is_error(&level));
//!
//! // Parse rustc-style levels
//! let warning = lintdiff_diagnostic_level::parse_level_rustc("warning");
//! assert_eq!(warning, DiagnosticLevel::Warning);
//! ```

#![warn(missing_docs)]

use std::collections::HashMap;
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Diagnostic severity levels.
///
/// Represents the severity of a diagnostic message from a linter or compiler.
/// Levels are ordered from least to most severe: `Hint < Note < Warning < Error < Fatal`.
///
/// # Examples
///
/// ```
/// use lintdiff_diagnostic_level::DiagnosticLevel;
///
/// let error = DiagnosticLevel::Error;
/// let warning = DiagnosticLevel::Warning;
///
/// assert!(warning < error);
/// assert!(error >= warning);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DiagnosticLevel {
    /// Hint or suggestion for improvement.
    Hint = 0,
    /// Note or informational message.
    Note = 1,
    /// Warning-level diagnostic.
    Warning = 2,
    /// Error-level diagnostic.
    Error = 3,
    /// Fatal error - compiler/linter cannot continue.
    Fatal = 4,
    /// Unrecognized or unknown level.
    #[default]
    Unknown = 255,
}

impl DiagnosticLevel {
    /// Get the numeric level value.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::DiagnosticLevel;
    ///
    /// assert_eq!(DiagnosticLevel::Hint.level(), 0);
    /// assert_eq!(DiagnosticLevel::Note.level(), 1);
    /// assert_eq!(DiagnosticLevel::Warning.level(), 2);
    /// assert_eq!(DiagnosticLevel::Error.level(), 3);
    /// assert_eq!(DiagnosticLevel::Fatal.level(), 4);
    /// assert_eq!(DiagnosticLevel::Unknown.level(), 255);
    /// ```
    #[must_use]
    pub const fn level(self) -> u8 {
        self as u8
    }

    /// Get a lowercase string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::DiagnosticLevel;
    ///
    /// assert_eq!(DiagnosticLevel::Error.as_str(), "error");
    /// assert_eq!(DiagnosticLevel::Warning.as_str(), "warning");
    /// assert_eq!(DiagnosticLevel::Unknown.as_str(), "unknown");
    /// ```
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hint => "hint",
            Self::Note => "note",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Fatal => "fatal",
            Self::Unknown => "unknown",
        }
    }

    /// Get an icon for display (Unicode symbols).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::DiagnosticLevel;
    ///
    /// assert_eq!(DiagnosticLevel::Hint.icon(), "💡");
    /// assert_eq!(DiagnosticLevel::Note.icon(), "📝");
    /// assert_eq!(DiagnosticLevel::Warning.icon(), "⚠️");
    /// assert_eq!(DiagnosticLevel::Error.icon(), "❌");
    /// assert_eq!(DiagnosticLevel::Fatal.icon(), "💀");
    /// assert_eq!(DiagnosticLevel::Unknown.icon(), "❓");
    /// ```
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Hint => "💡",
            Self::Note => "📝",
            Self::Warning => "⚠️",
            Self::Error => "❌",
            Self::Fatal => "💀",
            Self::Unknown => "❓",
        }
    }

    /// Check if this level represents a problem (warning or higher).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::DiagnosticLevel;
    ///
    /// assert!(!DiagnosticLevel::Hint.is_problem());
    /// assert!(!DiagnosticLevel::Note.is_problem());
    /// assert!(DiagnosticLevel::Warning.is_problem());
    /// assert!(DiagnosticLevel::Error.is_problem());
    /// assert!(DiagnosticLevel::Fatal.is_problem());
    /// assert!(!DiagnosticLevel::Unknown.is_problem());
    /// ```
    #[must_use]
    pub const fn is_problem(self) -> bool {
        matches!(self, Self::Warning | Self::Error | Self::Fatal)
    }

    /// Check if this level is an error or fatal.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::DiagnosticLevel;
    ///
    /// assert!(!DiagnosticLevel::Warning.is_error());
    /// assert!(DiagnosticLevel::Error.is_error());
    /// assert!(DiagnosticLevel::Fatal.is_error());
    /// assert!(!DiagnosticLevel::Unknown.is_error());
    /// ```
    #[must_use]
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error | Self::Fatal)
    }

    /// Check if this level is a warning.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::DiagnosticLevel;
    ///
    /// assert!(DiagnosticLevel::Warning.is_warning());
    /// assert!(!DiagnosticLevel::Error.is_warning());
    /// assert!(!DiagnosticLevel::Unknown.is_warning());
    /// ```
    #[must_use]
    pub const fn is_warning(self) -> bool {
        matches!(self, Self::Warning)
    }

    /// Check if this level is informational (hint or note).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::DiagnosticLevel;
    ///
    /// assert!(DiagnosticLevel::Hint.is_info());
    /// assert!(DiagnosticLevel::Note.is_info());
    /// assert!(!DiagnosticLevel::Warning.is_info());
    /// assert!(!DiagnosticLevel::Unknown.is_info());
    /// ```
    #[must_use]
    pub const fn is_info(self) -> bool {
        matches!(self, Self::Hint | Self::Note)
    }

    /// Parse from a string (case-insensitive, generic format).
    ///
    /// # Errors
    ///
    /// Returns a `DiagnosticLevelParseError` if the input string is not recognized
    /// as a valid diagnostic level.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::DiagnosticLevel;
    ///
    /// assert_eq!(DiagnosticLevel::parse("error"), Ok(DiagnosticLevel::Error));
    /// assert_eq!(DiagnosticLevel::parse("WARNING"), Ok(DiagnosticLevel::Warning));
    /// assert!(DiagnosticLevel::parse("invalid").is_err());
    /// ```
    pub fn parse(s: &str) -> Result<Self, DiagnosticLevelParseError> {
        match s.to_lowercase().as_str() {
            "hint" | "suggestion" | "help" | "style" => Ok(Self::Hint),
            "note" | "info" | "information" | "convention" | "refactor" => Ok(Self::Note),
            "warning" | "warn" => Ok(Self::Warning),
            "error" | "err" => Ok(Self::Error),
            "fatal" | "critical" | "fail" => Ok(Self::Fatal),
            "unknown" => Ok(Self::Unknown),
            _ => Err(DiagnosticLevelParseError(s.to_string())),
        }
    }

    /// Create from a numeric level.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::DiagnosticLevel;
    ///
    /// assert_eq!(DiagnosticLevel::from_number(0), DiagnosticLevel::Hint);
    /// assert_eq!(DiagnosticLevel::from_number(1), DiagnosticLevel::Note);
    /// assert_eq!(DiagnosticLevel::from_number(2), DiagnosticLevel::Warning);
    /// assert_eq!(DiagnosticLevel::from_number(3), DiagnosticLevel::Error);
    /// assert_eq!(DiagnosticLevel::from_number(4), DiagnosticLevel::Fatal);
    /// assert_eq!(DiagnosticLevel::from_number(99), DiagnosticLevel::Unknown);
    /// ```
    #[must_use]
    pub const fn from_number(n: u8) -> Self {
        match n {
            0 => Self::Hint,
            1 => Self::Note,
            2 => Self::Warning,
            3 => Self::Error,
            4 => Self::Fatal,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<u8> for DiagnosticLevel {
    fn from(n: u8) -> Self {
        Self::from_number(n)
    }
}

impl From<i32> for DiagnosticLevel {
    fn from(n: i32) -> Self {
        if (0..=255).contains(&n) {
            Self::from_number(u8::try_from(n).unwrap_or(255))
        } else {
            Self::Unknown
        }
    }
}

/// Error type for diagnostic level parsing failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticLevelParseError(String);

impl DiagnosticLevelParseError {
    /// Create a new parse error with the given input.
    #[must_use]
    pub fn new(input: impl Into<String>) -> Self {
        Self(input.into())
    }

    /// Get the invalid input that caused the error.
    #[must_use]
    pub fn input(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DiagnosticLevelParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid diagnostic level: {}", self.0)
    }
}

impl std::error::Error for DiagnosticLevelParseError {}

// =============================================================================
// Parsing functions
// =============================================================================

/// Parse a diagnostic level from a string (case-insensitive).
///
/// This is a convenience function that tries multiple formats.
/// For linter-specific parsing, use [`parse_level_rustc`] or [`parse_level_eslint`].
///
/// # Examples
///
/// ```
/// use lintdiff_diagnostic_level::{parse_level, DiagnosticLevel};
///
/// assert_eq!(parse_level("error"), DiagnosticLevel::Error);
/// assert_eq!(parse_level("WARNING"), DiagnosticLevel::Warning);
/// assert_eq!(parse_level("hint"), DiagnosticLevel::Hint);
/// assert_eq!(parse_level("unknown-value"), DiagnosticLevel::Unknown);
/// ```
#[must_use]
pub fn parse_level(s: &str) -> DiagnosticLevel {
    DiagnosticLevel::parse(s).unwrap_or(DiagnosticLevel::Unknown)
}

/// Parse a rustc-style diagnostic level.
///
/// # Supported values
/// - `"error"` → `Error`
/// - `"warning"` → `Warning`
/// - `"note"` → `Note`
/// - `"help"` → `Hint`
/// - `"fatal-error"`, `"fatal"` → `Fatal`
///
/// # Examples
///
/// ```
/// use lintdiff_diagnostic_level::{parse_level_rustc, DiagnosticLevel};
///
/// assert_eq!(parse_level_rustc("error"), DiagnosticLevel::Error);
/// assert_eq!(parse_level_rustc("warning"), DiagnosticLevel::Warning);
/// assert_eq!(parse_level_rustc("note"), DiagnosticLevel::Note);
/// assert_eq!(parse_level_rustc("help"), DiagnosticLevel::Hint);
/// assert_eq!(parse_level_rustc("fatal-error"), DiagnosticLevel::Fatal);
/// assert_eq!(parse_level_rustc("unknown"), DiagnosticLevel::Unknown);
/// ```
#[must_use]
pub fn parse_level_rustc(s: &str) -> DiagnosticLevel {
    match s.to_lowercase().as_str() {
        "error" => DiagnosticLevel::Error,
        "warning" => DiagnosticLevel::Warning,
        "note" => DiagnosticLevel::Note,
        "help" => DiagnosticLevel::Hint,
        "fatal-error" | "fatal" => DiagnosticLevel::Fatal,
        _ => DiagnosticLevel::Unknown,
    }
}

/// Parse an eslint-style diagnostic level.
///
/// # Supported values
/// - `"error"`, `"2"` → `Error`
/// - `"warning"`, `"warn"`, `"1"` → `Warning`
/// - `"off"`, `"0"` → `Unknown` (disabled)
/// - `"info"` → `Note`
/// - `"hint"` → `Hint`
///
/// # Examples
///
/// ```
/// use lintdiff_diagnostic_level::{parse_level_eslint, DiagnosticLevel};
///
/// assert_eq!(parse_level_eslint("error"), DiagnosticLevel::Error);
/// assert_eq!(parse_level_eslint("2"), DiagnosticLevel::Error);
/// assert_eq!(parse_level_eslint("warning"), DiagnosticLevel::Warning);
/// assert_eq!(parse_level_eslint("1"), DiagnosticLevel::Warning);
/// assert_eq!(parse_level_eslint("off"), DiagnosticLevel::Unknown);
/// assert_eq!(parse_level_eslint("0"), DiagnosticLevel::Unknown);
/// ```
#[must_use]
pub fn parse_level_eslint(s: &str) -> DiagnosticLevel {
    match s.to_lowercase().as_str() {
        "error" | "2" => DiagnosticLevel::Error,
        "warning" | "warn" | "1" => DiagnosticLevel::Warning,
        "info" => DiagnosticLevel::Note,
        "hint" => DiagnosticLevel::Hint,
        // "off", "0", and any unrecognized values all map to Unknown
        _ => DiagnosticLevel::Unknown,
    }
}

/// Parse a diagnostic level from a numeric value.
///
/// # Examples
///
/// ```
/// use lintdiff_diagnostic_level::{from_number, DiagnosticLevel};
///
/// assert_eq!(from_number(0), DiagnosticLevel::Hint);
/// assert_eq!(from_number(2), DiagnosticLevel::Warning);
/// assert_eq!(from_number(3), DiagnosticLevel::Error);
/// assert_eq!(from_number(99), DiagnosticLevel::Unknown);
/// ```
#[must_use]
pub const fn from_number(n: u8) -> DiagnosticLevel {
    DiagnosticLevel::from_number(n)
}

// =============================================================================
// Conversion functions
// =============================================================================

/// Canonical severity for unified representation.
///
/// This is a simplified severity type used for cross-linter compatibility.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum CanonicalSeverity {
    /// Unknown severity.
    #[default]
    Unknown = 0,
    /// Informational (hint/note).
    Info = 1,
    /// Warning level.
    Warning = 2,
    /// Error level (error/fatal).
    Error = 3,
}

impl CanonicalSeverity {
    /// Get a lowercase string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl fmt::Display for CanonicalSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Convert a diagnostic level to canonical severity.
///
/// # Examples
///
/// ```
/// use lintdiff_diagnostic_level::{DiagnosticLevel, to_canonical, CanonicalSeverity};
///
/// assert_eq!(to_canonical(&DiagnosticLevel::Error), CanonicalSeverity::Error);
/// assert_eq!(to_canonical(&DiagnosticLevel::Fatal), CanonicalSeverity::Error);
/// assert_eq!(to_canonical(&DiagnosticLevel::Warning), CanonicalSeverity::Warning);
/// assert_eq!(to_canonical(&DiagnosticLevel::Note), CanonicalSeverity::Info);
/// assert_eq!(to_canonical(&DiagnosticLevel::Hint), CanonicalSeverity::Info);
/// assert_eq!(to_canonical(&DiagnosticLevel::Unknown), CanonicalSeverity::Unknown);
/// ```
#[must_use]
pub const fn to_canonical(level: &DiagnosticLevel) -> CanonicalSeverity {
    match level {
        DiagnosticLevel::Hint | DiagnosticLevel::Note => CanonicalSeverity::Info,
        DiagnosticLevel::Warning => CanonicalSeverity::Warning,
        DiagnosticLevel::Error | DiagnosticLevel::Fatal => CanonicalSeverity::Error,
        DiagnosticLevel::Unknown => CanonicalSeverity::Unknown,
    }
}

/// Check if a diagnostic level represents a problem (warning or higher).
///
/// # Examples
///
/// ```
/// use lintdiff_diagnostic_level::{DiagnosticLevel, is_problem};
///
/// assert!(!is_problem(&DiagnosticLevel::Hint));
/// assert!(!is_problem(&DiagnosticLevel::Note));
/// assert!(is_problem(&DiagnosticLevel::Warning));
/// assert!(is_problem(&DiagnosticLevel::Error));
/// assert!(is_problem(&DiagnosticLevel::Fatal));
/// assert!(!is_problem(&DiagnosticLevel::Unknown));
/// ```
#[must_use]
pub const fn is_problem(level: &DiagnosticLevel) -> bool {
    level.is_problem()
}

/// Check if a diagnostic level is an error or fatal.
///
/// # Examples
///
/// ```
/// use lintdiff_diagnostic_level::{DiagnosticLevel, is_error};
///
/// assert!(!is_error(&DiagnosticLevel::Warning));
/// assert!(is_error(&DiagnosticLevel::Error));
/// assert!(is_error(&DiagnosticLevel::Fatal));
/// assert!(!is_error(&DiagnosticLevel::Unknown));
/// ```
#[must_use]
pub const fn is_error(level: &DiagnosticLevel) -> bool {
    level.is_error()
}

/// Check if a diagnostic level is a warning.
///
/// # Examples
///
/// ```
/// use lintdiff_diagnostic_level::{DiagnosticLevel, is_warning};
///
/// assert!(is_warning(&DiagnosticLevel::Warning));
/// assert!(!is_warning(&DiagnosticLevel::Error));
/// assert!(!is_warning(&DiagnosticLevel::Unknown));
/// ```
#[must_use]
pub const fn is_warning(level: &DiagnosticLevel) -> bool {
    level.is_warning()
}

// =============================================================================
// DiagnosticLevelParser
// =============================================================================

/// A configurable parser for diagnostic levels with custom mappings.
///
/// # Examples
///
/// ```
/// use lintdiff_diagnostic_level::{DiagnosticLevelParser, DiagnosticLevel};
///
/// let mut parser = DiagnosticLevelParser::new();
/// parser.with_custom_mapping("critical", DiagnosticLevel::Fatal);
///
/// assert_eq!(parser.parse("critical"), DiagnosticLevel::Fatal);
/// assert_eq!(parser.parse("error"), DiagnosticLevel::Error);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DiagnosticLevelParser {
    custom_mappings: HashMap<String, DiagnosticLevel>,
}

impl DiagnosticLevelParser {
    /// Create a new parser with default mappings.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::{DiagnosticLevelParser, DiagnosticLevel};
    ///
    /// let parser = DiagnosticLevelParser::new();
    /// assert_eq!(parser.parse("error"), DiagnosticLevel::Error);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            custom_mappings: HashMap::new(),
        }
    }

    /// Add a custom mapping from a string to a diagnostic level.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::{DiagnosticLevelParser, DiagnosticLevel};
    ///
    /// let mut parser = DiagnosticLevelParser::new();
    /// parser.with_custom_mapping("severe", DiagnosticLevel::Error);
    ///
    /// assert_eq!(parser.parse("severe"), DiagnosticLevel::Error);
    /// ```
    pub fn with_custom_mapping(&mut self, from: &str, to: DiagnosticLevel) -> &mut Self {
        self.custom_mappings.insert(from.to_lowercase(), to);
        self
    }

    /// Parse a diagnostic level from a string.
    ///
    /// Custom mappings are checked first, then falls back to default parsing.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::{DiagnosticLevelParser, DiagnosticLevel};
    ///
    /// let mut parser = DiagnosticLevelParser::new();
    /// parser.with_custom_mapping("critical", DiagnosticLevel::Fatal);
    ///
    /// assert_eq!(parser.parse("critical"), DiagnosticLevel::Fatal);
    /// assert_eq!(parser.parse("error"), DiagnosticLevel::Error);
    /// assert_eq!(parser.parse("unknown"), DiagnosticLevel::Unknown);
    /// ```
    #[must_use]
    pub fn parse(&self, s: &str) -> DiagnosticLevel {
        let lower = s.to_lowercase();

        // Check custom mappings first
        if let Some(&level) = self.custom_mappings.get(&lower) {
            return level;
        }

        // Fall back to default parsing
        parse_level(s)
    }

    /// Check if a custom mapping exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::{DiagnosticLevelParser, DiagnosticLevel};
    ///
    /// let mut parser = DiagnosticLevelParser::new();
    /// parser.with_custom_mapping("critical", DiagnosticLevel::Fatal);
    ///
    /// assert!(parser.has_mapping("critical"));
    /// assert!(!parser.has_mapping("error"));
    /// ```
    #[must_use]
    pub fn has_mapping(&self, s: &str) -> bool {
        self.custom_mappings.contains_key(&s.to_lowercase())
    }

    /// Remove a custom mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::{DiagnosticLevelParser, DiagnosticLevel};
    ///
    /// let mut parser = DiagnosticLevelParser::new();
    /// parser.with_custom_mapping("critical", DiagnosticLevel::Fatal);
    ///
    /// assert!(parser.has_mapping("critical"));
    /// parser.remove_mapping("critical");
    /// assert!(!parser.has_mapping("critical"));
    /// ```
    pub fn remove_mapping(&mut self, s: &str) -> &mut Self {
        self.custom_mappings.remove(&s.to_lowercase());
        self
    }

    /// Get the number of custom mappings.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::{DiagnosticLevelParser, DiagnosticLevel};
    ///
    /// let mut parser = DiagnosticLevelParser::new();
    /// assert_eq!(parser.mapping_count(), 0);
    ///
    /// parser.with_custom_mapping("critical", DiagnosticLevel::Fatal);
    /// assert_eq!(parser.mapping_count(), 1);
    /// ```
    #[must_use]
    pub fn mapping_count(&self) -> usize {
        self.custom_mappings.len()
    }

    /// Clear all custom mappings.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diagnostic_level::{DiagnosticLevelParser, DiagnosticLevel};
    ///
    /// let mut parser = DiagnosticLevelParser::new();
    /// parser.with_custom_mapping("critical", DiagnosticLevel::Fatal);
    /// parser.with_custom_mapping("severe", DiagnosticLevel::Error);
    ///
    /// assert_eq!(parser.mapping_count(), 2);
    /// parser.clear_mappings();
    /// assert_eq!(parser.mapping_count(), 0);
    /// ```
    pub fn clear_mappings(&mut self) -> &mut Self {
        self.custom_mappings.clear();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_level_ordering() {
        assert!(DiagnosticLevel::Hint < DiagnosticLevel::Note);
        assert!(DiagnosticLevel::Note < DiagnosticLevel::Warning);
        assert!(DiagnosticLevel::Warning < DiagnosticLevel::Error);
        assert!(DiagnosticLevel::Error < DiagnosticLevel::Fatal);
        assert!(DiagnosticLevel::Unknown > DiagnosticLevel::Fatal);
    }

    #[test]
    fn test_diagnostic_level_display() {
        assert_eq!(format!("{}", DiagnosticLevel::Error), "error");
        assert_eq!(format!("{}", DiagnosticLevel::Warning), "warning");
        assert_eq!(format!("{}", DiagnosticLevel::Unknown), "unknown");
    }

    #[test]
    fn test_from_u8() {
        assert_eq!(DiagnosticLevel::from(0u8), DiagnosticLevel::Hint);
        assert_eq!(DiagnosticLevel::from(3u8), DiagnosticLevel::Error);
        assert_eq!(DiagnosticLevel::from(255u8), DiagnosticLevel::Unknown);
    }

    #[test]
    fn test_from_i32() {
        assert_eq!(DiagnosticLevel::from(0i32), DiagnosticLevel::Hint);
        assert_eq!(DiagnosticLevel::from(3i32), DiagnosticLevel::Error);
        assert_eq!(DiagnosticLevel::from(-1i32), DiagnosticLevel::Unknown);
        assert_eq!(DiagnosticLevel::from(256i32), DiagnosticLevel::Unknown);
    }
}
