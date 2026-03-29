//! File location representation for lintdiff.
//!
//! A location represents a position in a specific file,
//! combining a file path with optional line/column information.

use std::fmt;
use std::path::{Path, PathBuf};

/// A location in a source file.
///
/// # Examples
/// ```
/// use lintdiff_location::Location;
///
/// let loc = Location::new("src/lib.rs", 42, 10);
/// assert_eq!(loc.path(), "src/lib.rs");
/// assert_eq!(loc.line_number(), Some(42));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Location {
    /// File path (normalized).
    path: String,
    /// Line number (1-based, optional).
    line: Option<u32>,
    /// Column number (1-based, optional).
    column: Option<u32>,
}

impl Location {
    /// Create a new location with path only.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::file("src/lib.rs");
    /// assert_eq!(loc.path(), "src/lib.rs");
    /// assert!(loc.line_number().is_none());
    /// assert!(loc.column().is_none());
    /// ```
    #[must_use]
    pub fn file(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            line: None,
            column: None,
        }
    }

    /// Create a new location with path and line.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::line("src/lib.rs", 42);
    /// assert_eq!(loc.path(), "src/lib.rs");
    /// assert_eq!(loc.line_number(), Some(42));
    /// assert!(loc.column().is_none());
    /// ```
    #[must_use]
    pub fn line(path: impl Into<String>, line: u32) -> Self {
        Self {
            path: path.into(),
            line: Some(line),
            column: None,
        }
    }

    /// Create a new location with path, line, and column.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::new("src/lib.rs", 42, 10);
    /// assert_eq!(loc.path(), "src/lib.rs");
    /// assert_eq!(loc.line_number(), Some(42));
    /// assert_eq!(loc.column(), Some(10));
    /// ```
    #[must_use]
    pub fn new(path: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            path: path.into(),
            line: Some(line),
            column: Some(column),
        }
    }

    /// Create a location from components.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::from_parts("src/lib.rs", Some(42), Some(10));
    /// assert_eq!(loc.path(), "src/lib.rs");
    /// assert_eq!(loc.line_number(), Some(42));
    /// assert_eq!(loc.column(), Some(10));
    /// ```
    #[must_use]
    pub fn from_parts(path: impl Into<String>, line: Option<u32>, column: Option<u32>) -> Self {
        Self {
            path: path.into(),
            line,
            column,
        }
    }

    /// Get the file path.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::file("src/lib.rs");
    /// assert_eq!(loc.path(), "src/lib.rs");
    /// ```
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the line number (if present).
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::line("src/lib.rs", 42);
    /// assert_eq!(loc.line_number(), Some(42));
    /// ```
    #[must_use]
    pub const fn line_number(&self) -> Option<u32> {
        self.line
    }

    /// Get the column number (if present).
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::new("src/lib.rs", 42, 10);
    /// assert_eq!(loc.column(), Some(10));
    /// ```
    #[must_use]
    pub const fn column(&self) -> Option<u32> {
        self.column
    }

    /// Check if this location has line information.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::line("src/lib.rs", 42);
    /// assert!(loc.has_line());
    /// ```
    #[must_use]
    pub const fn has_line(&self) -> bool {
        self.line.is_some()
    }

    /// Check if this location has column information.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::new("src/lib.rs", 42, 10);
    /// assert!(loc.has_column());
    /// ```
    #[must_use]
    pub const fn has_column(&self) -> bool {
        self.column.is_some()
    }

    /// Check if this is a file-only location (no line/column).
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::file("src/lib.rs");
    /// assert!(loc.is_file_only());
    /// ```
    #[must_use]
    pub const fn is_file_only(&self) -> bool {
        self.line.is_none()
    }

    /// Set the line number.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::file("src/lib.rs").with_line(42);
    /// assert_eq!(loc.line_number(), Some(42));
    /// ```
    #[must_use]
    pub const fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    /// Set the column number.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::line("src/lib.rs", 42).with_column(10);
    /// assert_eq!(loc.column(), Some(10));
    /// ```
    #[must_use]
    pub const fn with_column(mut self, column: u32) -> Self {
        self.column = Some(column);
        self
    }

    /// Convert to a Path reference.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    /// use std::path::Path;
    ///
    /// let loc = Location::file("src/lib.rs");
    /// assert_eq!(loc.as_path(), Path::new("src/lib.rs"));
    /// ```
    #[must_use]
    pub fn as_path(&self) -> &Path {
        Path::new(&self.path)
    }

    /// Get the file extension (if any).
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::file("src/lib.rs");
    /// assert_eq!(loc.extension(), Some("rs"));
    /// ```
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        Path::new(&self.path).extension().and_then(|s| s.to_str())
    }

    /// Get the file name (if any).
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::file("src/lib.rs");
    /// assert_eq!(loc.file_name(), Some("lib.rs"));
    /// ```
    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        Path::new(&self.path).file_name().and_then(|s| s.to_str())
    }

    /// Get the parent directory (if any).
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::file("src/lib.rs");
    /// assert_eq!(loc.parent(), Some("src"));
    /// ```
    #[must_use]
    pub fn parent(&self) -> Option<&str> {
        let path = Path::new(&self.path);
        path.parent().and_then(|p| p.to_str())
    }

    /// Check if this location matches a given path pattern.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::file("src/lib.rs");
    /// assert!(loc.matches_path("lib.rs"));
    /// assert!(loc.matches_path("src/lib.rs"));
    /// ```
    #[must_use]
    pub fn matches_path(&self, pattern: &str) -> bool {
        // Simple suffix matching for now
        self.path.ends_with(pattern) || self.path == pattern
    }

    /// Create a location with a different path.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::Location;
    ///
    /// let loc = Location::line("src/lib.rs", 42);
    /// let new_loc = loc.with_path("src/main.rs");
    /// assert_eq!(new_loc.path(), "src/main.rs");
    /// assert_eq!(new_loc.line_number(), Some(42));
    /// ```
    #[must_use]
    pub fn with_path(&self, path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            line: self.line,
            column: self.column,
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)?;
        if let Some(line) = self.line {
            write!(f, ":{line}")?;
            if let Some(column) = self.column {
                write!(f, ":{column}")?;
            }
        }
        Ok(())
    }
}

