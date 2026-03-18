//! Comprehensive tests for feature flag registry functionality.

use lintdiff_feature_flags::{
    feature_flags, parse_flag, set_feature_flag, set_feature_flag_by_name,
    set_feature_flag_by_name_and_value, set_feature_flag_from_assignment,
    set_feature_flags_from_assignments, FeatureFlag,
};
use lintdiff_types::FeatureFlags;

// =============================================================================
// Registry Structure Tests
// =============================================================================

mod registry_structure {
    use super::*;

    #[test]
    fn feature_flags_returns_non_empty_slice() {
        let flags = feature_flags();
        assert!(!flags.is_empty());
    }

    #[test]
    fn feature_flags_has_two_entries() {
        let flags = feature_flags();
        assert_eq!(flags.len(), 2);
    }

    #[test]
    fn all_flags_have_valid_keys() {
        for spec in feature_flags() {
            assert!(!spec.key.is_empty());
            assert!(
                spec.key.contains('_'),
                "Key '{}' should use snake_case",
                spec.key
            );
        }
    }

    #[test]
    fn all_flags_have_descriptions() {
        for spec in feature_flags() {
            assert!(
                !spec.description.is_empty(),
                "Flag '{}' missing description",
                spec.key
            );
        }
    }

    #[test]
    fn all_keys_are_unique() {
        let flags = feature_flags();
        let mut keys = std::collections::HashSet::new();
        for spec in flags {
            assert!(keys.insert(spec.key), "Duplicate key: {}", spec.key);
        }
    }

    #[test]
    fn all_ids_are_unique() {
        let flags = feature_flags();
        let mut ids: Vec<FeatureFlag> = Vec::new();
        for spec in flags {
            assert!(!ids.contains(&spec.id), "Duplicate id found");
            ids.push(spec.id);
        }
    }

    #[test]
    fn primary_span_matching_is_registered() {
        let flags = feature_flags();
        let found = flags
            .iter()
            .any(|f| f.id == FeatureFlag::PrimarySpanMatching);
        assert!(found);
    }

    #[test]
    fn path_filters_is_registered() {
        let flags = feature_flags();
        let found = flags.iter().any(|f| f.id == FeatureFlag::PathFilters);
        assert!(found);
    }
}

// =============================================================================
// Flag Lookup Tests
// =============================================================================

mod flag_lookup {
    use super::*;

    #[test]
    fn lookup_by_name_finds_primary_span_matching() {
        let result = parse_flag("primary_span_matching");
        assert_eq!(result, Some(FeatureFlag::PrimarySpanMatching));
    }

    #[test]
    fn lookup_by_name_finds_path_filters() {
        let result = parse_flag("path_filters");
        assert_eq!(result, Some(FeatureFlag::PathFilters));
    }

    #[test]
    fn lookup_is_case_insensitive() {
        assert_eq!(
            parse_flag("PRIMARY_SPAN_MATCHING"),
            Some(FeatureFlag::PrimarySpanMatching)
        );
        assert_eq!(parse_flag("Path_Filters"), Some(FeatureFlag::PathFilters));
        assert_eq!(parse_flag("PATH_FILTERS"), Some(FeatureFlag::PathFilters));
    }

    #[test]
    fn lookup_returns_none_for_unknown() {
        assert_eq!(parse_flag("nonexistent"), None);
        assert_eq!(parse_flag("primary-span-matching"), None); // wrong separator
        assert_eq!(parse_flag("primaryspanmatching"), None); // no separator
    }
}

// =============================================================================
// Flag Defaults Tests
// =============================================================================

mod flag_defaults {
    use super::*;

    #[test]
    fn feature_flags_defaults_all_true() {
        let flags = FeatureFlags::default();
        assert!(flags.prefer_primary_spans);
        assert!(flags.path_filters);
    }

    #[test]
    fn primary_span_matching_defaults_to_true() {
        assert!(FeatureFlag::PrimarySpanMatching.default_enabled());
    }

    #[test]
    fn path_filters_defaults_to_true() {
        assert!(FeatureFlag::PathFilters.default_enabled());
    }

    #[test]
    fn spec_defaults_match_enum_defaults() {
        for spec in feature_flags() {
            assert_eq!(
                spec.default_enabled,
                spec.id.default_enabled(),
                "Spec and enum default mismatch for {:?}",
                spec.id
            );
        }
    }

    #[test]
    fn spec_keys_match_enum_as_str() {
        for spec in feature_flags() {
            assert_eq!(
                spec.key,
                spec.id.as_str(),
                "Spec key and enum as_str mismatch for {:?}",
                spec.id
            );
        }
    }
}

// =============================================================================
// Setting Flags Tests
// =============================================================================

mod setting_flags {
    use super::*;

