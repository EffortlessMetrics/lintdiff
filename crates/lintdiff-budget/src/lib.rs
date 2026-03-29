//! Budget management for diagnostic counts in lintdiff.
//!
//! This microcrate provides budget tracking and enforcement for diagnostic
//! counts, allowing you to set limits on warnings, errors, and other
//! diagnostic types.
//!
//! # Example: Basic Budget
//!
//! ```
//! use lintdiff_budget::{Budget, BudgetConfig};
//! use lintdiff_counts::SeverityCounts;
//!
//! let config = BudgetConfig {
//!     max_warnings: Some(10),
//!     max_errors: Some(0),
//!     max_total: Some(50),
//!     ..Default::default()
//! };
//!
//! let mut budget = Budget::new(config);
//!
//! let counts = SeverityCounts::from_values(5, 10, 3, 0, 0);
//! assert!(budget.check(&counts).is_ok());
//!
//! let over_budget = SeverityCounts::from_values(5, 10, 15, 0, 0);
//! assert!(budget.check(&over_budget).is_err());
//! ```
//!
//! # Example: Tracking Budget Usage
//!
//! ```
//! use lintdiff_budget::{BudgetTracker, BudgetConfig};
//! use lintdiff_counts::{SeverityCounts, SeverityLevel};
//!
//! let config = BudgetConfig {
//!     max_warnings: Some(5),
//!     ..Default::default()
//! };
//!
//! let mut tracker = BudgetTracker::new(config);
//!
//! tracker.increment(SeverityLevel::Warning);
//! tracker.increment(SeverityLevel::Warning);
//!
//! assert_eq!(tracker.remaining_warnings(), Some(3));
//! assert!(!tracker.is_exceeded());
//! ```

#![warn(missing_docs)]

use lintdiff_config_types::FailOn;
use lintdiff_counts::{SeverityCounts, SeverityLevel};
use std::fmt;

/// Configuration for budget limits.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BudgetConfig {
    /// Maximum number of hints allowed. None means unlimited.
    pub max_hints: Option<u64>,
    /// Maximum number of notes allowed. None means unlimited.
    pub max_notes: Option<u64>,
    /// Maximum number of warnings allowed. None means unlimited.
    pub max_warnings: Option<u64>,
    /// Maximum number of errors allowed. None means unlimited.
    pub max_errors: Option<u64>,
    /// Maximum number of fatal errors allowed. None means unlimited.
    pub max_fatals: Option<u64>,
    /// Maximum total diagnostics allowed. None means unlimited.
    pub max_total: Option<u64>,
    /// Maximum problems (warnings + errors + fatals) allowed. None means unlimited.
    pub max_problems: Option<u64>,
}

impl BudgetConfig {
    /// Create a new budget config with no limits.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_hints: None,
            max_notes: None,
            max_warnings: None,
            max_errors: None,
            max_fatals: None,
            max_total: None,
            max_problems: None,
        }
    }

    /// Create a config that allows no diagnostics at all.
    #[must_use]
    pub const fn zero_tolerance() -> Self {
        Self {
            max_hints: Some(0),
            max_notes: Some(0),
            max_warnings: Some(0),
            max_errors: Some(0),
            max_fatals: Some(0),
            max_total: Some(0),
            max_problems: Some(0),
        }
    }

    /// Create a config from a `FailOn` setting.
    #[must_use]
    pub const fn from_fail_on(fail_on: FailOn) -> Self {
        match fail_on {
            FailOn::Never => Self::new(),
            FailOn::Error => Self {
                max_errors: Some(0),
                max_fatals: Some(0),
                ..Self::new()
            },
            FailOn::Warning => Self {
                max_warnings: Some(0),
                max_errors: Some(0),
                max_fatals: Some(0),
                ..Self::new()
            },
            FailOn::Any => Self::zero_tolerance(),
        }
    }

    /// Set the maximum warnings.
    #[must_use]
    pub const fn with_max_warnings(mut self, max: u64) -> Self {
        self.max_warnings = Some(max);
        self
    }

    /// Set the maximum errors.
    #[must_use]
    pub const fn with_max_errors(mut self, max: u64) -> Self {
        self.max_errors = Some(max);
        self
    }

    /// Set the maximum total.
    #[must_use]
    pub const fn with_max_total(mut self, max: u64) -> Self {
        self.max_total = Some(max);
        self
    }

    /// Check if this config has any limits set.
    #[must_use]
    pub const fn has_limits(&self) -> bool {
        self.max_hints.is_some()
            || self.max_notes.is_some()
            || self.max_warnings.is_some()
            || self.max_errors.is_some()
            || self.max_fatals.is_some()
            || self.max_total.is_some()
            || self.max_problems.is_some()
    }
}

