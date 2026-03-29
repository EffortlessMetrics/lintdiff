//! Verdict reason generation and formatting for lintdiff.
//!
//! This microcrate provides utilities for generating and formatting verdict reasons
//! that explain why a particular verdict was reached.
//!
//! # Overview
//!
//! - [`VerdictReason`] - Enum representing reasons for verdict decisions
//! - [`VerdictReasonBuilder`] - Builder for constructing verdict reasons
//! - [`VerdictSummary`] - Combined summary with reason, details, and suggestions
//! - Formatting functions for human-readable, short, and markdown output
//!
//! # Examples
//!
//! ```
//! use lintdiff_verdict_reason::{VerdictReason, VerdictReasonBuilder, format_reason};
//!
//! // Using the builder
//! let reason = VerdictReasonBuilder::new()
//!     .with_added(2, 5)
//!     .with_removed(1, 3)
//!     .build();
//!
//! // Format for display
//! println!("{}", format_reason(&reason));
//! ```

use std::fmt;

/// Reasons for verdict decisions.
///
/// Each variant represents a specific reason why a verdict was reached.
///
/// # Examples
///
/// ```
/// use lintdiff_verdict_reason::VerdictReason;
///
/// let reason = VerdictReason::AddedWarnings { count: 5 };
/// assert!(lintdiff_verdict_reason::is_failure_reason(&reason));
///
/// let reason = VerdictReason::NoChanges;
/// assert!(lintdiff_verdict_reason::is_success_reason(&reason));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VerdictReason {
    /// No diagnostic changes detected between base and head.
    #[default]
    NoChanges,
    /// New warnings introduced in the changes.
    AddedWarnings {
        /// Number of warnings added.
        count: usize,
    },
    /// New errors introduced in the changes.
    AddedErrors {
        /// Number of errors added.
        count: usize,
    },
    /// Warnings fixed by the changes.
    RemovedWarnings {
        /// Number of warnings removed.
        count: usize,
    },
    /// Errors fixed by the changes.
    RemovedErrors {
        /// Number of errors removed.
        count: usize,
    },
    /// Only unchanged (pre-existing) diagnostics found.
    OnlyUnchanged,
    /// A configured threshold was exceeded.
    ThresholdExceeded {
        /// The configured limit.
        limit: usize,
        /// The actual count observed.
        actual: usize,
    },
    /// A custom reason for specialized cases.
    Custom(String),
}

