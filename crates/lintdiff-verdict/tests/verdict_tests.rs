//! Comprehensive tests for lintdiff-verdict crate.
//!
//! Test coverage:
//! 1. Verdict enum methods (15 tests)
//! 2. Verdict::combine combinations (6 tests)
//! 3. FindingCounts methods (12 tests)
//! 4. FindingCounts::to_verdict with various thresholds (12 tests)
//! 5. VerdictReport builder methods (8 tests)
//! 6. VerdictThresholds presets and evaluation (7 tests)

use lintdiff_verdict::{FindingCounts, Verdict, VerdictReport, VerdictThresholds};

// =============================================================================
// 1. Verdict enum methods (15 tests)
// =============================================================================

mod verdict_methods {
    use super::*;

    #[test]
    fn verdict_pass_is_success() {
        assert!(Verdict::Pass.is_success());
    }

    #[test]
    fn verdict_warn_is_success() {
        assert!(Verdict::Warn.is_success());
    }

    #[test]
    fn verdict_fail_is_not_success() {
        assert!(!Verdict::Fail.is_success());
    }

    #[test]
    fn verdict_pass_is_not_failure() {
        assert!(!Verdict::Pass.is_failure());
    }

    #[test]
    fn verdict_warn_is_not_failure() {
        assert!(!Verdict::Warn.is_failure());
    }

    #[test]
    fn verdict_fail_is_failure() {
        assert!(Verdict::Fail.is_failure());
    }

    #[test]
    fn verdict_pass_is_not_warning() {
        assert!(!Verdict::Pass.is_warning());
    }

    #[test]
    fn verdict_warn_is_warning() {
        assert!(Verdict::Warn.is_warning());
    }

    #[test]
    fn verdict_fail_is_not_warning() {
        assert!(!Verdict::Fail.is_warning());
    }

    #[test]
    fn verdict_pass_exit_code() {
        assert_eq!(Verdict::Pass.exit_code(), 0);
    }

    #[test]
    fn verdict_warn_exit_code() {
        assert_eq!(Verdict::Warn.exit_code(), 0);
    }

    #[test]
    fn verdict_fail_exit_code() {
        assert_eq!(Verdict::Fail.exit_code(), 1);
    }

    #[test]
    fn verdict_as_str() {
        assert_eq!(Verdict::Pass.as_str(), "pass");
        assert_eq!(Verdict::Warn.as_str(), "warn");
        assert_eq!(Verdict::Fail.as_str(), "fail");
    }

    #[test]
    fn verdict_display() {
        assert_eq!(format!("{}", Verdict::Pass), "pass");
        assert_eq!(format!("{}", Verdict::Warn), "warn");
        assert_eq!(format!("{}", Verdict::Fail), "fail");
    }

    #[test]
    fn verdict_default() {
        assert_eq!(Verdict::default(), Verdict::Pass);
    }

    #[test]
    fn verdict_from_bool_true() {
        assert_eq!(Verdict::from_bool(true), Verdict::Pass);
    }

    #[test]
    fn verdict_from_bool_false() {
        assert_eq!(Verdict::from_bool(false), Verdict::Fail);
    }

    #[test]
    fn verdict_icon() {
        assert_eq!(Verdict::Pass.icon(), "✅");
        assert_eq!(Verdict::Warn.icon(), "⚠️");
        assert_eq!(Verdict::Fail.icon(), "❌");
    }
}

// =============================================================================
// 2. Verdict::combine combinations (6 tests)
// =============================================================================

mod verdict_combine {
    use super::*;

    #[test]
    fn combine_pass_with_pass() {
        assert_eq!(Verdict::Pass.combine(Verdict::Pass), Verdict::Pass);
    }

    #[test]
    fn combine_pass_with_warn() {
        assert_eq!(Verdict::Pass.combine(Verdict::Warn), Verdict::Warn);
    }

    #[test]
    fn combine_pass_with_fail() {
        assert_eq!(Verdict::Pass.combine(Verdict::Fail), Verdict::Fail);
    }

    #[test]
    fn combine_warn_with_pass() {
        assert_eq!(Verdict::Warn.combine(Verdict::Pass), Verdict::Warn);
    }

