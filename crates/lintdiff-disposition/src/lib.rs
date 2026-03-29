//! Diagnostic disposition for lintdiff.
//!
//! Provides types for tracking what happened to a diagnostic
//! (new, fixed, pre-existing, suppressed, etc.).

use std::fmt;

/// The disposition of a diagnostic finding.
///
/// # Examples
/// ```
/// use lintdiff_disposition::Disposition;
///
/// assert!(Disposition::New.is_actionable());
/// assert!(!Disposition::PreExisting.is_actionable());
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Disposition {
    /// New issue introduced in this change.
    #[default]
    New = 0,
    /// Issue that was fixed by this change.
    Fixed = 1,
    /// Pre-existing issue not introduced by this change.
    PreExisting = 2,
    /// Issue suppressed by configuration.
    Suppressed = 3,
    /// Issue outside the diff scope.
    OutsideDiff = 4,
    /// Issue that couldn't be processed.
    Skipped = 5,
}

impl Disposition {
    /// Check if this disposition represents an actionable finding.
    ///
    /// Actionable findings are ones the developer should address:
    /// - `New` - needs attention
    /// - `Fixed` - confirmation of improvement
    #[must_use]
    pub const fn is_actionable(&self) -> bool {
        matches!(self, Self::New | Self::Fixed)
    }

    /// Check if this is a new issue.
    #[must_use]
    pub const fn is_new(&self) -> bool {
        matches!(self, Self::New)
    }

    /// Check if this is a fixed issue.
    #[must_use]
    pub const fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed)
    }

    /// Check if this should be included in reports.
    ///
    /// Excluded: `Suppressed`, `Skipped`
    #[must_use]
    pub const fn is_reportable(&self) -> bool {
        !matches!(self, Self::Suppressed | Self::Skipped)
    }

    /// Check if this counts toward failure threshold.
    #[must_use]
    pub const fn counts_toward_failure(&self) -> bool {
        matches!(self, Self::New)
    }

    /// Get a string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Fixed => "fixed",
            Self::PreExisting => "pre-existing",
            Self::Suppressed => "suppressed",
            Self::OutsideDiff => "outside-diff",
            Self::Skipped => "skipped",
        }
    }

    /// Get an icon for display.
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::New => "🆕",
            Self::Fixed => "✅",
            Self::PreExisting => "⏳",
            Self::Suppressed => "🔇",
            Self::OutsideDiff => "📍",
            Self::Skipped => "⏭️",
        }
    }

    /// Get a human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::New => "New Issue",
            Self::Fixed => "Fixed Issue",
            Self::PreExisting => "Pre-existing Issue",
            Self::Suppressed => "Suppressed Issue",
            Self::OutsideDiff => "Outside Diff Scope",
            Self::Skipped => "Skipped",
        }
    }

    /// Parse from a string.
    ///
    /// # Errors
    ///
    /// Returns `DispositionParseError` if the string doesn't match any known disposition.
    pub fn parse(s: &str) -> Result<Self, DispositionParseError> {
        match s.to_lowercase().as_str() {
            "new" => Ok(Self::New),
            "fixed" => Ok(Self::Fixed),
            "pre-existing" | "preexisting" | "pre_existent" => Ok(Self::PreExisting),
            "suppressed" => Ok(Self::Suppressed),
            "outside-diff" | "outside_diff" | "outside" => Ok(Self::OutsideDiff),
            "skipped" => Ok(Self::Skipped),
            _ => Err(DispositionParseError::new(s)),
        }
    }
}

impl fmt::Display for Disposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Disposition {
    type Err = DispositionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Error when parsing a disposition.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Unknown disposition: '{0}'")]
pub struct DispositionParseError(String);

impl DispositionParseError {
    /// Create a new parse error.
    #[must_use]
    pub fn new(input: &str) -> Self {
        Self(input.to_string())
    }

    /// Get the unknown input that caused the error.
    #[must_use]
    pub fn unknown_input(&self) -> &str {
        &self.0
    }
}

/// A reason for a disposition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispositionReason {
    /// Matched a suppression rule.
    SuppressedByRule(String),
    /// Outside the diff hunks.
    OutsideDiffHunks,
    /// On a generated file.
    GeneratedFile,
    /// On a vendor/third-party file.
    VendorFile,
    /// No span information available.
    NoSpanInfo,
    /// Custom reason.
    Custom(String),
}

impl DispositionReason {
    /// Create a suppression rule reason.
    #[must_use]
    pub fn suppressed_by(rule: impl Into<String>) -> Self {
        Self::SuppressedByRule(rule.into())
    }

    /// Create a custom reason.
    #[must_use]
    pub fn custom(reason: impl Into<String>) -> Self {
        Self::Custom(reason.into())
    }

    /// Get a description of the reason.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::SuppressedByRule(rule) => {
                format!("Suppressed by rule: {rule}")
            }
            Self::OutsideDiffHunks => "Outside diff hunks".to_string(),
            Self::GeneratedFile => "Generated file".to_string(),
            Self::VendorFile => "Vendor/third-party file".to_string(),
            Self::NoSpanInfo => "No span information".to_string(),
            Self::Custom(reason) => reason.clone(),
        }
    }
}

