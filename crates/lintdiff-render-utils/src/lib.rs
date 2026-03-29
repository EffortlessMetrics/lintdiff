//! Rendering utilities for lintdiff.
//!
//! Provides shared utilities for output rendering across
//! different formats (markdown, annotations, plain text).

use std::fmt;

/// Indentation configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndentConfig {
    /// Number of spaces per indent level.
    pub spaces: usize,
    /// Character to use for indentation.
    pub char: char,
}

impl Default for IndentConfig {
    fn default() -> Self {
        Self {
            spaces: 2,
            char: ' ',
        }
    }
}

impl IndentConfig {
    /// Create a new indent config.
    #[must_use]
    pub fn new(spaces: usize) -> Self {
        Self {
            spaces,
            ..Self::default()
        }
    }

    /// Create an indent string for the given level.
    #[must_use]
    pub fn indent(&self, level: usize) -> String {
        self.char.to_string().repeat(self.spaces * level)
    }

    /// Indent a multi-line string.
    #[must_use]
    pub fn indent_string(&self, s: &str, level: usize) -> String {
        let indent = self.indent(level);
        s.lines()
            .map(|line| format!("{indent}{line}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Text wrapping configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapConfig {
    /// Maximum line width.
    pub width: usize,
    /// Initial indent (for first line).
    pub initial_indent: String,
    /// Subsequent indent (for wrapped lines).
    pub subsequent_indent: String,
    /// Break long words.
    pub break_long_words: bool,
}

impl Default for WrapConfig {
    fn default() -> Self {
        Self {
            width: 80,
            initial_indent: String::new(),
            subsequent_indent: String::new(),
            break_long_words: false,
        }
    }
}

impl WrapConfig {
    /// Create a new wrap config with the given width.
    #[must_use]
    pub fn new(width: usize) -> Self {
        Self {
            width,
            ..Self::default()
        }
    }

    /// Set initial indent.
    #[must_use]
    pub fn with_initial_indent(mut self, indent: impl Into<String>) -> Self {
        self.initial_indent = indent.into();
        self
    }

    /// Set subsequent indent.
    #[must_use]
    pub fn with_subsequent_indent(mut self, indent: impl Into<String>) -> Self {
        self.subsequent_indent = indent.into();
        self
    }

    /// Wrap text to the configured width.
    #[must_use]
    pub fn wrap(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }

        let mut result = String::new();
        let mut current_line = self.initial_indent.clone();
        let mut first_line = true;

        for word in text.split_whitespace() {
            let potential_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{current_line} {word}")
            };

            let effective_width = if first_line {
                self.width.saturating_sub(self.initial_indent.len())
            } else {
                self.width.saturating_sub(self.subsequent_indent.len())
            };

            if potential_line.len() <= effective_width {
                current_line = potential_line;
            } else {
                if !current_line.is_empty()
                    && current_line != self.initial_indent
                    && current_line != self.subsequent_indent
                {
                    if !result.is_empty() {
                        result.push('\n');
                    }
                    result.push_str(&current_line);
                }

                first_line = false;
                current_line = format!("{}{word}", self.subsequent_indent);
            }
        }

        if !current_line.is_empty()
            && current_line != self.initial_indent
            && current_line != self.subsequent_indent
        {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&current_line);
        }

        result
    }
}

/// Pluralization helper.
#[must_use]
pub fn pluralize(count: usize, singular: &str, plural: Option<&str>) -> String {
    if count == 1 {
        format!("{count} {singular}")
    } else {
        let default_plural = format!("{singular}s");
        let plural_form = plural.unwrap_or(&default_plural);
        format!("{count} {plural_form}")
    }
}

