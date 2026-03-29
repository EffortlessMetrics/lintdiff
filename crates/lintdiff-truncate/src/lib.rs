//! Message and output truncation utilities for lintdiff.
//!
//! This microcrate provides a single responsibility: truncating messages and outputs
//! to fit within configured limits, with support for word boundary preservation.
//!
//! # Example: Basic Truncation
//!
//! ```
//! use lintdiff_truncate::{truncate, TruncateConfig};
//!
//! let config = TruncateConfig::new(20);
//! let result = truncate("This is a very long string that needs truncation", &config);
//! assert_eq!(result, "This is a very...");
//! ```
//!
//! # Example: GitHub Annotation Config
//!
//! ```
//! use lintdiff_truncate::{truncate, TruncateConfig};
//!
//! let config = TruncateConfig::github();
//! assert_eq!(config.max_length, 140);
//! assert!(config.preserve_words);
//! ```
//!
//! # Example: Word Boundary Preservation
//!
//! ```
//! use lintdiff_truncate::{truncate, TruncateConfig};
//!
//! let config = TruncateConfig::new(15).with_word_preservation(true);
//! let result = truncate("Hello world test", &config);
//! // Preserves word boundary: "Hello world..."
//! assert!(result.len() <= 15);
//! ```

#![warn(missing_docs)]

use std::borrow::Cow;

/// Configuration for truncation.
#[derive(Debug, Clone)]
pub struct TruncateConfig {
    /// Maximum length before truncation.
    pub max_length: usize,
    /// String to append when truncated.
    pub ellipsis: String,
    /// Whether to preserve word boundaries.
    pub preserve_words: bool,
}

impl Default for TruncateConfig {
    fn default() -> Self {
        Self {
            max_length: 120,
            ellipsis: "...".to_string(),
            preserve_words: true,
        }
    }
}

impl TruncateConfig {
    /// Create a new config with the given max length.
    ///
    /// Uses default ellipsis ("...") and enables word boundary preservation.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_truncate::TruncateConfig;
    ///
    /// let config = TruncateConfig::new(50);
    /// assert_eq!(config.max_length, 50);
    /// assert_eq!(config.ellipsis, "...");
    /// assert!(config.preserve_words);
    /// ```
    #[must_use]
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            ..Self::default()
        }
    }

    /// Create a config for GitHub annotations (140 char limit).
    ///
    /// GitHub annotations have a title limit of 140 characters.
    /// This config uses that limit with word boundary preservation enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_truncate::TruncateConfig;
    ///
    /// let config = TruncateConfig::github();
    /// assert_eq!(config.max_length, 140);
    /// ```
    #[must_use]
    pub fn github() -> Self {
        Self {
            max_length: 140,
            ..Self::default()
        }
    }

    /// Create a config for terminal output (no limit).
    ///
    /// This config uses `usize::MAX` as the maximum length, effectively
    /// disabling truncation while still allowing the truncate functions
    /// to be called uniformly.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_truncate::{truncate, TruncateConfig};
    ///
    /// let config = TruncateConfig::unlimited();
    /// let long_string = "a".repeat(1000);
    /// let result = truncate(&long_string, &config);
    /// assert_eq!(result.len(), 1000);
    /// ```
    #[must_use]
    pub fn unlimited() -> Self {
        Self {
            max_length: usize::MAX,
            ..Self::default()
        }
    }

    /// Set whether to preserve word boundaries.
    ///
    /// When enabled (the default), truncation will try to avoid cutting
    /// words in the middle, instead finding a word boundary near the
    /// truncation point.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_truncate::TruncateConfig;
    ///
    /// let config = TruncateConfig::new(20).with_word_preservation(false);
    /// assert!(!config.preserve_words);
    /// ```
    #[must_use]
    pub fn with_word_preservation(mut self, preserve: bool) -> Self {
        self.preserve_words = preserve;
        self
    }

    /// Set a custom ellipsis string.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_truncate::TruncateConfig;
    ///
    /// let config = TruncateConfig::new(20).with_ellipsis("…");
    /// assert_eq!(config.ellipsis, "…");
    /// ```
    #[must_use]
    pub fn with_ellipsis(mut self, ellipsis: impl Into<String>) -> Self {
        self.ellipsis = ellipsis.into();
        self
    }
}

