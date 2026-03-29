//! Comprehensive BDD tests for lintdiff-range-merge crate.
//!
//! Test coverage:
//! 1. LineRange creation and methods (12 tests)
//! 2. merge_lines function (12 tests)
//! 3. merge_overlapping function (10 tests)
//! 4. Range operations - contains, intersect (8 tests)
//! 5. Range operations - union, adjacent (8 tests)
//! 6. Additional utility functions (8 tests)
//! 7. Edge cases and boundary conditions (8 tests)
//! 8. Property-based tests with proptest (10 tests)

use lintdiff_range_merge::{
    compare_by_end, compare_by_start, expand_to_include, gap_between, is_adjacent, merge_lines,
    merge_overlapping, overlaps_or_adjacent, range_contains, range_contains_range,
    range_intersection, range_len, range_union, ranges_intersect, LineRange,
};
use std::cmp::Ordering;

// =============================================================================
// 1. LineRange creation and methods (12 tests)
// =============================================================================

#[test]
fn line_range_new_creates_range_with_given_values() {
    let range = LineRange::new(1, 10);
    assert_eq!(range.start, 1);
    assert_eq!(range.end, 10);
}

#[test]
fn line_range_new_works_with_large_values() {
    let range = LineRange::new(1, usize::MAX);
    assert_eq!(range.start, 1);
    assert_eq!(range.end, usize::MAX);
}

#[test]
fn line_range_single_creates_range_with_same_start_and_end() {
    let range = LineRange::single(42);
    assert_eq!(range.start, 42);
    assert_eq!(range.end, 42);
}

#[test]
fn line_range_single_works_with_zero() {
    let range = LineRange::single(0);
    assert_eq!(range.start, 0);
    assert_eq!(range.end, 0);
}

#[test]
fn line_range_len_returns_number_of_lines() {
    let range = LineRange::new(1, 10);
    assert_eq!(range.len(), 10);
}

#[test]
fn line_range_len_returns_one_for_single_line() {
    let range = LineRange::single(5);
    assert_eq!(range.len(), 1);
}

#[test]
fn line_range_contains_returns_true_for_lines_in_range() {
    let range = LineRange::new(5, 10);
    assert!(range.contains(5));
    assert!(range.contains(7));
    assert!(range.contains(10));
}

#[test]
fn line_range_contains_returns_false_for_lines_outside_range() {
    let range = LineRange::new(5, 10);
    assert!(!range.contains(4));
    assert!(!range.contains(11));
}

#[test]
fn line_range_intersects_detects_overlapping_ranges() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(5, 15);
    assert!(a.intersects(&b));
}

#[test]
fn line_range_intersects_returns_false_for_non_overlapping() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(11, 20);
    assert!(!a.intersects(&b));
}

#[test]
fn line_range_is_adjacent_to_detects_adjacent_ranges() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(11, 20);
    assert!(a.is_adjacent_to(&b));
    assert!(b.is_adjacent_to(&a));
}

#[test]
fn line_range_display_formats_correctly() {
    let single = LineRange::single(5);
    assert_eq!(format!("{}", single), "5");

    let range = LineRange::new(1, 10);
    assert_eq!(format!("{}", range), "1-10");
}

// =============================================================================
// 2. merge_lines function (12 tests)
// =============================================================================

#[test]
fn merge_lines_returns_empty_for_empty_input() {
    let result = merge_lines(&[]);
    assert!(result.is_empty());
}

#[test]
fn merge_lines_single_line_returns_single_range() {
    let result = merge_lines(&[5]);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(5, 5));
}

#[test]
fn merge_lines_consecutive_lines_merge_into_one_range() {
    let result = merge_lines(&[1, 2, 3, 4, 5]);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(1, 5));
}

#[test]
fn merge_lines_gap_creates_separate_range() {
    let result = merge_lines(&[1, 2, 3, 5]);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], LineRange::new(1, 3));
    assert_eq!(result[1], LineRange::new(5, 5));
}

#[test]
fn merge_lines_example_from_spec() {
    // [1, 2, 3, 5, 7] → [(1,3), (5,5), (7,7)]
    let result = merge_lines(&[1, 2, 3, 5, 7]);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], LineRange::new(1, 3));
    assert_eq!(result[1], LineRange::new(5, 5));
    assert_eq!(result[2], LineRange::new(7, 7));
}

#[test]
fn merge_lines_multiple_consecutive_groups() {
    let result = merge_lines(&[1, 2, 3, 5, 6, 7, 9, 10, 11, 12]);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], LineRange::new(1, 3));
    assert_eq!(result[1], LineRange::new(5, 7));
    assert_eq!(result[2], LineRange::new(9, 12));
}

