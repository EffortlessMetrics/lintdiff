//! Glob pattern matching for file paths.
//!
//! Provides zero-dependency glob matching with support for common patterns.
//!
//! # Supported Patterns
//!
//! - `*` - Matches any sequence of characters except path separator
//! - `**` - Matches any sequence of characters including path separators (globstar)
//! - `?` - Matches exactly one character
//! - `[abc]` - Matches any character in the set
//! - `[!abc]` or `[^abc]` - Matches any character NOT in the set
//! - `[a-z]` - Matches any character in the range
//! - Literal characters match themselves
//!
//! # Examples
//!
//! ```
//! use lintdiff_glob::Glob;
//!
//! let glob = Glob::new("src/**/*.rs")?;
//! assert!(glob.is_match("src/lib.rs"));
//! assert!(glob.is_match("src/foo/bar.rs"));
//! assert!(!glob.is_match("tests/foo.rs"));
//! # Ok::<(), lintdiff_glob::GlobError>(())
//! ```

use std::path::Path;

/// Errors that can occur during glob parsing.
#[derive(Debug, Clone, thiserror::Error)]
pub enum GlobError {
    /// The glob pattern is invalid.
    #[error("Invalid glob pattern: {0}")]
    InvalidPattern(String),
    /// A character class (bracket expression) was not properly closed.
    #[error("Unclosed character class")]
    UnclosedClass,
    /// The pattern is empty.
    #[error("Empty pattern")]
    EmptyPattern,
}

/// A compiled glob pattern for matching file paths.
///
/// # Examples
///
/// ```
/// use lintdiff_glob::Glob;
///
/// let glob = Glob::new("src/**/*.rs")?;
/// assert!(glob.is_match("src/lib.rs"));
/// assert!(glob.is_match("src/foo/bar.rs"));
/// assert!(!glob.is_match("tests/foo.rs"));
/// # Ok::<(), lintdiff_glob::GlobError>(())
/// ```
#[derive(Debug, Clone)]
pub struct Glob {
    pattern: String,
    // Internal compiled representation
    segments: Vec<PatternSegment>,
    // Path separator to use for matching (None = no separator constraints)
    path_separator: Option<char>,
}

/// Represents a single segment in a glob pattern.
#[derive(Debug, Clone)]
enum PatternSegment {
    /// Literal text that must match exactly
    Literal(String),
    /// `*` - matches any sequence of non-separator characters
    Wildcard,
    /// `**` - matches any sequence including separators (only valid between separators)
    Globstar,
    /// `?` - matches exactly one character
    SingleChar,
    /// `[...]` - character class
    CharClass(CharClass),
}

/// Represents a character class like `[abc]` or `[a-z]` or `[!abc]`.
#[derive(Debug, Clone)]
struct CharClass {
    /// Whether this is a negated class (`[!...]` or `[^...]`)
    negated: bool,
    /// The characters or ranges in the class
    chars: Vec<CharClassEntry>,
}

#[derive(Debug, Clone)]
enum CharClassEntry {
    /// A single character
    Char(char),
    /// A range of characters (e.g., `a-z`)
    Range(char, char),
}

impl Glob {
    /// Create a new glob pattern from a string.
    ///
    /// # Errors
    ///
    /// Returns `GlobError` if the pattern is invalid:
    /// - `GlobError::EmptyPattern` if the pattern is empty
    /// - `GlobError::UnclosedClass` if a character class is not properly closed
    /// - `GlobError::InvalidPattern` for other parsing errors
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_glob::Glob;
    ///
    /// let glob = Glob::new("*.rs")?;
    /// assert!(glob.is_match("lib.rs"));
    /// assert!(!glob.is_match("src/lib.rs"));
    /// # Ok::<(), lintdiff_glob::GlobError>(())
    /// ```
    pub fn new(pattern: &str) -> Result<Self, GlobError> {
        if pattern.is_empty() {
            return Err(GlobError::EmptyPattern);
        }

        let segments = Self::parse_pattern(pattern)?;
        Ok(Self {
            pattern: pattern.to_string(),
            segments,
            path_separator: Some('/'),
        })
    }

    /// Create a new glob pattern with no path separator constraints.
    ///
    /// This is useful for matching code lines or other strings where
    /// path separators should not be treated specially.
    ///
    /// # Errors
    ///
    /// Returns `GlobError` if the pattern is invalid:
    /// - `GlobError::EmptyPattern` if the pattern is empty
    /// - `GlobError::UnclosedClass` if a character class is not properly closed
    /// - `GlobError::InvalidPattern` for other parsing errors
    pub fn new_no_separator(pattern: &str) -> Result<Self, GlobError> {
        if pattern.is_empty() {
            return Err(GlobError::EmptyPattern);
        }

        let segments = Self::parse_pattern(pattern)?;
        Ok(Self {
            pattern: pattern.to_string(),
            segments,
            path_separator: None,
        })
    }

