//! Line range intersection detection for lintdiff.
//!
//! This microcrate provides utilities for detecting intersections between
//! line ranges and checking if lines fall within sets of ranges.
//!
//! # Example: Range Intersection
//!
//! ```
//! use lintdiff_span_intersect::{ranges_intersect, range_intersection};
//! use lintdiff_line_range::LineRange;
//!
//! let a = LineRange::new(1, 10);
//! let b = LineRange::new(5, 15);
//!
//! assert!(ranges_intersect(&a, &b));
//! let intersection = range_intersection(&a, &b).unwrap();
//! assert_eq!(intersection.start, 5);
//! assert_eq!(intersection.end, 10);
//! ```
//!
//! # Example: Line in Ranges
//!
//! ```
//! use lintdiff_span_intersect::line_in_ranges;
//! use lintdiff_line_range::LineRange;
//!
//! let ranges = vec![
//!     LineRange::new(1, 10),
//!     LineRange::new(20, 30),
//! ];
//!
//! assert!(line_in_ranges(5, &ranges));
//! assert!(line_in_ranges(25, &ranges));
//! assert!(!line_in_ranges(15, &ranges));
//! ```
//!
//! # Example: Find Containing Ranges
//!
//! ```
//! use lintdiff_span_intersect::find_containing_ranges;
//! use lintdiff_line_range::LineRange;
//!
//! let ranges = vec![
//!     LineRange::new(1, 10),
//!     LineRange::new(5, 15),  // Overlaps with first
//!     LineRange::new(20, 30),
//! ];
//!
//! let containing = find_containing_ranges(7, &ranges);
//! assert_eq!(containing.len(), 2);
//! assert!(containing.contains(&0));
//! assert!(containing.contains(&1));
//! ```

#![warn(missing_docs)]

use lintdiff_line_range::LineRange;
use std::cmp::{max, min};

/// Check if two line ranges intersect.
///
/// Two ranges intersect if they share at least one line number.
/// This is a commutative operation: `ranges_intersect(a, b) == ranges_intersect(b, a)`.
///
/// # Example
///
/// ```
/// use lintdiff_span_intersect::ranges_intersect;
/// use lintdiff_line_range::LineRange;
///
/// let a = LineRange::new(1, 10);
/// let b = LineRange::new(5, 15);
/// let c = LineRange::new(20, 30);
///
/// assert!(ranges_intersect(&a, &b));
/// assert!(!ranges_intersect(&a, &c));
/// ```
#[must_use]
pub const fn ranges_intersect(a: &LineRange, b: &LineRange) -> bool {
    a.start <= b.end && b.start <= a.end
}

/// Find the intersection of two line ranges.
///
/// Returns `Some(LineRange)` if the ranges intersect, containing the
/// overlapping portion. Returns `None` if the ranges do not intersect.
///
/// # Example
///
/// ```
/// use lintdiff_span_intersect::range_intersection;
/// use lintdiff_line_range::LineRange;
///
/// let a = LineRange::new(1, 10);
/// let b = LineRange::new(5, 15);
/// let c = LineRange::new(20, 30);
///
/// let ab = range_intersection(&a, &b).unwrap();
/// assert_eq!(ab.start, 5);
/// assert_eq!(ab.end, 10);
///
/// assert!(range_intersection(&a, &c).is_none());
/// ```
#[must_use]
pub fn range_intersection(a: &LineRange, b: &LineRange) -> Option<LineRange> {
    if !ranges_intersect(a, b) {
        return None;
    }
    Some(LineRange::new(max(a.start, b.start), min(a.end, b.end)))
}

/// Check if a line number falls within any of the given ranges.
///
/// # Example
///
/// ```
/// use lintdiff_span_intersect::line_in_ranges;
/// use lintdiff_line_range::LineRange;
///
/// let ranges = vec![
///     LineRange::new(1, 10),
///     LineRange::new(20, 30),
/// ];
///
/// assert!(line_in_ranges(5, &ranges));
/// assert!(line_in_ranges(1, &ranges));   // Start boundary
/// assert!(line_in_ranges(10, &ranges));  // End boundary
/// assert!(line_in_ranges(25, &ranges));
/// assert!(!line_in_ranges(0, &ranges));  // Before first range
/// assert!(!line_in_ranges(15, &ranges)); // Between ranges
/// assert!(!line_in_ranges(35, &ranges)); // After last range
/// ```
#[must_use]
pub fn line_in_ranges(line: u32, ranges: &[LineRange]) -> bool {
    ranges.iter().any(|r| r.contains(line))
}

