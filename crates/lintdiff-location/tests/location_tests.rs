//! Comprehensive tests for lintdiff-location crate.
//!
//! Test coverage:
//! 1. Location creation methods (10 tests)
//! 2. Location accessor methods (8 tests)
//! 3. Location with_line/with_column builders (6 tests)
//! 4. Location path operations (8 tests)
//! 5. Location Display formatting (5 tests)
//! 6. LocationRange creation and methods (10 tests)
//! 7. LocationRange contains_line (5 tests)
//! 8. parse_location function (10 tests)
//! 9. Error cases (3 tests)

use lintdiff_location::{parse_location, Location, LocationParseError, LocationRange};

// =============================================================================
// 1. Location creation methods (10 tests)
// =============================================================================

#[test]
fn test_file_creates_file_only_location() {
    let loc = Location::file("src/lib.rs");
    assert_eq!(loc.path(), "src/lib.rs");
    assert!(loc.line_number().is_none());
    assert!(loc.column().is_none());
}

#[test]
fn test_file_accepts_string() {
    let path = String::from("src/lib.rs");
    let loc = Location::file(path);
    assert_eq!(loc.path(), "src/lib.rs");
}

#[test]
fn test_line_creates_location_with_line() {
    let loc = Location::line("src/lib.rs", 42);
    assert_eq!(loc.path(), "src/lib.rs");
    assert_eq!(loc.line_number(), Some(42));
    assert!(loc.column().is_none());
}

#[test]
fn test_line_with_line_one() {
    let loc = Location::line("src/lib.rs", 1);
    assert_eq!(loc.line_number(), Some(1));
}

#[test]
fn test_line_with_large_line_number() {
    let loc = Location::line("src/lib.rs", 1000000);
    assert_eq!(loc.line_number(), Some(1000000));
}

#[test]
fn test_new_creates_full_location() {
    let loc = Location::new("src/lib.rs", 42, 10);
    assert_eq!(loc.path(), "src/lib.rs");
    assert_eq!(loc.line_number(), Some(42));
    assert_eq!(loc.column(), Some(10));
}

#[test]
fn test_new_with_column_one() {
    let loc = Location::new("src/lib.rs", 42, 1);
    assert_eq!(loc.column(), Some(1));
}

#[test]
fn test_from_parts_with_all_none() {
    let loc = Location::from_parts("src/lib.rs", None, None);
    assert_eq!(loc.path(), "src/lib.rs");
    assert!(loc.line_number().is_none());
    assert!(loc.column().is_none());
}

#[test]
fn test_from_parts_with_line_only() {
    let loc = Location::from_parts("src/lib.rs", Some(42), None);
    assert_eq!(loc.line_number(), Some(42));
    assert!(loc.column().is_none());
}

#[test]
fn test_from_parts_with_all_values() {
    let loc = Location::from_parts("src/lib.rs", Some(42), Some(10));
    assert_eq!(loc.line_number(), Some(42));
    assert_eq!(loc.column(), Some(10));
}

// =============================================================================
// 2. Location accessor methods (8 tests)
// =============================================================================

#[test]
fn test_path_returns_path() {
    let loc = Location::file("src/lib.rs");
    assert_eq!(loc.path(), "src/lib.rs");
}

#[test]
fn test_line_returns_some_when_present() {
    let loc = Location::line("src/lib.rs", 42);
    assert_eq!(loc.line_number(), Some(42));
}

#[test]
fn test_line_returns_none_when_absent() {
    let loc = Location::file("src/lib.rs");
    assert!(loc.line_number().is_none());
}

#[test]
fn test_column_returns_some_when_present() {
    let loc = Location::new("src/lib.rs", 42, 10);
    assert_eq!(loc.column(), Some(10));
}

#[test]
fn test_column_returns_none_when_absent() {
    let loc = Location::line("src/lib.rs", 42);
    assert!(loc.column().is_none());
}

#[test]
fn test_has_line_returns_true_when_present() {
    let loc = Location::line("src/lib.rs", 42);
    assert!(loc.has_line());
}

