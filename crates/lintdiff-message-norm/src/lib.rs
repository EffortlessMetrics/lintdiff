//! Diagnostic message normalization for lintdiff.
//!
//! Provides utilities for normalizing, cleaning, and truncating
//! diagnostic messages for consistent display.

use std::borrow::Cow;

/// Normalize a diagnostic message.
///
/// This function:
/// 1. Trims leading/trailing whitespace
/// 2. Normalizes internal whitespace (multiple spaces -> single space)
/// 3. Normalizes line endings (CRLF -> LF)
/// 4. Removes control characters (except newlines)
///
/// # Examples
/// ```
/// use lintdiff_message_norm::normalize;
///
/// let msg = "  Hello   world  \r\n  ";
/// assert_eq!(normalize(msg), "Hello world");
/// ```
#[must_use]
pub fn normalize(message: &str) -> Cow<'_, str> {
    NormalizeConfig::default().normalize(message)
}

/// Normalize and return an owned String.
#[must_use]
pub fn normalize_owned(message: &str) -> String {
    normalize(message).into_owned()
}

/// Configuration for message normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::struct_excessive_bools)]
pub struct NormalizeConfig {
    /// Trim leading/trailing whitespace.
    pub trim: bool,
    /// Normalize internal whitespace.
    pub collapse_whitespace: bool,
    /// Normalize line endings (CRLF -> LF).
    pub normalize_line_endings: bool,
    /// Remove control characters.
    pub remove_control_chars: bool,
    /// Maximum message length (0 = unlimited).
    pub max_length: usize,
    /// Truncation suffix (e.g., "...").
    pub truncation_suffix: String,
}

impl Default for NormalizeConfig {
    fn default() -> Self {
        Self {
            trim: true,
            collapse_whitespace: true,
            normalize_line_endings: true,
            remove_control_chars: true,
            max_length: 0,
            truncation_suffix: "...".to_string(),
        }
    }
}

impl NormalizeConfig {
    /// Create a new config with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable all normalization (pass through).
    #[must_use]
    pub const fn none() -> Self {
        Self {
            trim: false,
            collapse_whitespace: false,
            normalize_line_endings: false,
            remove_control_chars: false,
            max_length: 0,
            truncation_suffix: String::new(),
        }
    }

    /// Set max length.
    #[must_use]
    pub const fn with_max_length(mut self, len: usize) -> Self {
        self.max_length = len;
        self
    }

    /// Set truncation suffix.
    #[must_use]
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.truncation_suffix = suffix.into();
        self
    }

    /// Disable trimming.
    #[must_use]
    pub const fn no_trim(mut self) -> Self {
        self.trim = false;
        self
    }

    /// Disable whitespace collapsing.
    #[must_use]
    pub const fn no_collapse(mut self) -> Self {
        self.collapse_whitespace = false;
        self
    }

    /// Disable line ending normalization.
    #[must_use]
    pub const fn no_line_ending_norm(mut self) -> Self {
        self.normalize_line_endings = false;
        self
    }

    /// Disable control character removal.
    #[must_use]
    pub const fn keep_control_chars(mut self) -> Self {
        self.remove_control_chars = false;
        self
    }

    /// Normalize a message with this configuration.
    #[must_use]
    pub fn normalize<'a>(&self, message: &'a str) -> Cow<'a, str> {
        let mut result = Cow::Borrowed(message);

        // Step 1: Normalize line endings (CRLF -> LF)
        if self.normalize_line_endings && result.contains("\r\n") {
            result = Cow::Owned(result.replace("\r\n", "\n"));
        }

        // Step 2: Remove control characters (except newlines and tabs)
        if self.remove_control_chars {
            let has_control_chars = result
                .chars()
                .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t');
            if has_control_chars {
                let cleaned: String = result
                    .chars()
                    .filter(|&c| !c.is_control() || c == '\n' || c == '\r' || c == '\t')
                    .collect();
                result = Cow::Owned(cleaned);
            }
        }

        // Step 3: Collapse internal whitespace
        if self.collapse_whitespace {
            let has_multiple_spaces =
                result.contains("  ") || result.contains("\t ") || result.contains("\t\t");
            if has_multiple_spaces {
                let mut collapsed = String::with_capacity(result.len());
                let mut prev_was_space = false;
                for c in result.chars() {
                    let is_space = c == ' ' || c == '\t';
                    if is_space {
                        if !prev_was_space {
                            collapsed.push(' ');
                        }
                        prev_was_space = true;
                    } else {
                        collapsed.push(c);
                        prev_was_space = false;
                    }
                }
                result = Cow::Owned(collapsed);
            }
        }

        // Step 4: Trim leading/trailing whitespace
        if self.trim {
            let trimmed = result.trim();
            if trimmed.len() != result.len() {
                result = Cow::Owned(trimmed.to_string());
            }
        }

        // Step 5: Truncate if max_length is set
        if self.max_length > 0 && result.len() > self.max_length {
            let truncated = truncate(&result, self.max_length, &self.truncation_suffix);
            result = Cow::Owned(truncated.into_owned());
        }

        result
    }
}