/// Budget checker for validating counts against limits.
#[derive(Debug, Clone)]
pub struct Budget {
    config: BudgetConfig,
}

impl Budget {
    /// Create a new budget with the given configuration.
    #[must_use]
    pub const fn new(config: BudgetConfig) -> Self {
        Self { config }
    }

    /// Create an unlimited budget.
    #[must_use]
    pub const fn unlimited() -> Self {
        Self::new(BudgetConfig::new())
    }

    /// Get the configuration.
    #[must_use]
    pub const fn config(&self) -> &BudgetConfig {
        &self.config
    }

    /// Check if counts are within budget.
    ///
    /// # Errors
    ///
    /// Returns `BudgetExceeded` if any limit is exceeded.
    pub fn check(&self, counts: &SeverityCounts) -> Result<(), BudgetExceeded> {
        // Check individual limits
        Self::check_limit("hints", counts.hints, self.config.max_hints)?;
        Self::check_limit("notes", counts.notes, self.config.max_notes)?;
        Self::check_limit("warnings", counts.warnings, self.config.max_warnings)?;
        Self::check_limit("errors", counts.errors, self.config.max_errors)?;
        Self::check_limit("fatals", counts.fatals, self.config.max_fatals)?;

        // Check combined limits
        Self::check_limit("total", counts.total(), self.config.max_total)?;
        Self::check_limit("problems", counts.problems(), self.config.max_problems)?;

        Ok(())
    }

    /// Check if counts would exceed budget.
    #[must_use]
    pub fn would_exceed(&self, counts: &SeverityCounts) -> bool {
        self.check(counts).is_err()
    }

    /// Get the remaining budget for each category.
    #[must_use]
    pub fn remaining(&self, counts: &SeverityCounts) -> RemainingBudget {
        RemainingBudget {
            hints: Self::remaining_for(counts.hints, self.config.max_hints),
            notes: Self::remaining_for(counts.notes, self.config.max_notes),
            warnings: Self::remaining_for(counts.warnings, self.config.max_warnings),
            errors: Self::remaining_for(counts.errors, self.config.max_errors),
            fatals: Self::remaining_for(counts.fatals, self.config.max_fatals),
            total: Self::remaining_for(counts.total(), self.config.max_total),
            problems: Self::remaining_for(counts.problems(), self.config.max_problems),
        }
    }

    const fn check_limit(
        name: &'static str,
        current: u64,
        limit: Option<u64>,
    ) -> Result<(), BudgetExceeded> {
        if let Some(max) = limit {
            if current > max {
                return Err(BudgetExceeded {
                    category: name,
                    current,
                    limit: max,
                });
            }
        }
        Ok(())
    }

    fn remaining_for(current: u64, limit: Option<u64>) -> Option<u64> {
        limit.map(|max| max.saturating_sub(current))
    }
}

impl Default for Budget {
    fn default() -> Self {
        Self::unlimited()
    }
}

