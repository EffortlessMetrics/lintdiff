//! Diff hunk header parsing for lintdiff.
//!
//! This crate provides parsing and formatting for unified diff hunk headers.
//! A hunk header has the format `@@ -a,b +c,d @@` where:
//! - `a` is the starting line in the old file
//! - `b` is the number of lines in the old file (optional, defaults to 1)
//! - `c` is the starting line in the new file
//! - `d` is the number of lines in the new file (optional, defaults to 1)
//!
//! # Example
//!
//! ```
//! use lintdiff_hunk_header::{HunkHeader, parse_hunk_header};
//!
//! let header = parse_hunk_header("@@ -1,4 +1,5 @@").unwrap().unwrap();
//! assert_eq!(header.old_start(), 1);
//! assert_eq!(header.old_count(), 4);
//! assert_eq!(header.new_start(), 1);
//! assert_eq!(header.new_count(), 5);
//! ```

use std::fmt;

use thiserror::Error;

/// Error type for hunk header parsing failures.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HunkHeaderError {
    /// The input is not a hunk header line (missing `@@` prefix).
    #[error("not a hunk header: missing '@@' prefix")]
    NotAHunkHeader,
    /// Missing the old file range (the `-` segment).
    #[error("missing old file range: expected '-a,b' segment")]
    MissingOldRange,
    /// Missing the new file range (the `+` segment).
    #[error("missing new file range: expected '+c,d' segment")]
    MissingNewRange,
    /// Invalid old start line number.
    #[error("invalid old start: {0}")]
    InvalidOldStart(String),
    /// Invalid old count.
    #[error("invalid old count: {0}")]
    InvalidOldCount(String),
    /// Invalid new start line number.
    #[error("invalid new start: {0}")]
    InvalidNewStart(String),
    /// Invalid new count.
    #[error("invalid new count: {0}")]
    InvalidNewCount(String),
    /// The hunk header format is malformed.
    #[error("malformed hunk header: {0}")]
    Malformed(String),
}

/// A parsed hunk header from a unified diff.
///
/// Represents the line range information for both old and new versions
/// of a file in a diff hunk.
///
/// # Examples
///
/// ```
/// use lintdiff_hunk_header::HunkHeader;
///
/// // Parse a standard hunk header
/// let header = HunkHeader::parse("@@ -1,4 +1,5 @@").unwrap().unwrap();
/// assert_eq!(header.old_start(), 1);
/// assert_eq!(header.old_count(), 4);
/// assert_eq!(header.new_start(), 1);
/// assert_eq!(header.new_count(), 5);
///
/// // Parse a hunk header without counts (implies count of 1)
/// let header = HunkHeader::parse("@@ -1 +1 @@").unwrap().unwrap();
/// assert_eq!(header.old_count(), 1);
/// assert_eq!(header.new_count(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HunkHeader {
    /// Starting line in old file (1-based).
    old_start: usize,
    /// Number of lines in old file.
    old_count: usize,
    /// Starting line in new file (1-based).
    new_start: usize,
    /// Number of lines in new file.
    new_count: usize,
}

impl HunkHeader {
    /// Create a new hunk header with the specified values.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(1, 4, 1, 5);
    /// assert_eq!(header.old_start(), 1);
    /// assert_eq!(header.old_count(), 4);
    /// assert_eq!(header.new_start(), 1);
    /// assert_eq!(header.new_count(), 5);
    /// ```
    #[must_use]
    pub const fn new(
        old_start: usize,
        old_count: usize,
        new_start: usize,
        new_count: usize,
    ) -> Self {
        Self {
            old_start,
            old_count,
            new_start,
            new_count,
        }
    }

