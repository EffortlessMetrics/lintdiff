//! Comprehensive tests for lintdiff-span-intersect.
//!
//! These tests cover all public API functions and edge cases.

use lintdiff_line_range::LineRange;
use lintdiff_span_intersect::*;

// ============================================================================
// ranges_intersect Tests
// ============================================================================

mod ranges_intersect_tests {
    use super::*;

    #[test]
    fn overlapping_ranges_intersect() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);
        assert!(ranges_intersect(&a, &b));
    }

    #[test]
    fn overlapping_ranges_is_commutative() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);
        assert_eq!(ranges_intersect(&a, &b), ranges_intersect(&b, &a));
    }

    #[test]
    fn disjoint_ranges_do_not_intersect() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(20, 30);
        assert!(!ranges_intersect(&a, &b));
    }

    #[test]
    fn adjacent_ranges_do_not_intersect() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(11, 20);
        assert!(!ranges_intersect(&a, &b));
    }

    #[test]
    fn contained_range_intersects() {
        let outer = LineRange::new(1, 20);
        let inner = LineRange::new(5, 10);
        assert!(ranges_intersect(&outer, &inner));
        assert!(ranges_intersect(&inner, &outer));
    }

    #[test]
    fn identical_ranges_intersect() {
        let a = LineRange::new(1, 10);
        assert!(ranges_intersect(&a, &a));
    }

    #[test]
    fn single_line_overlapping() {
        let a = LineRange::new(5, 5);
        let b = LineRange::new(1, 10);
        assert!(ranges_intersect(&a, &b));
    }

    #[test]
    fn single_line_disjoint() {
        let a = LineRange::new(5, 5);
        let b = LineRange::new(10, 20);
        assert!(!ranges_intersect(&a, &b));
    }

    #[test]
    fn boundary_touch_start() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(10, 20);
        assert!(ranges_intersect(&a, &b));
    }

    #[test]
    fn boundary_touch_end() {
        let a = LineRange::new(10, 20);
        let b = LineRange::new(1, 10);
        assert!(ranges_intersect(&a, &b));
    }
}

// ============================================================================
// range_intersection Tests
// ============================================================================

mod range_intersection_tests {
    use super::*;

    #[test]
    fn overlapping_returns_intersection() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);
        let result = range_intersection(&a, &b).unwrap();
        assert_eq!(result.start, 5);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn disjoint_returns_none() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(20, 30);
        assert!(range_intersection(&a, &b).is_none());
    }

    #[test]
    fn contained_returns_inner() {
        let outer = LineRange::new(1, 20);
        let inner = LineRange::new(5, 10);
        let result = range_intersection(&outer, &inner).unwrap();
        assert_eq!(result.start, 5);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn identical_returns_same() {
        let a = LineRange::new(1, 10);
        let result = range_intersection(&a, &a).unwrap();
        assert_eq!(result.start, a.start);
        assert_eq!(result.end, a.end);
    }

    #[test]
    fn single_line_intersection() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(10, 20);
        let result = range_intersection(&a, &b).unwrap();
        assert_eq!(result.start, 10);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn is_commutative() {
        let a = LineRange::new(1, 15);
        let b = LineRange::new(10, 20);
        let ab = range_intersection(&a, &b).unwrap();
        let ba = range_intersection(&b, &a).unwrap();
        assert_eq!(ab.start, ba.start);
        assert_eq!(ab.end, ba.end);
    }
}

// ============================================================================
// line_in_ranges Tests
// ============================================================================

mod line_in_ranges_tests {
    use super::*;

    #[test]
    fn line_in_single_range() {
        let ranges = vec![LineRange::new(5, 10)];
        assert!(line_in_ranges(5, &ranges));
        assert!(line_in_ranges(7, &ranges));
        assert!(line_in_ranges(10, &ranges));
    }

    #[test]
    fn line_not_in_range() {
        let ranges = vec![LineRange::new(5, 10)];
        assert!(!line_in_ranges(4, &ranges));
        assert!(!line_in_ranges(11, &ranges));
    }

