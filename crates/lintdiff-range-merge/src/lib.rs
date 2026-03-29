//! Line number to range merging utilities for lintdiff.
//!
//! This microcrate provides utilities for merging adjacent line numbers into
//! compact ranges, and for performing operations on line ranges.
//!
//! # Example: Merging Line Numbers
//!
//! ```
//! use lintdiff_range_merge::{LineRange, merge_lines};
//!
//! let lines = vec![1, 2, 3, 5, 7, 8, 9];
//! let ranges = merge_lines(&lines);
//! assert_eq!(ranges.len(), 3);
//! assert_eq!(ranges[0], LineRange::new(1, 3));
//! assert_eq!(ranges[1], LineRange::new(5, 5));
//! assert_eq!(ranges[2], LineRange::new(7, 9));
//! ```
//!
//! # Example: Merging Overlapping Ranges
//!
//! ```
//! use lintdiff_range_merge::{LineRange, merge_overlapping};
//!
//! let ranges = vec![
//!     LineRange::new(1, 5),
//!     LineRange::new(3, 7),
//!     LineRange::new(10, 15),
//! ];
//! let merged = merge_overlapping(&ranges);
//! assert_eq!(merged.len(), 2);
//! assert_eq!(merged[0], LineRange::new(1, 7));
//! assert_eq!(merged[1], LineRange::new(10, 15));
//! ```
//!
//! # Example: Range Operations
//!
//! ```
//! use lintdiff_range_merge::{LineRange, range_contains, ranges_intersect, range_union, is_adjacent};
//!
//! let a = LineRange::new(1, 10);
//! let b = LineRange::new(11, 20);
//!
//! assert!(range_contains(&a, 5));
//! assert!(!ranges_intersect(&a, &b));
//! assert!(is_adjacent(&a, &b));
//! let union = range_union(&a, &b).unwrap();
//! assert_eq!(union, LineRange::new(1, 20));
//! ```

#![warn(missing_docs)]

use std::cmp::{max, min, Ordering};

/// A range of lines (inclusive).
///
/// Both start and end are inclusive, so a range from 1 to 3
/// includes lines 1, 2, and 3.
///
/// # Invariants
/// - `start <= end` (enforced by constructors)
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::LineRange;
///
/// let range = LineRange::new(1, 5);
/// assert_eq!(range.start, 1);
/// assert_eq!(range.end, 5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LineRange {
    /// Start line (inclusive).
    pub start: usize,
    /// End line (inclusive).
    pub end: usize,
}

impl LineRange {
    /// Create a new line range.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `end < start`.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_range_merge::LineRange;
    ///
    /// let range = LineRange::new(1, 10);
    /// assert_eq!(range.start, 1);
    /// assert_eq!(range.end, 10);
    /// ```
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        debug_assert!(end >= start, "End must be >= start");
        Self { start, end }
    }

    /// Create a range representing a single line.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_range_merge::LineRange;
    ///
    /// let range = LineRange::single(5);
    /// assert_eq!(range.start, 5);
    /// assert_eq!(range.end, 5);
    /// ```
    #[must_use]
    pub const fn single(line: usize) -> Self {
        Self {
            start: line,
            end: line,
        }
    }

    /// Create a range without validation.
    ///
    /// This is useful for const contexts where validation is not possible.
    /// The caller must ensure `start <= end`.
    #[must_use]
    pub const fn new_unchecked(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Check if this range contains a line.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_range_merge::LineRange;
    ///
    /// let range = LineRange::new(1, 10);
    /// assert!(range.contains(5));
    /// assert!(!range.contains(15));
    /// ```
    #[must_use]
    pub const fn contains(&self, line: usize) -> bool {
        line >= self.start && line <= self.end
    }

    /// Get the number of lines in this range.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_range_merge::LineRange;
    ///
    /// let range = LineRange::new(1, 10);
    /// assert_eq!(range.len(), 10);
    /// ```
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end - self.start + 1
    }

    /// Check if this range is empty (contains no lines).
    ///
    /// This should always return `false` for valid ranges where `start <= end`.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.end < self.start
    }

    /// Check if this range intersects with another.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_range_merge::LineRange;
    ///
    /// let a = LineRange::new(1, 10);
    /// let b = LineRange::new(5, 15);
    /// assert!(a.intersects(&b));
    /// ```
    #[must_use]
    pub const fn intersects(&self, other: &Self) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    /// Check if this range is adjacent to another.
    ///
    /// Two ranges are adjacent if `a.end + 1 == b.start` or `b.end + 1 == a.start`.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_range_merge::LineRange;
    ///
    /// let a = LineRange::new(1, 10);
    /// let b = LineRange::new(11, 20);
    /// assert!(a.is_adjacent_to(&b));
    /// ```
    #[must_use]
    pub const fn is_adjacent_to(&self, other: &Self) -> bool {
        self.end + 1 == other.start || other.end + 1 == self.start
    }

    /// Check if this range overlaps or is adjacent to another.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_range_merge::LineRange;
    ///
    /// let a = LineRange::new(1, 10);
    /// let b = LineRange::new(10, 20);  // Touching at line 10
    /// assert!(a.overlaps_or_adjacent(&b));
    /// ```
    #[must_use]
    pub const fn overlaps_or_adjacent(&self, other: &Self) -> bool {
        self.intersects(other) || self.is_adjacent_to(other)
    }

    /// Get the union of this range with another.
    ///
    /// Returns `None` if the ranges don't intersect or aren't adjacent.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_range_merge::LineRange;
    ///
    /// let a = LineRange::new(1, 10);
    /// let b = LineRange::new(5, 15);
    /// let union = a.union(&b).unwrap();
    /// assert_eq!(union, LineRange::new(1, 15));
    /// ```
    #[must_use]
    pub fn union(&self, other: &Self) -> Option<Self> {
        if !self.overlaps_or_adjacent(other) {
            return None;
        }
        Some(Self::new(min(self.start, other.start), max(self.end, other.end)))
    }

    /// Get the intersection of this range with another.
    ///
    /// Returns `None` if the ranges don't intersect.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_range_merge::LineRange;
    ///
    /// let a = LineRange::new(1, 10);
    /// let b = LineRange::new(5, 15);
    /// let intersection = a.intersection(&b).unwrap();
    /// assert_eq!(intersection, LineRange::new(5, 10));
    /// ```
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }
        Some(Self::new(max(self.start, other.start), min(self.end, other.end)))
    }
}

