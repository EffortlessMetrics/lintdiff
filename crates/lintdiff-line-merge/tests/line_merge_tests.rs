//! BDD tests for lintdiff-line-merge crate.
//!
//! These tests follow Behavior-Driven Development principles, testing
//! the observable behavior of the line range merging utilities.

use lintdiff_line_merge::*;
use lintdiff_line_range::LineRange;
use proptest::prelude::*;

// ============================================================================
// Feature: ranges_overlap function
// ============================================================================

mod ranges_overlap_feature {
    use super::*;

    mod when_ranges_overlap {
        use super::*;

        #[test]
        fn it_returns_true_for_overlapping_ranges() {
            // Given: Two ranges that share lines
            let a = (1, 10);
            let b = (5, 15);

            // When: We check if they overlap
            let result = ranges_overlap(a, b);

            // Then: It returns true
            assert!(result);
        }

        #[test]
        fn it_returns_true_for_reversed_overlapping_ranges() {
            let a = (5, 15);
            let b = (1, 10);

            assert!(ranges_overlap(a, b));
        }

        #[test]
        fn it_returns_true_for_nested_ranges() {
            let outer = (1, 20);
            let inner = (5, 10);

            assert!(ranges_overlap(outer, inner));
            assert!(ranges_overlap(inner, outer));
        }

        #[test]
        fn it_returns_true_for_identical_ranges() {
            let range = (5, 10);

            assert!(ranges_overlap(range, range));
        }

        #[test]
        fn it_returns_true_for_edge_touching_ranges() {
            // Ranges that touch at exactly one point
            let a = (1, 10);
            let b = (10, 20);

            assert!(ranges_overlap(a, b));
            assert!(ranges_overlap(b, a));
        }
    }

    mod when_ranges_are_adjacent {
        use super::*;

        #[test]
        fn it_returns_true_for_adjacent_ranges() {
            // Adjacent: a.end + 1 == b.start
            let a = (1, 10);
            let b = (11, 20);

            assert!(ranges_overlap(a, b));
        }

        #[test]
        fn it_returns_true_for_adjacent_ranges_reversed() {
            let a = (11, 20);
            let b = (1, 10);

            assert!(ranges_overlap(a, b));
        }

        #[test]
        fn it_returns_true_for_single_line_adjacent() {
            let a = (1, 1);
            let b = (2, 2);

            assert!(ranges_overlap(a, b));
        }
    }

    mod when_ranges_do_not_overlap {
        use super::*;

        #[test]
        fn it_returns_false_for_non_overlapping_ranges() {
            let a = (1, 10);
            let b = (20, 30);

            assert!(!ranges_overlap(a, b));
            assert!(!ranges_overlap(b, a));
        }

        #[test]
        fn it_returns_false_for_gap_of_one() {
            // Gap of 1 line between ranges
            let a = (1, 10);
            let b = (12, 20);

            assert!(!ranges_overlap(a, b));
        }

        #[test]
        fn it_returns_false_for_large_gap() {
            let a = (1, 10);
            let b = (100, 200);

            assert!(!ranges_overlap(a, b));
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn it_handles_single_point_ranges() {
            assert!(ranges_overlap((1, 1), (1, 1)));
            assert!(ranges_overlap((1, 1), (2, 2)));
            assert!(!ranges_overlap((1, 1), (3, 3)));
        }

        #[test]
        fn it_handles_large_ranges() {
            let a = (1, 1_000_000);
            let b = (999_999, 2_000_000);

            assert!(ranges_overlap(a, b));
        }

        #[test]
        fn it_handles_zero_start() {
            // Line 0 is technically valid for usize
            assert!(ranges_overlap((0, 10), (5, 15)));
            assert!(!ranges_overlap((0, 0), (2, 5)));
        }
    }
}

// ============================================================================
// Feature: ranges_intersect function
// ============================================================================

mod ranges_intersect_feature {
    use super::*;

    mod when_ranges_strictly_intersect {
        use super::*;

        #[test]
        fn it_returns_true_for_overlapping_ranges() {
            assert!(ranges_intersect((1, 10), (5, 15)));
            assert!(ranges_intersect((5, 15), (1, 10)));
        }

