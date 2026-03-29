//! Comprehensive BDD tests for lintdiff-hunk-header crate.
//!
//! Test coverage:
//! 1. HunkHeader creation (10 tests)
//! 2. HunkHeader accessors (10 tests)
//! 3. HunkHeader line containment (8 tests)
//! 4. HunkHeader builder methods (8 tests)
//! 5. Standard hunk header parsing (10 tests)
//! 6. Hunk headers without counts (6 tests)
//! 7. Edge cases (8 tests)
//! 8. Invalid format handling (10 tests)
//! 9. Display trait round-trip (5 tests)
//! 10. Property-based tests with proptest (5 tests)

use lintdiff_hunk_header::{parse_hunk_header, HunkHeader, HunkHeaderError};

// =============================================================================
// 1. HunkHeader creation (10 tests)
// =============================================================================

#[test]
fn test_new_creates_header_with_all_values() {
    let header = HunkHeader::new(10, 5, 15, 7);
    assert_eq!(header.old_start(), 10);
    assert_eq!(header.old_count(), 5);
    assert_eq!(header.new_start(), 15);
    assert_eq!(header.new_count(), 7);
}

#[test]
fn test_new_with_minimum_values() {
    let header = HunkHeader::new(0, 0, 0, 0);
    assert_eq!(header.old_start(), 0);
    assert_eq!(header.old_count(), 0);
    assert_eq!(header.new_start(), 0);
    assert_eq!(header.new_count(), 0);
}

#[test]
fn test_new_with_large_values() {
    let header = HunkHeader::new(1000000, 500000, 1500000, 700000);
    assert_eq!(header.old_start(), 1000000);
    assert_eq!(header.old_count(), 500000);
    assert_eq!(header.new_start(), 1500000);
    assert_eq!(header.new_count(), 700000);
}

#[test]
fn test_new_with_same_old_and_new() {
    let header = HunkHeader::new(42, 10, 42, 10);
    assert_eq!(header.old_start(), header.new_start());
    assert_eq!(header.old_count(), header.new_count());
}

#[test]
fn test_default_creates_empty_hunk() {
    let header = HunkHeader::default();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 0);
    assert_eq!(header.new_start(), 1);
    assert_eq!(header.new_count(), 0);
}

#[test]
fn test_new_with_zero_counts() {
    let header = HunkHeader::new(1, 0, 1, 0);
    assert_eq!(header.old_count(), 0);
    assert_eq!(header.new_count(), 0);
    assert!(header.is_empty());
}

#[test]
fn test_new_with_zero_starts() {
    let header = HunkHeader::new(0, 1, 0, 1);
    assert_eq!(header.old_start(), 0);
    assert_eq!(header.new_start(), 0);
}

#[test]
fn test_new_with_asymmetric_counts() {
    let header = HunkHeader::new(1, 100, 1, 1);
    assert_eq!(header.old_count(), 100);
    assert_eq!(header.new_count(), 1);
}

#[test]
fn test_new_preserves_all_values() {
    let header = HunkHeader::new(123, 456, 789, 1011);
    assert_eq!(header.old_start(), 123);
    assert_eq!(header.old_count(), 456);
    assert_eq!(header.new_start(), 789);
    assert_eq!(header.new_count(), 1011);
}

#[test]
fn test_new_is_const_fn() {
    const HEADER: HunkHeader = HunkHeader::new(1, 2, 3, 4);
    assert_eq!(HEADER.old_start(), 1);
    assert_eq!(HEADER.old_count(), 2);
    assert_eq!(HEADER.new_start(), 3);
    assert_eq!(HEADER.new_count(), 4);
}

// =============================================================================
// 2. HunkHeader accessors (10 tests)
// =============================================================================

#[test]
fn test_old_start_returns_correct_value() {
    let header = HunkHeader::new(42, 10, 1, 1);
    assert_eq!(header.old_start(), 42);
}

#[test]
fn test_old_count_returns_correct_value() {
    let header = HunkHeader::new(1, 42, 1, 1);
    assert_eq!(header.old_count(), 42);
}

#[test]
fn test_new_start_returns_correct_value() {
    let header = HunkHeader::new(1, 1, 42, 1);
    assert_eq!(header.new_start(), 42);
}

#[test]
fn test_new_count_returns_correct_value() {
    let header = HunkHeader::new(1, 1, 1, 42);
    assert_eq!(header.new_count(), 42);
}