    /// Parse a hunk header line.
    ///
    /// Returns `Ok(None)` if the line doesn't look like a hunk header
    /// (i.e., doesn't start with `@@`).
    ///
    /// Returns `Ok(Some(header))` if parsing succeeds.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the line looks like a hunk header but is malformed:
    /// - [`HunkHeaderError::MissingOldRange`] if the `-` segment is missing
    /// - [`HunkHeaderError::MissingNewRange`] if the `+` segment is missing
    /// - [`HunkHeaderError::InvalidOldStart`] if the old start is not a valid number
    /// - [`HunkHeaderError::InvalidOldCount`] if the old count is not a valid number
    /// - [`HunkHeaderError::InvalidNewStart`] if the new start is not a valid number
    /// - [`HunkHeaderError::InvalidNewCount`] if the new count is not a valid number
    /// - [`HunkHeaderError::Malformed`] if the header structure is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// // Standard format
    /// let header = HunkHeader::parse("@@ -1,4 +1,5 @@").unwrap().unwrap();
    /// assert_eq!(header.old_start(), 1);
    ///
    /// // Without counts (implies 1)
    /// let header = HunkHeader::parse("@@ -1 +1 @@").unwrap().unwrap();
    /// assert_eq!(header.old_count(), 1);
    ///
    /// // Not a hunk header
    /// assert!(HunkHeader::parse("not a header").unwrap().is_none());
    /// ```
    pub fn parse(s: &str) -> Result<Option<Self>, HunkHeaderError> {
        parse_hunk_header(s)
    }

    /// Get the starting line in the old file (1-based).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(10, 5, 15, 7);
    /// assert_eq!(header.old_start(), 10);
    /// ```
    #[must_use]
    pub const fn old_start(&self) -> usize {
        self.old_start
    }

    /// Get the number of lines in the old file.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(1, 4, 1, 5);
    /// assert_eq!(header.old_count(), 4);
    /// ```
    #[must_use]
    pub const fn old_count(&self) -> usize {
        self.old_count
    }

    /// Get the starting line in the new file (1-based).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(10, 5, 15, 7);
    /// assert_eq!(header.new_start(), 15);
    /// ```
    #[must_use]
    pub const fn new_start(&self) -> usize {
        self.new_start
    }

    /// Get the number of lines in the new file.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(1, 4, 1, 5);
    /// assert_eq!(header.new_count(), 5);
    /// ```
    #[must_use]
    pub const fn new_count(&self) -> usize {
        self.new_count
    }

    /// Check if the hunk is empty (zero lines on both sides).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let empty = HunkHeader::new(0, 0, 0, 0);
    /// assert!(empty.is_empty());
    ///
    /// let non_empty = HunkHeader::new(1, 1, 1, 1);
    /// assert!(!non_empty.is_empty());
    /// ```
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.old_count == 0 && self.new_count == 0
    }

    /// Get the total line count (old + new).
    ///
    /// This is useful for estimating the size of a hunk.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(1, 4, 1, 5);
    /// assert_eq!(header.line_count(), 9);
    /// ```
    #[must_use]
    pub const fn line_count(&self) -> usize {
        self.old_count + self.new_count
    }

    /// Get the ending line in the old file (exclusive).
    ///
    /// This is `old_start + old_count`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(10, 5, 15, 7);
    /// assert_eq!(header.old_end(), 15);
    /// ```
    #[must_use]
    pub const fn old_end(&self) -> usize {
        self.old_start + self.old_count
    }

    /// Get the ending line in the new file (exclusive).
    ///
    /// This is `new_start + new_count`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(10, 5, 15, 7);
    /// assert_eq!(header.new_end(), 22);
    /// ```
    #[must_use]
    pub const fn new_end(&self) -> usize {
        self.new_start + self.new_count
    }

    /// Check if a line number is within the old file range.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(10, 5, 15, 7);
    /// assert!(header.contains_old_line(10));
    /// assert!(header.contains_old_line(14));
    /// assert!(!header.contains_old_line(9));
    /// assert!(!header.contains_old_line(15));
    /// ```
    #[must_use]
    pub const fn contains_old_line(&self, line: usize) -> bool {
        line >= self.old_start && line < self.old_end()
    }

    /// Check if a line number is within the new file range.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(10, 5, 15, 7);
    /// assert!(header.contains_new_line(15));
    /// assert!(header.contains_new_line(21));
    /// assert!(!header.contains_new_line(14));
    /// assert!(!header.contains_new_line(22));
    /// ```
    #[must_use]
    pub const fn contains_new_line(&self, line: usize) -> bool {
        line >= self.new_start && line < self.new_end()
    }

    /// Create a hunk header with a different old start.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(1, 4, 1, 5);
    /// let shifted = header.with_old_start(10);
    /// assert_eq!(shifted.old_start(), 10);
    /// assert_eq!(shifted.old_count(), 4);
    /// ```
    #[must_use]
    pub const fn with_old_start(&self, old_start: usize) -> Self {
        Self {
            old_start,
            old_count: self.old_count,
            new_start: self.new_start,
            new_count: self.new_count,
        }
    }

    /// Create a hunk header with a different old count.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(1, 4, 1, 5);
    /// let modified = header.with_old_count(10);
    /// assert_eq!(modified.old_count(), 10);
    /// ```
    #[must_use]
    pub const fn with_old_count(&self, old_count: usize) -> Self {
        Self {
            old_start: self.old_start,
            old_count,
            new_start: self.new_start,
            new_count: self.new_count,
        }
    }

    /// Create a hunk header with a different new start.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(1, 4, 1, 5);
    /// let shifted = header.with_new_start(10);
    /// assert_eq!(shifted.new_start(), 10);
    /// ```
    #[must_use]
    pub const fn with_new_start(&self, new_start: usize) -> Self {
        Self {
            old_start: self.old_start,
            old_count: self.old_count,
            new_start,
            new_count: self.new_count,
        }
    }

    /// Create a hunk header with a different new count.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_hunk_header::HunkHeader;
    ///
    /// let header = HunkHeader::new(1, 4, 1, 5);
    /// let modified = header.with_new_count(10);
    /// assert_eq!(modified.new_count(), 10);
    /// ```
    #[must_use]
    pub const fn with_new_count(&self, new_count: usize) -> Self {
        Self {
            old_start: self.old_start,
            old_count: self.old_count,
            new_start: self.new_start,
            new_count,
        }
    }
}

