//! Comprehensive tests for lintdiff-schema-diff.
//!
//! These tests cover all public API functions and edge cases.

use lintdiff_schema_diff::*;
use serde_json::json;

// ============================================================================
// PathSegment Tests
// ============================================================================

mod path_segment_tests {
    use super::*;

    #[test]
    fn key_creates_key_segment() {
        let seg = PathSegment::key("foo".to_string());
        assert!(seg.is_key());
        assert!(!seg.is_index());
    }

    #[test]
    fn index_creates_index_segment() {
        let seg = PathSegment::index(5);
        assert!(seg.is_index());
        assert!(!seg.is_key());
    }

    #[test]
    fn display_key() {
        let seg = PathSegment::key("foo".to_string());
        assert_eq!(seg.to_string(), ".foo");
    }

    #[test]
    fn display_index() {
        let seg = PathSegment::index(5);
        assert_eq!(seg.to_string(), "[5]");
    }
}

// ============================================================================
// format_path Tests
// ============================================================================

mod format_path_tests {
    use super::*;

    #[test]
    fn empty_path_is_root() {
        assert_eq!(format_path(&[]), "$");
    }

    #[test]
    fn single_key() {
        let path = vec![PathSegment::key("foo".to_string())];
        assert_eq!(format_path(&path), "$.foo");
    }

    #[test]
    fn single_index() {
        let path = vec![PathSegment::index(0)];
        assert_eq!(format_path(&path), "$[0]");
    }

    #[test]
    fn mixed_path() {
        let path = vec![
            PathSegment::key("users".to_string()),
            PathSegment::index(0),
            PathSegment::key("name".to_string()),
        ];
        assert_eq!(format_path(&path), "$.users[0].name");
    }

    #[test]
    fn nested_arrays() {
        let path = vec![
            PathSegment::key("matrix".to_string()),
            PathSegment::index(0),
            PathSegment::index(1),
        ];
        assert_eq!(format_path(&path), "$.matrix[0][1]");
    }
}

// ============================================================================
// DiffKind Tests
// ============================================================================

mod diff_kind_tests {
    use super::*;

    #[test]
    fn display_added() {
        assert_eq!(DiffKind::Added.to_string(), "added");
    }

    #[test]
    fn display_removed() {
        assert_eq!(DiffKind::Removed.to_string(), "removed");
    }

    #[test]
    fn display_changed() {
        assert_eq!(DiffKind::Changed.to_string(), "changed");
    }

    #[test]
    fn display_type_changed() {
        assert_eq!(DiffKind::TypeChanged.to_string(), "type changed");
    }

    #[test]
    fn display_array_length_changed() {
        assert_eq!(DiffKind::ArrayLengthChanged.to_string(), "array length changed");
    }

    #[test]
    fn display_keys_changed() {
        assert_eq!(DiffKind::KeysChanged.to_string(), "keys changed");
    }
}

// ============================================================================
// JsonDiff Tests
// ============================================================================

mod json_diff_tests {
    use super::*;

    #[test]
    fn new_creates_diff() {
        let diff = JsonDiff::new(
            vec![PathSegment::key("foo".to_string())],
            DiffKind::Changed,
            Some(json!(1)),
            Some(json!(2)),
        );
        assert_eq!(diff.path.len(), 1);
        assert_eq!(diff.kind, DiffKind::Changed);
    }

    #[test]
    fn added_creates_addition() {
        let diff = JsonDiff::added(vec![], json!(1));
        assert_eq!(diff.kind, DiffKind::Added);
        assert!(diff.old_value.is_none());
        assert_eq!(diff.new_value, Some(json!(1)));
    }

    #[test]
    fn removed_creates_removal() {
        let diff = JsonDiff::removed(vec![], json!(1));
        assert_eq!(diff.kind, DiffKind::Removed);
        assert_eq!(diff.old_value, Some(json!(1)));
        assert!(diff.new_value.is_none());
    }

    #[test]
    fn changed_creates_change() {
        let diff = JsonDiff::changed(vec![], json!(1), json!(2));
        assert_eq!(diff.kind, DiffKind::Changed);
        assert_eq!(diff.old_value, Some(json!(1)));
        assert_eq!(diff.new_value, Some(json!(2)));
    }

