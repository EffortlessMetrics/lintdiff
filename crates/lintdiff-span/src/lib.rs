//! Source code span representation for lintdiff.
//!
//! A span represents a contiguous region in a source file,
//! defined by start and end line/column positions.

use std::cmp::Ordering;
use std::fmt;

/// A position in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    /// Line number (1-based).
    pub line: u32,
    /// Column number (1-based, byte offset).
    pub column: u32,
}

impl Position {
    /// Create a new position.
    #[must_use]
    pub const fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }

    /// Create a position at the start of a line.
    #[must_use]
    pub const fn start_of_line(line: u32) -> Self {
        Self { line, column: 1 }
    }

    /// Create a position at line 1, column 1.
    #[must_use]
    pub const fn start() -> Self {
        Self { line: 1, column: 1 }
    }

    /// Check if this position is at the start of a line.
    #[must_use]
    pub const fn is_start_of_line(&self) -> bool {
        self.column == 1
    }

    /// Compare positions by line number only.
    #[must_use]
    pub fn cmp_by_line(&self, other: &Self) -> Ordering {
        self.line.cmp(&other.line)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::start()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A span representing a contiguous region in source code.
///
/// # Invariants
/// - `start <= end` (enforced by constructors)
/// - Line and column numbers are 1-based
///
/// # Examples
/// ```
/// use lintdiff_span::{Span, Position};
///
/// let span = Span::new(Position::new(1, 1), Position::new(5, 10));
/// assert!(span.contains_line(3));
/// assert_eq!(span.line_count(), 5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Start position (inclusive).
    pub start: Position,
    /// End position (inclusive).
    pub end: Position,
}

impl Span {
    /// Create a new span with validation.
    ///
    /// # Panics
    /// Panics if start > end.
    #[must_use]
    pub fn new(start: Position, end: Position) -> Self {
        assert!(start <= end, "Span start must be <= end");
        Self { start, end }
    }

    /// Create a span without validation.
    ///
    /// This method is intentionally not unsafe as it does not cause undefined behavior.
    /// However, violating the invariant (start > end) may cause logical errors.
    #[must_use]
    pub const fn new_unchecked(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a span for a single line.
    #[must_use]
    pub fn single_line(line: u32, start_col: u32, end_col: u32) -> Self {
        Self::new(Position::new(line, start_col), Position::new(line, end_col))
    }

    /// Create a span for an entire line.
    #[must_use]
    pub fn full_line(line: u32) -> Self {
        Self::new(
            Position::start_of_line(line),
            Position::new(line, u32::MAX), // Represents "end of line"
        )
    }

    /// Create a span for multiple full lines.
    #[must_use]
    pub fn full_lines(start_line: u32, end_line: u32) -> Self {
        Self::new(
            Position::start_of_line(start_line),
            Position::new(end_line, u32::MAX),
        )
    }

    /// Create a zero-width span at a position (cursor/insertion point).
    #[must_use]
    pub const fn point(line: u32, column: u32) -> Self {
        let pos = Position::new(line, column);
        Self {
            start: pos,
            end: pos,
        }
    }

    /// Create an empty span at the start of the file.
    #[must_use]
    pub const fn empty() -> Self {
        Self::point(1, 1)
    }

    /// Check if this span is zero-width (point).
    #[must_use]
    pub fn is_point(&self) -> bool {
        self.start == self.end
    }

    /// Check if this span covers a full line (or lines).
    #[must_use]
    pub const fn is_full_lines(&self) -> bool {
        self.start.column == 1 && self.end.column == u32::MAX
    }

    /// Get the number of lines this span covers.
    #[must_use]
    pub const fn line_count(&self) -> u32 {
        self.end.line - self.start.line + 1
    }

    /// Check if this span contains a given line number.
    #[must_use]
    pub const fn contains_line(&self, line: u32) -> bool {
        line >= self.start.line && line <= self.end.line
    }

    /// Check if this span contains a given position.
    #[must_use]
    pub fn contains_position(&self, pos: Position) -> bool {
        pos >= self.start && pos <= self.end
    }

    /// Check if this span overlaps with another.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    /// Get the intersection of two spans, if any.
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.overlaps(other) {
            return None;
        }
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        Some(Self { start, end })
    }

    /// Merge two spans (must overlap or be adjacent).
    #[must_use]
    pub fn merge(&self, other: &Self) -> Option<Self> {
        // Check if adjacent or overlapping
        if self.overlaps(other) || self.is_adjacent(other) {
            let start = self.start.min(other.start);
            let end = self.end.max(other.end);
            return Some(Self { start, end });
        }
        None
    }