/// Create a bulleted list.
#[must_use]
pub fn bullet_list(items: &[&str], bullet: &str) -> String {
    items
        .iter()
        .map(|item| format!("{bullet} {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Create a numbered list.
#[must_use]
pub fn numbered_list(items: &[&str]) -> String {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| format!("{}. {item}", i + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Create a table.
#[must_use]
pub fn table(headers: &[&str], rows: &[Vec<&str>]) -> String {
    if headers.is_empty() {
        return String::new();
    }

    // Calculate column widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    // Build header
    let header_row: String = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!(" {:width$} ", h, width = widths.get(i).copied().unwrap_or(0)))
        .collect::<Vec<_>>()
        .join("|");

    let separator: String = widths
        .iter()
        .map(|&w| "-".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("+");

    // Build rows
    let data_rows: Vec<String> = rows
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(i, cell)| {
                    format!(" {:width$} ", cell, width = widths.get(i).copied().unwrap_or(0))
                })
                .collect::<Vec<_>>()
                .join("|")
        })
        .collect();

    let mut result = vec![header_row, separator];
    result.extend(data_rows);
    result.join("\n")
}

/// Truncate with ellipsis.
#[must_use]
pub fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        ".".repeat(max_len)
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Pad a string to a given width.
#[must_use]
pub fn pad_left(s: &str, width: usize, pad: char) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", pad.to_string().repeat(width - s.len()), s)
    }
}

/// Pad a string to a given width (right).
#[must_use]
pub fn pad_right(s: &str, width: usize, pad: char) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, pad.to_string().repeat(width - s.len()))
    }
}

/// Center a string in a given width.
#[must_use]
pub fn center(s: &str, width: usize, pad: char) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        let left = (width - s.len()) / 2;
        let right = width - s.len() - left;
        format!(
            "{}{}{}",
            pad.to_string().repeat(left),
            s,
            pad.to_string().repeat(right)
        )
    }
}

/// A text builder for constructing formatted output.
#[derive(Debug, Clone, Default)]
pub struct TextBuilder {
    lines: Vec<String>,
    indent_config: IndentConfig,
    current_indent: usize,
}

impl TextBuilder {
    /// Create a new text builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set indent config.
    #[must_use]
    pub const fn with_indent(mut self, config: IndentConfig) -> Self {
        self.indent_config = config;
        self
    }

    /// Push a line.
    pub fn line(&mut self, text: &str) -> &mut Self {
        let indent = self.indent_config.indent(self.current_indent);
        self.lines.push(format!("{indent}{text}"));
        self
    }

    /// Push an empty line.
    pub fn empty_line(&mut self) -> &mut Self {
        self.lines.push(String::new());
        self
    }

    /// Increase indent.
    #[allow(clippy::missing_const_for_fn)]
    pub fn indent(&mut self) -> &mut Self {
        self.current_indent += 1;
        self
    }

    /// Decrease indent.
    #[allow(clippy::missing_const_for_fn)]
    pub fn dedent(&mut self) -> &mut Self {
        if self.current_indent > 0 {
            self.current_indent -= 1;
        }
        self
    }

    /// Build the final string.
    #[must_use]
    pub fn build(&self) -> String {
        self.lines.join("\n")
    }
}