    #[test]
    fn line_in_multiple_ranges() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(20, 30),
            LineRange::new(40, 50),
        ];
        assert!(line_in_ranges(5, &ranges));
        assert!(line_in_ranges(25, &ranges));
        assert!(line_in_ranges(45, &ranges));
    }

    #[test]
    fn line_between_ranges() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        assert!(!line_in_ranges(15, &ranges));
    }

    #[test]
    fn empty_ranges() {
        let ranges: Vec<LineRange> = vec![];
        assert!(!line_in_ranges(5, &ranges));
    }

    #[test]
    fn boundary_values() {
        let ranges = vec![LineRange::new(5, 10)];
        assert!(line_in_ranges(5, &ranges)); // Start
        assert!(line_in_ranges(10, &ranges)); // End
    }

    #[test]
    fn overlapping_ranges() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(5, 15)];
        assert!(line_in_ranges(7, &ranges)); // In both
        assert!(line_in_ranges(3, &ranges)); // Only in first
        assert!(line_in_ranges(12, &ranges)); // Only in second
    }
}

// ============================================================================
// find_containing_ranges Tests
// ============================================================================

mod find_containing_ranges_tests {
    use super::*;

    #[test]
    fn finds_single_range() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        let result = find_containing_ranges(5, &ranges);
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn finds_multiple_ranges() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(5, 15),
            LineRange::new(20, 30),
        ];
        let result = find_containing_ranges(7, &ranges);
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn finds_no_ranges() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        let result = find_containing_ranges(15, &ranges);
        assert!(result.is_empty());
    }

    #[test]
    fn empty_ranges_returns_empty() {
        let ranges: Vec<LineRange> = vec![];
        let result = find_containing_ranges(5, &ranges);
        assert!(result.is_empty());
    }

    #[test]
    fn finds_all_when_all_contain() {
        let ranges = vec![
            LineRange::new(1, 20),
            LineRange::new(5, 15),
            LineRange::new(10, 30),
        ];
        let result = find_containing_ranges(10, &ranges);
        assert_eq!(result, vec![0, 1, 2]);
    }
}

// ============================================================================
// find_first_containing Tests
// ============================================================================

mod find_first_containing_tests {
    use super::*;

    #[test]
    fn finds_first_match() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(5, 15)];
        assert_eq!(find_first_containing(7, &ranges), Some(0));
    }

    #[test]
    fn finds_later_match() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        assert_eq!(find_first_containing(25, &ranges), Some(1));
    }

    #[test]
    fn no_match_returns_none() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        assert_eq!(find_first_containing(15, &ranges), None);
    }

    #[test]
    fn empty_ranges_returns_none() {
        let ranges: Vec<LineRange> = vec![];
        assert_eq!(find_first_containing(5, &ranges), None);
    }
}

// ============================================================================
// count_containing Tests
// ============================================================================

mod count_containing_tests {
    use super::*;

    #[test]
    fn counts_zero() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        assert_eq!(count_containing(15, &ranges), 0);
    }

    #[test]
    fn counts_one() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        assert_eq!(count_containing(5, &ranges), 1);
    }

    #[test]
    fn counts_multiple() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(5, 15),
            LineRange::new(8, 20),
        ];
        assert_eq!(count_containing(9, &ranges), 3);
    }

    #[test]
    fn empty_ranges_counts_zero() {
        let ranges: Vec<LineRange> = vec![];
        assert_eq!(count_containing(5, &ranges), 0);
    }
}

// ============================================================================
// range_intersects_any Tests
// ============================================================================

mod range_intersects_any_tests {
    use super::*;

    #[test]
    fn intersects_one() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        assert!(range_intersects_any(&LineRange::new(5, 15), &ranges));
    }

    #[test]
    fn intersects_none() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        assert!(!range_intersects_any(&LineRange::new(12, 18), &ranges));
    }

    #[test]
    fn intersects_multiple() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(5, 15),
            LineRange::new(20, 30),
        ];
        assert!(range_intersects_any(&LineRange::new(8, 25), &ranges));
    }

    #[test]
    fn empty_ranges_returns_false() {
        let ranges: Vec<LineRange> = vec![];
        assert!(!range_intersects_any(&LineRange::new(1, 10), &ranges));
    }
}