#[test]
fn test_has_column_returns_true_when_present() {
    let loc = Location::new("src/lib.rs", 42, 10);
    assert!(loc.has_column());
}

#[test]
fn test_is_file_only_returns_true_for_file_only() {
    let loc = Location::file("src/lib.rs");
    assert!(loc.is_file_only());
}

// =============================================================================
// 3. Location with_line/with_column builders (6 tests)
// =============================================================================

#[test]
fn test_with_line_adds_line() {
    let loc = Location::file("src/lib.rs").with_line(42);
    assert_eq!(loc.line_number(), Some(42));
}

#[test]
fn test_with_line_overwrites_existing_line() {
    let loc = Location::line("src/lib.rs", 10).with_line(42);
    assert_eq!(loc.line_number(), Some(42));
}

#[test]
fn test_with_line_preserves_column() {
    let loc = Location::new("src/lib.rs", 10, 5).with_line(42);
    assert_eq!(loc.line_number(), Some(42));
    assert_eq!(loc.column(), Some(5));
}

#[test]
fn test_with_column_adds_column() {
    let loc = Location::line("src/lib.rs", 42).with_column(10);
    assert_eq!(loc.column(), Some(10));
}

#[test]
fn test_with_column_overwrites_existing_column() {
    let loc = Location::new("src/lib.rs", 42, 5).with_column(10);
    assert_eq!(loc.column(), Some(10));
}

#[test]
fn test_with_line_and_with_column_chained() {
    let loc = Location::file("src/lib.rs").with_line(42).with_column(10);
    assert_eq!(loc.line_number(), Some(42));
    assert_eq!(loc.column(), Some(10));
}

// =============================================================================
// 4. Location path operations (8 tests)
// =============================================================================

#[test]
fn test_as_path_returns_path() {
    let loc = Location::file("src/lib.rs");
    let path = loc.as_path();
    assert!(path.to_str() == Some("src/lib.rs"));
}

#[test]
fn test_extension_returns_extension() {
    let loc = Location::file("src/lib.rs");
    assert_eq!(loc.extension(), Some("rs"));
}

#[test]
fn test_extension_returns_none_for_no_extension() {
    let loc = Location::file("src/lib");
    assert!(loc.extension().is_none());
}

#[test]
fn test_file_name_returns_name() {
    let loc = Location::file("src/lib.rs");
    assert_eq!(loc.file_name(), Some("lib.rs"));
}

#[test]
fn test_file_name_returns_name_for_directory_with_slash() {
    // Note: std::path::Path returns "src" for "src/" not None
    let loc = Location::file("src/");
    assert_eq!(loc.file_name(), Some("src"));
}

#[test]
fn test_parent_returns_parent() {
    let loc = Location::file("src/lib.rs");
    assert_eq!(loc.parent(), Some("src"));
}

#[test]
fn test_parent_returns_empty_for_file_in_root() {
    // Note: On some platforms, parent of "lib.rs" returns "" not None
    let loc = Location::file("lib.rs");
    // Either None or empty string is acceptable
    let parent = loc.parent();
    assert!(parent.is_none() || parent == Some(""));
}

#[test]
fn test_matches_path_with_suffix() {
    let loc = Location::file("src/lib.rs");
    assert!(loc.matches_path("lib.rs"));
    assert!(loc.matches_path("src/lib.rs"));
    assert!(!loc.matches_path("main.rs"));
}

// =============================================================================
// 5. Location Display formatting (5 tests)
// =============================================================================

#[test]
fn test_display_file_only_format() {
    let loc = Location::file("src/lib.rs");
    assert_eq!(format!("{}", loc), "src/lib.rs");
}

#[test]
fn test_display_with_line_format() {
    let loc = Location::line("src/lib.rs", 42);
    assert_eq!(format!("{}", loc), "src/lib.rs:42");
}

#[test]
fn test_display_with_line_and_column_format() {
    let loc = Location::new("src/lib.rs", 42, 10);
    assert_eq!(format!("{}", loc), "src/lib.rs:42:10");
}

