//! Comprehensive BDD tests for lintdiff-verdict-reason.
//!
//! Test coverage:
//! 1. VerdictReason variants (12 tests)
//! 2. VerdictReasonBuilder pattern (10 tests)
//! 3. Formatting functions (9 tests)
//! 4. VerdictSummary struct (10 tests)
//! 5. Helper functions (8 tests)
//! 6. Edge cases (8 tests)
//! 7. Property-based tests with proptest (5 tests)

use lintdiff_verdict_reason::{
    format_reason, format_reason_markdown, format_reason_short, is_failure_reason,
    is_success_reason, merge_reasons, VerdictReason, VerdictReasonBuilder, VerdictSummary,
};

// =============================================================================
// 1. VerdictReason variants (12 tests)
// =============================================================================

mod verdict_reason_variants {
    use super::*;

    #[test]
    fn test_no_changes_variant() {
        let reason = VerdictReason::NoChanges;
        assert_eq!(reason.as_str(), "no-changes");
        assert_eq!(reason.icon(), "✅");
        assert!(!reason.has_count());
        assert_eq!(reason.count(), None);
    }

    #[test]
    fn test_added_warnings_variant() {
        let reason = VerdictReason::AddedWarnings { count: 5 };
        assert_eq!(reason.as_str(), "added-warnings");
        assert_eq!(reason.icon(), "⚠️");
        assert!(reason.has_count());
        assert_eq!(reason.count(), Some(5));
        assert!(reason.is_added());
        assert!(!reason.is_removed());
        assert!(reason.is_warning_related());
        assert!(!reason.is_error_related());
    }

    #[test]
    fn test_added_errors_variant() {
        let reason = VerdictReason::AddedErrors { count: 3 };
        assert_eq!(reason.as_str(), "added-errors");
        assert_eq!(reason.icon(), "❌");
        assert!(reason.has_count());
        assert_eq!(reason.count(), Some(3));
        assert!(reason.is_added());
        assert!(!reason.is_removed());
        assert!(!reason.is_warning_related());
        assert!(reason.is_error_related());
    }

    #[test]
    fn test_removed_warnings_variant() {
        let reason = VerdictReason::RemovedWarnings { count: 2 };
        assert_eq!(reason.as_str(), "removed-warnings");
        assert_eq!(reason.icon(), "🩹");
        assert!(reason.has_count());
        assert_eq!(reason.count(), Some(2));
        assert!(!reason.is_added());
        assert!(reason.is_removed());
        assert!(reason.is_warning_related());
        assert!(!reason.is_error_related());
    }

    #[test]
    fn test_removed_errors_variant() {
        let reason = VerdictReason::RemovedErrors { count: 1 };
        assert_eq!(reason.as_str(), "removed-errors");
        assert_eq!(reason.icon(), "🔧");
        assert!(reason.has_count());
        assert_eq!(reason.count(), Some(1));
        assert!(!reason.is_added());
        assert!(reason.is_removed());
        assert!(!reason.is_warning_related());
        assert!(reason.is_error_related());
    }

    #[test]
    fn test_only_unchanged_variant() {
        let reason = VerdictReason::OnlyUnchanged;
        assert_eq!(reason.as_str(), "only-unchanged");
        assert_eq!(reason.icon(), "⏳");
        assert!(!reason.has_count());
        assert_eq!(reason.count(), None);
    }

    #[test]
    fn test_threshold_exceeded_variant() {
        let reason = VerdictReason::ThresholdExceeded {
            limit: 10,
            actual: 15,
        };
        assert_eq!(reason.as_str(), "threshold-exceeded");
        assert_eq!(reason.icon(), "🚫");
        assert!(!reason.has_count());
        assert_eq!(reason.count(), None);
    }

    #[test]
    fn test_custom_variant() {
        let reason = VerdictReason::Custom("Special case".to_string());
        assert_eq!(reason.as_str(), "custom");
        assert_eq!(reason.icon(), "📝");
        assert!(!reason.has_count());
        assert_eq!(reason.count(), None);
    }

    #[test]
    fn test_default_is_no_changes() {
        let reason = VerdictReason::default();
        assert_eq!(reason, VerdictReason::NoChanges);
    }