    /// Check if two spans are adjacent (end of one is start of other).
    #[must_use]
    pub fn is_adjacent(&self, other: &Self) -> bool {
        self.end == other.start || other.end == self.start
    }

    /// Expand this span to include another.
    pub fn expand_to_include(&mut self, other: &Self) {
        self.start = self.start.min(other.start);
        self.end = self.end.max(other.end);
    }

    /// Get the start line number.
    #[must_use]
    pub const fn start_line(&self) -> u32 {
        self.start.line
    }

    /// Get the end line number.
    #[must_use]
    pub const fn end_line(&self) -> u32 {
        self.end.line
    }

    /// Get the start column.
    #[must_use]
    pub const fn start_column(&self) -> u32 {
        self.start.column
    }

    /// Get the end column.
    #[must_use]
    pub const fn end_column(&self) -> u32 {
        self.end.column
    }

    /// Convert to a line range (for use with lintdiff-line-range).
    #[must_use]
    pub const fn to_line_range(&self) -> (u32, u32) {
        (self.start.line, self.end.line)
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_point() {
            write!(f, "{}", self.start)
        } else if self.start.line == self.end.line {
            write!(
                f,
                "{}:{}-{}",
                self.start.line, self.start.column, self.end.column
            )
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

/// A span with an associated file path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileSpan {
    /// File path (normalized).
    pub path: String,
    /// The span within the file.
    pub span: Span,
}

impl FileSpan {
    /// Create a new file span.
    #[must_use]
    pub fn new(path: impl Into<String>, span: Span) -> Self {
        Self {
            path: path.into(),
            span,
        }
    }

    /// Create a file span for a single line.
    #[must_use]
    pub fn single_line(path: impl Into<String>, line: u32, start_col: u32, end_col: u32) -> Self {
        Self::new(path, Span::single_line(line, start_col, end_col))
    }

    /// Create a file span for a full line.
    #[must_use]
    pub fn full_line(path: impl Into<String>, line: u32) -> Self {
        Self::new(path, Span::full_line(line))
    }

    /// Create a point file span.
    #[must_use]
    pub fn point(path: impl Into<String>, line: u32, column: u32) -> Self {
        Self::new(path, Span::point(line, column))
    }

    /// Get the file path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the span.
    #[must_use]
    pub const fn span(&self) -> &Span {
        &self.span
    }
}

impl fmt::Display for FileSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.path, self.span)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use serde::{Deserialize, Serialize};

    use super::{FileSpan, Position, Span};

    impl Serialize for Position {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            #[derive(Serialize)]
            struct PositionData {
                line: u32,
                column: u32,
            }
            PositionData {
                line: self.line,
                column: self.column,
            }
            .serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Position {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct PositionData {
                line: u32,
                column: u32,
            }
            let data = PositionData::deserialize(deserializer)?;
            Ok(Self {
                line: data.line,
                column: data.column,
            })
        }
    }

    impl Serialize for Span {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            #[derive(Serialize)]
            struct SpanData {
                start: Position,
                end: Position,
            }
            SpanData {
                start: self.start,
                end: self.end,
            }
            .serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Span {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct SpanData {
                start: Position,
                end: Position,
            }
            let data = SpanData::deserialize(deserializer)?;
            Ok(Self {
                start: data.start,
                end: data.end,
            })
        }
    }