#[test]
fn test_display_with_windows_path() {
    let loc = Location::file("src\\lib.rs");
    assert_eq!(format!("{}", loc), "src\\lib.rs");
}

#[test]
fn test_display_with_absolute_path() {
    let loc = Location::line("/home/user/project/src/lib.rs", 42);
    assert_eq!(format!("{}", loc), "/home/user/project/src/lib.rs:42");
}

// =============================================================================
// 6. LocationRange creation and methods (10 tests)
// =============================================================================

#[test]
fn test_location_range_new() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert_eq!(range.path(), "src/lib.rs");
    assert_eq!(range.start_line(), 10);
    assert_eq!(range.end_line(), 20);
}

#[test]
fn test_location_range_with_columns() {
    let range = LocationRange::with_columns("src/lib.rs", 10, 5, 20, 15);
    assert_eq!(range.start_line(), 10);
    assert_eq!(range.start_column(), Some(5));
    assert_eq!(range.end_line(), 20);
    assert_eq!(range.end_column(), Some(15));
}

#[test]
fn test_location_range_path() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert_eq!(range.path(), "src/lib.rs");
}

#[test]
fn test_location_range_start_line() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert_eq!(range.start_line(), 10);
}

#[test]
fn test_location_range_end_line() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert_eq!(range.end_line(), 20);
}

#[test]
fn test_location_range_start_column_none() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert!(range.start_column().is_none());
}

#[test]
fn test_location_range_end_column_none() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert!(range.end_column().is_none());
}

#[test]
fn test_location_range_is_single_line_true() {
    let range = LocationRange::new("src/lib.rs", 10, 10);
    assert!(range.is_single_line());
}

#[test]
fn test_location_range_is_single_line_false() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert!(!range.is_single_line());
}

#[test]
fn test_location_range_line_count() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert_eq!(range.line_count(), 11);
}

#[test]
fn test_location_range_line_count_single() {
    let range = LocationRange::new("src/lib.rs", 10, 10);
    assert_eq!(range.line_count(), 1);
}

// =============================================================================
// 7. LocationRange contains_line (5 tests)
// =============================================================================

#[test]
fn test_contains_line_at_start() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert!(range.contains_line(10));
}

#[test]
fn test_contains_line_at_end() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert!(range.contains_line(20));
}

#[test]
fn test_contains_line_in_middle() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert!(range.contains_line(15));
}

#[test]
fn test_contains_line_before_range() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert!(!range.contains_line(5));
}

#[test]
fn test_contains_line_after_range() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert!(!range.contains_line(25));
}

// =============================================================================
// 8. parse_location function (10 tests)
// =============================================================================

#[test]
fn test_parse_file_only() {
    let loc = parse_location("src/lib.rs").unwrap();
    assert_eq!(loc.path(), "src/lib.rs");
    assert!(loc.line_number().is_none());
    assert!(loc.column().is_none());
}

#[test]
fn test_parse_with_line() {
    let loc = parse_location("src/lib.rs:42").unwrap();
    assert_eq!(loc.path(), "src/lib.rs");
    assert_eq!(loc.line_number(), Some(42));
    assert!(loc.column().is_none());
}

#[test]
fn test_parse_with_line_and_column() {
    let loc = parse_location("src/lib.rs:42:10").unwrap();
    assert_eq!(loc.path(), "src/lib.rs");
    assert_eq!(loc.line_number(), Some(42));
    assert_eq!(loc.column(), Some(10));
}

#[test]
fn test_parse_with_absolute_path() {
    let loc = parse_location("/home/user/src/lib.rs").unwrap();
    assert_eq!(loc.path(), "/home/user/src/lib.rs");
}

#[test]
fn test_parse_with_absolute_path_and_line() {
    let loc = parse_location("/home/user/src/lib.rs:42").unwrap();
    assert_eq!(loc.path(), "/home/user/src/lib.rs");
    assert_eq!(loc.line_number(), Some(42));
}