impl fmt::Display for TextBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_config_default() {
        let config = IndentConfig::default();
        assert_eq!(config.spaces, 2);
        assert_eq!(config.char, ' ');
    }

    #[test]
    fn test_indent_config_new() {
        let config = IndentConfig::new(4);
        assert_eq!(config.spaces, 4);
        assert_eq!(config.char, ' ');
    }

    #[test]
    fn test_indent_config_indent_level_0() {
        let config = IndentConfig::default();
        assert_eq!(config.indent(0), "");
    }

    #[test]
    fn test_indent_config_indent_level_1() {
        let config = IndentConfig::default();
        assert_eq!(config.indent(1), "  ");
    }

    #[test]
    fn test_indent_config_indent_level_2() {
        let config = IndentConfig::new(4);
        assert_eq!(config.indent(2), "        ");
    }

    #[test]
    fn test_indent_string_single_line() {
        let config = IndentConfig::default();
        assert_eq!(config.indent_string("hello", 1), "  hello");
    }

    #[test]
    fn test_indent_string_multi_line() {
        let config = IndentConfig::default();
        assert_eq!(
            config.indent_string("hello\nworld", 1),
            "  hello\n  world"
        );
    }

    #[test]
    fn test_wrap_config_default() {
        let config = WrapConfig::default();
        assert_eq!(config.width, 80);
        assert!(config.initial_indent.is_empty());
        assert!(config.subsequent_indent.is_empty());
        assert!(!config.break_long_words);
    }

    #[test]
    fn test_wrap_config_new() {
        let config = WrapConfig::new(40);
        assert_eq!(config.width, 40);
    }

    #[test]
    fn test_wrap_config_with_initial_indent() {
        let config = WrapConfig::new(40).with_initial_indent("  ");
        assert_eq!(config.initial_indent, "  ");
    }

    #[test]
    fn test_wrap_config_with_subsequent_indent() {
        let config = WrapConfig::new(40).with_subsequent_indent("    ");
        assert_eq!(config.subsequent_indent, "    ");
    }

    #[test]
    fn test_pluralize_singular() {
        assert_eq!(pluralize(1, "item", None), "1 item");
    }

    #[test]
    fn test_pluralize_plural_default() {
        assert_eq!(pluralize(2, "item", None), "2 items");
    }

    #[test]
    fn test_pluralize_plural_custom() {
        assert_eq!(pluralize(2, "child", Some("children")), "2 children");
    }

    #[test]
    fn test_pluralize_zero() {
        assert_eq!(pluralize(0, "item", None), "0 items");
    }

    #[test]
    fn test_bullet_list() {
        let items = vec!["one", "two", "three"];
        assert_eq!(bullet_list(&items, "-"), "- one\n- two\n- three");
    }

    #[test]
    fn test_bullet_list_empty() {
        let items: Vec<&str> = vec![];
        assert_eq!(bullet_list(&items, "-"), "");
    }

    #[test]
    fn test_numbered_list() {
        let items = vec!["one", "two", "three"];
        assert_eq!(numbered_list(&items), "1. one\n2. two\n3. three");
    }

    #[test]
    fn test_numbered_list_empty() {
        let items: Vec<&str> = vec![];
        assert_eq!(numbered_list(&items), "");
    }

    #[test]
    fn test_table_basic() {
        let headers = vec!["Name", "Age"];
        let rows = vec![vec!["Alice", "30"]];
        let result = table(&headers, &rows);
        assert!(result.contains("Name"));
        assert!(result.contains("Age"));
        assert!(result.contains("Alice"));
        assert!(result.contains("30"));
    }

    #[test]
    fn test_truncate_with_ellipsis_no_truncation() {
        assert_eq!(truncate_with_ellipsis("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_with_ellipsis_truncation() {
        assert_eq!(truncate_with_ellipsis("hello world", 8), "hello...");
    }

    #[test]
    fn test_truncate_with_ellipsis_short_max_len() {
        assert_eq!(truncate_with_ellipsis("hello", 2), "..");
    }

    #[test]
    fn test_pad_left() {
        assert_eq!(pad_left("42", 5, '0'), "00042");
    }

    #[test]
    fn test_pad_left_no_padding_needed() {
        assert_eq!(pad_left("hello", 3, ' '), "hello");
    }

    #[test]
    fn test_pad_right() {
        assert_eq!(pad_right("hello", 10, '.'), "hello.....");
    }

    #[test]
    fn test_pad_right_no_padding_needed() {
        assert_eq!(pad_right("hello", 3, ' '), "hello");
    }

    #[test]
    fn test_center() {
        assert_eq!(center("hi", 6, '-'), "--hi--");
    }

    #[test]
    fn test_center_odd_width() {
        assert_eq!(center("hi", 5, '-'), "-hi--");
    }

    #[test]
    fn test_center_no_padding_needed() {
        assert_eq!(center("hello", 3, ' '), "hello");
    }

    #[test]
    fn test_text_builder_new() {
        let builder = TextBuilder::new();
        assert!(builder.build().is_empty());
    }

    #[test]
    fn test_text_builder_line() {
        let mut builder = TextBuilder::new();
        builder.line("hello");
        assert_eq!(builder.build(), "hello");
    }

    #[test]
    fn test_text_builder_multiple_lines() {
        let mut builder = TextBuilder::new();
        builder.line("hello").line("world");
        assert_eq!(builder.build(), "hello\nworld");
    }

    #[test]
    fn test_text_builder_indent() {
        let mut builder = TextBuilder::new().with_indent(IndentConfig::new(2));
        builder.line("level 0").indent().line("level 1");
        assert_eq!(builder.build(), "level 0\n  level 1");
    }

    #[test]
    fn test_text_builder_empty_line() {
        let mut builder = TextBuilder::new();
        builder.line("hello").empty_line().line("world");
        assert_eq!(builder.build(), "hello\n\nworld");
    }
}