    impl Serialize for FileSpan {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            #[derive(Serialize)]
            struct FileSpanData<'a> {
                path: &'a str,
                span: Span,
            }
            FileSpanData {
                path: &self.path,
                span: self.span,
            }
            .serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for FileSpan {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct FileSpanData {
                path: String,
                span: Span,
            }
            let data = FileSpanData::deserialize(deserializer)?;
            Ok(Self {
                path: data.path,
                span: data.span,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_position_start_of_line() {
        let pos = Position::start_of_line(3);
        assert_eq!(pos.line, 3);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn test_position_start() {
        let pos = Position::start();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn test_position_is_start_of_line() {
        assert!(Position::start_of_line(5).is_start_of_line());
        assert!(!Position::new(5, 2).is_start_of_line());
    }

    #[test]
    fn test_position_cmp_by_line() {
        let pos1 = Position::new(3, 10);
        let pos2 = Position::new(5, 1);
        assert_eq!(pos1.cmp_by_line(&pos2), Ordering::Less);
        assert_eq!(pos2.cmp_by_line(&pos1), Ordering::Greater);
        assert_eq!(pos1.cmp_by_line(&pos1), Ordering::Equal);
    }

    #[test]
    fn test_position_default() {
        let pos = Position::default();
        assert_eq!(pos, Position::start());
    }

    #[test]
    fn test_position_display() {
        let pos = Position::new(5, 10);
        assert_eq!(format!("{pos}"), "5:10");
    }

    #[test]
    fn test_span_new() {
        let start = Position::new(1, 1);
        let end = Position::new(5, 10);
        let span = Span::new(start, end);
        assert_eq!(span.start, start);
        assert_eq!(span.end, end);
    }

    #[test]
    #[should_panic(expected = "Span start must be <= end")]
    fn test_span_new_panics_on_invalid() {
        let start = Position::new(5, 10);
        let end = Position::new(1, 1);
        let _ = Span::new(start, end);
    }

    #[test]
    fn test_span_single_line() {
        let span = Span::single_line(3, 5, 10);
        assert_eq!(span.start.line, 3);
        assert_eq!(span.start.column, 5);
        assert_eq!(span.end.line, 3);
        assert_eq!(span.end.column, 10);
    }

    #[test]
    fn test_span_full_line() {
        let span = Span::full_line(5);
        assert_eq!(span.start.line, 5);
        assert_eq!(span.start.column, 1);
        assert_eq!(span.end.line, 5);
        assert_eq!(span.end.column, u32::MAX);
    }

    #[test]
    fn test_span_full_lines() {
        let span = Span::full_lines(3, 7);
        assert_eq!(span.start.line, 3);
        assert_eq!(span.start.column, 1);
        assert_eq!(span.end.line, 7);
        assert_eq!(span.end.column, u32::MAX);
    }

    #[test]
    fn test_span_point() {
        let span = Span::point(5, 10);
        assert!(span.is_point());
        assert_eq!(span.start, span.end);
    }

    #[test]
    fn test_span_empty() {
        let span = Span::empty();
        assert!(span.is_point());
        assert_eq!(span.start, Position::start());
    }

    #[test]
    fn test_span_is_full_lines() {
        assert!(Span::full_line(5).is_full_lines());
        assert!(Span::full_lines(3, 7).is_full_lines());
        assert!(!Span::single_line(5, 1, 10).is_full_lines());
    }

    #[test]
    fn test_span_line_count() {
        assert_eq!(Span::single_line(5, 1, 10).line_count(), 1);
        assert_eq!(
            Span::new(Position::new(1, 1), Position::new(5, 10)).line_count(),
            5
        );
    }

    #[test]
    fn test_span_contains_line() {
        let span = Span::new(Position::new(3, 1), Position::new(7, 10));
        assert!(!span.contains_line(2));
        assert!(span.contains_line(3));
        assert!(span.contains_line(5));
        assert!(span.contains_line(7));
        assert!(!span.contains_line(8));
    }

    #[test]
    fn test_span_contains_position() {
        let span = Span::new(Position::new(3, 5), Position::new(7, 10));
        assert!(!span.contains_position(Position::new(3, 4)));
        assert!(span.contains_position(Position::new(3, 5)));
        assert!(span.contains_position(Position::new(5, 1)));
        assert!(span.contains_position(Position::new(7, 10)));
        assert!(!span.contains_position(Position::new(7, 11)));
    }

    #[test]
    fn test_span_overlaps() {
        let span1 = Span::new(Position::new(3, 1), Position::new(7, 10));
        let span2 = Span::new(Position::new(5, 1), Position::new(10, 1));
        let span3 = Span::new(Position::new(8, 1), Position::new(10, 1));
        assert!(span1.overlaps(&span2));
        assert!(span2.overlaps(&span1));
        assert!(!span1.overlaps(&span3));
        assert!(!span3.overlaps(&span1));
    }

    #[test]
    fn test_span_intersection() {
        let span1 = Span::new(Position::new(3, 1), Position::new(7, 10));
        let span2 = Span::new(Position::new(5, 1), Position::new(10, 1));
        let intersection = span1.intersection(&span2);
        assert!(intersection.is_some());
        let inter = intersection.unwrap();
        assert_eq!(inter.start, Position::new(5, 1));
        assert_eq!(inter.end, Position::new(7, 10));
    }

    #[test]
    fn test_span_intersection_no_overlap() {
        let span1 = Span::new(Position::new(3, 1), Position::new(5, 10));
        let span2 = Span::new(Position::new(7, 1), Position::new(10, 1));
        assert!(span1.intersection(&span2).is_none());
    }

    #[test]
    fn test_span_merge_overlapping() {
        let span1 = Span::new(Position::new(3, 1), Position::new(7, 10));
        let span2 = Span::new(Position::new(5, 1), Position::new(10, 1));
        let merged = span1.merge(&span2);
        assert!(merged.is_some());
        let m = merged.unwrap();
        assert_eq!(m.start, Position::new(3, 1));
        assert_eq!(m.end, Position::new(10, 1));
    }

    #[test]
    fn test_span_merge_adjacent() {
        let span1 = Span::new(Position::new(3, 1), Position::new(5, 10));
        let span2 = Span::new(Position::new(5, 10), Position::new(10, 1));
        let merged = span1.merge(&span2);
        assert!(merged.is_some());
    }

    #[test]
    fn test_span_merge_no_overlap_no_adjacent() {
        let span1 = Span::new(Position::new(3, 1), Position::new(5, 9));
        let span2 = Span::new(Position::new(5, 11), Position::new(10, 1));
        assert!(span1.merge(&span2).is_none());
    }

    #[test]
    fn test_span_is_adjacent() {
        let span1 = Span::new(Position::new(3, 1), Position::new(5, 10));
        let span2 = Span::new(Position::new(5, 10), Position::new(10, 1));
        let span3 = Span::new(Position::new(6, 1), Position::new(10, 1));
        assert!(span1.is_adjacent(&span2));
        assert!(span2.is_adjacent(&span1));
        assert!(!span1.is_adjacent(&span3));
    }

    #[test]
    fn test_span_expand_to_include() {
        let mut span1 = Span::new(Position::new(3, 1), Position::new(7, 10));
        let span2 = Span::new(Position::new(1, 1), Position::new(10, 5));
        span1.expand_to_include(&span2);
        assert_eq!(span1.start, Position::new(1, 1));
        assert_eq!(span1.end, Position::new(10, 5));
    }

    #[test]
    fn test_span_accessors() {
        let span = Span::new(Position::new(3, 5), Position::new(7, 10));
        assert_eq!(span.start_line(), 3);
        assert_eq!(span.end_line(), 7);
        assert_eq!(span.start_column(), 5);
        assert_eq!(span.end_column(), 10);
    }

    #[test]
    fn test_span_to_line_range() {
        let span = Span::new(Position::new(3, 5), Position::new(7, 10));
        assert_eq!(span.to_line_range(), (3, 7));
    }

    #[test]
    fn test_span_default() {
        let span = Span::default();
        assert_eq!(span, Span::empty());
    }

    #[test]
    fn test_span_display_point() {
        let span = Span::point(5, 10);
        assert_eq!(format!("{span}"), "5:10");
    }

    #[test]
    fn test_span_display_single_line() {
        let span = Span::single_line(5, 1, 10);
        assert_eq!(format!("{span}"), "5:1-10");
    }

    #[test]
    fn test_span_display_multi_line() {
        let span = Span::new(Position::new(3, 5), Position::new(7, 10));
        assert_eq!(format!("{span}"), "3:5-7:10");
    }

    #[test]
    fn test_file_span_new() {
        let span = Span::single_line(5, 1, 10);
        let file_span = FileSpan::new("src/main.rs", span);
        assert_eq!(file_span.path, "src/main.rs");
        assert_eq!(file_span.span, span);
    }

    #[test]
    fn test_file_span_single_line() {
        let file_span = FileSpan::single_line("src/main.rs", 5, 1, 10);
        assert_eq!(file_span.path, "src/main.rs");
        assert_eq!(file_span.span.start.line, 5);
    }

    #[test]
    fn test_file_span_full_line() {
        let file_span = FileSpan::full_line("src/main.rs", 5);
        assert!(file_span.span.is_full_lines());
    }

    #[test]
    fn test_file_span_point() {
        let file_span = FileSpan::point("src/main.rs", 5, 10);
        assert!(file_span.span.is_point());
    }

    #[test]
    fn test_file_span_accessors() {
        let span = Span::single_line(5, 1, 10);
        let file_span = FileSpan::new("src/main.rs", span);
        assert_eq!(file_span.path(), "src/main.rs");
        assert_eq!(*file_span.span(), span);
    }

    #[test]
    fn test_file_span_display() {
        let span = Span::single_line(5, 1, 10);
        let file_span = FileSpan::new("src/main.rs", span);
        assert_eq!(format!("{file_span}"), "src/main.rs:5:1-10");
    }
}
