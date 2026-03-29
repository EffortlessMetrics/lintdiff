//! Line range merging utilities for lintdiff.
//!
//! This microcrate provides utilities for merging overlapping line ranges,
//! useful for combining diff hunks or diagnostic line ranges.
//!
//! # Example: Basic Usage
//!
//! ```
//! use lintdiff_line_merge::{merge_ranges, ranges_overlap, ranges_intersect};
//!
//! let ranges = vec![(1, 10), (5, 15), (20, 25)];
//! let merged = merge_ranges(&ranges);
//! assert_eq!(merged, vec![(1, 15), (20, 25)]);
//! ```
//!
//! # Example: Adjacent Ranges
//!
//! ```
//! use lintdiff_line_merge::{merge_ranges, ranges_overlap, ranges_intersect};
//!
//! // Adjacent ranges (end + 1 == start) count as overlapping
//! assert!(ranges_overlap((1, 10), (11, 20)));
//! // But they don't strictly intersect
//! assert!(!ranges_intersect((1, 10), (11, 20)));
//! ```
//!
//! # Example: In-place Merge
//!
//! ```
//! use lintdiff_line_merge::merge_ranges_inplace;
//!
//! let mut ranges = vec![(1, 10), (5, 15), (20, 25)];
//! merge_ranges_inplace(&mut ranges);
//! assert_eq!(ranges, vec![(1, 15), (20, 25)]);
//! ```

#![warn(missing_docs)]

use std::cmp::{max, min};

pub use lintdiff_line_range::LineRange;

/// A trait for types that represent a range of lines.
///
/// This trait allows the merge functions to work with any type that
/// has a start and end line number.
///
/// # Example
///
/// ```
/// use lintdiff_line_merge::LineRange;
///
/// fn process_range(range: &impl lintdiff_line_merge::LineRangeOps) {
///     println!("Range: {}-{}", range.start(), range.end());
/// }
/// ```
pub trait LineRangeOps {
    /// Returns the start line (inclusive, 1-based).
    fn start(&self) -> usize;
    /// Returns the end line (inclusive).
    fn end(&self) -> usize;
}

impl LineRangeOps for LineRange {
    #[inline]
    fn start(&self) -> usize {
        self.start as usize
    }

    #[inline]
    fn end(&self) -> usize {
        self.end as usize
    }
}

impl LineRangeOps for (usize, usize) {
    #[inline]
    fn start(&self) -> usize {
        self.0
    }

    #[inline]
    fn end(&self) -> usize {
        self.1
    }
}

/// Check if two ranges overlap (including adjacent ranges).
///
/// Two ranges overlap if they share at least one line, OR if they are adjacent
/// (i.e., `a.end + 1 == b.start` or `b.end + 1 == a.start`).
///
/// This is useful for merging ranges where adjacent ranges should be combined.
///
/// # Example
///
/// ```
/// use lintdiff_line_merge::ranges_overlap;
///
/// // Overlapping ranges
/// assert!(ranges_overlap((1, 10), (5, 15)));
/// assert!(ranges_overlap((5, 15), (1, 10)));
///
/// // Adjacent ranges (end + 1 == start)
/// assert!(ranges_overlap((1, 10), (11, 20)));
/// assert!(ranges_overlap((11, 20), (1, 10)));
///
/// // Non-overlapping, non-adjacent
/// assert!(!ranges_overlap((1, 10), (20, 30)));
/// ```
#[must_use]
pub const fn ranges_overlap(a: (usize, usize), b: (usize, usize)) -> bool {
    // Adjacent or overlapping: a.end + 1 >= b.start AND b.end + 1 >= a.start
    // Using saturating_add to avoid overflow
    a.1.saturating_add(1) >= b.0 && b.1.saturating_add(1) >= a.0
}