impl VerdictReason {
    /// Get a short string identifier for this reason.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReason;
    ///
    /// assert_eq!(VerdictReason::NoChanges.as_str(), "no-changes");
    /// assert_eq!(VerdictReason::AddedWarnings { count: 5 }.as_str(), "added-warnings");
    /// ```
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NoChanges => "no-changes",
            Self::AddedWarnings { .. } => "added-warnings",
            Self::AddedErrors { .. } => "added-errors",
            Self::RemovedWarnings { .. } => "removed-warnings",
            Self::RemovedErrors { .. } => "removed-errors",
            Self::OnlyUnchanged => "only-unchanged",
            Self::ThresholdExceeded { .. } => "threshold-exceeded",
            Self::Custom(_) => "custom",
        }
    }

    /// Check if this reason has an associated count.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReason;
    ///
    /// assert!(VerdictReason::AddedWarnings { count: 5 }.has_count());
    /// assert!(!VerdictReason::NoChanges.has_count());
    /// ```
    #[must_use]
    pub const fn has_count(&self) -> bool {
        matches!(
            self,
            Self::AddedWarnings { .. }
                | Self::AddedErrors { .. }
                | Self::RemovedWarnings { .. }
                | Self::RemovedErrors { .. }
        )
    }

    /// Get the count if this reason has one.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReason;
    ///
    /// assert_eq!(VerdictReason::AddedWarnings { count: 5 }.count(), Some(5));
    /// assert_eq!(VerdictReason::NoChanges.count(), None);
    /// ```
    #[must_use]
    pub const fn count(&self) -> Option<usize> {
        match self {
            Self::AddedWarnings { count }
            | Self::AddedErrors { count }
            | Self::RemovedWarnings { count }
            | Self::RemovedErrors { count } => Some(*count),
            _ => None,
        }
    }

    /// Check if this reason is related to added diagnostics.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReason;
    ///
    /// assert!(VerdictReason::AddedWarnings { count: 1 }.is_added());
    /// assert!(VerdictReason::AddedErrors { count: 1 }.is_added());
    /// assert!(!VerdictReason::RemovedWarnings { count: 1 }.is_added());
    /// ```
    #[must_use]
    pub const fn is_added(&self) -> bool {
        matches!(self, Self::AddedWarnings { .. } | Self::AddedErrors { .. })
    }

    /// Check if this reason is related to removed diagnostics.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReason;
    ///
    /// assert!(VerdictReason::RemovedWarnings { count: 1 }.is_removed());
    /// assert!(VerdictReason::RemovedErrors { count: 1 }.is_removed());
    /// assert!(!VerdictReason::AddedWarnings { count: 1 }.is_removed());
    /// ```
    #[must_use]
    pub const fn is_removed(&self) -> bool {
        matches!(
            self,
            Self::RemovedWarnings { .. } | Self::RemovedErrors { .. }
        )
    }

    /// Check if this reason is related to warnings.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReason;
    ///
    /// assert!(VerdictReason::AddedWarnings { count: 1 }.is_warning_related());
    /// assert!(VerdictReason::RemovedWarnings { count: 1 }.is_warning_related());
    /// assert!(!VerdictReason::AddedErrors { count: 1 }.is_warning_related());
    /// ```
    #[must_use]
    pub const fn is_warning_related(&self) -> bool {
        matches!(
            self,
            Self::AddedWarnings { .. } | Self::RemovedWarnings { .. }
        )
    }

    /// Check if this reason is related to errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReason;
    ///
    /// assert!(VerdictReason::AddedErrors { count: 1 }.is_error_related());
    /// assert!(VerdictReason::RemovedErrors { count: 1 }.is_error_related());
    /// assert!(!VerdictReason::AddedWarnings { count: 1 }.is_error_related());
    /// ```
    #[must_use]
    pub const fn is_error_related(&self) -> bool {
        matches!(self, Self::AddedErrors { .. } | Self::RemovedErrors { .. })
    }

    /// Get an icon for this reason.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReason;
    ///
    /// assert_eq!(VerdictReason::NoChanges.icon(), "✅");
    /// assert_eq!(VerdictReason::AddedErrors { count: 1 }.icon(), "❌");
    /// ```
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::NoChanges => "✅",
            Self::AddedWarnings { .. } => "⚠️",
            Self::AddedErrors { .. } => "❌",
            Self::RemovedWarnings { .. } => "🩹",
            Self::RemovedErrors { .. } => "🔧",
            Self::OnlyUnchanged => "⏳",
            Self::ThresholdExceeded { .. } => "🚫",
            Self::Custom(_) => "📝",
        }
    }
}

impl fmt::Display for VerdictReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoChanges => write!(f, "No diagnostic changes detected"),
            Self::AddedWarnings { count } => {
                write!(
                    f,
                    "Added {} warning{}",
                    count,
                    if *count == 1 { "" } else { "s" }
                )
            }
            Self::AddedErrors { count } => {
                write!(
                    f,
                    "Added {} error{}",
                    count,
                    if *count == 1 { "" } else { "s" }
                )
            }
            Self::RemovedWarnings { count } => {
                write!(
                    f,
                    "Fixed {} warning{}",
                    count,
                    if *count == 1 { "" } else { "s" }
                )
            }
            Self::RemovedErrors { count } => {
                write!(
                    f,
                    "Fixed {} error{}",
                    count,
                    if *count == 1 { "" } else { "s" }
                )
            }
            Self::OnlyUnchanged => write!(f, "Only unchanged diagnostics found"),
            Self::ThresholdExceeded { limit, actual } => {
                write!(f, "Threshold exceeded: {actual} > {limit}")
            }
            Self::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

