//! Comprehensive tests for the lintdiff-severity crate.
//!
//! This test module covers:
//! - Severity ordering (10 tests)
//! - from_str parsing with all aliases (15 tests)
//! - at_least and at_most comparisons (10 tests)
//! - is_problem and is_blocking checks (8 tests)
//! - Display and as_str formatting (6 tests)
//! - SeverityThreshold functionality (10 tests)
//! - Edge cases and error conditions (6 tests)

use lintdiff_severity::{Severity, SeverityParseError, SeverityThreshold};
use std::str::FromStr;

// =============================================================================
// Severity Ordering Tests (10 tests)
// =============================================================================

mod severity_ordering {
    use super::*;

    #[test]
    fn test_hint_is_least_severe() {
        assert!(Severity::Hint < Severity::Note);
        assert!(Severity::Hint < Severity::Warning);
        assert!(Severity::Hint < Severity::Error);
        assert!(Severity::Hint < Severity::Fatal);
    }

    #[test]
    fn test_note_is_less_than_warning() {
        assert!(Severity::Note < Severity::Warning);
        assert!(Severity::Note < Severity::Error);
        assert!(Severity::Note < Severity::Fatal);
        assert!(Severity::Note > Severity::Hint);
    }

    #[test]
    fn test_warning_is_less_than_error() {
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Warning < Severity::Fatal);
        assert!(Severity::Warning > Severity::Note);
        assert!(Severity::Warning > Severity::Hint);
    }

    #[test]
    fn test_error_is_less_than_fatal() {
        assert!(Severity::Error < Severity::Fatal);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Error > Severity::Note);
        assert!(Severity::Error > Severity::Hint);
    }

    #[test]
    fn test_fatal_is_most_severe() {
        assert!(Severity::Fatal > Severity::Error);
        assert!(Severity::Fatal > Severity::Warning);
        assert!(Severity::Fatal > Severity::Note);
        assert!(Severity::Fatal > Severity::Hint);
    }

    #[test]
    fn test_equality() {
        assert_eq!(Severity::Hint, Severity::Hint);
        assert_eq!(Severity::Note, Severity::Note);
        assert_eq!(Severity::Warning, Severity::Warning);
        assert_eq!(Severity::Error, Severity::Error);
        assert_eq!(Severity::Fatal, Severity::Fatal);
    }

    #[test]
    fn test_less_than_or_equal() {
        assert!(Severity::Hint <= Severity::Hint);
        assert!(Severity::Hint <= Severity::Note);
        assert!(Severity::Warning <= Severity::Warning);
        assert!(Severity::Warning <= Severity::Error);
    }

    #[test]
    fn test_greater_than_or_equal() {
        assert!(Severity::Fatal >= Severity::Fatal);
        assert!(Severity::Fatal >= Severity::Error);
        assert!(Severity::Error >= Severity::Warning);
        assert!(Severity::Note >= Severity::Hint);
    }

    #[test]
    fn test_level_values() {
        assert_eq!(Severity::Hint.level(), 0);
        assert_eq!(Severity::Note.level(), 1);
        assert_eq!(Severity::Warning.level(), 2);
        assert_eq!(Severity::Error.level(), 3);
        assert_eq!(Severity::Fatal.level(), 4);
    }

    #[test]
    fn test_level_ordering_consistency() {
        // Verify that level() returns values consistent with Ord ordering
        let severities = [
            Severity::Hint,
            Severity::Note,
            Severity::Warning,
            Severity::Error,
            Severity::Fatal,
        ];
        for (i, &sev) in severities.iter().enumerate() {
            assert_eq!(sev.level(), i as u8);
        }
    }
}

// =============================================================================
// from_str Parsing Tests (15 tests)
// =============================================================================

mod from_str_parsing {
    use super::*;

    // Hint aliases
    #[test]
    fn test_parse_hint_lowercase() {
        assert_eq!(Severity::from_str("hint").unwrap(), Severity::Hint);
    }

