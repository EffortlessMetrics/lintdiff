//! Comprehensive BDD tests for lintdiff-diagnostic-level crate.
//!
//! Test coverage:
//! 1. DiagnosticLevel enum variants and methods (20 tests)
//! 2. Parsing functions - generic (10 tests)
//! 3. Parsing functions - rustc (8 tests)
//! 4. Parsing functions - eslint (10 tests)
//! 5. Numeric level parsing (8 tests)
//! 6. Canonical severity conversion (8 tests)
//! 7. Helper functions - is_problem, is_error, is_warning (12 tests)
//! 8. DiagnosticLevelParser struct (15 tests)
//! 9. Edge cases and error handling (10 tests)
//! 10. Property-based tests with proptest (6 tests)
//! 11. Display/Debug implementations (6 tests)
//! 12. Serde serialization (conditional) (6 tests)

use lintdiff_diagnostic_level::{
    from_number, is_error, is_problem, is_warning, parse_level, parse_level_eslint,
    parse_level_rustc, to_canonical, CanonicalSeverity, DiagnosticLevel, DiagnosticLevelParser,
};

// =============================================================================
// 1. DiagnosticLevel enum variants and methods (20 tests)
// =============================================================================

mod diagnostic_level_variants {
    use super::*;

    #[test]
    fn hint_variant_has_level_zero() {
        assert_eq!(DiagnosticLevel::Hint.level(), 0);
    }

    #[test]
    fn note_variant_has_level_one() {
        assert_eq!(DiagnosticLevel::Note.level(), 1);
    }

    #[test]
    fn warning_variant_has_level_two() {
        assert_eq!(DiagnosticLevel::Warning.level(), 2);
    }

    #[test]
    fn error_variant_has_level_three() {
        assert_eq!(DiagnosticLevel::Error.level(), 3);
    }

    #[test]
    fn fatal_variant_has_level_four() {
        assert_eq!(DiagnosticLevel::Fatal.level(), 4);
    }

    #[test]
    fn unknown_variant_has_level_255() {
        assert_eq!(DiagnosticLevel::Unknown.level(), 255);
    }

    #[test]
    fn hint_as_str_returns_hint() {
        assert_eq!(DiagnosticLevel::Hint.as_str(), "hint");
    }

    #[test]
    fn note_as_str_returns_note() {
        assert_eq!(DiagnosticLevel::Note.as_str(), "note");
    }

    #[test]
    fn warning_as_str_returns_warning() {
        assert_eq!(DiagnosticLevel::Warning.as_str(), "warning");
    }

    #[test]
    fn error_as_str_returns_error() {
        assert_eq!(DiagnosticLevel::Error.as_str(), "error");
    }

    #[test]
    fn fatal_as_str_returns_fatal() {
        assert_eq!(DiagnosticLevel::Fatal.as_str(), "fatal");
    }

    #[test]
    fn unknown_as_str_returns_unknown() {
        assert_eq!(DiagnosticLevel::Unknown.as_str(), "unknown");
    }

    #[test]
    fn hint_icon_is_lightbulb() {
        assert_eq!(DiagnosticLevel::Hint.icon(), "💡");
    }

    #[test]
    fn note_icon_is_memo() {
        assert_eq!(DiagnosticLevel::Note.icon(), "📝");
    }

    #[test]
    fn warning_icon_is_warning_sign() {
        assert_eq!(DiagnosticLevel::Warning.icon(), "⚠️");
    }

    #[test]
    fn error_icon_is_x_mark() {
        assert_eq!(DiagnosticLevel::Error.icon(), "❌");
    }

    #[test]
    fn fatal_icon_is_skull() {
        assert_eq!(DiagnosticLevel::Fatal.icon(), "💀");
    }

    #[test]
    fn unknown_icon_is_question_mark() {
        assert_eq!(DiagnosticLevel::Unknown.icon(), "❓");
    }

    #[test]
    fn default_is_unknown() {
        assert_eq!(DiagnosticLevel::default(), DiagnosticLevel::Unknown);
    }

