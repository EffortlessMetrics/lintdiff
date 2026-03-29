//! Comprehensive tests for lintdiff-budget.
//!
//! These tests cover all public API functions and edge cases.

use lintdiff_budget::{
    Budget, BudgetConfig, BudgetCounter, BudgetExceeded, BudgetTracker, RemainingBudget,
};
use lintdiff_config_types::FailOn;
use lintdiff_counts::SeverityCounts;
use lintdiff_counts::SeverityLevel;

// ============================================================================
// BudgetConfig Tests
// ============================================================================

mod budget_config_tests {
    use super::*;

    #[test]
    fn default_has_no_limits() {
        let config = BudgetConfig::default();
        assert!(config.max_hints.is_none());
        assert!(config.max_notes.is_none());
        assert!(config.max_warnings.is_none());
        assert!(config.max_errors.is_none());
        assert!(config.max_fatals.is_none());
        assert!(config.max_total.is_none());
        assert!(config.max_problems.is_none());
    }

    #[test]
    fn new_has_no_limits() {
        let config = BudgetConfig::new();
        assert!(!config.has_limits());
    }

    #[test]
    fn zero_tolerance_allows_none() {
        let config = BudgetConfig::zero_tolerance();
        assert_eq!(config.max_hints, Some(0));
        assert_eq!(config.max_notes, Some(0));
        assert_eq!(config.max_warnings, Some(0));
        assert_eq!(config.max_errors, Some(0));
        assert_eq!(config.max_fatals, Some(0));
        assert_eq!(config.max_total, Some(0));
        assert_eq!(config.max_problems, Some(0));
    }

    #[test]
    fn from_fail_on_never_is_unlimited() {
        let config = BudgetConfig::from_fail_on(FailOn::Never);
        assert!(!config.has_limits());
    }

    #[test]
    fn from_fail_on_error_limits_errors() {
        let config = BudgetConfig::from_fail_on(FailOn::Error);
        assert_eq!(config.max_errors, Some(0));
        assert_eq!(config.max_fatals, Some(0));
        assert!(config.max_warnings.is_none());
        assert!(config.max_hints.is_none());
    }

    #[test]
    fn from_fail_on_warning_limits_warnings_and_errors() {
        let config = BudgetConfig::from_fail_on(FailOn::Warning);
        assert_eq!(config.max_warnings, Some(0));
        assert_eq!(config.max_errors, Some(0));
        assert_eq!(config.max_fatals, Some(0));
        assert!(config.max_hints.is_none());
    }

    #[test]
    fn from_fail_on_any_is_zero_tolerance() {
        let config = BudgetConfig::from_fail_on(FailOn::Any);
        assert_eq!(config.max_hints, Some(0));
        assert_eq!(config.max_warnings, Some(0));
        assert_eq!(config.max_errors, Some(0));
    }

    #[test]
    fn with_max_warnings_sets_limit() {
        let config = BudgetConfig::new().with_max_warnings(10);
        assert_eq!(config.max_warnings, Some(10));
    }

    #[test]
    fn with_max_errors_sets_limit() {
        let config = BudgetConfig::new().with_max_errors(5);
        assert_eq!(config.max_errors, Some(5));
    }

    #[test]
    fn with_max_total_sets_limit() {
        let config = BudgetConfig::new().with_max_total(100);
        assert_eq!(config.max_total, Some(100));
    }

    #[test]
    fn builder_chaining() {
        let config = BudgetConfig::new()
            .with_max_warnings(10)
            .with_max_errors(5)
            .with_max_total(50);

        assert_eq!(config.max_warnings, Some(10));
        assert_eq!(config.max_errors, Some(5));
        assert_eq!(config.max_total, Some(50));
    }

    #[test]
    fn has_limits_true_when_any_set() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            ..Default::default()
        };
        assert!(config.has_limits());
    }

    #[test]
    fn has_limits_false_when_none_set() {
        let config = BudgetConfig::default();
        assert!(!config.has_limits());
    }
}

// ============================================================================
// Budget Tests
// ============================================================================

mod budget_tests {
    use super::*;

    #[test]
    fn unlimited_accepts_any_counts() {
        let budget = Budget::unlimited();
        let counts = SeverityCounts::from_values(1000, 1000, 1000, 1000, 1000);
        assert!(budget.check(&counts).is_ok());
    }