/// Builder for constructing verdict reasons.
///
/// This builder accumulates information about diagnostic changes and
/// produces an appropriate [`VerdictReason`].
///
/// # Examples
///
/// ```
/// use lintdiff_verdict_reason::VerdictReasonBuilder;
///
/// let reason = VerdictReasonBuilder::new()
///     .with_added(2, 5)
///     .with_removed(1, 3)
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct VerdictReasonBuilder {
    added_errors: usize,
    added_warnings: usize,
    removed_errors: usize,
    removed_warnings: usize,
    unchanged: usize,
    threshold_limit: Option<usize>,
    threshold_actual: Option<usize>,
    custom_reason: Option<String>,
}

impl VerdictReasonBuilder {
    /// Create a new builder with default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReasonBuilder;
    ///
    /// let builder = VerdictReasonBuilder::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            added_errors: 0,
            added_warnings: 0,
            removed_errors: 0,
            removed_warnings: 0,
            unchanged: 0,
            threshold_limit: None,
            threshold_actual: None,
            custom_reason: None,
        }
    }

    /// Add information about added diagnostics.
    ///
    /// # Arguments
    ///
    /// * `errors` - Number of errors added
    /// * `warnings` - Number of warnings added
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReasonBuilder;
    ///
    /// let reason = VerdictReasonBuilder::new()
    ///     .with_added(2, 5)
    ///     .build();
    /// ```
    #[must_use]
    pub const fn with_added(mut self, errors: usize, warnings: usize) -> Self {
        self.added_errors = errors;
        self.added_warnings = warnings;
        self
    }

    /// Add information about removed (fixed) diagnostics.
    ///
    /// # Arguments
    ///
    /// * `errors` - Number of errors removed
    /// * `warnings` - Number of warnings removed
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReasonBuilder;
    ///
    /// let reason = VerdictReasonBuilder::new()
    ///     .with_removed(1, 3)
    ///     .build();
    /// ```
    #[must_use]
    pub const fn with_removed(mut self, errors: usize, warnings: usize) -> Self {
        self.removed_errors = errors;
        self.removed_warnings = warnings;
        self
    }

    /// Add information about unchanged diagnostics.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of unchanged diagnostics
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReasonBuilder;
    ///
    /// let reason = VerdictReasonBuilder::new()
    ///     .with_unchanged(10)
    ///     .build();
    /// ```
    #[must_use]
    pub const fn with_unchanged(mut self, count: usize) -> Self {
        self.unchanged = count;
        self
    }

    /// Set threshold information.
    ///
    /// # Arguments
    ///
    /// * `limit` - The configured threshold limit
    /// * `actual` - The actual count observed
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReasonBuilder;
    ///
    /// let reason = VerdictReasonBuilder::new()
    ///     .with_threshold(10, 15)
    ///     .build();
    /// ```
    #[must_use]
    pub const fn with_threshold(mut self, limit: usize, actual: usize) -> Self {
        self.threshold_limit = Some(limit);
        self.threshold_actual = Some(actual);
        self
    }

    /// Set a custom reason.
    ///
    /// # Arguments
    ///
    /// * `reason` - Custom reason string
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::VerdictReasonBuilder;
    ///
    /// let reason = VerdictReasonBuilder::new()
    ///     .with_custom("Special case handled".to_string())
    ///     .build();
    /// ```
    #[must_use]
    pub fn with_custom(mut self, reason: String) -> Self {
        self.custom_reason = Some(reason);
        self
    }

    /// Build the final verdict reason.
    ///
    /// The builder prioritizes reasons in the following order:
    /// 1. Custom reason (if set)
    /// 2. Threshold exceeded (if actual > limit)
    /// 3. Added errors (if any)
    /// 4. Added warnings (if any)
    /// 5. Removed errors (if any)
    /// 6. Removed warnings (if any)
    /// 7. Only unchanged (if unchanged > 0)
    /// 8. No changes (default)
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::{VerdictReasonBuilder, VerdictReason};
    ///
    /// let reason = VerdictReasonBuilder::new()
    ///     .with_added(2, 5)
    ///     .build();
    ///
    /// // Errors take priority over warnings
    /// assert!(matches!(reason, VerdictReason::AddedErrors { count: 2 }));
    /// ```
    #[must_use]
    pub fn build(&self) -> VerdictReason {
        // Custom reason takes highest priority
        if let Some(ref reason) = self.custom_reason {
            return VerdictReason::Custom(reason.clone());
        }

        // Check threshold exceeded
        if let (Some(limit), Some(actual)) = (self.threshold_limit, self.threshold_actual) {
            if actual > limit {
                return VerdictReason::ThresholdExceeded { limit, actual };
            }
        }

        // Added diagnostics take priority (errors > warnings)
        if self.added_errors > 0 {
            return VerdictReason::AddedErrors {
                count: self.added_errors,
            };
        }
        if self.added_warnings > 0 {
            return VerdictReason::AddedWarnings {
                count: self.added_warnings,
            };
        }

        // Removed diagnostics (errors > warnings)
        if self.removed_errors > 0 {
            return VerdictReason::RemovedErrors {
                count: self.removed_errors,
            };
        }
        if self.removed_warnings > 0 {
            return VerdictReason::RemovedWarnings {
                count: self.removed_warnings,
            };
        }

        // Only unchanged diagnostics
        if self.unchanged > 0 {
            return VerdictReason::OnlyUnchanged;
        }

        // Default: no changes
        VerdictReason::NoChanges
    }
}