    #[test]
    fn set_feature_flag_primary_span_matching_to_false() {
        let mut flags = FeatureFlags::default();
        set_feature_flag(&mut flags, FeatureFlag::PrimarySpanMatching, false);
        assert!(!flags.prefer_primary_spans);
    }

    #[test]
    fn set_feature_flag_path_filters_to_false() {
        let mut flags = FeatureFlags::default();
        set_feature_flag(&mut flags, FeatureFlag::PathFilters, false);
        assert!(!flags.path_filters);
    }

    #[test]
    fn set_feature_flag_to_true_when_already_true() {
        let mut flags = FeatureFlags::default();
        set_feature_flag(&mut flags, FeatureFlag::PrimarySpanMatching, true);
        assert!(flags.prefer_primary_spans);
    }

    #[test]
    fn set_feature_flag_to_false_then_true() {
        let mut flags = FeatureFlags::default();
        set_feature_flag(&mut flags, FeatureFlag::PathFilters, false);
        assert!(!flags.path_filters);
        set_feature_flag(&mut flags, FeatureFlag::PathFilters, true);
        assert!(flags.path_filters);
    }

    #[test]
    fn set_feature_flag_by_name_primary_span_matching() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_by_name(&mut flags, "primary_span_matching", false);
        assert!(result.is_ok());
        assert!(!flags.prefer_primary_spans);
    }

    #[test]
    fn set_feature_flag_by_name_path_filters() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_by_name(&mut flags, "path_filters", false);
        assert!(result.is_ok());
        assert!(!flags.path_filters);
    }

    #[test]
    fn set_feature_flag_by_name_is_case_insensitive() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_by_name(&mut flags, "PATH_FILTERS", false);
        assert!(result.is_ok());
        assert!(!flags.path_filters);
    }

    #[test]
    fn set_multiple_flags_independently() {
        let mut flags = FeatureFlags::default();
        set_feature_flag(&mut flags, FeatureFlag::PrimarySpanMatching, false);
        set_feature_flag(&mut flags, FeatureFlag::PathFilters, false);
        assert!(!flags.prefer_primary_spans);
        assert!(!flags.path_filters);

        // Set one back
        set_feature_flag(&mut flags, FeatureFlag::PrimarySpanMatching, true);
        assert!(flags.prefer_primary_spans);
        assert!(!flags.path_filters);
    }
}

// =============================================================================
// Unknown Flag Handling Tests
// =============================================================================

mod unknown_flag_handling {
    use super::*;

    #[test]
    fn set_feature_flag_by_name_rejects_unknown_flag() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_by_name(&mut flags, "unknown_flag", true);
        assert!(result.is_err());
    }

    #[test]
    fn set_feature_flag_by_name_error_message_contains_flag_name() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_by_name(&mut flags, "bad_flag", true);
        let err = result.unwrap_err();
        assert!(err.contains("bad_flag"));
        assert!(err.contains("unknown feature flag"));
    }

    #[test]
    fn set_feature_flag_by_name_and_value_rejects_unknown_flag() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_by_name_and_value(&mut flags, "nonexistent", "true");
        assert!(result.is_err());
    }

    #[test]
    fn set_feature_flag_from_assignment_rejects_unknown_flag() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_from_assignment(&mut flags, "unknown=true");
        assert!(result.is_err());
    }

    #[test]
    fn set_flags_from_assignments_stops_on_first_error() {
        let mut flags = FeatureFlags::default();
        // First one is valid, second is invalid
        let result = set_feature_flags_from_assignments(
            &mut flags,
            vec!["path_filters=false", "unknown_flag=true"],
        );
        assert!(result.is_err());
        // First flag should have been set
        assert!(!flags.path_filters);
    }

    #[test]
    fn set_flags_from_assignments_rejects_invalid_value() {
        let mut flags = FeatureFlags::default();
        let result =
            set_feature_flags_from_assignments(&mut flags, vec!["path_filters=invalid_value"]);
        assert!(result.is_err());
    }
}

// =============================================================================
// Set by Name and Value Tests
// =============================================================================

mod set_by_name_and_value {
    use super::*;

    #[test]
    fn accepts_true_value() {
        let mut flags = FeatureFlags {
            path_filters: false,
            ..Default::default()
        };
        let result = set_feature_flag_by_name_and_value(&mut flags, "path_filters", "true");
        assert!(result.is_ok());
        assert!(flags.path_filters);
    }