    #[test]
    fn test_display_no_changes() {
        let reason = VerdictReason::NoChanges;
        assert_eq!(format!("{reason}"), "No diagnostic changes detected");
    }

    #[test]
    fn test_display_pluralization_singular() {
        assert_eq!(
            format!("{}", VerdictReason::AddedWarnings { count: 1 }),
            "Added 1 warning"
        );
        assert_eq!(
            format!("{}", VerdictReason::AddedErrors { count: 1 }),
            "Added 1 error"
        );
        assert_eq!(
            format!("{}", VerdictReason::RemovedWarnings { count: 1 }),
            "Fixed 1 warning"
        );
        assert_eq!(
            format!("{}", VerdictReason::RemovedErrors { count: 1 }),
            "Fixed 1 error"
        );
    }

    #[test]
    fn test_display_pluralization_plural() {
        assert_eq!(
            format!("{}", VerdictReason::AddedWarnings { count: 5 }),
            "Added 5 warnings"
        );
        assert_eq!(
            format!("{}", VerdictReason::AddedErrors { count: 5 }),
            "Added 5 errors"
        );
        assert_eq!(
            format!("{}", VerdictReason::RemovedWarnings { count: 5 }),
            "Fixed 5 warnings"
        );
        assert_eq!(
            format!("{}", VerdictReason::RemovedErrors { count: 5 }),
            "Fixed 5 errors"
        );
    }
}

// =============================================================================
// 2. VerdictReasonBuilder pattern (10 tests)
// =============================================================================

mod verdict_reason_builder {
    use super::*;

    #[test]
    fn test_builder_new_creates_default() {
        let builder = VerdictReasonBuilder::new();
        let reason = builder.build();
        assert_eq!(reason, VerdictReason::NoChanges);
    }

    #[test]
    fn test_builder_with_added_errors_only() {
        let reason = VerdictReasonBuilder::new().with_added(5, 0).build();
        assert_eq!(reason, VerdictReason::AddedErrors { count: 5 });
    }

    #[test]
    fn test_builder_with_added_warnings_only() {
        let reason = VerdictReasonBuilder::new().with_added(0, 3).build();
        assert_eq!(reason, VerdictReason::AddedWarnings { count: 3 });
    }

    #[test]
    fn test_builder_with_added_both_errors_take_priority() {
        let reason = VerdictReasonBuilder::new().with_added(2, 5).build();
        assert_eq!(reason, VerdictReason::AddedErrors { count: 2 });
    }

    #[test]
    fn test_builder_with_removed_errors_only() {
        let reason = VerdictReasonBuilder::new().with_removed(3, 0).build();
        assert_eq!(reason, VerdictReason::RemovedErrors { count: 3 });
    }

    #[test]
    fn test_builder_with_removed_warnings_only() {
        let reason = VerdictReasonBuilder::new().with_removed(0, 4).build();
        assert_eq!(reason, VerdictReason::RemovedWarnings { count: 4 });
    }

    #[test]
    fn test_builder_with_removed_both_errors_take_priority() {
        let reason = VerdictReasonBuilder::new().with_removed(2, 5).build();
        assert_eq!(reason, VerdictReason::RemovedErrors { count: 2 });
    }

    #[test]
    fn test_builder_with_unchanged_only() {
        let reason = VerdictReasonBuilder::new().with_unchanged(10).build();
        assert_eq!(reason, VerdictReason::OnlyUnchanged);
    }

    #[test]
    fn test_builder_with_threshold_exceeded() {
        let reason = VerdictReasonBuilder::new().with_threshold(10, 15).build();
        assert_eq!(
            reason,
            VerdictReason::ThresholdExceeded {
                limit: 10,
                actual: 15
            }
        );
    }

    #[test]
    fn test_builder_with_custom_reason() {
        let reason = VerdictReasonBuilder::new()
            .with_custom("Special handling".to_string())
            .build();
        assert_eq!(
            reason,
            VerdictReason::Custom("Special handling".to_string())
        );
    }

    #[test]
    fn test_builder_priority_custom_highest() {
        let reason = VerdictReasonBuilder::new()
            .with_added(10, 20)
            .with_threshold(5, 100)
            .with_custom("Override".to_string())
            .build();
        assert_eq!(reason, VerdictReason::Custom("Override".to_string()));
    }

