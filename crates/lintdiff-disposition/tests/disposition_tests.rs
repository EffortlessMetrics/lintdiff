//! Comprehensive tests for lintdiff-disposition.
//!
//! Test coverage:
//! 1. Disposition classification methods (10 tests)
//! 2. Disposition as_str/icon/label (6 tests)
//! 3. Disposition parsing (8 tests)
//! 4. Disposition Display (4 tests)
//! 5. DispositionReason variants (6 tests)
//! 6. DispositionWithReason methods (12 tests)
//! 7. DispositionWithReason Display (5 tests)
//! 8. Edge cases (4 tests)

use lintdiff_disposition::{
    Disposition, DispositionParseError, DispositionReason, DispositionWithReason,
};

// =============================================================================
// 1. Disposition classification methods (10 tests)
// =============================================================================

mod disposition_classification {
    use super::*;

    #[test]
    fn test_is_actionable_for_new() {
        assert!(Disposition::New.is_actionable());
    }

    #[test]
    fn test_is_actionable_for_fixed() {
        assert!(Disposition::Fixed.is_actionable());
    }

    #[test]
    fn test_is_actionable_for_pre_existing() {
        assert!(!Disposition::PreExisting.is_actionable());
    }

    #[test]
    fn test_is_actionable_for_suppressed() {
        assert!(!Disposition::Suppressed.is_actionable());
    }

    #[test]
    fn test_is_actionable_for_outside_diff() {
        assert!(!Disposition::OutsideDiff.is_actionable());
    }

    #[test]
    fn test_is_actionable_for_skipped() {
        assert!(!Disposition::Skipped.is_actionable());
    }

    #[test]
    fn test_is_new_returns_true_only_for_new() {
        assert!(Disposition::New.is_new());
        assert!(!Disposition::Fixed.is_new());
        assert!(!Disposition::PreExisting.is_new());
        assert!(!Disposition::Suppressed.is_new());
        assert!(!Disposition::OutsideDiff.is_new());
        assert!(!Disposition::Skipped.is_new());
    }

    #[test]
    fn test_is_fixed_returns_true_only_for_fixed() {
        assert!(!Disposition::New.is_fixed());
        assert!(Disposition::Fixed.is_fixed());
        assert!(!Disposition::PreExisting.is_fixed());
        assert!(!Disposition::Suppressed.is_fixed());
        assert!(!Disposition::OutsideDiff.is_fixed());
        assert!(!Disposition::Skipped.is_fixed());
    }

    #[test]
    fn test_is_reportable_excludes_suppressed_and_skipped() {
        assert!(Disposition::New.is_reportable());
        assert!(Disposition::Fixed.is_reportable());
        assert!(Disposition::PreExisting.is_reportable());
        assert!(!Disposition::Suppressed.is_reportable());
        assert!(Disposition::OutsideDiff.is_reportable());
        assert!(!Disposition::Skipped.is_reportable());
    }

    #[test]
    fn test_counts_toward_failure_only_for_new() {
        assert!(Disposition::New.counts_toward_failure());
        assert!(!Disposition::Fixed.counts_toward_failure());
        assert!(!Disposition::PreExisting.counts_toward_failure());
        assert!(!Disposition::Suppressed.counts_toward_failure());
        assert!(!Disposition::OutsideDiff.counts_toward_failure());
        assert!(!Disposition::Skipped.counts_toward_failure());
    }
}

// =============================================================================
// 2. Disposition as_str/icon/label (6 tests)
// =============================================================================

mod disposition_string_representations {
    use super::*;

    #[test]
    fn test_as_str_returns_correct_values() {
        assert_eq!(Disposition::New.as_str(), "new");
        assert_eq!(Disposition::Fixed.as_str(), "fixed");
        assert_eq!(Disposition::PreExisting.as_str(), "pre-existing");
        assert_eq!(Disposition::Suppressed.as_str(), "suppressed");
        assert_eq!(Disposition::OutsideDiff.as_str(), "outside-diff");
        assert_eq!(Disposition::Skipped.as_str(), "skipped");
    }

    #[test]
    fn test_icon_returns_correct_emoji() {
        assert_eq!(Disposition::New.icon(), "🆕");
        assert_eq!(Disposition::Fixed.icon(), "✅");
        assert_eq!(Disposition::PreExisting.icon(), "⏳");
        assert_eq!(Disposition::Suppressed.icon(), "🔇");
        assert_eq!(Disposition::OutsideDiff.icon(), "📍");
        assert_eq!(Disposition::Skipped.icon(), "⏭️");
    }