    #[test]
    fn path_string_formats_path() {
        let diff = JsonDiff::added(
            vec![PathSegment::key("foo".to_string()), PathSegment::index(0)],
            json!(1),
        );
        assert_eq!(diff.path_string(), "$.foo[0]");
    }

    #[test]
    fn display_shows_path_and_kind() {
        let diff = JsonDiff::added(vec![PathSegment::key("foo".to_string())], json!(1));
        assert_eq!(diff.to_string(), "$.foo: added");
    }
}

// ============================================================================
// diff_json Tests
// ============================================================================

mod diff_json_tests {
    use super::*;

    #[test]
    fn equal_values_no_diff() {
        let a = json!(1);
        let b = json!(1);
        assert!(diff_json(&a, &b).is_empty());
    }

    #[test]
    fn equal_strings_no_diff() {
        let a = json!("hello");
        let b = json!("hello");
        assert!(diff_json(&a, &b).is_empty());
    }

    #[test]
    fn equal_objects_no_diff() {
        let a = json!({"a": 1, "b": 2});
        let b = json!({"a": 1, "b": 2});
        assert!(diff_json(&a, &b).is_empty());
    }

    #[test]
    fn equal_arrays_no_diff() {
        let a = json!([1, 2, 3]);
        let b = json!([1, 2, 3]);
        assert!(diff_json(&a, &b).is_empty());
    }

    #[test]
    fn null_equal() {
        let a = json!(null);
        let b = json!(null);
        assert!(diff_json(&a, &b).is_empty());
    }

    #[test]
    fn bool_equal() {
        let a = json!(true);
        let b = json!(true);
        assert!(diff_json(&a, &b).is_empty());
    }

    #[test]
    fn primitive_change() {
        let a = json!(1);
        let b = json!(2);
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].kind, DiffKind::Changed);
    }

    #[test]
    fn object_key_added() {
        let a = json!({"a": 1});
        let b = json!({"a": 1, "b": 2});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].kind, DiffKind::Added);
    }

    #[test]
    fn object_key_removed() {
        let a = json!({"a": 1, "b": 2});
        let b = json!({"a": 1});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].kind, DiffKind::Removed);
    }

    #[test]
    fn object_value_changed() {
        let a = json!({"a": 1});
        let b = json!({"a": 2});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].kind, DiffKind::Changed);
    }

    #[test]
    fn nested_object_change() {
        let a = json!({"user": {"name": "Alice"}});
        let b = json!({"user": {"name": "Bob"}});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string(), "$.user.name");
    }

    #[test]
    fn array_element_added() {
        let a = json!([1, 2]);
        let b = json!([1, 2, 3]);
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].kind, DiffKind::Added);
    }

    #[test]
    fn array_element_removed() {
        let a = json!([1, 2, 3]);
        let b = json!([1, 2]);
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].kind, DiffKind::Removed);
    }

    #[test]
    fn array_element_changed() {
        let a = json!([1, 2, 3]);
        let b = json!([1, 99, 3]);
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string(), "$[1]");
    }

    #[test]
    fn type_change_primitive_to_object() {
        let a = json!(1);
        let b = json!({"a": 1});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].kind, DiffKind::Changed);
    }

    #[test]
    fn type_change_object_to_array() {
        let a = json!({"0": "a"});
        let b = json!(["a"]);
        let diffs = diff_json(&a, &b);
        assert!(!diffs.is_empty());
    }

    #[test]
    fn multiple_changes() {
        let a = json!({"a": 1, "b": 2, "c": 3});
        let b = json!({"a": 1, "b": 3, "d": 4});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 3); // b changed, c removed, d added
    }

    #[test]
    fn deeply_nested_change() {
        let a = json!({"a": {"b": {"c": {"d": 1}}}});
        let b = json!({"a": {"b": {"c": {"d": 2}}}});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string(), "$.a.b.c.d");
    }
}

// ============================================================================
// SchemaDiffConfig Tests
// ============================================================================

mod schema_diff_config_tests {
    use super::*;

    #[test]
    fn new_creates_empty_config() {
        let config = SchemaDiffConfig::new();
        assert!(config.ignore_keys.is_empty());
        assert!(config.equivalent_keys.is_empty());
        assert!(!config.ignore_array_order);
    }