/// Check if two ranges strictly intersect.
///
/// Two ranges intersect if they share at least one line.
/// Adjacent ranges (where `a.end + 1 == b.start`) do NOT count as intersecting.
///
/// # Example
///
/// ```
/// use lintdiff_line_merge::ranges_intersect;
///
/// // Overlapping ranges
/// assert!(ranges_intersect((1, 10), (5, 15)));
/// assert!(ranges_intersect((5, 15), (1, 10)));
///
/// // Adjacent ranges do NOT intersect
/// assert!(!ranges_intersect((1, 10), (11, 20)));
///
/// // Non-overlapping, non-adjacent
/// assert!(!ranges_intersect((1, 10), (20, 30)));
/// ```
#[must_use]
pub const fn ranges_intersect(a: (usize, usize), b: (usize, usize)) -> bool {
    // Strictly intersecting: a.end >= b.start AND b.end >= a.start
    a.1 >= b.0 && b.1 >= a.0
}

/// Get the union of two overlapping or adjacent ranges.
///
/// Returns a new range that spans from the minimum start to the maximum end
/// of both ranges.
///
/// # Panics
///
/// Panics in debug mode if the ranges are not overlapping or adjacent.
///
/// # Example
///
/// ```
/// use lintdiff_line_merge::union_of_ranges;
///
/// assert_eq!(union_of_ranges((1, 10), (5, 15)), (1, 15));
/// assert_eq!(union_of_ranges((1, 10), (11, 20)), (1, 20));
/// assert_eq!(union_of_ranges((5, 15), (1, 10)), (1, 15));
/// ```
#[must_use]
pub fn union_of_ranges(a: (usize, usize), b: (usize, usize)) -> (usize, usize) {
    debug_assert!(
        ranges_overlap(a, b),
        "Ranges must overlap or be adjacent for union"
    );
    (min(a.0, b.0), max(a.1, b.1))
}

/// Get the intersection of two ranges.
///
/// Returns `Some((start, end))` if the ranges intersect, `None` otherwise.
/// Adjacent ranges do NOT count as intersecting.
///
/// # Example
///
/// ```
/// use lintdiff_line_merge::intersect_of_ranges;
///
/// assert_eq!(intersect_of_ranges((1, 10), (5, 15)), Some((5, 10)));
/// assert_eq!(intersect_of_ranges((1, 10), (11, 20)), None); // Adjacent, not intersecting
/// assert_eq!(intersect_of_ranges((1, 10), (20, 30)), None); // Non-overlapping
/// ```
#[must_use]
pub fn intersect_of_ranges(a: (usize, usize), b: (usize, usize)) -> Option<(usize, usize)> {
    if !ranges_intersect(a, b) {
        return None;
    }
    let start = max(a.0, b.0);
    let end = min(a.1, b.1);
    Some((start, end))
}

/// Merge overlapping and adjacent ranges into non-overlapping ranges.
///
/// Takes a slice of ranges and returns a new vector of merged ranges where:
/// - All overlapping ranges are combined
/// - Adjacent ranges (where end + 1 == start) are combined
/// - The result is sorted by start line
/// - No ranges in the result overlap or are adjacent
///
/// # Example
///
/// ```
/// use lintdiff_line_merge::merge_ranges;
///
/// // Overlapping ranges are merged
/// let ranges = vec![(1, 10), (5, 15)];
/// assert_eq!(merge_ranges(&ranges), vec![(1, 15)]);
///
/// // Adjacent ranges are merged
/// let ranges = vec![(1, 10), (11, 20)];
/// assert_eq!(merge_ranges(&ranges), vec![(1, 20)]);
///
/// // Non-overlapping, non-adjacent ranges stay separate
/// let ranges = vec![(1, 10), (20, 30)];
/// assert_eq!(merge_ranges(&ranges), vec![(1, 10), (20, 30)]);
///
/// // Empty input returns empty output
/// let ranges: Vec<(usize, usize)> = vec![];
/// assert_eq!(merge_ranges(&ranges), Vec::<(usize, usize)>::new());
/// ```
#[must_use]
pub fn merge_ranges<T: LineRangeOps>(ranges: &[T]) -> Vec<(usize, usize)> {
    if ranges.is_empty() {
        return Vec::new();
    }

    // Convert to tuples and sort by start
    let mut sorted: Vec<(usize, usize)> = ranges.iter().map(|r| (r.start(), r.end())).collect();
    sorted.sort_by_key(|r| r.0);

    let mut result = Vec::with_capacity(sorted.len());
    let mut current = sorted[0];

    for next in sorted.iter().skip(1) {
        if ranges_overlap(current, *next) {
            // Merge the ranges
            current = union_of_ranges(current, *next);
        } else {
            // Push current and start a new range
            result.push(current);
            current = *next;
        }
    }
    result.push(current);

    result
}