    #[test]
    fn ordering_is_correct() {
        assert!(DiagnosticLevel::Hint < DiagnosticLevel::Note);
        assert!(DiagnosticLevel::Note < DiagnosticLevel::Warning);
        assert!(DiagnosticLevel::Warning < DiagnosticLevel::Error);
        assert!(DiagnosticLevel::Error < DiagnosticLevel::Fatal);
        // Unknown has highest numeric value (255)
        assert!(DiagnosticLevel::Fatal < DiagnosticLevel::Unknown);
    }
}

// =============================================================================
// 2. Parsing functions - generic (10 tests)
// =============================================================================

mod parse_level_generic {
    use super::*;

    #[test]
    fn parses_error_lowercase() {
        assert_eq!(parse_level("error"), DiagnosticLevel::Error);
    }

    #[test]
    fn parses_error_uppercase() {
        assert_eq!(parse_level("ERROR"), DiagnosticLevel::Error);
    }

    #[test]
    fn parses_warning_mixed_case() {
        assert_eq!(parse_level("Warning"), DiagnosticLevel::Warning);
    }

    #[test]
    fn parses_warn_as_warning() {
        assert_eq!(parse_level("warn"), DiagnosticLevel::Warning);
    }

    #[test]
    fn parses_hint() {
        assert_eq!(parse_level("hint"), DiagnosticLevel::Hint);
    }

    #[test]
    fn parses_suggestion_as_hint() {
        assert_eq!(parse_level("suggestion"), DiagnosticLevel::Hint);
    }

    #[test]
    fn parses_note() {
        assert_eq!(parse_level("note"), DiagnosticLevel::Note);
    }

    #[test]
    fn parses_info_as_note() {
        assert_eq!(parse_level("info"), DiagnosticLevel::Note);
    }

    #[test]
    fn parses_fatal() {
        assert_eq!(parse_level("fatal"), DiagnosticLevel::Fatal);
    }

    #[test]
    fn returns_unknown_for_invalid() {
        assert_eq!(parse_level("invalid"), DiagnosticLevel::Unknown);
    }
}

// =============================================================================
// 3. Parsing functions - rustc (8 tests)
// =============================================================================

mod parse_level_rustc_tests {
    use super::*;

    #[test]
    fn parses_rustc_error() {
        assert_eq!(parse_level_rustc("error"), DiagnosticLevel::Error);
    }

    #[test]
    fn parses_rustc_warning() {
        assert_eq!(parse_level_rustc("warning"), DiagnosticLevel::Warning);
    }

    #[test]
    fn parses_rustc_note() {
        assert_eq!(parse_level_rustc("note"), DiagnosticLevel::Note);
    }

    #[test]
    fn parses_rustc_help_as_hint() {
        assert_eq!(parse_level_rustc("help"), DiagnosticLevel::Hint);
    }

    #[test]
    fn parses_rustc_fatal() {
        assert_eq!(parse_level_rustc("fatal"), DiagnosticLevel::Fatal);
    }

    #[test]
    fn parses_rustc_fatal_error() {
        assert_eq!(parse_level_rustc("fatal-error"), DiagnosticLevel::Fatal);
    }

    #[test]
    fn parses_rustc_case_insensitive() {
        assert_eq!(parse_level_rustc("ERROR"), DiagnosticLevel::Error);
        assert_eq!(parse_level_rustc("Warning"), DiagnosticLevel::Warning);
    }

    #[test]
    fn returns_unknown_for_invalid_rustc() {
        assert_eq!(parse_level_rustc("bug"), DiagnosticLevel::Unknown);
    }
}

// =============================================================================
// 4. Parsing functions - eslint (10 tests)
// =============================================================================

mod parse_level_eslint_tests {
    use super::*;

    #[test]
    fn parses_eslint_error() {
        assert_eq!(parse_level_eslint("error"), DiagnosticLevel::Error);
    }

    #[test]
    fn parses_eslint_error_numeric() {
        assert_eq!(parse_level_eslint("2"), DiagnosticLevel::Error);
    }

    #[test]
    fn parses_eslint_warning() {
        assert_eq!(parse_level_eslint("warning"), DiagnosticLevel::Warning);
    }