    #[test]
    fn with_ignore_key_adds_key() {
        let config = SchemaDiffConfig::new().with_ignore_key("timestamp");
        assert!(config.ignore_keys.contains("timestamp"));
    }

    #[test]
    fn with_ignore_key_chainable() {
        let config = SchemaDiffConfig::new()
            .with_ignore_key("timestamp")
            .with_ignore_key("id");
        assert!(config.ignore_keys.contains("timestamp"));
        assert!(config.ignore_keys.contains("id"));
    }
}

// ============================================================================
// SchemaDiff Tests
// ============================================================================

mod schema_diff_tests {
    use super::*;

    #[test]
    fn new_creates_empty_diff() {
        let diff = SchemaDiff::new();
        assert!(!diff.has_changes());
        assert_eq!(diff.change_count(), 0);
    }

    #[test]
    fn compare_equal_no_changes() {
        let a = json!({"a": 1});
        let b = json!({"a": 1});
        let diff = SchemaDiff::compare(&a, &b);
        assert!(!diff.has_changes());
    }

    #[test]
    fn compare_different_has_changes() {
        let a = json!({"a": 1});
        let b = json!({"a": 2});
        let diff = SchemaDiff::compare(&a, &b);
        assert!(diff.has_changes());
    }

    #[test]
    fn change_count() {
        let a = json!({"a": 1, "b": 2});
        let b = json!({"a": 2, "b": 3});
        let diff = SchemaDiff::compare(&a, &b);
        assert_eq!(diff.change_count(), 2);
    }

    #[test]
    fn changes_of_kind() {
        let a = json!({"a": 1, "b": 2});
        let b = json!({"a": 2, "c": 3});
        let diff = SchemaDiff::compare(&a, &b);

        let added = diff.changes_of_kind(DiffKind::Added);
        assert_eq!(added.len(), 1);

        let removed = diff.changes_of_kind(DiffKind::Removed);
        assert_eq!(removed.len(), 1);

        let changed = diff.changes_of_kind(DiffKind::Changed);
        assert_eq!(changed.len(), 1);
    }

    #[test]
    fn added_removed_changed_helpers() {
        let a = json!({"a": 1, "b": 2, "c": 3});
        let b = json!({"a": 2, "b": 2, "d": 4});
        let diff = SchemaDiff::compare(&a, &b);

        assert_eq!(diff.added().len(), 1); // d
        assert_eq!(diff.removed().len(), 1); // c
        assert_eq!(diff.changed().len(), 1); // a
    }

    #[test]
    fn compare_with_config_ignores_keys() {
        let config = SchemaDiffConfig::new().with_ignore_key("timestamp");
        let a = json!({"a": 1, "timestamp": "2024-01-01"});
        let b = json!({"a": 1, "timestamp": "2024-01-02"});

        let diff = SchemaDiff::compare_with_config(&a, &b, config);
        assert!(!diff.has_changes());
    }

    #[test]
    fn compare_with_config_without_ignore() {
        let a = json!({"a": 1, "timestamp": "2024-01-01"});
        let b = json!({"a": 1, "timestamp": "2024-01-02"});

        let diff = SchemaDiff::compare(&a, &b);
        assert!(diff.has_changes());
    }
}

// ============================================================================
// SchemaDiffSummary Tests
// ============================================================================

mod schema_diff_summary_tests {
    use super::*;

    #[test]
    fn from_diff_empty() {
        let diff = SchemaDiff::new();
        let summary = SchemaDiffSummary::from_diff(&diff);

        assert_eq!(summary.total_changes, 0);
        assert!(!summary.has_changes());
    }

    #[test]
    fn from_diff_with_changes() {
        let a = json!({"a": 1, "b": 2, "c": 3});
        let b = json!({"a": 2, "b": 2, "d": 4});
        let diff = SchemaDiff::compare(&a, &b);
        let summary = SchemaDiffSummary::from_diff(&diff);

        assert_eq!(summary.total_changes, 3);
        assert_eq!(summary.additions, 1); // d
        assert_eq!(summary.removals, 1); // c
        assert_eq!(summary.modifications, 1); // a
    }

    #[test]
    fn has_changes() {
        let summary = SchemaDiffSummary {
            total_changes: 1,
            additions: 1,
            removals: 0,
            modifications: 0,
            added_paths: vec!["$a".to_string()],
            removed_paths: vec![],
            modified_paths: vec![],
        };
        assert!(summary.has_changes());
    }