        #[test]
        fn it_returns_true_for_nested_ranges() {
            assert!(ranges_intersect((1, 20), (5, 10)));
            assert!(ranges_intersect((5, 10), (1, 20)));
        }

        #[test]
        fn it_returns_true_for_edge_touching() {
            // Edge touching IS intersecting
            assert!(ranges_intersect((1, 10), (10, 20)));
            assert!(ranges_intersect((10, 20), (1, 10)));
        }
    }

    mod when_ranges_are_adjacent {
        use super::*;

        #[test]
        fn it_returns_false_for_adjacent_ranges() {
            // Adjacent (gap of 0, but not touching) is NOT intersecting
            let a = (1, 10);
            let b = (11, 20);

            assert!(!ranges_intersect(a, b));
            assert!(!ranges_intersect(b, a));
        }

        #[test]
        fn it_returns_false_for_adjacent_single_points() {
            assert!(!ranges_intersect((1, 1), (2, 2)));
        }
    }

    mod when_ranges_do_not_intersect {
        use super::*;

        #[test]
        fn it_returns_false_for_separate_ranges() {
            assert!(!ranges_intersect((1, 10), (20, 30)));
            assert!(!ranges_intersect((20, 30), (1, 10)));
        }
    }
}

// ============================================================================
// Feature: union_of_ranges function
// ============================================================================

mod union_of_ranges_feature {
    use super::*;

    mod when_ranges_overlap {
        use super::*;

        #[test]
        fn it_returns_range_from_min_start_to_max_end() {
            let a = (1, 10);
            let b = (5, 15);

            assert_eq!(union_of_ranges(a, b), (1, 15));
            assert_eq!(union_of_ranges(b, a), (1, 15));
        }

        #[test]
        fn it_handles_nested_ranges() {
            let outer = (1, 20);
            let inner = (5, 10);

            assert_eq!(union_of_ranges(outer, inner), (1, 20));
            assert_eq!(union_of_ranges(inner, outer), (1, 20));
        }

        #[test]
        fn it_handles_identical_ranges() {
            let range = (5, 10);
            assert_eq!(union_of_ranges(range, range), (5, 10));
        }
    }

    mod when_ranges_are_adjacent {
        use super::*;

        #[test]
        fn it_merges_adjacent_ranges() {
            let a = (1, 10);
            let b = (11, 20);

            assert_eq!(union_of_ranges(a, b), (1, 20));
            assert_eq!(union_of_ranges(b, a), (1, 20));
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn it_handles_single_point_ranges() {
            assert_eq!(union_of_ranges((1, 1), (1, 1)), (1, 1));
            assert_eq!(union_of_ranges((1, 1), (2, 2)), (1, 2));
        }
    }
}

// ============================================================================
// Feature: intersect_of_ranges function
// ============================================================================

mod intersect_of_ranges_feature {
    use super::*;

    mod when_ranges_intersect {
        use super::*;

        #[test]
        fn it_returns_the_intersection() {
            let a = (1, 10);
            let b = (5, 15);

            assert_eq!(intersect_of_ranges(a, b), Some((5, 10)));
            assert_eq!(intersect_of_ranges(b, a), Some((5, 10)));
        }

        #[test]
        fn it_handles_nested_ranges() {
            let outer = (1, 20);
            let inner = (5, 10);

            assert_eq!(intersect_of_ranges(outer, inner), Some((5, 10)));
            assert_eq!(intersect_of_ranges(inner, outer), Some((5, 10)));
        }

        #[test]
        fn it_handles_edge_touching() {
            assert_eq!(intersect_of_ranges((1, 10), (10, 20)), Some((10, 10)));
        }
    }

    mod when_ranges_do_not_intersect {
        use super::*;

        #[test]
        fn it_returns_none_for_adjacent() {
            assert_eq!(intersect_of_ranges((1, 10), (11, 20)), None);
        }

        #[test]
        fn it_returns_none_for_separate() {
            assert_eq!(intersect_of_ranges((1, 10), (20, 30)), None);
        }
    }
}

// ============================================================================
// Feature: merge_ranges function
// ============================================================================

mod merge_ranges_feature {
    use super::*;

    mod when_input_is_empty {
        use super::*;

        #[test]
        fn it_returns_empty_vector() {
            let ranges: Vec<(usize, usize)> = vec![];
            let result = merge_ranges(&ranges);

            assert!(result.is_empty());
        }
    }

