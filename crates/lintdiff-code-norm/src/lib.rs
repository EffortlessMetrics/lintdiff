//! Code normalization utilities for lintdiff.
//!
//! This microcrate provides utilities for normalizing code snippets for
//! consistent comparison. It handles whitespace, indentation, and line endings.
//!
//! # Example: Basic Normalization
//!
//! ```
//! use lintdiff_code_norm::{normalize_code, normalize_whitespace, normalize_line_endings};
//!
//! // Apply all standard normalizations
//! let code = "  hello   world  ";
//! assert_eq!(normalize_code(code), "hello world");
//!
//! // Or use individual functions
//! assert_eq!(normalize_whitespace("hello    world"), "hello world");
//! assert_eq!(normalize_line_endings("a\r\nb\rc"), "a\nb\nc");
//! ```
//!
//! # Example: Builder Pattern
//!
//! ```
//! use lintdiff_code_norm::{CodeNormalizer, LineEnding};
//!
//! let normalizer = CodeNormalizer::new()
//!     .trim_whitespace(true)
//!     .collapse_spaces(true)
//!     .tab_width(2)
//!     .line_ending(LineEnding::Unix);
//!
//! let result = normalizer.normalize("\tfoo\t\tbar");
//! assert_eq!(result, "foo bar");
//! ```
//!
//! # Example: Indentation Normalization
//!
//! ```
//! use lintdiff_code_norm::normalize_indentation;
//!
//! let code = "    line1\n        line2\n    line3";
//! let result = normalize_indentation(code);
//! assert_eq!(result, "line1\n    line2\nline3");
//! ```

#![warn(missing_docs)]

use std::borrow::Cow;

/// Line ending configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LineEnding {
    /// Unix-style line ending (`\n`).
    #[default]
    Unix,
    /// Windows-style line ending (`\r\n`).
    Windows,
    /// Platform-specific line ending.
    Native,
}

impl LineEnding {
    /// Get the line ending string for this configuration.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Unix => "\n",
            Self::Windows => "\r\n",
            Self::Native => {
                #[cfg(target_os = "windows")]
                {
                    "\r\n"
                }
                #[cfg(not(target_os = "windows"))]
                {
                    "\n"
                }
            }
        }
    }

    /// Get the line ending bytes for this configuration.
    #[must_use]
    pub const fn as_bytes(&self) -> &'static [u8] {
        match self {
            Self::Unix => b"\n",
            Self::Windows => b"\r\n",
            Self::Native => {
                #[cfg(target_os = "windows")]
                {
                    b"\r\n"
                }
                #[cfg(not(target_os = "windows"))]
                {
                    b"\n"
                }
            }
        }
    }
}

impl std::fmt::Display for LineEnding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unix => write!(f, "Unix (LF)"),
            Self::Windows => write!(f, "Windows (CRLF)"),
            Self::Native => write!(f, "Native (platform-specific)"),
        }
    }
}

/// Normalize whitespace while preserving structure.
///
/// This function:
/// 1. Trims leading/trailing whitespace from each line
/// 2. Collapses multiple spaces to a single space
/// 3. Preserves newlines by default
///
/// # Examples
/// ```
/// use lintdiff_code_norm::normalize_whitespace;
///
/// assert_eq!(normalize_whitespace("  hello   world  "), "hello world");
/// assert_eq!(normalize_whitespace("line1\n  line2"), "line1\nline2");
/// ```
#[must_use]
pub fn normalize_whitespace(s: &str) -> String {
    normalize_whitespace_with_options(s, true, true)
}