impl From<&str> for Location {
    fn from(path: &str) -> Self {
        Self::file(path)
    }
}

impl From<String> for Location {
    fn from(path: String) -> Self {
        Self::file(path)
    }
}

impl From<PathBuf> for Location {
    fn from(path: PathBuf) -> Self {
        Self::file(path.to_string_lossy().to_string())
    }
}

/// A range of locations in a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LocationRange {
    /// File path.
    path: String,
    /// Start line (1-based).
    start_line: u32,
    /// Start column (1-based, optional).
    start_column: Option<u32>,
    /// End line (1-based).
    end_line: u32,
    /// End column (1-based, optional).
    end_column: Option<u32>,
}

impl LocationRange {
    /// Create a new location range.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::LocationRange;
    ///
    /// let range = LocationRange::new("src/lib.rs", 10, 20);
    /// assert_eq!(range.path(), "src/lib.rs");
    /// assert_eq!(range.start_line(), 10);
    /// assert_eq!(range.end_line(), 20);
    /// ```
    #[must_use]
    pub fn new(path: impl Into<String>, start_line: u32, end_line: u32) -> Self {
        Self {
            path: path.into(),
            start_line,
            start_column: None,
            end_line,
            end_column: None,
        }
    }

    /// Create a range with full position info.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::LocationRange;
    ///
    /// let range = LocationRange::with_columns("src/lib.rs", 10, 5, 20, 15);
    /// assert_eq!(range.start_column(), Some(5));
    /// assert_eq!(range.end_column(), Some(15));
    /// ```
    #[must_use]
    pub fn with_columns(
        path: impl Into<String>,
        start_line: u32,
        start_column: u32,
        end_line: u32,
        end_column: u32,
    ) -> Self {
        Self {
            path: path.into(),
            start_line,
            start_column: Some(start_column),
            end_line,
            end_column: Some(end_column),
        }
    }