    mod when_input_has_single_range {
        use super::*;

        #[test]
        fn it_returns_that_range() {
            let ranges = vec![(5, 10)];
            let result = merge_ranges(&ranges);

            assert_eq!(result, vec![(5, 10)]);
        }

        #[test]
        fn it_handles_single_point_range() {
            let ranges = vec![(1, 1)];
            let result = merge_ranges(&ranges);

            assert_eq!(result, vec![(1, 1)]);
        }
    }

    mod when_ranges_do_not_overlap {
        use super::*;

        #[test]
        fn it_keeps_ranges_separate() {
            let ranges = vec![(1, 10), (20, 30)];
            let result = merge_ranges(&ranges);

            assert_eq!(result, vec![(1, 10), (20, 30)]);
        }

        #[test]
        fn it_sorts_by_start_line() {
            let ranges = vec![(20, 30), (1, 10), (50, 60)];
            let result = merge_ranges(&ranges);

            assert_eq!(result, vec![(1, 10), (20, 30), (50, 60)]);
        }

        #[test]
        fn it_handles_multiple_non_overlapping() {
            let ranges = vec![(1, 5), (10, 15), (20, 25), (30, 35)];
            let result = merge_ranges(&ranges);

            assert_eq!(result, vec![(1, 5), (10, 15), (20, 25), (30, 35)]);
        }
    }

    mod when_ranges_overlap {
        use super::*;

        #[test]
        fn it_merges_two_overlapping_ranges() {
            let ranges = vec![(1, 10), (5, 15)];
            let result = merge_ranges(&ranges);

            assert_eq!(result, vec![(1, 15)]);
        }

        #[test]
        fn it_merges_adjacent_ranges() {
            let ranges = vec![(1, 10), (11, 20)];
            let result = merge_ranges(&ranges);

            assert_eq!(result, vec![(1, 20)]);
        }

        #[test]
        fn it_merges_chained_overlapping_ranges() {
            // (1,10) overlaps (5,15) which overlaps (14,20)
            let ranges = vec![(1, 10), (5, 15), (14, 20)];
            let result = merge_ranges(&ranges);

            assert_eq!(result, vec![(1, 20)]);
        }

        #[test]
        fn it_absorbs_nested_ranges() {
            let ranges = vec![(1, 20), (5, 10)];
            let result = merge_ranges(&ranges);

            assert_eq!(result, vec![(1, 20)]);
        }

        #[test]
        fn it_handles_complex_merge_scenario() {
            let ranges = vec![(5, 10), (1, 3), (2, 4), (6, 8), (15, 20), (18, 25)];
            let result = merge_ranges(&ranges);

            // (1,3) + (2,4) -> (1,4)
            // (1,4) + (5,10) -> adjacent -> (1,10)
            // (5,10) + (6,8) -> (1,10)
            // (15,20) + (18,25) -> (15,25)
            assert_eq!(result, vec![(1, 10), (15, 25)]);
        }
    }

    mod when_input_is_unsorted {
        use super::*;

        #[test]
        fn it_sorts_and_merges_correctly() {
            let ranges = vec![(20, 30), (1, 10), (5, 15)];
            let result = merge_ranges(&ranges);

            assert_eq!(result, vec![(1, 15), (20, 30)]);
        }

        #[test]
        fn it_handles_reverse_sorted() {
            let ranges = vec![(30, 40), (20, 30), (10, 20)];
            let result = merge_ranges(&ranges);

            // All adjacent, should merge to one
            assert_eq!(result, vec![(10, 40)]);
        }
    }

    mod with_line_range_type {
        use super::*;

        #[test]
        fn it_works_with_line_range_struct() {
            let ranges = vec![
                LineRange::new(1, 10),
                LineRange::new(5, 15),
                LineRange::new(20, 25),
            ];
            let result = merge_ranges(&ranges);

            assert_eq!(result, vec![(1, 15), (20, 25)]);
        }

        #[test]
        fn it_handles_single_line_range() {
            let ranges = vec![LineRange::new(5, 10)];
            let result = merge_ranges(&ranges);

            assert_eq!(result, vec![(5, 10)]);
        }
    }
}

// ============================================================================
// Feature: merge_ranges_inplace function
// ============================================================================

mod merge_ranges_inplace_feature {
    use super::*;