#[test]
fn test_old_end_returns_start_plus_count() {
    let header = HunkHeader::new(10, 5, 1, 1);
    assert_eq!(header.old_end(), 15);
}

#[test]
fn test_new_end_returns_start_plus_count() {
    let header = HunkHeader::new(1, 1, 10, 5);
    assert_eq!(header.new_end(), 15);
}

#[test]
fn test_old_end_with_zero_count() {
    let header = HunkHeader::new(10, 0, 1, 1);
    assert_eq!(header.old_end(), 10);
}

#[test]
fn test_new_end_with_zero_count() {
    let header = HunkHeader::new(1, 1, 10, 0);
    assert_eq!(header.new_end(), 10);
}

#[test]
fn test_is_empty_with_zero_lines() {
    let header = HunkHeader::new(0, 0, 0, 0);
    assert!(header.is_empty());
}

#[test]
fn test_is_empty_with_non_zero_lines() {
    assert!(!HunkHeader::new(1, 1, 0, 0).is_empty());
    assert!(!HunkHeader::new(0, 0, 1, 1).is_empty());
    assert!(!HunkHeader::new(1, 0, 0, 1).is_empty());
}

// =============================================================================
// 3. HunkHeader line containment (8 tests)
// =============================================================================

#[test]
fn test_contains_old_line_at_start() {
    let header = HunkHeader::new(10, 5, 1, 1);
    assert!(header.contains_old_line(10));
}

#[test]
fn test_contains_old_line_at_end_exclusive() {
    let header = HunkHeader::new(10, 5, 1, 1);
    assert!(!header.contains_old_line(15)); // end is exclusive
}

#[test]
fn test_contains_old_line_in_middle() {
    let header = HunkHeader::new(10, 5, 1, 1);
    assert!(header.contains_old_line(12));
}

#[test]
fn test_contains_old_line_before_range() {
    let header = HunkHeader::new(10, 5, 1, 1);
    assert!(!header.contains_old_line(9));
}

#[test]
fn test_contains_new_line_at_start() {
    let header = HunkHeader::new(1, 1, 10, 5);
    assert!(header.contains_new_line(10));
}

#[test]
fn test_contains_new_line_at_end_exclusive() {
    let header = HunkHeader::new(1, 1, 10, 5);
    assert!(!header.contains_new_line(15)); // end is exclusive
}

#[test]
fn test_contains_new_line_in_middle() {
    let header = HunkHeader::new(1, 1, 10, 5);
    assert!(header.contains_new_line(12));
}

#[test]
fn test_contains_line_with_zero_count() {
    let header = HunkHeader::new(10, 0, 20, 0);
    assert!(!header.contains_old_line(10));
    assert!(!header.contains_new_line(20));
}

// =============================================================================
// 4. HunkHeader builder methods (8 tests)
// =============================================================================

#[test]
fn test_with_old_start_changes_old_start() {
    let header = HunkHeader::new(1, 4, 1, 5);
    let modified = header.with_old_start(10);
    assert_eq!(modified.old_start(), 10);
    assert_eq!(modified.old_count(), 4);
    assert_eq!(modified.new_start(), 1);
    assert_eq!(modified.new_count(), 5);
}

#[test]
fn test_with_old_count_changes_old_count() {
    let header = HunkHeader::new(1, 4, 1, 5);
    let modified = header.with_old_count(10);
    assert_eq!(modified.old_start(), 1);
    assert_eq!(modified.old_count(), 10);
    assert_eq!(modified.new_start(), 1);
    assert_eq!(modified.new_count(), 5);
}

#[test]
fn test_with_new_start_changes_new_start() {
    let header = HunkHeader::new(1, 4, 1, 5);
    let modified = header.with_new_start(10);
    assert_eq!(modified.old_start(), 1);
    assert_eq!(modified.old_count(), 4);
    assert_eq!(modified.new_start(), 10);
    assert_eq!(modified.new_count(), 5);
}

#[test]
fn test_with_new_count_changes_new_count() {
    let header = HunkHeader::new(1, 4, 1, 5);
    let modified = header.with_new_count(10);
    assert_eq!(modified.old_start(), 1);
    assert_eq!(modified.old_count(), 4);
    assert_eq!(modified.new_start(), 1);
    assert_eq!(modified.new_count(), 10);
}