    #[test]
    fn parses_eslint_warn() {
        assert_eq!(parse_level_eslint("warn"), DiagnosticLevel::Warning);
    }

    #[test]
    fn parses_eslint_warning_numeric() {
        assert_eq!(parse_level_eslint("1"), DiagnosticLevel::Warning);
    }

    #[test]
    fn parses_eslint_off_as_unknown() {
        assert_eq!(parse_level_eslint("off"), DiagnosticLevel::Unknown);
    }

    #[test]
    fn parses_eslint_off_numeric() {
        assert_eq!(parse_level_eslint("0"), DiagnosticLevel::Unknown);
    }

    #[test]
    fn parses_eslint_info_as_note() {
        assert_eq!(parse_level_eslint("info"), DiagnosticLevel::Note);
    }

    #[test]
    fn parses_eslint_hint() {
        assert_eq!(parse_level_eslint("hint"), DiagnosticLevel::Hint);
    }

    #[test]
    fn parses_eslint_case_insensitive() {
        assert_eq!(parse_level_eslint("ERROR"), DiagnosticLevel::Error);
        assert_eq!(parse_level_eslint("Warning"), DiagnosticLevel::Warning);
    }
}

// =============================================================================
// 5. Numeric level parsing (8 tests)
// =============================================================================

mod numeric_parsing {
    use super::*;

    #[test]
    fn from_number_zero_is_hint() {
        assert_eq!(from_number(0), DiagnosticLevel::Hint);
    }

    #[test]
    fn from_number_one_is_note() {
        assert_eq!(from_number(1), DiagnosticLevel::Note);
    }

    #[test]
    fn from_number_two_is_warning() {
        assert_eq!(from_number(2), DiagnosticLevel::Warning);
    }

    #[test]
    fn from_number_three_is_error() {
        assert_eq!(from_number(3), DiagnosticLevel::Error);
    }

    #[test]
    fn from_number_four_is_fatal() {
        assert_eq!(from_number(4), DiagnosticLevel::Fatal);
    }

    #[test]
    fn from_number_five_is_unknown() {
        assert_eq!(from_number(5), DiagnosticLevel::Unknown);
    }

    #[test]
    fn from_number_large_is_unknown() {
        assert_eq!(from_number(255), DiagnosticLevel::Unknown);
        assert_eq!(from_number(100), DiagnosticLevel::Unknown);
    }

    #[test]
    fn from_u8_trait() {
        assert_eq!(DiagnosticLevel::from(0u8), DiagnosticLevel::Hint);
        assert_eq!(DiagnosticLevel::from(3u8), DiagnosticLevel::Error);
    }
}

// =============================================================================
// 6. Canonical severity conversion (8 tests)
// =============================================================================

mod canonical_severity_conversion {
    use super::*;

    #[test]
    fn hint_to_canonical_is_info() {
        assert_eq!(to_canonical(&DiagnosticLevel::Hint), CanonicalSeverity::Info);
    }

    #[test]
    fn note_to_canonical_is_info() {
        assert_eq!(to_canonical(&DiagnosticLevel::Note), CanonicalSeverity::Info);
    }

    #[test]
    fn warning_to_canonical_is_warning() {
        assert_eq!(to_canonical(&DiagnosticLevel::Warning), CanonicalSeverity::Warning);
    }

    #[test]
    fn error_to_canonical_is_error() {
        assert_eq!(to_canonical(&DiagnosticLevel::Error), CanonicalSeverity::Error);
    }

    #[test]
    fn fatal_to_canonical_is_error() {
        assert_eq!(to_canonical(&DiagnosticLevel::Fatal), CanonicalSeverity::Error);
    }

    #[test]
    fn unknown_to_canonical_is_unknown() {
        assert_eq!(to_canonical(&DiagnosticLevel::Unknown), CanonicalSeverity::Unknown);
    }

    #[test]
    fn canonical_severity_as_str() {
        assert_eq!(CanonicalSeverity::Unknown.as_str(), "unknown");
        assert_eq!(CanonicalSeverity::Info.as_str(), "info");
        assert_eq!(CanonicalSeverity::Warning.as_str(), "warning");
        assert_eq!(CanonicalSeverity::Error.as_str(), "error");
    }