    /// Parse a pattern string into segments.
    fn parse_pattern(pattern: &str) -> Result<Vec<PatternSegment>, GlobError> {
        let mut segments = Vec::new();
        let mut chars = pattern.chars().peekable();
        let mut current_literal = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                '*' => {
                    // Check if this is a globstar (**)
                    if chars.peek() == Some(&'*') {
                        chars.next(); // consume the second *
                                      // Flush any pending literal
                        if !current_literal.is_empty() {
                            segments.push(PatternSegment::Literal(current_literal.clone()));
                            current_literal.clear();
                        }
                        segments.push(PatternSegment::Globstar);
                        // Skip trailing slash after globstar - it's implied
                        if chars.peek() == Some(&'/') {
                            chars.next();
                        }
                    } else {
                        // Flush any pending literal
                        if !current_literal.is_empty() {
                            segments.push(PatternSegment::Literal(current_literal.clone()));
                            current_literal.clear();
                        }
                        segments.push(PatternSegment::Wildcard);
                    }
                }
                '?' => {
                    // Flush any pending literal
                    if !current_literal.is_empty() {
                        segments.push(PatternSegment::Literal(current_literal.clone()));
                        current_literal.clear();
                    }
                    segments.push(PatternSegment::SingleChar);
                }
                '[' => {
                    // Flush any pending literal
                    if !current_literal.is_empty() {
                        segments.push(PatternSegment::Literal(current_literal.clone()));
                        current_literal.clear();
                    }
                    let char_class = Self::parse_char_class(&mut chars)?;
                    segments.push(PatternSegment::CharClass(char_class));
                }
                _ => {
                    current_literal.push(ch);
                }
            }
        }

        // Flush any remaining literal
        if !current_literal.is_empty() {
            segments.push(PatternSegment::Literal(current_literal));
        }

        Ok(segments)
    }

    /// Parse a character class starting after the opening `[`.
    fn parse_char_class(
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Result<CharClass, GlobError> {
        let mut negated = false;
        let mut entries = Vec::new();

        // Check for negation (! or ^)
        if matches!(chars.peek(), Some(&'!' | &'^')) {
            negated = true;
            chars.next();
        }

        // Handle ] as first character (literal])
        if chars.peek() == Some(&']') {
            entries.push(CharClassEntry::Char(']'));
            chars.next();
        }

        loop {
            match chars.next() {
                None => return Err(GlobError::UnclosedClass),
                Some(']') => break,
                Some('\\') => {
                    // Escape sequence
                    if let Some(escaped) = chars.next() {
                        entries.push(CharClassEntry::Char(escaped));
                    }
                }
                Some(ch) => {
                    // Check for range (a-z)
                    if chars.peek() == Some(&'-') {
                        chars.next(); // consume -
                        if chars.peek() == Some(&']') {
                            // Trailing dash: [a-] means 'a' or '-'
                            entries.push(CharClassEntry::Char(ch));
                            entries.push(CharClassEntry::Char('-'));
                        } else if let Some(end) = chars.next() {
                            if end == ']' {
                                return Err(GlobError::UnclosedClass);
                            }
                            entries.push(CharClassEntry::Range(ch, end));
                        }
                    } else {
                        entries.push(CharClassEntry::Char(ch));
                    }
                }
            }
        }

        Ok(CharClass {
            negated,
            chars: entries,
        })
    }

    /// Check if a path matches this glob pattern.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check (can be string or Path)
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_glob::Glob;
    ///
    /// let glob = Glob::new("*.rs")?;
    /// assert!(glob.is_match("lib.rs"));
    /// assert!(!glob.is_match("src/lib.rs"));
    /// # Ok::<(), lintdiff_glob::GlobError>(())
    /// ```
    pub fn is_match<P: AsRef<Path>>(&self, path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy();
        // Normalize path separators to forward slashes if we have a separator
        let normalized = self.path_separator.map_or_else(
            || path_str.to_string(),
            |sep| path_str.replace('\\', &sep.to_string()),
        );
        Self::match_segments(&self.segments, &normalized, self.path_separator)
    }

    /// Match a path against the parsed segments.
    fn match_segments(
        segments: &[PatternSegment],
        path: &str,
        path_separator: Option<char>,
    ) -> bool {
        if segments.is_empty() {
            return path.is_empty();
        }

        let mut seg_idx = 0;
        let mut path_idx = 0;
        let chars: Vec<char> = path.chars().collect();

        while seg_idx < segments.len() {
            let segment = &segments[seg_idx];

            match segment {
                PatternSegment::Literal(lit) => {
                    let lit_chars: Vec<char> = lit.chars().collect();
                    if path_idx + lit_chars.len() > chars.len() {
                        return false;
                    }
                    for (i, &c) in lit_chars.iter().enumerate() {
                        if chars[path_idx + i] != c {
                            return false;
                        }
                    }
                    path_idx += lit_chars.len();
                }
                PatternSegment::Wildcard => {
                    // Find the next non-wildcard segment
                    let next_seg_idx = seg_idx + 1;
                    if next_seg_idx >= segments.len() {
                        // * at the end matches everything remaining
                        // But only if there's no path separator (if we have one)
                        if let Some(sep) = path_separator {
                            return !chars[path_idx..].contains(&sep);
                        }
                        return true;
                    }

                    // Find how many characters * can consume
                    let remaining = Self::match_after_wildcard(
                        &segments[next_seg_idx..],
                        &chars[path_idx..],
                        path_separator,
                    );
                    match remaining {
                        Some(new_idx) => {
                            path_idx += new_idx;
                            seg_idx = next_seg_idx;
                            continue;
                        }
                        None => return false,
                    }
                }
                PatternSegment::Globstar => {
                    // ** matches anything including path separators
                    let next_seg_idx = seg_idx + 1;
                    if next_seg_idx >= segments.len() {
                        // ** at the end matches everything
                        return true;
                    }

                    // Try matching from each possible position
                    for start_pos in path_idx..=chars.len() {
                        if Self::match_segments(
                            &segments[next_seg_idx..],
                            &chars[start_pos..].iter().collect::<String>(),
                            path_separator,
                        ) {
                            return true;
                        }
                    }
                    return false;
                }
                PatternSegment::SingleChar => {
                    if path_idx >= chars.len() {
                        return false;
                    }
                    // ? does not match path separator
                    if chars[path_idx] == '/' {
                        return false;
                    }
                    path_idx += 1;
                }
                PatternSegment::CharClass(class) => {
                    if path_idx >= chars.len() {
                        return false;
                    }
                    let c = chars[path_idx];
                    // Character classes do not match path separator
                    if c == '/' {
                        return false;
                    }
                    if !class.matches(c) {
                        return false;
                    }
                    path_idx += 1;
                }
            }
            seg_idx += 1;
        }

        path_idx == chars.len()
    }

    /// Find the position after a wildcard that allows the rest to match.
    fn match_after_wildcard(
        remaining_segments: &[PatternSegment],
        chars: &[char],
        path_separator: Option<char>,
    ) -> Option<usize> {
        for end_pos in 0..=chars.len() {
            // Check if we hit a path separator (if we have one)
            if let Some(sep) = path_separator {
                if end_pos > 0 && chars[..end_pos].contains(&sep) {
                    break;
                }
            }
            if Self::match_segments(
                remaining_segments,
                &chars[end_pos..].iter().collect::<String>(),
                path_separator,
            ) {
                return Some(end_pos);
            }
        }
        None
    }

    /// Get the original pattern string.
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

impl CharClass {
    /// Check if a character matches this character class.
    fn matches(&self, c: char) -> bool {
        let mut found = false;
        for entry in &self.chars {
            match entry {
                CharClassEntry::Char(ch) => {
                    if *ch == c {
                        found = true;
                        break;
                    }
                }
                CharClassEntry::Range(start, end) => {
                    if c >= *start && c <= *end {
                        found = true;
                        break;
                    }
                }
            }
        }
        if self.negated {
            !found
        } else {
            found
        }
    }
}

/// A set of glob patterns for matching against multiple patterns.
///
/// # Examples
///
/// ```
/// use lintdiff_glob::{Glob, GlobSet};
///
/// let set = GlobSet::new(vec!["*.rs", "*.toml"])?;
/// assert!(set.matches_any("lib.rs"));
/// assert!(set.matches_any("Cargo.toml"));
/// assert!(!set.matches_any("README.md"));
/// # Ok::<(), lintdiff_glob::GlobError>(())
/// ```
#[derive(Debug, Clone)]
pub struct GlobSet {
    patterns: Vec<Glob>,
}

impl GlobSet {
    /// Create a new glob set from multiple patterns.
    ///
    /// # Errors
    ///
    /// Returns `GlobError` if any pattern is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_glob::GlobSet;
    ///
    /// let set = GlobSet::new(vec!["*.rs", "*.toml"])?;
    /// assert_eq!(set.len(), 2);
    /// # Ok::<(), lintdiff_glob::GlobError>(())
    /// ```
    pub fn new(patterns: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Self, GlobError> {
        let globs = patterns
            .into_iter()
            .map(|p| Glob::new(p.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { patterns: globs })
    }

    /// Check if a path matches any pattern in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_glob::GlobSet;
    ///
    /// let set = GlobSet::new(vec!["*.rs", "*.toml"])?;
    /// assert!(set.matches_any("lib.rs"));
    /// assert!(set.matches_any("Cargo.toml"));
    /// assert!(!set.matches_any("README.md"));
    /// # Ok::<(), lintdiff_glob::GlobError>(())
    /// ```
    pub fn matches_any<P: AsRef<Path>>(&self, path: P) -> bool {
        self.patterns.iter().any(|g| g.is_match(&path))
    }

    /// Filter a collection of paths, returning only those that match any pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_glob::GlobSet;
    ///
    /// let set = GlobSet::new(vec!["*.rs", "*.toml"])?;
    /// let paths = vec!["lib.rs", "README.md", "Cargo.toml"];
    /// let filtered: Vec<_> = set.filter(paths).collect();
    /// assert_eq!(filtered, vec!["lib.rs", "Cargo.toml"]);
    /// # Ok::<(), lintdiff_glob::GlobError>(())
    /// ```
    pub fn filter<'a, P: AsRef<Path>>(
        &'a self,
        paths: impl IntoIterator<Item = P> + 'a,
    ) -> impl Iterator<Item = P> + 'a {
        paths.into_iter().filter(move |p| self.matches_any(p))
    }

    /// Get the number of patterns in the set.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Check if the set is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pattern() {
        assert!(matches!(Glob::new(""), Err(GlobError::EmptyPattern)));
    }

    #[test]
    fn test_literal_match() {
        let glob = Glob::new("foo").unwrap();
        assert!(glob.is_match("foo"));
        assert!(!glob.is_match("bar"));
        assert!(!glob.is_match("foobar"));
    }

    #[test]
    fn test_wildcard_match() {
        let glob = Glob::new("*.rs").unwrap();
        assert!(glob.is_match("lib.rs"));
        assert!(glob.is_match("main.rs"));
        assert!(!glob.is_match("lib.txt"));
        assert!(!glob.is_match("src/lib.rs"));
    }

    #[test]
    fn test_single_char_match() {
        let glob = Glob::new("file?.txt").unwrap();
        assert!(glob.is_match("file1.txt"));
        assert!(glob.is_match("fileA.txt"));
        assert!(!glob.is_match("file.txt"));
        assert!(!glob.is_match("file12.txt"));
    }

    #[test]
    fn test_char_class() {
        let glob = Glob::new("[abc].txt").unwrap();
        assert!(glob.is_match("a.txt"));
        assert!(glob.is_match("b.txt"));
        assert!(glob.is_match("c.txt"));
        assert!(!glob.is_match("d.txt"));
    }

    #[test]
    fn test_char_class_range() {
        let glob = Glob::new("[a-c].txt").unwrap();
        assert!(glob.is_match("a.txt"));
        assert!(glob.is_match("b.txt"));
        assert!(glob.is_match("c.txt"));
        assert!(!glob.is_match("d.txt"));
    }

    #[test]
    fn test_negated_char_class() {
        let glob = Glob::new("[!abc].txt").unwrap();
        assert!(!glob.is_match("a.txt"));
        assert!(!glob.is_match("b.txt"));
        assert!(!glob.is_match("c.txt"));
        assert!(glob.is_match("d.txt"));
    }

    #[test]
    fn test_globstar() {
        let glob = Glob::new("src/**/*.rs").unwrap();
        assert!(glob.is_match("src/lib.rs"));
        assert!(glob.is_match("src/foo/bar.rs"));
        assert!(glob.is_match("src/a/b/c/d.rs"));
        assert!(!glob.is_match("tests/lib.rs"));
    }

    #[test]
    fn test_globset() {
        let set = GlobSet::new(vec!["*.rs", "*.toml"]).unwrap();
        assert!(set.matches_any("lib.rs"));
        assert!(set.matches_any("Cargo.toml"));
        assert!(!set.matches_any("README.md"));
    }
}
