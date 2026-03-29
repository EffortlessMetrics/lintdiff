//! String slugification utilities for lintdiff.
//!
//! This microcrate provides a single responsibility: converting strings to URL-safe slugs
//! with configurable options for character handling and formatting.
//!
//! # Example: Basic Slugification
//!
//! ```
//! use lintdiff_slugify::slugify;
//!
//! let slug = slugify("Hello World");
//! assert_eq!(slug, "hello-world");
//! ```
//!
//! # Example: Custom Options
//!
//! ```
//! use lintdiff_slugify::{slugify_with_options, SlugOptions};
//!
//! let options = SlugOptions::new()
//!     .with_separator('_')
//!     .with_max_length(20);
//! let slug = slugify_with_options("This is a Very Long Title Example", &options);
//! assert_eq!(slug, "this_is_a_very_long");
//! ```
//!
//! # Example: Builder Pattern
//!
//! ```
//! use lintdiff_slugify::SlugifierBuilder;
//!
//! let slugifier = SlugifierBuilder::new()
//!     .with_lowercase(true)
//!     .with_separator('-')
//!     .build();
//! let slug = slugifier.slugify("Test String");
//! assert_eq!(slug, "test-string");
//! ```

#![warn(missing_docs)]

use std::borrow::Cow;

/// Configuration options for slug generation.
///
/// Controls how strings are converted to slugs, including case handling,
/// character preservation, length limits, and separators.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlugOptions {
    /// Convert to lowercase (default: true).
    pub lowercase: bool,
    /// Keep special characters like @, #, $ (default: false).
    pub preserve_special: bool,
    /// Keep alphanumeric characters (default: true, always preserved).
    pub preserve_alphanumeric: bool,
    /// Maximum length limit (default: None).
    pub max_length: Option<usize>,
    /// Separator between words (default: '-').
    pub separator: char,
}

impl Default for SlugOptions {
    fn default() -> Self {
        Self {
            lowercase: true,
            preserve_special: false,
            preserve_alphanumeric: true,
            max_length: None,
            separator: '-',
        }
    }
}

impl SlugOptions {
    /// Create new options with defaults.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugOptions;
    ///
    /// let options = SlugOptions::new();
    /// assert!(options.lowercase);
    /// assert_eq!(options.separator, '-');
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            lowercase: true,
            preserve_special: false,
            preserve_alphanumeric: true,
            max_length: None,
            separator: '-',
        }
    }

    /// Set lowercase option.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugOptions;
    ///
    /// let options = SlugOptions::new().with_lowercase(false);
    /// assert!(!options.lowercase);
    /// ```
    #[must_use]
    pub fn with_lowercase(mut self, lowercase: bool) -> Self {
        self.lowercase = lowercase;
        self
    }

    /// Set special character preservation.
    ///
    /// When enabled, characters like @, #, $ are preserved in the output.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugOptions;
    ///
    /// let options = SlugOptions::new().with_preserve_special(true);
    /// assert!(options.preserve_special);
    /// ```
    #[must_use]
    pub fn with_preserve_special(mut self, preserve_special: bool) -> Self {
        self.preserve_special = preserve_special;
        self
    }

    /// Set alphanumeric character preservation.
    ///
    /// Note: Alphanumeric characters are always preserved regardless of this setting.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugOptions;
    ///
    /// let options = SlugOptions::new().with_preserve_alphanumeric(true);
    /// assert!(options.preserve_alphanumeric);
    /// ```
    #[must_use]
    pub fn with_preserve_alphanumeric(mut self, preserve_alphanumeric: bool) -> Self {
        self.preserve_alphanumeric = preserve_alphanumeric;
        self
    }

    /// Set maximum length limit.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugOptions;
    ///
    /// let options = SlugOptions::new().with_max_length(50);
    /// assert_eq!(options.max_length, Some(50));
    /// ```
    #[must_use]
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set separator character.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugOptions;
    ///
    /// let options = SlugOptions::new().with_separator('_');
    /// assert_eq!(options.separator, '_');
    /// ```
    #[must_use]
    pub fn with_separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }
}

/// Check if a character is a special character that might be preserved.
const fn is_special_char(c: char) -> bool {
    matches!(c, '@' | '#' | '$' | '%' | '&' | '+' | '=' | '~' | '`')
}

/// Slugifier with configured options.
///
/// Created by [`SlugifierBuilder`] to provide optimized slugification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slugifier {
    options: SlugOptions,
}

impl Slugifier {
    /// Create a new slugifier with the given options.
    #[must_use]
    pub const fn new(options: SlugOptions) -> Self {
        Self { options }
    }

    /// Convert a string to a slug using configured options.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::{Slugifier, SlugOptions};
    ///
    /// let slugifier = Slugifier::new(SlugOptions::new());
    /// assert_eq!(slugifier.slugify("Hello World"), "hello-world");
    /// ```
    #[must_use]
    pub fn slugify(&self, s: &str) -> String {
        slugify_with_options(s, &self.options)
    }

