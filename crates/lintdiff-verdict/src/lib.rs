//! Lint check verdict types for lintdiff.
//!
//! Provides types for representing the result of a lint comparison
//! between the base and head revisions.

use std::fmt;

/// The overall verdict of a lintdiff check.
///
/// # Examples
/// ```
/// use lintdiff_verdict::Verdict;
///
/// let verdict = Verdict::Pass;
/// assert!(verdict.is_success());
/// assert!(!verdict.is_failure());
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Verdict {
    /// No new issues introduced, no blocking findings.
    #[default]
    Pass,
    /// New issues introduced but below failure threshold.
    Warn,
    /// Blocking issues found or failure threshold exceeded.
    Fail,
}

impl Verdict {
    /// Check if this verdict represents a successful outcome.
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Pass | Self::Warn)
    }

    /// Check if this verdict represents a failure.
    #[must_use]
    pub const fn is_failure(self) -> bool {
        matches!(self, Self::Fail)
    }

    /// Check if this is a warning verdict.
    #[must_use]
    pub const fn is_warning(self) -> bool {
        matches!(self, Self::Warn)
    }

    /// Get an exit code for this verdict.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_verdict::Verdict;
    ///
    /// assert_eq!(Verdict::Pass.exit_code(), 0);
    /// assert_eq!(Verdict::Warn.exit_code(), 0);
    /// assert_eq!(Verdict::Fail.exit_code(), 1);
    /// ```
    #[must_use]
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::Pass | Self::Warn => 0,
            Self::Fail => 1,
        }
    }

    /// Get a string representation for reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
    }

    /// Get an icon for display.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Pass => "✅",
            Self::Warn => "⚠️",
            Self::Fail => "❌",
        }
    }

    /// Combine two verdicts, taking the more severe one.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_verdict::Verdict;
    ///
    /// assert_eq!(Verdict::Pass.combine(Verdict::Warn), Verdict::Warn);
    /// assert_eq!(Verdict::Warn.combine(Verdict::Pass), Verdict::Warn);
    /// assert_eq!(Verdict::Pass.combine(Verdict::Fail), Verdict::Fail);
    /// ```
    #[must_use]
    pub const fn combine(self, other: Self) -> Self {
        // Order: Pass < Warn < Fail
        match (self, other) {
            (Self::Fail, _) | (_, Self::Fail) => Self::Fail,
            (Self::Warn, _) | (_, Self::Warn) => Self::Warn,
            _ => Self::Pass,
        }
    }

    /// Create a verdict from a boolean.
    #[must_use]
    pub const fn from_bool(success: bool) -> Self {
        if success {
            Self::Pass
        } else {
            Self::Fail
        }
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Statistics about findings that contribute to a verdict.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FindingCounts {
    /// Number of new errors introduced.
    pub new_errors: u32,
    /// Number of new warnings introduced.
    pub new_warnings: u32,
    /// Number of fixed issues.
    pub fixed: u32,
    /// Number of pre-existing issues.
    pub pre_existing: u32,
}

impl FindingCounts {
    /// Create a new empty count.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            new_errors: 0,
            new_warnings: 0,
            fixed: 0,
            pre_existing: 0,
        }
    }

    /// Create counts with specific values.
    #[must_use]
    pub const fn from_counts(new_errors: u32, new_warnings: u32, fixed: u32, pre_existing: u32) -> Self {
        Self {
            new_errors,
            new_warnings,
            fixed,
            pre_existing,
        }
    }

    /// Total number of new issues.
    #[must_use]
    pub const fn total_new(&self) -> u32 {
        self.new_errors + self.new_warnings
    }

    /// Check if there are any new issues.
    #[must_use]
    pub const fn has_new_issues(&self) -> bool {
        self.total_new() > 0
    }

    /// Check if there are any new errors.
    #[must_use]
    pub const fn has_new_errors(&self) -> bool {
        self.new_errors > 0
    }

    /// Determine the verdict based on counts and thresholds.
    ///
    /// # Arguments
    /// * `fail_on_error` - Fail if any new errors exist
    /// * `max_warnings` - Maximum allowed new warnings (None = unlimited)
    ///
    /// # Examples
    /// ```
    /// use lintdiff_verdict::{Verdict, FindingCounts};
    ///
    /// let counts = FindingCounts::from_counts(0, 5, 2, 10);
    /// assert_eq!(counts.to_verdict(true, Some(10)), Verdict::Warn);
    /// assert_eq!(counts.to_verdict(true, Some(3)), Verdict::Fail);
    /// ```
    #[must_use]
    pub const fn to_verdict(&self, fail_on_error: bool, max_warnings: Option<u32>) -> Verdict {
        if fail_on_error && self.new_errors > 0 {
            return Verdict::Fail;
        }

        if let Some(max) = max_warnings {
            if self.new_warnings > max {
                return Verdict::Fail;
            }
        }

        if self.has_new_issues() {
            Verdict::Warn
        } else {
            Verdict::Pass
        }
    }
}

