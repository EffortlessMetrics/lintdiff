//! Unicode-aware message truncation utilities for lintdiff.
//!
//! This microcrate provides a single responsibility: truncating messages while
//! preserving Unicode character boundaries, word boundaries, and sentence boundaries.
//!
//! # Example: Basic Character Truncation
//!
//! ```
//! use lintdiff_message_truncate::truncate;
//!
//! // With max_chars=8, we can fit 5 chars + 3 ellipsis
//! let result = truncate("Hello, world!", 8);
//! assert_eq!(result, "Hello...");
//! ```
//!
//! # Example: Unicode Safety
//!
//! ```
//! use lintdiff_message_truncate::truncate;
//!
//! // The emoji is 4 bytes but 1 character - we won't split it
//! // "Hello 😀 world" is 13 chars, with max_chars=10 we can fit 7 chars + 3 ellipsis
//! let result = truncate("Hello 😀 world", 10);
//! assert_eq!(result, "Hello 😀...");
//! ```
//!
//! # Example: Word Boundary Preservation
//!
//! ```
//! use lintdiff_message_truncate::truncate_words;
//!
//! let result = truncate_words("Hello beautiful world", 2);
//! assert_eq!(result, "Hello beautiful...");
//! ```
//!
//! # Example: Custom Options
//!
//! ```
//! use lintdiff_message_truncate::{truncate_with_options, TruncateOptions};
//!
//! let options = TruncateOptions::new(20)
//!     .with_ellipsis("…")
//!     .with_preserve_words(true);
//! let result = truncate_with_options("This is a very long message", &options);
//! assert_eq!(result, "This is a very…");
//! ```

#![warn(missing_docs)]

/// Default ellipsis string used when truncating.
pub const DEFAULT_ELLIPSIS: &str = "...";

/// Configuration options for truncation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TruncateOptions {
    /// Maximum length (interpretation depends on mode).
    pub max_length: usize,
    /// Ellipsis string to append when truncated.
    pub ellipsis: String,
    /// Whether to preserve word boundaries.
    pub preserve_words: bool,
    /// Whether to preserve sentence boundaries.
    pub preserve_sentences: bool,
}

impl Default for TruncateOptions {
    fn default() -> Self {
        Self {
            max_length: 100,
            ellipsis: DEFAULT_ELLIPSIS.to_string(),
            preserve_words: false,
            preserve_sentences: false,
        }
    }
}

impl TruncateOptions {
    /// Create new options with the specified maximum length.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_message_truncate::TruncateOptions;
    ///
    /// let options = TruncateOptions::new(50);
    /// assert_eq!(options.max_length, 50);
    /// ```
    #[must_use]
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            ..Self::default()
        }
    }

    /// Set a custom ellipsis string.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_message_truncate::TruncateOptions;
    ///
    /// let options = TruncateOptions::new(20).with_ellipsis("…");
    /// assert_eq!(options.ellipsis, "…");
    /// ```
    #[must_use]
    pub fn with_ellipsis(mut self, ellipsis: impl Into<String>) -> Self {
        self.ellipsis = ellipsis.into();
        self
    }

    /// Enable or disable word boundary preservation.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_message_truncate::TruncateOptions;
    ///
    /// let options = TruncateOptions::new(20).with_preserve_words(true);
    /// assert!(options.preserve_words);
    /// ```
    #[must_use]
    pub fn with_preserve_words(mut self, preserve: bool) -> Self {
        self.preserve_words = preserve;
        self
    }

    /// Enable or disable sentence boundary preservation.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_message_truncate::TruncateOptions;
    ///
    /// let options = TruncateOptions::new(100).with_preserve_sentences(true);
    /// assert!(options.preserve_sentences);
    /// ```
    #[must_use]
    pub fn with_preserve_sentences(mut self, preserve: bool) -> Self {
        self.preserve_sentences = preserve;
        self
    }
}