/// Truncate a message to a maximum length.
///
/// # Examples
/// ```
/// use lintdiff_message_norm::truncate;
///
/// let msg = "This is a very long message";
/// assert_eq!(truncate(msg, 10, "..."), "This is...");
/// ```
#[must_use]
pub fn truncate<'a>(message: &'a str, max_len: usize, suffix: &str) -> Cow<'a, str> {
    let msg_char_count = message.chars().count();
    if max_len == 0 || msg_char_count <= max_len {
        return Cow::Borrowed(message);
    }

    let suffix_char_count = suffix.chars().count();
    if max_len <= suffix_char_count {
        // If max_len is too small to fit any content, just return suffix truncated
        let suffix_truncated: String = suffix.chars().take(max_len).collect();
        return Cow::Owned(suffix_truncated);
    }

    let target_content_len = max_len - suffix_char_count;

    // Find byte boundary for target content length
    let byte_index = message
        .char_indices()
        .nth(target_content_len)
        .map_or(message.len(), |(i, _)| i);

    let truncated = format!("{}{}", &message[..byte_index], suffix);
    Cow::Owned(truncated)
}

/// Truncate at a word boundary when possible.
///
/// # Examples
/// ```
/// use lintdiff_message_norm::truncate_at_word;
///
/// let msg = "This is a long message";
/// assert_eq!(truncate_at_word(msg, 10, "..."), "This...");
/// ```
#[must_use]
pub fn truncate_at_word<'a>(message: &'a str, max_len: usize, suffix: &str) -> Cow<'a, str> {
    let msg_char_count = message.chars().count();
    if max_len == 0 || msg_char_count <= max_len {
        return Cow::Borrowed(message);
    }

    let suffix_char_count = suffix.chars().count();
    if max_len <= suffix_char_count {
        let suffix_truncated: String = suffix.chars().take(max_len).collect();
        return Cow::Owned(suffix_truncated);
    }

    let target_content_len = max_len - suffix_char_count;

    // Find byte boundary for target content length
    let byte_index = message
        .char_indices()
        .nth(target_content_len)
        .map_or(message.len(), |(i, _)| i);

    // Look for a word boundary (space) before the truncation point
    let search_str = &message[..byte_index];
    let truncate_at = search_str.rfind(' ').map_or(byte_index, |space_pos| {
        // Only use word boundary if it doesn't make the result too short
        // (at least half of target length)
        let min_len = target_content_len / 2;
        if space_pos >= min_len {
            space_pos
        } else {
            byte_index
        }
    });

    let truncated = format!("{}{}", &message[..truncate_at], suffix);
    Cow::Owned(truncated)
}

/// Remove ANSI escape sequences from a message.
///
/// # Examples
/// ```
/// use lintdiff_message_norm::strip_ansi;
///
/// let msg = "\x1b[31mError\x1b[0m: something went wrong";
/// assert_eq!(strip_ansi(msg), "Error: something went wrong");
/// ```
#[must_use]
pub fn strip_ansi(message: &str) -> Cow<'_, str> {
    if !has_ansi(message) {
        return Cow::Borrowed(message);
    }

    let mut result = String::with_capacity(message.len());
    let chars: Vec<char> = message.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\x1b' {
            // Start of escape sequence
            if i + 1 < chars.len() && chars[i + 1] == '[' {
                // CSI sequence: ESC [ ... <letter>
                i += 2; // Skip ESC [
                while i < chars.len() {
                    let c = chars[i];
                    i += 1;
                    // CSI sequences end with a letter (0x40-0x7E)
                    if c.is_ascii_alphabetic() || ('@'..='~').contains(&c) {
                        break;
                    }
                }
            } else if i + 1 < chars.len() && chars[i + 1] == ']' {
                // OSC sequence: ESC ] ... BEL or ESC ] ... ST
                i += 2; // Skip ESC ]
                while i < chars.len() {
                    let c = chars[i];
                    i += 1;
                    if c == '\x07' {
                        // BEL
                        break;
                    }
                    if c == '\x1b' && i < chars.len() && chars[i] == '\\' {
                        // ST (String Terminator): ESC \
                        i += 1;
                        break;
                    }
                }
            } else if i + 1 < chars.len() && (chars[i + 1] == '(' || chars[i + 1] == ')') {
                // Character set designation: ESC ( <char> or ESC ) <char>
                i += 3; // Skip ESC ( or ) and the following char
            } else {
                // Other escape sequences: ESC <char>
                i += 2;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    Cow::Owned(result)
}