/// Information about remaining budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemainingBudget {
    /// Remaining hints (None = unlimited).
    pub hints: Option<u64>,
    /// Remaining notes (None = unlimited).
    pub notes: Option<u64>,
    /// Remaining warnings (None = unlimited).
    pub warnings: Option<u64>,
    /// Remaining errors (None = unlimited).
    pub errors: Option<u64>,
    /// Remaining fatals (None = unlimited).
    pub fatals: Option<u64>,
    /// Remaining total (None = unlimited).
    pub total: Option<u64>,
    /// Remaining problems (None = unlimited).
    pub problems: Option<u64>,
}

impl RemainingBudget {
    /// Check if any budget is exhausted (remaining = 0).
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.hints == Some(0)
            || self.notes == Some(0)
            || self.warnings == Some(0)
            || self.errors == Some(0)
            || self.fatals == Some(0)
            || self.total == Some(0)
            || self.problems == Some(0)
    }

    /// Check if all budgets are unlimited.
    #[must_use]
    pub const fn is_unlimited(&self) -> bool {
        self.hints.is_none()
            && self.notes.is_none()
            && self.warnings.is_none()
            && self.errors.is_none()
            && self.fatals.is_none()
            && self.total.is_none()
            && self.problems.is_none()
    }
}

/// Error returned when budget is exceeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetExceeded {
    /// The category that exceeded the limit.
    pub category: &'static str,
    /// The current count.
    pub current: u64,
    /// The limit that was exceeded.
    pub limit: u64,
}

impl fmt::Display for BudgetExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Budget exceeded for {}: {} > {}",
            self.category, self.current, self.limit
        )
    }
}

impl std::error::Error for BudgetExceeded {}

/// Tracker for incrementally tracking budget usage.
#[derive(Debug, Clone)]
pub struct BudgetTracker {
    config: BudgetConfig,
    counts: SeverityCounts,
}

impl BudgetTracker {
    /// Create a new budget tracker.
    #[must_use]
    pub const fn new(config: BudgetConfig) -> Self {
        Self {
            config,
            counts: SeverityCounts::new(),
        }
    }

    /// Create an unlimited tracker.
    #[must_use]
    pub const fn unlimited() -> Self {
        Self::new(BudgetConfig::new())
    }

    /// Get the current counts.
    #[must_use]
    pub const fn counts(&self) -> &SeverityCounts {
        &self.counts
    }

    /// Get the configuration.
    #[must_use]
    pub const fn config(&self) -> &BudgetConfig {
        &self.config
    }

    /// Increment a severity count.
    pub const fn increment(&mut self, severity: SeverityLevel) {
        self.counts.increment(severity);
    }

    /// Add counts to the tracker.
    pub fn add(&mut self, counts: &SeverityCounts) {
        self.counts += counts.clone();
    }

    /// Check if the budget is exceeded.
    #[must_use]
    pub fn is_exceeded(&self) -> bool {
        let budget = Budget::new(self.config.clone());
        budget.would_exceed(&self.counts)
    }

    /// Check if adding counts would exceed the budget.
    #[must_use]
    pub fn would_exceed_with(&self, counts: &SeverityCounts) -> bool {
        let combined = self.counts.clone() + counts.clone();
        let budget = Budget::new(self.config.clone());
        budget.would_exceed(&combined)
    }

    /// Get remaining warnings.
    #[must_use]
    pub fn remaining_warnings(&self) -> Option<u64> {
        self.config
            .max_warnings
            .map(|max| max.saturating_sub(self.counts.warnings))
    }

    /// Get remaining errors.
    #[must_use]
    pub fn remaining_errors(&self) -> Option<u64> {
        self.config
            .max_errors
            .map(|max| max.saturating_sub(self.counts.errors))
    }

    /// Get remaining total.
    #[must_use]
    pub fn remaining_total(&self) -> Option<u64> {
        self.config
            .max_total
            .map(|max| max.saturating_sub(self.counts.total()))
    }