impl fmt::Display for HunkHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "@@ -{},{} +{},{} @@",
            self.old_start, self.old_count, self.new_start, self.new_count
        )
    }
}

impl Default for HunkHeader {
    fn default() -> Self {
        Self::new(1, 0, 1, 0)
    }
}

/// Parse a hunk header line.
///
/// This is a convenience function that delegates to [`HunkHeader::parse`].
///
/// Returns `Ok(None)` if the line doesn't look like a hunk header.
/// Returns `Ok(Some(header))` if parsing succeeds.
///
/// # Errors
///
/// Returns `Err` if the line looks like a hunk header but is malformed:
/// - [`HunkHeaderError::MissingOldRange`] if the `-` segment is missing
/// - [`HunkHeaderError::MissingNewRange`] if the `+` segment is missing
/// - [`HunkHeaderError::InvalidOldStart`] if the old start is not a valid number
/// - [`HunkHeaderError::InvalidOldCount`] if the old count is not a valid number
/// - [`HunkHeaderError::InvalidNewStart`] if the new start is not a valid number
/// - [`HunkHeaderError::InvalidNewCount`] if the new count is not a valid number
/// - [`HunkHeaderError::Malformed`] if the header structure is invalid
///
/// # Examples
///
/// ```
/// use lintdiff_hunk_header::parse_hunk_header;
///
/// let header = parse_hunk_header("@@ -1,4 +1,5 @@").unwrap().unwrap();
/// assert_eq!(header.old_start(), 1);
/// assert_eq!(header.old_count(), 4);
/// ```
pub fn parse_hunk_header(s: &str) -> Result<Option<HunkHeader>, HunkHeaderError> {
    let line = s.trim();

    // Check if this looks like a hunk header
    if !line.starts_with("@@") {
        return Ok(None);
    }

    // Find the - and + segments
    let minus_pos = line.find('-').ok_or(HunkHeaderError::MissingOldRange)?;
    let plus_pos = line.find('+').ok_or(HunkHeaderError::MissingNewRange)?;

    // Ensure + comes after - (valid position check)
    if plus_pos <= minus_pos {
        return Err(HunkHeaderError::Malformed(
            "'+' segment must come after '-' segment".to_string(),
        ));
    }

    // Extract the minus segment
    let after_minus = &line[minus_pos + 1..];
    let minus_seg = after_minus
        .split_whitespace()
        .next()
        .ok_or(HunkHeaderError::MissingOldRange)?;

    // Extract the plus segment
    let after_plus = &line[plus_pos + 1..];
    let plus_seg = after_plus
        .split_whitespace()
        .next()
        .ok_or(HunkHeaderError::MissingNewRange)?;

    // Parse old range (start,count or just start)
    let (old_start, old_count) = parse_range(minus_seg, true)?;
    let (new_start, new_count) = parse_range(plus_seg, false)?;

    Ok(Some(HunkHeader::new(
        old_start, old_count, new_start, new_count,
    )))
}