impl fmt::Display for DispositionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// A disposition with an optional reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispositionWithReason {
    /// The disposition.
    pub disposition: Disposition,
    /// The reason (if any).
    pub reason: Option<DispositionReason>,
}

impl DispositionWithReason {
    /// Create a new disposition without a reason.
    #[must_use]
    pub const fn new(disposition: Disposition) -> Self {
        Self {
            disposition,
            reason: None,
        }
    }

    /// Create a disposition with a reason.
    #[must_use]
    pub const fn with_reason(disposition: Disposition, reason: DispositionReason) -> Self {
        Self {
            disposition,
            reason: Some(reason),
        }
    }

    /// Create a new issue disposition.
    #[must_use]
    pub const fn new_issue() -> Self {
        Self::new(Disposition::New)
    }

    /// Create a fixed issue disposition.
    #[must_use]
    pub const fn fixed() -> Self {
        Self::new(Disposition::Fixed)
    }

    /// Create a pre-existing issue disposition.
    #[must_use]
    pub const fn pre_existing() -> Self {
        Self::new(Disposition::PreExisting)
    }

    /// Create a suppressed disposition with reason.
    #[must_use]
    pub fn suppressed(rule: impl Into<String>) -> Self {
        Self::with_reason(
            Disposition::Suppressed,
            DispositionReason::suppressed_by(rule),
        )
    }

    /// Create an outside-diff disposition.
    #[must_use]
    pub const fn outside_diff() -> Self {
        Self::with_reason(
            Disposition::OutsideDiff,
            DispositionReason::OutsideDiffHunks,
        )
    }

    /// Get the disposition.
    #[must_use]
    pub const fn as_disposition(&self) -> Disposition {
        self.disposition
    }

    /// Check if this has a reason.
    #[must_use]
    pub const fn has_reason(&self) -> bool {
        self.reason.is_some()
    }
}

impl From<Disposition> for DispositionWithReason {
    fn from(disposition: Disposition) -> Self {
        Self::new(disposition)
    }
}

impl fmt::Display for DispositionWithReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.disposition)?;
        if let Some(reason) = &self.reason {
            write!(f, " ({reason})")?;
        }
        Ok(())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Disposition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Disposition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disposition_default() {
        assert_eq!(Disposition::default(), Disposition::New);
    }

    #[test]
    fn test_disposition_is_new() {
        assert!(Disposition::New.is_new());
        assert!(!Disposition::Fixed.is_new());
    }

    #[test]
    fn test_disposition_is_fixed() {
        assert!(Disposition::Fixed.is_fixed());
        assert!(!Disposition::New.is_fixed());
    }

    #[test]
    fn test_disposition_is_actionable() {
        assert!(Disposition::New.is_actionable());
        assert!(Disposition::Fixed.is_actionable());
        assert!(!Disposition::PreExisting.is_actionable());
        assert!(!Disposition::Suppressed.is_actionable());
        assert!(!Disposition::OutsideDiff.is_actionable());
        assert!(!Disposition::Skipped.is_actionable());
    }

    #[test]
    fn test_disposition_is_reportable() {
        assert!(Disposition::New.is_reportable());
        assert!(Disposition::Fixed.is_reportable());
        assert!(Disposition::PreExisting.is_reportable());
        assert!(!Disposition::Suppressed.is_reportable());
        assert!(Disposition::OutsideDiff.is_reportable());
        assert!(!Disposition::Skipped.is_reportable());
    }

    #[test]
    fn test_disposition_counts_toward_failure() {
        assert!(Disposition::New.counts_toward_failure());
        assert!(!Disposition::Fixed.counts_toward_failure());
        assert!(!Disposition::PreExisting.counts_toward_failure());
    }

    #[test]
    fn test_disposition_as_str() {
        assert_eq!(Disposition::New.as_str(), "new");
        assert_eq!(Disposition::Fixed.as_str(), "fixed");
        assert_eq!(Disposition::PreExisting.as_str(), "pre-existing");
        assert_eq!(Disposition::Suppressed.as_str(), "suppressed");
        assert_eq!(Disposition::OutsideDiff.as_str(), "outside-diff");
        assert_eq!(Disposition::Skipped.as_str(), "skipped");
    }

    #[test]
    fn test_disposition_display() {
        assert_eq!(format!("{}", Disposition::New), "new");
        assert_eq!(format!("{}", Disposition::Fixed), "fixed");
    }

    #[test]
    fn test_disposition_parse() {
        assert_eq!(Disposition::parse("new").unwrap(), Disposition::New);
        assert_eq!(Disposition::parse("NEW").unwrap(), Disposition::New);
        assert_eq!(
            Disposition::parse("pre-existing").unwrap(),
            Disposition::PreExisting
        );
        assert!(Disposition::parse("invalid").is_err());
    }

    #[test]
    fn test_disposition_with_reason() {
        let dwr = DispositionWithReason::new(Disposition::New);
        assert_eq!(dwr.disposition, Disposition::New);
        assert!(!dwr.has_reason());
    }

    #[test]
    fn test_disposition_reason_description() {
        let reason = DispositionReason::suppressed_by("test-rule");
        assert_eq!(reason.description(), "Suppressed by rule: test-rule");
    }
}