    #[test]
    fn check_within_limits() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            ..Default::default()
        };
        let budget = Budget::new(config);

        let counts = SeverityCounts::from_values(0, 0, 5, 0, 0);
        assert!(budget.check(&counts).is_ok());
    }

    #[test]
    fn check_at_limit() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            ..Default::default()
        };
        let budget = Budget::new(config);

        let counts = SeverityCounts::from_values(0, 0, 10, 0, 0);
        assert!(budget.check(&counts).is_ok());
    }

    #[test]
    fn check_over_limit() {
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
    fn check_zero_tolerance() {
        let budget = Budget::new(BudgetConfig::zero_tolerance());
        let counts = SeverityCounts::from_values(1, 0, 0, 0, 0);
        assert!(budget.check(&counts).is_err());
    }

    #[test]
    fn check_multiple_limits() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            max_errors: Some(5),
            ..Default::default()
        };
        let budget = Budget::new(config);

        // Within both
        let ok = SeverityCounts::from_values(0, 0, 5, 3, 0);
        assert!(budget.check(&ok).is_ok());

        // Over warnings
        let over_warnings = SeverityCounts::from_values(0, 0, 15, 3, 0);
        assert!(budget.check(&over_warnings).is_err());

        // Over errors
        let over_errors = SeverityCounts::from_values(0, 0, 5, 10, 0);
        assert!(budget.check(&over_errors).is_err());
    }

    #[test]
    fn check_total_limit() {
        let config = BudgetConfig {
            max_total: Some(20),
            ..Default::default()
        };
        let budget = Budget::new(config);

        let ok = SeverityCounts::from_values(5, 5, 5, 4, 1);
        assert!(budget.check(&ok).is_ok());

        let over = SeverityCounts::from_values(5, 5, 5, 5, 1);
        assert!(budget.check(&over).is_err());
    }

    #[test]
    fn check_problems_limit() {
        let config = BudgetConfig {
            max_problems: Some(10),
            ..Default::default()
        };
        let budget = Budget::new(config);

        // warnings + errors + fatals = 10
        let ok = SeverityCounts::from_values(100, 100, 5, 3, 2);
        assert!(budget.check(&ok).is_ok());

        // warnings + errors + fatals = 11
        let over = SeverityCounts::from_values(100, 100, 6, 3, 2);
        assert!(budget.check(&over).is_err());
    }

    #[test]
    fn would_exceed_returns_bool() {
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
    fn remaining_calculates_correctly() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            max_errors: Some(5),
            ..Default::default()
        };
        let budget = Budget::new(config);

        let counts = SeverityCounts::from_values(0, 0, 3, 2, 0);
        let remaining = budget.remaining(&counts);

        assert_eq!(remaining.warnings, Some(7));
        assert_eq!(remaining.errors, Some(3));
        assert!(remaining.hints.is_none());
        assert!(remaining.notes.is_none());
    }

    #[test]
    fn remaining_saturates_at_zero() {
        let config = BudgetConfig {
            max_warnings: Some(5),
            ..Default::default()
        };
        let budget = Budget::new(config);

        let counts = SeverityCounts::from_values(0, 0, 10, 0, 0);
        let remaining = budget.remaining(&counts);

        assert_eq!(remaining.warnings, Some(0));
    }

    #[test]
    fn default_is_unlimited() {
        let budget = Budget::default();
        let counts = SeverityCounts::from_values(1000, 1000, 1000, 1000, 1000);
        assert!(budget.check(&counts).is_ok());
    }
}

// ============================================================================
// RemainingBudget Tests
// ============================================================================

mod remaining_budget_tests {
    use super::*;