    #[test]
    fn test_parse_hint_uppercase() {
        assert_eq!(Severity::from_str("HINT").unwrap(), Severity::Hint);
    }

    #[test]
    fn test_parse_hint_mixed_case() {
        assert_eq!(Severity::from_str("HiNt").unwrap(), Severity::Hint);
    }

    #[test]
    fn test_parse_info_alias() {
        assert_eq!(Severity::from_str("info").unwrap(), Severity::Hint);
        assert_eq!(Severity::from_str("INFO").unwrap(), Severity::Hint);
    }

    #[test]
    fn test_parse_information_alias() {
        assert_eq!(Severity::from_str("information").unwrap(), Severity::Hint);
        assert_eq!(Severity::from_str("INFORMATION").unwrap(), Severity::Hint);
    }

    // Note aliases
    #[test]
    fn test_parse_note() {
        assert_eq!(Severity::from_str("note").unwrap(), Severity::Note);
        assert_eq!(Severity::from_str("NOTE").unwrap(), Severity::Note);
    }

    #[test]
    fn test_parse_suggestion_alias() {
        assert_eq!(Severity::from_str("suggestion").unwrap(), Severity::Note);
        assert_eq!(Severity::from_str("SUGGESTION").unwrap(), Severity::Note);
    }

    // Warning aliases
    #[test]
    fn test_parse_warning() {
        assert_eq!(Severity::from_str("warning").unwrap(), Severity::Warning);
        assert_eq!(Severity::from_str("WARNING").unwrap(), Severity::Warning);
    }

    #[test]
    fn test_parse_warn_alias() {
        assert_eq!(Severity::from_str("warn").unwrap(), Severity::Warning);
        assert_eq!(Severity::from_str("WARN").unwrap(), Severity::Warning);
    }

    // Error aliases
    #[test]
    fn test_parse_error() {
        assert_eq!(Severity::from_str("error").unwrap(), Severity::Error);
        assert_eq!(Severity::from_str("ERROR").unwrap(), Severity::Error);
    }

    #[test]
    fn test_parse_err_alias() {
        assert_eq!(Severity::from_str("err").unwrap(), Severity::Error);
        assert_eq!(Severity::from_str("ERR").unwrap(), Severity::Error);
    }

    // Fatal aliases
    #[test]
    fn test_parse_fatal() {
        assert_eq!(Severity::from_str("fatal").unwrap(), Severity::Fatal);
        assert_eq!(Severity::from_str("FATAL").unwrap(), Severity::Fatal);
    }

    #[test]
    fn test_parse_critical_alias() {
        assert_eq!(Severity::from_str("critical").unwrap(), Severity::Fatal);
        assert_eq!(Severity::from_str("CRITICAL").unwrap(), Severity::Fatal);
    }

    #[test]
    fn test_parse_fail_alias() {
        assert_eq!(Severity::from_str("fail").unwrap(), Severity::Fatal);
        assert_eq!(Severity::from_str("FAIL").unwrap(), Severity::Fatal);
    }

    // Standard FromStr trait
    #[test]
    fn test_std_from_str_trait() {
        assert_eq!(Severity::from_str("warning"), Ok(Severity::Warning));
        assert_eq!("error".parse::<Severity>(), Ok(Severity::Error));
    }
}

// =============================================================================
// at_least and at_most Comparison Tests (10 tests)
// =============================================================================

mod comparison_methods {
    use super::*;

    #[test]
    fn test_at_least_same_severity() {
        assert!(Severity::Warning.at_least(Severity::Warning));
        assert!(Severity::Error.at_least(Severity::Error));
        assert!(Severity::Hint.at_least(Severity::Hint));
    }

    #[test]
    fn test_at_least_greater_severity() {
        assert!(Severity::Error.at_least(Severity::Warning));
        assert!(Severity::Fatal.at_least(Severity::Error));
        assert!(Severity::Warning.at_least(Severity::Note));
    }

