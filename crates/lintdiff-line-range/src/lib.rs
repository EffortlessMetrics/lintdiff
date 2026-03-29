//! Line range operations for lintdiff.
//!
//! This microcrate provides a single responsibility: line range operations,
//! including intersection detection, containment checks, and range merging.
//!
//! # Example: Basic Usage
//!
//! ```
//! use lintdiff_line_range::LineRange;
//!
//! let range = LineRange::new(1, 10);
//! assert!(range.contains(5));
//! assert!(!range.contains(15));
//! ```
//!
//! # Example: Range Intersection
//!
//! ```
//! use lintdiff_line_range::{LineRange, ranges_intersect, range_intersection};
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
//! # Example: Merging Ranges
//!
//! ```
//! use lintdiff_line_range::{LineRange, merge_ranges};
//!
//! let ranges = vec![
//!     LineRange::new(1, 10),
//!     LineRange::new(5, 15),
//!     LineRange::new(20, 25),
//! ];
//! let merged = merge_ranges(&ranges).unwrap();
//! assert_eq!(merged.start, 1);
//! assert_eq!(merged.end, 25);
//! ```

#![warn(missing_docs)]

use std::cmp::{max, min};

/// A range of lines (inclusive, 1-based).
///
/// Line numbers are 1-based, meaning the first line is line 1.
/// Both the start and end are inclusive, so a range from 1 to 3
/// includes lines 1, 2, and 3.
///
/// # Example
///
/// ```
/// use lintdiff_line_range::LineRange;
///
/// let range = LineRange::new(1, 5);
/// assert_eq!(range.start, 1);
/// assert_eq!(range.end, 5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct LineRange {
    /// Start line (1-based, inclusive).
    pub start: u32,
    /// End line (1-based, inclusive).
    pub end: u32,
}

impl LineRange {
    /// Create a new line range.
    ///
    /// # Panics
    ///
    /// Panics if `start` is 0 (line numbers are 1-based).
    /// Panics if `end < start` (end must be >= start).
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_line_range::LineRange;
    ///
    /// let range = LineRange::new(1, 10);
    /// assert_eq!(range.start, 1);
    /// assert_eq!(range.end, 10);
    /// ```
    #[must_use]
    pub fn new(start: u32, end: u32) -> Self {
        debug_assert!(start >= 1, "Line numbers are 1-based, start must be >= 1");
        debug_assert!(end >= start, "End must be >= start");
        Self { start, end }
    }

    /// Create a range from start and end.
    ///
    /// This is an alias for [`LineRange::new`] with validation.
    ///
    /// # Panics
    ///
    /// Panics if `start > end`.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_line_range::LineRange;
    ///
    /// let range = LineRange::from_start_end(5, 10);
    /// assert_eq!(range.start, 5);
    /// assert_eq!(range.end, 10);
    /// ```
    #[must_use]
    pub fn from_start_end(start: u32, end: u32) -> Self {
        assert!(start <= end, "Start must be <= end");
        Self::new(start, end)
    }

    /// Check if a line is within this range.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_line_range::LineRange;
    ///
    /// let range = LineRange::new(5, 10);
    /// assert!(range.contains(5));  // start is inclusive
    /// assert!(range.contains(7));
    /// assert!(range.contains(10)); // end is inclusive
    /// assert!(!range.contains(4));
    /// assert!(!range.contains(11));
    /// ```
    #[must_use]
    pub fn contains(&self, line: u32) -> bool {
        self.start <= line && line <= self.end
    }

    /// Get the number of lines in this range.
    ///
    /// Returns the count of lines from start to end (inclusive).
    /// Uses saturating arithmetic to prevent overflow.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_line_range::LineRange;
    ///
    /// let range = LineRange::new(5, 10);
    /// assert_eq!(range.len(), 6); // lines 5, 6, 7, 8, 9, 10
    ///
    /// let single = LineRange::new(1, 1);
    /// assert_eq!(single.len(), 1);
    /// ```
    #[must_use]
    pub fn len(&self) -> u32 {
        self.end.saturating_sub(self.start).saturating_add(1)
    }

    /// Check if this range is empty (has zero length).
    ///
    /// Note: A range is never truly empty since start <= end always,
    /// but this method is provided for API completeness.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_line_range::LineRange;
    ///
    /// let range = LineRange::new(1, 1);
    /// assert!(!range.is_empty());
    /// ```
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false // A valid LineRange always has at least one line
    }

    /// Check if this range overlaps with another.
    ///
    /// Two ranges overlap if they share at least one line.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_line_range::LineRange;
    ///
    /// let a = LineRange::new(1, 10);
    /// let b = LineRange::new(5, 15);
    /// let c = LineRange::new(20, 30);
    ///
    /// assert!(a.overlaps(&b));
    /// assert!(!a.overlaps(&c));
    /// ```
    #[must_use]
    pub fn overlaps(&self, other: &LineRange) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    /// Compute the intersection of two ranges.
    ///
    /// Returns `Some(LineRange)` if the ranges overlap, containing the
    /// overlapping portion. Returns `None` if they don't overlap.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_line_range::LineRange;
    ///
    /// let a = LineRange::new(1, 10);
    /// let b = LineRange::new(5, 15);
    /// let c = LineRange::new(20, 30);
    ///
    /// let intersection = a.intersection(&b).unwrap();
    /// assert_eq!(intersection.start, 5);
    /// assert_eq!(intersection.end, 10);
    ///
    /// assert!(a.intersection(&c).is_none());
    /// ```
    #[must_use]
    pub fn intersection(&self, other: &LineRange) -> Option<LineRange> {
        let overlap_start = max(self.start, other.start);
        let overlap_end = min(self.end, other.end);

        if overlap_start <= overlap_end {
            Some(LineRange::new(overlap_start, overlap_end))
        } else {
            None
        }
    }

    /// Merge this range with another, extending to cover both.
    ///
    /// Returns a new range that spans from the minimum start to the maximum end
    /// of both ranges. This is useful for combining adjacent or overlapping ranges.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_line_range::LineRange;
    ///
    /// let a = LineRange::new(1, 10);
    /// let b = LineRange::new(5, 15);
    ///
    /// let merged = a.merge(&b);
    /// assert_eq!(merged.start, 1);
    /// assert_eq!(merged.end, 15);
    /// ```
    #[must_use]
    pub fn merge(&self, other: &LineRange) -> LineRange {
        LineRange::new(min(self.start, other.start), max(self.end, other.end))
    }
}