    #[test]
    fn is_exhausted_when_zero() {
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
    fn not_exhausted_when_positive() {
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
    fn not_exhausted_when_unlimited() {
        let remaining = RemainingBudget {
            hints: None,
            notes: None,
            warnings: None,
            errors: None,
            fatals: None,
            total: None,
            problems: None,
        };
        assert!(!remaining.is_exhausted());
    }

    #[test]
    fn is_unlimited_when_all_none() {
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
    fn not_unlimited_when_any_set() {
        let remaining = RemainingBudget {
            hints: None,
            notes: None,
            warnings: Some(10),
            errors: None,
            fatals: None,
            total: None,
            problems: None,
        };
        assert!(!remaining.is_unlimited());
    }
}

// ============================================================================
// BudgetExceeded Tests
// ============================================================================

mod budget_exceeded_tests {
    use super::*;

    #[test]
    fn display_format() {
        let err = BudgetExceeded {
            category: "warnings",
            current: 15,
            limit: 10,
        };
        assert_eq!(format!("{}", err), "Budget exceeded for warnings: 15 > 10");
    }

    #[test]
    fn fields_accessible() {
        let err = BudgetExceeded {
            category: "errors",
            current: 5,
            limit: 0,
        };
        assert_eq!(err.category, "errors");
        assert_eq!(err.current, 5);
        assert_eq!(err.limit, 0);
    }
}

// ============================================================================
// BudgetTracker Tests
// ============================================================================

mod budget_tracker_tests {
    use super::*;

    #[test]
    fn new_creates_zero_counts() {
        let tracker = BudgetTracker::new(BudgetConfig::new());
        assert!(tracker.counts().is_empty());
    }

    #[test]
    fn unlimited_creates_zero_counts() {
        let tracker = BudgetTracker::unlimited();
        assert!(tracker.counts().is_empty());
    }

    #[test]
    fn increment_increases_count() {
        let mut tracker = BudgetTracker::unlimited();
        tracker.increment(SeverityLevel::Warning);
        assert_eq!(tracker.counts().warnings, 1);
    }

    #[test]
    fn increment_multiple_severities() {
        let mut tracker = BudgetTracker::unlimited();
        tracker.increment(SeverityLevel::Hint);
        tracker.increment(SeverityLevel::Note);
        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Error);
        tracker.increment(SeverityLevel::Fatal);

        assert_eq!(tracker.counts().hints, 1);
        assert_eq!(tracker.counts().notes, 1);
        assert_eq!(tracker.counts().warnings, 1);
        assert_eq!(tracker.counts().errors, 1);
        assert_eq!(tracker.counts().fatals, 1);
    }

    #[test]
    fn add_increases_counts() {
        let mut tracker = BudgetTracker::unlimited();
        let counts = SeverityCounts::from_values(1, 2, 3, 4, 5);
        tracker.add(&counts);

        assert_eq!(tracker.counts().hints, 1);
        assert_eq!(tracker.counts().notes, 2);
        assert_eq!(tracker.counts().warnings, 3);
        assert_eq!(tracker.counts().errors, 4);
        assert_eq!(tracker.counts().fatals, 5);
    }

    #[test]
    fn is_exceeded_within_limit() {
        let config = BudgetConfig {
            max_warnings: Some(5),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Warning);
        assert!(!tracker.is_exceeded());
    }

    #[test]
    fn is_exceeded_over_limit() {
        let config = BudgetConfig {
            max_warnings: Some(2),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Warning);
        assert!(tracker.is_exceeded());
    }

    #[test]
    fn remaining_warnings() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        assert_eq!(tracker.remaining_warnings(), Some(10));

        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Warning);
        assert_eq!(tracker.remaining_warnings(), Some(8));
    }