    #[test]
    fn combine_warn_with_warn() {
        assert_eq!(Verdict::Warn.combine(Verdict::Warn), Verdict::Warn);
    }

    #[test]
    fn combine_warn_with_fail() {
        assert_eq!(Verdict::Warn.combine(Verdict::Fail), Verdict::Fail);
    }

    #[test]
    fn combine_fail_with_pass() {
        assert_eq!(Verdict::Fail.combine(Verdict::Pass), Verdict::Fail);
    }

    #[test]
    fn combine_fail_with_warn() {
        assert_eq!(Verdict::Fail.combine(Verdict::Warn), Verdict::Fail);
    }

    #[test]
    fn combine_fail_with_fail() {
        assert_eq!(Verdict::Fail.combine(Verdict::Fail), Verdict::Fail);
    }

    #[test]
    fn combine_is_commutative() {
        assert_eq!(
            Verdict::Pass.combine(Verdict::Warn),
            Verdict::Warn.combine(Verdict::Pass)
        );
        assert_eq!(
            Verdict::Pass.combine(Verdict::Fail),
            Verdict::Fail.combine(Verdict::Pass)
        );
        assert_eq!(
            Verdict::Warn.combine(Verdict::Fail),
            Verdict::Fail.combine(Verdict::Warn)
        );
    }
}

// =============================================================================
// 3. FindingCounts methods (12 tests)
// =============================================================================

mod finding_counts {
    use super::*;

    #[test]
    fn finding_counts_new() {
        let counts = FindingCounts::new();
        assert_eq!(counts.new_errors, 0);
        assert_eq!(counts.new_warnings, 0);
        assert_eq!(counts.fixed, 0);
        assert_eq!(counts.pre_existing, 0);
    }

    #[test]
    fn finding_counts_default() {
        let counts = FindingCounts::default();
        assert_eq!(counts.new_errors, 0);
        assert_eq!(counts.new_warnings, 0);
        assert_eq!(counts.fixed, 0);
        assert_eq!(counts.pre_existing, 0);
    }

    #[test]
    fn finding_counts_from_counts() {
        let counts = FindingCounts::from_counts(5, 10, 3, 20);
        assert_eq!(counts.new_errors, 5);
        assert_eq!(counts.new_warnings, 10);
        assert_eq!(counts.fixed, 3);
        assert_eq!(counts.pre_existing, 20);
    }

    #[test]
    fn finding_counts_total_new_with_both() {
        let counts = FindingCounts::from_counts(5, 10, 0, 0);
        assert_eq!(counts.total_new(), 15);
    }

    #[test]
    fn finding_counts_total_new_with_only_errors() {
        let counts = FindingCounts::from_counts(5, 0, 0, 0);
        assert_eq!(counts.total_new(), 5);
    }

    #[test]
    fn finding_counts_total_new_with_only_warnings() {
        let counts = FindingCounts::from_counts(0, 10, 0, 0);
        assert_eq!(counts.total_new(), 10);
    }

    #[test]
    fn finding_counts_total_new_empty() {
        let counts = FindingCounts::new();
        assert_eq!(counts.total_new(), 0);
    }

    #[test]
    fn finding_counts_has_new_issues_true_with_errors() {
        let counts = FindingCounts::from_counts(1, 0, 0, 0);
        assert!(counts.has_new_issues());
    }

    #[test]
    fn finding_counts_has_new_issues_true_with_warnings() {
        let counts = FindingCounts::from_counts(0, 1, 0, 0);
        assert!(counts.has_new_issues());
    }

    #[test]
    fn finding_counts_has_new_issues_false() {
        let counts = FindingCounts::new();
        assert!(!counts.has_new_issues());
    }

    #[test]
    fn finding_counts_has_new_errors_true() {
        let counts = FindingCounts::from_counts(1, 0, 0, 0);
        assert!(counts.has_new_errors());
    }

    #[test]
    fn finding_counts_has_new_errors_false() {
        let counts = FindingCounts::from_counts(0, 10, 0, 0);
        assert!(!counts.has_new_errors());
    }