    mod when_input_is_empty {
        use super::*;

        #[test]
        fn it_leaves_vector_empty() {
            let mut ranges: Vec<(usize, usize)> = vec![];
            merge_ranges_inplace(&mut ranges);

            assert!(ranges.is_empty());
        }
    }

    mod when_input_has_single_range {
        use super::*;

        #[test]
        fn it_keeps_single_range() {
            let mut ranges = vec![(5, 10)];
            merge_ranges_inplace(&mut ranges);

            assert_eq!(ranges, vec![(5, 10)]);
        }
    }

    mod when_ranges_need_merging {
        use super::*;

        #[test]
        fn it_merges_in_place() {
            let mut ranges = vec![(1, 10), (5, 15)];
            merge_ranges_inplace(&mut ranges);

            assert_eq!(ranges, vec![(1, 15)]);
        }

        #[test]
        fn it_reduces_vector_size() {
            let mut ranges = vec![(1, 10), (5, 15), (20, 30), (25, 35)];
            let original_len = ranges.len();
            merge_ranges_inplace(&mut ranges);

            assert!(ranges.len() < original_len);
            assert_eq!(ranges, vec![(1, 15), (20, 35)]);
        }

        #[test]
        fn it_handles_complex_merge() {
            let mut ranges = vec![(20, 30), (1, 10), (5, 15), (25, 35)];
            merge_ranges_inplace(&mut ranges);

            assert_eq!(ranges, vec![(1, 15), (20, 35)]);
        }
    }

    mod efficiency {
        use super::*;

        #[test]
        fn it_preserves_capacity_for_large_inputs() {
            let mut ranges: Vec<(usize, usize)> = (0..1000)
                .map(|i| (i * 20, i * 20 + 10))
                .collect();

            merge_ranges_inplace(&mut ranges);

            // All ranges are separated by gaps, so none should merge
            assert_eq!(ranges.len(), 1000);
            // Capacity should be at least as large as the result
            assert!(ranges.capacity() >= 1000);
        }

        #[test]
        fn it_dramatically_reduces_size_for_overlapping() {
            let mut ranges: Vec<(usize, usize)> = (0..1000)
                .map(|i| (1, i + 10))
                .collect();

            merge_ranges_inplace(&mut ranges);

            // All ranges overlap, should merge to single range
            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0], (1, 1009));
        }
    }
}

// ============================================================================
// Feature: range_contains function
// ============================================================================

mod range_contains_feature {
    use super::*;

    mod when_inner_is_contained {
        use super::*;

        #[test]
        fn it_returns_true_for_nested_range() {
            assert!(range_contains((1, 20), (5, 10)));
        }

        #[test]
        fn it_returns_true_for_identical_range() {
            assert!(range_contains((5, 10), (5, 10)));
        }

        #[test]
        fn it_returns_true_for_edge_alignment() {
            assert!(range_contains((1, 20), (1, 10)));
            assert!(range_contains((1, 20), (10, 20)));
        }
    }

    mod when_inner_is_not_contained {
        use super::*;

        #[test]
        fn it_returns_false_for_larger_range() {
            assert!(!range_contains((5, 10), (1, 20)));
        }

        #[test]
        fn it_returns_false_for_partially_overlapping() {
            assert!(!range_contains((5, 10), (1, 7)));
            assert!(!range_contains((5, 10), (7, 15)));
        }

        #[test]
        fn it_returns_false_for_separate_range() {
            assert!(!range_contains((1, 10), (20, 30)));
        }
    }
}

// ============================================================================
// Feature: line_in_range function
// ============================================================================

mod line_in_range_feature {
    use super::*;

    #[test]
    fn it_returns_true_for_line_at_start() {
        assert!(line_in_range((1, 10), 1));
    }

    #[test]
    fn it_returns_true_for_line_at_end() {
        assert!(line_in_range((1, 10), 10));
    }

    #[test]
    fn it_returns_true_for_line_in_middle() {
        assert!(line_in_range((1, 10), 5));
    }

    #[test]
    fn it_returns_false_for_line_before_start() {
        assert!(!line_in_range((5, 10), 4));
    }

