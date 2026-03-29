//! BDD Tests for lintdiff-severity-map
//!
//! Comprehensive test suite covering:
//! - CanonicalSeverity enum behavior
//! - SeverityMapper functionality
//! - SeverityMapBuilder pattern
//! - Built-in linter mappings
//! - Convenience functions
//! - Edge cases and property-based tests

use lintdiff_severity_map::{
    is_error_level, is_problem_level, is_warning_level, map_severity, CanonicalSeverity,
    SeverityMapBuilder, SeverityMapper, SeverityParseError,
};

// =============================================================================
// Feature: CanonicalSeverity Enum
// =============================================================================

mod canonical_severity_feature {
    use super::*;

    // -------------------------------------------------------------------------
    // Scenario: Severity ordering
    // -------------------------------------------------------------------------

    #[test]
    fn severity_levels_are_ordered_correctly() {
        assert!(CanonicalSeverity::Unknown < CanonicalSeverity::Hint);
        assert!(CanonicalSeverity::Hint < CanonicalSeverity::Info);
        assert!(CanonicalSeverity::Info < CanonicalSeverity::Warning);
        assert!(CanonicalSeverity::Warning < CanonicalSeverity::Error);
    }

    #[test]
    fn severity_ordering_is_transitive() {
        // If A < B and B < C, then A < C
        let levels = [
            CanonicalSeverity::Unknown,
            CanonicalSeverity::Hint,
            CanonicalSeverity::Info,
            CanonicalSeverity::Warning,
            CanonicalSeverity::Error,
        ];
        for (i, &a) in levels.iter().enumerate() {
            for (j, &b) in levels.iter().enumerate() {
                if i < j {
                    assert!(a < b, "{:?} should be < {:?}", a, b);
                }
            }
        }
    }

    #[test]
    fn severity_equality_works() {
        assert_eq!(CanonicalSeverity::Error, CanonicalSeverity::Error);
        assert_eq!(CanonicalSeverity::Warning, CanonicalSeverity::Warning);
        assert_ne!(CanonicalSeverity::Error, CanonicalSeverity::Warning);
    }

    // -------------------------------------------------------------------------
    // Scenario: Parsing severity strings
    // -------------------------------------------------------------------------

    #[test]
    fn parse_error_variants() {
        let error_cases = [
            "error", "ERROR", "Error", "err", "ERR", "Err", "fatal", "FATAL", "Fatal", "critical",
            "CRITICAL", "fail", "FAIL", "Fail", "2",
        ];
        for case in error_cases {
            assert_eq!(
                CanonicalSeverity::parse(case),
                Ok(CanonicalSeverity::Error),
                "Failed to parse '{}' as Error",
                case
            );
        }
    }

    #[test]
    fn parse_warning_variants() {
        let warning_cases = ["warning", "WARNING", "Warning", "warn", "WARN", "Warn", "1"];
        for case in warning_cases {
            assert_eq!(
                CanonicalSeverity::parse(case),
                Ok(CanonicalSeverity::Warning),
                "Failed to parse '{}' as Warning",
                case
            );
        }
    }

    #[test]
    fn parse_info_variants() {
        let info_cases = [
            "info",
            "INFO",
            "Info",
            "information",
            "INFORMATION",
            "note",
            "NOTE",
            "Note",
            "convention",
            "CONVENTION",
            "refactor",
            "REFACTOR",
        ];
        for case in info_cases {
            assert_eq!(
                CanonicalSeverity::parse(case),
                Ok(CanonicalSeverity::Info),
                "Failed to parse '{}' as Info",
                case
            );
        }
    }

    #[test]
    fn parse_hint_variants() {
        let hint_cases = [
            "hint",
            "HINT",
            "Hint",
            "suggestion",
            "SUGGESTION",
            "help",
            "HELP",
            "Help",
            "style",
            "STYLE",
            "Style",
        ];
        for case in hint_cases {
            assert_eq!(
                CanonicalSeverity::parse(case),
                Ok(CanonicalSeverity::Hint),
                "Failed to parse '{}' as Hint",
                case
            );
        }
    }

    #[test]
    fn parse_unknown_variants() {
        let unknown_cases = ["unknown", "UNKNOWN", "Unknown", "off", "OFF", "Off", "0"];
        for case in unknown_cases {
            assert_eq!(
                CanonicalSeverity::parse(case),
                Ok(CanonicalSeverity::Unknown),
                "Failed to parse '{}' as Unknown",
                case
            );
        }
    }

    #[test]
    fn parse_invalid_returns_error() {
        let invalid_cases = ["", "   ", "invalid", "xyz", "123", "error!", "error "];
        for case in invalid_cases {
            assert!(
                CanonicalSeverity::parse(case).is_err(),
                "'{}' should not parse as valid severity",
                case
            );
        }
    }

    // -------------------------------------------------------------------------
    // Scenario: String representation
    // -------------------------------------------------------------------------

    #[test]
    fn as_str_returns_lowercase() {
        assert_eq!(CanonicalSeverity::Error.as_str(), "error");
        assert_eq!(CanonicalSeverity::Warning.as_str(), "warning");
        assert_eq!(CanonicalSeverity::Info.as_str(), "info");
        assert_eq!(CanonicalSeverity::Hint.as_str(), "hint");
        assert_eq!(CanonicalSeverity::Unknown.as_str(), "unknown");
    }