/// Find the indices of all ranges that contain a given line number.
///
/// Returns a vector of indices into the original slice.
///
/// # Example
///
/// ```
/// use lintdiff_span_intersect::find_containing_ranges;
/// use lintdiff_line_range::LineRange;
///
/// let ranges = vec![
///     LineRange::new(1, 10),
///     LineRange::new(5, 15),  // Overlaps with first
///     LineRange::new(20, 30),
/// ];
///
/// let containing = find_containing_ranges(7, &ranges);
/// assert_eq!(containing, vec![0, 1]);
///
/// let none = find_containing_ranges(17, &ranges);
/// assert!(none.is_empty());
/// ```
#[must_use]
pub fn find_containing_ranges(line: u32, ranges: &[LineRange]) -> Vec<usize> {
    ranges
        .iter()
        .enumerate()
        .filter_map(|(i, r)| if r.contains(line) { Some(i) } else { None })
        .collect()
}

/// Find the first range that contains a given line number.
///
/// Returns `Some(index)` if found, where index is the position in the slice.
/// Returns `None` if no range contains the line.
///
/// # Example
///
/// ```
/// use lintdiff_span_intersect::find_first_containing;
/// use lintdiff_line_range::LineRange;
///
/// let ranges = vec![
///     LineRange::new(1, 10),
///     LineRange::new(5, 15),
/// ];
///
/// assert_eq!(find_first_containing(7, &ranges), Some(0));
/// assert_eq!(find_first_containing(12, &ranges), Some(1));
/// assert_eq!(find_first_containing(20, &ranges), None);
/// ```
#[must_use]
pub fn find_first_containing(line: u32, ranges: &[LineRange]) -> Option<usize> {
    ranges.iter().position(|r| r.contains(line))
}

/// Count how many ranges contain a given line number.
///
/// # Example
///
/// ```
/// use lintdiff_span_intersect::count_containing;
/// use lintdiff_line_range::LineRange;
///
/// let ranges = vec![
///     LineRange::new(1, 10),
///     LineRange::new(5, 15),
///     LineRange::new(20, 30),
/// ];
///
/// assert_eq!(count_containing(7, &ranges), 2);  // In first two
/// assert_eq!(count_containing(25, &ranges), 1); // Only in third
/// assert_eq!(count_containing(17, &ranges), 0); // In none
/// ```
#[must_use]
pub fn count_containing(line: u32, ranges: &[LineRange]) -> usize {
    ranges.iter().filter(|r| r.contains(line)).count()
}

/// Check if a range intersects with any range in a slice.
///
/// # Example
///
/// ```
/// use lintdiff_span_intersect::range_intersects_any;
/// use lintdiff_line_range::LineRange;
///
/// let ranges = vec![
///     LineRange::new(1, 10),
///     LineRange::new(20, 30),
/// ];
///
/// let a = LineRange::new(5, 15);
/// let b = LineRange::new(12, 18);
///
/// assert!(range_intersects_any(&a, &ranges));
/// assert!(!range_intersects_any(&b, &ranges));
/// ```
#[must_use]
pub fn range_intersects_any(range: &LineRange, ranges: &[LineRange]) -> bool {
    ranges.iter().any(|r| ranges_intersect(range, r))
}