impl Default for LineRange {
    fn default() -> Self {
        Self::single(1)
    }
}

impl std::fmt::Display for LineRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

impl From<(usize, usize)> for LineRange {
    fn from((start, end): (usize, usize)) -> Self {
        Self::new(start, end)
    }
}

impl From<usize> for LineRange {
    fn from(line: usize) -> Self {
        Self::single(line)
    }
}

// =============================================================================
// Free functions for functional style usage
// =============================================================================

/// Merge adjacent line numbers into compact ranges.
///
/// Takes a sorted list of line numbers and merges consecutive/adjacent
/// lines into ranges.
///
/// # Note
///
/// The input should be sorted. If not sorted, the function will still work
/// but may produce more ranges than necessary.
///
/// # Panics
///
/// This function does not panic. It safely handles empty input.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, merge_lines};
///
/// let lines = vec![1, 2, 3, 5, 7, 8, 9];
/// let ranges = merge_lines(&lines);
/// assert_eq!(ranges, vec![
///     LineRange::new(1, 3),
///     LineRange::new(5, 5),
///     LineRange::new(7, 9),
/// ]);
/// ```
///
/// # Edge Cases
///
/// ```
/// use lintdiff_range_merge::merge_lines;
///
/// // Empty input
/// assert!(merge_lines(&[]).is_empty());
///
/// // Single line
/// let ranges = merge_lines(&[5]);
/// assert_eq!(ranges.len(), 1);
/// assert_eq!(ranges[0].start, 5);
/// assert_eq!(ranges[0].end, 5);
/// ```
#[must_use]
pub fn merge_lines(lines: &[usize]) -> Vec<LineRange> {
    if lines.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    
    // Get the first line to start - safe because we checked is_empty above
    let (first, rest) = lines.split_first().unwrap_or((&0, &[]));
    let mut current_start = *first;
    let mut current_end = *first;

    // Process remaining lines
    for &line in rest {
        // Check if this line continues the current range
        if line == current_end || line == current_end + 1 {
            // Continue the current range
        } else {
            // Save the current range and start a new one
            result.push(LineRange::new(current_start, current_end));
            current_start = line;
        }
        current_end = line;
    }

    // Don't forget the last range
    result.push(LineRange::new(current_start, current_end));
    
    result
}