    #[test]
    fn canonical_severity_default_is_unknown() {
        assert_eq!(CanonicalSeverity::default(), CanonicalSeverity::Unknown);
    }
}

// =============================================================================
// 7. Helper functions - is_problem, is_error, is_warning (12 tests)
// =============================================================================

mod helper_functions {
    use super::*;

    // is_problem tests
    #[test]
    fn hint_is_not_problem() {
        assert!(!is_problem(&DiagnosticLevel::Hint));
    }

    #[test]
    fn note_is_not_problem() {
        assert!(!is_problem(&DiagnosticLevel::Note));
    }

    #[test]
    fn warning_is_problem() {
        assert!(is_problem(&DiagnosticLevel::Warning));
    }

    #[test]
    fn error_is_problem() {
        assert!(is_problem(&DiagnosticLevel::Error));
    }

    // is_error tests
    #[test]
    fn warning_is_not_error() {
        assert!(!is_error(&DiagnosticLevel::Warning));
    }

    #[test]
    fn error_is_error() {
        assert!(is_error(&DiagnosticLevel::Error));
    }

    #[test]
    fn fatal_is_error() {
        assert!(is_error(&DiagnosticLevel::Fatal));
    }

    #[test]
    fn unknown_is_not_error() {
        assert!(!is_error(&DiagnosticLevel::Unknown));
    }

    // is_warning tests
    #[test]
    fn warning_is_warning() {
        assert!(is_warning(&DiagnosticLevel::Warning));
    }

    #[test]
    fn error_is_not_warning() {
        assert!(!is_warning(&DiagnosticLevel::Error));
    }

    #[test]
    fn hint_is_not_warning() {
        assert!(!is_warning(&DiagnosticLevel::Hint));
    }
}

// =============================================================================
// 8. DiagnosticLevelParser struct (15 tests)
// =============================================================================

mod diagnostic_level_parser {
    use super::*;

    #[test]
    fn parser_new_creates_empty_parser() {
        let parser = DiagnosticLevelParser::new();
        assert_eq!(parser.mapping_count(), 0);
    }

    #[test]
    fn parser_default_creates_empty_parser() {
        let parser = DiagnosticLevelParser::default();
        assert_eq!(parser.mapping_count(), 0);
    }

    #[test]
    fn parser_adds_custom_mapping() {
        let mut parser = DiagnosticLevelParser::new();
        parser.with_custom_mapping("critical", DiagnosticLevel::Fatal);

        assert_eq!(parser.parse("critical"), DiagnosticLevel::Fatal);
    }

    #[test]
    fn parser_custom_mapping_is_case_insensitive() {
        let mut parser = DiagnosticLevelParser::new();
        parser.with_custom_mapping("Critical", DiagnosticLevel::Fatal);

        assert_eq!(parser.parse("critical"), DiagnosticLevel::Fatal);
        assert_eq!(parser.parse("CRITICAL"), DiagnosticLevel::Fatal);
    }

    #[test]
    fn parser_falls_back_to_default() {
        let mut parser = DiagnosticLevelParser::new();
        parser.with_custom_mapping("critical", DiagnosticLevel::Fatal);

        // Custom mapping works
        assert_eq!(parser.parse("critical"), DiagnosticLevel::Fatal);
        // Default parsing still works
        assert_eq!(parser.parse("error"), DiagnosticLevel::Error);
        assert_eq!(parser.parse("warning"), DiagnosticLevel::Warning);
    }

    #[test]
    fn parser_returns_unknown_for_unrecognized() {
        let parser = DiagnosticLevelParser::new();
        assert_eq!(parser.parse("unrecognized"), DiagnosticLevel::Unknown);
    }

    #[test]
    fn parser_has_mapping_returns_true_for_custom() {
        let mut parser = DiagnosticLevelParser::new();
        parser.with_custom_mapping("critical", DiagnosticLevel::Fatal);

        assert!(parser.has_mapping("critical"));
    }