    #[test]
    fn has_breaking_changes_with_removal() {
        let summary = SchemaDiffSummary {
            total_changes: 1,
            additions: 0,
            removals: 1,
            modifications: 0,
            added_paths: vec![],
            removed_paths: vec!["$a".to_string()],
            modified_paths: vec![],
        };
        assert!(summary.has_breaking_changes());
    }

    #[test]
    fn has_breaking_changes_with_modification() {
        let summary = SchemaDiffSummary {
            total_changes: 1,
            additions: 0,
            removals: 0,
            modifications: 1,
            added_paths: vec![],
            removed_paths: vec![],
            modified_paths: vec!["$a".to_string()],
        };
        assert!(summary.has_breaking_changes());
    }

    #[test]
    fn no_breaking_changes_with_only_additions() {
        let summary = SchemaDiffSummary {
            total_changes: 1,
            additions: 1,
            removals: 0,
            modifications: 0,
            added_paths: vec!["$a".to_string()],
            removed_paths: vec![],
            modified_paths: vec![],
        };
        assert!(!summary.has_breaking_changes());
    }

    #[test]
    fn display() {
        let summary = SchemaDiffSummary {
            total_changes: 5,
            additions: 2,
            removals: 1,
            modifications: 2,
            added_paths: vec![],
            removed_paths: vec![],
            modified_paths: vec![],
        };
        assert_eq!(
            summary.to_string(),
            "Schema diff: 5 changes (2 added, 1 removed, 2 modified)"
        );
    }
}

// ============================================================================
// json_eq Tests
// ============================================================================

mod json_eq_tests {
    use super::*;

    #[test]
    fn equal_primitives() {
        assert!(json_eq(&json!(1), &json!(1)));
        assert!(json_eq(&json!("hello"), &json!("hello")));
        assert!(json_eq(&json!(true), &json!(true)));
        assert!(json_eq(&json!(null), &json!(null)));
    }

    #[test]
    fn unequal_primitives() {
        assert!(!json_eq(&json!(1), &json!(2)));
        assert!(!json_eq(&json!("hello"), &json!("world")));
        assert!(!json_eq(&json!(true), &json!(false)));
    }

    #[test]
    fn equal_objects() {
        let a = json!({"a": 1, "b": 2});
        let b = json!({"a": 1, "b": 2});
        assert!(json_eq(&a, &b));
    }

    #[test]
    fn unequal_objects() {
        let a = json!({"a": 1});
        let b = json!({"a": 2});
        assert!(!json_eq(&a, &b));
    }

    #[test]
    fn equal_arrays() {
        let a = json!([1, 2, 3]);
        let b = json!([1, 2, 3]);
        assert!(json_eq(&a, &b));
    }

    #[test]
    fn unequal_arrays() {
        let a = json!([1, 2, 3]);
        let b = json!([1, 2, 4]);
        assert!(!json_eq(&a, &b));
    }
}

// ============================================================================
// json_eq_ignore_order Tests
// ============================================================================

mod json_eq_ignore_order_tests {
    use super::*;

    #[test]
    fn equal_primitives() {
        assert!(json_eq_ignore_order(&json!(1), &json!(1)));
    }

    #[test]
    fn equal_objects_different_key_order() {
        let a = json!({"a": 1, "b": 2});
        let b = json!({"b": 2, "a": 1});
        assert!(json_eq_ignore_order(&a, &b));
    }

    #[test]
    fn equal_arrays_same_order() {
        let a = json!([1, 2, 3]);
        let b = json!([1, 2, 3]);
        assert!(json_eq_ignore_order(&a, &b));
    }

    #[test]
    fn unequal_arrays_different_order() {
        // Note: This function doesn't ignore array order, only object key order
        let a = json!([1, 2, 3]);
        let b = json!([3, 2, 1]);
        assert!(!json_eq_ignore_order(&a, &b));
    }

    #[test]
    fn nested_objects() {
        let a = json!({"outer": {"inner": {"a": 1, "b": 2}}});
        let b = json!({"outer": {"inner": {"b": 2, "a": 1}}});
        assert!(json_eq_ignore_order(&a, &b));
    }
}

// ============================================================================
// merge_json Tests
// ============================================================================

mod merge_json_tests {
    use super::*;