/// Merge overlapping or adjacent ranges.
///
/// Takes a slice of ranges that may overlap or be adjacent and merges them
/// into non-overlapping ranges.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, merge_overlapping};
///
/// let ranges = vec![
///     LineRange::new(1, 5),
///     LineRange::new(3, 7),
///     LineRange::new(10, 15),
/// ];
/// let merged = merge_overlapping(&ranges);
/// assert_eq!(merged, vec![
///     LineRange::new(1, 7),
///     LineRange::new(10, 15),
/// ]);
/// ```
///
/// # Adjacent Ranges
///
/// ```
/// use lintdiff_range_merge::{LineRange, merge_overlapping};
///
/// // Adjacent ranges are merged
/// let ranges = vec![
///     LineRange::new(1, 10),
///     LineRange::new(11, 20),
/// ];
/// let merged = merge_overlapping(&ranges);
/// assert_eq!(merged, vec![LineRange::new(1, 20)]);
/// ```
#[must_use]
pub fn merge_overlapping(ranges: &[LineRange]) -> Vec<LineRange> {
    if ranges.is_empty() {
        return Vec::new();
    }

    // Sort ranges by start position
    let mut sorted: Vec<_> = ranges.to_vec();
    sorted.sort_by_key(|r| r.start);

    let mut result = Vec::new();
    let mut current = sorted[0];

    for next in sorted.iter().skip(1) {
        if current.overlaps_or_adjacent(next) {
            // Merge the ranges
            current = LineRange::new(
                min(current.start, next.start),
                max(current.end, next.end),
            );
        } else {
            result.push(current);
            current = *next;
        }
    }

    result.push(current);
    result
}

/// Check if a line is contained within a range.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, range_contains};
///
/// let range = LineRange::new(1, 10);
/// assert!(range_contains(&range, 5));
/// assert!(!range_contains(&range, 15));
/// ```
#[must_use]
pub const fn range_contains(range: &LineRange, line: usize) -> bool {
    range.contains(line)
}

/// Check if two ranges intersect (share at least one line).
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, ranges_intersect};
///
/// let a = LineRange::new(1, 10);
/// let b = LineRange::new(5, 15);
/// assert!(ranges_intersect(&a, &b));
///
/// let c = LineRange::new(11, 20);
/// assert!(!ranges_intersect(&a, &c));
/// ```
#[must_use]
pub const fn ranges_intersect(a: &LineRange, b: &LineRange) -> bool {
    a.intersects(b)
}

/// Get the number of lines in a range.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, range_len};
///
/// let range = LineRange::new(1, 10);
/// assert_eq!(range_len(&range), 10);
///
/// let single = LineRange::single(5);
/// assert_eq!(range_len(&single), 1);
/// ```
#[must_use]
pub const fn range_len(range: &LineRange) -> usize {
    range.len()
}

/// Get the union of two ranges.
///
/// Returns `None` if the ranges don't intersect and aren't adjacent.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, range_union};
///
/// let a = LineRange::new(1, 10);
/// let b = LineRange::new(5, 15);
/// let union = range_union(&a, &b).unwrap();
/// assert_eq!(union, LineRange::new(1, 15));
///
/// // Non-overlapping, non-adjacent ranges
/// let c = LineRange::new(20, 30);
/// assert!(range_union(&a, &c).is_none());
/// ```
#[must_use]
pub fn range_union(a: &LineRange, b: &LineRange) -> Option<LineRange> {
    a.union(b)
}

/// Check if two ranges are adjacent.
///
/// Two ranges are adjacent if `a.end + 1 == b.start` or `b.end + 1 == a.start`.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, is_adjacent};
///
/// let a = LineRange::new(1, 10);
/// let b = LineRange::new(11, 20);
/// assert!(is_adjacent(&a, &b));
///
/// let c = LineRange::new(12, 20);
/// assert!(!is_adjacent(&a, &c));
/// ```
#[must_use]
pub const fn is_adjacent(a: &LineRange, b: &LineRange) -> bool {
    a.is_adjacent_to(b)
}

/// Compare two ranges by their start position.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, compare_by_start};
/// use std::cmp::Ordering;
///
/// let a = LineRange::new(1, 10);
/// let b = LineRange::new(5, 15);
/// assert_eq!(compare_by_start(&a, &b), Ordering::Less);
/// ```
#[must_use]
pub fn compare_by_start(a: &LineRange, b: &LineRange) -> Ordering {
    a.start.cmp(&b.start)
}

/// Compare two ranges by their end position.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, compare_by_end};
/// use std::cmp::Ordering;
///
/// let a = LineRange::new(1, 10);
/// let b = LineRange::new(5, 8);
/// assert_eq!(compare_by_end(&a, &b), Ordering::Greater);
/// ```
#[must_use]
pub fn compare_by_end(a: &LineRange, b: &LineRange) -> Ordering {
    a.end.cmp(&b.end)
}

/// Get the intersection of two ranges.
///
/// Returns `None` if the ranges don't intersect.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, range_intersection};
///
/// let a = LineRange::new(1, 10);
/// let b = LineRange::new(5, 15);
/// let intersection = range_intersection(&a, &b).unwrap();
/// assert_eq!(intersection, LineRange::new(5, 10));
///
/// // Non-overlapping ranges
/// let c = LineRange::new(20, 30);
/// assert!(range_intersection(&a, &c).is_none());
/// ```
#[must_use]
pub fn range_intersection(a: &LineRange, b: &LineRange) -> Option<LineRange> {
    a.intersection(b)
}