    #[test]
    fn parser_has_mapping_returns_false_for_default() {
        let parser = DiagnosticLevelParser::new();

        // "error" is parsed by default logic, not a custom mapping
        assert!(!parser.has_mapping("error"));
    }

    #[test]
    fn parser_remove_mapping() {
        let mut parser = DiagnosticLevelParser::new();
        parser.with_custom_mapping("critical", DiagnosticLevel::Fatal);

        assert!(parser.has_mapping("critical"));
        parser.remove_mapping("critical");
        assert!(!parser.has_mapping("critical"));
    }

    #[test]
    fn parser_mapping_count() {
        let mut parser = DiagnosticLevelParser::new();
        assert_eq!(parser.mapping_count(), 0);

        parser.with_custom_mapping("critical", DiagnosticLevel::Fatal);
        assert_eq!(parser.mapping_count(), 1);

        parser.with_custom_mapping("severe", DiagnosticLevel::Error);
        assert_eq!(parser.mapping_count(), 2);
    }

    #[test]
    fn parser_clear_mappings() {
        let mut parser = DiagnosticLevelParser::new();
        parser.with_custom_mapping("critical", DiagnosticLevel::Fatal);
        parser.with_custom_mapping("severe", DiagnosticLevel::Error);

        assert_eq!(parser.mapping_count(), 2);
        parser.clear_mappings();
        assert_eq!(parser.mapping_count(), 0);
    }

    #[test]
    fn parser_chained_builders() {
        let mut parser = DiagnosticLevelParser::new();
        parser
            .with_custom_mapping("critical", DiagnosticLevel::Fatal)
            .with_custom_mapping("severe", DiagnosticLevel::Error)
            .with_custom_mapping("minor", DiagnosticLevel::Warning);

        assert_eq!(parser.parse("critical"), DiagnosticLevel::Fatal);
        assert_eq!(parser.parse("severe"), DiagnosticLevel::Error);
        assert_eq!(parser.parse("minor"), DiagnosticLevel::Warning);
    }

    #[test]
    fn parser_overwrites_existing_mapping() {
        let mut parser = DiagnosticLevelParser::new();
        parser.with_custom_mapping("level", DiagnosticLevel::Error);
        parser.with_custom_mapping("level", DiagnosticLevel::Warning);

        assert_eq!(parser.parse("level"), DiagnosticLevel::Warning);
        assert_eq!(parser.mapping_count(), 1);
    }

    #[test]
    fn parser_handles_empty_string() {
        let parser = DiagnosticLevelParser::new();
        assert_eq!(parser.parse(""), DiagnosticLevel::Unknown);
    }

    #[test]
    fn parser_handles_whitespace() {
        let parser = DiagnosticLevelParser::new();
        assert_eq!(parser.parse("  error  "), DiagnosticLevel::Unknown);
    }
}

// =============================================================================
// 9. Edge cases and error handling (10 tests)
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn empty_string_returns_unknown() {
        assert_eq!(parse_level(""), DiagnosticLevel::Unknown);
    }

    #[test]
    fn whitespace_only_returns_unknown() {
        assert_eq!(parse_level("   "), DiagnosticLevel::Unknown);
    }

    #[test]
    fn leading_trailing_whitespace_returns_unknown() {
        // The parser doesn't trim whitespace
        assert_eq!(parse_level("  error  "), DiagnosticLevel::Unknown);
    }

    #[test]
    fn numeric_string_not_parsed_by_parse_level() {
        // parse_level doesn't handle numeric strings
        assert_eq!(parse_level("2"), DiagnosticLevel::Unknown);
    }

    #[test]
    fn special_characters_return_unknown() {
        assert_eq!(parse_level("error!"), DiagnosticLevel::Unknown);
        assert_eq!(parse_level("@error"), DiagnosticLevel::Unknown);
    }

    #[test]
    fn unicode_strings_handled_gracefully() {
        assert_eq!(parse_level("érror"), DiagnosticLevel::Unknown);
        assert_eq!(parse_level("错误"), DiagnosticLevel::Unknown);
    }

    #[test]
    fn very_long_string_handled() {
        let long_string = "error".repeat(1000);
        assert_eq!(parse_level(&long_string), DiagnosticLevel::Unknown);
    }

    #[test]
    fn null_byte_in_string() {
        assert_eq!(parse_level("error\0"), DiagnosticLevel::Unknown);
    }

    #[test]
    fn diagnostic_level_parse_returns_error_for_invalid() {
        let result = DiagnosticLevel::parse("invalid");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().input(),
            "invalid"
        );
    }

    #[test]
    fn diagnostic_level_parse_error_display() {
        let err = DiagnosticLevel::parse("bad").unwrap_err();
        assert!(err.to_string().contains("bad"));
    }
}