    /// Get the configured options.
    #[must_use]
    pub const fn options(&self) -> &SlugOptions {
        &self.options
    }
}

/// Builder for creating configured slugifiers.
///
/// Provides a fluent interface for constructing [`Slugifier`] instances
/// with custom options.
///
/// # Example
///
/// ```
/// use lintdiff_slugify::SlugifierBuilder;
///
/// let slugifier = SlugifierBuilder::new()
///     .with_lowercase(true)
///     .with_separator('_')
///     .with_max_length(30)
///     .build();
///
/// let slug = slugifier.slugify("My Awesome Blog Post Title");
/// assert_eq!(slug, "my_awesome_blog_post_title");
/// ```
#[derive(Debug, Clone, Default)]
pub struct SlugifierBuilder {
    options: SlugOptions,
}

impl SlugifierBuilder {
    /// Create a new builder with default options.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugifierBuilder;
    ///
    /// let builder = SlugifierBuilder::new();
    /// let slugifier = builder.build();
    /// assert_eq!(slugifier.slugify("Test"), "test");
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: SlugOptions::new(),
        }
    }

    /// Set lowercase option.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugifierBuilder;
    ///
    /// let slugifier = SlugifierBuilder::new()
    ///     .with_lowercase(false)
    ///     .build();
    /// assert_eq!(slugifier.slugify("Hello"), "Hello");
    /// ```
    #[must_use]
    pub fn with_lowercase(mut self, lowercase: bool) -> Self {
        self.options.lowercase = lowercase;
        self
    }

    /// Set special character preservation.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugifierBuilder;
    ///
    /// let slugifier = SlugifierBuilder::new()
    ///     .with_preserve_special(true)
    ///     .build();
    /// assert_eq!(slugifier.slugify("Test@123"), "test@123");
    /// ```
    #[must_use]
    pub fn with_preserve_special(mut self, preserve_special: bool) -> Self {
        self.options.preserve_special = preserve_special;
        self
    }

    /// Set alphanumeric character preservation.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugifierBuilder;
    ///
    /// let slugifier = SlugifierBuilder::new()
    ///     .with_preserve_alphanumeric(true)
    ///     .build();
    /// assert_eq!(slugifier.slugify("Test123"), "test123");
    /// ```
    #[must_use]
    pub fn with_preserve_alphanumeric(mut self, preserve_alphanumeric: bool) -> Self {
        self.options.preserve_alphanumeric = preserve_alphanumeric;
        self
    }

    /// Set maximum length limit.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugifierBuilder;
    ///
    /// let slugifier = SlugifierBuilder::new()
    ///     .with_max_length(10)
    ///     .build();
    /// assert_eq!(slugifier.slugify("Very Long String"), "very-long");
    /// ```
    #[must_use]
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.options.max_length = Some(max_length);
        self
    }

    /// Set separator character.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugifierBuilder;
    ///
    /// let slugifier = SlugifierBuilder::new()
    ///     .with_separator('_')
    ///     .build();
    /// assert_eq!(slugifier.slugify("Hello World"), "hello_world");
    /// ```
    #[must_use]
    pub fn with_separator(mut self, separator: char) -> Self {
        self.options.separator = separator;
        self
    }

    /// Build the slugifier with configured options.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_slugify::SlugifierBuilder;
    ///
    /// let slugifier = SlugifierBuilder::new().build();
    /// let slug = slugifier.slugify("Test String");
    /// assert_eq!(slug, "test-string");
    /// ```
    #[must_use]
    pub fn build(self) -> Slugifier {
        Slugifier::new(self.options)
    }
}

/// Convert a string to a URL-safe slug with default options.
///
/// This is the main entry point for slugification. It converts the input
/// to lowercase, replaces whitespace and special characters with hyphens,
/// and collapses consecutive hyphens.
///
/// # Example
///
/// ```
/// use lintdiff_slugify::slugify;
///
/// assert_eq!(slugify("Hello World"), "hello-world");
/// assert_eq!(slugify("Test@123!"), "test-123");
/// assert_eq!(slugify("Multiple   Spaces"), "multiple-spaces");
/// ```
#[must_use]
pub fn slugify(s: &str) -> String {
    slugify_with_options(s, &SlugOptions::new())
}

