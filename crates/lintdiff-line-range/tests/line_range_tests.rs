//! Comprehensive tests for lintdiff-line-range.
//!
//! These tests cover all public API functions and edge cases.

use lintdiff_line_range::{merge_ranges, range_intersection, ranges_intersect, LineRange};

// ============================================================================
// LineRange Construction Tests
// ============================================================================

mod construction_tests {
    use super::*;

    #[test]
    fn new_creates_range() {
        let range = LineRange::new(1, 10);
        assert_eq!(range.start, 1);
        assert_eq!(range.end, 10);
    }

    #[test]
    fn new_single_line_range() {
        let range = LineRange::new(5, 5);
        assert_eq!(range.start, 5);
        assert_eq!(range.end, 5);
    }

    #[test]
    fn from_start_end_creates_range() {
        let range = LineRange::from_start_end(5, 10);
        assert_eq!(range.start, 5);
        assert_eq!(range.end, 10);
    }

    #[test]
    fn from_start_end_same_values() {
        let range = LineRange::from_start_end(7, 7);
        assert_eq!(range.start, 7);
        assert_eq!(range.end, 7);
    }

    #[test]
    #[should_panic(expected = "Start must be <= end")]
    fn from_start_end_panics_on_invalid_order() {
        let _ = LineRange::from_start_end(10, 5);
    }

    #[test]
    fn default_is_line_one() {
        let range = LineRange::default();
        assert_eq!(range.start, 1);
        assert_eq!(range.end, 1);
    }
}

// ============================================================================
// LineRange::contains Tests
// ============================================================================

mod contains_tests {
    use super::*;

    #[test]
    fn contains_start_boundary() {
        let range = LineRange::new(5, 10);
        assert!(range.contains(5));
    }

    #[test]
    fn contains_end_boundary() {
        let range = LineRange::new(5, 10);
        assert!(range.contains(10));
    }

    #[test]
    fn contains_middle() {
        let range = LineRange::new(5, 10);
        assert!(range.contains(7));
    }

    #[test]
    fn does_not_contain_before_start() {
        let range = LineRange::new(5, 10);
        assert!(!range.contains(4));
    }

    #[test]
    fn does_not_contain_after_end() {
        let range = LineRange::new(5, 10);
        assert!(!range.contains(11));
    }

    #[test]
    fn contains_single_line() {
        let range = LineRange::new(5, 5);
        assert!(range.contains(5));
        assert!(!range.contains(4));
        assert!(!range.contains(6));
    }

    #[test]
    fn contains_zero_line() {
        let range = LineRange::new(1, 10);
        assert!(!range.contains(0));
    }
}

// ============================================================================
// LineRange::len Tests
// ============================================================================

mod len_tests {
    use super::*;

    #[test]
    fn len_single_line() {
        let range = LineRange::new(1, 1);
        assert_eq!(range.len(), 1);
    }

    #[test]
    fn len_multiple_lines() {
        let range = LineRange::new(1, 5);
        assert_eq!(range.len(), 5);
    }

    #[test]
    fn len_includes_both_boundaries() {
        let range = LineRange::new(5, 10);
        assert_eq!(range.len(), 6); // 5, 6, 7, 8, 9, 10
    }

    #[test]
    fn len_large_range() {
        let range = LineRange::new(1, 1000);
        assert_eq!(range.len(), 1000);
    }

    #[test]
    fn len_non_contiguous_start() {
        let range = LineRange::new(100, 200);
        assert_eq!(range.len(), 101);
    }
}

// ============================================================================
// LineRange::is_empty Tests
// ============================================================================

mod is_empty_tests {
    use super::*;

    #[test]
    fn is_empty_always_false() {
        let range = LineRange::new(1, 1);
        assert!(!range.is_empty());
    }

    #[test]
    fn is_empty_false_for_larger_range() {
        let range = LineRange::new(1, 100);
        assert!(!range.is_empty());
    }
}

// ============================================================================
// LineRange::overlaps Tests
// ============================================================================