    #[test]
    fn remaining_errors() {
        let config = BudgetConfig {
            max_errors: Some(5),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        tracker.increment(SeverityLevel::Error);
        assert_eq!(tracker.remaining_errors(), Some(4));
    }

    #[test]
    fn remaining_total() {
        let config = BudgetConfig {
            max_total: Some(20),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Error);
        assert_eq!(tracker.remaining_total(), Some(18));
    }

    #[test]
    fn remaining_problems() {
        let config = BudgetConfig {
            max_problems: Some(10),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Error);
        tracker.increment(SeverityLevel::Hint); // Not a problem
        assert_eq!(tracker.remaining_problems(), Some(8));
    }

    #[test]
    fn remaining_unlimited() {
        let tracker = BudgetTracker::unlimited();
        assert!(tracker.remaining_warnings().is_none());
        assert!(tracker.remaining_errors().is_none());
    }

    #[test]
    fn would_exceed_with() {
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

    #[test]
    fn reset_clears_counts() {
        let mut tracker = BudgetTracker::unlimited();
        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Error);

        tracker.reset();

        assert!(tracker.counts().is_empty());
    }

    #[test]
    fn check_returns_ok_within_limit() {
        let config = BudgetConfig {
            max_warnings: Some(5),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        tracker.increment(SeverityLevel::Warning);
        assert!(tracker.check().is_ok());
    }

    #[test]
    fn check_returns_err_over_limit() {
        let config = BudgetConfig {
            max_warnings: Some(1),
            ..Default::default()
        };
        let mut tracker = BudgetTracker::new(config);

        tracker.increment(SeverityLevel::Warning);
        tracker.increment(SeverityLevel::Warning);
        assert!(tracker.check().is_err());
    }

    #[test]
    fn default_is_unlimited() {
        let tracker = BudgetTracker::default();
        assert!(tracker.config().max_warnings.is_none());
    }
}

// ============================================================================
// BudgetCounter Tests
// ============================================================================

mod budget_counter_tests {
    use super::*;

    #[test]
    fn tracking_only_never_rejects() {
        let config = BudgetConfig {
            max_warnings: Some(2),
            ..Default::default()
        };
        let mut counter = BudgetCounter::tracking_only(config);

        assert!(counter.try_increment(SeverityLevel::Warning).is_ok());
        assert!(counter.try_increment(SeverityLevel::Warning).is_ok());
        assert!(counter.try_increment(SeverityLevel::Warning).is_ok());

        assert_eq!(counter.counts().warnings, 3);
        assert!(counter.is_exceeded());
    }

    #[test]
    fn enforcing_rejects_over_limit() {
        let config = BudgetConfig {
            max_warnings: Some(2),
            ..Default::default()
        };
        let mut counter = BudgetCounter::enforcing(config);

        assert!(counter.try_increment(SeverityLevel::Warning).is_ok());
        assert!(counter.try_increment(SeverityLevel::Warning).is_ok());
        assert!(counter.try_increment(SeverityLevel::Warning).is_err());

        assert_eq!(counter.counts().warnings, 2);
    }

    #[test]
    fn force_increment_bypasses_limit() {
        let config = BudgetConfig {
            max_warnings: Some(2),
            ..Default::default()
        };
        let mut counter = BudgetCounter::enforcing(config);

        counter.force_increment(SeverityLevel::Warning);
        counter.force_increment(SeverityLevel::Warning);
        counter.force_increment(SeverityLevel::Warning);

        assert_eq!(counter.counts().warnings, 3);
        assert!(counter.is_exceeded());
    }

    #[test]
    fn is_enforcing() {
        let tracking = BudgetCounter::tracking_only(BudgetConfig::new());
        assert!(!tracking.is_enforcing());

        let enforcing = BudgetCounter::enforcing(BudgetConfig::new());
        assert!(enforcing.is_enforcing());
    }

    #[test]
    fn reset_clears_counts() {
        let mut counter = BudgetCounter::enforcing(BudgetConfig::new());
        counter.force_increment(SeverityLevel::Warning);
        counter.reset();
        assert!(counter.counts().is_empty());
    }

    #[test]
    fn remaining() {
        let config = BudgetConfig {
            max_warnings: Some(10),
            ..Default::default()
        };
        let mut counter = BudgetCounter::enforcing(config);

        counter.force_increment(SeverityLevel::Warning);
        counter.force_increment(SeverityLevel::Warning);

        let remaining = counter.remaining();
        assert_eq!(remaining.warnings, Some(8));
    }
}

// ============================================================================
// Property-based Tests
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_counts()(hints in 0u64..100, notes in 0u64..100, warnings in 0u64..100, errors in 0u64..100, fatals in 0u64..100) -> SeverityCounts {
            SeverityCounts::from_values(hints, notes, warnings, errors, fatals)
        }
    }

    prop_compose! {
        fn arb_budget_config()(
        max_hints in proptest::option::of(0u64..1000u64),
        max_notes in proptest::option::of(0u64..1000u64),
        max_warnings in proptest::option::of(0u64..1000u64),
        max_errors in proptest::option::of(0u64..1000u64),
        max_fatals in proptest::option::of(0u64..1000u64),
        max_total in proptest::option::of(0u64..5000u64),
        max_problems in proptest::option::of(0u64..3000u64),
    ) -> BudgetConfig {
        BudgetConfig {
            max_hints,
            max_notes,
            max_warnings,
            max_errors,
            max_fatals,
            max_total,
            max_problems,
        }
    }
    }

    proptest! {
        #[test]
        fn unlimited_budget_never_exceeds(counts in arb_counts()) {
            let budget = Budget::unlimited();
            prop_assert!(budget.check(&counts).is_ok());
        }

        #[test]
        fn remaining_never_negative(counts in arb_counts(), config in arb_budget_config()) {
            let budget = Budget::new(config);
            let remaining = budget.remaining(&counts);

            if let Some(r) = remaining.hints {
                prop_assert!(r <= budget.config().max_hints.unwrap_or(u64::MAX));
            }
            if let Some(r) = remaining.warnings {
                prop_assert!(r <= budget.config().max_warnings.unwrap_or(u64::MAX));
            }
        }

        #[test]
        fn tracker_remaining_consistent(config in arb_budget_config()) {
            let mut tracker = BudgetTracker::new(config.clone());
            let budget = Budget::new(config);

            // After any increments, remaining should be consistent
            for _ in 0..10 {
                tracker.increment(SeverityLevel::Warning);
            }

            let tracker_remaining = tracker.remaining_warnings();
            let budget_remaining = budget.remaining(tracker.counts()).warnings;
            prop_assert_eq!(tracker_remaining, budget_remaining);
        }

        #[test]
        fn zero_tolerance_rejects_any_positive(counts in arb_counts()) {
            let budget = Budget::new(BudgetConfig::zero_tolerance());
            let is_ok = budget.check(&counts).is_ok();
            let is_zero = counts.total() == 0;
            prop_assert_eq!(is_ok, is_zero);
        }
    }
}