#[test]
fn merge_lines_all_isolated_lines() {
    let result = merge_lines(&[1, 3, 5, 7, 9]);
    assert_eq!(result.len(), 5);
    for (i, range) in result.iter().enumerate() {
        assert_eq!(range.start, 1 + i * 2);
        assert_eq!(range.end, range.start);
    }
}

#[test]
fn merge_lines_two_lines_with_gap() {
    let result = merge_lines(&[1, 100]);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], LineRange::new(1, 1));
    assert_eq!(result[1], LineRange::new(100, 100));
}

#[test]
fn merge_lines_two_adjacent_lines() {
    let result = merge_lines(&[1, 2]);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(1, 2));
}

#[test]
fn merge_lines_with_zero_line_numbers() {
    let result = merge_lines(&[0, 1, 2]);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(0, 2));
}

#[test]
fn merge_lines_with_large_numbers() {
    let result = merge_lines(&[usize::MAX - 2, usize::MAX - 1, usize::MAX]);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(usize::MAX - 2, usize::MAX));
}

#[test]
fn merge_lines_large_range() {
    let lines: Vec<usize> = (1..=1000).collect();
    let result = merge_lines(&lines);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(1, 1000));
}

// =============================================================================
// 3. merge_overlapping function (10 tests)
// =============================================================================

#[test]
fn merge_overlapping_returns_empty_for_empty_input() {
    let result = merge_overlapping(&[]);
    assert!(result.is_empty());
}

#[test]
fn merge_overlapping_single_range_returns_unchanged() {
    let ranges = vec![LineRange::new(1, 10)];
    let result = merge_overlapping(&ranges);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(1, 10));
}

#[test]
fn merge_overlapping_example_from_spec() {
    // [(1,5), (3,7)] → [(1,7)]
    let ranges = vec![LineRange::new(1, 5), LineRange::new(3, 7)];
    let result = merge_overlapping(&ranges);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(1, 7));
}

#[test]
fn merge_overlapping_adjacent_ranges_merge() {
    let ranges = vec![LineRange::new(1, 10), LineRange::new(11, 20)];
    let result = merge_overlapping(&ranges);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(1, 20));
}

#[test]
fn merge_overlapping_non_overlapping_ranges_stay_separate() {
    let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
    let result = merge_overlapping(&ranges);
    assert_eq!(result.len(), 2);
}

#[test]
fn merge_overlapping_fully_contained_range() {
    let ranges = vec![LineRange::new(1, 20), LineRange::new(5, 10)];
    let result = merge_overlapping(&ranges);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(1, 20));
}

#[test]
fn merge_overlapping_chain_of_overlapping() {
    let ranges = vec![
        LineRange::new(1, 5),
        LineRange::new(4, 8),
        LineRange::new(7, 12),
        LineRange::new(11, 15),
    ];
    let result = merge_overlapping(&ranges);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(1, 15));
}

#[test]
fn merge_overlapping_unsorted_input_works() {
    let ranges = vec![
        LineRange::new(20, 30),
        LineRange::new(1, 10),
        LineRange::new(5, 15),
    ];
    let result = merge_overlapping(&ranges);
    assert_eq!(result.len(), 2);
    // Should be sorted by start
    assert_eq!(result[0], LineRange::new(1, 15));
    assert_eq!(result[1], LineRange::new(20, 30));
}

#[test]
fn merge_overlapping_multiple_groups() {
    let ranges = vec![
        LineRange::new(1, 5),
        LineRange::new(3, 7),
        LineRange::new(20, 25),
        LineRange::new(22, 30),
    ];
    let result = merge_overlapping(&ranges);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], LineRange::new(1, 7));
    assert_eq!(result[1], LineRange::new(20, 30));
}

#[test]
fn merge_overlapping_same_range_multiple_times() {
    let ranges = vec![
        LineRange::new(1, 10),
        LineRange::new(1, 10),
        LineRange::new(1, 10),
    ];
    let result = merge_overlapping(&ranges);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(1, 10));
}

// =============================================================================
// 4. Range operations - contains, intersect (8 tests)
// =============================================================================

#[test]
fn range_contains_returns_true_for_line_in_range() {
    let range = LineRange::new(5, 10);
    assert!(range_contains(&range, 5));
    assert!(range_contains(&range, 7));
    assert!(range_contains(&range, 10));
}