/// Find all ranges that intersect with a given range.
///
/// Returns a vector of indices into the original slice.
///
/// # Example
///
/// ```
/// use lintdiff_span_intersect::find_intersecting_ranges;
/// use lintdiff_line_range::LineRange;
///
/// let ranges = vec![
///     LineRange::new(1, 10),
///     LineRange::new(5, 15),
///     LineRange::new(20, 30),
/// ];
///
/// let query = LineRange::new(8, 25);
/// let intersecting = find_intersecting_ranges(&query, &ranges);
/// assert_eq!(intersecting, vec![0, 1, 2]);
/// ```
#[must_use]
pub fn find_intersecting_ranges(range: &LineRange, ranges: &[LineRange]) -> Vec<usize> {
    ranges
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            if ranges_intersect(range, r) {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

/// Compute the union of all intersecting ranges.
///
/// Given a slice of ranges, this function merges all overlapping or adjacent
/// ranges into a minimal set of non-overlapping ranges.
///
/// # Example
///
/// ```
/// use lintdiff_span_intersect::merge_intersecting_ranges;
/// use lintdiff_line_range::LineRange;
///
/// let ranges = vec![
///     LineRange::new(1, 10),
///     LineRange::new(5, 15),  // Overlaps with first
///     LineRange::new(20, 25),
///     LineRange::new(26, 30), // Adjacent to previous
/// ];
///
/// let merged = merge_intersecting_ranges(&ranges);
/// assert_eq!(merged.len(), 2);
/// assert_eq!(merged[0], LineRange::new(1, 15));
/// assert_eq!(merged[1], LineRange::new(20, 30));
/// ```
#[must_use]
pub fn merge_intersecting_ranges(ranges: &[LineRange]) -> Vec<LineRange> {
    if ranges.is_empty() {
        return vec![];
    }

    // Sort by start position
    let mut sorted: Vec<LineRange> = ranges.to_vec();
    sorted.sort_by_key(|r| r.start);

    let mut merged: Vec<LineRange> = vec![];
    let mut current = sorted[0];

    for range in sorted.iter().skip(1) {
        // Check if current overlaps or is adjacent to range
        if current.end + 1 >= range.start {
            // Merge: extend current to cover both
            current = LineRange::new(current.start, max(current.end, range.end));
        } else {
            // No overlap: push current and start new
            merged.push(current);
            current = *range;
        }
    }
    merged.push(current);

    merged
}

/// Calculate the total number of lines covered by a set of ranges.
///
/// This accounts for overlaps, so lines are only counted once.
///
/// # Example
///
/// ```
/// use lintdiff_span_intersect::total_covered_lines;
/// use lintdiff_line_range::LineRange;
///
/// let ranges = vec![
///     LineRange::new(1, 10),  // 10 lines
///     LineRange::new(5, 15),  // 5 new lines (11-15)
///     LineRange::new(20, 25), // 6 lines
/// ];
///
/// // Total: 10 + 5 + 6 = 21 unique lines
/// assert_eq!(total_covered_lines(&ranges), 21);
/// ```
#[must_use]
pub fn total_covered_lines(ranges: &[LineRange]) -> u64 {
    let merged = merge_intersecting_ranges(ranges);
    merged.iter().map(|r| u64::from(r.end - r.start + 1)).sum()
}

/// Check if a set of ranges is sorted and non-overlapping.
///
/// # Example
///
/// ```
/// use lintdiff_span_intersect::is_sorted_and_disjoint;
/// use lintdiff_line_range::LineRange;
///
/// let sorted = vec![
///     LineRange::new(1, 10),
///     LineRange::new(20, 30),
/// ];
/// assert!(is_sorted_and_disjoint(&sorted));
///
/// let overlapping = vec![
///     LineRange::new(1, 10),
///     LineRange::new(5, 15),
/// ];
/// assert!(!is_sorted_and_disjoint(&overlapping));
///
/// let unsorted = vec![
///     LineRange::new(20, 30),
///     LineRange::new(1, 10),
/// ];
/// assert!(!is_sorted_and_disjoint(&unsorted));
/// ```
#[must_use]
pub fn is_sorted_and_disjoint(ranges: &[LineRange]) -> bool {
    for i in 1..ranges.len() {
        // Check sorted by start
        if ranges[i].start < ranges[i - 1].start {
            return false;
        }
        // Check non-overlapping (end of previous < start of current)
        if ranges[i].start <= ranges[i - 1].end {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ranges_intersect_overlapping() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);
        assert!(ranges_intersect(&a, &b));
        assert!(ranges_intersect(&b, &a)); // Commutative
    }

    #[test]
    fn test_ranges_intersect_disjoint() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(20, 30);
        assert!(!ranges_intersect(&a, &b));
        assert!(!ranges_intersect(&b, &a));
    }

    #[test]
    fn test_ranges_intersect_adjacent() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(11, 20);
        assert!(!ranges_intersect(&a, &b)); // Adjacent but not overlapping
    }

    #[test]
    fn test_ranges_intersect_contained() {
        let a = LineRange::new(1, 20);
        let b = LineRange::new(5, 10);
        assert!(ranges_intersect(&a, &b));
    }

    #[test]
    fn test_ranges_intersect_same() {
        let a = LineRange::new(1, 10);
        assert!(ranges_intersect(&a, &a));
    }

    #[test]
    fn test_range_intersection_overlapping() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);
        let result = range_intersection(&a, &b).unwrap();
        assert_eq!(result.start, 5);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn test_range_intersection_disjoint() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(20, 30);
        assert!(range_intersection(&a, &b).is_none());
    }

    #[test]
    fn test_range_intersection_contained() {
        let a = LineRange::new(1, 20);
        let b = LineRange::new(5, 10);
        let result = range_intersection(&a, &b).unwrap();
        assert_eq!(result.start, 5);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn test_range_intersection_same() {
        let a = LineRange::new(1, 10);
        let result = range_intersection(&a, &a).unwrap();
        assert_eq!(result.start, 1);
        assert_eq!(result.end, 10);
    }

    #[test]
    fn test_line_in_ranges_found() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        assert!(line_in_ranges(5, &ranges));
        assert!(line_in_ranges(25, &ranges));
    }

    #[test]
    fn test_line_in_ranges_not_found() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        assert!(!line_in_ranges(15, &ranges));
        assert!(!line_in_ranges(0, &ranges));
        assert!(!line_in_ranges(35, &ranges));
    }

    #[test]
    fn test_line_in_ranges_empty() {
        let ranges: Vec<LineRange> = vec![];
        assert!(!line_in_ranges(5, &ranges));
    }

    #[test]
    fn test_line_in_ranges_boundary() {
        let ranges = vec![LineRange::new(5, 10)];
        assert!(line_in_ranges(5, &ranges)); // Start
        assert!(line_in_ranges(10, &ranges)); // End
        assert!(!line_in_ranges(4, &ranges)); // Before
        assert!(!line_in_ranges(11, &ranges)); // After
    }

    #[test]
    fn test_find_containing_ranges_multiple() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(5, 15),
            LineRange::new(20, 30),
        ];
        let result = find_containing_ranges(7, &ranges);
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn test_find_containing_ranges_single() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        let result = find_containing_ranges(5, &ranges);
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn test_find_containing_ranges_none() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        let result = find_containing_ranges(15, &ranges);
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_first_containing_found() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(5, 15)];
        assert_eq!(find_first_containing(7, &ranges), Some(0));
        assert_eq!(find_first_containing(12, &ranges), Some(1));
    }

    #[test]
    fn test_find_first_containing_none() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        assert_eq!(find_first_containing(15, &ranges), None);
    }

    #[test]
    fn test_count_containing() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(5, 15),
            LineRange::new(20, 30),
        ];
        assert_eq!(count_containing(7, &ranges), 2);
        assert_eq!(count_containing(25, &ranges), 1);
        assert_eq!(count_containing(17, &ranges), 0);
    }

    #[test]
    fn test_range_intersects_any() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        assert!(range_intersects_any(&LineRange::new(5, 15), &ranges));
        assert!(!range_intersects_any(&LineRange::new(12, 18), &ranges));
    }

    #[test]
    fn test_find_intersecting_ranges() {
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
    fn test_merge_intersecting_ranges() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(5, 15),
            LineRange::new(20, 25),
            LineRange::new(26, 30),
        ];
        let merged = merge_intersecting_ranges(&ranges);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], LineRange::new(1, 15));
        assert_eq!(merged[1], LineRange::new(20, 30));
    }

    #[test]
    fn test_merge_intersecting_ranges_empty() {
        let ranges: Vec<LineRange> = vec![];
        let merged = merge_intersecting_ranges(&ranges);
        assert!(merged.is_empty());
    }

    #[test]
    fn test_merge_intersecting_ranges_single() {
        let ranges = vec![LineRange::new(1, 10)];
        let merged = merge_intersecting_ranges(&ranges);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], LineRange::new(1, 10));
    }

    #[test]
    fn test_total_covered_lines() {
        let ranges = vec![
            LineRange::new(1, 10),  // 10 lines
            LineRange::new(5, 15),  // 5 new lines
            LineRange::new(20, 25), // 6 lines
        ];
        assert_eq!(total_covered_lines(&ranges), 21);
    }

    #[test]
    fn test_total_covered_lines_no_overlap() {
        let ranges = vec![
            LineRange::new(1, 10),  // 10 lines
            LineRange::new(20, 30), // 11 lines
        ];
        assert_eq!(total_covered_lines(&ranges), 21);
    }

    #[test]
    fn test_is_sorted_and_disjoint_true() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(20, 30)];
        assert!(is_sorted_and_disjoint(&ranges));
    }

    #[test]
    fn test_is_sorted_and_disjoint_overlapping() {
        let ranges = vec![LineRange::new(1, 10), LineRange::new(5, 15)];
        assert!(!is_sorted_and_disjoint(&ranges));
    }

    #[test]
    fn test_is_sorted_and_disjoint_unsorted() {
        let ranges = vec![LineRange::new(20, 30), LineRange::new(1, 10)];
        assert!(!is_sorted_and_disjoint(&ranges));
    }
}