/// Combined summary with reason, details, and suggestions.
///
/// # Examples
///
/// ```
/// use lintdiff_verdict_reason::{VerdictSummary, VerdictReason};
///
/// let summary = VerdictSummary::new(VerdictReason::AddedWarnings { count: 5 })
///     .with_detail("Found in src/lib.rs")
///     .with_suggestion("Review the new warnings and fix if necessary");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VerdictSummary {
    /// The primary reason for the verdict.
    pub reason: VerdictReason,
    /// Additional details about the verdict.
    pub details: Vec<String>,
    /// Suggested action to take.
    pub suggestion: Option<String>,
}

impl VerdictSummary {
    /// Create a new summary with the given reason.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::{VerdictSummary, VerdictReason};
    ///
    /// let summary = VerdictSummary::new(VerdictReason::NoChanges);
    /// ```
    #[must_use]
    pub const fn new(reason: VerdictReason) -> Self {
        Self {
            reason,
            details: Vec::new(),
            suggestion: None,
        }
    }

    /// Add a detail to the summary.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::{VerdictSummary, VerdictReason};
    ///
    /// let summary = VerdictSummary::new(VerdictReason::NoChanges)
    ///     .with_detail("No diagnostics found in the diff");
    /// ```
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }

    /// Add multiple details to the summary.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::{VerdictSummary, VerdictReason};
    ///
    /// let summary = VerdictSummary::new(VerdictReason::NoChanges)
    ///     .with_details(vec!["Detail 1".to_string(), "Detail 2".to_string()]);
    /// ```
    #[must_use]
    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }

    /// Set a suggestion for the summary.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::{VerdictSummary, VerdictReason};
    ///
    /// let summary = VerdictSummary::new(VerdictReason::AddedWarnings { count: 5 })
    ///     .with_suggestion("Review and fix the warnings");
    /// ```
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Check if this summary indicates a failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::{VerdictSummary, VerdictReason};
    ///
    /// let summary = VerdictSummary::new(VerdictReason::AddedErrors { count: 1 });
    /// assert!(summary.is_failure());
    /// ```
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        is_failure_reason(&self.reason)
    }

    /// Check if this summary indicates a success.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::{VerdictSummary, VerdictReason};
    ///
    /// let summary = VerdictSummary::new(VerdictReason::NoChanges);
    /// assert!(summary.is_success());
    /// ```
    #[must_use]
    pub const fn is_success(&self) -> bool {
        is_success_reason(&self.reason)
    }

    /// Check if this summary has any details.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::{VerdictSummary, VerdictReason};
    ///
    /// let summary = VerdictSummary::new(VerdictReason::NoChanges)
    ///     .with_detail("Some detail");
    /// assert!(summary.has_details());
    /// ```
    #[must_use]
    pub const fn has_details(&self) -> bool {
        !self.details.is_empty()
    }

    /// Check if this summary has a suggestion.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_verdict_reason::{VerdictSummary, VerdictReason};
    ///
    /// let summary = VerdictSummary::new(VerdictReason::NoChanges)
    ///     .with_suggestion("Some suggestion");
    /// assert!(summary.has_suggestion());
    /// ```
    #[must_use]
    pub const fn has_suggestion(&self) -> bool {
        self.suggestion.is_some()
    }
}