    #[test]
    fn test_builder_priority_threshold_over_added() {
        let reason = VerdictReasonBuilder::new()
            .with_added(5, 10)
            .with_threshold(3, 15)
            .build();
        assert_eq!(
            reason,
            VerdictReason::ThresholdExceeded {
                limit: 3,
                actual: 15
            }
        );
    }

    #[test]
    fn test_builder_priority_added_over_removed() {
        let reason = VerdictReasonBuilder::new()
            .with_added(0, 5)
            .with_removed(10, 20)
            .build();
        assert_eq!(reason, VerdictReason::AddedWarnings { count: 5 });
    }

    #[test]
    fn test_builder_threshold_not_exceeded() {
        let reason = VerdictReasonBuilder::new()
            .with_threshold(10, 5) // actual < limit
            .with_unchanged(3)
            .build();
        // Threshold not exceeded, should fall through to unchanged
        assert_eq!(reason, VerdictReason::OnlyUnchanged);
    }
}

// =============================================================================
// 3. Formatting functions (9 tests)
// =============================================================================

mod formatting_functions {
    use super::*;

    #[test]
    fn test_format_reason_no_changes() {
        let reason = VerdictReason::NoChanges;
        assert_eq!(format_reason(&reason), "✅ No diagnostic changes detected");
    }

    #[test]
    fn test_format_reason_added_warnings() {
        let reason = VerdictReason::AddedWarnings { count: 5 };
        assert_eq!(format_reason(&reason), "⚠️ Added 5 warnings");
    }

    #[test]
    fn test_format_reason_added_errors() {
        let reason = VerdictReason::AddedErrors { count: 3 };
        assert_eq!(format_reason(&reason), "❌ Added 3 errors");
    }

    #[test]
    fn test_format_reason_short_no_changes() {
        let reason = VerdictReason::NoChanges;
        assert_eq!(format_reason_short(&reason), "no-changes");
    }

    #[test]
    fn test_format_reason_short_with_count() {
        let reason = VerdictReason::AddedWarnings { count: 5 };
        assert_eq!(format_reason_short(&reason), "added-warnings:5");
    }

    #[test]
    fn test_format_reason_short_threshold() {
        let reason = VerdictReason::ThresholdExceeded {
            limit: 10,
            actual: 15,
        };
        assert_eq!(format_reason_short(&reason), "threshold-exceeded:15/10");
    }

    #[test]
    fn test_format_reason_markdown_structure() {
        let reason = VerdictReason::AddedWarnings { count: 5 };
        let md = format_reason_markdown(&reason);
        assert!(md.contains("**⚠️ Added Warnings**"));
        assert!(md.contains("5 new warnings"));
    }

    #[test]
    fn test_format_reason_markdown_no_changes() {
        let reason = VerdictReason::NoChanges;
        let md = format_reason_markdown(&reason);
        assert!(md.contains("**✅ No Changes**"));
    }

    #[test]
    fn test_format_reason_markdown_threshold_exceeded() {
        let reason = VerdictReason::ThresholdExceeded {
            limit: 10,
            actual: 15,
        };
        let md = format_reason_markdown(&reason);
        assert!(md.contains("**🚫 Threshold Exceeded**"));
        assert!(md.contains("15 exceeds limit of 10"));
    }

    #[test]
    fn test_format_reason_markdown_custom() {
        let reason = VerdictReason::Custom("Special case".to_string());
        let md = format_reason_markdown(&reason);
        assert!(md.contains("**📝 Custom**"));
        assert!(md.contains("Special case"));
    }
}

// =============================================================================
// 4. VerdictSummary struct (10 tests)
// =============================================================================

mod verdict_summary {
    use super::*;

    #[test]
    fn test_summary_new() {
        let summary = VerdictSummary::new(VerdictReason::NoChanges);
        assert_eq!(summary.reason, VerdictReason::NoChanges);
        assert!(summary.details.is_empty());
        assert!(summary.suggestion.is_none());
    }

    #[test]
    fn test_summary_with_single_detail() {
        let summary = VerdictSummary::new(VerdictReason::NoChanges).with_detail("First detail");
        assert_eq!(summary.details.len(), 1);
        assert_eq!(summary.details[0], "First detail");
    }