// =============================================================================
// 10. Property-based tests with proptest (6 tests)
// =============================================================================

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn from_number_roundtrip(n in 0u8..=4) {
            let level = from_number(n);
            prop_assert_eq!(level.level(), n);
        }

        #[test]
        fn parse_level_case_insensitive(s in "[eE][rR][rR][oO][rR]") {
            prop_assert_eq!(parse_level(&s), DiagnosticLevel::Error);
        }

        #[test]
        fn is_problem_consistent_with_level(n in 0u8..=5) {
            let level = from_number(n);
            let is_problem_result = is_problem(&level);
            let expected = matches!(level, DiagnosticLevel::Warning | DiagnosticLevel::Error | DiagnosticLevel::Fatal);
            prop_assert_eq!(is_problem_result, expected);
        }

        #[test]
        fn is_error_consistent_with_level(n in 0u8..=5) {
            let level = from_number(n);
            let is_error_result = is_error(&level);
            let expected = matches!(level, DiagnosticLevel::Error | DiagnosticLevel::Fatal);
            prop_assert_eq!(is_error_result, expected);
        }

        #[test]
        fn to_canonical_never_panics(s in ".*") {
            let level = parse_level(&s);
            let _canonical = to_canonical(&level);
        }

        #[test]
        fn parser_never_panics(s in ".*") {
            let parser = DiagnosticLevelParser::new();
            let _result = parser.parse(&s);
        }
    }
}

// =============================================================================
// 11. Display/Debug implementations (6 tests)
// =============================================================================

mod display_debug_impls {
    use super::*;

    #[test]
    fn diagnostic_level_display() {
        assert_eq!(format!("{}", DiagnosticLevel::Error), "error");
        assert_eq!(format!("{}", DiagnosticLevel::Warning), "warning");
        assert_eq!(format!("{}", DiagnosticLevel::Hint), "hint");
        assert_eq!(format!("{}", DiagnosticLevel::Note), "note");
        assert_eq!(format!("{}", DiagnosticLevel::Fatal), "fatal");
        assert_eq!(format!("{}", DiagnosticLevel::Unknown), "unknown");
    }

    #[test]
    fn diagnostic_level_debug() {
        assert!(format!("{:?}", DiagnosticLevel::Error).contains("Error"));
        assert!(format!("{:?}", DiagnosticLevel::Warning).contains("Warning"));
    }

    #[test]
    fn canonical_severity_display() {
        assert_eq!(format!("{}", CanonicalSeverity::Error), "error");
        assert_eq!(format!("{}", CanonicalSeverity::Warning), "warning");
        assert_eq!(format!("{}", CanonicalSeverity::Info), "info");
        assert_eq!(format!("{}", CanonicalSeverity::Unknown), "unknown");
    }

    #[test]
    fn canonical_severity_debug() {
        assert!(format!("{:?}", CanonicalSeverity::Error).contains("Error"));
    }

    #[test]
    fn diagnostic_level_parser_debug() {
        let parser = DiagnosticLevelParser::new();
        let debug_str = format!("{:?}", parser);
        assert!(debug_str.contains("DiagnosticLevelParser"));
    }

    #[test]
    fn parse_error_display() {
        let err = lintdiff_diagnostic_level::DiagnosticLevelParseError::new("test");
        assert!(err.to_string().contains("test"));
    }
}