/// Check if a message contains ANSI escape sequences.
#[must_use]
pub fn has_ansi(message: &str) -> bool {
    message.contains('\x1b')
}

/// Escape a message for a specific output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeFormat {
    /// JSON string.
    Json,
    /// HTML content.
    Html,
    /// Shell argument.
    Shell,
    /// Markdown (escape special chars).
    Markdown,
}

/// Escape a message for the given format.
///
/// # Examples
/// ```
/// use lintdiff_message_norm::{escape, EscapeFormat};
///
/// let msg = r#"He said "hello""#;
/// assert_eq!(escape(msg, EscapeFormat::Json), r#"He said \"hello\""#);
/// ```
#[must_use]
pub fn escape(message: &str, format: EscapeFormat) -> Cow<'_, str> {
    match format {
        EscapeFormat::Json => escape_json(message),
        EscapeFormat::Html => escape_html(message),
        EscapeFormat::Shell => escape_shell(message),
        EscapeFormat::Markdown => escape_markdown(message),
    }
}

/// Unescape a message from a specific format.
#[must_use]
pub fn unescape(message: &str, format: EscapeFormat) -> Cow<'_, str> {
    match format {
        EscapeFormat::Json => unescape_json(message),
        EscapeFormat::Html => unescape_html(message),
        EscapeFormat::Shell => unescape_shell(message),
        EscapeFormat::Markdown => unescape_markdown(message),
    }
}

fn escape_json(message: &str) -> Cow<'_, str> {
    let needs_escape = message
        .chars()
        .any(|c| matches!(c, '"' | '\\' | '\n' | '\r' | '\t' | '\x08' | '\x0c'));

    if !needs_escape {
        return Cow::Borrowed(message);
    }

    let mut result = String::with_capacity(message.len() + message.len() / 4);
    for c in message.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\x08' => result.push_str("\\b"),
            '\x0c' => result.push_str("\\f"),
            c => result.push(c),
        }
    }
    Cow::Owned(result)
}

fn unescape_json(message: &str) -> Cow<'_, str> {
    if !message.contains('\\') {
        return Cow::Borrowed(message);
    }

    let mut result = String::with_capacity(message.len());
    let mut chars = message.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('b') => result.push('\x08'),
                Some('f') => result.push('\x0c'),
                Some('u') => {
                    // Unicode escape: \uXXXX
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        } else {
                            result.push_str("\\u");
                            result.push_str(&hex);
                        }
                    } else {
                        result.push_str("\\u");
                        result.push_str(&hex);
                    }
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => {
                    result.push('\\');
                }
            }
        } else {
            result.push(c);
        }
    }
    Cow::Owned(result)
}

// HTML entity constants (using \x26 for & to avoid encoding issues)
const HTML_AMP: &str = "\x26amp;";
const HTML_LT: &str = "\x26lt;";
const HTML_GT: &str = "\x26gt;";
const HTML_QUOT: &str = "\x26quot;";
const HTML_APOS: &str = "\x26#39;";

fn escape_html(message: &str) -> Cow<'_, str> {
    let needs_escape = message
        .chars()
        .any(|c| matches!(c, '&' | '<' | '>' | '"' | '\''));

    if !needs_escape {
        return Cow::Borrowed(message);
    }

    let mut result = String::with_capacity(message.len() + message.len() / 4);
    for c in message.chars() {
        match c {
            '&' => result.push_str(HTML_AMP),
            '<' => result.push_str(HTML_LT),
            '>' => result.push_str(HTML_GT),
            '"' => result.push_str(HTML_QUOT),
            '\'' => result.push_str(HTML_APOS),
            c => result.push(c),
        }
    }
    Cow::Owned(result)
}