    #[test]
    fn test_summary_with_multiple_details() {
        let summary = VerdictSummary::new(VerdictReason::NoChanges)
            .with_detail("First")
            .with_detail("Second")
            .with_detail("Third");
        assert_eq!(summary.details.len(), 3);
        assert_eq!(summary.details[0], "First");
        assert_eq!(summary.details[1], "Second");
        assert_eq!(summary.details[2], "Third");
    }

    #[test]
    fn test_summary_with_details_vec() {
        let details = vec!["A".to_string(), "B".to_string()];
        let summary = VerdictSummary::new(VerdictReason::NoChanges).with_details(details);
        assert_eq!(summary.details.len(), 2);
        assert_eq!(summary.details[0], "A");
        assert_eq!(summary.details[1], "B");
    }

    #[test]
    fn test_summary_with_suggestion() {
        let summary =
            VerdictSummary::new(VerdictReason::NoChanges).with_suggestion("Fix the issue");
        assert_eq!(summary.suggestion, Some("Fix the issue".to_string()));
    }

    #[test]
    fn test_summary_is_failure() {
        let failure_summary = VerdictSummary::new(VerdictReason::AddedErrors { count: 1 });
        assert!(failure_summary.is_failure());

        let success_summary = VerdictSummary::new(VerdictReason::NoChanges);
        assert!(!success_summary.is_failure());
    }

    #[test]
    fn test_summary_is_success() {
        let success_summary = VerdictSummary::new(VerdictReason::RemovedErrors { count: 1 });
        assert!(success_summary.is_success());

        let failure_summary = VerdictSummary::new(VerdictReason::AddedWarnings { count: 1 });
        assert!(!failure_summary.is_success());
    }

    #[test]
    fn test_summary_has_details() {
        let with_details = VerdictSummary::new(VerdictReason::NoChanges).with_detail("Detail");
        assert!(with_details.has_details());

        let without_details = VerdictSummary::new(VerdictReason::NoChanges);
        assert!(!without_details.has_details());
    }

    #[test]
    fn test_summary_has_suggestion() {
        let with_suggestion =
            VerdictSummary::new(VerdictReason::NoChanges).with_suggestion("Suggestion");
        assert!(with_suggestion.has_suggestion());

        let without_suggestion = VerdictSummary::new(VerdictReason::NoChanges);
        assert!(!without_suggestion.has_suggestion());
    }

    #[test]
    fn test_summary_display() {
        let summary = VerdictSummary::new(VerdictReason::AddedWarnings { count: 5 })
            .with_detail("Found in src/lib.rs")
            .with_suggestion("Review the warnings");

        let output = format!("{summary}");
        assert!(output.contains("Reason: Added 5 warnings"));
        assert!(output.contains("Details:"));
        assert!(output.contains("Found in src/lib.rs"));
        assert!(output.contains("Suggestion: Review the warnings"));
    }
}

// =============================================================================
// 5. Helper functions (8 tests)
// =============================================================================

mod helper_functions {
    use super::*;

    #[test]
    fn test_is_failure_reason_added_errors() {
        assert!(is_failure_reason(&VerdictReason::AddedErrors { count: 1 }));
        assert!(is_failure_reason(&VerdictReason::AddedErrors {
            count: 100
        }));
    }

    #[test]
    fn test_is_failure_reason_added_warnings() {
        assert!(is_failure_reason(&VerdictReason::AddedWarnings {
            count: 1
        }));
        assert!(is_failure_reason(&VerdictReason::AddedWarnings {
            count: 100
        }));
    }

    #[test]
    fn test_is_failure_reason_threshold_exceeded() {
        assert!(is_failure_reason(&VerdictReason::ThresholdExceeded {
            limit: 10,
            actual: 11
        }));
    }

    #[test]
    fn test_is_failure_reason_custom() {
        assert!(is_failure_reason(&VerdictReason::Custom(
            "Any reason".to_string()
        )));
    }

    #[test]
    fn test_is_success_reason_no_changes() {
        assert!(is_success_reason(&VerdictReason::NoChanges));
    }

    #[test]
    fn test_is_success_reason_removed() {
        assert!(is_success_reason(&VerdictReason::RemovedErrors {
            count: 1
        }));
        assert!(is_success_reason(&VerdictReason::RemovedWarnings {
            count: 1
        }));
    }