    #[test]
    fn it_returns_false_for_line_after_end() {
        assert!(!line_in_range((5, 10), 11));
    }

    #[test]
    fn it_handles_single_point_range() {
        assert!(line_in_range((5, 5), 5));
        assert!(!line_in_range((5, 5), 4));
        assert!(!line_in_range((5, 5), 6));
    }
}

// ============================================================================
// Feature: range_len function
// ============================================================================

mod range_len_feature {
    use super::*;

    #[test]
    fn it_returns_1_for_single_point() {
        assert_eq!(range_len((5, 5)), 1);
    }

    #[test]
    fn it_counts_inclusive_lines() {
        assert_eq!(range_len((1, 5)), 5); // 1, 2, 3, 4, 5
        assert_eq!(range_len((5, 10)), 6); // 5, 6, 7, 8, 9, 10
    }

    #[test]
    fn it_handles_large_ranges() {
        assert_eq!(range_len((1, 1000)), 1000);
    }

    #[test]
    fn it_saturates_for_invalid_range() {
        // Invalid range where end < start
        assert_eq!(range_len((10, 5)), 1); // saturates
    }
}

// ============================================================================
// Feature: total_covered_lines function
// ============================================================================

mod total_covered_lines_feature {
    use super::*;

    #[test]
    fn it_returns_zero_for_empty_input() {
        let ranges: Vec<(usize, usize)> = vec![];
        assert_eq!(total_covered_lines(&ranges), 0);
    }

    #[test]
    fn it_counts_single_range() {
        let ranges = vec![(1, 10)];
        assert_eq!(total_covered_lines(&ranges), 10);
    }

    #[test]
    fn it_counts_after_merging() {
        let ranges = vec![(1, 10), (5, 15)];
        // Merged: (1, 15) = 15 lines
        assert_eq!(total_covered_lines(&ranges), 15);
    }

    #[test]
    fn it_counts_separate_ranges() {
        let ranges = vec![(1, 10), (20, 30)];
        // (1, 10) = 10 lines, (20, 30) = 11 lines
        assert_eq!(total_covered_lines(&ranges), 21);
    }

    #[test]
    fn it_counts_complex_scenario() {
        let ranges = vec![(1, 10), (5, 15), (20, 25)];
        // Merged: (1, 15) = 15 lines, (20, 25) = 6 lines
        assert_eq!(total_covered_lines(&ranges), 21);
    }
}

// ============================================================================
// Feature: find_gaps function
// ============================================================================

mod find_gaps_feature {
    use super::*;

    #[test]
    fn it_returns_empty_for_single_range() {
        let ranges = vec![(1, 10)];
        assert!(find_gaps(&ranges).is_empty());
    }

    #[test]
    fn it_returns_empty_for_overlapping_ranges() {
        let ranges = vec![(1, 10), (5, 15)];
        assert!(find_gaps(&ranges).is_empty());
    }

    #[test]
    fn it_returns_empty_for_adjacent_ranges() {
        let ranges = vec![(1, 10), (11, 20)];
        assert!(find_gaps(&ranges).is_empty());
    }

    #[test]
    fn it_finds_single_gap() {
        let ranges = vec![(1, 10), (20, 30)];
        let gaps = find_gaps(&ranges);

        assert_eq!(gaps, vec![(11, 19)]);
    }

    #[test]
    fn it_finds_multiple_gaps() {
        let ranges = vec![(1, 10), (20, 30), (40, 50)];
        let gaps = find_gaps(&ranges);

        assert_eq!(gaps, vec![(11, 19), (31, 39)]);
    }

    #[test]
    fn it_handles_merged_ranges() {
        let ranges = vec![(1, 10), (5, 15), (30, 40)];
        // Merged: (1, 15), (30, 40)
        let gaps = find_gaps(&ranges);

        assert_eq!(gaps, vec![(16, 29)]);
    }

    #[test]
    fn it_returns_empty_for_empty_input() {
        let ranges: Vec<(usize, usize)> = vec![];
        assert!(find_gaps(&ranges).is_empty());
    }
}

// ============================================================================
// Feature: LineRangeOps trait
// ============================================================================

mod line_range_ops_trait {
    use super::*;