    #[test]
    fn test_at_least_lesser_severity_fails() {
        assert!(!Severity::Warning.at_least(Severity::Error));
        assert!(!Severity::Note.at_least(Severity::Warning));
        assert!(!Severity::Hint.at_least(Severity::Fatal));
    }

    #[test]
    fn test_at_least_with_fatal() {
        // Only Fatal is at least Fatal
        assert!(Severity::Fatal.at_least(Severity::Fatal));
        assert!(!Severity::Error.at_least(Severity::Fatal));
        assert!(!Severity::Warning.at_least(Severity::Fatal));
    }

    #[test]
    fn test_at_least_with_hint() {
        // Everything is at least Hint
        assert!(Severity::Hint.at_least(Severity::Hint));
        assert!(Severity::Note.at_least(Severity::Hint));
        assert!(Severity::Warning.at_least(Severity::Hint));
        assert!(Severity::Error.at_least(Severity::Hint));
        assert!(Severity::Fatal.at_least(Severity::Hint));
    }

    #[test]
    fn test_at_most_same_severity() {
        assert!(Severity::Warning.at_most(Severity::Warning));
        assert!(Severity::Error.at_most(Severity::Error));
        assert!(Severity::Hint.at_most(Severity::Hint));
    }

    #[test]
    fn test_at_most_lesser_severity() {
        assert!(Severity::Hint.at_most(Severity::Note));
        assert!(Severity::Note.at_most(Severity::Warning));
        assert!(Severity::Warning.at_most(Severity::Error));
    }

    #[test]
    fn test_at_most_greater_severity_fails() {
        assert!(!Severity::Error.at_most(Severity::Warning));
        assert!(!Severity::Warning.at_most(Severity::Note));
        assert!(!Severity::Fatal.at_most(Severity::Hint));
    }

    #[test]
    fn test_at_most_with_hint() {
        // Only Hint is at most Hint
        assert!(Severity::Hint.at_most(Severity::Hint));
        assert!(!Severity::Note.at_most(Severity::Hint));
        assert!(!Severity::Warning.at_most(Severity::Hint));
    }

    #[test]
    fn test_at_most_with_fatal() {
        // Everything is at most Fatal
        assert!(Severity::Hint.at_most(Severity::Fatal));
        assert!(Severity::Note.at_most(Severity::Fatal));
        assert!(Severity::Warning.at_most(Severity::Fatal));
        assert!(Severity::Error.at_most(Severity::Fatal));
        assert!(Severity::Fatal.at_most(Severity::Fatal));
    }
}

// =============================================================================
// is_problem and is_blocking Tests (8 tests)
// =============================================================================

mod problem_and_blocking {
    use super::*;

    #[test]
    fn test_is_problem_hint() {
        assert!(!Severity::Hint.is_problem());
    }

    #[test]
    fn test_is_problem_note() {
        assert!(!Severity::Note.is_problem());
    }

    #[test]
    fn test_is_problem_warning_and_above() {
        assert!(Severity::Warning.is_problem());
        assert!(Severity::Error.is_problem());
        assert!(Severity::Fatal.is_problem());
    }

    #[test]
    fn test_is_problem_all_severities() {
        let results = [
            Severity::Hint.is_problem(),
            Severity::Note.is_problem(),
            Severity::Warning.is_problem(),
            Severity::Error.is_problem(),
            Severity::Fatal.is_problem(),
        ];
        assert_eq!(results, [false, false, true, true, true]);
    }

    #[test]
    fn test_is_blocking_warning() {
        assert!(!Severity::Warning.is_blocking());
    }

    #[test]
    fn test_is_blocking_error_and_fatal() {
        assert!(Severity::Error.is_blocking());
        assert!(Severity::Fatal.is_blocking());
    }

    #[test]
    fn test_is_blocking_hint_and_note() {
        assert!(!Severity::Hint.is_blocking());
        assert!(!Severity::Note.is_blocking());
    }

