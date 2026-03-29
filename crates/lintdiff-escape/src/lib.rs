//! Output format escaping utilities for lintdiff.
//!
//! This microcrate provides a single responsibility: escaping strings for various
//! output formats, ensuring safe rendering in different contexts.
//!
//! # Supported Formats
//!
//! - **GitHub Actions**: Escapes percent, carriage return, newline, and colon
//! - **Markdown**: Escapes asterisk, underscore, brackets, angle brackets, backtick, hash
//! - **HTML**: Escapes angle brackets, ampersand, and quotes
//! - **JSON**: Escapes double-quote, backslash, and control characters
//! - **Plain Text**: Minimal escaping (pass-through)
//!
//! # Example: Format-Specific Escaping
//!
//! ```
//! use lintdiff_escape::{escape, OutputFormat};
//!
//! let input = "Error: 100% complete\nDone";
//! let escaped = escape(input, OutputFormat::GitHubActions);
//! assert_eq!(escaped, "Error%3A 100%25 complete%0ADone");
//! ```
//!
//! # Example: Direct Function Calls
//!
//! ```
//! use lintdiff_escape::escape_html;
//!
//! let html = escape_html("<script>alert('xss')</script>");
//! assert!(html.contains("lt;"));
//! assert!(html.contains("gt;"));
//! ```
//!
//! # Zero-Copy Optimization
//!
//! All escape functions return `Cow<str>` to avoid allocations when no escaping
//! is needed:
//!
//! ```
//! use lintdiff_escape::{escape, OutputFormat};
//! use std::borrow::Cow;
//!
//! let input = "no special chars";
//! let escaped = escape(input, OutputFormat::PlainText);
//! // No allocation occurred - returns borrowed reference
//! assert!(matches!(escaped, Cow::Borrowed(_)));
//! ```

#![warn(missing_docs)]

use std::borrow::Cow;

/// Output format for escaping.
///
/// Specifies the target format for string escaping. Each format has specific
/// characters that need to be escaped for safe rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputFormat {
    /// GitHub Actions workflow commands.
    ///
    /// Escapes: percent, carriage return, newline, colon
    GitHubActions,
    /// Markdown content.
    ///
    /// Escapes: asterisk, underscore, brackets, angle brackets, backtick, hash
    Markdown,
    /// Plain text (terminal).
    ///
    /// No escaping is performed - pass-through.
    PlainText,
    /// HTML content.
    ///
    /// Escapes: angle brackets, ampersand, quotes
    Html,
    /// JSON string content.
    ///
    /// Escapes: double-quote, backslash, control characters (U+0000 through U+001F)
    Json,
}

/// Escape a string for the given output format.
///
/// This is the main entry point for escaping strings. It dispatches to the
/// appropriate format-specific escape function.
///
/// # Example
///
/// ```
/// use lintdiff_escape::{escape, OutputFormat};
///
/// let markdown = escape("Hello *world*", OutputFormat::Markdown);
/// assert_eq!(markdown, "Hello \\*world\\*");
/// ```
///
/// # Zero-Copy
///
/// Returns `Cow::Borrowed` when no escaping is needed:
///
/// ```
/// use lintdiff_escape::{escape, OutputFormat};
/// use std::borrow::Cow;
///
/// let plain = escape("hello world", OutputFormat::PlainText);
/// assert!(matches!(plain, Cow::Borrowed(_)));
/// ```
#[must_use]
pub fn escape(s: &str, format: OutputFormat) -> Cow<'_, str> {
    match format {
        OutputFormat::GitHubActions => escape_github(s),
        OutputFormat::Markdown => escape_markdown(s),
        OutputFormat::PlainText => escape_plain(s),
        OutputFormat::Html => escape_html(s),
        OutputFormat::Json => escape_json(s),
    }
}