/// Get the character length of a string (not byte length).
///
/// This counts Unicode scalar values, not grapheme clusters.
/// For most use cases, this is the correct measure of "character" count.
///
/// # Example
///
/// ```
/// use lintdiff_message_truncate::char_len;
///
/// assert_eq!(char_len("Hello"), 5);
/// assert_eq!(char_len("Hello 😀"), 7); // emoji is 1 character
/// assert_eq!(char_len(""), 0);
/// ```
#[must_use]
pub fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Truncate a string to a maximum number of characters.
///
/// Preserves Unicode character boundaries and appends an ellipsis if truncated.
///
/// # Example
///
/// ```
/// use lintdiff_message_truncate::truncate;
///
/// // With max_chars=8, we can fit 5 chars + 3 ellipsis
/// assert_eq!(truncate("Hello, world!", 8), "Hello...");
/// assert_eq!(truncate("Hi", 10), "Hi"); // No truncation needed
/// assert_eq!(truncate("", 5), ""); // Empty string
/// ```
#[must_use]
pub fn truncate(s: &str, max_chars: usize) -> String {
    truncate_impl(s, max_chars, DEFAULT_ELLIPSIS, false, false)
}

/// Truncate a string to a maximum number of bytes while preserving UTF-8 boundaries.
///
/// This is useful when you need to fit within byte limits (e.g., database columns).
///
/// # Example
///
/// ```
/// use lintdiff_message_truncate::truncate_bytes;
///
/// // "Hello 😀" is 10 bytes (5 + 1 space + 4 for emoji)
/// let result = truncate_bytes("Hello 😀 world", 8);
/// assert_eq!(result, "Hello...");
/// ```
#[must_use]
pub fn truncate_bytes(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }

    let ellipsis = DEFAULT_ELLIPSIS;
    let ellipsis_len = ellipsis.len();

    if max_bytes == 0 {
        return String::new();
    }

    if max_bytes <= ellipsis_len {
        // Return as much of ellipsis as fits
        return ellipsis.chars().take(max_bytes).collect();
    }

    let target_bytes = max_bytes - ellipsis_len;

    // Find valid UTF-8 boundary
    let mut end = target_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }

    if end == 0 {
        return ellipsis.chars().take(max_bytes).collect();
    }

    format!("{}{}", &s[..end], ellipsis)
}

/// Truncate a string to a maximum number of words.
///
/// Words are separated by whitespace. Preserves word boundaries.
///
/// # Example
///
/// ```
/// use lintdiff_message_truncate::truncate_words;
///
/// assert_eq!(truncate_words("Hello beautiful world", 2), "Hello beautiful...");
/// assert_eq!(truncate_words("One", 5), "One"); // No truncation needed
/// ```
#[must_use]
pub fn truncate_words(s: &str, max_words: usize) -> String {
    if max_words == 0 {
        return DEFAULT_ELLIPSIS.to_string();
    }

    // Split into words (whitespace-separated)
    let words: Vec<&str> = s.split_whitespace().collect();

    // If we have fewer or equal words than max, return as-is
    if words.len() <= max_words {
        return s.to_string();
    }

    // Take only max_words words and join them
    let result: String = words.into_iter().take(max_words).collect::<Vec<_>>().join(" ");

    if result.is_empty() {
        return DEFAULT_ELLIPSIS.to_string();
    }

    format!("{}{}", result, DEFAULT_ELLIPSIS)
}

/// Truncate a string using the provided options.
///
/// This is the most flexible truncation function, allowing customization of
/// the ellipsis, word preservation, and sentence preservation.
///
/// # Example
///
/// ```
/// use lintdiff_message_truncate::{truncate_with_options, TruncateOptions};
///
/// let options = TruncateOptions::new(15)
///     .with_ellipsis("…")
///     .with_preserve_words(true);
/// let result = truncate_with_options("This is a long message", &options);
/// assert_eq!(result, "This is a…");
/// ```
#[must_use]
pub fn truncate_with_options(s: &str, options: &TruncateOptions) -> String {
    truncate_impl(
        s,
        options.max_length,
        &options.ellipsis,
        options.preserve_words,
        options.preserve_sentences,
    )
}

/// Internal implementation of truncation.
fn truncate_impl(
    s: &str,
    max_chars: usize,
    ellipsis: &str,
    preserve_words: bool,
    preserve_sentences: bool,
) -> String {
    // Fast path: empty string
    if s.is_empty() {
        return String::new();
    }

    // Fast path: no truncation needed
    let char_count = char_len(s);
    if char_count <= max_chars {
        return s.to_string();
    }

    // Handle zero max_chars
    if max_chars == 0 {
        return ellipsis.to_string();
    }

    let ellipsis_len = char_len(ellipsis);

    // If ellipsis is longer than max, just return ellipsis truncated
    if ellipsis_len >= max_chars {
        // Return as much of ellipsis as fits
        return ellipsis.chars().take(max_chars).collect();
    }

    let target_chars = max_chars - ellipsis_len;

    // Find the truncation point
    let truncate_at = if preserve_sentences {
        find_sentence_boundary(s, target_chars)
    } else if preserve_words {
        find_word_boundary(s, target_chars)
    } else {
        // Character-indexed boundary
        s.char_indices()
            .nth(target_chars)
            .map(|(i, _)| i)
            .unwrap_or(s.len())
    };

    if truncate_at == 0 {
        return ellipsis.to_string();
    }

    format!("{}{}", &s[..truncate_at], ellipsis)
}