/// Convert a string to a slug with custom options.
///
/// # Example
///
/// ```
/// use lintdiff_slugify::{slugify_with_options, SlugOptions};
///
/// let options = SlugOptions::new()
///     .with_separator('_')
///     .with_max_length(15);
///
/// assert_eq!(slugify_with_options("Hello World Test", &options), "hello_world");
/// ```
#[must_use]
pub fn slugify_with_options(s: &str, options: &SlugOptions) -> String {
    if s.is_empty() {
        return String::new();
    }

    let separator = options.separator;
    let mut result = String::with_capacity(s.len());

    // Process each character
    let mut last_was_separator = false;
    for c in s.chars() {
        // Only preserve ASCII alphanumeric characters for URL-safe slugs
        if c.is_ascii_alphanumeric() {
            // Always preserve alphanumeric
            let ch = if options.lowercase {
                c.to_ascii_lowercase()
            } else {
                c
            };
            result.push(ch);
            last_was_separator = false;
        } else if options.preserve_special && is_special_char(c) {
            // Preserve special characters when enabled
            let ch = if options.lowercase {
                c.to_ascii_lowercase()
            } else {
                c
            };
            result.push(ch);
            last_was_separator = false;
        } else if c.is_whitespace() {
            // Replace whitespace with separator (if not consecutive)
            if !last_was_separator {
                result.push(separator);
                last_was_separator = true;
            }
        } else {
            // Replace other characters with separator (if not consecutive)
            if !last_was_separator {
                result.push(separator);
                last_was_separator = true;
            }
        }
    }

    // Trim trailing separator
    if result.ends_with(separator) {
        result.pop();
    }

    // Trim leading separator
    if result.starts_with(separator) {
        result.remove(0);
    }

    // Apply max length if specified
    if let Some(max_len) = options.max_length {
        if result.len() > max_len {
            // Try to truncate at a separator boundary
            truncate_at_boundary(&mut result, separator, max_len);
        }
    }

    result
}

/// Truncate string at a word boundary if possible.
fn truncate_at_boundary(s: &mut String, separator: char, max_len: usize) {
    if s.len() <= max_len {
        return;
    }

    // Find the byte position corresponding to max_len characters
    // or the last valid char boundary before max_len bytes
    let byte_pos = s
        .char_indices()
        .take_while(|(i, _)| *i < max_len)
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(0);

    // Find the last separator before the byte position
    let mut truncate_byte_pos = byte_pos;
    for (i, c) in s.char_indices() {
        if i >= byte_pos {
            break;
        }
        if i > 0 && c == separator {
            truncate_byte_pos = i;
        }
    }

    // If we found a good boundary, use it
    if truncate_byte_pos > 0 && truncate_byte_pos <= byte_pos {
        s.truncate(truncate_byte_pos);
        // Remove trailing separator
        if s.ends_with(separator) {
            s.pop();
        }
    } else {
        // Just truncate at the calculated byte position
        s.truncate(byte_pos);
        // Remove trailing separator if present
        if s.ends_with(separator) {
            s.pop();
        }
    }
}

/// Slugify using Cow for zero-copy when possible.
///
/// Returns a borrowed reference when no slugification is needed.
///
/// # Example
///
/// ```
/// use lintdiff_slugify::slugify_cow;
/// use std::borrow::Cow;
///
/// let result = slugify_cow("already-slug");
/// assert!(matches!(result, Cow::Borrowed(_)));
///
/// let result = slugify_cow("Needs Conversion");
/// assert!(matches!(result, Cow::Owned(_)));
/// ```
#[must_use]
pub fn slugify_cow(s: &str) -> Cow<'_, str> {
    // Check if slugification is needed
    let needs_conversion = s.chars().any(|c| {
        c.is_whitespace()
            || (!c.is_alphanumeric() && c != '-')
            || c.is_uppercase()
    });

    if !needs_conversion {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(slugify(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("hello"), "hello");
        assert_eq!(slugify("HELLO"), "hello");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn test_special_chars() {
        assert_eq!(slugify("Test@123!"), "test-123");
        assert_eq!(slugify("foo@bar.com"), "foo-bar-com");
    }

    #[test]
    fn test_multiple_spaces() {
        assert_eq!(slugify("Multiple   Spaces"), "multiple-spaces");
        assert_eq!(slugify("  Leading"), "leading");
        assert_eq!(slugify("Trailing  "), "trailing");
    }

    #[test]
    fn test_preserve_special() {
        let options = SlugOptions::new().with_preserve_special(true);
        assert_eq!(slugify_with_options("Test@123", &options), "test@123");
        assert_eq!(slugify_with_options("Special#Char", &options), "special#char");
    }

    #[test]
    fn test_custom_separator() {
        let options = SlugOptions::new().with_separator('_');
        assert_eq!(slugify_with_options("Hello World", &options), "hello_world");
    }

    #[test]
    fn test_max_length() {
        let options = SlugOptions::new().with_max_length(10);
        assert_eq!(slugify_with_options("Very Long String", &options), "very-long");
    }

    #[test]
    fn test_no_lowercase() {
        let options = SlugOptions::new().with_lowercase(false);
        assert_eq!(slugify_with_options("Hello World", &options), "Hello-World");
    }

    #[test]
    fn test_slugifier_builder() {
        let slugifier = SlugifierBuilder::new()
            .with_separator('_')
            .with_lowercase(true)
            .build();
        assert_eq!(slugifier.slugify("Hello World"), "hello_world");
    }

    #[test]
    fn test_slugify_cow_borrowed() {
        use std::borrow::Cow;
        let result = slugify_cow("already-slug");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_slugify_cow_owned() {
        use std::borrow::Cow;
        let result = slugify_cow("Needs Conversion");
        assert!(matches!(result, Cow::Owned(_)));
    }
}