    #[test]
    fn it_works_with_tuple() {
        let tuple: (usize, usize) = (5, 10);
        assert_eq!(tuple.start(), 5);
        assert_eq!(tuple.end(), 10);
    }

    #[test]
    fn it_works_with_line_range() {
        let range = LineRange::new(5, 10);
        assert_eq!(range.start(), 5);
        assert_eq!(range.end(), 10);
    }

    #[test]
    fn it_enables_generic_functions() {
        fn get_start<T: LineRangeOps>(range: &T) -> usize {
            range.start()
        }

        let tuple = (5, 10);
        let line_range = LineRange::new(5, 10);

        assert_eq!(get_start(&tuple), 5);
        assert_eq!(get_start(&line_range), 5);
    }
}

// ============================================================================
// Property-based tests using proptest
// ============================================================================

mod property_tests {
    use super::*;

    proptest! {
        #[test]
        fn ranges_overlap_is_symmetric(a: (usize, usize), b: (usize, usize)) {
            // Ensure valid ranges
            let a = if a.0 > a.1 { (a.1, a.0) } else { a };
            let b = if b.0 > b.1 { (b.1, b.0) } else { b };

            prop_assert_eq!(ranges_overlap(a, b), ranges_overlap(b, a));
        }

        #[test]
        fn ranges_intersect_is_symmetric(a: (usize, usize), b: (usize, usize)) {
            let a = if a.0 > a.1 { (a.1, a.0) } else { a };
            let b = if b.0 > b.1 { (b.1, b.0) } else { b };

            prop_assert_eq!(ranges_intersect(a, b), ranges_intersect(b, a));
        }

        #[test]
        fn union_of_ranges_is_commutative(a: (usize, usize), b: (usize, usize)) {
            let a = if a.0 > a.1 { (a.1, a.0) } else { a };
            let b = if b.0 > b.1 { (b.1, b.0) } else { b };

            // Only test when ranges overlap
            if ranges_overlap(a, b) {
                prop_assert_eq!(union_of_ranges(a, b), union_of_ranges(b, a));
            }
        }

        #[test]
        fn merge_ranges_never_increases_count(ranges: Vec<(usize, usize)>) {
            // Normalize ranges
            let ranges: Vec<(usize, usize)> = ranges
                .into_iter()
                .map(|(a, b)| if a > b { (b, a) } else { (a, b) })
                .collect();

            let merged = merge_ranges(&ranges);
            prop_assert!(merged.len() <= ranges.len());
        }

        #[test]
        fn merge_ranges_result_is_sorted(ranges: Vec<(usize, usize)>) {
            let ranges: Vec<(usize, usize)> = ranges
                .into_iter()
                .map(|(a, b)| if a > b { (b, a) } else { (a, b) })
                .collect();

            let merged = merge_ranges(&ranges);

            for window in merged.windows(2) {
                prop_assert!(window[0].0 <= window[1].0);
            }
        }

        #[test]
        fn merge_ranges_result_has_no_overlaps(ranges: Vec<(usize, usize)>) {
            let ranges: Vec<(usize, usize)> = ranges
                .into_iter()
                .map(|(a, b)| if a > b { (b, a) } else { (a, b) })
                .collect();

            let merged = merge_ranges(&ranges);

            for window in merged.windows(2) {
                // Adjacent or overlapping ranges should be merged
                prop_assert!(!ranges_overlap(window[0], window[1]));
            }
        }

        #[test]
        fn merge_ranges_inplace_equals_merge_ranges(mut ranges: Vec<(usize, usize)>) {
            // Normalize ranges first (ensure start <= end)
            ranges = ranges
                .into_iter()
                .map(|(a, b)| if a > b { (b, a) } else { (a, b) })
                .collect();

            let ranges_copy = ranges.clone();

            let expected = merge_ranges(&ranges_copy);
            merge_ranges_inplace(&mut ranges);

            prop_assert_eq!(ranges, expected);
        }

        #[test]
        fn total_covered_lines_is_positive(ranges: Vec<(usize, usize)>) {
            let ranges: Vec<(usize, usize)> = ranges
                .into_iter()
                .map(|(a, b)| if a > b { (b, a) } else { (a, b) })
                .collect();

            if !ranges.is_empty() {
                let total = total_covered_lines(&ranges);
                prop_assert!(total > 0);
            }
        }

        #[test]
        fn intersect_of_ranges_is_subset(a: (usize, usize), b: (usize, usize)) {
            let a = if a.0 > a.1 { (a.1, a.0) } else { a };
            let b = if b.0 > b.1 { (b.1, b.0) } else { b };

            if let Some(intersection) = intersect_of_ranges(a, b) {
                prop_assert!(range_contains(a, intersection));
                prop_assert!(range_contains(b, intersection));
            }
        }

        #[test]
        fn range_len_is_consistent(range: (usize, usize)) {
            let range = if range.0 > range.1 { (range.1, range.0) } else { range };

            let len = range_len(range);
            let expected = range.1 - range.0 + 1;

            prop_assert_eq!(len, expected);
        }

        #[test]
        fn find_gaps_returns_valid_gaps(ranges: Vec<(usize, usize)>) {
            let ranges: Vec<(usize, usize)> = ranges
                .into_iter()
                .map(|(a, b)| if a > b { (b, a) } else { (a, b) })
                .collect();

            let merged = merge_ranges(&ranges);
            let gaps = find_gaps(&ranges);

            // Gaps should be between merged ranges
            if merged.len() > 1 && !gaps.is_empty() {
                for gap in &gaps {
                    // Gap should not overlap with any merged range
                    for merged_range in &merged {
                        prop_assert!(!ranges_intersect(*gap, *merged_range));
                    }
                }
            }
        }
    }
}