/// A complete verdict with context.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VerdictReport {
    /// The overall verdict.
    pub verdict: Verdict,
    /// Finding counts.
    pub counts: FindingCounts,
    /// Optional message explaining the verdict.
    pub message: Option<String>,
}

impl VerdictReport {
    /// Create a new report.
    #[must_use]
    pub const fn new(verdict: Verdict, counts: FindingCounts) -> Self {
        Self {
            verdict,
            counts,
            message: None,
        }
    }

    /// Add a message to the report.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Create a passing report.
    #[must_use]
    pub const fn pass(counts: FindingCounts) -> Self {
        Self::new(Verdict::Pass, counts)
    }

    /// Create a warning report.
    #[must_use]
    pub const fn warn(counts: FindingCounts) -> Self {
        Self::new(Verdict::Warn, counts)
    }

    /// Create a failing report.
    #[must_use]
    pub const fn fail(counts: FindingCounts) -> Self {
        Self::new(Verdict::Fail, counts)
    }

    /// Check if this is a passing report.
    #[must_use]
    pub fn is_pass(&self) -> bool {
        self.verdict == Verdict::Pass
    }
}

impl From<VerdictReport> for Verdict {
    fn from(report: VerdictReport) -> Self {
        report.verdict
    }
}

/// Threshold configuration for determining verdicts.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VerdictThresholds {
    /// Fail if any new errors exist.
    pub fail_on_error: bool,
    /// Maximum allowed new warnings.
    pub max_warnings: Option<u32>,
}

impl VerdictThresholds {
    /// Create new thresholds.
    #[must_use]
    pub const fn new(fail_on_error: bool, max_warnings: Option<u32>) -> Self {
        Self {
            fail_on_error,
            max_warnings,
        }
    }

    /// Fail on any new issues (strict mode).
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            fail_on_error: true,
            max_warnings: Some(0),
        }
    }

    /// Allow warnings but fail on errors.
    #[must_use]
    pub const fn fail_on_errors_only() -> Self {
        Self {
            fail_on_error: true,
            max_warnings: None,
        }
    }

    /// Allow all issues (reporting only).
    #[must_use]
    pub const fn lenient() -> Self {
        Self {
            fail_on_error: false,
            max_warnings: None,
        }
    }

    /// Evaluate counts against these thresholds.
    #[must_use]
    pub const fn evaluate(&self, counts: &FindingCounts) -> Verdict {
        counts.to_verdict(self.fail_on_error, self.max_warnings)
    }
}

impl Default for VerdictThresholds {
    fn default() -> Self {
        Self::fail_on_errors_only()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verdict_pass_is_success() {
        assert!(Verdict::Pass.is_success());
    }

    #[test]
    fn test_verdict_warn_is_success() {
        assert!(Verdict::Warn.is_success());
    }

    #[test]
    fn test_verdict_fail_is_not_success() {
        assert!(!Verdict::Fail.is_success());
    }
}