/// Normalize whitespace with configurable options.
///
/// # Arguments
/// * `s` - The input string
/// * `trim` - Whether to trim leading/trailing whitespace
/// * `collapse` - Whether to collapse multiple spaces to single space
///
/// # Examples
/// ```
/// use lintdiff_code_norm::normalize_whitespace_with_options;
///
/// let result = normalize_whitespace_with_options("  hello   world  ", true, true);
/// assert_eq!(result, "hello world");
/// ```
#[must_use]
pub fn normalize_whitespace_with_options(s: &str, trim: bool, collapse: bool) -> String {
    if s.is_empty() {
        return String::new();
    }

    let lines: Vec<&str> = s.split('\n').collect();
    let mut result = String::with_capacity(s.len());

    for (i, line) in lines.iter().enumerate() {
        // Handle \r\n by stripping \r from each line
        let line = line.strip_suffix('\r').unwrap_or(line);

        let processed = if collapse {
            // Collapse multiple spaces/tabs to single space
            let collapsed: String = line
                .split(char::is_whitespace)
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            collapsed
        } else {
            line.to_string()
        };

        let final_line = if trim {
            processed.trim()
        } else {
            &processed
        };

        if i > 0 {
            result.push('\n');
        }
        result.push_str(final_line);
    }

    if trim {
        result.trim().to_string()
    } else {
        result
    }
}

/// Normalize indentation by detecting and removing common leading whitespace.
///
/// This function:
/// 1. Detects the minimum indentation across all non-empty lines
/// 2. Removes that common indentation from all lines
/// 3. Converts tabs to spaces (4 spaces by default)
///
/// # Examples
/// ```
/// use lintdiff_code_norm::normalize_indentation;
///
/// let code = "    line1\n        line2\n    line3";
/// let result = normalize_indentation(code);
/// assert_eq!(result, "line1\n    line2\nline3");
/// ```
#[must_use]
pub fn normalize_indentation(s: &str) -> String {
    normalize_indentation_with_tab_width(s, 4)
}

/// Normalize indentation with configurable tab width.
///
/// # Arguments
/// * `s` - The input string
/// * `tab_width` - Number of spaces to convert tabs to (default: 4)
///
/// # Examples
/// ```
/// use lintdiff_code_norm::normalize_indentation_with_tab_width;
///
/// let code = "\tline1\n\t\tline2";
/// let result = normalize_indentation_with_tab_width(code, 2);
/// assert_eq!(result, "line1\n  line2");
/// ```
#[must_use]
pub fn normalize_indentation_with_tab_width(s: &str, tab_width: usize) -> String {
    if s.is_empty() {
        return String::new();
    }

    // First, normalize line endings and convert tabs to spaces
    let normalized = normalize_line_endings(s);
    let tab_spaces = " ".repeat(tab_width);
    let converted = normalized.replace('\t', &tab_spaces);

    let lines: Vec<&str> = converted.split('\n').collect();

    // Find minimum indentation (ignoring empty lines)
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| *c == ' ').count())
        .min()
        .unwrap_or(0);

    if min_indent == 0 {
        return converted;
    }

    // Remove common indentation
    let result: Vec<&str> = lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                *line
            } else {
                &line[min_indent..]
            }
        })
        .collect();

    result.join("\n")
}

/// Normalize line endings to Unix-style (`\n`).
///
/// Converts all line ending variants (`\r\n`, `\r`, `\n`) to `\n`.
///
/// # Examples
/// ```
/// use lintdiff_code_norm::normalize_line_endings;
///
/// assert_eq!(normalize_line_endings("a\r\nb\rc"), "a\nb\nc");
/// assert_eq!(normalize_line_endings("a\nb\r\nc"), "a\nb\nc");
/// ```
#[must_use]
pub fn normalize_line_endings(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    // First replace \r\n with \n, then standalone \r with \n
    s.replace("\r\n", "\n").replace('\r', "\n")
}

/// Normalize line endings to a specific format.
///
/// # Arguments
/// * `s` - The input string
/// * `ending` - The target line ending format
///
/// # Examples
/// ```
/// use lintdiff_code_norm::{normalize_line_endings_to, LineEnding};
///
/// let result = normalize_line_endings_to("a\nb\nc", LineEnding::Windows);
/// assert_eq!(result, "a\r\nb\r\nc");
/// ```
#[must_use]
pub fn normalize_line_endings_to(s: &str, ending: LineEnding) -> String {
    if s.is_empty() {
        return String::new();
    }

    // First normalize to \n, then convert to target
    let normalized = normalize_line_endings(s);
    match ending {
        LineEnding::Unix | LineEnding::Native => normalized,
        LineEnding::Windows => normalized.replace('\n', "\r\n"),
    }
}