#[test]
fn range_contains_returns_false_for_line_outside_range() {
    let range = LineRange::new(5, 10);
    assert!(!range_contains(&range, 4));
    assert!(!range_contains(&range, 11));
}

#[test]
fn range_contains_works_for_single_line_range() {
    let range = LineRange::single(5);
    assert!(range_contains(&range, 5));
    assert!(!range_contains(&range, 4));
    assert!(!range_contains(&range, 6));
}

#[test]
fn ranges_intersect_returns_true_for_overlapping() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(5, 15);
    assert!(ranges_intersect(&a, &b));
    assert!(ranges_intersect(&b, &a)); // Symmetric
}

#[test]
fn ranges_intersect_returns_true_for_touching_at_endpoint() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(10, 20);
    assert!(ranges_intersect(&a, &b));
}

#[test]
fn ranges_intersect_returns_false_for_adjacent() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(11, 20);
    assert!(!ranges_intersect(&a, &b));
}

#[test]
fn ranges_intersect_returns_false_for_separate() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(20, 30);
    assert!(!ranges_intersect(&a, &b));
}

#[test]
fn range_len_returns_correct_count() {
    assert_eq!(range_len(&LineRange::new(1, 10)), 10);
    assert_eq!(range_len(&LineRange::new(5, 5)), 1);
    assert_eq!(range_len(&LineRange::new(100, 200)), 101);
}

// =============================================================================
// 5. Range operations - union, adjacent (8 tests)
// =============================================================================

#[test]
fn range_union_overlapping_returns_merged() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(5, 15);
    let union = range_union(&a, &b).unwrap();
    assert_eq!(union, LineRange::new(1, 15));
}

#[test]
fn range_union_adjacent_returns_merged() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(11, 20);
    let union = range_union(&a, &b).unwrap();
    assert_eq!(union, LineRange::new(1, 20));
}

#[test]
fn range_union_non_adjacent_returns_none() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(20, 30);
    assert!(range_union(&a, &b).is_none());
}

#[test]
fn range_union_is_symmetric() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(5, 15);
    let union_ab = range_union(&a, &b);
    let union_ba = range_union(&b, &a);
    assert_eq!(union_ab, union_ba);
}

#[test]
fn is_adjacent_returns_true_for_adjacent_ranges() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(11, 20);
    assert!(is_adjacent(&a, &b));
    assert!(is_adjacent(&b, &a)); // Symmetric
}

#[test]
fn is_adjacent_returns_false_for_overlapping() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(10, 20);
    assert!(!is_adjacent(&a, &b));
}

#[test]
fn is_adjacent_returns_false_for_separate() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(20, 30);
    assert!(!is_adjacent(&a, &b));
}

#[test]
fn overlaps_or_adjacent_catches_both_cases() {
    let a = LineRange::new(1, 10);
    let overlapping = LineRange::new(5, 15);
    let adjacent = LineRange::new(11, 20);
    let separate = LineRange::new(20, 30);

    assert!(overlaps_or_adjacent(&a, &overlapping));
    assert!(overlaps_or_adjacent(&a, &adjacent));
    assert!(!overlaps_or_adjacent(&a, &separate));
}

// =============================================================================
// 6. Additional utility functions (8 tests)
// =============================================================================

#[test]
fn compare_by_start_orders_by_start_position() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(5, 15);
    assert_eq!(compare_by_start(&a, &b), Ordering::Less);
    assert_eq!(compare_by_start(&b, &a), Ordering::Greater);
    assert_eq!(compare_by_start(&a, &a), Ordering::Equal);
}

#[test]
fn compare_by_end_orders_by_end_position() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(5, 8);
    assert_eq!(compare_by_end(&a, &b), Ordering::Greater);
    assert_eq!(compare_by_end(&b, &a), Ordering::Less);
    assert_eq!(compare_by_end(&a, &a), Ordering::Equal);
}

#[test]
fn range_intersection_overlapping_returns_intersection() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(5, 15);
    let intersection = range_intersection(&a, &b).unwrap();
    assert_eq!(intersection, LineRange::new(5, 10));
}

#[test]
fn range_intersection_non_overlapping_returns_none() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(20, 30);
    assert!(range_intersection(&a, &b).is_none());
}

#[test]
fn expand_to_include_expands_start() {
    let range = LineRange::new(5, 10);
    let expanded = expand_to_include(&range, 2);
    assert_eq!(expanded, LineRange::new(2, 10));
}

#[test]
fn expand_to_include_expands_end() {
    let range = LineRange::new(5, 10);
    let expanded = expand_to_include(&range, 15);
    assert_eq!(expanded, LineRange::new(5, 15));
}