/// Truncate a string to fit within the configured length.
///
/// Returns a `Cow<str>` that borrows the original string if no truncation
/// is needed, or an owned string if truncation occurred.
///
/// # Example
///
/// ```
/// use lintdiff_truncate::{truncate, TruncateConfig};
///
/// let config = TruncateConfig::new(10);
/// let result = truncate("Hello world, this is long", &config);
/// assert_eq!(result, "Hello...");
/// ```
#[must_use]
pub fn truncate<'a>(s: &'a str, config: &TruncateConfig) -> Cow<'a, str> {
    // Fast path: no truncation needed
    if !would_truncate(s, config) {
        return Cow::Borrowed(s);
    }

    // Calculate the target length (accounting for ellipsis)
    let ellipsis_len = config.ellipsis.len();
    let target_len = config.max_length.saturating_sub(ellipsis_len);

    if target_len == 0 {
        return Cow::Owned(config.ellipsis.clone());
    }

    // Find the truncation point
    let truncate_at = if config.preserve_words {
        find_word_boundary(s, target_len)
    } else {
        // Ensure we don't cut in the middle of a multi-byte character
        find_char_boundary(s, target_len)
    };

    // Build the truncated string, trimming trailing whitespace
    let truncated = s[..truncate_at].trim_end();
    let mut result = truncated.to_string();
    result.push_str(&config.ellipsis);
    Cow::Owned(result)
}

/// Truncate a string, returning an owned String.
///
/// This is a convenience wrapper around [`truncate`] that always returns
/// an owned `String`.
///
/// # Example
///
/// ```
/// use lintdiff_truncate::{truncate_owned, TruncateConfig};
///
/// let config = TruncateConfig::new(10);
/// let result = truncate_owned("Hello world, this is long", &config);
/// assert_eq!(result, "Hello...");
/// ```
#[must_use]
pub fn truncate_owned(s: &str, config: &TruncateConfig) -> String {
    truncate(s, config).into_owned()
}

/// Truncate multiple lines, keeping total under limit.
///
/// This function distributes the available length across all lines,
/// truncating each line proportionally if the total exceeds the limit.
/// Lines are processed in order, and earlier lines may use more of the
/// available space if later lines are short.
///
/// # Example
///
/// ```
/// use lintdiff_truncate::{truncate_lines, TruncateConfig};
///
/// let config = TruncateConfig::new(30);
/// let lines = vec![
///     "This is line one which is quite long".to_string(),
///     "Short line".to_string(),
/// ];
/// let result = truncate_lines(&lines, &config);
/// assert!(result.iter().map(|s| s.len()).sum::<usize>() <= 30);
/// ```
#[must_use]
pub fn truncate_lines(lines: &[String], config: &TruncateConfig) -> Vec<String> {
    if lines.is_empty() {
        return Vec::new();
    }

    let total_len: usize = lines.iter().map(|s| s.len()).sum();

    // If total fits, return as-is
    if total_len <= config.max_length {
        return lines.to_vec();
    }

    // Calculate how much to allocate per line on average
    let available = config.max_length.saturating_sub(config.ellipsis.len() * lines.len());
    let per_line = available / lines.len().max(1);
    let per_line = per_line.max(config.ellipsis.len());

    // Create a per-line config
    let line_config = TruncateConfig {
        max_length: per_line + config.ellipsis.len(),
        ellipsis: config.ellipsis.clone(),
        preserve_words: config.preserve_words,
    };

    // Truncate each line
    lines.iter().map(|line| truncate_owned(line, &line_config)).collect()
}

/// Check if a string would be truncated.
///
/// Returns `true` if the string length exceeds the configured maximum length.
///
/// # Example
///
/// ```
/// use lintdiff_truncate::{would_truncate, TruncateConfig};
///
/// let config = TruncateConfig::new(10);
/// assert!(would_truncate("This is a long string", &config));
/// assert!(!would_truncate("Short", &config));
/// ```
#[must_use]
pub fn would_truncate(s: &str, config: &TruncateConfig) -> bool {
    s.len() > config.max_length
}

/// Find a character boundary near the target byte position.
///
/// This ensures we don't cut in the middle of a multi-byte character.
fn find_char_boundary(s: &str, target_bytes: usize) -> usize {
    if target_bytes >= s.len() {
        return s.len();
    }

    // Find the nearest valid character boundary at or before target_bytes
    let mut pos = target_bytes;
    while !s.is_char_boundary(pos) && pos > 0 {
        pos -= 1;
    }
    pos
}