/// Apply all standard normalizations to code.
///
/// This is a convenience function that applies:
/// 1. Line ending normalization (`\r\n` and `\r` to `\n`)
/// 2. Indentation normalization (remove common leading whitespace)
/// 3. Whitespace normalization (trim and collapse spaces)
///
/// # Examples
/// ```
/// use lintdiff_code_norm::normalize_code;
///
/// let code = "  hello   world  \r\n    indented";
/// let result = normalize_code(code);
/// assert!(result.contains("hello world"));
/// ```
#[must_use]
pub fn normalize_code(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    // Apply normalizations in order
    let result = normalize_line_endings(s);
    let result = normalize_indentation(&result);
    normalize_whitespace(&result)
}

/// Builder for configurable code normalization.
///
/// This struct provides a fluent API for configuring normalization options.
///
/// # Examples
/// ```
/// use lintdiff_code_norm::{CodeNormalizer, LineEnding};
///
/// let normalizer = CodeNormalizer::new()
///     .trim_whitespace(true)
///     .collapse_spaces(true)
///     .tab_width(2)
///     .line_ending(LineEnding::Unix);
///
/// let result = normalizer.normalize("\tfoo\t\tbar");
/// assert_eq!(result, "foo bar");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::struct_excessive_bools)]
pub struct CodeNormalizer {
    /// Whether to trim leading/trailing whitespace.
    pub trim: bool,
    /// Whether to collapse multiple spaces to single space.
    pub collapse_spaces: bool,
    /// Tab width for conversion to spaces.
    pub tab_width: usize,
    /// Target line ending format.
    pub line_ending: LineEnding,
    /// Whether to normalize indentation (remove common prefix).
    pub normalize_indent: bool,
    /// Whether to preserve empty lines.
    pub preserve_empty_lines: bool,
}

impl Default for CodeNormalizer {
    fn default() -> Self {
        Self {
            trim: true,
            collapse_spaces: true,
            tab_width: 4,
            line_ending: LineEnding::Unix,
            normalize_indent: true,
            preserve_empty_lines: false,
        }
    }
}

impl CodeNormalizer {
    /// Create a new normalizer with default settings.
    ///
    /// Defaults:
    /// - `trim`: true
    /// - `collapse_spaces`: true
    /// - `tab_width`: 4
    /// - `line_ending`: `LineEnding::Unix`
    /// - `normalize_indent`: true
    /// - `preserve_empty_lines`: false
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_norm::CodeNormalizer;
    ///
    /// let normalizer = CodeNormalizer::new();
    /// assert!(normalizer.trim);
    /// assert!(normalizer.collapse_spaces);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a normalizer that performs no normalization.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_norm::CodeNormalizer;
    ///
    /// let normalizer = CodeNormalizer::none();
    /// assert!(!normalizer.trim);
    /// assert!(!normalizer.collapse_spaces);
    /// ```
    #[must_use]
    pub const fn none() -> Self {
        Self {
            trim: false,
            collapse_spaces: false,
            tab_width: 4,
            line_ending: LineEnding::Unix,
            normalize_indent: false,
            preserve_empty_lines: true,
        }
    }

    /// Set whether to trim leading/trailing whitespace.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_norm::CodeNormalizer;
    ///
    /// let normalizer = CodeNormalizer::new().trim_whitespace(false);
    /// assert!(!normalizer.trim);
    /// ```
    #[must_use]
    pub const fn trim_whitespace(mut self, trim: bool) -> Self {
        self.trim = trim;
        self
    }

    /// Set whether to collapse multiple spaces to single space.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_norm::CodeNormalizer;
    ///
    /// let normalizer = CodeNormalizer::new().collapse_spaces(false);
    /// assert!(!normalizer.collapse_spaces);
    /// ```
    #[must_use]
    pub const fn collapse_spaces(mut self, collapse: bool) -> Self {
        self.collapse_spaces = collapse;
        self
    }

    /// Set tab width for conversion to spaces.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_norm::CodeNormalizer;
    ///
    /// let normalizer = CodeNormalizer::new().tab_width(2);
    /// assert_eq!(normalizer.tab_width, 2);
    /// ```
    #[must_use]
    pub const fn tab_width(mut self, width: usize) -> Self {
        self.tab_width = width;
        self
    }