/// Merge overlapping and adjacent ranges in-place.
///
/// This is more efficient than [`merge_ranges`] for large collections
/// as it modifies the vector directly.
///
/// The resulting vector will:
/// - Have all overlapping ranges combined
/// - Have adjacent ranges combined
/// - Be sorted by start line
/// - Have no overlapping or adjacent ranges
///
/// # Example
///
/// ```
/// use lintdiff_line_merge::merge_ranges_inplace;
///
/// let mut ranges = vec![(1, 10), (5, 15), (20, 30)];
/// merge_ranges_inplace(&mut ranges);
/// assert_eq!(ranges, vec![(1, 15), (20, 30)]);
/// ```
pub fn merge_ranges_inplace(ranges: &mut Vec<(usize, usize)>) {
    if ranges.len() <= 1 {
        // Sort single element or empty for consistency
        ranges.sort_by_key(|r| r.0);
        return;
    }

    // Sort by start
    ranges.sort_by_key(|r| r.0);

    // Merge in-place
    let mut write_idx = 0;
    for read_idx in 1..ranges.len() {
        let current = ranges[write_idx];
        let next = ranges[read_idx];

        if ranges_overlap(current, next) {
            // Merge into write position
            ranges[write_idx] = union_of_ranges(current, next);
        } else {
            // Move to next write position
            write_idx += 1;
            ranges[write_idx] = next;
        }
    }

    // Truncate to merged size
    ranges.truncate(write_idx + 1);
}

/// Check if a range contains another range.
///
/// Returns `true` if `inner` is completely contained within `outer`.
///
/// # Example
///
/// ```
/// use lintdiff_line_merge::range_contains;
///
/// assert!(range_contains((1, 20), (5, 10)));
/// assert!(range_contains((1, 20), (1, 20))); // Equal ranges
/// assert!(!range_contains((5, 10), (1, 20)));
/// assert!(!range_contains((1, 10), (15, 20)));
/// ```
#[must_use]
pub const fn range_contains(outer: (usize, usize), inner: (usize, usize)) -> bool {
    outer.0 <= inner.0 && outer.1 >= inner.1
}

/// Check if a line is within a range.
///
/// # Example
///
/// ```
/// use lintdiff_line_merge::line_in_range;
///
/// assert!(line_in_range((1, 10), 5));
/// assert!(line_in_range((1, 10), 1));  // Start is inclusive
/// assert!(line_in_range((1, 10), 10)); // End is inclusive
/// assert!(!line_in_range((1, 10), 0));
/// assert!(!line_in_range((1, 10), 11));
/// ```
#[must_use]
pub const fn line_in_range(range: (usize, usize), line: usize) -> bool {
    range.0 <= line && line <= range.1
}

/// Get the length of a range (number of lines).
///
/// Returns the count of lines from start to end (inclusive).
///
/// # Example
///
/// ```
/// use lintdiff_line_merge::range_len;
///
/// assert_eq!(range_len((1, 1)), 1);
/// assert_eq!(range_len((1, 5)), 5);
/// assert_eq!(range_len((5, 10)), 6);
/// ```
#[must_use]
pub const fn range_len(range: (usize, usize)) -> usize {
    range.1.saturating_sub(range.0).saturating_add(1)
}