    #[test]
    fn test_label_returns_human_readable_text() {
        assert_eq!(Disposition::New.label(), "New Issue");
        assert_eq!(Disposition::Fixed.label(), "Fixed Issue");
        assert_eq!(Disposition::PreExisting.label(), "Pre-existing Issue");
        assert_eq!(Disposition::Suppressed.label(), "Suppressed Issue");
        assert_eq!(Disposition::OutsideDiff.label(), "Outside Diff Scope");
        assert_eq!(Disposition::Skipped.label(), "Skipped");
    }

    #[test]
    fn test_default_is_new() {
        assert_eq!(Disposition::default(), Disposition::New);
    }

    #[test]
    fn test_discriminant_values() {
        assert_eq!(Disposition::New as u8, 0);
        assert_eq!(Disposition::Fixed as u8, 1);
        assert_eq!(Disposition::PreExisting as u8, 2);
        assert_eq!(Disposition::Suppressed as u8, 3);
        assert_eq!(Disposition::OutsideDiff as u8, 4);
        assert_eq!(Disposition::Skipped as u8, 5);
    }

    #[test]
    fn test_all_variants_are_distinct() {
        let variants = [
            Disposition::New,
            Disposition::Fixed,
            Disposition::PreExisting,
            Disposition::Suppressed,
            Disposition::OutsideDiff,
            Disposition::Skipped,
        ];
        for (i, v1) in variants.iter().enumerate() {
            for v2 in variants.iter().skip(i + 1) {
                assert_ne!(v1, v2);
            }
        }
    }
}

// =============================================================================
// 3. Disposition parsing (8 tests)
// =============================================================================

mod disposition_parsing {
    use super::*;

    #[test]
    fn test_parse_new() {
        assert_eq!(Disposition::parse("new").unwrap(), Disposition::New);
        assert_eq!(Disposition::parse("NEW").unwrap(), Disposition::New);
        assert_eq!(Disposition::parse("New").unwrap(), Disposition::New);
    }

    #[test]
    fn test_parse_fixed() {
        assert_eq!(Disposition::parse("fixed").unwrap(), Disposition::Fixed);
        assert_eq!(Disposition::parse("FIXED").unwrap(), Disposition::Fixed);
    }

    #[test]
    fn test_parse_pre_existing_variants() {
        assert_eq!(
            Disposition::parse("pre-existing").unwrap(),
            Disposition::PreExisting
        );
        assert_eq!(
            Disposition::parse("preexisting").unwrap(),
            Disposition::PreExisting
        );
        assert_eq!(
            Disposition::parse("pre_existent").unwrap(),
            Disposition::PreExisting
        );
        assert_eq!(
            Disposition::parse("PRE-EXISTING").unwrap(),
            Disposition::PreExisting
        );
    }

    #[test]
    fn test_parse_suppressed() {
        assert_eq!(
            Disposition::parse("suppressed").unwrap(),
            Disposition::Suppressed
        );
        assert_eq!(
            Disposition::parse("SUPPRESSED").unwrap(),
            Disposition::Suppressed
        );
    }

    #[test]
    fn test_parse_outside_diff_variants() {
        assert_eq!(
            Disposition::parse("outside-diff").unwrap(),
            Disposition::OutsideDiff
        );
        assert_eq!(
            Disposition::parse("outside_diff").unwrap(),
            Disposition::OutsideDiff
        );
        assert_eq!(
            Disposition::parse("outside").unwrap(),
            Disposition::OutsideDiff
        );
    }

    #[test]
    fn test_parse_skipped() {
        assert_eq!(Disposition::parse("skipped").unwrap(), Disposition::Skipped);
        assert_eq!(Disposition::parse("SKIPPED").unwrap(), Disposition::Skipped);
    }

    #[test]
    fn test_parse_invalid_returns_error() {
        let result = Disposition::parse("invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid"));
    }

    #[test]
    fn test_std_from_str_trait() {
        // Test that the std::str::FromStr trait works
        let disposition: Disposition = "new".parse().unwrap();
        assert_eq!(disposition, Disposition::New);

        let disposition: Result<Disposition, _> = "fixed".parse();
        assert_eq!(disposition.unwrap(), Disposition::Fixed);
    }
}

// =============================================================================
// 4. Disposition Display (4 tests)
// =============================================================================

mod disposition_display {
    use super::*;