impl fmt::Display for VerdictSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Reason: {}", self.reason)?;

        if !self.details.is_empty() {
            writeln!(f, "Details:")?;
            for detail in &self.details {
                writeln!(f, "  - {detail}")?;
            }
        }

        if let Some(ref suggestion) = self.suggestion {
            writeln!(f, "Suggestion: {suggestion}")?;
        }

        Ok(())
    }
}

/// Format a reason in human-readable form.
///
/// # Examples
///
/// ```
/// use lintdiff_verdict_reason::{VerdictReason, format_reason};
///
/// let reason = VerdictReason::AddedWarnings { count: 5 };
/// assert_eq!(format_reason(&reason), "⚠️ Added 5 warnings");
/// ```
#[must_use]
pub fn format_reason(reason: &VerdictReason) -> String {
    format!("{} {}", reason.icon(), reason)
}

/// Format a reason in short form for CI output.
///
/// # Examples
///
/// ```
/// use lintdiff_verdict_reason::{VerdictReason, format_reason_short};
///
/// let reason = VerdictReason::AddedWarnings { count: 5 };
/// assert_eq!(format_reason_short(&reason), "added-warnings:5");
/// ```
#[must_use]
pub fn format_reason_short(reason: &VerdictReason) -> String {
    match reason {
        VerdictReason::NoChanges => "no-changes".to_string(),
        VerdictReason::AddedWarnings { count } => format!("added-warnings:{count}"),
        VerdictReason::AddedErrors { count } => format!("added-errors:{count}"),
        VerdictReason::RemovedWarnings { count } => format!("removed-warnings:{count}"),
        VerdictReason::RemovedErrors { count } => format!("removed-errors:{count}"),
        VerdictReason::OnlyUnchanged => "only-unchanged".to_string(),
        VerdictReason::ThresholdExceeded { limit, actual } => {
            format!("threshold-exceeded:{actual}/{limit}")
        }
        VerdictReason::Custom(msg) => format!("custom:{msg}"),
    }
}

/// Format a reason in markdown format.
///
/// # Examples
///
/// ```
/// use lintdiff_verdict_reason::{VerdictReason, format_reason_markdown};
///
/// let reason = VerdictReason::AddedWarnings { count: 5 };
/// let md = format_reason_markdown(&reason);
/// assert!(md.contains("**⚠️ Added Warnings**"));
/// assert!(md.contains("5 new warnings"));
/// ```
#[must_use]
pub fn format_reason_markdown(reason: &VerdictReason) -> String {
    match reason {
        VerdictReason::NoChanges => {
            "**✅ No Changes** - No diagnostic changes detected".to_string()
        }
        VerdictReason::AddedWarnings { count } => {
            format!(
                "**⚠️ Added Warnings** - {count} new warning{} introduced",
                if *count == 1 { "" } else { "s" }
            )
        }
        VerdictReason::AddedErrors { count } => {
            format!(
                "**❌ Added Errors** - {count} new error{} introduced",
                if *count == 1 { "" } else { "s" }
            )
        }
        VerdictReason::RemovedWarnings { count } => {
            format!(
                "**🩹 Fixed Warnings** - {count} warning{} resolved",
                if *count == 1 { "" } else { "s" }
            )
        }
        VerdictReason::RemovedErrors { count } => {
            format!(
                "**🔧 Fixed Errors** - {count} error{} resolved",
                if *count == 1 { "" } else { "s" }
            )
        }
        VerdictReason::OnlyUnchanged => {
            "**⏳ Only Unchanged** - Only pre-existing diagnostics found".to_string()
        }
        VerdictReason::ThresholdExceeded { limit, actual } => {
            format!("**🚫 Threshold Exceeded** - {actual} exceeds limit of {limit}")
        }
        VerdictReason::Custom(msg) => format!("**📝 Custom** - {msg}"),
    }
}

