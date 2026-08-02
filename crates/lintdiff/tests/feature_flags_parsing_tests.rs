//! Comprehensive tests for feature flag parsing functionality.

use lintdiff::config::feature_flags::{
    parse_feature_flag_assignment, parse_feature_flag_value, parse_flag, FeatureFlag, FALSE_VALUES,
    TRUE_VALUES,
};

// =============================================================================
// Flag Name Parsing Tests
// =============================================================================

mod flag_name_parsing {
    use super::*;

    #[test]
    fn parses_primary_span_matching_flag() {
        let result = parse_flag("primary_span_matching");
        assert_eq!(result, Some(FeatureFlag::PrimarySpanMatching));
    }

    #[test]
    fn parses_path_filters_flag() {
        let result = parse_flag("path_filters");
        assert_eq!(result, Some(FeatureFlag::PathFilters));
    }

    #[test]
    fn returns_none_for_unknown_flag() {
        let result = parse_flag("unknown_flag");
        assert!(result.is_none());
    }

    #[test]
    fn returns_none_for_empty_string() {
        let result = parse_flag("");
        assert!(result.is_none());
    }

    #[test]
    fn returns_none_for_whitespace_only() {
        let result = parse_flag("   ");
        assert!(result.is_none());
    }

    #[test]
    fn returns_none_for_special_characters() {
        let result = parse_flag("flag!@#$%");
        assert!(result.is_none());
    }

    #[test]
    fn returns_none_for_numeric_prefix() {
        let result = parse_flag("123_flag");
        assert!(result.is_none());
    }

    // Case insensitivity tests
    mod case_insensitivity {
        use super::*;

        #[test]
        fn accepts_uppercase_flag_name() {
            let result = parse_flag("PRIMARY_SPAN_MATCHING");
            assert_eq!(result, Some(FeatureFlag::PrimarySpanMatching));
        }

        #[test]
        fn accepts_mixed_case_flag_name() {
            let result = parse_flag("Primary_Span_Matching");
            assert_eq!(result, Some(FeatureFlag::PrimarySpanMatching));
        }

        #[test]
        fn accepts_lowercase_flag_name() {
            let result = parse_flag("path_filters");
            assert_eq!(result, Some(FeatureFlag::PathFilters));
        }

        #[test]
        fn accepts_all_caps_path_filters() {
            let result = parse_flag("PATH_FILTERS");
            assert_eq!(result, Some(FeatureFlag::PathFilters));
        }

        #[test]
        fn accepts_camel_case_variant() {
            let result = parse_flag("PathFilters");
            assert!(result.is_none(), "Should not match without underscores");
        }
    }
}

// =============================================================================
// Boolean Value Parsing Tests
// =============================================================================

mod boolean_value_parsing {
    use super::*;

    // True value tests
    mod true_values {
        use super::*;

        #[test]
        fn parses_true() {
            assert_eq!(parse_feature_flag_value("true"), Ok(true));
        }

        #[test]
        fn parses_1() {
            assert_eq!(parse_feature_flag_value("1"), Ok(true));
        }

        #[test]
        fn parses_on() {
            assert_eq!(parse_feature_flag_value("on"), Ok(true));
        }

        #[test]
        fn parses_enabled() {
            assert_eq!(parse_feature_flag_value("enabled"), Ok(true));
        }

        #[test]
        fn parses_yes() {
            assert_eq!(parse_feature_flag_value("yes"), Ok(true));
        }

        #[test]
        fn verifies_all_true_constants() {
            for &value in TRUE_VALUES.iter() {
                assert_eq!(
                    parse_feature_flag_value(value),
                    Ok(true),
                    "Failed to parse '{}' as true",
                    value
                );
            }
        }
    }

    // False value tests
    mod false_values {
        use super::*;

        #[test]
        fn parses_false() {
            assert_eq!(parse_feature_flag_value("false"), Ok(false));
        }

        #[test]
        fn parses_0() {
            assert_eq!(parse_feature_flag_value("0"), Ok(false));
        }

        #[test]
        fn parses_off() {
            assert_eq!(parse_feature_flag_value("off"), Ok(false));
        }

        #[test]
        fn parses_disabled() {
            assert_eq!(parse_feature_flag_value("disabled"), Ok(false));
        }

        #[test]
        fn parses_no() {
            assert_eq!(parse_feature_flag_value("no"), Ok(false));
        }

        #[test]
        fn verifies_all_false_constants() {
            for &value in FALSE_VALUES.iter() {
                assert_eq!(
                    parse_feature_flag_value(value),
                    Ok(false),
                    "Failed to parse '{}' as false",
                    value
                );
            }
        }
    }

    // Case insensitivity for values
    mod value_case_insensitivity {
        use super::*;

        #[test]
        fn parses_uppercase_true() {
            assert_eq!(parse_feature_flag_value("TRUE"), Ok(true));
        }

        #[test]
        fn parses_mixed_case_true() {
            assert_eq!(parse_feature_flag_value("TrUe"), Ok(true));
        }