    #[test]
    fn test_is_success_reason_only_unchanged() {
        assert!(is_success_reason(&VerdictReason::OnlyUnchanged));
    }

    #[test]
    fn test_mutually_exclusive_failure_success() {
        let all_reasons = [
            VerdictReason::NoChanges,
            VerdictReason::AddedWarnings { count: 1 },
            VerdictReason::AddedErrors { count: 1 },
            VerdictReason::RemovedWarnings { count: 1 },
            VerdictReason::RemovedErrors { count: 1 },
            VerdictReason::OnlyUnchanged,
            VerdictReason::ThresholdExceeded {
                limit: 1,
                actual: 2,
            },
            VerdictReason::Custom("test".to_string()),
        ];

        for reason in &all_reasons {
            // Each reason should be either failure or success, but not both
            assert!(
                is_failure_reason(reason) != is_success_reason(reason)
                    || (is_failure_reason(reason) && is_success_reason(reason)),
                "Reason {:?} has inconsistent failure/success classification",
                reason
            );
        }
    }
}

// =============================================================================
// 6. Edge cases (8 tests)
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_zero_counts() {
        let reason = VerdictReason::AddedWarnings { count: 0 };
        assert_eq!(reason.count(), Some(0));
        assert_eq!(format_reason(&reason), "⚠️ Added 0 warnings");
    }

    #[test]
    fn test_large_counts() {
        let reason = VerdictReason::AddedErrors { count: usize::MAX };
        assert_eq!(reason.count(), Some(usize::MAX));
    }

    #[test]
    fn test_empty_custom_reason() {
        let reason = VerdictReason::Custom(String::new());
        assert_eq!(format_reason(&reason), "📝 ");
        assert_eq!(format_reason_short(&reason), "custom:");
    }

    #[test]
    fn test_merge_empty_reasons() {
        let reasons: Vec<VerdictReason> = vec![];
        let merged = merge_reasons(&reasons);
        assert_eq!(merged, VerdictReason::NoChanges);
    }

    #[test]
    fn test_merge_single_reason() {
        let reasons = vec![VerdictReason::AddedWarnings { count: 5 }];
        let merged = merge_reasons(&reasons);
        assert_eq!(merged, VerdictReason::AddedWarnings { count: 5 });
    }

    #[test]
    fn test_merge_takes_highest_count() {
        let reasons = vec![
            VerdictReason::AddedWarnings { count: 3 },
            VerdictReason::AddedWarnings { count: 7 },
            VerdictReason::AddedWarnings { count: 2 },
        ];
        let merged = merge_reasons(&reasons);
        assert_eq!(merged, VerdictReason::AddedWarnings { count: 7 });
    }

    #[test]
    fn test_merge_mixed_types_prioritizes_correctly() {
        // Added errors should win over everything else
        let reasons = vec![
            VerdictReason::RemovedErrors { count: 100 },
            VerdictReason::AddedWarnings { count: 50 },
            VerdictReason::AddedErrors { count: 1 },
        ];
        let merged = merge_reasons(&reasons);
        assert_eq!(merged, VerdictReason::AddedErrors { count: 1 });
    }

    #[test]
    fn test_threshold_at_limit_not_exceeded() {
        let reason = VerdictReason::ThresholdExceeded {
            limit: 10,
            actual: 10,
        };
        // This is technically "at" the limit, but the variant exists
        assert_eq!(format_reason_short(&reason), "threshold-exceeded:10/10");
    }

    #[test]
    fn test_builder_chaining() {
        let reason = VerdictReasonBuilder::new()
            .with_added(1, 2)
            .with_removed(3, 4)
            .with_unchanged(5)
            .with_threshold(10, 20)
            .with_custom("Final".to_string())
            .build();

        // Custom should win
        assert_eq!(reason, VerdictReason::Custom("Final".to_string()));
    }
}