#[test]
fn range_contains_range_returns_true_for_contained() {
    let outer = LineRange::new(1, 20);
    let inner = LineRange::new(5, 10);
    assert!(range_contains_range(&outer, &inner));
}

#[test]
fn gap_between_returns_correct_gap() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(15, 20);
    assert_eq!(gap_between(&a, &b), Some(4));
    assert_eq!(gap_between(&b, &a), Some(4)); // Symmetric
}

// =============================================================================
// 7. Edge cases and boundary conditions (8 tests)
// =============================================================================

#[test]
fn edge_case_zero_line_numbers() {
    let range = LineRange::new(0, 0);
    assert_eq!(range.len(), 1);
    assert!(range.contains(0));
}

#[test]
fn edge_case_very_large_range() {
    let range = LineRange::new(1, usize::MAX);
    assert_eq!(range.len(), usize::MAX);
}

#[test]
fn edge_case_merge_lines_with_duplicates() {
    // Duplicates should still work (though input should ideally be deduplicated)
    let result = merge_lines(&[1, 1, 2, 2, 3, 3]);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], LineRange::new(1, 3));
}

#[test]
fn edge_case_single_element_operations() {
    let range = LineRange::single(5);
    assert!(range.contains(5));
    assert_eq!(range.len(), 1);
    assert!(!range.is_empty());
}

#[test]
fn edge_case_range_at_boundary() {
    let range = LineRange::new(usize::MAX - 1, usize::MAX);
    assert_eq!(range.len(), 2);
    assert!(range.contains(usize::MAX - 1));
    assert!(range.contains(usize::MAX));
}

#[test]
fn edge_case_adjacent_at_boundary() {
    let a = LineRange::new(usize::MAX - 10, usize::MAX - 1);
    let b = LineRange::new(usize::MAX, usize::MAX);
    assert!(is_adjacent(&a, &b));
}

#[test]
fn edge_case_from_tuple() {
    let range: LineRange = (5, 10).into();
    assert_eq!(range.start, 5);
    assert_eq!(range.end, 10);
}

#[test]
fn edge_case_from_usize() {
    let range: LineRange = 5.into();
    assert_eq!(range.start, 5);
    assert_eq!(range.end, 5);
}

// =============================================================================
// 8. Property-based tests with proptest (10 tests)
// =============================================================================

use proptest::prelude::*;

/// Generate arbitrary line ranges
fn arb_range() -> impl Strategy<Value = LineRange> {
    (any::<usize>(), any::<usize>()).prop_map(|(a, b)| {
        let start = a.min(b);
        let end = a.max(b);
        LineRange::new(start, end)
    })
}

proptest! {
    #[test]
    fn prop_range_len_is_positive(range in arb_range()) {
        prop_assert!(range.len() >= 1);
    }

    #[test]
    fn prop_range_contains_start_and_end(range in arb_range()) {
        prop_assert!(range.contains(range.start));
        prop_assert!(range.contains(range.end));
    }

    #[test]
    fn prop_range_union_symmetric(a in arb_range(), b in arb_range()) {
        prop_assert_eq!(range_union(&a, &b), range_union(&b, &a));
    }

    #[test]
    fn prop_range_intersection_symmetric(a in arb_range(), b in arb_range()) {
        prop_assert_eq!(
            range_intersection(&a, &b),
            range_intersection(&b, &a)
        );
    }

    #[test]
    fn prop_is_adjacent_symmetric(a in arb_range(), b in arb_range()) {
        prop_assert_eq!(is_adjacent(&a, &b), is_adjacent(&b, &a));
    }

    #[test]
    fn prop_ranges_intersect_symmetric(a in arb_range(), b in arb_range()) {
        prop_assert_eq!(ranges_intersect(&a, &b), ranges_intersect(&b, &a));
    }

    #[test]
    fn prop_merge_lines_produces_non_overlapping(lines in prop::collection::vec(any::<usize>(), 0..100)) {
        let mut sorted = lines;
        sorted.sort();
        sorted.dedup();

        let ranges = merge_lines(&sorted);

        // Check that ranges don't overlap
        for i in 0..ranges.len().saturating_sub(1) {
            prop_assert!(!ranges[i].overlaps_or_adjacent(&ranges[i + 1]));
        }
    }

    #[test]
    fn prop_merge_overlapping_produces_non_overlapping(
        ranges in prop::collection::vec(arb_range(), 0..50)
    ) {
        let merged = merge_overlapping(&ranges);

        // Check that merged ranges don't overlap
        for i in 0..merged.len().saturating_sub(1) {
            for j in (i + 1)..merged.len() {
                prop_assert!(!merged[i].intersects(&merged[j]));
                prop_assert!(!merged[i].is_adjacent_to(&merged[j]));
            }
        }
    }

    #[test]
    fn prop_range_union_contains_both(a in arb_range(), b in arb_range()) {
        if let Some(union) = range_union(&a, &b) {
            prop_assert!(range_contains_range(&union, &a));
            prop_assert!(range_contains_range(&union, &b));
        }
    }

    #[test]
    fn prop_range_intersection_contained_in_both(a in arb_range(), b in arb_range()) {
        if let Some(intersection) = range_intersection(&a, &b) {
            prop_assert!(range_contains_range(&a, &intersection));
            prop_assert!(range_contains_range(&b, &intersection));
        }
    }
}