fn unescape_html(message: &str) -> Cow<'_, str> {
    if !message.contains('&') {
        return Cow::Borrowed(message);
    }

    let mut result = String::with_capacity(message.len());
    let chars: Vec<char> = message.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '&' {
            // Look for semicolon
            let mut semicolon_pos = None;
            for (j, &ch) in chars
                .iter()
                .enumerate()
                .take(chars.len().min(i + 12))
                .skip(i + 1)
            {
                if ch == ';' {
                    semicolon_pos = Some(j);
                    break;
                }
                if !ch.is_ascii_alphanumeric() && ch != '#' {
                    break;
                }
            }

            if let Some(semi) = semicolon_pos {
                let entity: String = chars[i..=semi].iter().collect();
                let unescaped = match entity.as_str() {
                    "\x26amp;" => "\x26",
                    "\x26lt;" => "<",
                    "\x26gt;" => ">",
                    "\x26quot;" => "\"",
                    "\x26apos;" | "\x26#39;" => "'",
                    "\x26nbsp;" => "\u{00A0}",
                    _ => {
                        // Check for numeric entity
                        if let Some(hex) = entity.strip_prefix("&#x") {
                            if let Ok(code) = u32::from_str_radix(hex.trim_end_matches(';'), 16) {
                                if let Some(ch) = char::from_u32(code) {
                                    result.push(ch);
                                    i = semi + 1;
                                    continue;
                                }
                            }
                        } else if let Some(dec) = entity.strip_prefix("&#") {
                            if let Ok(code) = dec.trim_end_matches(';').parse::<u32>() {
                                if let Some(ch) = char::from_u32(code) {
                                    result.push(ch);
                                    i = semi + 1;
                                    continue;
                                }
                            }
                        }
                        &entity
                    }
                };
                result.push_str(unescaped);
                i = semi + 1;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    Cow::Owned(result)
}

fn escape_shell(message: &str) -> Cow<'_, str> {
    // Check if we need to escape anything
    let needs_escape = message.chars().any(|c| {
        c.is_whitespace()
            || matches!(
                c,
                '"' | '\\' | '$' | '`' | '!' | '*' | '?' | '[' | ']' | '(' | ')' | '{' | '}'
            )
    });

    if !needs_escape {
        return Cow::Borrowed(message);
    }

    // Wrap in single quotes and escape any single quotes in the message
    let mut result = String::with_capacity(message.len() + 2);
    result.push('\'');
    for c in message.chars() {
        if c == '\'' {
            result.push_str("'\"'\"'");
        } else {
            result.push(c);
        }
    }
    result.push('\'');
    Cow::Owned(result)
}

fn unescape_shell(message: &str) -> Cow<'_, str> {
    // Handle single-quoted strings
    if message.starts_with('\'') && message.ends_with('\'') && message.len() >= 2 {
        let inner = &message[1..message.len() - 1];
        // Single quotes in shell are literal, except for the special "'\"'\"' pattern
        let result = inner.replace("'\"'\"'", "'");
        return Cow::Owned(result);
    }

    // Handle double-quoted strings
    if message.starts_with('"') && message.ends_with('"') && message.len() >= 2 {
        let inner = &message[1..message.len() - 1];
        let mut result = String::with_capacity(inner.len());
        let mut chars = inner.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.peek().copied() {
                    Some('"') => {
                        result.push('"');
                        chars.next();
                    }
                    Some('\\') => {
                        result.push('\\');
                        chars.next();
                    }
                    Some('$') => {
                        result.push('$');
                        chars.next();
                    }
                    Some('`') => {
                        result.push('`');
                        chars.next();
                    }
                    Some('\n') => {
                        // Line continuation
                        chars.next();
                    }
                    Some('n') => {
                        result.push('\n');
                        chars.next();
                    }
                    Some('t') => {
                        result.push('\t');
                        chars.next();
                    }
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                        chars.next();
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }
        return Cow::Owned(result);
    }

    // Not a quoted string, return as-is
    Cow::Borrowed(message)
}