#[test]
fn test_parse_with_windows_path_relative() {
    // Windows absolute paths with drive letters don't work well with our simple parser
    // Use relative path with backslashes instead
    let loc = parse_location("src\\lib.rs:42").unwrap();
    assert_eq!(loc.path(), "src\\lib.rs");
    assert_eq!(loc.line_number(), Some(42));
}

#[test]
fn test_parse_with_line_one() {
    let loc = parse_location("src/lib.rs:1").unwrap();
    assert_eq!(loc.line_number(), Some(1));
}

#[test]
fn test_parse_with_column_one() {
    let loc = parse_location("src/lib.rs:42:1").unwrap();
    assert_eq!(loc.column(), Some(1));
}

#[test]
fn test_parse_with_large_numbers() {
    let loc = parse_location("src/lib.rs:999999:888888").unwrap();
    assert_eq!(loc.line_number(), Some(999999));
    assert_eq!(loc.column(), Some(888888));
}

#[test]
fn test_parse_empty_path() {
    let loc = parse_location("").unwrap();
    assert_eq!(loc.path(), "");
}

// =============================================================================
// 9. Error cases (3 tests)
// =============================================================================

#[test]
fn test_parse_invalid_line_returns_error() {
    let result = parse_location("src/lib.rs:abc");
    assert!(result.is_err());
    match result {
        Err(LocationParseError::InvalidLine(s)) => assert_eq!(s, "abc"),
        _ => panic!("Expected InvalidLine error"),
    }
}

#[test]
fn test_parse_invalid_column_returns_error() {
    let result = parse_location("src/lib.rs:42:abc");
    assert!(result.is_err());
    match result {
        Err(LocationParseError::InvalidColumn(s)) => assert_eq!(s, "abc"),
        _ => panic!("Expected InvalidColumn error"),
    }
}

#[test]
fn test_parse_invalid_line_in_three_part_format() {
    let result = parse_location("src/lib.rs:xyz:10");
    assert!(result.is_err());
}

// =============================================================================
// Additional tests for From implementations
// =============================================================================

#[test]
fn test_from_str() {
    let loc = Location::from("src/lib.rs");
    assert_eq!(loc.path(), "src/lib.rs");
    assert!(loc.is_file_only());
}

#[test]
fn test_from_string() {
    let path = String::from("src/lib.rs");
    let loc = Location::from(path);
    assert_eq!(loc.path(), "src/lib.rs");
}

#[test]
fn test_from_path_buf() {
    use std::path::PathBuf;
    let path = PathBuf::from("src/lib.rs");
    let loc = Location::from(path);
    assert_eq!(loc.path(), "src/lib.rs");
}

// =============================================================================
// Additional tests for with_path
// =============================================================================

#[test]
fn test_with_path_changes_path() {
    let loc = Location::line("src/lib.rs", 42);
    let new_loc = loc.with_path("src/main.rs");
    assert_eq!(new_loc.path(), "src/main.rs");
    assert_eq!(new_loc.line_number(), Some(42));
}

#[test]
fn test_with_path_preserves_line_and_column() {
    let loc = Location::new("src/lib.rs", 42, 10);
    let new_loc = loc.with_path("src/main.rs");
    assert_eq!(new_loc.path(), "src/main.rs");
    assert_eq!(new_loc.line_number(), Some(42));
    assert_eq!(new_loc.column(), Some(10));
}

// =============================================================================
// Additional tests for LocationRange start/end
// =============================================================================

#[test]
fn test_location_range_start() {
    let range = LocationRange::with_columns("src/lib.rs", 10, 5, 20, 15);
    let start = range.start();
    assert_eq!(start.path(), "src/lib.rs");
    assert_eq!(start.line_number(), Some(10));
    assert_eq!(start.column(), Some(5));
}

#[test]
fn test_location_range_end() {
    let range = LocationRange::with_columns("src/lib.rs", 10, 5, 20, 15);
    let end = range.end();
    assert_eq!(end.path(), "src/lib.rs");
    assert_eq!(end.line_number(), Some(20));
    assert_eq!(end.column(), Some(15));
}