// =============================================================================
// Additional tests to reach 35+ target
// =============================================================================

#[test]
fn line_range_default_is_single_line_one() {
    let range = LineRange::default();
    assert_eq!(range.start, 1);
    assert_eq!(range.end, 1);
}

#[test]
fn line_range_equality_works() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(1, 10);
    let c = LineRange::new(1, 11);

    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn line_range_ordering_by_start_then_end() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(2, 5);
    let c = LineRange::new(2, 10);

    assert!(a < b); // Lower start comes first
    assert!(b < c); // Same start, lower end comes first
}

#[test]
fn merge_lines_preserves_all_input_lines() {
    let lines = vec![1, 2, 3, 5, 7, 8, 9, 12];
    let ranges = merge_lines(&lines);

    let mut reconstructed = Vec::new();
    for range in &ranges {
        for line in range.start..=range.end {
            reconstructed.push(line);
        }
    }

    assert_eq!(lines, reconstructed);
}

#[test]
fn merge_overlapping_preserves_coverage() {
    let ranges = vec![
        LineRange::new(1, 10),
        LineRange::new(5, 15),
        LineRange::new(20, 30),
    ];

    let merged = merge_overlapping(&ranges);

    // Every line in original ranges should be in merged ranges
    for range in &ranges {
        for line in range.start..=range.end {
            let covered = merged.iter().any(|r| r.contains(line));
            assert!(covered, "Line {} should be covered", line);
        }
    }
}

#[test]
fn gap_between_adjacent_is_none() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(11, 20);
    assert_eq!(gap_between(&a, &b), None);
}

#[test]
fn gap_between_overlapping_is_none() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(5, 15);
    assert_eq!(gap_between(&a, &b), None);
}

#[test]
fn expand_to_include_line_already_in_range() {
    let range = LineRange::new(5, 10);
    let expanded = expand_to_include(&range, 7);
    assert_eq!(expanded, range); // Should be unchanged
}

#[test]
fn range_contains_range_same_range() {
    let range = LineRange::new(1, 10);
    assert!(range_contains_range(&range, &range));
}

#[test]
fn range_contains_range_partial_overlap_false() {
    let a = LineRange::new(1, 10);
    let b = LineRange::new(5, 15);
    assert!(!range_contains_range(&a, &b));
    assert!(!range_contains_range(&b, &a));
}

#[test]
fn line_range_new_unchecked_bypasses_assertion() {
    // This should not panic even though end < start
    let range = LineRange::new_unchecked(10, 5);
    assert_eq!(range.start, 10);
    assert_eq!(range.end, 5);
}

#[test]
fn line_range_is_empty_for_invalid_range() {
    let range = LineRange::new_unchecked(10, 5);
    assert!(range.is_empty());
}

#[test]
fn merge_overlapping_with_single_line_ranges() {
    let ranges = vec![
        LineRange::single(1),
        LineRange::single(2),
        LineRange::single(3),
        LineRange::single(5),
    ];
    let result = merge_overlapping(&ranges);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], LineRange::new(1, 3));
    assert_eq!(result[1], LineRange::new(5, 5));
}

#[test]
fn large_scale_merge_test() {
    // Create 1000 individual lines
    let lines: Vec<usize> = (1..=1000).collect();
    let ranges = merge_lines(&lines);
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0], LineRange::new(1, 1000));
}

#[test]
fn alternating_merge_test() {
    // Every other line
    let lines: Vec<usize> = (1..=100).filter(|n| n % 2 == 1).collect();
    let ranges = merge_lines(&lines);
    assert_eq!(ranges.len(), 50); // 50 separate single-line ranges
}