    /// Get remaining problems.
    #[must_use]
    pub fn remaining_problems(&self) -> Option<u64> {
        self.config
            .max_problems
            .map(|max| max.saturating_sub(self.counts.problems()))
    }

    /// Get the remaining budget.
    #[must_use]
    pub fn remaining(&self) -> RemainingBudget {
        let budget = Budget::new(self.config.clone());
        budget.remaining(&self.counts)
    }

    /// Reset the tracker.
    pub const fn reset(&mut self) {
        self.counts = SeverityCounts::new();
    }

    /// Check the current budget status.
    ///
    /// # Errors
    ///
    /// Returns `BudgetExceeded` if any limit is exceeded.
    pub fn check(&self) -> Result<(), BudgetExceeded> {
        let budget = Budget::new(self.config.clone());
        budget.check(&self.counts)
    }
}

impl Default for BudgetTracker {
    fn default() -> Self {
        Self::unlimited()
    }
}

/// Budget-aware counter that can optionally enforce limits.
#[derive(Debug, Clone)]
pub struct BudgetCounter {
    tracker: BudgetTracker,
    enforce: bool,
}

impl BudgetCounter {
    /// Create a new budget counter.
    #[must_use]
    pub const fn new(config: BudgetConfig, enforce: bool) -> Self {
        Self {
            tracker: BudgetTracker::new(config),
            enforce,
        }
    }

    /// Create a counter that tracks but doesn't enforce.
    #[must_use]
    pub const fn tracking_only(config: BudgetConfig) -> Self {
        Self::new(config, false)
    }

    /// Create a counter that enforces limits.
    #[must_use]
    pub const fn enforcing(config: BudgetConfig) -> Self {
        Self::new(config, true)
    }

    /// Get the current counts.
    #[must_use]
    pub const fn counts(&self) -> &SeverityCounts {
        self.tracker.counts()
    }

    /// Check if enforcement is enabled.
    #[must_use]
    pub const fn is_enforcing(&self) -> bool {
        self.enforce
    }

    /// Try to increment a severity count.
    ///
    /// # Errors
    ///
    /// Returns `BudgetExceeded` if enforcement is enabled and the increment would exceed budget.
    pub fn try_increment(&mut self, severity: SeverityLevel) -> Result<(), BudgetExceeded> {
        // Create a temporary count to check
        let mut temp = SeverityCounts::new();
        temp.increment(severity);

        if self.enforce && self.tracker.would_exceed_with(&temp) {
            // Check which limit would be exceeded
            let combined = self.tracker.counts().clone() + temp;
            let budget = Budget::new(self.tracker.config().clone());
            budget.check(&combined)?;
        }

        self.tracker.increment(severity);
        Ok(())
    }

    /// Increment a severity count, ignoring budget limits.
    pub const fn force_increment(&mut self, severity: SeverityLevel) {
        self.tracker.increment(severity);
    }

    /// Check if the budget is exceeded.
    #[must_use]
    pub fn is_exceeded(&self) -> bool {
        self.tracker.is_exceeded()
    }

    /// Get the remaining budget.
    #[must_use]
    pub fn remaining(&self) -> RemainingBudget {
        self.tracker.remaining()
    }