    /// Set the target line ending format.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_norm::{CodeNormalizer, LineEnding};
    ///
    /// let normalizer = CodeNormalizer::new().line_ending(LineEnding::Windows);
    /// assert_eq!(normalizer.line_ending, LineEnding::Windows);
    /// ```
    #[must_use]
    pub const fn line_ending(mut self, ending: LineEnding) -> Self {
        self.line_ending = ending;
        self
    }

    /// Set whether to normalize indentation.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_norm::CodeNormalizer;
    ///
    /// let normalizer = CodeNormalizer::new().normalize_indent(false);
    /// assert!(!normalizer.normalize_indent);
    /// ```
    #[must_use]
    pub const fn normalize_indent(mut self, normalize: bool) -> Self {
        self.normalize_indent = normalize;
        self
    }

    /// Set whether to preserve empty lines.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_norm::CodeNormalizer;
    ///
    /// let normalizer = CodeNormalizer::new().preserve_empty_lines(true);
    /// assert!(normalizer.preserve_empty_lines);
    /// ```
    #[must_use]
    pub const fn preserve_empty_lines(mut self, preserve: bool) -> Self {
        self.preserve_empty_lines = preserve;
        self
    }

    /// Normalize the input string according to the configured options.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_norm::{CodeNormalizer, LineEnding};
    ///
    /// let normalizer = CodeNormalizer::new()
    ///     .trim_whitespace(true)
    ///     .collapse_spaces(true);
    ///
    /// let result = normalizer.normalize("  hello   world  ");
    /// assert_eq!(result, "hello world");
    /// ```
    #[must_use]
    pub fn normalize(&self, s: &str) -> String {
        if s.is_empty() {
            return String::new();
        }

        let mut result = s.to_string();

        // Step 1: Normalize line endings to \n first
        result = normalize_line_endings(&result);

        // Step 2: Convert tabs to spaces if needed
        if self.tab_width > 0 {
            let tab_spaces = " ".repeat(self.tab_width);
            result = result.replace('\t', &tab_spaces);
        }

        // Step 3: Normalize indentation if enabled
        if self.normalize_indent {
            result = Self::normalize_indentation_internal(&result);
        }

        // Step 4: Process each line for whitespace
        result = self.process_whitespace(&result);

        // Step 5: Convert to target line ending
        result = normalize_line_endings_to(&result, self.line_ending);

        // Step 6: Final trim if enabled
        if self.trim {
            result = result.trim().to_string();
        }

        result
    }

    fn normalize_indentation_internal(s: &str) -> String {
        let lines: Vec<&str> = s.split('\n').collect();

        // Find minimum indentation (ignoring empty lines)
        let min_indent = lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.chars().take_while(|c| *c == ' ').count())
            .min()
            .unwrap_or(0);

        if min_indent == 0 {
            return s.to_string();
        }