    #[test]
    fn test_display_uses_as_str() {
        assert_eq!(format!("{}", Disposition::New), "new");
        assert_eq!(format!("{}", Disposition::Fixed), "fixed");
        assert_eq!(format!("{}", Disposition::PreExisting), "pre-existing");
    }

    #[test]
    fn test_display_matches_parse_roundtrip() {
        for disposition in [
            Disposition::New,
            Disposition::Fixed,
            Disposition::PreExisting,
            Disposition::Suppressed,
            Disposition::OutsideDiff,
            Disposition::Skipped,
        ] {
            let displayed = format!("{}", disposition);
            let parsed: Disposition = displayed.parse().unwrap();
            assert_eq!(disposition, parsed);
        }
    }

    #[test]
    fn test_display_in_format_string() {
        let msg = format!("Found: {}", Disposition::New);
        assert_eq!(msg, "Found: new");
    }

    #[test]
    fn test_display_with_alternate_format() {
        // Alternate format should still work (no special handling)
        assert_eq!(format!("{:#}", Disposition::New), "new");
    }
}

// =============================================================================
// 5. DispositionReason variants (6 tests)
// =============================================================================

mod disposition_reason {
    use super::*;

    #[test]
    fn test_suppressed_by_rule() {
        let reason = DispositionReason::suppressed_by("clippy::all");
        assert_eq!(
            reason,
            DispositionReason::SuppressedByRule("clippy::all".to_string())
        );
        assert_eq!(reason.description(), "Suppressed by rule: clippy::all");
    }

    #[test]
    fn test_outside_diff_hunks() {
        let reason = DispositionReason::OutsideDiffHunks;
        assert_eq!(reason.description(), "Outside diff hunks");
    }

    #[test]
    fn test_generated_file() {
        let reason = DispositionReason::GeneratedFile;
        assert_eq!(reason.description(), "Generated file");
    }

    #[test]
    fn test_vendor_file() {
        let reason = DispositionReason::VendorFile;
        assert_eq!(reason.description(), "Vendor/third-party file");
    }

    #[test]
    fn test_no_span_info() {
        let reason = DispositionReason::NoSpanInfo;
        assert_eq!(reason.description(), "No span information");
    }

    #[test]
    fn test_custom_reason() {
        let reason = DispositionReason::custom("Some custom reason");
        assert_eq!(
            reason,
            DispositionReason::Custom("Some custom reason".to_string())
        );
        assert_eq!(reason.description(), "Some custom reason");
    }
}

// =============================================================================
// 6. DispositionWithReason methods (12 tests)
// =============================================================================

mod disposition_with_reason_methods {
    use super::*;

    #[test]
    fn test_new_without_reason() {
        let dwr = DispositionWithReason::new(Disposition::New);
        assert_eq!(dwr.disposition, Disposition::New);
        assert!(dwr.reason.is_none());
        assert!(!dwr.has_reason());
    }

    #[test]
    fn test_with_reason() {
        let dwr = DispositionWithReason::with_reason(
            Disposition::Suppressed,
            DispositionReason::suppressed_by("test-rule"),
        );
        assert_eq!(dwr.disposition, Disposition::Suppressed);
        assert!(dwr.has_reason());
    }

    #[test]
    fn test_new_issue() {
        let dwr = DispositionWithReason::new_issue();
        assert_eq!(dwr.disposition, Disposition::New);
        assert!(!dwr.has_reason());
    }

    #[test]
    fn test_fixed() {
        let dwr = DispositionWithReason::fixed();
        assert_eq!(dwr.disposition, Disposition::Fixed);
        assert!(!dwr.has_reason());
    }

    #[test]
    fn test_pre_existing() {
        let dwr = DispositionWithReason::pre_existing();
        assert_eq!(dwr.disposition, Disposition::PreExisting);
        assert!(!dwr.has_reason());
    }

    #[test]
    fn test_suppressed_with_rule() {
        let dwr = DispositionWithReason::suppressed("my-rule");
        assert_eq!(dwr.disposition, Disposition::Suppressed);
        assert!(dwr.has_reason());
        assert_eq!(
            dwr.reason,
            Some(DispositionReason::SuppressedByRule("my-rule".to_string()))
        );
    }

    #[test]
    fn test_outside_diff() {
        let dwr = DispositionWithReason::outside_diff();
        assert_eq!(dwr.disposition, Disposition::OutsideDiff);
        assert!(dwr.has_reason());
        assert_eq!(dwr.reason, Some(DispositionReason::OutsideDiffHunks));
    }