/// Find a word boundary near the target byte position.
///
/// This looks for whitespace near the truncation point to avoid
/// cutting words in the middle.
fn find_word_boundary(s: &str, target_bytes: usize) -> usize {
    // First, ensure we're at a valid character boundary
    let target = find_char_boundary(s, target_bytes);

    if target >= s.len() {
        return s.len();
    }

    // Look for whitespace before the target position
    // We search backwards from the target to find a word boundary
    let search_start = target.min(s.len());
    let substring = &s[..search_start];

    // Find the last whitespace before the target
    if let Some(last_space) = substring.rfind(|c: char| c.is_whitespace()) {
        // Make sure we're not returning an empty string
        // Only use the word boundary if it's not at the very start
        if last_space > 0 {
            // Return the position after the whitespace
            let after_space = last_space + s[last_space..].chars().next().map_or(0, |c| c.len_utf8());
            return after_space.min(target);
        }
    }

    // No word boundary found, fall back to character boundary
    target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TruncateConfig::default();
        assert_eq!(config.max_length, 120);
        assert_eq!(config.ellipsis, "...");
        assert!(config.preserve_words);
    }

    #[test]
    fn test_new_config() {
        let config = TruncateConfig::new(50);
        assert_eq!(config.max_length, 50);
        assert_eq!(config.ellipsis, "...");
        assert!(config.preserve_words);
    }

    #[test]
    fn test_github_config() {
        let config = TruncateConfig::github();
        assert_eq!(config.max_length, 140);
    }

    #[test]
    fn test_unlimited_config() {
        let config = TruncateConfig::unlimited();
        assert_eq!(config.max_length, usize::MAX);
    }

    #[test]
    fn test_no_truncation_needed() {
        let config = TruncateConfig::new(20);
        let result = truncate("Short string", &config);
        assert_eq!(result, "Short string");
    }

    #[test]
    fn test_basic_truncation() {
        let config = TruncateConfig::new(10);
        let result = truncate("Hello world, this is long", &config);
        assert_eq!(result, "Hello...");
        assert!(result.len() <= 10);
    }

    #[test]
    fn test_exact_length() {
        let config = TruncateConfig::new(11);
        let result = truncate("Hello world", &config);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_empty_string() {
        let config = TruncateConfig::new(10);
        let result = truncate("", &config);
        assert_eq!(result, "");
    }

    #[test]
    fn test_word_boundary_preservation() {
        let config = TruncateConfig::new(15).with_word_preservation(true);
        let result = truncate("Hello world test", &config);
        // Should truncate at word boundary: "Hello world..." (14 chars)
        assert!(result.ends_with("..."));
        assert_eq!(result, "Hello world...");
    }

    #[test]
    fn test_no_word_preservation() {
        let config = TruncateConfig::new(10).with_word_preservation(false);
        let result = truncate("Hello world", &config);
        // Should truncate exactly at 7 chars + ellipsis
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_custom_ellipsis() {
        let config = TruncateConfig::new(10).with_ellipsis("…");
        let result = truncate("Hello world, this is long", &config);
        assert!(result.ends_with("…"));
    }

    #[test]
    fn test_truncate_owned() {
        let config = TruncateConfig::new(10);
        let result = truncate_owned("Hello world, this is long", &config);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn test_would_truncate() {
        let config = TruncateConfig::new(10);
        assert!(would_truncate("This is a long string", &config));
        assert!(!would_truncate("Short", &config));
    }

    #[test]
    fn test_multibyte_characters() {
        let config = TruncateConfig::new(10);
        let result = truncate("Hello 世界世界世界世界", &config);
        // Should not panic and should be valid UTF-8
        assert!(result.len() <= 10);
    }

    #[test]
    fn test_truncate_lines_empty() {
        let config = TruncateConfig::new(30);
        let lines: Vec<String> = Vec::new();
        let result = truncate_lines(&lines, &config);
        assert!(result.is_empty());
    }

    #[test]
    fn test_truncate_lines_fits() {
        let config = TruncateConfig::new(30);
        let lines = vec!["Short line".to_string(), "Another".to_string()];
        let result = truncate_lines(&lines, &config);
        assert_eq!(result, lines);
    }

    #[test]
    fn test_truncate_lines_needs_truncation() {
        let config = TruncateConfig::new(30);
        let lines = vec![
            "This is line one which is quite long".to_string(),
            "Short line".to_string(),
        ];
        let result = truncate_lines(&lines, &config);
        // Total should be under limit
        let total: usize = result.iter().map(|s| s.len()).sum();
        assert!(total <= 30);
    }
}