    #[test]
    fn display_trait_uses_as_str() {
        assert_eq!(format!("{}", CanonicalSeverity::Error), "error");
        assert_eq!(format!("{}", CanonicalSeverity::Warning), "warning");
        assert_eq!(format!("{}", CanonicalSeverity::Info), "info");
        assert_eq!(format!("{}", CanonicalSeverity::Hint), "hint");
        assert_eq!(format!("{}", CanonicalSeverity::Unknown), "unknown");
    }

    // -------------------------------------------------------------------------
    // Scenario: Numeric level
    // -------------------------------------------------------------------------

    #[test]
    fn level_returns_numeric_value() {
        assert_eq!(CanonicalSeverity::Unknown.level(), 0);
        assert_eq!(CanonicalSeverity::Hint.level(), 1);
        assert_eq!(CanonicalSeverity::Info.level(), 2);
        assert_eq!(CanonicalSeverity::Warning.level(), 3);
        assert_eq!(CanonicalSeverity::Error.level(), 4);
    }

    #[test]
    fn level_increases_with_severity() {
        assert!(CanonicalSeverity::Unknown.level() < CanonicalSeverity::Hint.level());
        assert!(CanonicalSeverity::Hint.level() < CanonicalSeverity::Info.level());
        assert!(CanonicalSeverity::Info.level() < CanonicalSeverity::Warning.level());
        assert!(CanonicalSeverity::Warning.level() < CanonicalSeverity::Error.level());
    }

    // -------------------------------------------------------------------------
    // Scenario: Comparison methods
    // -------------------------------------------------------------------------

    #[test]
    fn at_least_compares_severity() {
        let error = CanonicalSeverity::Error;
        assert!(error.at_least(CanonicalSeverity::Unknown));
        assert!(error.at_least(CanonicalSeverity::Hint));
        assert!(error.at_least(CanonicalSeverity::Info));
        assert!(error.at_least(CanonicalSeverity::Warning));
        assert!(error.at_least(CanonicalSeverity::Error)); // Error >= Error

        let hint = CanonicalSeverity::Hint;
        assert!(hint.at_least(CanonicalSeverity::Unknown));
        assert!(hint.at_least(CanonicalSeverity::Hint)); // Hint >= Hint
        assert!(!hint.at_least(CanonicalSeverity::Info));
    }

    #[test]
    fn is_problem_identifies_warnings_and_errors() {
        assert!(CanonicalSeverity::Error.is_problem());
        assert!(CanonicalSeverity::Warning.is_problem());
        assert!(!CanonicalSeverity::Info.is_problem());
        assert!(!CanonicalSeverity::Hint.is_problem());
        assert!(!CanonicalSeverity::Unknown.is_problem());
    }

    #[test]
    fn is_blocking_identifies_only_errors() {
        assert!(CanonicalSeverity::Error.is_blocking());
        assert!(!CanonicalSeverity::Warning.is_blocking());
        assert!(!CanonicalSeverity::Info.is_blocking());
        assert!(!CanonicalSeverity::Hint.is_blocking());
        assert!(!CanonicalSeverity::Unknown.is_blocking());
    }

    // -------------------------------------------------------------------------
    // Scenario: Default value
    // -------------------------------------------------------------------------

    #[test]
    fn default_is_unknown() {
        assert_eq!(CanonicalSeverity::default(), CanonicalSeverity::Unknown);
    }

    // -------------------------------------------------------------------------
    // Scenario: Debug representation
    // -------------------------------------------------------------------------

    #[test]
    fn debug_format_shows_variant_name() {
        assert_eq!(format!("{:?}", CanonicalSeverity::Error), "Error");
        assert_eq!(format!("{:?}", CanonicalSeverity::Warning), "Warning");
        assert_eq!(format!("{:?}", CanonicalSeverity::Info), "Info");
        assert_eq!(format!("{:?}", CanonicalSeverity::Hint), "Hint");
        assert_eq!(format!("{:?}", CanonicalSeverity::Unknown), "Unknown");
    }
}

// =============================================================================
// Feature: SeverityMapper
// =============================================================================

mod severity_mapper_feature {
    use super::*;

    // -------------------------------------------------------------------------
    // Scenario: Creating mappers
    // -------------------------------------------------------------------------

    #[test]
    fn new_creates_empty_mapper() {
        let mapper = SeverityMapper::new();
        assert!(mapper.is_empty());
        assert_eq!(mapper.mapping_count(), 0);
    }

    #[test]
    fn default_creates_empty_mapper() {
        let mapper = SeverityMapper::default();
        assert!(mapper.is_empty());
    }

    #[test]
    fn from_defaults_creates_populated_mapper() {
        let mapper = SeverityMapper::from_defaults();
        assert!(!mapper.is_empty());
        assert!(mapper.mapping_count() > 0);
    }

    // -------------------------------------------------------------------------
    // Scenario: Adding mappings
    // -------------------------------------------------------------------------

    #[test]
    fn add_mapping_stores_mapping() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("test-linter", "test-severity", CanonicalSeverity::Error);