    #[test]
    fn test_as_disposition() {
        let dwr = DispositionWithReason::new(Disposition::Fixed);
        assert_eq!(dwr.as_disposition(), Disposition::Fixed);
    }

    #[test]
    fn test_has_reason_false_when_none() {
        let dwr = DispositionWithReason::new(Disposition::New);
        assert!(!dwr.has_reason());
    }

    #[test]
    fn test_has_reason_true_when_some() {
        let dwr =
            DispositionWithReason::with_reason(Disposition::New, DispositionReason::GeneratedFile);
        assert!(dwr.has_reason());
    }

    #[test]
    fn test_from_disposition_trait() {
        let dwr: DispositionWithReason = Disposition::New.into();
        assert_eq!(dwr.disposition, Disposition::New);
        assert!(!dwr.has_reason());
    }

    #[test]
    fn test_clone_and_eq() {
        let dwr1 = DispositionWithReason::suppressed("rule");
        let dwr2 = dwr1.clone();
        assert_eq!(dwr1, dwr2);
    }
}

// =============================================================================
// 7. DispositionWithReason Display (5 tests)
// =============================================================================

mod disposition_with_reason_display {
    use super::*;

    #[test]
    fn test_display_without_reason() {
        let dwr = DispositionWithReason::new(Disposition::New);
        assert_eq!(format!("{}", dwr), "new");
    }

    #[test]
    fn test_display_with_reason() {
        let dwr = DispositionWithReason::suppressed("test-rule");
        let displayed = format!("{}", dwr);
        assert!(displayed.contains("suppressed"));
        assert!(displayed.contains("test-rule"));
    }

    #[test]
    fn test_display_with_outside_diff_reason() {
        let dwr = DispositionWithReason::outside_diff();
        let displayed = format!("{}", dwr);
        assert!(displayed.contains("outside-diff"));
        assert!(displayed.contains("Outside diff hunks"));
    }

    #[test]
    fn test_display_format_for_all_dispositions() {
        for disposition in [
            Disposition::New,
            Disposition::Fixed,
            Disposition::PreExisting,
            Disposition::Suppressed,
            Disposition::OutsideDiff,
            Disposition::Skipped,
        ] {
            let dwr = DispositionWithReason::new(disposition);
            let displayed = format!("{}", dwr);
            assert_eq!(displayed, disposition.as_str());
        }
    }

    #[test]
    fn test_display_with_custom_reason() {
        let dwr = DispositionWithReason::with_reason(
            Disposition::Skipped,
            DispositionReason::custom("Custom explanation"),
        );
        let displayed = format!("{}", dwr);
        assert!(displayed.contains("skipped"));
        assert!(displayed.contains("Custom explanation"));
    }
}

// =============================================================================
// 8. Edge cases (4 tests)
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_disposition_parse_error_contains_input() {
        let err = Disposition::parse("foobar").unwrap_err();
        assert!(err.to_string().contains("foobar"));
    }

    #[test]
    fn test_disposition_equality_and_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Disposition::New);
        set.insert(Disposition::Fixed);
        set.insert(Disposition::New); // Duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_disposition_reason_equality() {
        let r1 = DispositionReason::suppressed_by("rule");
        let r2 = DispositionReason::SuppressedByRule("rule".to_string());
        assert_eq!(r1, r2);

        let r3 = DispositionReason::suppressed_by("other");
        assert_ne!(r1, r3);
    }

    #[test]
    fn test_empty_string_parse_error() {
        let result = Disposition::parse("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown disposition"));
    }
}

// =============================================================================
// Additional tests to meet the 55 test requirement
// =============================================================================

mod additional_coverage {
    use super::*;

    #[test]
    fn test_disposition_copy_trait() {
        fn takes_copy(d: Disposition) -> Disposition {
            d
        }
        let d = Disposition::New;
        let d2 = takes_copy(d);
        assert_eq!(d, d2);
    }

    #[test]
    fn test_disposition_debug_trait() {
        let debug_str = format!("{:?}", Disposition::New);
        assert!(debug_str.contains("New"));
    }

    #[test]
    fn test_disposition_reason_debug() {
        let reason = DispositionReason::OutsideDiffHunks;
        let debug_str = format!("{:?}", reason);
        assert!(debug_str.contains("OutsideDiffHunks"));
    }

    #[test]
    fn test_disposition_with_reason_debug() {
        let dwr = DispositionWithReason::new(Disposition::Fixed);
        let debug_str = format!("{:?}", dwr);
        assert!(debug_str.contains("Fixed"));
    }