    /// Get the file path.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::LocationRange;
    ///
    /// let range = LocationRange::new("src/lib.rs", 10, 20);
    /// assert_eq!(range.path(), "src/lib.rs");
    /// ```
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the start line.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::LocationRange;
    ///
    /// let range = LocationRange::new("src/lib.rs", 10, 20);
    /// assert_eq!(range.start_line(), 10);
    /// ```
    #[must_use]
    pub const fn start_line(&self) -> u32 {
        self.start_line
    }

    /// Get the start column (if present).
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::LocationRange;
    ///
    /// let range = LocationRange::with_columns("src/lib.rs", 10, 5, 20, 15);
    /// assert_eq!(range.start_column(), Some(5));
    /// ```
    #[must_use]
    pub const fn start_column(&self) -> Option<u32> {
        self.start_column
    }

    /// Get the end line.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::LocationRange;
    ///
    /// let range = LocationRange::new("src/lib.rs", 10, 20);
    /// assert_eq!(range.end_line(), 20);
    /// ```
    #[must_use]
    pub const fn end_line(&self) -> u32 {
        self.end_line
    }

    /// Get the end column (if present).
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::LocationRange;
    ///
    /// let range = LocationRange::with_columns("src/lib.rs", 10, 5, 20, 15);
    /// assert_eq!(range.end_column(), Some(15));
    /// ```
    #[must_use]
    pub const fn end_column(&self) -> Option<u32> {
        self.end_column
    }

    /// Check if this is a single-line range.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::LocationRange;
    ///
    /// let range = LocationRange::new("src/lib.rs", 10, 10);
    /// assert!(range.is_single_line());
    /// ```
    #[must_use]
    pub const fn is_single_line(&self) -> bool {
        self.start_line == self.end_line
    }

    /// Get the line count.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::LocationRange;
    ///
    /// let range = LocationRange::new("src/lib.rs", 10, 20);
    /// assert_eq!(range.line_count(), 11);
    /// ```
    #[must_use]
    pub const fn line_count(&self) -> u32 {
        self.end_line - self.start_line + 1
    }

    /// Check if a line is within this range.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::LocationRange;
    ///
    /// let range = LocationRange::new("src/lib.rs", 10, 20);
    /// assert!(range.contains_line(15));
    /// assert!(!range.contains_line(5));
    /// ```
    #[must_use]
    pub const fn contains_line(&self, line: u32) -> bool {
        line >= self.start_line && line <= self.end_line
    }

    /// Get the start location.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::LocationRange;
    ///
    /// let range = LocationRange::with_columns("src/lib.rs", 10, 5, 20, 15);
    /// let start = range.start();
    /// assert_eq!(start.line_number(), Some(10));
    /// assert_eq!(start.column(), Some(5));
    /// ```
    #[must_use]
    pub fn start(&self) -> Location {
        Location::from_parts(self.path.clone(), Some(self.start_line), self.start_column)
    }

    /// Get the end location.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_location::LocationRange;
    ///
    /// let range = LocationRange::with_columns("src/lib.rs", 10, 5, 20, 15);
    /// let end = range.end();
    /// assert_eq!(end.line_number(), Some(20));
    /// assert_eq!(end.column(), Some(15));
    /// ```
    #[must_use]
    pub fn end(&self) -> Location {
        Location::from_parts(self.path.clone(), Some(self.end_line), self.end_column)
    }
}

impl fmt::Display for LocationRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}", self.path, self.start_line, self.end_line)?;
        match (self.start_column, self.end_column) {
            (Some(sc), Some(ec)) => write!(f, ":{sc}-{ec}"),
            _ => Ok(()),
        }
    }
}