impl Default for LineRange {
    /// Returns a default LineRange covering just line 1.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_line_range::LineRange;
    ///
    /// let range = LineRange::default();
    /// assert_eq!(range.start, 1);
    /// assert_eq!(range.end, 1);
    /// ```
    fn default() -> Self {
        Self { start: 1, end: 1 }
    }
}

/// Check if two ranges intersect (overlap).
///
/// This is a convenience function that delegates to [`LineRange::overlaps`].
///
/// # Example
///
/// ```
/// use lintdiff_line_range::{LineRange, ranges_intersect};
///
/// let a = LineRange::new(1, 10);
/// let b = LineRange::new(5, 15);
/// let c = LineRange::new(20, 30);
///
/// assert!(ranges_intersect(&a, &b));
/// assert!(!ranges_intersect(&a, &c));
/// ```
#[must_use]
pub fn ranges_intersect(a: &LineRange, b: &LineRange) -> bool {
    a.overlaps(b)
}

/// Compute the intersection of two ranges.
///
/// This is a convenience function that delegates to [`LineRange::intersection`].
///
/// # Example
///
/// ```
/// use lintdiff_line_range::{LineRange, range_intersection};
///
/// let a = LineRange::new(1, 10);
/// let b = LineRange::new(5, 15);
///
/// let intersection = range_intersection(&a, &b).unwrap();
/// assert_eq!(intersection.start, 5);
/// assert_eq!(intersection.end, 10);
/// ```
#[must_use]
pub fn range_intersection(a: &LineRange, b: &LineRange) -> Option<LineRange> {
    a.intersection(b)
}

/// Merge multiple ranges into one.
///
/// Returns a single range that spans from the minimum start to the maximum end
/// of all provided ranges. Returns `None` if the input slice is empty.
///
/// # Example
///
/// ```
/// use lintdiff_line_range::{LineRange, merge_ranges};
///
/// let ranges = vec![
///     LineRange::new(1, 10),
///     LineRange::new(5, 15),
///     LineRange::new(20, 25),
/// ];
/// let merged = merge_ranges(&ranges).unwrap();
/// assert_eq!(merged.start, 1);
/// assert_eq!(merged.end, 25);
///
/// let empty: Vec<LineRange> = vec![];
/// assert!(merge_ranges(&empty).is_none());
/// ```
#[must_use]
pub fn merge_ranges(ranges: &[LineRange]) -> Option<LineRange> {
    if ranges.is_empty() {
        return None;
    }

    let mut result = ranges[0];
    for range in &ranges[1..] {
        result = result.merge(range);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_range() {
        let range = LineRange::new(1, 10);
        assert_eq!(range.start, 1);
        assert_eq!(range.end, 10);
    }

    #[test]
    fn test_contains() {
        let range = LineRange::new(5, 10);
        assert!(!range.contains(4));
        assert!(range.contains(5));
        assert!(range.contains(7));
        assert!(range.contains(10));
        assert!(!range.contains(11));
    }

    #[test]
    fn test_len() {
        assert_eq!(LineRange::new(1, 1).len(), 1);
        assert_eq!(LineRange::new(1, 5).len(), 5);
        assert_eq!(LineRange::new(5, 10).len(), 6);
    }

    #[test]
    fn test_overlaps() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);
        let c = LineRange::new(11, 20);
        let d = LineRange::new(10, 15); // Touches at edge

        assert!(a.overlaps(&b));
        assert!(!a.overlaps(&c));
        assert!(a.overlaps(&d)); // Edge touch is overlap
    }

    #[test]
    fn test_intersection() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);
        let c = LineRange::new(20, 30);

        let intersection = a.intersection(&b).unwrap();
        assert_eq!(intersection.start, 5);
        assert_eq!(intersection.end, 10);

        assert!(a.intersection(&c).is_none());
    }

    #[test]
    fn test_merge() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);

        let merged = a.merge(&b);
        assert_eq!(merged.start, 1);
        assert_eq!(merged.end, 15);
    }

    #[test]
    fn test_merge_ranges() {
        let ranges = vec![
            LineRange::new(1, 10),
            LineRange::new(5, 15),
            LineRange::new(20, 25),
        ];
        let merged = merge_ranges(&ranges).unwrap();
        assert_eq!(merged.start, 1);
        assert_eq!(merged.end, 25);

        let empty: Vec<LineRange> = vec![];
        assert!(merge_ranges(&empty).is_none());
    }

    #[test]
    fn test_default() {
        let range = LineRange::default();
        assert_eq!(range.start, 1);
        assert_eq!(range.end, 1);
    }
}