// =============================================================================
// 12. Serde serialization (conditional) (6 tests)
// =============================================================================

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    #[test]
    fn diagnostic_level_serializes_to_string() {
        let level = DiagnosticLevel::Error;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"Error\"");
    }

    #[test]
    fn diagnostic_level_deserializes_from_string() {
        let level: DiagnosticLevel = serde_json::from_str("\"Warning\"").unwrap();
        assert_eq!(level, DiagnosticLevel::Warning);
    }

    #[test]
    fn canonical_severity_serializes_to_string() {
        let severity = CanonicalSeverity::Warning;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"Warning\"");
    }

    #[test]
    fn canonical_severity_deserializes_from_string() {
        let severity: CanonicalSeverity = serde_json::from_str("\"Error\"").unwrap();
        assert_eq!(severity, CanonicalSeverity::Error);
    }

    #[test]
    fn diagnostic_level_roundtrip_serde() {
        let original = DiagnosticLevel::Fatal;
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: DiagnosticLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn canonical_severity_roundtrip_serde() {
        let original = CanonicalSeverity::Info;
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: CanonicalSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }
}

// =============================================================================
// Additional integration-style tests
// =============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn full_workflow_generic_parsing() {
        // Parse various levels
        let error = parse_level("error");
        let warning = parse_level("warning");
        let hint = parse_level("hint");

        // Check their properties
        assert!(is_error(&error));
        assert!(!is_error(&warning));
        assert!(is_problem(&warning));
        assert!(!is_problem(&hint));

        // Convert to canonical
        assert_eq!(to_canonical(&error), CanonicalSeverity::Error);
        assert_eq!(to_canonical(&warning), CanonicalSeverity::Warning);
        assert_eq!(to_canonical(&hint), CanonicalSeverity::Info);
    }

    #[test]
    fn full_workflow_rustc_parsing() {
        let error = parse_level_rustc("error");
        let note = parse_level_rustc("note");
        let help = parse_level_rustc("help");

        assert!(is_error(&error));
        assert!(is_problem(&error));
        assert!(!is_problem(&note));
        assert!(note.is_info());
        assert!(help.is_info());
    }

    #[test]
    fn full_workflow_eslint_parsing() {
        let error = parse_level_eslint("2"); // numeric
        let warning = parse_level_eslint("1"); // numeric
        let off = parse_level_eslint("0"); // numeric

        assert!(is_error(&error));
        assert!(is_warning(&warning));
        assert!(!is_problem(&off));
        assert_eq!(off, DiagnosticLevel::Unknown);
    }

    #[test]
    fn custom_parser_workflow() {
        let mut parser = DiagnosticLevelParser::new();

        // Add custom mappings for a hypothetical linter
        parser
            .with_custom_mapping("blocker", DiagnosticLevel::Fatal)
            .with_custom_mapping("critical", DiagnosticLevel::Error)
            .with_custom_mapping("major", DiagnosticLevel::Warning)
            .with_custom_mapping("minor", DiagnosticLevel::Note)
            .with_custom_mapping("info", DiagnosticLevel::Hint);

        // Parse custom levels
        assert_eq!(parser.parse("blocker"), DiagnosticLevel::Fatal);
        assert_eq!(parser.parse("critical"), DiagnosticLevel::Error);
        assert_eq!(parser.parse("major"), DiagnosticLevel::Warning);

        // Verify properties
        assert!(is_error(&parser.parse("blocker")));
        assert!(is_error(&parser.parse("critical")));
        assert!(is_warning(&parser.parse("major")));
    }

    #[test]
    fn level_comparison_workflow() {
        // Ordering can be used for filtering
        let min_level = DiagnosticLevel::Warning;

        let levels = [
            DiagnosticLevel::Hint,
            DiagnosticLevel::Note,
            DiagnosticLevel::Warning,
            DiagnosticLevel::Error,
            DiagnosticLevel::Fatal,
        ];

        let filtered: Vec<_> = levels.iter().filter(|&l| l >= &min_level).collect();
        assert_eq!(filtered.len(), 3);
        assert!(filtered.contains(&&DiagnosticLevel::Warning));
        assert!(filtered.contains(&&DiagnosticLevel::Error));
        assert!(filtered.contains(&&DiagnosticLevel::Fatal));
    }
}