/// Check if a reason indicates a failure.
///
/// Failure reasons are:
/// - Added errors
/// - Added warnings
/// - Threshold exceeded
/// - Custom reasons (assumed potentially failing)
///
/// # Examples
///
/// ```
/// use lintdiff_verdict_reason::{VerdictReason, is_failure_reason};
///
/// assert!(is_failure_reason(&VerdictReason::AddedErrors { count: 1 }));
/// assert!(is_failure_reason(&VerdictReason::AddedWarnings { count: 1 }));
/// assert!(!is_failure_reason(&VerdictReason::NoChanges));
/// assert!(!is_failure_reason(&VerdictReason::RemovedErrors { count: 1 }));
/// ```
#[must_use]
pub const fn is_failure_reason(reason: &VerdictReason) -> bool {
    matches!(
        reason,
        VerdictReason::AddedErrors { .. }
            | VerdictReason::AddedWarnings { .. }
            | VerdictReason::ThresholdExceeded { .. }
            | VerdictReason::Custom(_)
    )
}

/// Check if a reason indicates a success.
///
/// Success reasons are:
/// - No changes
/// - Removed errors
/// - Removed warnings
/// - Only unchanged
///
/// # Examples
///
/// ```
/// use lintdiff_verdict_reason::{VerdictReason, is_success_reason};
///
/// assert!(is_success_reason(&VerdictReason::NoChanges));
/// assert!(is_success_reason(&VerdictReason::RemovedErrors { count: 1 }));
/// assert!(!is_success_reason(&VerdictReason::AddedErrors { count: 1 }));
/// ```
#[must_use]
pub const fn is_success_reason(reason: &VerdictReason) -> bool {
    matches!(
        reason,
        VerdictReason::NoChanges
            | VerdictReason::RemovedErrors { .. }
            | VerdictReason::RemovedWarnings { .. }
            | VerdictReason::OnlyUnchanged
    )
}