/// Check if two ranges overlap or are adjacent.
///
/// This is useful for determining if two ranges can be merged.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, overlaps_or_adjacent};
///
/// let a = LineRange::new(1, 10);
/// let b = LineRange::new(11, 20);
/// assert!(overlaps_or_adjacent(&a, &b));
///
/// let c = LineRange::new(12, 20);
/// assert!(!overlaps_or_adjacent(&a, &c));
/// ```
#[must_use]
pub const fn overlaps_or_adjacent(a: &LineRange, b: &LineRange) -> bool {
    a.overlaps_or_adjacent(b)
}

/// Expand a range to include a line.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, expand_to_include};
///
/// let range = LineRange::new(5, 10);
/// let expanded = expand_to_include(&range, 3);
/// assert_eq!(expanded, LineRange::new(3, 10));
///
/// let expanded = expand_to_include(&range, 15);
/// assert_eq!(expanded, LineRange::new(5, 15));
/// ```
#[must_use]
pub fn expand_to_include(range: &LineRange, line: usize) -> LineRange {
    LineRange::new(
        min(range.start, line),
        max(range.end, line),
    )
}

/// Check if one range fully contains another.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, range_contains_range};
///
/// let outer = LineRange::new(1, 20);
/// let inner = LineRange::new(5, 10);
/// assert!(range_contains_range(&outer, &inner));
///
/// let partial = LineRange::new(15, 25);
/// assert!(!range_contains_range(&outer, &partial));
/// ```
#[must_use]
pub const fn range_contains_range(outer: &LineRange, inner: &LineRange) -> bool {
    outer.start <= inner.start && outer.end >= inner.end
}

/// Calculate the gap between two non-overlapping ranges.
///
/// Returns `None` if the ranges overlap or are adjacent.
/// Returns the number of lines between the ranges.
///
/// # Example
///
/// ```
/// use lintdiff_range_merge::{LineRange, gap_between};
///
/// let a = LineRange::new(1, 10);
/// let b = LineRange::new(15, 20);
/// assert_eq!(gap_between(&a, &b), Some(4));
///
/// let c = LineRange::new(11, 20);  // Adjacent to a
/// assert_eq!(gap_between(&a, &c), None);
/// ```
#[must_use]
pub const fn gap_between(a: &LineRange, b: &LineRange) -> Option<usize> {
    if a.overlaps_or_adjacent(b) {
        return None;
    }
    
    if a.end < b.start {
        Some(b.start - a.end - 1)
    } else {
        Some(a.start - b.end - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_range_new_creates_valid_range() {
        let range = LineRange::new(1, 10);
        assert_eq!(range.start, 1);
        assert_eq!(range.end, 10);
    }

    #[test]
    fn line_range_single_creates_single_line_range() {
        let range = LineRange::single(5);
        assert_eq!(range.start, 5);
        assert_eq!(range.end, 5);
    }

    #[test]
    fn line_range_len_returns_correct_count() {
        assert_eq!(LineRange::new(1, 10).len(), 10);
        assert_eq!(LineRange::new(5, 5).len(), 1);
        assert_eq!(LineRange::new(1, 100).len(), 100);
    }

    #[test]
    fn line_range_contains_works_correctly() {
        let range = LineRange::new(5, 10);
        assert!(!range.contains(4));
        assert!(range.contains(5));
        assert!(range.contains(7));
        assert!(range.contains(10));
        assert!(!range.contains(11));
    }

    #[test]
    fn merge_lines_empty_input_returns_empty() {
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
    fn merge_lines_consecutive_lines_merge() {
        let result = merge_lines(&[1, 2, 3, 4, 5]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], LineRange::new(1, 5));
    }

    #[test]
    fn merge_lines_with_gaps_creates_multiple_ranges() {
        let result = merge_lines(&[1, 2, 3, 5, 7, 8, 9]);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], LineRange::new(1, 3));
        assert_eq!(result[1], LineRange::new(5, 5));
        assert_eq!(result[2], LineRange::new(7, 9));
    }

    #[test]
    fn merge_overlapping_empty_input_returns_empty() {
        let result = merge_overlapping(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn merge_overlapping_overlapping_ranges_merge() {
        let ranges = vec![
            LineRange::new(1, 5),
            LineRange::new(3, 7),
        ];
        let result = merge_overlapping(&ranges);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], LineRange::new(1, 7));
    }

    #[test]
    fn merge_overlapping_adjacent_ranges_merge() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(11, 20),
        ];
        let result = merge_overlapping(&ranges);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], LineRange::new(1, 20));
    }
}