    #[test]
    fn merge_primitives() {
        let base = json!(1);
        let overlay = json!(2);
        assert_eq!(merge_json(&base, &overlay), json!(2));
    }

    #[test]
    fn merge_objects() {
        let base = json!({"a": 1, "b": 2});
        let overlay = json!({"b": 3, "c": 4});
        let merged = merge_json(&base, &overlay);

        assert_eq!(merged["a"], 1); // From base
        assert_eq!(merged["b"], 3); // From overlay (overwrites)
        assert_eq!(merged["c"], 4); // From overlay
    }

    #[test]
    fn merge_nested() {
        let base = json!({"a": {"b": 1, "c": 2}});
        let overlay = json!({"a": {"c": 3, "d": 4}});
        let merged = merge_json(&base, &overlay);

        assert_eq!(merged["a"]["b"], 1);
        assert_eq!(merged["a"]["c"], 3);
        assert_eq!(merged["a"]["d"], 4);
    }

    #[test]
    fn merge_array_replaces() {
        let base = json!({"arr": [1, 2, 3]});
        let overlay = json!({"arr": [4, 5]});
        let merged = merge_json(&base, &overlay);

        assert_eq!(merged["arr"], json!([4, 5]));
    }

    #[test]
    fn merge_empty_base() {
        let base = json!({});
        let overlay = json!({"a": 1});
        let merged = merge_json(&base, &overlay);

        assert_eq!(merged, json!({"a": 1}));
    }

    #[test]
    fn merge_empty_overlay() {
        let base = json!({"a": 1});
        let overlay = json!({});
        let merged = merge_json(&base, &overlay);

        assert_eq!(merged, json!({"a": 1}));
    }
}

// ============================================================================
// extract_paths Tests
// ============================================================================

mod extract_paths_tests {
    use super::*;

    #[test]
    fn primitive_has_root_only() {
        let value = json!(1);
        let paths = extract_paths(&value);
        assert_eq!(paths.len(), 1);
        assert!(paths.contains(&vec![]));
    }

    #[test]
    fn object_has_keys() {
        let value = json!({"a": 1, "b": 2});
        let paths = extract_paths(&value);

        assert!(paths.contains(&vec![]));
        assert!(paths.contains(&vec![PathSegment::key("a".to_string())]));
        assert!(paths.contains(&vec![PathSegment::key("b".to_string())]));
    }

    #[test]
    fn array_has_indices() {
        let value = json!([1, 2, 3]);
        let paths = extract_paths(&value);

        assert!(paths.contains(&vec![]));
        assert!(paths.contains(&vec![PathSegment::index(0)]));
        assert!(paths.contains(&vec![PathSegment::index(1)]));
        assert!(paths.contains(&vec![PathSegment::index(2)]));
    }

    #[test]
    fn nested_paths() {
        let value = json!({"a": {"b": [1, 2]}});
        let paths = extract_paths(&value);

        assert!(paths.contains(&vec![
            PathSegment::key("a".to_string()),
            PathSegment::key("b".to_string()),
            PathSegment::index(0)
        ]));
    }
}

// ============================================================================
// get_at_path Tests
// ============================================================================

mod get_at_path_tests {
    use super::*;

    #[test]
    fn get_root() {
        let value = json!({"a": 1});
        let result = get_at_path(&value, &[]);
        assert_eq!(result, Some(json!({"a": 1})));
    }

    #[test]
    fn get_object_key() {
        let value = json!({"a": 1, "b": 2});
        let result = get_at_path(&value, &[PathSegment::key("a".to_string())]);
        assert_eq!(result, Some(json!(1)));
    }

    #[test]
    fn get_array_index() {
        let value = json!([1, 2, 3]);
        let result = get_at_path(&value, &[PathSegment::index(1)]);
        assert_eq!(result, Some(json!(2)));
    }

    #[test]
    fn get_nested() {
        let value = json!({"a": {"b": [1, 2, 3]}});
        let result = get_at_path(
            &value,
            &[
                PathSegment::key("a".to_string()),
                PathSegment::key("b".to_string()),
                PathSegment::index(2),
            ],
        );
        assert_eq!(result, Some(json!(3)));
    }

    #[test]
    fn get_missing_key() {
        let value = json!({"a": 1});
        let result = get_at_path(&value, &[PathSegment::key("b".to_string())]);
        assert_eq!(result, None);
    }