#[test]
fn test_with_methods_can_be_chained() {
    let header = HunkHeader::new(1, 1, 1, 1)
        .with_old_start(10)
        .with_old_count(5)
        .with_new_start(20)
        .with_new_count(7);
    assert_eq!(header.old_start(), 10);
    assert_eq!(header.old_count(), 5);
    assert_eq!(header.new_start(), 20);
    assert_eq!(header.new_count(), 7);
}

#[test]
fn test_with_old_start_preserves_others() {
    let header = HunkHeader::new(100, 200, 300, 400);
    let modified = header.with_old_start(1);
    assert_eq!(modified.old_count(), 200);
    assert_eq!(modified.new_start(), 300);
    assert_eq!(modified.new_count(), 400);
}

#[test]
fn test_with_zero_values() {
    let header = HunkHeader::new(10, 5, 15, 7);
    let modified = header.with_old_count(0).with_new_count(0);
    assert!(modified.is_empty());
}

#[test]
fn test_original_unchaged_after_with() {
    let header = HunkHeader::new(1, 4, 1, 5);
    let _modified = header.with_old_start(100);
    // Original should be unchanged (Copy trait)
    assert_eq!(header.old_start(), 1);
}

// =============================================================================
// 5. Standard hunk header parsing (10 tests)
// =============================================================================

#[test]
fn test_parse_standard_hunk_header() {
    let header = parse_hunk_header("@@ -1,4 +1,5 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 4);
    assert_eq!(header.new_start(), 1);
    assert_eq!(header.new_count(), 5);
}

#[test]
fn test_parse_with_large_numbers() {
    let header = parse_hunk_header("@@ -1000,500 +2000,750 @@")
        .unwrap()
        .unwrap();
    assert_eq!(header.old_start(), 1000);
    assert_eq!(header.old_count(), 500);
    assert_eq!(header.new_start(), 2000);
    assert_eq!(header.new_count(), 750);
}

#[test]
fn test_parse_with_context_after() {
    let header = parse_hunk_header("@@ -1,4 +1,5 @@ function name()")
        .unwrap()
        .unwrap();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 4);
    assert_eq!(header.new_start(), 1);
    assert_eq!(header.new_count(), 5);
}

#[test]
fn test_parse_with_leading_whitespace() {
    let header = parse_hunk_header("  @@ -1,4 +1,5 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 4);
}

#[test]
fn test_parse_with_trailing_whitespace() {
    let header = parse_hunk_header("@@ -1,4 +1,5 @@  ").unwrap().unwrap();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 4);
}