#[test]
fn test_location_range_start_without_columns() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    let start = range.start();
    assert_eq!(start.line_number(), Some(10));
    assert!(start.column().is_none());
}

#[test]
fn test_location_range_end_without_columns() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    let end = range.end();
    assert_eq!(end.line_number(), Some(20));
    assert!(end.column().is_none());
}

// =============================================================================
// Additional tests for LocationRange Display
// =============================================================================

#[test]
fn test_location_range_display_without_columns() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    assert_eq!(format!("{}", range), "src/lib.rs:10-20");
}

#[test]
fn test_location_range_display_with_columns() {
    let range = LocationRange::with_columns("src/lib.rs", 10, 5, 20, 15);
    assert_eq!(format!("{}", range), "src/lib.rs:10-20:5-15");
}

#[test]
fn test_location_range_display_single_line() {
    let range = LocationRange::new("src/lib.rs", 10, 10);
    assert_eq!(format!("{}", range), "src/lib.rs:10-10");
}

// =============================================================================
// Clone and PartialEq tests
// =============================================================================

#[test]
fn test_location_clone() {
    let loc = Location::new("src/lib.rs", 42, 10);
    let cloned = loc.clone();
    assert_eq!(loc, cloned);
}

#[test]
fn test_location_eq() {
    let loc1 = Location::new("src/lib.rs", 42, 10);
    let loc2 = Location::new("src/lib.rs", 42, 10);
    assert_eq!(loc1, loc2);
}

#[test]
fn test_location_neq_path() {
    let loc1 = Location::file("src/lib.rs");
    let loc2 = Location::file("src/main.rs");
    assert_ne!(loc1, loc2);
}

#[test]
fn test_location_neq_line() {
    let loc1 = Location::line("src/lib.rs", 42);
    let loc2 = Location::line("src/lib.rs", 43);
    assert_ne!(loc1, loc2);
}

#[test]
fn test_location_range_clone() {
    let range = LocationRange::new("src/lib.rs", 10, 20);
    let cloned = range.clone();
    assert_eq!(range, cloned);
}

#[test]
fn test_location_range_eq() {
    let range1 = LocationRange::new("src/lib.rs", 10, 20);
    let range2 = LocationRange::new("src/lib.rs", 10, 20);
    assert_eq!(range1, range2);
}

#[test]
fn test_location_range_neq() {
    let range1 = LocationRange::new("src/lib.rs", 10, 20);
    let range2 = LocationRange::new("src/lib.rs", 10, 21);
    assert_ne!(range1, range2);
}

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
fn test_location_with_special_chars_in_path() {
    let loc = Location::file("src/lib-test_v2.rs");
    assert_eq!(loc.path(), "src/lib-test_v2.rs");
}

#[test]
fn test_location_with_dot_in_path() {
    let loc = Location::file("./src/lib.rs");
    assert_eq!(loc.path(), "./src/lib.rs");
}

#[test]
fn test_location_with_double_dot_in_path() {
    let loc = Location::file("../src/lib.rs");
    assert_eq!(loc.path(), "../src/lib.rs");
}

#[test]
fn test_matches_path_exact_match() {
    let loc = Location::file("src/lib.rs");
    assert!(loc.matches_path("src/lib.rs"));
}

#[test]
fn test_matches_path_partial_match() {
    let loc = Location::file("crates/lintdiff-location/src/lib.rs");
    assert!(loc.matches_path("src/lib.rs"));
}

#[test]
fn test_matches_path_no_match() {
    let loc = Location::file("src/lib.rs");
    assert!(!loc.matches_path("main.rs"));
}

#[test]
fn test_parse_with_dot_in_filename() {
    let loc = parse_location("src/lib.test.rs:42").unwrap();
    assert_eq!(loc.path(), "src/lib.test.rs");
    assert_eq!(loc.line_number(), Some(42));
}

#[test]
fn test_extension_with_multiple_dots() {
    let loc = Location::file("src/lib.test.rs");
    assert_eq!(loc.extension(), Some("rs"));
}