/// Escape for GitHub Actions workflow commands.
///
/// GitHub Actions uses percent-encoding for special characters in workflow commands.
/// The following characters are escaped:
///
/// - percent sign becomes `%25`
/// - carriage return becomes `%0D`
/// - newline becomes `%0A`
/// - colon becomes `%3A`
///
/// # Example
///
/// ```
/// use lintdiff_escape::escape_github;
///
/// let input = "Error: 50%\nLine2";
/// let escaped = escape_github(input);
/// assert_eq!(escaped, "Error%3A 50%25%0ALine2");
/// ```
///
/// # References
///
/// - [GitHub Actions Workflow Commands](https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions)
#[must_use]
pub fn escape_github(s: &str) -> Cow<'_, str> {
    if !needs_github_escaping(s) {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len() + s.len() / 4);
    for c in s.chars() {
        match c {
            '%' => result.push_str("%25"),
            '\r' => result.push_str("%0D"),
            '\n' => result.push_str("%0A"),
            ':' => result.push_str("%3A"),
            _ => result.push(c),
        }
    }
    Cow::Owned(result)
}

/// Escape for Markdown content.
///
/// Escapes special Markdown characters that could affect formatting:
///
/// - asterisk becomes backslash-asterisk
/// - underscore becomes backslash-underscore
/// - left bracket becomes backslash-left-bracket
/// - right bracket becomes backslash-right-bracket
/// - less-than becomes backslash-less-than
/// - greater-than becomes backslash-greater-than
/// - backtick becomes backslash-backtick
/// - hash becomes backslash-hash
///
/// # Example
///
/// ```
/// use lintdiff_escape::escape_markdown;
///
/// let input = "**bold** and `code`";
/// let escaped = escape_markdown(input);
/// assert_eq!(escaped, "\\*\\*bold\\*\\* and \\`code\\`");
/// ```
///
/// # Note
///
/// This escapes characters that have special meaning in Markdown. For inline code
/// blocks or code fences, you may want different escaping.
#[must_use]
pub fn escape_markdown(s: &str) -> Cow<'_, str> {
    if !needs_markdown_escaping(s) {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len() + s.len() / 4);
    for c in s.chars() {
        match c {
            '*' => result.push_str("\\*"),
            '_' => result.push_str("\\_"),
            '[' => result.push_str("\\["),
            ']' => result.push_str("\\]"),
            '<' => result.push_str("\\<"),
            '>' => result.push_str("\\>"),
            '`' => result.push_str("\\`"),
            '#' => result.push_str("\\#"),
            _ => result.push(c),
        }
    }
    Cow::Owned(result)
}

/// Escape for plain text (minimal escaping).
///
/// Performs no escaping - returns the input string unchanged.
/// This is useful for terminal output or when no escaping is needed.
///
/// # Example
///
/// ```
/// use lintdiff_escape::escape_plain;
/// use std::borrow::Cow;
///
/// let input = "Hello, world!";
/// let escaped = escape_plain(input);
/// assert!(matches!(escaped, Cow::Borrowed("Hello, world!")));
/// ```
#[must_use]
pub fn escape_plain(s: &str) -> Cow<'_, str> {
    Cow::Borrowed(s)
}

/// Escape for HTML content.
///
/// Escapes characters that have special meaning in HTML:
///
/// - ampersand becomes `&`
/// - less-than becomes `<`
/// - greater-than becomes `>`
/// - double-quote becomes `"`
/// - single-quote becomes `&#x27;`
///
/// # Example
///
/// ```
/// use lintdiff_escape::escape_html;
///
/// let input = "<div>Hello & goodbye</div>";
/// let escaped = escape_html(input);
/// assert!(escaped.contains("lt;"));
/// assert!(escaped.contains("gt;"));
/// assert!(escaped.contains("amp;"));
/// ```
///
/// # Security
///
/// This function helps prevent XSS attacks when inserting user content into HTML.
/// Always escape user-provided content before rendering in HTML context.
#[must_use]
pub fn escape_html(s: &str) -> Cow<'_, str> {
    if !needs_html_escaping(s) {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len() + s.len() / 4);
    for c in s.chars() {
        match c {
            '&' => result.push_str("\x26amp;"),
            '<' => result.push_str("\x26lt;"),
            '>' => result.push_str("\x26gt;"),
            '"' => result.push_str("\x26quot;"),
            '\'' => result.push_str("\x26#x27;"),
            _ => result.push(c),
        }
    }
    Cow::Owned(result)
}