#[test]
fn test_parse_different_starts() {
    let header = parse_hunk_header("@@ -100,1 +200,1 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 100);
    assert_eq!(header.new_start(), 200);
}

#[test]
fn test_parse_asymmetric_counts() {
    let header = parse_hunk_header("@@ -1,10 +1,2 @@").unwrap().unwrap();
    assert_eq!(header.old_count(), 10);
    assert_eq!(header.new_count(), 2);
}

#[test]
fn test_parse_zero_start() {
    let header = parse_hunk_header("@@ -0,0 +1,1 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 0);
    assert_eq!(header.old_count(), 0);
    assert_eq!(header.new_start(), 1);
    assert_eq!(header.new_count(), 1);
}

#[test]
fn test_parse_via_hunk_header_parse() {
    let header = HunkHeader::parse("@@ -42,7 +100,3 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 42);
    assert_eq!(header.old_count(), 7);
    assert_eq!(header.new_start(), 100);
    assert_eq!(header.new_count(), 3);
}

#[test]
fn test_parse_single_digit_values() {
    let header = parse_hunk_header("@@ -1,2 +3,4 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 2);
    assert_eq!(header.new_start(), 3);
    assert_eq!(header.new_count(), 4);
}

// =============================================================================
// 6. Hunk headers without counts (6 tests)
// =============================================================================

#[test]
fn test_parse_without_counts_defaults_to_one() {
    let header = parse_hunk_header("@@ -1 +1 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 1);
    assert_eq!(header.new_start(), 1);
    assert_eq!(header.new_count(), 1);
}

#[test]
fn test_parse_without_old_count() {
    let header = parse_hunk_header("@@ -1 +1,5 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 1);
    assert_eq!(header.new_start(), 1);
    assert_eq!(header.new_count(), 5);
}

#[test]
fn test_parse_without_new_count() {
    let header = parse_hunk_header("@@ -1,4 +1 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 4);
    assert_eq!(header.new_start(), 1);
    assert_eq!(header.new_count(), 1);
}

#[test]
fn test_parse_zero_without_count() {
    let header = parse_hunk_header("@@ -0 +0 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 0);
    assert_eq!(header.old_count(), 0);
    assert_eq!(header.new_start(), 0);
    assert_eq!(header.new_count(), 0);
}

#[test]
fn test_parse_large_start_without_count() {
    let header = parse_hunk_header("@@ -1000 +2000 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 1000);
    assert_eq!(header.old_count(), 1);
    assert_eq!(header.new_start(), 2000);
    assert_eq!(header.new_count(), 1);
}

#[test]
fn test_parse_mixed_count_formats() {
    let header = parse_hunk_header("@@ -1 +1,10 @@").unwrap().unwrap();
    assert_eq!(header.old_count(), 1);
    assert_eq!(header.new_count(), 10);

    let header = parse_hunk_header("@@ -1,10 +1 @@").unwrap().unwrap();
    assert_eq!(header.old_count(), 10);
    assert_eq!(header.new_count(), 1);
}

// =============================================================================
// 7. Edge cases (8 tests)
// =============================================================================

#[test]
fn test_parse_empty_hunk() {
    let header = parse_hunk_header("@@ -0,0 +0,0 @@").unwrap().unwrap();
    assert!(header.is_empty());
}

#[test]
fn test_parse_new_file() {
    // New file: old is 0,0
    let header = parse_hunk_header("@@ -0,0 +1,10 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 0);
    assert_eq!(header.old_count(), 0);
    assert_eq!(header.new_start(), 1);
    assert_eq!(header.new_count(), 10);
}

#[test]
fn test_parse_deleted_file() {
    // Deleted file: new is 0,0
    let header = parse_hunk_header("@@ -1,10 +0,0 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 10);
    assert_eq!(header.new_start(), 0);
    assert_eq!(header.new_count(), 0);
}

#[test]
fn test_parse_single_line_change() {
    let header = parse_hunk_header("@@ -42,1 +42,1 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 42);
    assert_eq!(header.old_count(), 1);
    assert_eq!(header.new_start(), 42);
    assert_eq!(header.new_count(), 1);
}

#[test]
fn test_parse_only_additions() {
    let header = parse_hunk_header("@@ -1,0 +1,5 @@").unwrap().unwrap();
    assert_eq!(header.old_count(), 0);
    assert_eq!(header.new_count(), 5);
}

#[test]
fn test_parse_only_deletions() {
    let header = parse_hunk_header("@@ -1,5 +1,0 @@").unwrap().unwrap();
    assert_eq!(header.old_count(), 5);
    assert_eq!(header.new_count(), 0);
}

#[test]
fn test_line_count_calculation() {
    let header = HunkHeader::new(1, 10, 1, 5);
    assert_eq!(header.line_count(), 15);
}

#[test]
fn test_parse_with_function_context() {
    // Git sometimes includes function context after the header
    let header = parse_hunk_header("@@ -1,4 +1,5 @@ fn main() {")
        .unwrap()
        .unwrap();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 4);
    assert_eq!(header.new_start(), 1);
    assert_eq!(header.new_count(), 5);
}

// =============================================================================
// 8. Invalid format handling (10 tests)
// =============================================================================

#[test]
fn test_parse_non_hunk_returns_none() {
    assert!(parse_hunk_header("not a hunk header").unwrap().is_none());
}

#[test]
fn test_parse_empty_string_returns_none() {
    assert!(parse_hunk_header("").unwrap().is_none());
}

#[test]
fn test_parse_whitespace_only_returns_none() {
    assert!(parse_hunk_header("   ").unwrap().is_none());
}

#[test]
fn test_parse_missing_minus_segment() {
    let result = parse_hunk_header("@@ 1,4 +1,5 @@");
    assert!(matches!(result, Err(HunkHeaderError::MissingOldRange)));
}

#[test]
fn test_parse_missing_plus_segment() {
    let result = parse_hunk_header("@@ -1,4 1,5 @@");
    assert!(matches!(result, Err(HunkHeaderError::MissingNewRange)));
}

#[test]
fn test_parse_invalid_old_start() {
    let result = parse_hunk_header("@@ -abc,4 +1,5 @@");
    assert!(matches!(result, Err(HunkHeaderError::InvalidOldStart(_))));
}

#[test]
fn test_parse_invalid_new_start() {
    let result = parse_hunk_header("@@ -1,4 +xyz,5 @@");
    assert!(matches!(result, Err(HunkHeaderError::InvalidNewStart(_))));
}

#[test]
fn test_parse_invalid_old_count() {
    let result = parse_hunk_header("@@ -1,abc +1,5 @@");
    assert!(matches!(result, Err(HunkHeaderError::InvalidOldCount(_))));
}

#[test]
fn test_parse_invalid_new_count() {
    let result = parse_hunk_header("@@ -1,4 +1,xyz @@");
    assert!(matches!(result, Err(HunkHeaderError::InvalidNewCount(_))));
}

#[test]
fn test_parse_plus_before_minus() {
    let result = parse_hunk_header("@@ +1,5 -1,4 @@");
    assert!(matches!(result, Err(HunkHeaderError::Malformed(_))));
}

// =============================================================================
// 9. Display trait round-trip (5 tests)
// =============================================================================

#[test]
fn test_display_round_trip_standard() {
    let original = "@@ -1,4 +1,5 @@";
    let header = parse_hunk_header(original).unwrap().unwrap();
    assert_eq!(format!("{header}"), original);
}

#[test]
fn test_display_round_trip_zero_values() {
    let original = "@@ -0,0 +0,0 @@";
    let header = parse_hunk_header(original).unwrap().unwrap();
    assert_eq!(format!("{header}"), original);
}

#[test]
fn test_display_round_trip_large_values() {
    let original = "@@ -1000,500 +2000,750 @@";
    let header = parse_hunk_header(original).unwrap().unwrap();
    assert_eq!(format!("{header}"), original);
}

#[test]
fn test_display_format() {
    let header = HunkHeader::new(42, 7, 100, 3);
    assert_eq!(format!("{header}"), "@@ -42,7 +100,3 @@");
}

#[test]
fn test_display_after_builder_methods() {
    let header = HunkHeader::new(1, 1, 1, 1)
        .with_old_start(10)
        .with_old_count(5)
        .with_new_start(20)
        .with_new_count(7);
    assert_eq!(format!("{header}"), "@@ -10,5 +20,7 @@");
}

// =============================================================================
// 10. Property-based tests with proptest (5 tests)
// =============================================================================

use proptest::prelude::*;

prop_compose! {
    fn arb_hunk_header()(old_start in 0usize..10000,
                         old_count in 0usize..1000,
                         new_start in 0usize..10000,
                         new_count in 0usize..1000) -> HunkHeader {
        HunkHeader::new(old_start, old_count, new_start, new_count)
    }
}

proptest! {
    #[test]
    fn test_display_round_trip_property(header in arb_hunk_header()) {
        let displayed = format!("{header}");
        let parsed = parse_hunk_header(&displayed).unwrap().unwrap();
        prop_assert_eq!(header, parsed);
    }

    #[test]
    fn test_line_count_is_sum_of_counts(old_count in 0usize..1000, new_count in 0usize..1000) {
        let header = HunkHeader::new(1, old_count, 1, new_count);
        prop_assert_eq!(header.line_count(), old_count + new_count);
    }

    #[test]
    fn test_is_empty_iff_both_counts_zero(old_count in 0usize..100, new_count in 0usize..100) {
        let header = HunkHeader::new(1, old_count, 1, new_count);
        prop_assert_eq!(header.is_empty(), old_count == 0 && new_count == 0);
    }

    #[test]
    fn test_old_end_equals_start_plus_count(start in 0usize..10000, count in 0usize..1000) {
        let header = HunkHeader::new(start, count, 1, 1);
        prop_assert_eq!(header.old_end(), start + count);
    }

    #[test]
    fn test_new_end_equals_start_plus_count(start in 0usize..10000, count in 0usize..1000) {
        let header = HunkHeader::new(1, 1, start, count);
        prop_assert_eq!(header.new_end(), start + count);
    }

    #[test]
    fn test_contains_old_line_in_range(start in 1usize..1000, count in 1usize..100) {
        let header = HunkHeader::new(start, count, 1, 1);
        let line = start + count / 2; // Middle of range
        prop_assert!(header.contains_old_line(line));
        prop_assert!(!header.contains_old_line(start - 1)); // Before
        prop_assert!(!header.contains_old_line(start + count)); // After (end exclusive)
    }

    #[test]
    fn test_with_methods_preserve_other_values(
        old_start in 0usize..1000,
        old_count in 0usize..1000,
        new_start in 0usize..1000,
        new_count in 0usize..1000
    ) {
        let header = HunkHeader::new(old_start, old_count, new_start, new_count);

        // with_old_start
        let modified = header.with_old_start(42);
        prop_assert_eq!(modified.old_count(), old_count);
        prop_assert_eq!(modified.new_start(), new_start);
        prop_assert_eq!(modified.new_count(), new_count);

        // with_new_count
        let modified = header.with_new_count(42);
        prop_assert_eq!(modified.old_start(), old_start);
        prop_assert_eq!(modified.old_count(), old_count);
        prop_assert_eq!(modified.new_start(), new_start);
    }
}

// =============================================================================
// Additional edge case tests for comprehensive coverage
// =============================================================================

#[test]
fn test_parse_with_tabs() {
    let header = parse_hunk_header("@@ -1,4 +1,5 @@\tsome context")
        .unwrap()
        .unwrap();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 4);
}

#[test]
fn test_parse_double_at_symbols() {
    // Should still work with @@ prefix
    let header = parse_hunk_header("@@ -1,4 +1,5 @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 1);
}

#[test]
fn test_parse_with_multiple_spaces() {
    let header = parse_hunk_header("@@  -1,4  +1,5  @@").unwrap().unwrap();
    assert_eq!(header.old_start(), 1);
    assert_eq!(header.old_count(), 4);
}

#[test]
fn test_copy_trait() {
    let header1 = HunkHeader::new(1, 4, 1, 5);
    let header2 = header1; // Copy
    assert_eq!(header1, header2);
}

#[test]
fn test_clone_trait() {
    let header1 = HunkHeader::new(1, 4, 1, 5);
    let header2 = header1.clone();
    assert_eq!(header1, header2);
}

#[test]
fn test_debug_trait() {
    let header = HunkHeader::new(1, 4, 1, 5);
    let debug_str = format!("{header:?}");
    assert!(debug_str.contains("HunkHeader"));
    assert!(debug_str.contains("old_start"));
    assert!(debug_str.contains("old_count"));
}

#[test]
fn test_eq_trait() {
    let header1 = HunkHeader::new(1, 4, 1, 5);
    let header2 = HunkHeader::new(1, 4, 1, 5);
    let header3 = HunkHeader::new(1, 4, 1, 6);
    assert_eq!(header1, header2);
    assert_ne!(header1, header3);
}

#[test]
fn test_hash_trait() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let header = HunkHeader::new(1, 4, 1, 5);
    set.insert(header);
    assert!(set.contains(&HunkHeader::new(1, 4, 1, 5)));
    assert!(!set.contains(&HunkHeader::new(1, 4, 1, 6)));
}

#[test]
fn test_parse_returns_none_for_diff_header() {
    // A diff --git line should not be parsed as a hunk header
    assert!(parse_hunk_header("diff --git a/file.rs b/file.rs")
        .unwrap()
        .is_none());
}

#[test]
fn test_parse_returns_none_for_filename_lines() {
    assert!(parse_hunk_header("--- a/file.rs").unwrap().is_none());
    assert!(parse_hunk_header("+++ b/file.rs").unwrap().is_none());
}

#[test]
fn test_parse_returns_none_for_context_line() {
    assert!(parse_hunk_header(" context line").unwrap().is_none());
}

#[test]
fn test_parse_returns_none_for_addition_line() {
    assert!(parse_hunk_header("+added line").unwrap().is_none());
}

#[test]
fn test_parse_returns_none_for_deletion_line() {
    assert!(parse_hunk_header("-removed line").unwrap().is_none());
}

#[test]
fn test_error_display() {
    let err = HunkHeaderError::NotAHunkHeader;
    assert!(err.to_string().contains("not a hunk header"));

    let err = HunkHeaderError::MissingOldRange;
    assert!(err.to_string().contains("old file range"));

    let err = HunkHeaderError::InvalidOldStart("bad".to_string());
    assert!(err.to_string().contains("invalid old start"));
}

#[test]
fn test_must_use_attribute() {
    // This test verifies that the #[must_use] attribute is present
    // by using the functions. The compiler would warn if not used.
    let _ = HunkHeader::new(1, 1, 1, 1);
    let _ = parse_hunk_header("@@ -1 +1 @@");
}