        #[test]
        fn parses_uppercase_false() {
            assert_eq!(parse_feature_flag_value("FALSE"), Ok(false));
        }

        #[test]
        fn parses_mixed_case_false() {
            assert_eq!(parse_feature_flag_value("FaLsE"), Ok(false));
        }

        #[test]
        fn parses_uppercase_on() {
            assert_eq!(parse_feature_flag_value("ON"), Ok(true));
        }

        #[test]
        fn parses_uppercase_off() {
            assert_eq!(parse_feature_flag_value("OFF"), Ok(false));
        }

        #[test]
        fn parses_uppercase_yes() {
            assert_eq!(parse_feature_flag_value("YES"), Ok(true));
        }

        #[test]
        fn parses_uppercase_no() {
            assert_eq!(parse_feature_flag_value("NO"), Ok(false));
        }

        #[test]
        fn parses_uppercase_enabled() {
            assert_eq!(parse_feature_flag_value("ENABLED"), Ok(true));
        }

        #[test]
        fn parses_uppercase_disabled() {
            assert_eq!(parse_feature_flag_value("DISABLED"), Ok(false));
        }
    }

    // Whitespace handling
    mod whitespace_handling {
        use super::*;

        #[test]
        fn trims_leading_whitespace() {
            assert_eq!(parse_feature_flag_value("  true"), Ok(true));
        }

        #[test]
        fn trims_trailing_whitespace() {
            assert_eq!(parse_feature_flag_value("false  "), Ok(false));
        }

        #[test]
        fn trims_both_leading_and_trailing_whitespace() {
            assert_eq!(parse_feature_flag_value("  on  "), Ok(true));
        }

        #[test]
        fn handles_tabs() {
            assert_eq!(parse_feature_flag_value("\ton\t"), Ok(true));
        }

        #[test]
        fn handles_mixed_whitespace() {
            assert_eq!(parse_feature_flag_value(" \t yes \t "), Ok(true));
        }

        #[test]
        fn empty_after_trim_is_error() {
            assert!(parse_feature_flag_value("   ").is_err());
        }
    }

    // Invalid value handling
    mod invalid_values {
        use super::*;

        #[test]
        fn rejects_invalid_string() {
            let result = parse_feature_flag_value("maybe");
            assert!(result.is_err());
        }

        #[test]
        fn rejects_numeric_values_other_than_0_1() {
            let result = parse_feature_flag_value("2");
            assert!(result.is_err());
        }

        #[test]
        fn rejects_negative_numbers() {
            let result = parse_feature_flag_value("-1");
            assert!(result.is_err());
        }

        #[test]
        fn rejects_float_values() {
            let result = parse_feature_flag_value("1.0");
            assert!(result.is_err());
        }

        #[test]
        fn rejects_yes_no_with_spaces() {
            // "yes please" should not be accepted as "yes"
            let result = parse_feature_flag_value("yes please");
            assert!(result.is_err());
        }

        #[test]
        fn error_message_contains_original_value() {
            let result = parse_feature_flag_value("invalid");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.contains("invalid"));
        }

        #[test]
        fn error_message_lists_expected_values() {
            let result = parse_feature_flag_value("bad");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.contains("true/false"));
            assert!(err.contains("on/off"));
            assert!(err.contains("1/0"));
        }
    }
}

// =============================================================================
// Assignment Parsing Tests
// =============================================================================

mod assignment_parsing {
    use super::*;

    #[test]
    fn parses_valid_assignment() {
        let result = parse_feature_flag_assignment("primary_span_matching=true");
        assert!(result.is_ok());
        let (flag, value) = result.unwrap();
        assert_eq!(flag, FeatureFlag::PrimarySpanMatching);
        assert!(value);
    }

    #[test]
    fn parses_assignment_with_false_value() {
        let result = parse_feature_flag_assignment("path_filters=false");
        assert!(result.is_ok());
        let (flag, value) = result.unwrap();
        assert_eq!(flag, FeatureFlag::PathFilters);
        assert!(!value);
    }

    #[test]
    fn parses_assignment_with_numeric_value() {
        let result = parse_feature_flag_assignment("primary_span_matching=1");
        assert!(result.is_ok());
        let (_, value) = result.unwrap();
        assert!(value);
    }

    #[test]
    fn parses_assignment_with_on_off() {
        let result = parse_feature_flag_assignment("path_filters=off");
        assert!(result.is_ok());
        let (_, value) = result.unwrap();
        assert!(!value);
    }

    #[test]
    fn parses_assignment_with_enabled_disabled() {
        let result = parse_feature_flag_assignment("primary_span_matching=disabled");
        assert!(result.is_ok());
        let (_, value) = result.unwrap();
        assert!(!value);
    }

    #[test]
    fn parses_assignment_with_yes_no() {
        let result = parse_feature_flag_assignment("path_filters=yes");
        assert!(result.is_ok());
        let (_, value) = result.unwrap();
        assert!(value);
    }