// ============================================================================
// find_intersecting_ranges Tests
// ============================================================================

mod find_intersecting_ranges_tests {
    use super::*;

    #[test]
    fn finds_all_intersecting() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(5, 15),
            LineRange::new(20, 30),
        ];
        let query = LineRange::new(8, 25);
        let result = find_intersecting_ranges(&query, &ranges);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn finds_subset() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(20, 30),
            LineRange::new(40, 50),
        ];
        let query = LineRange::new(5, 25);
        let result = find_intersecting_ranges(&query, &ranges);
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn finds_none() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        let query = LineRange::new(12, 18);
        let result = find_intersecting_ranges(&query, &ranges);
        assert!(result.is_empty());
    }

    #[test]
    fn empty_ranges_returns_empty() {
        let ranges: Vec<LineRange> = vec![];
        let result = find_intersecting_ranges(&LineRange::new(1, 10), &ranges);
        assert!(result.is_empty());
    }
}

// ============================================================================
// merge_intersecting_ranges Tests
// ============================================================================

mod merge_intersecting_ranges_tests {
    use super::*;

    #[test]
    fn merges_overlapping() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(5, 15)];
        let merged = merge_intersecting_ranges(&ranges);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], LineRange::new(1, 15));
    }

    #[test]
    fn merges_adjacent() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(11, 20)];
        let merged = merge_intersecting_ranges(&ranges);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], LineRange::new(1, 20));
    }

    #[test]
    fn keeps_disjoint_separate() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        let merged = merge_intersecting_ranges(&ranges);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn empty_input() {
        let ranges: Vec<LineRange> = vec![];
        let merged = merge_intersecting_ranges(&ranges);
        assert!(merged.is_empty());
    }

    #[test]
    fn single_range() {
        let ranges = vec![LineRange::new(1, 10)];
        let merged = merge_intersecting_ranges(&ranges);
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn complex_merge() {
        let ranges = vec![
            LineRange::new(5, 10),
            LineRange::new(1, 3),
            LineRange::new(2, 7),
            LineRange::new(20, 25),
            LineRange::new(24, 30),
        ];
        let merged = merge_intersecting_ranges(&ranges);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], LineRange::new(1, 10));
        assert_eq!(merged[1], LineRange::new(20, 30));
    }

    #[test]
    fn unsorted_input_is_sorted() {
        let ranges = vec![
            LineRange::new(20, 30),
            LineRange::new(1, 10),
            LineRange::new(5, 15),
        ];
        let merged = merge_intersecting_ranges(&ranges);
        assert_eq!(merged[0].start, 1);
    }
}

// ============================================================================
// total_covered_lines Tests
// ============================================================================

mod total_covered_lines_tests {
    use super::*;

    #[test]
    fn counts_no_overlap() {
        let ranges = vec![
            LineRange::new(1, 10),  // 10 lines
            LineRange::new(20, 30), // 11 lines
        ];
        assert_eq!(total_covered_lines(&ranges), 21);
    }

    #[test]
    fn counts_with_overlap() {
        let ranges = vec![
            LineRange::new(1, 10), // 10 lines
            LineRange::new(5, 15), // 5 new lines (11-15)
        ];
        assert_eq!(total_covered_lines(&ranges), 15);
    }

    #[test]
    fn empty_ranges() {
        let ranges: Vec<LineRange> = vec![];
        assert_eq!(total_covered_lines(&ranges), 0);
    }

    #[test]
    fn single_range() {
        let ranges = vec![LineRange::new(1, 10)];
        assert_eq!(total_covered_lines(&ranges), 10);
    }