fn escape_markdown(message: &str) -> Cow<'_, str> {
    // Characters that have special meaning in Markdown
    let needs_escape = message.chars().any(|c| {
        matches!(
            c,
            '\\' | '`'
                | '*'
                | '_'
                | '{'
                | '}'
                | '['
                | ']'
                | '('
                | ')'
                | '#'
                | '+'
                | '-'
                | '.'
                | '!'
                | '|'
                | '~'
                | '>'
        )
    });

    if !needs_escape {
        return Cow::Borrowed(message);
    }

    let mut result = String::with_capacity(message.len() + message.len() / 4);
    for c in message.chars() {
        match c {
            '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.'
            | '!' | '|' | '~' | '>' => {
                result.push('\\');
                result.push(c);
            }
            c => result.push(c),
        }
    }
    Cow::Owned(result)
}

fn unescape_markdown(message: &str) -> Cow<'_, str> {
    if !message.contains('\\') {
        return Cow::Borrowed(message);
    }

    let mut result = String::with_capacity(message.len());
    let mut chars = message.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                // Unescape known markdown characters
                if matches!(
                    next,
                    '\\' | '`'
                        | '*'
                        | '_'
                        | '{'
                        | '}'
                        | '['
                        | ']'
                        | '('
                        | ')'
                        | '#'
                        | '+'
                        | '-'
                        | '.'
                        | '!'
                        | '|'
                        | '~'
                        | '>'
                ) {
                    result.push(next);
                    chars.next();
                    continue;
                }
            }
            result.push('\\');
        } else {
            result.push(c);
        }
    }
    Cow::Owned(result)
}

/// A normalized message wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NormalizedMessage(String);

impl NormalizedMessage {
    /// Create a new normalized message.
    #[must_use]
    pub fn new(message: &str) -> Self {
        Self(normalize_owned(message))
    }

    /// Create with custom config.
    #[must_use]
    pub fn with_config(message: &str, config: &NormalizeConfig) -> Self {
        Self(config.normalize(message).into_owned())
    }

    /// Get the normalized message.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the length.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Truncate in place.
    pub fn truncate(&mut self, max_len: usize, suffix: &str) {
        let truncated = truncate(&self.0, max_len, suffix);
        self.0 = truncated.into_owned();
    }

    /// Convert to inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for NormalizedMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for NormalizedMessage {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&str> for NormalizedMessage {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

impl From<String> for NormalizedMessage {
    fn from(message: String) -> Self {
        Self::new(&message)
    }
}

impl From<NormalizedMessage> for String {
    fn from(msg: NormalizedMessage) -> Self {
        msg.into_inner()
    }
}

impl Default for NormalizedMessage {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_basic() {
        assert_eq!(normalize("hello"), "hello");
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize("  hello  "), "hello");
        assert_eq!(normalize("hello   world"), "hello world");
    }

    #[test]
    fn test_normalize_crlf() {
        assert_eq!(normalize("hello\r\nworld"), "hello\nworld");
    }

    #[test]
    fn test_normalize_control_chars() {
        assert_eq!(normalize("hello\x00world"), "helloworld");
        assert_eq!(normalize("hello\x1bworld"), "helloworld");
    }

    #[test]
    fn test_truncate_basic() {
        assert_eq!(truncate("hello world", 5, ""), "hello");
        assert_eq!(truncate("hello world", 5, "..."), "he...");
    }

    #[test]
    fn test_truncate_at_word_basic() {
        assert_eq!(truncate_at_word("hello world", 8, "..."), "hello...");
    }

    #[test]
    fn test_strip_ansi_basic() {
        assert_eq!(strip_ansi("hello"), "hello");
        assert_eq!(strip_ansi("\x1b[31mhello\x1b[0m"), "hello");
    }

    #[test]
    fn test_escape_json_basic() {
        assert_eq!(escape("hello", EscapeFormat::Json), "hello");
        assert_eq!(escape("say \"hi\"", EscapeFormat::Json), "say \\\"hi\\\"");
    }

    #[test]
    fn test_escape_html_basic() {
        assert_eq!(escape("hello", EscapeFormat::Html), "hello");
        assert_eq!(escape("<div>", EscapeFormat::Html), "\x26lt;div\x26gt;");
    }

    #[test]
    fn test_normalized_message() {
        let msg = NormalizedMessage::new("  hello  ");
        assert_eq!(msg.as_str(), "hello");
    }
}