/// Calculate the total number of lines covered by a set of merged ranges.
///
/// This merges the ranges first, then sums the lengths.
///
/// # Example
///
/// ```
/// use lintdiff_line_merge::total_covered_lines;
///
/// let ranges = vec![(1, 10), (5, 15), (20, 25)];
/// // After merge: [(1, 15), (20, 25)]
/// // Total: 15 + 6 = 21
/// assert_eq!(total_covered_lines(&ranges), 21);
/// ```
#[must_use]
pub fn total_covered_lines<T: LineRangeOps>(ranges: &[T]) -> usize {
    let merged = merge_ranges(ranges);
    merged.iter().map(|r| range_len(*r)).sum()
}

/// Find gaps between ranges.
///
/// Returns a vector of ranges representing the gaps between the input ranges.
/// The input ranges are merged first, so overlapping/adjacent ranges don't create gaps.
///
/// # Example
///
/// ```
/// use lintdiff_line_merge::find_gaps;
///
/// let ranges = vec![(1, 10), (20, 30), (40, 50)];
/// let gaps = find_gaps(&ranges);
/// assert_eq!(gaps, vec![(11, 19), (31, 39)]);
///
/// // Overlapping ranges don't create gaps
/// let ranges = vec![(1, 10), (5, 15), (20, 30)];
/// let gaps = find_gaps(&ranges);
/// assert_eq!(gaps, vec![(16, 19)]);
/// ```
#[must_use]
pub fn find_gaps<T: LineRangeOps>(ranges: &[T]) -> Vec<(usize, usize)> {
    let merged = merge_ranges(ranges);

    if merged.len() <= 1 {
        return Vec::new();
    }

    let mut gaps = Vec::with_capacity(merged.len() - 1);
    for window in merged.windows(2) {
        let gap_start = window[0].1.saturating_add(1);
        let gap_end = window[1].0.saturating_sub(1);
        if gap_start <= gap_end {
            gaps.push((gap_start, gap_end));
        }
    }

    gaps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ranges_overlap_overlapping() {
        assert!(ranges_overlap((1, 10), (5, 15)));
        assert!(ranges_overlap((5, 15), (1, 10)));
        assert!(ranges_overlap((1, 20), (5, 10))); // Nested
        assert!(ranges_overlap((5, 10), (1, 20))); // Nested reversed
    }

    #[test]
    fn test_ranges_overlap_adjacent() {
        assert!(ranges_overlap((1, 10), (11, 20)));
        assert!(ranges_overlap((11, 20), (1, 10)));
    }

    #[test]
    fn test_ranges_overlap_non_overlapping() {
        assert!(!ranges_overlap((1, 10), (20, 30)));
        assert!(!ranges_overlap((20, 30), (1, 10)));
    }

    #[test]
    fn test_ranges_overlap_edge_cases() {
        assert!(ranges_overlap((1, 1), (1, 1))); // Same point
        assert!(ranges_overlap((1, 1), (2, 2))); // Adjacent points
        assert!(!ranges_overlap((1, 1), (3, 3))); // Non-adjacent points
    }

    #[test]
    fn test_ranges_intersect_overlapping() {
        assert!(ranges_intersect((1, 10), (5, 15)));
        assert!(ranges_intersect((5, 15), (1, 10)));
        assert!(ranges_intersect((1, 20), (5, 10))); // Nested
    }

    #[test]
    fn test_ranges_intersect_adjacent() {
        // Adjacent ranges do NOT intersect
        assert!(!ranges_intersect((1, 10), (11, 20)));
        assert!(!ranges_intersect((11, 20), (1, 10)));
    }

    #[test]
    fn test_ranges_intersect_non_overlapping() {
        assert!(!ranges_intersect((1, 10), (20, 30)));
        assert!(!ranges_intersect((20, 30), (1, 10)));
    }

    #[test]
    fn test_ranges_intersect_edge_cases() {
        assert!(ranges_intersect((1, 1), (1, 1))); // Same point
        assert!(!ranges_intersect((1, 1), (2, 2))); // Adjacent points don't intersect
        assert!(!ranges_intersect((1, 1), (3, 3))); // Non-adjacent points
    }

    #[test]
    fn test_union_of_ranges() {
        assert_eq!(union_of_ranges((1, 10), (5, 15)), (1, 15));
        assert_eq!(union_of_ranges((5, 15), (1, 10)), (1, 15));
        assert_eq!(union_of_ranges((1, 10), (11, 20)), (1, 20));
        assert_eq!(union_of_ranges((1, 20), (5, 10)), (1, 20));
    }

    #[test]
    fn test_intersect_of_ranges() {
        assert_eq!(intersect_of_ranges((1, 10), (5, 15)), Some((5, 10)));
        assert_eq!(intersect_of_ranges((5, 15), (1, 10)), Some((5, 10)));
        assert_eq!(intersect_of_ranges((1, 10), (11, 20)), None);
        assert_eq!(intersect_of_ranges((1, 10), (20, 30)), None);
        assert_eq!(intersect_of_ranges((1, 20), (5, 10)), Some((5, 10)));
    }

    #[test]
    fn test_merge_ranges_empty() {
        let ranges: Vec<(usize, usize)> = vec![];
        assert_eq!(merge_ranges(&ranges), Vec::<(usize, usize)>::new());
    }

    #[test]
    fn test_merge_ranges_single() {
        let ranges = vec![(1, 10)];
        assert_eq!(merge_ranges(&ranges), vec![(1, 10)]);
    }

    #[test]
    fn test_merge_ranges_no_overlap() {
        let ranges = vec![(1, 10), (20, 30), (40, 50)];
        assert_eq!(merge_ranges(&ranges), vec![(1, 10), (20, 30), (40, 50)]);
    }

    #[test]
    fn test_merge_ranges_overlapping() {
        let ranges = vec![(1, 10), (5, 15)];
        assert_eq!(merge_ranges(&ranges), vec![(1, 15)]);
    }

    #[test]
    fn test_merge_ranges_adjacent() {
        let ranges = vec![(1, 10), (11, 20)];
        assert_eq!(merge_ranges(&ranges), vec![(1, 20)]);
    }

    #[test]
    fn test_merge_ranges_nested() {
        let ranges = vec![(1, 20), (5, 10)];
        assert_eq!(merge_ranges(&ranges), vec![(1, 20)]);
    }

    #[test]
    fn test_merge_ranges_multiple_merges() {
        let ranges = vec![(1, 10), (5, 15), (14, 20), (30, 40)];
        assert_eq!(merge_ranges(&ranges), vec![(1, 20), (30, 40)]);
    }

    #[test]
    fn test_merge_ranges_unsorted_input() {
        let ranges = vec![(20, 30), (1, 10), (5, 15)];
        assert_eq!(merge_ranges(&ranges), vec![(1, 15), (20, 30)]);
    }

    #[test]
    fn test_merge_ranges_complex() {
        let ranges = vec![(5, 10), (1, 3), (2, 4), (6, 8), (15, 20), (18, 25)];
        assert_eq!(merge_ranges(&ranges), vec![(1, 10), (15, 25)]);
    }

    #[test]
    fn test_merge_ranges_inplace_empty() {
        let mut ranges: Vec<(usize, usize)> = vec![];
        merge_ranges_inplace(&mut ranges);
        assert_eq!(ranges, Vec::<(usize, usize)>::new());
    }

    #[test]
    fn test_merge_ranges_inplace_single() {
        let mut ranges = vec![(1, 10)];
        merge_ranges_inplace(&mut ranges);
        assert_eq!(ranges, vec![(1, 10)]);
    }

    #[test]
    fn test_merge_ranges_inplace_no_overlap() {
        let mut ranges = vec![(1, 10), (20, 30), (40, 50)];
        merge_ranges_inplace(&mut ranges);
        assert_eq!(ranges, vec![(1, 10), (20, 30), (40, 50)]);
    }

    #[test]
    fn test_merge_ranges_inplace_overlapping() {
        let mut ranges = vec![(1, 10), (5, 15)];
        merge_ranges_inplace(&mut ranges);
        assert_eq!(ranges, vec![(1, 15)]);
    }

    #[test]
    fn test_merge_ranges_inplace_adjacent() {
        let mut ranges = vec![(1, 10), (11, 20)];
        merge_ranges_inplace(&mut ranges);
        assert_eq!(ranges, vec![(1, 20)]);
    }

    #[test]
    fn test_merge_ranges_inplace_unsorted() {
        let mut ranges = vec![(20, 30), (1, 10), (5, 15)];
        merge_ranges_inplace(&mut ranges);
        assert_eq!(ranges, vec![(1, 15), (20, 30)]);
    }

    #[test]
    fn test_range_contains() {
        assert!(range_contains((1, 20), (5, 10)));
        assert!(range_contains((1, 20), (1, 20)));
        assert!(range_contains((5, 10), (5, 10)));
        assert!(!range_contains((5, 10), (1, 20)));
        assert!(!range_contains((1, 10), (15, 20)));
        assert!(!range_contains((1, 10), (5, 15)));
    }

    #[test]
    fn test_line_in_range() {
        assert!(line_in_range((1, 10), 1));
        assert!(line_in_range((1, 10), 5));
        assert!(line_in_range((1, 10), 10));
        assert!(!line_in_range((1, 10), 0));
        assert!(!line_in_range((1, 10), 11));
    }

    #[test]
    fn test_range_len() {
        assert_eq!(range_len((1, 1)), 1);
        assert_eq!(range_len((1, 5)), 5);
        assert_eq!(range_len((5, 10)), 6);
        assert_eq!(range_len((10, 5)), 1); // Invalid range, saturates
    }

    #[test]
    fn test_total_covered_lines() {
        let ranges = vec![(1, 10), (5, 15), (20, 25)];
        // Merged: [(1, 15), (20, 25)]
        // Lengths: 15 + 6 = 21
        assert_eq!(total_covered_lines(&ranges), 21);
    }

    #[test]
    fn test_total_covered_lines_empty() {
        let ranges: Vec<(usize, usize)> = vec![];
        assert_eq!(total_covered_lines(&ranges), 0);
    }

    #[test]
    fn test_total_covered_lines_single() {
        let ranges = vec![(1, 10)];
        assert_eq!(total_covered_lines(&ranges), 10);
    }

    #[test]
    fn test_find_gaps_no_gaps() {
        let ranges = vec![(1, 10), (5, 15)]; // Merged to (1, 15)
        assert_eq!(find_gaps(&ranges), Vec::<(usize, usize)>::new());
    }

    #[test]
    fn test_find_gaps_single_range() {
        let ranges = vec![(1, 10)];
        assert_eq!(find_gaps(&ranges), Vec::<(usize, usize)>::new());
    }

    #[test]
    fn test_find_gaps_multiple() {
        let ranges = vec![(1, 10), (20, 30), (40, 50)];
        assert_eq!(find_gaps(&ranges), vec![(11, 19), (31, 39)]);
    }

    #[test]
    fn test_find_gaps_adjacent() {
        let ranges = vec![(1, 10), (11, 20)]; // Merged to (1, 20)
        assert_eq!(find_gaps(&ranges), Vec::<(usize, usize)>::new());
    }

    #[test]
    fn test_line_range_ops_tuple() {
        let tuple: (usize, usize) = (5, 10);
        assert_eq!(tuple.start(), 5);
        assert_eq!(tuple.end(), 10);
    }

    #[test]
    fn test_line_range_ops_line_range() {
        let range = LineRange::new(5, 10);
        assert_eq!(range.start(), 5);
        assert_eq!(range.end(), 10);
    }

    #[test]
    fn test_merge_with_line_range_type() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(5, 15),
            LineRange::new(20, 25),
        ];
        let merged = merge_ranges(&ranges);
        assert_eq!(merged, vec![(1, 15), (20, 25)]);
    }
}