/// Parse a location from a string.
///
/// Supported formats:
/// - `"path/to/file.rs"` - file only
/// - `"path/to/file.rs:42"` - file and line
/// - `"path/to/file.rs:42:10"` - file, line, and column
///
/// # Errors
/// Returns `LocationParseError::InvalidLine` if the line number is not a valid u32.
/// Returns `LocationParseError::InvalidColumn` if the column number is not a valid u32.
///
/// # Examples
/// ```
/// use lintdiff_location::{Location, parse_location};
///
/// let loc = parse_location("src/lib.rs:42:10").unwrap();
/// assert_eq!(loc.path(), "src/lib.rs");
/// assert_eq!(loc.line_number(), Some(42));
/// assert_eq!(loc.column(), Some(10));
/// ```
pub fn parse_location(s: &str) -> Result<Location, LocationParseError> {
    let parts: Vec<&str> = s.rsplitn(3, ':').collect();

    match parts.len() {
        1 => Ok(Location::file(parts[0])),
        2 => {
            let line: u32 = parts[0]
                .parse()
                .map_err(|_| LocationParseError::InvalidLine(parts[0].to_string()))?;
            Ok(Location::line(parts[1], line))
        }
        3 => {
            let column: u32 = parts[0]
                .parse()
                .map_err(|_| LocationParseError::InvalidColumn(parts[0].to_string()))?;
            let line: u32 = parts[1]
                .parse()
                .map_err(|_| LocationParseError::InvalidLine(parts[1].to_string()))?;
            Ok(Location::new(parts[2], line, column))
        }
        _ => unreachable!(),
    }
}

/// Error when parsing a location string.
#[derive(Debug, Clone, thiserror::Error)]
pub enum LocationParseError {
    /// Invalid line number.
    #[error("Invalid line number: '{0}'")]
    InvalidLine(String),
    /// Invalid column number.
    #[error("Invalid column number: '{0}'")]
    InvalidColumn(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_creation() {
        let loc = Location::file("src/lib.rs");
        assert_eq!(loc.path(), "src/lib.rs");
        assert!(loc.line_number().is_none());
        assert!(loc.column().is_none());
    }

    #[test]
    fn test_line_creation() {
        let loc = Location::line("src/lib.rs", 42);
        assert_eq!(loc.path(), "src/lib.rs");
        assert_eq!(loc.line_number(), Some(42));
        assert!(loc.column().is_none());
    }

    #[test]
    fn test_new_creation() {
        let loc = Location::new("src/lib.rs", 42, 10);
        assert_eq!(loc.path(), "src/lib.rs");
        assert_eq!(loc.line_number(), Some(42));
        assert_eq!(loc.column(), Some(10));
    }

    #[test]
    fn test_display_file_only() {
        let loc = Location::file("src/lib.rs");
        assert_eq!(format!("{}", loc), "src/lib.rs");
    }

    #[test]
    fn test_display_with_line() {
        let loc = Location::line("src/lib.rs", 42);
        assert_eq!(format!("{}", loc), "src/lib.rs:42");
    }

    #[test]
    fn test_display_with_column() {
        let loc = Location::new("src/lib.rs", 42, 10);
        assert_eq!(format!("{}", loc), "src/lib.rs:42:10");
    }

    #[test]
    fn test_parse_file_only() {
        let loc = parse_location("src/lib.rs").unwrap();
        assert_eq!(loc.path(), "src/lib.rs");
        assert!(loc.line_number().is_none());
    }

    #[test]
    fn test_parse_with_line() {
        let loc = parse_location("src/lib.rs:42").unwrap();
        assert_eq!(loc.path(), "src/lib.rs");
        assert_eq!(loc.line_number(), Some(42));
    }

    #[test]
    fn test_parse_with_column() {
        let loc = parse_location("src/lib.rs:42:10").unwrap();
        assert_eq!(loc.path(), "src/lib.rs");
        assert_eq!(loc.line_number(), Some(42));
        assert_eq!(loc.column(), Some(10));
    }

    #[test]
    fn test_parse_invalid_line() {
        let result = parse_location("src/lib.rs:abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_column() {
        let result = parse_location("src/lib.rs:42:abc");
        assert!(result.is_err());
    }
}