    #[test]
    fn test_disposition_reason_display() {
        let reason = DispositionReason::GeneratedFile;
        assert_eq!(format!("{}", reason), "Generated file");
    }

    #[test]
    fn test_disposition_parse_error_debug() {
        let err = DispositionParseError::new("test");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("DispositionParseError"));
    }

    #[test]
    fn test_suppressed_with_empty_rule() {
        let dwr = DispositionWithReason::suppressed("");
        assert_eq!(dwr.disposition, Disposition::Suppressed);
        assert!(dwr.has_reason());
    }

    #[test]
    fn test_custom_reason_with_empty_string() {
        let reason = DispositionReason::custom("");
        assert_eq!(reason.description(), "");
    }

    #[test]
    fn test_disposition_ordering() {
        // Verify discriminant order
        assert!((Disposition::New as u8) < Disposition::Fixed as u8);
        assert!((Disposition::Fixed as u8) < Disposition::PreExisting as u8);
        assert!((Disposition::PreExisting as u8) < Disposition::Suppressed as u8);
        assert!((Disposition::Suppressed as u8) < Disposition::OutsideDiff as u8);
        assert!((Disposition::OutsideDiff as u8) < Disposition::Skipped as u8);
    }

    #[test]
    fn test_multiple_suppressed_rules() {
        let dwr1 = DispositionWithReason::suppressed("rule1");
        let dwr2 = DispositionWithReason::suppressed("rule2");
        assert_ne!(dwr1, dwr2);
    }

    #[test]
    fn test_disposition_with_reason_clone() {
        let dwr = DispositionWithReason::suppressed("rule");
        #[allow(clippy::clone_on_copy)]
        let cloned = dwr.clone();
        assert_eq!(dwr, cloned);
    }

    #[test]
    fn test_parse_with_whitespace() {
        // Whitespace should cause parse failure
        let result = Disposition::parse(" new ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_tabs() {
        let result = Disposition::parse("\tnew\t");
        assert!(result.is_err());
    }

    #[test]
    fn test_all_labels_are_unique() {
        let labels: Vec<&str> = [
            Disposition::New.label(),
            Disposition::Fixed.label(),
            Disposition::PreExisting.label(),
            Disposition::Suppressed.label(),
            Disposition::OutsideDiff.label(),
            Disposition::Skipped.label(),
        ]
        .to_vec();

        for (i, label) in labels.iter().enumerate() {
            for other in labels.iter().skip(i + 1) {
                assert_ne!(label, other);
            }
        }
    }

    #[test]
    fn test_all_icons_are_unique() {
        let icons: Vec<&str> = [
            Disposition::New.icon(),
            Disposition::Fixed.icon(),
            Disposition::PreExisting.icon(),
            Disposition::Suppressed.icon(),
            Disposition::OutsideDiff.icon(),
            Disposition::Skipped.icon(),
        ]
        .to_vec();

        for (i, icon) in icons.iter().enumerate() {
            for other in icons.iter().skip(i + 1) {
                assert_ne!(icon, other);
            }
        }
    }

    #[test]
    fn test_disposition_reason_clone() {
        let reason = DispositionReason::suppressed_by("rule");
        let cloned = reason.clone();
        assert_eq!(reason, cloned);
    }

    #[test]
    fn test_disposition_reason_display_for_all_variants() {
        assert_eq!(
            format!("{}", DispositionReason::OutsideDiffHunks),
            "Outside diff hunks"
        );
        assert_eq!(
            format!("{}", DispositionReason::GeneratedFile),
            "Generated file"
        );
        assert_eq!(
            format!("{}", DispositionReason::VendorFile),
            "Vendor/third-party file"
        );
        assert_eq!(
            format!("{}", DispositionReason::NoSpanInfo),
            "No span information"
        );
    }

    #[test]
    fn test_disposition_with_reason_partial_eq() {
        let dwr1 = DispositionWithReason::new(Disposition::New);
        let dwr2 = DispositionWithReason::new(Disposition::New);
        assert_eq!(dwr1, dwr2);

        let dwr3 = DispositionWithReason::new(Disposition::Fixed);
        assert_ne!(dwr1, dwr3);
    }

    #[test]
    fn test_disposition_parse_error_unknown_input() {
        let err = DispositionParseError::new("bad-input");
        assert_eq!(err.unknown_input(), "bad-input");
    }
}