mod overlaps_tests {
    use super::*;

    #[test]
    fn overlaps_completely() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(3, 7);
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
    }

    #[test]
    fn overlaps_partial_start() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
    }

    #[test]
    fn overlaps_partial_end() {
        let a = LineRange::new(5, 15);
        let b = LineRange::new(1, 10);
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
    }

    #[test]
    fn overlaps_at_boundary() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(10, 20);
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
    }

    #[test]
    fn no_overlaps_before() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(11, 20);
        assert!(!a.overlaps(&b));
        assert!(!b.overlaps(&a));
    }

    #[test]
    fn no_overlaps_after() {
        let a = LineRange::new(11, 20);
        let b = LineRange::new(1, 10);
        assert!(!a.overlaps(&b));
        assert!(!b.overlaps(&a));
    }

    #[test]
    fn overlaps_with_self() {
        let range = LineRange::new(5, 10);
        assert!(range.overlaps(&range));
    }

    #[test]
    fn overlaps_same_range() {
        let a = LineRange::new(5, 10);
        let b = LineRange::new(5, 10);
        assert!(a.overlaps(&b));
    }
}

// ============================================================================
// LineRange::intersection Tests
// ============================================================================

mod intersection_tests {
    use super::*;

    #[test]
    fn intersection_partial_overlap() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);

        let result = a.intersection(&b).unwrap();
        assert_eq!(result.start, 5);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn intersection_complete_containment() {
        let a = LineRange::new(1, 20);
        let b = LineRange::new(5, 10);

        let result = a.intersection(&b).unwrap();
        assert_eq!(result.start, 5);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn intersection_at_boundary() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(10, 20);

        let result = a.intersection(&b).unwrap();
        assert_eq!(result.start, 10);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn intersection_no_overlap() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(11, 20);

        assert!(a.intersection(&b).is_none());
    }

    #[test]
    fn intersection_same_range() {
        let a = LineRange::new(5, 10);
        let b = LineRange::new(5, 10);

        let result = a.intersection(&b).unwrap();
        assert_eq!(result.start, 5);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn intersection_is_symmetric() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);

        let ab = a.intersection(&b).unwrap();
        let ba = b.intersection(&a).unwrap();

        assert_eq!(ab.start, ba.start);
        assert_eq!(ab.end, ba.end);
    }
}

// ============================================================================
// LineRange::merge Tests
// ============================================================================

mod merge_tests {
    use super::*;

    #[test]
    fn merge_overlapping() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);

        let result = a.merge(&b);
        assert_eq!(result.start, 1);
        assert_eq!(result.end, 15);
    }

    #[test]
    fn merge_non_overlapping() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(20, 30);

        let result = a.merge(&b);
        assert_eq!(result.start, 1);
        assert_eq!(result.end, 30);
    }

    #[test]
    fn merge_adjacent() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(11, 20);

        let result = a.merge(&b);
        assert_eq!(result.start, 1);
        assert_eq!(result.end, 20);
    }

    #[test]
    fn merge_same_range() {
        let a = LineRange::new(5, 10);
        let b = LineRange::new(5, 10);

        let result = a.merge(&b);
        assert_eq!(result.start, 5);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn merge_is_symmetric() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(20, 30);

        let ab = a.merge(&b);
        let ba = b.merge(&a);

        assert_eq!(ab.start, ba.start);
        assert_eq!(ab.end, ba.end);
    }

    #[test]
    fn merge_contained_range() {
        let a = LineRange::new(1, 20);
        let b = LineRange::new(5, 10);

        let result = a.merge(&b);
        assert_eq!(result.start, 1);
        assert_eq!(result.end, 20);
    }
}

// ============================================================================
// ranges_intersect Function Tests
// ============================================================================

mod ranges_intersect_tests {
    use super::*;