    // Whitespace handling in assignments
    mod assignment_whitespace {
        use super::*;

        #[test]
        fn trims_whitespace_around_flag_name() {
            let result = parse_feature_flag_assignment("  primary_span_matching=true");
            assert!(result.is_ok());
        }

        #[test]
        fn trims_whitespace_around_value() {
            let result = parse_feature_flag_assignment("path_filters=  false  ");
            assert!(result.is_ok());
        }

        #[test]
        fn trims_whitespace_on_both_sides() {
            let result = parse_feature_flag_assignment("  path_filters  =  off  ");
            assert!(result.is_ok());
        }
    }

    // Error cases for assignments
    mod assignment_errors {
        use super::*;

        #[test]
        fn rejects_assignment_without_equals() {
            let result = parse_feature_flag_assignment("primary_span_matching_true");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.contains("expected name=value"));
        }

        #[test]
        fn rejects_unknown_flag_in_assignment() {
            let result = parse_feature_flag_assignment("unknown_flag=true");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.contains("unknown feature flag"));
        }

        #[test]
        fn rejects_invalid_value_in_assignment() {
            let result = parse_feature_flag_assignment("primary_span_matching=maybe");
            assert!(result.is_err());
        }

        #[test]
        fn rejects_empty_flag_name() {
            let result = parse_feature_flag_assignment("=true");
            assert!(result.is_err());
        }

        #[test]
        fn rejects_empty_value() {
            let result = parse_feature_flag_assignment("primary_span_matching=");
            assert!(result.is_err());
        }

        #[test]
        fn rejects_multiple_equals_signs() {
            // split_once would take the first =, so this would try to parse "true=extra" as value
            let result = parse_feature_flag_assignment("primary_span_matching=true=extra");
            assert!(result.is_err());
        }
    }

    // Case insensitivity in assignments
    mod assignment_case_insensitivity {
        use super::*;

        #[test]
        fn accepts_uppercase_flag_name() {
            let result = parse_feature_flag_assignment("PRIMARY_SPAN_MATCHING=true");
            assert!(result.is_ok());
        }

        #[test]
        fn accepts_uppercase_value() {
            let result = parse_feature_flag_assignment("path_filters=TRUE");
            assert!(result.is_ok());
            let (_, value) = result.unwrap();
            assert!(value);
        }

        #[test]
        fn accepts_mixed_case_everywhere() {
            let result = parse_feature_flag_assignment("Path_Filters=FALSE");
            assert!(result.is_ok());
            let (flag, value) = result.unwrap();
            assert_eq!(flag, FeatureFlag::PathFilters);
            assert!(!value);
        }
    }
}

// =============================================================================
// FeatureFlag Enum Tests
// =============================================================================

mod feature_flag_enum {
    use super::*;

    #[test]
    fn as_str_returns_correct_string() {
        assert_eq!(
            FeatureFlag::PrimarySpanMatching.as_str(),
            "primary_span_matching"
        );
        assert_eq!(FeatureFlag::PathFilters.as_str(), "path_filters");
    }

    #[test]
    fn default_enabled_returns_correct_defaults() {
        assert!(FeatureFlag::PrimarySpanMatching.default_enabled());
        assert!(FeatureFlag::PathFilters.default_enabled());
    }

    #[test]
    fn clone_works() {
        let flag = FeatureFlag::PrimarySpanMatching;
        let cloned = flag;
        assert_eq!(flag, cloned);
    }

    #[test]
    fn debug_trait_works() {
        let flag = FeatureFlag::PathFilters;
        let debug_str = format!("{:?}", flag);
        assert!(debug_str.contains("PathFilters"));
    }

    #[test]
    fn equality_works() {
        assert_eq!(
            FeatureFlag::PrimarySpanMatching,
            FeatureFlag::PrimarySpanMatching
        );
        assert_ne!(FeatureFlag::PrimarySpanMatching, FeatureFlag::PathFilters);
    }
}

// =============================================================================
// Constants Tests
// =============================================================================

mod constants {
    use super::*;

    #[test]
    fn true_values_has_five_entries() {
        assert_eq!(TRUE_VALUES.len(), 5);
    }

    #[test]
    fn false_values_has_five_entries() {
        assert_eq!(FALSE_VALUES.len(), 5);
    }

    #[test]
    fn true_values_are_all_lowercase() {
        for &value in TRUE_VALUES.iter() {
            assert_eq!(
                value,
                value.to_lowercase(),
                "'{}' should be lowercase",
                value
            );
        }
    }

    #[test]
    fn false_values_are_all_lowercase() {
        for &value in FALSE_VALUES.iter() {
            assert_eq!(
                value,
                value.to_lowercase(),
                "'{}' should be lowercase",
                value
            );
        }
    }

    #[test]
    fn no_overlap_between_true_and_false_values() {
        for &true_val in TRUE_VALUES.iter() {
            assert!(
                !FALSE_VALUES.contains(&true_val),
                "'{}' in both TRUE and FALSE",
                true_val
            );
        }
    }
}