    #[test]
    fn finding_counts_equality() {
        let a = FindingCounts::from_counts(1, 2, 3, 4);
        let b = FindingCounts::from_counts(1, 2, 3, 4);
        let c = FindingCounts::from_counts(1, 2, 3, 5);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn finding_counts_clone() {
        let original = FindingCounts::from_counts(1, 2, 3, 4);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}

// =============================================================================
// 4. FindingCounts::to_verdict with various thresholds (12 tests)
// =============================================================================

mod finding_counts_to_verdict {
    use super::*;

    #[test]
    fn to_verdict_empty_counts_pass() {
        let counts = FindingCounts::new();
        assert_eq!(counts.to_verdict(true, None), Verdict::Pass);
    }

    #[test]
    fn to_verdict_empty_counts_with_max_warnings_pass() {
        let counts = FindingCounts::new();
        assert_eq!(counts.to_verdict(true, Some(0)), Verdict::Pass);
    }

    #[test]
    fn to_verdict_warnings_only_below_threshold_warn() {
        let counts = FindingCounts::from_counts(0, 5, 0, 0);
        assert_eq!(counts.to_verdict(true, Some(10)), Verdict::Warn);
    }

    #[test]
    fn to_verdict_warnings_only_at_threshold_warn() {
        let counts = FindingCounts::from_counts(0, 10, 0, 0);
        assert_eq!(counts.to_verdict(true, Some(10)), Verdict::Warn);
    }

    #[test]
    fn to_verdict_warnings_only_above_threshold_fail() {
        let counts = FindingCounts::from_counts(0, 5, 2, 10);
        assert_eq!(counts.to_verdict(true, Some(3)), Verdict::Fail);
    }

    #[test]
    fn to_verdict_errors_with_fail_on_error_fail() {
        let counts = FindingCounts::from_counts(1, 0, 0, 0);
        assert_eq!(counts.to_verdict(true, None), Verdict::Fail);
    }

    #[test]
    fn to_verdict_errors_without_fail_on_error_warn() {
        let counts = FindingCounts::from_counts(1, 0, 0, 0);
        assert_eq!(counts.to_verdict(false, None), Verdict::Warn);
    }

    #[test]
    fn to_verdict_errors_with_zero_max_warnings_fail() {
        let counts = FindingCounts::from_counts(1, 0, 0, 0);
        assert_eq!(counts.to_verdict(true, Some(0)), Verdict::Fail);
    }

    #[test]
    fn to_verdict_warnings_unlimited() {
        let counts = FindingCounts::from_counts(0, 100, 0, 0);
        assert_eq!(counts.to_verdict(true, None), Verdict::Warn);
    }

    #[test]
    fn to_verdict_mixed_errors_and_warnings_fail() {
        let counts = FindingCounts::from_counts(1, 5, 0, 0);
        assert_eq!(counts.to_verdict(true, Some(10)), Verdict::Fail);
    }

    #[test]
    fn to_verdict_zero_max_warnings_with_no_warnings_pass() {
        let counts = FindingCounts::from_counts(0, 0, 5, 10);
        assert_eq!(counts.to_verdict(true, Some(0)), Verdict::Pass);
    }

    #[test]
    fn to_verdict_zero_max_warnings_with_warnings_fail() {
        let counts = FindingCounts::from_counts(0, 1, 5, 10);
        assert_eq!(counts.to_verdict(true, Some(0)), Verdict::Fail);
    }

    #[test]
    fn to_verdict_fixed_only_pass() {
        let counts = FindingCounts::from_counts(0, 0, 10, 5);
        assert_eq!(counts.to_verdict(true, Some(0)), Verdict::Pass);
    }

    #[test]
    fn to_verdict_pre_existing_only_pass() {
        let counts = FindingCounts::from_counts(0, 0, 0, 100);
        assert_eq!(counts.to_verdict(true, Some(0)), Verdict::Pass);
    }
}

// =============================================================================
// 5. VerdictReport builder methods (8 tests)
// =============================================================================

mod verdict_report {
    use super::*;

    #[test]
    fn verdict_report_new() {
        let counts = FindingCounts::from_counts(1, 2, 3, 4);
        let report = VerdictReport::new(Verdict::Pass, counts.clone());
        assert_eq!(report.verdict, Verdict::Pass);
        assert_eq!(report.counts, counts);
        assert!(report.message.is_none());
    }

    #[test]
    fn verdict_report_with_message() {
        let counts = FindingCounts::new();
        let report = VerdictReport::new(Verdict::Pass, counts).with_message("All checks passed");
        assert_eq!(report.message, Some("All checks passed".to_string()));
    }

    #[test]
    fn verdict_report_pass() {
        let counts = FindingCounts::new();
        let report = VerdictReport::pass(counts);
        assert_eq!(report.verdict, Verdict::Pass);
        assert!(report.is_pass());
    }

    #[test]
    fn verdict_report_warn() {
        let counts = FindingCounts::from_counts(0, 5, 0, 0);
        let report = VerdictReport::warn(counts);
        assert_eq!(report.verdict, Verdict::Warn);
        assert!(!report.is_pass());
    }

    #[test]
    fn verdict_report_fail() {
        let counts = FindingCounts::from_counts(1, 0, 0, 0);
        let report = VerdictReport::fail(counts);
        assert_eq!(report.verdict, Verdict::Fail);
        assert!(!report.is_pass());
    }

    #[test]
    fn verdict_report_is_pass_true_only_for_pass() {
        let counts = FindingCounts::new();
        assert!(VerdictReport::pass(counts.clone()).is_pass());
        assert!(!VerdictReport::warn(counts.clone()).is_pass());
        assert!(!VerdictReport::fail(counts).is_pass());
    }

    #[test]
    fn verdict_report_into_verdict() {
        let counts = FindingCounts::from_counts(1, 2, 3, 4);
        let report = VerdictReport::warn(counts);
        let verdict: Verdict = report.into();
        assert_eq!(verdict, Verdict::Warn);
    }

    #[test]
    fn verdict_report_with_message_builder_pattern() {
        let report = VerdictReport::pass(FindingCounts::new())
            .with_message("Everything looks good!")
            .with_message("Updated message");
        assert_eq!(report.message, Some("Updated message".to_string()));
    }

    #[test]
    fn verdict_report_clone() {
        let report = VerdictReport::pass(FindingCounts::new()).with_message("Test");
        let cloned = report.clone();
        assert_eq!(report.verdict, cloned.verdict);
        assert_eq!(report.counts, cloned.counts);
        assert_eq!(report.message, cloned.message);
    }
}

// =============================================================================
// 6. VerdictThresholds presets and evaluation (7 tests)
// =============================================================================

mod verdict_thresholds {
    use super::*;

    #[test]
    fn verdict_thresholds_new() {
        let thresholds = VerdictThresholds::new(true, Some(10));
        assert!(thresholds.fail_on_error);
        assert_eq!(thresholds.max_warnings, Some(10));
    }

    #[test]
    fn verdict_thresholds_strict() {
        let thresholds = VerdictThresholds::strict();
        assert!(thresholds.fail_on_error);
        assert_eq!(thresholds.max_warnings, Some(0));
    }

    #[test]
    fn verdict_thresholds_fail_on_errors_only() {
        let thresholds = VerdictThresholds::fail_on_errors_only();
        assert!(thresholds.fail_on_error);
        assert_eq!(thresholds.max_warnings, None);
    }

    #[test]
    fn verdict_thresholds_lenient() {
        let thresholds = VerdictThresholds::lenient();
        assert!(!thresholds.fail_on_error);
        assert_eq!(thresholds.max_warnings, None);
    }

    #[test]
    fn verdict_thresholds_default() {
        let thresholds = VerdictThresholds::default();
        assert_eq!(thresholds, VerdictThresholds::fail_on_errors_only());
    }

    #[test]
    fn verdict_thresholds_evaluate_strict() {
        let thresholds = VerdictThresholds::strict();

        // No issues -> Pass
        let clean = FindingCounts::new();
        assert_eq!(thresholds.evaluate(&clean), Verdict::Pass);

        // Any warning -> Fail (max_warnings = 0)
        let with_warning = FindingCounts::from_counts(0, 1, 0, 0);
        assert_eq!(thresholds.evaluate(&with_warning), Verdict::Fail);

        // Any error -> Fail
        let with_error = FindingCounts::from_counts(1, 0, 0, 0);
        assert_eq!(thresholds.evaluate(&with_error), Verdict::Fail);
    }

    #[test]
    fn verdict_thresholds_evaluate_fail_on_errors_only() {
        let thresholds = VerdictThresholds::fail_on_errors_only();

        // No issues -> Pass
        let clean = FindingCounts::new();
        assert_eq!(thresholds.evaluate(&clean), Verdict::Pass);

        // Warnings allowed -> Warn
        let with_warning = FindingCounts::from_counts(0, 100, 0, 0);
        assert_eq!(thresholds.evaluate(&with_warning), Verdict::Warn);

        // Errors fail -> Fail
        let with_error = FindingCounts::from_counts(1, 0, 0, 0);
        assert_eq!(thresholds.evaluate(&with_error), Verdict::Fail);
    }

    #[test]
    fn verdict_thresholds_evaluate_lenient() {
        let thresholds = VerdictThresholds::lenient();

        // No issues -> Pass
        let clean = FindingCounts::new();
        assert_eq!(thresholds.evaluate(&clean), Verdict::Pass);

        // Warnings -> Warn (not fail)
        let with_warning = FindingCounts::from_counts(0, 100, 0, 0);
        assert_eq!(thresholds.evaluate(&with_warning), Verdict::Warn);

        // Even errors -> Warn (not fail)
        let with_error = FindingCounts::from_counts(10, 0, 0, 0);
        assert_eq!(thresholds.evaluate(&with_error), Verdict::Warn);
    }

    #[test]
    fn verdict_thresholds_evaluate_custom() {
        let thresholds = VerdictThresholds::new(true, Some(5));

        // Exactly at limit -> Warn
        let at_limit = FindingCounts::from_counts(0, 5, 0, 0);
        assert_eq!(thresholds.evaluate(&at_limit), Verdict::Warn);

        // Over limit -> Fail
        let over_limit = FindingCounts::from_counts(0, 6, 0, 0);
        assert_eq!(thresholds.evaluate(&over_limit), Verdict::Fail);

        // Error always fails when fail_on_error is true
        let with_error = FindingCounts::from_counts(1, 0, 0, 0);
        assert_eq!(thresholds.evaluate(&with_error), Verdict::Fail);
    }

    #[test]
    fn verdict_thresholds_clone() {
        let original = VerdictThresholds::strict();
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn verdict_thresholds_equality() {
        let a = VerdictThresholds::new(true, Some(5));
        let b = VerdictThresholds::new(true, Some(5));
        let c = VerdictThresholds::new(false, Some(5));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}

// =============================================================================
// Additional edge case tests
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn combine_multiple_verdicts() {
        let verdicts = [Verdict::Pass, Verdict::Warn, Verdict::Pass, Verdict::Fail];
        let combined = verdicts.iter().fold(Verdict::Pass, |acc, &v| acc.combine(v));
        assert_eq!(combined, Verdict::Fail);
    }

    #[test]
    fn combine_empty_iterator() {
        let verdicts: [Verdict; 0] = [];
        let combined = verdicts.iter().fold(Verdict::Pass, |acc, &v| acc.combine(v));
        assert_eq!(combined, Verdict::Pass);
    }

    #[test]
    fn finding_counts_large_values() {
        // Use large but non-overflowing values for total_new
        let counts = FindingCounts::from_counts(1_000_000, 2_000_000, 3_000_000, 4_000_000);
        assert_eq!(counts.total_new(), 3_000_000);
        assert!(counts.has_new_issues());
        assert!(counts.has_new_errors());
    }

    #[test]
    fn verdict_report_message_can_be_empty() {
        let report = VerdictReport::pass(FindingCounts::new()).with_message("");
        assert_eq!(report.message, Some("".to_string()));
    }

    #[test]
    fn verdict_thresholds_with_zero_max_warnings() {
        let thresholds = VerdictThresholds::new(true, Some(0));
        let counts = FindingCounts::from_counts(0, 0, 0, 0);
        assert_eq!(thresholds.evaluate(&counts), Verdict::Pass);
    }
}