    #[test]
    fn get_out_of_bounds() {
        let value = json!([1, 2]);
        let result = get_at_path(&value, &[PathSegment::index(5)]);
        assert_eq!(result, None);
    }

    #[test]
    fn get_wrong_type() {
        let value = json!({"a": 1});
        let result = get_at_path(&value, &[PathSegment::index(0)]);
        assert_eq!(result, None);
    }
}

// ============================================================================
// set_at_path Tests
// ============================================================================

mod set_at_path_tests {
    use super::*;

    #[test]
    fn set_root() {
        let mut value = json!({"a": 1});
        set_at_path(&mut value, &[], json!({"b": 2})).unwrap();
        assert_eq!(value, json!({"b": 2}));
    }

    #[test]
    fn set_object_key() {
        let mut value = json!({"a": 1});
        set_at_path(&mut value, &[PathSegment::key("a".to_string())], json!(2)).unwrap();
        assert_eq!(value, json!({"a": 2}));
    }

    #[test]
    fn set_array_index() {
        let mut value = json!([1, 2, 3]);
        set_at_path(&mut value, &[PathSegment::index(1)], json!(99)).unwrap();
        assert_eq!(value, json!([1, 99, 3]));
    }

    #[test]
    fn set_nested() {
        let mut value = json!({"a": {"b": 1}});
        set_at_path(
            &mut value,
            &[PathSegment::key("a".to_string()), PathSegment::key("b".to_string())],
            json!(2),
        ).unwrap();
        assert_eq!(value, json!({"a": {"b": 2}}));
    }

    #[test]
    fn set_missing_key_inserts() {
        let mut value = json!({"a": 1});
        let result = set_at_path(&mut value, &[PathSegment::key("b".to_string())], json!(2));
        // Note: set_at_path inserts new keys into objects
        assert!(result.is_ok());
        assert_eq!(value, json!({"a": 1, "b": 2}));
    }

    #[test]
    fn set_out_of_bounds_fails() {
        let mut value = json!([1, 2]);
        let result = set_at_path(&mut value, &[PathSegment::index(5)], json!(99));
        assert!(result.is_err());
    }
}

// ============================================================================
// Property-based Tests
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn diff_json_reflexive_int(value in 0i64..1000i64) {
            let json_val = json!(value);
            prop_assert!(diff_json(&json_val, &json_val).is_empty());
        }

        #[test]
        fn diff_json_reflexive_string(value in ".*") {
            let json_val = json!(value);
            prop_assert!(diff_json(&json_val, &json_val).is_empty());
        }

        #[test]
        fn diff_json_reflexive_bool(value in proptest::bool::ANY) {
            let json_val = json!(value);
            prop_assert!(diff_json(&json_val, &json_val).is_empty());
        }

        #[test]
        fn json_eq_consistent_ints(a in 0i64..100i64, b in 0i64..100i64) {
            let json_a = json!(a);
            let json_b = json!(b);
            let eq = json_eq(&json_a, &json_b);
            let diff_empty = diff_json(&json_a, &json_b).is_empty();
            prop_assert_eq!(eq, diff_empty);
        }

        #[test]
        fn json_eq_consistent_strings(a in ".*", b in ".*") {
            let json_a = json!(a);
            let json_b = json!(b);
            let eq = json_eq(&json_a, &json_b);
            let diff_empty = diff_json(&json_a, &json_b).is_empty();
            prop_assert_eq!(eq, diff_empty);
        }

        #[test]
        fn merge_json_object_with_int(value in 0i64..100i64) {
            let base = json!({"a": 1});
            let overlay = json!(value);
            let merged = merge_json(&base, &overlay);
            // When overlay is a primitive, it replaces the base
            prop_assert_eq!(merged, overlay);
        }

        #[test]
        fn get_at_path_root_returns_int(value in 0i64..100i64) {
            let json_val = json!(value);
            let result = get_at_path(&json_val, &[]);
            prop_assert_eq!(result, Some(json_val));
        }

        #[test]
        fn get_at_path_root_returns_string(value in ".*") {
            let json_val = json!(value);
            let result = get_at_path(&json_val, &[]);
            prop_assert_eq!(result, Some(json_val));
        }
    }
}