/// Find a word boundary near the target character position.
fn find_word_boundary(s: &str, target_chars: usize) -> usize {
    // First, find the byte position for target_chars
    let target_byte_pos = s
        .char_indices()
        .nth(target_chars)
        .map(|(i, _)| i)
        .unwrap_or(s.len());

    // Look for a word boundary (space) before the target
    // Search backwards from target for a space
    let mut best_boundary = 0;

    for (char_count, (i, c)) in s.char_indices().enumerate() {
        if char_count >= target_chars {
            break;
        }
        if c.is_whitespace() {
            best_boundary = i;
        }
    }

    // If we found a reasonable boundary (at least 50% of target), use it
    if best_boundary > 0 {
        best_boundary
    } else {
        // No word boundary found, fall back to character boundary
        target_byte_pos
    }
}

/// Find a sentence boundary near the target character position.
fn find_sentence_boundary(s: &str, target_chars: usize) -> usize {
    // Sentence terminators
    const TERMINATORS: &[char] = &['.', '!', '?'];

    // Look for a sentence boundary before the target
    let mut best_boundary = 0;

    for (char_count, (i, c)) in s.char_indices().enumerate() {
        if char_count >= target_chars {
            break;
        }
        if TERMINATORS.contains(&c) {
            // Check if followed by space or end
            let remaining = &s[i + c.len_utf8()..];
            let next_is_space = remaining.chars().next().is_none_or(|c| c.is_whitespace());
            if next_is_space {
                best_boundary = i + c.len_utf8();
            }
        }
    }

    // If we found a sentence boundary, use it
    if best_boundary > 0 {
        best_boundary
    } else {
        // Fall back to word boundary, then character boundary
        find_word_boundary(s, target_chars)
    }
}

/// Check if a string was truncated by comparing original and truncated versions.
///
/// # Example
///
/// ```
/// use lintdiff_message_truncate::is_truncated;
///
/// assert!(is_truncated("Hello, world!", "Hello..."));
/// assert!(!is_truncated("Hello", "Hello"));
/// ```
#[must_use]
pub fn is_truncated(original: &str, truncated: &str) -> bool {
    // Simple check: if lengths differ, it was truncated
    // More sophisticated: check if truncated is a prefix + ellipsis
    if original.len() != truncated.len() {
        return true;
    }

    // Check if they're actually different
    original != truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_len_basic() {
        assert_eq!(char_len("Hello"), 5);
        assert_eq!(char_len(""), 0);
        assert_eq!(char_len("Hello 😀"), 7);
    }

    #[test]
    fn test_truncate_basic() {
        // With max_chars=8, we can fit 5 chars + 3 ellipsis
        assert_eq!(truncate("Hello, world!", 8), "Hello...");
        assert_eq!(truncate("Hi", 10), "Hi");
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn test_truncate_unicode() {
        // Emoji is 4 bytes, 1 character
        // "Hello 😀" is 7 chars, with max_chars=6 we can fit 3 chars + 3 ellipsis
        assert_eq!(truncate("Hello 😀", 6), "Hel...");
        // "Hello 😀 world" is 13 chars, with max_chars=10 we can fit 7 chars + 3 ellipsis
        assert_eq!(truncate("Hello 😀 world", 10), "Hello 😀...");
    }

    #[test]
    fn test_truncate_bytes_basic() {
        assert_eq!(truncate_bytes("Hello", 10), "Hello");
        // With max_bytes=3, only ellipsis fits
        assert_eq!(truncate_bytes("Hello", 3), "...");
    }

    #[test]
    fn test_truncate_words_basic() {
        assert_eq!(truncate_words("Hello beautiful world", 2), "Hello beautiful...");
        assert_eq!(truncate_words("One two", 5), "One two");
    }

    #[test]
    fn test_is_truncated() {
        assert!(is_truncated("Hello, world!", "Hello..."));
        assert!(!is_truncated("Hello", "Hello"));
    }
}