    #[test]
    fn accepts_false_value() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_by_name_and_value(&mut flags, "path_filters", "false");
        assert!(result.is_ok());
        assert!(!flags.path_filters);
    }

    #[test]
    fn accepts_on_off_values() {
        let mut flags = FeatureFlags::default();

        let result = set_feature_flag_by_name_and_value(&mut flags, "path_filters", "off");
        assert!(result.is_ok());
        assert!(!flags.path_filters);

        let result = set_feature_flag_by_name_and_value(&mut flags, "path_filters", "on");
        assert!(result.is_ok());
        assert!(flags.path_filters);
    }

    #[test]
    fn accepts_yes_no_values() {
        let mut flags = FeatureFlags::default();

        let result = set_feature_flag_by_name_and_value(&mut flags, "primary_span_matching", "no");
        assert!(result.is_ok());
        assert!(!flags.prefer_primary_spans);

        let result = set_feature_flag_by_name_and_value(&mut flags, "primary_span_matching", "yes");
        assert!(result.is_ok());
        assert!(flags.prefer_primary_spans);
    }

    #[test]
    fn accepts_enabled_disabled_values() {
        let mut flags = FeatureFlags::default();

        let result = set_feature_flag_by_name_and_value(&mut flags, "path_filters", "disabled");
        assert!(result.is_ok());
        assert!(!flags.path_filters);

        let result = set_feature_flag_by_name_and_value(&mut flags, "path_filters", "enabled");
        assert!(result.is_ok());
        assert!(flags.path_filters);
    }

    #[test]
    fn accepts_numeric_values() {
        let mut flags = FeatureFlags::default();

        let result = set_feature_flag_by_name_and_value(&mut flags, "path_filters", "0");
        assert!(result.is_ok());
        assert!(!flags.path_filters);

        let result = set_feature_flag_by_name_and_value(&mut flags, "path_filters", "1");
        assert!(result.is_ok());
        assert!(flags.path_filters);
    }

    #[test]
    fn rejects_invalid_value() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_by_name_and_value(&mut flags, "path_filters", "maybe");
        assert!(result.is_err());
    }

    #[test]
    fn is_case_insensitive_for_name() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_by_name_and_value(&mut flags, "PATH_FILTERS", "false");
        assert!(result.is_ok());
        assert!(!flags.path_filters);
    }

    #[test]
    fn is_case_insensitive_for_value() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_by_name_and_value(&mut flags, "path_filters", "FALSE");
        assert!(result.is_ok());
        assert!(!flags.path_filters);
    }
}

// =============================================================================
// Set from Assignment Tests
// =============================================================================

mod set_from_assignment {
    use super::*;

    #[test]
    fn parses_and_applies_valid_assignment() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_from_assignment(&mut flags, "path_filters=false");
        assert!(result.is_ok());
        assert!(!flags.path_filters);
    }

    #[test]
    fn handles_whitespace_in_assignment() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_from_assignment(&mut flags, "  path_filters  =  false  ");
        assert!(result.is_ok());
        assert!(!flags.path_filters);
    }

    #[test]
    fn returns_error_for_missing_equals() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_from_assignment(&mut flags, "path_filters_true");
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_unknown_flag() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_from_assignment(&mut flags, "unknown_flag=true");
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_invalid_value() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flag_from_assignment(&mut flags, "path_filters=maybe");
        assert!(result.is_err());
    }
}

// =============================================================================
// Set from Assignments Batch Tests
// =============================================================================

mod set_from_assignments_batch {
    use super::*;

    #[test]
    fn applies_multiple_valid_assignments() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flags_from_assignments(
            &mut flags,
            vec!["primary_span_matching=false", "path_filters=false"],
        );
        assert!(result.is_ok());
        assert!(!flags.prefer_primary_spans);
        assert!(!flags.path_filters);
    }

    #[test]
    fn applies_assignments_in_order() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flags_from_assignments(
            &mut flags,
            vec!["path_filters=false", "path_filters=true"],
        );
        assert!(result.is_ok());
        assert!(flags.path_filters); // Last one wins
    }

    #[test]
    fn handles_empty_iterator() {
        let mut flags = FeatureFlags::default();
        let result: Result<(), String> =
            set_feature_flags_from_assignments(&mut flags, Vec::<&str>::new());
        assert!(result.is_ok());
        // Defaults unchanged
        assert!(flags.prefer_primary_spans);
        assert!(flags.path_filters);
    }

    #[test]
    fn stops_on_first_error() {
        let mut flags = FeatureFlags::default();
        let result = set_feature_flags_from_assignments(
            &mut flags,
            vec![
                "path_filters=false",
                "invalid",
                "primary_span_matching=false",
            ],
        );
        assert!(result.is_err());
        // First valid assignment was applied
        assert!(!flags.path_filters);
        // Third assignment should not be processed
        assert!(flags.prefer_primary_spans);
    }

    #[test]
    fn accepts_iterator_of_strings() {
        let mut flags = FeatureFlags::default();
        let assignments = vec![
            "primary_span_matching=false".to_string(),
            "path_filters=false".to_string(),
        ];
        let result = set_feature_flags_from_assignments(&mut flags, assignments);
        assert!(result.is_ok());
        assert!(!flags.prefer_primary_spans);
        assert!(!flags.path_filters);
    }

    #[test]
    fn accepts_array_slice() {
        let mut flags = FeatureFlags::default();
        let assignments = ["primary_span_matching=false", "path_filters=false"];
        let result = set_feature_flags_from_assignments(&mut flags, assignments);
        assert!(result.is_ok());
        assert!(!flags.prefer_primary_spans);
        assert!(!flags.path_filters);
    }
}