    /// Reset the counter.
    pub const fn reset(&mut self) {
        self.tracker.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_config_default() {
        let config = BudgetConfig::default();
        assert!(config.max_warnings.is_none());
        assert!(config.max_errors.is_none());
    }

    #[test]
    fn test_budget_config_zero_tolerance() {
        let config = BudgetConfig::zero_tolerance();
        assert_eq!(config.max_warnings, Some(0));
        assert_eq!(config.max_errors, Some(0));
    }

    #[test]
    fn test_budget_config_from_fail_on_never() {
        let config = BudgetConfig::from_fail_on(FailOn::Never);
        assert!(!config.has_limits());
    }

    #[test]
    fn test_budget_config_from_fail_on_error() {
        let config = BudgetConfig::from_fail_on(FailOn::Error);
        assert_eq!(config.max_errors, Some(0));
        assert_eq!(config.max_fatals, Some(0));
        assert!(config.max_warnings.is_none());
    }

    #[test]
    fn test_budget_config_from_fail_on_warning() {
        let config = BudgetConfig::from_fail_on(FailOn::Warning);
        assert_eq!(config.max_warnings, Some(0));
        assert_eq!(config.max_errors, Some(0));
    }

    #[test]
    fn test_budget_config_from_fail_on_any() {
        let config = BudgetConfig::from_fail_on(FailOn::Any);
        assert_eq!(config.max_hints, Some(0));
        assert_eq!(config.max_warnings, Some(0));
    }

    #[test]
    fn test_budget_check_within_limits() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            ..Default::default()
        };
        let budget = Budget::new(config);