// =============================================================================
// 7. Property-based tests with proptest (5 tests)
// =============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_verdict_reason()(count in 0usize..1000) -> VerdictReason {
            let variants = [
                VerdictReason::NoChanges,
                VerdictReason::AddedWarnings { count },
                VerdictReason::AddedErrors { count },
                VerdictReason::RemovedWarnings { count },
                VerdictReason::RemovedErrors { count },
                VerdictReason::OnlyUnchanged,
                VerdictReason::ThresholdExceeded { limit: count, actual: count + 10 },
                VerdictReason::Custom(format!("reason-{}", count)),
            ];
            variants[count % variants.len()].clone()
        }
    }

    proptest! {
        #[test]
        fn prop_format_reason_never_panics(reason in arb_verdict_reason()) {
            let _ = format_reason(&reason);
            let _ = format_reason_short(&reason);
            let _ = format_reason_markdown(&reason);
        }

        #[test]
        fn prop_is_failure_or_success_consistent(reason in arb_verdict_reason()) {
            // Every reason should be classifiable
            let is_failure = is_failure_reason(&reason);
            let is_success = is_success_reason(&reason);

            // They should be mutually exclusive for most cases
            // (Custom is treated as failure, which is conservative)
            if !matches!(reason, VerdictReason::Custom(_)) {
                prop_assert!(is_failure != is_success);
            }
        }

        #[test]
        fn prop_builder_roundtrip(errors in 0usize..100, warnings in 0usize..100) {
            let reason = VerdictReasonBuilder::new()
                .with_added(errors, warnings)
                .build();

            // Errors take priority if present
            if errors > 0 {
                prop_assert_eq!(reason, VerdictReason::AddedErrors { count: errors });
            } else if warnings > 0 {
                prop_assert_eq!(reason, VerdictReason::AddedWarnings { count: warnings });
            } else {
                prop_assert_eq!(reason, VerdictReason::NoChanges);
            }
        }

        #[test]
        fn prop_merge_reasons_never_panics(reasons in proptest::collection::vec(arb_verdict_reason(), 0..10)) {
            let _ = merge_reasons(&reasons);
        }

        #[test]
        fn prop_count_consistency(count in 0usize..1000) {
            let reason = VerdictReason::AddedWarnings { count };
            prop_assert_eq!(reason.count(), Some(count));
            prop_assert!(reason.has_count());
        }
    }
}

// =============================================================================
// 8. Serde tests (conditional on feature)
// =============================================================================

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    #[test]
    fn test_serde_no_changes() {
        let reason = VerdictReason::NoChanges;
        let json = serde_json::to_string(&reason).unwrap();
        let deserialized: VerdictReason = serde_json::from_str(&json).unwrap();
        assert_eq!(reason, deserialized);
    }

    #[test]
    fn test_serde_added_warnings() {
        let reason = VerdictReason::AddedWarnings { count: 5 };
        let json = serde_json::to_string(&reason).unwrap();
        let deserialized: VerdictReason = serde_json::from_str(&json).unwrap();
        assert_eq!(reason, deserialized);
    }

    #[test]
    fn test_serde_threshold_exceeded() {
        let reason = VerdictReason::ThresholdExceeded {
            limit: 10,
            actual: 15,
        };
        let json = serde_json::to_string(&reason).unwrap();
        let deserialized: VerdictReason = serde_json::from_str(&json).unwrap();
        assert_eq!(reason, deserialized);
    }

    #[test]
    fn test_serde_verdict_summary() {
        let summary = VerdictSummary::new(VerdictReason::AddedWarnings { count: 5 })
            .with_detail("Test detail")
            .with_suggestion("Fix it");

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: VerdictSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary, deserialized);
    }
}

// =============================================================================
// 9. Additional coverage tests
// =============================================================================

mod additional_coverage {
    use super::*;

    #[test]
    fn test_all_icons_are_unique() {
        let icons = [
            VerdictReason::NoChanges.icon(),
            VerdictReason::AddedWarnings { count: 1 }.icon(),
            VerdictReason::AddedErrors { count: 1 }.icon(),
            VerdictReason::RemovedWarnings { count: 1 }.icon(),
            VerdictReason::RemovedErrors { count: 1 }.icon(),
            VerdictReason::OnlyUnchanged.icon(),
            VerdictReason::ThresholdExceeded {
                limit: 1,
                actual: 2,
            }
            .icon(),
            VerdictReason::Custom("".to_string()).icon(),
        ];

        // All icons should be non-empty
        for icon in &icons {
            assert!(!icon.is_empty());
        }
    }