    #[test]
    fn ranges_intersect_overlapping() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);

        assert!(ranges_intersect(&a, &b));
    }

    #[test]
    fn ranges_intersect_at_boundary() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(10, 20);

        assert!(ranges_intersect(&a, &b));
    }

    #[test]
    fn ranges_intersect_no_overlap() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(11, 20);

        assert!(!ranges_intersect(&a, &b));
    }
}

// ============================================================================
// range_intersection Function Tests
// ============================================================================

mod range_intersection_fn_tests {
    use super::*;

    #[test]
    fn range_intersection_overlapping() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);

        let result = range_intersection(&a, &b).unwrap();
        assert_eq!(result.start, 5);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn range_intersection_no_overlap() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(11, 20);

        assert!(range_intersection(&a, &b).is_none());
    }
}

// ============================================================================
// merge_ranges Function Tests
// ============================================================================

mod merge_ranges_tests {
    use super::*;

    #[test]
    fn merge_ranges_single() {
        let ranges = vec![LineRange::new(5, 10)];
        let result = merge_ranges(&ranges).unwrap();

        assert_eq!(result.start, 5);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn merge_ranges_multiple() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(5, 15),
            LineRange::new(20, 25),
        ];
        let result = merge_ranges(&ranges).unwrap();

        assert_eq!(result.start, 1);
        assert_eq!(result.end, 25);
    }

    #[test]
    fn merge_ranges_empty() {
        let ranges: Vec<LineRange> = vec![];
        assert!(merge_ranges(&ranges).is_none());
    }

    #[test]
    fn merge_ranges_two() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        let result = merge_ranges(&ranges).unwrap();

        assert_eq!(result.start, 1);
        assert_eq!(result.end, 30);
    }

    #[test]
    fn merge_ranges_unordered() {
        let ranges = vec![
            LineRange::new(20, 30),
            LineRange::new(1, 10),
            LineRange::new(5, 15),
        ];
        let result = merge_ranges(&ranges).unwrap();

        assert_eq!(result.start, 1);
        assert_eq!(result.end, 30);
    }
}

// ============================================================================
// Trait Derivation Tests
// ============================================================================

mod trait_tests {
    use super::*;

    #[test]
    fn clone_creates_equal_range() {
        let original = LineRange::new(5, 10);
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn copy_creates_equal_range() {
        let original = LineRange::new(5, 10);
        let copied = original; // Copy happens here

        assert_eq!(original, copied);
    }

    #[test]
    fn equality_works() {
        let a = LineRange::new(5, 10);
        let b = LineRange::new(5, 10);
        let c = LineRange::new(5, 11);

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn ordering_works() {
        let a = LineRange::new(1, 5);
        let b = LineRange::new(2, 5);
        let c = LineRange::new(1, 6);

        assert!(a < b); // Lower start comes first
        assert!(a < c); // Same start, lower end comes first
    }

    #[test]
    fn debug_format() {
        let range = LineRange::new(5, 10);
        let debug_str = format!("{:?}", range);

        assert!(debug_str.contains("LineRange"));
        assert!(debug_str.contains("start"));
        assert!(debug_str.contains("end"));
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn large_line_numbers() {
        let range = LineRange::new(1_000_000, 2_000_000);
        assert_eq!(range.start, 1_000_000);
        assert_eq!(range.end, 2_000_000);
        assert_eq!(range.len(), 1_000_001);
    }

    #[test]
    fn max_line_number() {
        let range = LineRange::new(u32::MAX - 1, u32::MAX);
        assert_eq!(range.len(), 2);
    }

    #[test]
    fn intersection_with_single_line() {
        let a = LineRange::new(5, 5);
        let b = LineRange::new(1, 10);

        let result = a.intersection(&b).unwrap();
        assert_eq!(result.start, 5);
        assert_eq!(result.end, 5);
    }

    #[test]
    fn merge_preserves_minimum_start() {
        let ranges = vec![
            LineRange::new(100, 200),
            LineRange::new(1, 50),
            LineRange::new(75, 150),
        ];
        let result = merge_ranges(&ranges).unwrap();

        assert_eq!(result.start, 1);
        assert_eq!(result.end, 200);
    }
}