/// Escape for JSON strings.
///
/// Escapes characters that have special meaning in JSON strings:
///
/// - double-quote becomes backslash-double-quote
/// - backslash becomes double-backslash
/// - newline becomes backslash-n
/// - carriage return becomes backslash-r
/// - tab becomes backslash-t
/// - Other control characters become backslash-u followed by hex code
///
/// # Example
///
/// ```
/// use lintdiff_escape::escape_json;
///
/// let input = r#"He said "hi""#;
/// let escaped = escape_json(input);
/// assert_eq!(escaped, r#"He said \"hi\""#);
/// ```
///
/// # Note
///
/// This function escapes content for embedding in JSON string literals.
/// It does not produce a complete JSON string (no surrounding quotes).
#[must_use]
pub fn escape_json(s: &str) -> Cow<'_, str> {
    if !needs_json_escaping(s) {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len() + s.len() / 4);
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => result.push(c),
        }
    }
    Cow::Owned(result)
}

/// Check if escaping is needed for the given format.
///
/// Returns `true` if the string contains characters that would be escaped
/// for the specified format.
///
/// # Example
///
/// ```
/// use lintdiff_escape::{needs_escaping, OutputFormat};
///
/// assert!(needs_escaping("<script>", OutputFormat::Html));
/// assert!(!needs_escaping("hello world", OutputFormat::Html));
/// assert!(!needs_escaping("anything", OutputFormat::PlainText));
/// ```
#[must_use]
pub fn needs_escaping(s: &str, format: OutputFormat) -> bool {
    match format {
        OutputFormat::GitHubActions => needs_github_escaping(s),
        OutputFormat::Markdown => needs_markdown_escaping(s),
        OutputFormat::PlainText => false,
        OutputFormat::Html => needs_html_escaping(s),
        OutputFormat::Json => needs_json_escaping(s),
    }
}

fn needs_github_escaping(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '%' | '\r' | '\n' | ':'))
}

fn needs_markdown_escaping(s: &str) -> bool {
    s.chars()
        .any(|c| matches!(c, '*' | '_' | '[' | ']' | '<' | '>' | '`' | '#'))
}

fn needs_html_escaping(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '&' | '<' | '>' | '"' | '\''))
}

fn needs_json_escaping(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '"' | '\\') || c.is_control())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_github_basic() {
        assert_eq!(escape_github("hello"), "hello");
        assert_eq!(escape_github("100%"), "100%25");
        assert_eq!(escape_github("Error: test"), "Error%3A test");
    }

    #[test]
    fn test_escape_github_newlines() {
        assert_eq!(escape_github("line1\nline2"), "line1%0Aline2");
        assert_eq!(escape_github("line1\r\nline2"), "line1%0D%0Aline2");
    }

    #[test]
    fn test_escape_markdown_basic() {
        assert_eq!(escape_markdown("hello"), "hello");
        assert_eq!(escape_markdown("*bold*"), "\\*bold\\*");
        assert_eq!(escape_markdown("_italic_"), "\\_italic\\_");
    }

    #[test]
    fn test_escape_html_basic() {
        assert_eq!(escape_html("hello"), "hello");
        assert_eq!(escape_html("<div>"), "\x26lt;div\x26gt;");
        assert_eq!(escape_html("a \x26 b"), "a \x26amp; b");
    }

    #[test]
    fn test_escape_json_basic() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_json("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_needs_escaping() {
        assert!(needs_escaping("100%", OutputFormat::GitHubActions));
        assert!(!needs_escaping("hello", OutputFormat::GitHubActions));
        assert!(!needs_escaping("anything", OutputFormat::PlainText));
    }

    #[test]
    fn test_escape_dispatch() {
        assert_eq!(escape("100%", OutputFormat::GitHubActions), "100%25");
        assert_eq!(escape("*bold*", OutputFormat::Markdown), "\\*bold\\*");
        assert_eq!(escape("hello", OutputFormat::PlainText), "hello");
    }
}