// ============================================================================
// Integration tests combining multiple functions
// ============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn diff_hunk_merging_scenario() {
        // Simulate diff hunks from a file
        let hunks = vec![
            (10, 20),  // First change
            (15, 25),  // Overlapping change
            (50, 60),  // Separate change
            (55, 65),  // Overlapping with third
            (100, 110), // Separate change
        ];

        let merged = merge_ranges(&hunks);

        assert_eq!(merged, vec![(10, 25), (50, 65), (100, 110)]);

        // Verify total lines changed
        let total = total_covered_lines(&hunks);
        assert_eq!(total, 16 + 16 + 11); // 43 lines total
    }

    #[test]
    fn diagnostic_range_merging_scenario() {
        // Simulate diagnostic line ranges
        let diagnostics = vec![
            (1, 1),   // Single line error
            (5, 10),  // Multi-line warning
            (8, 12),  // Overlapping with above
            (20, 25), // Separate
            (21, 21), // Inside the above
        ];

        let merged = merge_ranges(&diagnostics);

        assert_eq!(merged, vec![(1, 1), (5, 12), (20, 25)]);
    }

    #[test]
    fn coverage_calculation_scenario() {
        // Calculate coverage percentage
        let file_lines = 100;
        let covered = vec![(1, 10), (15, 25), (30, 50), (55, 60), (70, 85)];

        let total_covered = total_covered_lines(&covered);
        let coverage_percent = (total_covered as f64 / file_lines as f64) * 100.0;

        // 10 + 11 + 21 + 6 + 16 = 64 lines
        assert_eq!(total_covered, 64);
        assert!((coverage_percent - 64.0).abs() < 0.01);
    }

    #[test]
    fn gap_finding_for_review_scenario() {
        // Find gaps in code review coverage
        let reviewed_ranges = vec![(1, 50), (100, 150), (200, 250)];
        let gaps = find_gaps(&reviewed_ranges);

        assert_eq!(gaps, vec![(51, 99), (151, 199)]);
    }

    #[test]
    fn large_scale_merge_performance() {
        // Test with many ranges that are NOT adjacent (gap of 1 line between each)
        let ranges: Vec<(usize, usize)> = (0..100)
            .map(|i| (i * 4, i * 4 + 2)) // Ranges like (0,2), (4,6), (8,10) - gap of 1
            .collect();

        let merged = merge_ranges(&ranges);

        // None should merge since they're all separated by at least 1 line
        assert_eq!(merged.len(), 100);
    }

    #[test]
    fn worst_case_merge_scenario() {
        // All ranges overlap - worst case for merge algorithm
        let ranges: Vec<(usize, usize)> = (0..100)
            .map(|i| (1, 100 + i))
            .collect();

        let merged = merge_ranges(&ranges);

        // Should all merge to single range
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], (1, 199));
    }
}