        let counts = SeverityCounts::from_values(0, 0, 5, 0, 0);
        assert!(budget.check(&counts).is_ok());
    }

    #[test]
    fn test_budget_check_at_limit() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            ..Default::default()
        };
        let budget = Budget::new(config);

        let counts = SeverityCounts::from_values(0, 0, 10, 0, 0);
        assert!(budget.check(&counts).is_ok());
    }

    #[test]
    fn test_budget_check_over_limit() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            ..Default::default()
        };
        let budget = Budget::new(config);

        let counts = SeverityCounts::from_values(0, 0, 15, 0, 0);
        let result = budget.check(&counts);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.category, "warnings");
        assert_eq!(err.current, 15);
        assert_eq!(err.limit, 10);
    }

    #[test]
    fn test_budget_check_total_limit() {
        let config = BudgetConfig {
            max_total: Some(30),
            ..Default::default()
        };
        let budget = Budget::new(config);

        let counts = SeverityCounts::from_values(5, 5, 5, 5, 5);
        assert!(budget.check(&counts).is_ok());

        let over = SeverityCounts::from_values(5, 5, 5, 5, 5);
        let combined = counts.clone() + over;
        assert!(budget.check(&combined).is_err());
    }

    #[test]
    fn test_budget_unlimited() {
        let budget = Budget::unlimited();
        let counts = SeverityCounts::from_values(1000, 1000, 1000, 1000, 1000);
        assert!(budget.check(&counts).is_ok());
    }

    #[test]
    fn test_budget_remaining() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            max_errors: Some(0),
            ..Default::default()
        };
        let budget = Budget::new(config);

        let counts = SeverityCounts::from_values(0, 0, 3, 0, 0);
        let remaining = budget.remaining(&counts);

        assert_eq!(remaining.warnings, Some(7));
        assert_eq!(remaining.errors, Some(0));
        assert!(remaining.hints.is_none());
    }

    #[test]
    fn test_remaining_budget_exhausted() {
        let remaining = RemainingBudget {
            hints: None,
            notes: None,
            warnings: Some(0),
            errors: None,
            fatals: None,
            total: None,
            problems: None,
        };
        assert!(remaining.is_exhausted());
    }

    #[test]
    fn test_remaining_budget_not_exhausted() {
        let remaining = RemainingBudget {
            hints: None,
            notes: None,
            warnings: Some(5),
            errors: None,
            fatals: None,
            total: None,
            problems: None,
        };
        assert!(!remaining.is_exhausted());
    }

    #[test]
    fn test_remaining_budget_unlimited() {
        let remaining = RemainingBudget {
            hints: None,
            notes: None,
            warnings: None,
            errors: None,
            fatals: None,
            total: None,
            problems: None,
        };
        assert!(remaining.is_unlimited());
    }

    #[test]
    fn test_budget_tracker_basic() {
        let config = BudgetConfig {
            max_warnings: Some(5),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Warning);

        assert_eq!(tracker.remaining_warnings(), Some(3));
        assert!(!tracker.is_exceeded());
    }

    #[test]
    fn test_budget_tracker_exceeded() {
        let config = BudgetConfig {
            max_warnings: Some(2),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Warning);
        assert!(!tracker.is_exceeded());

        tracker.increment(SeverityLevel::Warning);
        assert!(tracker.is_exceeded());
    }

    #[test]
    fn test_budget_tracker_reset() {
        let config = BudgetConfig {
            max_warnings: Some(5),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Warning);
        assert_eq!(tracker.counts().warnings, 2);

        tracker.reset();
        assert_eq!(tracker.counts().warnings, 0);
    }

    #[test]
    fn test_budget_tracker_add() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        let counts = SeverityCounts::from_values(0, 0, 5, 0, 0);
        tracker.add(&counts);

        assert_eq!(tracker.counts().warnings, 5);
        assert_eq!(tracker.remaining_warnings(), Some(5));
    }

    #[test]
    fn test_budget_counter_tracking_only() {
        let config = BudgetConfig {
            max_warnings: Some(2),
            ..Default::default()
        };
        let mut counter = BudgetCounter::tracking_only(config);

        assert!(counter.try_increment(SeverityLevel::Warning).is_ok());
        assert!(counter.try_increment(SeverityLevel::Warning).is_ok());
        assert!(counter.try_increment(SeverityLevel::Warning).is_ok()); // Still ok, just tracking

        assert!(counter.is_exceeded());
    }

    #[test]
    fn test_budget_counter_enforcing() {
        let config = BudgetConfig {
            max_warnings: Some(2),
            ..Default::default()
        };
        let mut counter = BudgetCounter::enforcing(config);

        assert!(counter.try_increment(SeverityLevel::Warning).is_ok());
        assert!(counter.try_increment(SeverityLevel::Warning).is_ok());
        assert!(counter.try_increment(SeverityLevel::Warning).is_err()); // Rejected

        assert_eq!(counter.counts().warnings, 2);
    }

    #[test]
    fn test_budget_counter_force_increment() {
        let config = BudgetConfig {
            max_warnings: Some(2),
            ..Default::default()
        };
        let mut counter = BudgetCounter::enforcing(config);

        counter.force_increment(SeverityLevel::Warning);
        counter.force_increment(SeverityLevel::Warning);
        counter.force_increment(SeverityLevel::Warning); // Force past limit

        assert_eq!(counter.counts().warnings, 3);
        assert!(counter.is_exceeded());
    }

    #[test]
    fn test_budget_exceeded_display() {
        let err = BudgetExceeded {
            category: "warnings",
            current: 15,
            limit: 10,
        };
        assert_eq!(err.to_string(), "Budget exceeded for warnings: 15 > 10");
    }

    #[test]
    fn test_budget_config_builder() {
        let config = BudgetConfig::new()
            .with_max_warnings(10)
            .with_max_errors(0)
            .with_max_total(50);

        assert_eq!(config.max_warnings, Some(10));
        assert_eq!(config.max_errors, Some(0));
        assert_eq!(config.max_total, Some(50));
    }

    #[test]
    fn test_budget_would_exceed() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            ..Default::default()
        };
        let budget = Budget::new(config);

        let ok = SeverityCounts::from_values(0, 0, 5, 0, 0);
        assert!(!budget.would_exceed(&ok));

        let over = SeverityCounts::from_values(0, 0, 15, 0, 0);
        assert!(budget.would_exceed(&over));
    }

    #[test]
    fn test_budget_tracker_would_exceed_with() {
        let config = BudgetConfig {
            max_warnings: Some(5),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Warning);

        let small = SeverityCounts::from_values(0, 0, 2, 0, 0);
        assert!(!tracker.would_exceed_with(&small));

        let large = SeverityCounts::from_values(0, 0, 5, 0, 0);
        assert!(tracker.would_exceed_with(&large));
    }
}