    #[test]
    fn single_line_ranges() {
        let ranges = vec![
            LineRange::new(1, 1),
            LineRange::new(2, 2),
            LineRange::new(3, 3),
        ];
        assert_eq!(total_covered_lines(&ranges), 3);
    }
}

// ============================================================================
// is_sorted_and_disjoint Tests
// ============================================================================

mod is_sorted_and_disjoint_tests {
    use super::*;

    #[test]
    fn sorted_and_disjoint() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(20, 30),
            LineRange::new(40, 50),
        ];
        assert!(is_sorted_and_disjoint(&ranges));
    }

    #[test]
    fn overlapping_fails() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(5, 15)];
        assert!(!is_sorted_and_disjoint(&ranges));
    }

    #[test]
    fn unsorted_fails() {
        let ranges = vec![LineRange::new(20, 30), LineRange::new(1, 10)];
        assert!(!is_sorted_and_disjoint(&ranges));
    }

    #[test]
    fn adjacent_is_disjoint() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(11, 20)];
        assert!(is_sorted_and_disjoint(&ranges));
    }

    #[test]
    fn empty_is_sorted() {
        let ranges: Vec<LineRange> = vec![];
        assert!(is_sorted_and_disjoint(&ranges));
    }

    #[test]
    fn single_is_sorted() {
        let ranges = vec![LineRange::new(1, 10)];
        assert!(is_sorted_and_disjoint(&ranges));
    }
}

// ============================================================================
// Property-based Tests
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Generate a valid line range where start <= end
    prop_compose! {
        fn arb_line_range()(start in 1u32..1000u32, len in 0u32..100u32) -> LineRange {
            LineRange::new(start, start + len)
        }
    }

    proptest! {
        #[test]
        fn ranges_intersect_is_commutative(a in arb_line_range(), b in arb_line_range()) {
            prop_assert_eq!(ranges_intersect(&a, &b), ranges_intersect(&b, &a));
        }

        #[test]
        fn range_intersection_is_commutative(a in arb_line_range(), b in arb_line_range()) {
            let ab = range_intersection(&a, &b);
            let ba = range_intersection(&b, &a);
            prop_assert_eq!(ab, ba);
        }

        #[test]
        fn range_intersection_is_subset(a in arb_line_range(), b in arb_line_range()) {
            if let Some(intersection) = range_intersection(&a, &b) {
                prop_assert!(intersection.start >= a.start.min(b.start));
                prop_assert!(intersection.end <= a.end.max(b.end));
            }
        }

        #[test]
        fn merge_is_idempotent(ranges in proptest::collection::vec(arb_line_range(), 0..20)) {
            let first_merge = merge_intersecting_ranges(&ranges);
            let second_merge = merge_intersecting_ranges(&first_merge);
            prop_assert_eq!(first_merge, second_merge);
        }

        #[test]
        fn merged_ranges_are_disjoint(ranges in proptest::collection::vec(arb_line_range(), 0..20)) {
            let merged = merge_intersecting_ranges(&ranges);
            prop_assert!(is_sorted_and_disjoint(&merged));
        }

        #[test]
        fn total_covered_lines_matches_merged(ranges in proptest::collection::vec(arb_line_range(), 0..20)) {
            let merged = merge_intersecting_ranges(&ranges);
            let expected: u64 = merged.iter().map(|r| (r.end - r.start + 1) as u64).sum();
            prop_assert_eq!(total_covered_lines(&ranges), expected);
        }

        #[test]
        fn line_in_ranges_consistent_with_find(line in 1u32..1000u32, ranges in proptest::collection::vec(arb_line_range(), 0..10)) {
            let in_ranges = line_in_ranges(line, &ranges);
            let found = find_first_containing(line, &ranges);
            prop_assert_eq!(in_ranges, found.is_some());
        }

        #[test]
        fn count_containing_matches_find_count(line in 1u32..1000u32, ranges in proptest::collection::vec(arb_line_range(), 0..10)) {
            let count = count_containing(line, &ranges);
            let found = find_containing_ranges(line, &ranges);
            prop_assert_eq!(count, found.len() as usize);
        }
    }
}