        // Remove common indentation
        lines
            .iter()
            .map(|line| {
                if line.trim().is_empty() {
                    (*line).to_string()
                } else {
                    line.chars().skip(min_indent).collect()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn process_whitespace(&self, s: &str) -> String {
        let lines: Vec<&str> = s.split('\n').collect();
        let mut result = Vec::with_capacity(lines.len());

        for line in lines {
            let processed = if self.collapse_spaces {
                let collapsed: String = line
                    .split(char::is_whitespace)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                collapsed
            } else {
                line.to_string()
            };

            let final_line = if self.trim {
                processed.trim()
            } else {
                &processed
            };

            // Handle empty lines
            if final_line.is_empty() {
                if self.preserve_empty_lines {
                    result.push(String::new());
                }
            } else {
                result.push(final_line.to_string());
            }
        }

        result.join("\n")
    }
}

/// Normalize a string using a Cow to avoid allocations when possible.
///
/// Returns a borrowed string if no normalization is needed.
///
/// # Examples
/// ```
/// use lintdiff_code_norm::normalize_cow;
/// use std::borrow::Cow;
///
/// let result = normalize_cow("hello world");
/// assert!(matches!(result, Cow::Borrowed(_)));
///
/// let result = normalize_cow("hello   world");
/// assert!(matches!(result, Cow::Owned(_)));
/// ```
#[must_use]
pub fn normalize_cow(s: &str) -> Cow<'_, str> {
    if needs_normalization(s) {
        Cow::Owned(normalize_code(s))
    } else {
        Cow::Borrowed(s)
    }
}

/// Check if a string needs normalization.
///
/// # Examples
/// ```
/// use lintdiff_code_norm::needs_normalization;
///
/// assert!(!needs_normalization("hello world"));
/// assert!(needs_normalization("hello   world"));
/// assert!(needs_normalization("hello\r\nworld"));
/// ```
#[must_use]
pub fn needs_normalization(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Check for various conditions that need normalization
    let mut prev_char = '\0';
    for c in s.chars() {
        // Check for CRLF or CR
        if c == '\r' {
            return true;
        }
        // Check for multiple consecutive spaces
        if c == ' ' && prev_char == ' ' {
            return true;
        }
        // Check for tabs
        if c == '\t' {
            return true;
        }
        prev_char = c;
    }

    // Check for leading/trailing whitespace
    s != s.trim()
}

/// Count the number of lines in a string.
///
/// # Examples
/// ```
/// use lintdiff_code_norm::count_lines;
///
/// assert_eq!(count_lines("hello\nworld"), 2);
/// assert_eq!(count_lines("single line"), 1);
/// assert_eq!(count_lines(""), 0);
/// ```
#[must_use]
pub fn count_lines(s: &str) -> usize {
    if s.is_empty() {
        return 0;
    }
    s.split('\n').count()
}

/// Detect the dominant line ending style in a string.
///
/// Returns the most common line ending style, or Unix if no line endings are found.
///
/// # Examples
/// ```
/// use lintdiff_code_norm::{detect_line_ending, LineEnding};
///
/// assert_eq!(detect_line_ending("a\nb\nc"), LineEnding::Unix);
/// assert_eq!(detect_line_ending("a\r\nb\r\nc"), LineEnding::Windows);
/// ```
#[must_use]
pub fn detect_line_ending(s: &str) -> LineEnding {
    let unix_count = s.matches('\n').count();
    let crlf_count = s.matches("\r\n").count();
    let cr_only_count = s.matches('\r').count() - crlf_count;

    // Determine dominant style
    if crlf_count > unix_count - crlf_count && crlf_count >= cr_only_count {
        LineEnding::Windows
    } else {
        LineEnding::Unix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_ending_default() {
        assert_eq!(LineEnding::default(), LineEnding::Unix);
    }

    #[test]
    fn test_line_ending_as_str() {
        assert_eq!(LineEnding::Unix.as_str(), "\n");
        assert_eq!(LineEnding::Windows.as_str(), "\r\n");
    }

    #[test]
    fn test_normalize_whitespace_simple() {
        assert_eq!(normalize_whitespace("hello world"), "hello world");
    }

    #[test]
    fn test_normalize_whitespace_multiple_spaces() {
        assert_eq!(normalize_whitespace("hello    world"), "hello world");
    }

    #[test]
    fn test_normalize_line_endings_crlf() {
        assert_eq!(normalize_line_endings("a\r\nb"), "a\nb");
    }

    #[test]
    fn test_normalize_line_endings_cr_only() {
        assert_eq!(normalize_line_endings("a\rb"), "a\nb");
    }

    #[test]
    fn test_code_normalizer_default() {
        let n = CodeNormalizer::new();
        assert!(n.trim);
        assert!(n.collapse_spaces);
        assert_eq!(n.tab_width, 4);
    }

    #[test]
    fn test_needs_normalization_false() {
        assert!(!needs_normalization("hello world"));
    }

    #[test]
    fn test_needs_normalization_true_spaces() {
        assert!(needs_normalization("hello  world"));
    }

    #[test]
    fn test_count_lines() {
        assert_eq!(count_lines("a\nb\nc"), 3);
        assert_eq!(count_lines("single"), 1);
        assert_eq!(count_lines(""), 0);
    }
}