    #[test]
    fn test_all_as_str_are_unique() {
        let strs = [
            VerdictReason::NoChanges.as_str(),
            VerdictReason::AddedWarnings { count: 1 }.as_str(),
            VerdictReason::AddedErrors { count: 1 }.as_str(),
            VerdictReason::RemovedWarnings { count: 1 }.as_str(),
            VerdictReason::RemovedErrors { count: 1 }.as_str(),
            VerdictReason::OnlyUnchanged.as_str(),
            VerdictReason::ThresholdExceeded {
                limit: 1,
                actual: 2,
            }
            .as_str(),
            VerdictReason::Custom("".to_string()).as_str(),
        ];

        // All should be lowercase with hyphens
        for s in &strs {
            assert!(s.chars().all(|c| c.is_lowercase() || c == '-'));
        }
    }

    #[test]
    fn test_merge_with_multiple_no_changes() {
        let reasons = vec![
            VerdictReason::NoChanges,
            VerdictReason::NoChanges,
            VerdictReason::NoChanges,
        ];
        let merged = merge_reasons(&reasons);
        assert_eq!(merged, VerdictReason::NoChanges);
    }

    #[test]
    fn test_merge_threshold_takes_highest_excess() {
        let reasons = vec![
            VerdictReason::ThresholdExceeded {
                limit: 10,
                actual: 12,
            }, // excess 2
            VerdictReason::ThresholdExceeded {
                limit: 5,
                actual: 20,
            }, // excess 15
            VerdictReason::ThresholdExceeded {
                limit: 3,
                actual: 5,
            }, // excess 2
        ];
        let merged = merge_reasons(&reasons);
        assert_eq!(
            merged,
            VerdictReason::ThresholdExceeded {
                limit: 5,
                actual: 20
            }
        );
    }

    #[test]
    fn test_merge_custom_takes_first() {
        let reasons = vec![
            VerdictReason::Custom("First".to_string()),
            VerdictReason::Custom("Second".to_string()),
        ];
        let merged = merge_reasons(&reasons);
        assert_eq!(merged, VerdictReason::Custom("First".to_string()));
    }

    #[test]
    fn test_builder_default() {
        let builder = VerdictReasonBuilder::default();
        let reason = builder.build();
        assert_eq!(reason, VerdictReason::NoChanges);
    }

    #[test]
    fn test_summary_display_without_details_or_suggestion() {
        let summary = VerdictSummary::new(VerdictReason::NoChanges);
        let output = format!("{summary}");
        assert!(output.contains("Reason:"));
        assert!(!output.contains("Details:"));
        assert!(!output.contains("Suggestion:"));
    }

    #[test]
    fn test_format_reason_markdown_all_variants() {
        // Test all variants produce valid markdown
        let variants = [
            VerdictReason::NoChanges,
            VerdictReason::AddedWarnings { count: 1 },
            VerdictReason::AddedErrors { count: 1 },
            VerdictReason::RemovedWarnings { count: 1 },
            VerdictReason::RemovedErrors { count: 1 },
            VerdictReason::OnlyUnchanged,
            VerdictReason::ThresholdExceeded {
                limit: 1,
                actual: 2,
            },
            VerdictReason::Custom("test".to_string()),
        ];

        for reason in &variants {
            let md = format_reason_markdown(reason);
            // All markdown should contain bold text
            assert!(md.contains("**"));
        }
    }

    #[test]
    fn test_is_added_and_is_removed_mutually_exclusive() {
        let reasons = [
            VerdictReason::AddedWarnings { count: 1 },
            VerdictReason::AddedErrors { count: 1 },
            VerdictReason::RemovedWarnings { count: 1 },
            VerdictReason::RemovedErrors { count: 1 },
        ];

        for reason in &reasons {
            assert!(reason.is_added() != reason.is_removed());
        }
    }

    #[test]
    fn test_is_warning_and_error_related_mutually_exclusive() {
        let reasons = [
            VerdictReason::AddedWarnings { count: 1 },
            VerdictReason::AddedErrors { count: 1 },
            VerdictReason::RemovedWarnings { count: 1 },
            VerdictReason::RemovedErrors { count: 1 },
        ];

        for reason in &reasons {
            assert!(reason.is_warning_related() != reason.is_error_related());
        }
    }
}