    #[test]
    fn test_is_blocking_all_severities() {
        let results = [
            Severity::Hint.is_blocking(),
            Severity::Note.is_blocking(),
            Severity::Warning.is_blocking(),
            Severity::Error.is_blocking(),
            Severity::Fatal.is_blocking(),
        ];
        assert_eq!(results, [false, false, false, true, true]);
    }
}

// =============================================================================
// Display and as_str Formatting Tests (6 tests)
// =============================================================================

mod formatting {
    use super::*;

    #[test]
    fn test_as_str_all_variants() {
        assert_eq!(Severity::Hint.as_str(), "hint");
        assert_eq!(Severity::Note.as_str(), "note");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Fatal.as_str(), "fatal");
    }

    #[test]
    fn test_display_all_variants() {
        assert_eq!(format!("{}", Severity::Hint), "hint");
        assert_eq!(format!("{}", Severity::Note), "note");
        assert_eq!(format!("{}", Severity::Warning), "warning");
        assert_eq!(format!("{}", Severity::Error), "error");
        assert_eq!(format!("{}", Severity::Fatal), "fatal");
    }

    #[test]
    fn test_display_matches_as_str() {
        for severity in [
            Severity::Hint,
            Severity::Note,
            Severity::Warning,
            Severity::Error,
            Severity::Fatal,
        ] {
            assert_eq!(format!("{}", severity), severity.as_str());
        }
    }

    #[test]
    fn test_icon_all_variants() {
        assert_eq!(Severity::Hint.icon(), "💡");
        assert_eq!(Severity::Note.icon(), "📝");
        assert_eq!(Severity::Warning.icon(), "⚠️");
        assert_eq!(Severity::Error.icon(), "❌");
        assert_eq!(Severity::Fatal.icon(), "🔥");
    }

    #[test]
    fn test_debug_impl() {
        // Verify Debug trait is implemented and works
        assert!(format!("{:?}", Severity::Warning).contains("Warning"));
        assert!(format!("{:?}", Severity::Error).contains("Error"));
    }

    #[test]
    fn test_clone_impl() {
        let original = Severity::Error;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}

// =============================================================================
// SeverityThreshold Functionality Tests (10 tests)
// =============================================================================

mod threshold_functionality {
    use super::*;

    #[test]
    fn test_threshold_minimum_constructor() {
        let threshold = SeverityThreshold::minimum(Severity::Warning);
        assert_eq!(threshold.min_severity(), Severity::Warning);
    }

    #[test]
    fn test_threshold_allows_at_minimum() {
        let threshold = SeverityThreshold::minimum(Severity::Warning);
        assert!(threshold.allows(Severity::Warning));
    }

    #[test]
    fn test_threshold_allows_above_minimum() {
        let threshold = SeverityThreshold::minimum(Severity::Warning);
        assert!(threshold.allows(Severity::Error));
        assert!(threshold.allows(Severity::Fatal));
    }

    #[test]
    fn test_threshold_rejects_below_minimum() {
        let threshold = SeverityThreshold::minimum(Severity::Warning);
        assert!(!threshold.allows(Severity::Hint));
        assert!(!threshold.allows(Severity::Note));
    }

    #[test]
    fn test_threshold_default_allows_all() {
        let threshold = SeverityThreshold::default();
        assert!(threshold.allows(Severity::Hint));
        assert!(threshold.allows(Severity::Note));
        assert!(threshold.allows(Severity::Warning));
        assert!(threshold.allows(Severity::Error));
        assert!(threshold.allows(Severity::Fatal));
    }

    #[test]
    fn test_threshold_default_minimum_is_hint() {
        let threshold = SeverityThreshold::default();
        assert_eq!(threshold.min_severity(), Severity::Hint);
    }