/// Parse a range segment like "1,4" or "1".
///
/// Returns (start, count). If count is omitted, defaults to 1.
/// When `is_old` is true and start is 0, treats it as 0 (special case for empty files).
fn parse_range(s: &str, is_old: bool) -> Result<(usize, usize), HunkHeaderError> {
    let parts: Vec<&str> = s.split(',').collect();

    let start = parts
        .first()
        .ok_or_else(|| HunkHeaderError::Malformed("empty range".to_string()))?
        .parse::<usize>()
        .map_err(|e| {
            if is_old {
                HunkHeaderError::InvalidOldStart(e.to_string())
            } else {
                HunkHeaderError::InvalidNewStart(e.to_string())
            }
        })?;

    let count = if parts.len() > 1 {
        parts[1].parse::<usize>().map_err(|e| {
            if is_old {
                HunkHeaderError::InvalidOldCount(e.to_string())
            } else {
                HunkHeaderError::InvalidNewCount(e.to_string())
            }
        })?
    } else {
        // When count is omitted, it defaults to 1
        // But if start is 0, count should be 0 too (empty file case)
        usize::from(start != 0)
    };

    Ok((start, count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_header() {
        let header = HunkHeader::new(1, 4, 1, 5);
        assert_eq!(header.old_start(), 1);
        assert_eq!(header.old_count(), 4);
        assert_eq!(header.new_start(), 1);
        assert_eq!(header.new_count(), 5);
    }

    #[test]
    fn test_default() {
        let header = HunkHeader::default();
        assert_eq!(header.old_start(), 1);
        assert_eq!(header.old_count(), 0);
        assert_eq!(header.new_start(), 1);
        assert_eq!(header.new_count(), 0);
    }

    #[test]
    fn test_is_empty() {
        assert!(HunkHeader::new(0, 0, 0, 0).is_empty());
        assert!(!HunkHeader::new(1, 1, 0, 0).is_empty());
        assert!(!HunkHeader::new(0, 0, 1, 1).is_empty());
    }

    #[test]
    fn test_line_count() {
        assert_eq!(HunkHeader::new(1, 4, 1, 5).line_count(), 9);
        assert_eq!(HunkHeader::new(1, 0, 1, 0).line_count(), 0);
    }

    #[test]
    fn test_display() {
        let header = HunkHeader::new(1, 4, 1, 5);
        assert_eq!(format!("{header}"), "@@ -1,4 +1,5 @@");
    }

    #[test]
    fn test_parse_standard() {
        let header = parse_hunk_header("@@ -1,4 +1,5 @@").unwrap().unwrap();
        assert_eq!(header.old_start(), 1);
        assert_eq!(header.old_count(), 4);
        assert_eq!(header.new_start(), 1);
        assert_eq!(header.new_count(), 5);
    }

    #[test]
    fn test_parse_without_counts() {
        let header = parse_hunk_header("@@ -1 +1 @@").unwrap().unwrap();
        assert_eq!(header.old_start(), 1);
        assert_eq!(header.old_count(), 1);
        assert_eq!(header.new_start(), 1);
        assert_eq!(header.new_count(), 1);
    }

    #[test]
    fn test_parse_not_a_header() {
        assert!(parse_hunk_header("not a header").unwrap().is_none());
    }

    #[test]
    fn test_parse_empty_string() {
        assert!(parse_hunk_header("").unwrap().is_none());
    }
}