/// Merge multiple reasons into a single reason.
///
/// The merge prioritizes reasons in the following order:
/// 1. Added errors (highest count)
/// 2. Added warnings (highest count)
/// 3. Threshold exceeded (highest excess)
/// 4. Custom reasons (first one)
/// 5. Removed errors (highest count)
/// 6. Removed warnings (highest count)
/// 7. Only unchanged
/// 8. No changes (default)
///
/// # Examples
///
/// ```
/// use lintdiff_verdict_reason::{VerdictReason, merge_reasons};
///
/// let reasons = vec![
///     VerdictReason::AddedWarnings { count: 2 },
///     VerdictReason::RemovedErrors { count: 1 },
/// ];
/// let merged = merge_reasons(&reasons);
/// // Added warnings takes priority over removed errors
/// assert!(matches!(merged, VerdictReason::AddedWarnings { .. }));
/// ```
#[must_use]
pub fn merge_reasons(reasons: &[VerdictReason]) -> VerdictReason {
    if reasons.is_empty() {
        return VerdictReason::NoChanges;
    }

    let mut max_added_errors: usize = 0;
    let mut max_added_warnings: usize = 0;
    let mut max_removed_errors: usize = 0;
    let mut max_removed_warnings: usize = 0;
    let mut has_only_unchanged = false;
    let mut threshold_exceeded: Option<(usize, usize)> = None;
    let mut custom_reason: Option<String> = None;

    for reason in reasons {
        match reason {
            VerdictReason::AddedErrors { count } => {
                max_added_errors = max_added_errors.max(*count);
            }
            VerdictReason::AddedWarnings { count } => {
                max_added_warnings = max_added_warnings.max(*count);
            }
            VerdictReason::RemovedErrors { count } => {
                max_removed_errors = max_removed_errors.max(*count);
            }
            VerdictReason::RemovedWarnings { count } => {
                max_removed_warnings = max_removed_warnings.max(*count);
            }
            VerdictReason::OnlyUnchanged => {
                has_only_unchanged = true;
            }
            VerdictReason::ThresholdExceeded { limit, actual } => {
                let excess = actual.saturating_sub(*limit);
                let current_excess = threshold_exceeded.map_or(0, |(l, a)| a.saturating_sub(l));
                if excess > current_excess {
                    threshold_exceeded = Some((*limit, *actual));
                }
            }
            VerdictReason::Custom(msg) => {
                if custom_reason.is_none() {
                    custom_reason = Some(msg.clone());
                }
            }
            VerdictReason::NoChanges => {}
        }
    }

    // Return in priority order
    if let Some(msg) = custom_reason {
        return VerdictReason::Custom(msg);
    }

    if let Some((limit, actual)) = threshold_exceeded {
        return VerdictReason::ThresholdExceeded { limit, actual };
    }

    if max_added_errors > 0 {
        return VerdictReason::AddedErrors {
            count: max_added_errors,
        };
    }

    if max_added_warnings > 0 {
        return VerdictReason::AddedWarnings {
            count: max_added_warnings,
        };
    }

    if max_removed_errors > 0 {
        return VerdictReason::RemovedErrors {
            count: max_removed_errors,
        };
    }

    if max_removed_warnings > 0 {
        return VerdictReason::RemovedWarnings {
            count: max_removed_warnings,
        };
    }

    if has_only_unchanged {
        return VerdictReason::OnlyUnchanged;
    }

    VerdictReason::NoChanges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verdict_reason_default() {
        let reason = VerdictReason::default();
        assert_eq!(reason, VerdictReason::NoChanges);
    }

    #[test]
    fn test_verdict_reason_as_str() {
        assert_eq!(VerdictReason::NoChanges.as_str(), "no-changes");
        assert_eq!(
            VerdictReason::AddedWarnings { count: 5 }.as_str(),
            "added-warnings"
        );
        assert_eq!(
            VerdictReason::AddedErrors { count: 3 }.as_str(),
            "added-errors"
        );
        assert_eq!(
            VerdictReason::RemovedWarnings { count: 2 }.as_str(),
            "removed-warnings"
        );
        assert_eq!(
            VerdictReason::RemovedErrors { count: 1 }.as_str(),
            "removed-errors"
        );
        assert_eq!(VerdictReason::OnlyUnchanged.as_str(), "only-unchanged");
        assert_eq!(
            VerdictReason::ThresholdExceeded {
                limit: 10,
                actual: 15
            }
            .as_str(),
            "threshold-exceeded"
        );
        assert_eq!(VerdictReason::Custom("test".to_string()).as_str(), "custom");
    }

    #[test]
    fn test_verdict_reason_display() {
        assert_eq!(
            format!("{}", VerdictReason::NoChanges),
            "No diagnostic changes detected"
        );
        assert_eq!(
            format!("{}", VerdictReason::AddedWarnings { count: 1 }),
            "Added 1 warning"
        );
        assert_eq!(
            format!("{}", VerdictReason::AddedWarnings { count: 5 }),
            "Added 5 warnings"
        );
        assert_eq!(
            format!("{}", VerdictReason::AddedErrors { count: 1 }),
            "Added 1 error"
        );
        assert_eq!(
            format!("{}", VerdictReason::AddedErrors { count: 3 }),
            "Added 3 errors"
        );
    }

    #[test]
    fn test_builder_new() {
        let builder = VerdictReasonBuilder::new();
        assert_eq!(builder.added_errors, 0);
        assert_eq!(builder.added_warnings, 0);
        assert_eq!(builder.removed_errors, 0);
        assert_eq!(builder.removed_warnings, 0);
        assert_eq!(builder.unchanged, 0);
    }

    #[test]
    fn test_builder_build_no_changes() {
        let reason = VerdictReasonBuilder::new().build();
        assert_eq!(reason, VerdictReason::NoChanges);
    }

    #[test]
    fn test_builder_build_with_added_errors() {
        let reason = VerdictReasonBuilder::new().with_added(3, 0).build();
        assert_eq!(reason, VerdictReason::AddedErrors { count: 3 });
    }

    #[test]
    fn test_builder_build_with_added_warnings() {
        let reason = VerdictReasonBuilder::new().with_added(0, 5).build();
        assert_eq!(reason, VerdictReason::AddedWarnings { count: 5 });
    }

    #[test]
    fn test_builder_errors_take_priority() {
        let reason = VerdictReasonBuilder::new().with_added(2, 5).build();
        assert_eq!(reason, VerdictReason::AddedErrors { count: 2 });
    }

    #[test]
    fn test_format_reason() {
        let reason = VerdictReason::NoChanges;
        assert_eq!(format_reason(&reason), "✅ No diagnostic changes detected");

        let reason = VerdictReason::AddedWarnings { count: 5 };
        assert_eq!(format_reason(&reason), "⚠️ Added 5 warnings");
    }

    #[test]
    fn test_format_reason_short() {
        assert_eq!(format_reason_short(&VerdictReason::NoChanges), "no-changes");
        assert_eq!(
            format_reason_short(&VerdictReason::AddedWarnings { count: 5 }),
            "added-warnings:5"
        );
        assert_eq!(
            format_reason_short(&VerdictReason::ThresholdExceeded {
                limit: 10,
                actual: 15
            }),
            "threshold-exceeded:15/10"
        );
    }

    #[test]
    fn test_is_failure_reason() {
        assert!(is_failure_reason(&VerdictReason::AddedErrors { count: 1 }));
        assert!(is_failure_reason(&VerdictReason::AddedWarnings {
            count: 1
        }));
        assert!(is_failure_reason(&VerdictReason::ThresholdExceeded {
            limit: 10,
            actual: 15
        }));
        assert!(is_failure_reason(&VerdictReason::Custom(
            "test".to_string()
        )));

        assert!(!is_failure_reason(&VerdictReason::NoChanges));
        assert!(!is_failure_reason(&VerdictReason::RemovedErrors {
            count: 1
        }));
        assert!(!is_failure_reason(&VerdictReason::RemovedWarnings {
            count: 1
        }));
        assert!(!is_failure_reason(&VerdictReason::OnlyUnchanged));
    }

    #[test]
    fn test_is_success_reason() {
        assert!(is_success_reason(&VerdictReason::NoChanges));
        assert!(is_success_reason(&VerdictReason::RemovedErrors {
            count: 1
        }));
        assert!(is_success_reason(&VerdictReason::RemovedWarnings {
            count: 1
        }));
        assert!(is_success_reason(&VerdictReason::OnlyUnchanged));

        assert!(!is_success_reason(&VerdictReason::AddedErrors { count: 1 }));
        assert!(!is_success_reason(&VerdictReason::AddedWarnings {
            count: 1
        }));
        assert!(!is_success_reason(&VerdictReason::ThresholdExceeded {
            limit: 10,
            actual: 15
        }));
        assert!(!is_success_reason(&VerdictReason::Custom(
            "test".to_string()
        )));
    }

    #[test]
    fn test_merge_reasons_empty() {
        let reasons: Vec<VerdictReason> = vec![];
        let merged = merge_reasons(&reasons);
        assert_eq!(merged, VerdictReason::NoChanges);
    }

    #[test]
    fn test_merge_reasons_single() {
        let reasons = vec![VerdictReason::AddedWarnings { count: 5 }];
        let merged = merge_reasons(&reasons);
        assert_eq!(merged, VerdictReason::AddedWarnings { count: 5 });
    }

    #[test]
    fn test_merge_reasons_added_takes_priority() {
        let reasons = vec![
            VerdictReason::RemovedErrors { count: 10 },
            VerdictReason::AddedWarnings { count: 2 },
        ];
        let merged = merge_reasons(&reasons);
        assert_eq!(merged, VerdictReason::AddedWarnings { count: 2 });
    }

    #[test]
    fn test_verdict_summary_new() {
        let summary = VerdictSummary::new(VerdictReason::NoChanges);
        assert_eq!(summary.reason, VerdictReason::NoChanges);
        assert!(summary.details.is_empty());
        assert!(summary.suggestion.is_none());
    }

    #[test]
    fn test_verdict_summary_with_detail() {
        let summary = VerdictSummary::new(VerdictReason::NoChanges).with_detail("Test detail");
        assert_eq!(summary.details.len(), 1);
        assert_eq!(summary.details[0], "Test detail");
    }

    #[test]
    fn test_verdict_summary_with_suggestion() {
        let summary =
            VerdictSummary::new(VerdictReason::NoChanges).with_suggestion("Test suggestion");
        assert_eq!(summary.suggestion, Some("Test suggestion".to_string()));
    }
}