    #[test]
    fn test_threshold_at_error_level() {
        let threshold = SeverityThreshold::minimum(Severity::Error);
        assert!(!threshold.allows(Severity::Hint));
        assert!(!threshold.allows(Severity::Note));
        assert!(!threshold.allows(Severity::Warning));
        assert!(threshold.allows(Severity::Error));
        assert!(threshold.allows(Severity::Fatal));
    }

    #[test]
    fn test_threshold_at_fatal_level() {
        let threshold = SeverityThreshold::minimum(Severity::Fatal);
        assert!(!threshold.allows(Severity::Hint));
        assert!(!threshold.allows(Severity::Note));
        assert!(!threshold.allows(Severity::Warning));
        assert!(!threshold.allows(Severity::Error));
        assert!(threshold.allows(Severity::Fatal));
    }

    #[test]
    fn test_threshold_equality() {
        let t1 = SeverityThreshold::minimum(Severity::Warning);
        let t2 = SeverityThreshold::minimum(Severity::Warning);
        let t3 = SeverityThreshold::minimum(Severity::Error);

        assert_eq!(t1, t2);
        assert_ne!(t1, t3);
    }

    #[test]
    fn test_threshold_debug_impl() {
        let threshold = SeverityThreshold::minimum(Severity::Warning);
        let debug_str = format!("{:?}", threshold);
        assert!(debug_str.contains("SeverityThreshold"));
        assert!(debug_str.contains("Warning"));
    }
}

// =============================================================================
// Edge Cases and Error Conditions Tests (6 tests)
// =============================================================================

mod edge_cases_and_errors {
    use super::*;

    #[test]
    fn test_parse_error_unknown_string() {
        let result = Severity::from_str("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_message() {
        let err = Severity::from_str("invalid_severity").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("invalid_severity"));
        assert!(msg.contains("Unknown severity level"));
    }

    #[test]
    fn test_parse_error_input_accessor() {
        let err = Severity::from_str("bad_input").unwrap_err();
        assert_eq!(err.input(), "bad_input");
    }

    #[test]
    fn test_parse_empty_string() {
        let result = Severity::from_str("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown severity level"));
    }

    #[test]
    fn test_parse_whitespace_string() {
        let result = Severity::from_str("   warning   ");
        assert!(result.is_err()); // Whitespace is not trimmed
    }

    #[test]
    fn test_parse_with_underscores() {
        // These should fail - underscores not supported
        assert!(Severity::from_str("some_error").is_err());
        assert!(Severity::from_str("high_priority").is_err());
    }
}

// =============================================================================
// Additional Tests for Coverage
// =============================================================================

mod additional_coverage {
    use super::*;

    #[test]
    fn test_severity_hash_consistency() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Severity::Warning);
        set.insert(Severity::Error);
        set.insert(Severity::Warning); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_severity_copy_trait() {
        let s1 = Severity::Warning;
        let s2 = s1; // Copy
        let s3 = s1; // Copy again

        assert_eq!(s1, s2);
        assert_eq!(s2, s3);
    }

    #[test]
    fn test_threshold_copy_trait() {
        let t1 = SeverityThreshold::minimum(Severity::Error);
        let t2 = t1; // Copy
        let t3 = t1; // Copy again

        assert_eq!(t1, t2);
        assert_eq!(t2, t3);
    }

    #[test]
    fn test_all_severity_values_in_order() {
        let severities = [
            Severity::Hint,
            Severity::Note,
            Severity::Warning,
            Severity::Error,
            Severity::Fatal,
        ];

        // Verify each is strictly less than the next
        for i in 0..severities.len() - 1 {
            assert!(severities[i] < severities[i + 1]);
        }
    }

    #[test]
    fn test_roundtrip_str_to_severity_to_str() {
        let inputs = ["hint", "note", "warning", "error", "fatal"];

        for input in inputs {
            let severity = Severity::from_str(input).unwrap();
            assert_eq!(severity.as_str(), input);
        }
    }

    #[test]
    fn test_threshold_clone() {
        let original = SeverityThreshold::minimum(Severity::Error);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