        assert_eq!(
            mapper.map("test-linter", "test-severity"),
            CanonicalSeverity::Error
        );
        assert_eq!(mapper.mapping_count(), 1);
    }

    #[test]
    fn add_mapping_is_case_insensitive() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("TEST", "ERROR", CanonicalSeverity::Error);

        assert_eq!(mapper.map("test", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("Test", "Error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("TEST", "ERROR"), CanonicalSeverity::Error);
    }

    #[test]
    fn add_mapping_overwrites_existing() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("linter", "error", CanonicalSeverity::Error);
        mapper.add_mapping("linter", "error", CanonicalSeverity::Warning);

        assert_eq!(mapper.map("linter", "error"), CanonicalSeverity::Warning);
    }

    #[test]
    fn multiple_mappings_for_same_linter() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("linter", "error", CanonicalSeverity::Error);
        mapper.add_mapping("linter", "warning", CanonicalSeverity::Warning);
        mapper.add_mapping("linter", "info", CanonicalSeverity::Info);

        assert_eq!(mapper.map("linter", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("linter", "warning"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("linter", "info"), CanonicalSeverity::Info);
        assert_eq!(mapper.mapping_count(), 3);
    }

    // -------------------------------------------------------------------------
    // Scenario: Mapping severities
    // -------------------------------------------------------------------------

    #[test]
    fn map_returns_canonical_for_known_mapping() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("linter", "error", CanonicalSeverity::Error);

        assert_eq!(mapper.map("linter", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn map_returns_unknown_for_unknown_linter() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(
            mapper.map("unknown-linter", "error"),
            CanonicalSeverity::Unknown
        );
    }

    #[test]
    fn map_returns_unknown_for_unknown_severity() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(
            mapper.map("eslint", "unknown-severity"),
            CanonicalSeverity::Unknown
        );
    }

    #[test]
    fn map_is_case_insensitive_for_linter() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("ESLINT", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("EsLint", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn map_is_case_insensitive_for_severity() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("eslint", "ERROR"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("eslint", "Error"), CanonicalSeverity::Error);
    }

    // -------------------------------------------------------------------------
    // Scenario: Checking mappings
    // -------------------------------------------------------------------------

    #[test]
    fn has_mapping_returns_true_for_known() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("linter", "error", CanonicalSeverity::Error);

        assert!(mapper.has_mapping("linter", "error"));
    }

    #[test]
    fn has_mapping_returns_false_for_unknown() {
        let mapper = SeverityMapper::new();

        assert!(!mapper.has_mapping("linter", "error"));
        assert!(!mapper.has_mapping("unknown", "error"));
    }

    #[test]
    fn has_mapping_is_case_insensitive() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("TEST", "ERROR", CanonicalSeverity::Error);

        assert!(mapper.has_mapping("test", "error"));
        assert!(mapper.has_mapping("TEST", "ERROR"));
    }

    // -------------------------------------------------------------------------
    // Scenario: Removing mappings
    // -------------------------------------------------------------------------

    #[test]
    fn remove_linter_removes_all_mappings_for_linter() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("linter1", "error", CanonicalSeverity::Error);
        mapper.add_mapping("linter1", "warning", CanonicalSeverity::Warning);
        mapper.add_mapping("linter2", "error", CanonicalSeverity::Error);

        mapper.remove_linter("linter1");

        assert_eq!(mapper.map("linter1", "error"), CanonicalSeverity::Unknown);
        assert_eq!(mapper.map("linter1", "warning"), CanonicalSeverity::Unknown);
        assert_eq!(mapper.map("linter2", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn remove_linter_is_case_insensitive() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("TEST", "error", CanonicalSeverity::Error);

        mapper.remove_linter("test");

        assert_eq!(mapper.map("test", "error"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn remove_linter_on_empty_mapper_is_safe() {
        let mut mapper = SeverityMapper::new();
        // Should not panic
        mapper.remove_linter("nonexistent");
    }

    // -------------------------------------------------------------------------
    // Scenario: Merging mappers
    // -------------------------------------------------------------------------

    #[test]
    fn merge_combines_mappings() {
        let mut mapper1 = SeverityMapper::new();
        mapper1.add_mapping("linter1", "error", CanonicalSeverity::Error);

        let mut mapper2 = SeverityMapper::new();
        mapper2.add_mapping("linter2", "error", CanonicalSeverity::Error);

        mapper1.merge(mapper2);

        assert_eq!(mapper1.map("linter1", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper1.map("linter2", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn merge_overwrites_conflicting_mappings() {
        let mut mapper1 = SeverityMapper::new();
        mapper1.add_mapping("linter", "error", CanonicalSeverity::Error);

        let mut mapper2 = SeverityMapper::new();
        mapper2.add_mapping("linter", "error", CanonicalSeverity::Warning);

        mapper1.merge(mapper2);

        assert_eq!(mapper1.map("linter", "error"), CanonicalSeverity::Warning);
    }

    #[test]
    fn merge_with_empty_mapper() {
        let mut mapper = SeverityMapper::from_defaults();
        let original_count = mapper.mapping_count();

        mapper.merge(SeverityMapper::new());

        assert_eq!(mapper.mapping_count(), original_count);
    }

    // -------------------------------------------------------------------------
    // Scenario: Clone and Debug
    // -------------------------------------------------------------------------

    #[test]
    fn clone_creates_equal_mapper() {
        let mapper = SeverityMapper::from_defaults();
        let cloned = mapper.clone();

        assert_eq!(mapper.map("eslint", "error"), cloned.map("eslint", "error"));
        assert_eq!(
            mapper.map("rustc", "warning"),
            cloned.map("rustc", "warning")
        );
    }

    #[test]
    fn debug_format_includes_type_name() {
        let mapper = SeverityMapper::new();
        let debug = format!("{:?}", mapper);
        assert!(debug.contains("SeverityMapper"));
    }
}

// =============================================================================
// Feature: Built-in Linter Mappings
// =============================================================================

mod builtin_linter_mappings_feature {
    use super::*;

    fn create_mapper() -> SeverityMapper {
        SeverityMapper::from_defaults()
    }

    // -------------------------------------------------------------------------
    // Scenario: ESLint mappings
    // -------------------------------------------------------------------------

    #[test]
    fn eslint_error_maps_to_error() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn eslint_warn_maps_to_warning() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("eslint", "warn"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("eslint", "warning"), CanonicalSeverity::Warning);
    }

    #[test]
    fn eslint_info_maps_to_info() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("eslint", "info"), CanonicalSeverity::Info);
    }

    #[test]
    fn eslint_off_maps_to_unknown() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("eslint", "off"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn eslint_numeric_severity_2_maps_to_error() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("eslint", "2"), CanonicalSeverity::Error);
    }

    #[test]
    fn eslint_numeric_severity_1_maps_to_warning() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("eslint", "1"), CanonicalSeverity::Warning);
    }

    #[test]
    fn eslint_numeric_severity_0_maps_to_unknown() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("eslint", "0"), CanonicalSeverity::Unknown);
    }

    // -------------------------------------------------------------------------
    // Scenario: Rustc mappings
    // -------------------------------------------------------------------------

    #[test]
    fn rustc_error_maps_to_error() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("rustc", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn rustc_warning_maps_to_warning() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("rustc", "warning"), CanonicalSeverity::Warning);
    }

    #[test]
    fn rustc_note_maps_to_info() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("rustc", "note"), CanonicalSeverity::Info);
    }

    #[test]
    fn rustc_help_maps_to_hint() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("rustc", "help"), CanonicalSeverity::Hint);
    }

    // -------------------------------------------------------------------------
    // Scenario: Clippy mappings
    // -------------------------------------------------------------------------

    #[test]
    fn clippy_error_maps_to_error() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("clippy", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn clippy_warning_maps_to_warning() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("clippy", "warning"), CanonicalSeverity::Warning);
    }

    #[test]
    fn clippy_note_maps_to_info() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("clippy", "note"), CanonicalSeverity::Info);
    }

    #[test]
    fn clippy_help_maps_to_hint() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("clippy", "help"), CanonicalSeverity::Hint);
    }

    // -------------------------------------------------------------------------
    // Scenario: Pylint mappings
    // -------------------------------------------------------------------------

    #[test]
    fn pylint_fatal_maps_to_error() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("pylint", "fatal"), CanonicalSeverity::Error);
    }

    #[test]
    fn pylint_error_maps_to_error() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("pylint", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn pylint_warning_maps_to_warning() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("pylint", "warning"), CanonicalSeverity::Warning);
    }

    #[test]
    fn pylint_convention_maps_to_info() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("pylint", "convention"), CanonicalSeverity::Info);
    }

    #[test]
    fn pylint_refactor_maps_to_info() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("pylint", "refactor"), CanonicalSeverity::Info);
    }

    #[test]
    fn pylint_info_maps_to_info() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("pylint", "info"), CanonicalSeverity::Info);
    }

    // -------------------------------------------------------------------------
    // Scenario: Golint mappings
    // -------------------------------------------------------------------------

    #[test]
    fn golint_error_maps_to_error() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("golint", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn golint_warning_maps_to_warning() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("golint", "warning"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("golint", "warn"), CanonicalSeverity::Warning);
    }

    #[test]
    fn golint_info_maps_to_info() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("golint", "info"), CanonicalSeverity::Info);
    }

    // -------------------------------------------------------------------------
    // Scenario: ShellCheck mappings
    // -------------------------------------------------------------------------

    #[test]
    fn shellcheck_error_maps_to_error() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("shellcheck", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn shellcheck_warning_maps_to_warning() {
        let mapper = create_mapper();
        assert_eq!(
            mapper.map("shellcheck", "warning"),
            CanonicalSeverity::Warning
        );
    }

    #[test]
    fn shellcheck_info_maps_to_info() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("shellcheck", "info"), CanonicalSeverity::Info);
    }

    #[test]
    fn shellcheck_style_maps_to_hint() {
        let mapper = create_mapper();
        assert_eq!(mapper.map("shellcheck", "style"), CanonicalSeverity::Hint);
    }

    // -------------------------------------------------------------------------
    // Scenario: Case insensitivity
    // -------------------------------------------------------------------------

    #[test]
    fn all_linters_case_insensitive() {
        let mapper = create_mapper();

        // Test various case combinations
        assert_eq!(mapper.map("ESLINT", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("RUSTC", "WARNING"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("Pylint", "Fatal"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("GoLint", "Error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("SHELLCHECK", "STYLE"), CanonicalSeverity::Hint);
    }
}

// =============================================================================
// Feature: SeverityMapBuilder
// =============================================================================

mod severity_map_builder_feature {
    use super::*;

    // -------------------------------------------------------------------------
    // Scenario: Creating builders
    // -------------------------------------------------------------------------

    #[test]
    fn new_creates_builder_with_empty_mapper() {
        let mapper = SeverityMapBuilder::new().build();
        assert!(mapper.is_empty());
    }

    #[test]
    fn default_creates_builder_with_empty_mapper() {
        let mapper = SeverityMapBuilder::default().build();
        assert!(mapper.is_empty());
    }

    #[test]
    fn with_defaults_creates_builder_with_default_mappings() {
        let mapper = SeverityMapBuilder::with_defaults().build();
        assert!(!mapper.is_empty());
        assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
    }

    // -------------------------------------------------------------------------
    // Scenario: Adding linter mappings
    // -------------------------------------------------------------------------

    #[test]
    fn with_linter_adds_multiple_mappings() {
        let mapper = SeverityMapBuilder::new()
            .with_linter(
                "custom",
                [
                    ("error", CanonicalSeverity::Error),
                    ("warning", CanonicalSeverity::Warning),
                    ("info", CanonicalSeverity::Info),
                ],
            )
            .build();

        assert_eq!(mapper.map("custom", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("custom", "warning"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("custom", "info"), CanonicalSeverity::Info);
    }

    #[test]
    fn with_linter_can_be_called_multiple_times() {
        let mapper = SeverityMapBuilder::new()
            .with_linter("linter1", [("error", CanonicalSeverity::Error)])
            .with_linter("linter2", [("error", CanonicalSeverity::Warning)])
            .build();

        assert_eq!(mapper.map("linter1", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("linter2", "error"), CanonicalSeverity::Warning);
    }

    #[test]
    fn with_linter_accepts_iterator() {
        let mappings = vec![
            ("error".to_string(), CanonicalSeverity::Error),
            ("warning".to_string(), CanonicalSeverity::Warning),
        ];
        let mapper = SeverityMapBuilder::new()
            .with_linter("custom", mappings)
            .build();

        assert_eq!(mapper.map("custom", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("custom", "warning"), CanonicalSeverity::Warning);
    }

    // -------------------------------------------------------------------------
    // Scenario: Adding single mappings
    // -------------------------------------------------------------------------

    #[test]
    fn with_mapping_adds_single_mapping() {
        let mapper = SeverityMapBuilder::new()
            .with_mapping("linter", "error", CanonicalSeverity::Error)
            .build();

        assert_eq!(mapper.map("linter", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn with_mapping_can_be_called_multiple_times() {
        let mapper = SeverityMapBuilder::new()
            .with_mapping("linter", "error", CanonicalSeverity::Error)
            .with_mapping("linter", "warning", CanonicalSeverity::Warning)
            .with_mapping("other", "error", CanonicalSeverity::Error)
            .build();

        assert_eq!(mapper.map("linter", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("linter", "warning"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("other", "error"), CanonicalSeverity::Error);
    }

    // -------------------------------------------------------------------------
    // Scenario: Method chaining
    // -------------------------------------------------------------------------

    #[test]
    fn builder_supports_chaining() {
        let mapper = SeverityMapBuilder::new()
            .with_linter("linter1", [("error", CanonicalSeverity::Error)])
            .with_mapping("linter2", "error", CanonicalSeverity::Error)
            .with_linter("linter3", [("warning", CanonicalSeverity::Warning)])
            .build();

        assert_eq!(mapper.map("linter1", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("linter2", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("linter3", "warning"), CanonicalSeverity::Warning);
    }

    #[test]
    fn with_defaults_then_custom() {
        let mapper = SeverityMapBuilder::with_defaults()
            .with_mapping("custom", "error", CanonicalSeverity::Error)
            .build();

        // Has defaults
        assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
        // And custom
        assert_eq!(mapper.map("custom", "error"), CanonicalSeverity::Error);
    }

    // -------------------------------------------------------------------------
    // Scenario: Build
    // -------------------------------------------------------------------------

    #[test]
    fn build_returns_mapper() {
        let builder =
            SeverityMapBuilder::new().with_mapping("linter", "error", CanonicalSeverity::Error);
        let mapper = builder.build();

        assert_eq!(mapper.map("linter", "error"), CanonicalSeverity::Error);
    }

    #[test]
    fn debug_format_includes_type_name() {
        let builder = SeverityMapBuilder::new();
        let debug = format!("{:?}", builder);
        assert!(debug.contains("SeverityMapBuilder"));
    }
}

// =============================================================================
// Feature: Convenience Functions
// =============================================================================

mod convenience_functions_feature {
    use super::*;

    // -------------------------------------------------------------------------
    // Scenario: map_severity function
    // -------------------------------------------------------------------------

    #[test]
    fn map_severity_uses_default_mapper() {
        assert_eq!(map_severity("eslint", "error"), CanonicalSeverity::Error);
        assert_eq!(map_severity("rustc", "warning"), CanonicalSeverity::Warning);
        assert_eq!(map_severity("pylint", "fatal"), CanonicalSeverity::Error);
    }

    #[test]
    fn map_severity_returns_unknown_for_unknown_linter() {
        assert_eq!(
            map_severity("unknown-linter", "error"),
            CanonicalSeverity::Unknown
        );
    }

    #[test]
    fn map_severity_is_case_insensitive() {
        assert_eq!(map_severity("ESLINT", "ERROR"), CanonicalSeverity::Error);
        assert_eq!(map_severity("rustc", "WARNING"), CanonicalSeverity::Warning);
    }

    // -------------------------------------------------------------------------
    // Scenario: is_error_level function
    // -------------------------------------------------------------------------

    #[test]
    fn is_error_level_returns_true_for_error() {
        assert!(is_error_level(&CanonicalSeverity::Error));
    }

    #[test]
    fn is_error_level_returns_false_for_non_error() {
        assert!(!is_error_level(&CanonicalSeverity::Warning));
        assert!(!is_error_level(&CanonicalSeverity::Info));
        assert!(!is_error_level(&CanonicalSeverity::Hint));
        assert!(!is_error_level(&CanonicalSeverity::Unknown));
    }

    // -------------------------------------------------------------------------
    // Scenario: is_warning_level function
    // -------------------------------------------------------------------------

    #[test]
    fn is_warning_level_returns_true_for_warning_and_above() {
        assert!(is_warning_level(&CanonicalSeverity::Error));
        assert!(is_warning_level(&CanonicalSeverity::Warning));
    }

    #[test]
    fn is_warning_level_returns_false_for_below_warning() {
        assert!(!is_warning_level(&CanonicalSeverity::Info));
        assert!(!is_warning_level(&CanonicalSeverity::Hint));
        assert!(!is_warning_level(&CanonicalSeverity::Unknown));
    }

    // -------------------------------------------------------------------------
    // Scenario: is_problem_level function
    // -------------------------------------------------------------------------

    #[test]
    fn is_problem_level_returns_true_for_problems() {
        assert!(is_problem_level(&CanonicalSeverity::Error));
        assert!(is_problem_level(&CanonicalSeverity::Warning));
    }

    #[test]
    fn is_problem_level_returns_false_for_non_problems() {
        assert!(!is_problem_level(&CanonicalSeverity::Info));
        assert!(!is_problem_level(&CanonicalSeverity::Hint));
        assert!(!is_problem_level(&CanonicalSeverity::Unknown));
    }
}

// =============================================================================
// Feature: Edge Cases
// =============================================================================

mod edge_cases_feature {
    use super::*;

    // -------------------------------------------------------------------------
    // Scenario: Empty strings
    // -------------------------------------------------------------------------

    #[test]
    fn empty_linter_name_returns_unknown() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("", "error"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn empty_severity_returns_unknown() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("eslint", ""), CanonicalSeverity::Unknown);
    }

    #[test]
    fn both_empty_returns_unknown() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("", ""), CanonicalSeverity::Unknown);
    }

    #[test]
    fn add_mapping_with_empty_strings_works() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("", "", CanonicalSeverity::Error);
        assert_eq!(mapper.map("", ""), CanonicalSeverity::Error);
    }

    // -------------------------------------------------------------------------
    // Scenario: Whitespace handling
    // -------------------------------------------------------------------------

    #[test]
    fn whitespace_in_severity_returns_unknown() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("eslint", " error"), CanonicalSeverity::Unknown);
        assert_eq!(mapper.map("eslint", "error "), CanonicalSeverity::Unknown);
        assert_eq!(mapper.map("eslint", " error "), CanonicalSeverity::Unknown);
    }

    #[test]
    fn whitespace_in_linter_returns_unknown() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map(" eslint", "error"), CanonicalSeverity::Unknown);
        assert_eq!(mapper.map("eslint ", "error"), CanonicalSeverity::Unknown);
    }

    // -------------------------------------------------------------------------
    // Scenario: Unicode handling
    // -------------------------------------------------------------------------

    #[test]
    fn unicode_severity_returns_unknown() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("eslint", "错误"), CanonicalSeverity::Unknown);
        assert_eq!(mapper.map("eslint", "エラー"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn unicode_linter_returns_unknown() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("工具", "error"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn add_mapping_with_unicode_works() {
        let mut mapper = SeverityMapper::new();
        mapper.add_mapping("工具", "错误", CanonicalSeverity::Error);
        assert_eq!(mapper.map("工具", "错误"), CanonicalSeverity::Error);
    }

    // -------------------------------------------------------------------------
    // Scenario: Long strings
    // -------------------------------------------------------------------------

    #[test]
    fn very_long_severity_string() {
        let mapper = SeverityMapper::from_defaults();
        let long_severity = "error".repeat(100);
        assert_eq!(
            mapper.map("eslint", &long_severity),
            CanonicalSeverity::Unknown
        );
    }

    #[test]
    fn very_long_linter_name() {
        let mapper = SeverityMapper::from_defaults();
        let long_linter = "linter".repeat(100);
        assert_eq!(
            mapper.map(&long_linter, "error"),
            CanonicalSeverity::Unknown
        );
    }

    // -------------------------------------------------------------------------
    // Scenario: Numeric strings
    // -------------------------------------------------------------------------

    #[test]
    fn numeric_severity_strings() {
        let mapper = SeverityMapper::from_defaults();

        // ESLint numeric severities
        assert_eq!(mapper.map("eslint", "2"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("eslint", "1"), CanonicalSeverity::Warning);
        assert_eq!(mapper.map("eslint", "0"), CanonicalSeverity::Unknown);

        // Unknown numeric for other linters
        assert_eq!(mapper.map("rustc", "2"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn large_numeric_severity() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("eslint", "999"), CanonicalSeverity::Unknown);
    }

    // -------------------------------------------------------------------------
    // Scenario: Special characters
    // -------------------------------------------------------------------------

    #[test]
    fn special_characters_in_severity() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(mapper.map("eslint", "error!"), CanonicalSeverity::Unknown);
        assert_eq!(
            mapper.map("eslint", "error@host"),
            CanonicalSeverity::Unknown
        );
        assert_eq!(mapper.map("eslint", "error\n"), CanonicalSeverity::Unknown);
    }

    #[test]
    fn special_characters_in_linter() {
        let mapper = SeverityMapper::from_defaults();
        assert_eq!(
            mapper.map("eslint-plugin", "error"),
            CanonicalSeverity::Unknown
        );
        assert_eq!(
            mapper.map("eslint/plugin", "error"),
            CanonicalSeverity::Unknown
        );
    }
}

// =============================================================================
// Feature: Error Handling
// =============================================================================

mod error_handling_feature {
    use super::*;

    // -------------------------------------------------------------------------
    // Scenario: SeverityParseError
    // -------------------------------------------------------------------------

    #[test]
    fn severity_parse_error_display() {
        let error = SeverityParseError("invalid-severity".to_string());
        assert_eq!(format!("{}", error), "Unknown severity: invalid-severity");
    }

    #[test]
    fn severity_parse_error_debug() {
        let error = SeverityParseError("invalid".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("SeverityParseError"));
    }

    #[test]
    fn severity_parse_error_equality() {
        let error1 = SeverityParseError("invalid".to_string());
        let error2 = SeverityParseError("invalid".to_string());
        let error3 = SeverityParseError("other".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn severity_parse_error_clone() {
        let error = SeverityParseError("invalid".to_string());
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }
}

// =============================================================================
// Feature: Property-Based Tests
// =============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // -------------------------------------------------------------------------
        // Property: Case insensitivity
        // -------------------------------------------------------------------------

        #[test]
        fn map_is_case_insensitive_for_linter(
            linter in "eslint|rustc|pylint",
            severity in "error|warning|info"
        ) {
            let mapper = SeverityMapper::from_defaults();
            let upper = mapper.map(&linter.to_uppercase(), &severity);
            let lower = mapper.map(&linter.to_lowercase(), &severity);
            prop_assert_eq!(upper, lower);
        }

        #[test]
        fn map_is_case_insensitive_for_severity(
            linter in "eslint|rustc|pylint",
            severity in "error|warning|info"
        ) {
            let mapper = SeverityMapper::from_defaults();
            let upper = mapper.map(&linter, &severity.to_uppercase());
            let lower = mapper.map(&linter, &severity.to_lowercase());
            prop_assert_eq!(upper, lower);
        }

        // -------------------------------------------------------------------------
        // Property: Ordering
        // -------------------------------------------------------------------------

        #[test]
        fn severity_ordering_is_consistent(
            a in 0u8..5,
            b in 0u8..5
        ) {
            let levels = [
                CanonicalSeverity::Unknown,
                CanonicalSeverity::Hint,
                CanonicalSeverity::Info,
                CanonicalSeverity::Warning,
                CanonicalSeverity::Error,
            ];

            let sev_a = levels[a as usize % levels.len()];
            let sev_b = levels[b as usize % levels.len()];

            // Ordering should match numeric level
            prop_assert_eq!(sev_a < sev_b, sev_a.level() < sev_b.level());
        }

        // -------------------------------------------------------------------------
        // Property: Round-trip for known severities
        // -------------------------------------------------------------------------

        #[test]
        fn parse_roundtrip_for_known_severities(
            severity in prop_oneof![
                Just(CanonicalSeverity::Error),
                Just(CanonicalSeverity::Warning),
                Just(CanonicalSeverity::Info),
                Just(CanonicalSeverity::Hint),
                Just(CanonicalSeverity::Unknown),
            ]
        ) {
            let parsed = CanonicalSeverity::parse(severity.as_str());
            prop_assert_eq!(parsed, Ok(severity));
        }

        // -------------------------------------------------------------------------
        // Property: Unknown strings return Unknown
        // -------------------------------------------------------------------------

        #[test]
        fn unknown_linter_returns_unknown(
            linter in "[a-z]+",
            severity in "[a-z]+"
        ) {
            // Skip known linters
            prop_assume!(!matches!(linter.as_str(), "eslint" | "rustc" | "pylint" | "golint" | "shellcheck" | "clippy"));

            let mapper = SeverityMapper::new();
            prop_assert_eq!(mapper.map(&linter, &severity), CanonicalSeverity::Unknown);
        }

        // -------------------------------------------------------------------------
        // Property: Level values are unique
        // -------------------------------------------------------------------------

        #[test]
        fn severity_levels_are_unique(
            a in 0u8..5,
            b in 0u8..5
        ) {
            let levels = [
                CanonicalSeverity::Unknown,
                CanonicalSeverity::Hint,
                CanonicalSeverity::Info,
                CanonicalSeverity::Warning,
                CanonicalSeverity::Error,
            ];

            let sev_a = levels[a as usize % levels.len()];
            let sev_b = levels[b as usize % levels.len()];

            if sev_a != sev_b {
                prop_assert_ne!(sev_a.level(), sev_b.level());
            } else {
                prop_assert_eq!(sev_a.level(), sev_b.level());
            }
        }

        // -------------------------------------------------------------------------
        // Property: Builder produces consistent results
        // -------------------------------------------------------------------------

        #[test]
        fn builder_produces_consistent_mapper(
            mappings in proptest::collection::vec(
                (any::<String>().prop_filter("non-empty", |s| !s.is_empty()), 0u8..5),
                0..10
            )
        ) {
            let mut builder = SeverityMapBuilder::new();
            let canonical_levels = [
                CanonicalSeverity::Unknown,
                CanonicalSeverity::Hint,
                CanonicalSeverity::Info,
                CanonicalSeverity::Warning,
                CanonicalSeverity::Error,
            ];
            let mut expected = std::collections::HashMap::new();

            for (linter, level) in &mappings {
                let canonical = canonical_levels[(*level as usize) % canonical_levels.len()];
                builder = builder.with_mapping(
                    linter,
                    "severity",
                    canonical,
                );
                expected.insert(linter.to_lowercase(), canonical);
            }

            let mapper = builder.build();

            // Duplicate keys overwrite earlier entries, so only the last write per linter survives.
            for (linter, expected) in expected {
                prop_assert_eq!(mapper.map(&linter, "severity"), expected);
            }
        }
    }
}

// =============================================================================
// Feature: Integration Tests
// =============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn full_workflow_with_custom_linter() {
        // Create a mapper for a custom linter
        let mapper = SeverityMapBuilder::new()
            .with_linter(
                "my-custom-linter",
                [
                    ("critical", CanonicalSeverity::Error),
                    ("major", CanonicalSeverity::Warning),
                    ("minor", CanonicalSeverity::Info),
                    ("suggestion", CanonicalSeverity::Hint),
                ],
            )
            .build();

        // Use the mapper
        assert_eq!(
            mapper.map("my-custom-linter", "critical"),
            CanonicalSeverity::Error
        );
        assert_eq!(
            mapper.map("my-custom-linter", "major"),
            CanonicalSeverity::Warning
        );
        assert_eq!(
            mapper.map("my-custom-linter", "minor"),
            CanonicalSeverity::Info
        );
        assert_eq!(
            mapper.map("my-custom-linter", "suggestion"),
            CanonicalSeverity::Hint
        );

        // Check problem detection
        assert!(is_error_level(&mapper.map("my-custom-linter", "critical")));
        assert!(!is_error_level(&mapper.map("my-custom-linter", "major")));
    }

    #[test]
    fn combining_default_and_custom_mappings() {
        let mapper = SeverityMapBuilder::with_defaults()
            .with_linter(
                "custom",
                [
                    ("bad", CanonicalSeverity::Error),
                    ("not-great", CanonicalSeverity::Warning),
                ],
            )
            .build();

        // Default mappings work
        assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
        assert_eq!(mapper.map("rustc", "warning"), CanonicalSeverity::Warning);

        // Custom mappings work
        assert_eq!(mapper.map("custom", "bad"), CanonicalSeverity::Error);
        assert_eq!(
            mapper.map("custom", "not-great"),
            CanonicalSeverity::Warning
        );
    }

    #[test]
    fn mapper_merge_workflow() {
        // Create base mapper with defaults
        let mut mapper = SeverityMapper::from_defaults();

        // Create custom mapper for internal tools
        let internal = SeverityMapBuilder::new()
            .with_linter(
                "internal-linter",
                [
                    ("severe", CanonicalSeverity::Error),
                    ("moderate", CanonicalSeverity::Warning),
                ],
            )
            .build();

        // Merge internal mappings
        mapper.merge(internal);

        // Both default and internal work
        assert_eq!(mapper.map("eslint", "error"), CanonicalSeverity::Error);
        assert_eq!(
            mapper.map("internal-linter", "severe"),
            CanonicalSeverity::Error
        );
    }

    #[test]
    fn filtering_by_severity_level() {
        let mapper = SeverityMapper::from_defaults();

        let severities = ["error", "warning", "info", "hint"];
        let mapped: Vec<_> = severities
            .iter()
            .map(|&s| mapper.map("eslint", s))
            .collect();

        // Filter to only errors
        let errors: Vec<_> = mapped.iter().filter(|s| is_error_level(s)).collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(*errors[0], CanonicalSeverity::Error);

        // Filter to warnings and above
        let warnings_and_above: Vec<_> = mapped.iter().filter(|s| is_warning_level(s)).collect();
        assert_eq!(warnings_and_above.len(), 2);

        // Filter to problems
        let problems: Vec<_> = mapped.iter().filter(|s| is_problem_level(s)).collect();
        assert_eq!(problems.len(), 2);
    }

    #[test]
    fn severity_counts_by_level() {
        let mapper = SeverityMapper::from_defaults();

        let findings = [
            ("eslint", "error"),
            ("eslint", "warning"),
            ("rustc", "error"),
            ("pylint", "fatal"),
            ("unknown", "error"), // Unknown linter
        ];

        let mut error_count = 0;
        let mut warning_count = 0;
        let mut unknown_count = 0;

        for (linter, severity) in findings {
            match mapper.map(linter, severity) {
                CanonicalSeverity::Error => error_count += 1,
                CanonicalSeverity::Warning => warning_count += 1,
                CanonicalSeverity::Unknown => unknown_count += 1,
                _ => {}
            }
        }

        assert_eq!(error_count, 3); // eslint error, rustc error, pylint fatal
        assert_eq!(warning_count, 1); // eslint warning
        assert_eq!(unknown_count, 1); // unknown linter
    }
}