// =============================================================================
// FeatureFlagSpec Tests
// =============================================================================

mod feature_flag_spec {
    use super::*;

    #[test]
    fn spec_has_correct_id() {
        let flags = feature_flags();
        let primary_span_spec = flags
            .iter()
            .find(|f| f.id == FeatureFlag::PrimarySpanMatching)
            .expect("PrimarySpanMatching should exist");
        assert_eq!(primary_span_spec.id, FeatureFlag::PrimarySpanMatching);
    }

    #[test]
    fn spec_has_correct_key() {
        let flags = feature_flags();
        let path_filters_spec = flags
            .iter()
            .find(|f| f.key == "path_filters")
            .expect("path_filters should exist");
        assert_eq!(path_filters_spec.key, "path_filters");
    }

    #[test]
    fn spec_description_is_meaningful() {
        for spec in feature_flags() {
            // Description should be more than just the key
            assert!(
                spec.description.len() > spec.key.len(),
                "Description for '{}' should be more than just the key",
                spec.key
            );
        }
    }

    #[test]
    fn spec_can_be_copied() {
        let flags = feature_flags();
        let spec = flags[0];
        let copied = spec;
        assert_eq!(spec.key, copied.key);
        assert_eq!(spec.description, copied.description);
    }

    #[test]
    fn spec_debug_includes_all_fields() {
        let flags = feature_flags();
        let spec = flags[0];
        let debug = format!("{:?}", spec);
        assert!(debug.contains("id"));
        assert!(debug.contains("key"));
        assert!(debug.contains("description"));
        assert!(debug.contains("default_enabled"));
    }
}

// =============================================================================
// Integration Tests
// =============================================================================

mod integration {
    use super::*;

    #[test]
    fn full_workflow_set_all_flags_to_false() {
        let mut flags = FeatureFlags::default();

        // Verify defaults
        assert!(flags.prefer_primary_spans);
        assert!(flags.path_filters);

        // Disable all
        for spec in feature_flags() {
            set_feature_flag(&mut flags, spec.id, false);
        }

        // Verify all disabled
        assert!(!flags.prefer_primary_spans);
        assert!(!flags.path_filters);
    }

    #[test]
    fn full_workflow_toggle_individual_flags() {
        let mut flags = FeatureFlags::default();

        // Toggle primary_span_matching
        set_feature_flag(&mut flags, FeatureFlag::PrimarySpanMatching, false);
        assert!(!flags.prefer_primary_spans);
        assert!(flags.path_filters); // Should be unchanged

        // Toggle path_filters
        set_feature_flag(&mut flags, FeatureFlag::PathFilters, false);
        assert!(!flags.prefer_primary_spans);
        assert!(!flags.path_filters);

        // Toggle back
        set_feature_flag(&mut flags, FeatureFlag::PrimarySpanMatching, true);
        assert!(flags.prefer_primary_spans);
        assert!(!flags.path_filters);
    }

    #[test]
    fn full_workflow_from_assignments() {
        let mut flags = FeatureFlags::default();

        // Apply via assignments
        set_feature_flags_from_assignments(
            &mut flags,
            vec!["primary_span_matching=off", "path_filters=disabled"],
        )
        .unwrap();

        assert!(!flags.prefer_primary_spans);
        assert!(!flags.path_filters);
    }

    #[test]
    fn full_workflow_mixed_operations() {
        let mut flags = FeatureFlags::default();

        // Set via different methods
        set_feature_flag(&mut flags, FeatureFlag::PrimarySpanMatching, false);
        set_feature_flag_by_name(&mut flags, "path_filters", false).unwrap();
        set_feature_flag_from_assignment(&mut flags, "primary_span_matching=yes").unwrap();

        assert!(flags.prefer_primary_spans);
        assert!(!flags.path_filters);
    }

    #[test]
    fn verify_registry_consistency() {
        // For each flag in the registry, verify:
        // 1. Can be looked up by key
        // 2. as_str() matches the key
        // 3. default_enabled() matches spec
        for spec in feature_flags() {
            let looked_up = parse_flag(spec.key);
            assert_eq!(looked_up, Some(spec.id), "Lookup failed for {}", spec.key);
            assert_eq!(
                spec.id.as_str(),
                spec.key,
                "as_str mismatch for {:?}",
                spec.id
            );
            assert_eq!(
                spec.id.default_enabled(),
                spec.default_enabled,
                "Default mismatch for {:?}",
                spec.id
            );
        }
    }
}
