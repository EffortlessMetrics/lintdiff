//! Builder for creating human-readable explain output in lintdiff.
//!
//! This microcrate provides a single responsibility: building formatted explain output
//! for diagnostic explanations in both markdown and plain text formats.
//!
//! # Features
//!
//! - Builder pattern for constructing explain output
//! - Support for sections, bullet points, code blocks, and tables
//! - Markdown and plain text output formats
//! - Optional ANSI color codes
//! - Configurable indentation and line width
//!
//! # Example
//!
//! ```
//! use lintdiff_explain_builder::{ExplainBuilder, ExplainConfig};
//!
//! // Create a simple explanation
//! let output = lintdiff_explain_builder::explain_simple(
//!     "Diagnostic Explanation",
//!     "This diagnostic was triggered by unused code."
//! );
//! assert!(output.contains("# Diagnostic Explanation"));
//!
//! // Use the builder for complex output
//! let mut builder = ExplainBuilder::new();
//! builder
//!     .with_title("Analysis Results")
//!     .with_summary("Found 3 issues in the codebase")
//!     .add_section("Details", "See below for more information")
//!     .add_bullet("Issue 1: Missing documentation")
//!     .add_bullet("Issue 2: Unused variable");
//!
//! let output = builder.build();
//! assert!(output.contains("# Analysis Results"));
//! ```

#![warn(missing_docs)]

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Section types for explain output.
///
/// Represents different kinds of content that can be added to an explanation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub enum ExplainSection {
    /// Plain text content.
    Text(String),
    /// Bullet list items.
    Bullets(Vec<String>),
    /// Code block with optional language.
    Code {
        /// The code content.
        code: String,
        /// The programming language for syntax highlighting.
        language: String,
    },
    /// Table with headers and rows.
    Table {
        /// Column headers.
        headers: Vec<String>,
        /// Table rows (each row is a vector of cell values).
        rows: Vec<Vec<String>>,
    },
    /// Section with heading and content.
    Section {
        /// Section heading.
        heading: String,
        /// Section content.
        content: String,
    },
}

impl ExplainSection {
    /// Create a new text section.
    #[must_use]
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    /// Create a new bullets section.
    #[must_use]
    pub const fn bullets(items: Vec<String>) -> Self {
        Self::Bullets(items)
    }

    /// Create a new code section.
    #[must_use]
    pub fn code(code: impl Into<String>, language: impl Into<String>) -> Self {
        Self::Code {
            code: code.into(),
            language: language.into(),
        }
    }

    /// Create a new table section.
    #[must_use]
    pub const fn table(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self::Table { headers, rows }
    }

    /// Create a new section with heading.
    #[must_use]
    pub fn section(heading: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Section {
            heading: heading.into(),
            content: content.into(),
        }
    }

    /// Format this section as markdown.
    #[must_use]
    pub fn to_markdown(&self, config: &ExplainConfig) -> String {
        match self {
            Self::Text(text) => {
                if config.indent > 0 {
                    let indent = " ".repeat(config.indent);
                    text.lines()
                        .map(|line| format!("{indent}{line}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    text.clone()
                }
            }
            Self::Bullets(items) => items
                .iter()
                .map(|item| {
                    if config.indent > 0 {
                        let indent = " ".repeat(config.indent);
                        format!("{indent}- {item}")
                    } else {
                        format!("- {item}")
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Code { code, language } => {
                let indent = if config.indent > 0 {
                    " ".repeat(config.indent)
                } else {
                    String::new()
                };
                let code_lines = code
                    .lines()
                    .map(|line| format!("{indent}{line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{indent}```{language}\n{code_lines}\n{indent}```")
            }
            Self::Table { headers, rows } => {
                if headers.is_empty() {
                    return String::new();
                }
                let indent = if config.indent > 0 {
                    " ".repeat(config.indent)
                } else {
                    String::new()
                };

                // Calculate column widths
                let mut widths: Vec<usize> = headers.iter().map(String::len).collect();
                for row in rows {
                    for (i, cell) in row.iter().enumerate() {
                        if i < widths.len() {
                            widths[i] = widths[i].max(cell.len());
                        }
                    }
                }

                // Build header row
                let header_row: String = headers
                    .iter()
                    .enumerate()
                    .map(|(i, h)| format!(" {} ", pad_right(h, widths.get(i).copied().unwrap_or(0))))
                    .collect::<Vec<_>>()
                    .join("|");
                
                // Build separator
                let separator: String = widths
                    .iter()
                    .map(|&w| format!("{}{}{}", "-".repeat(w + 2), "", ""))
                    .collect::<Vec<_>>()
                    .join("|");
                
                // Build data rows
                let data_rows: String = rows
                    .iter()
                    .map(|row| {
                        let cells: String = row
                            .iter()
                            .enumerate()
                            .map(|(i, cell)| {
                                format!(" {} ", pad_right(cell, widths.get(i).copied().unwrap_or(0)))
                            })
                            .collect::<Vec<_>>()
                            .join("|");
                        format!("{indent}|{cells}|")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                if data_rows.is_empty() {
                    format!("{indent}|{header_row}|\n{indent}|{separator}|")
                } else {
                    format!("{indent}|{header_row}|\n{indent}|{separator}|\n{data_rows}")
                }
            }
            Self::Section { heading, content } => {
                let indent = if config.indent > 0 {
                    " ".repeat(config.indent)
                } else {
                    String::new()
                };
                format!("{indent}## {heading}\n\n{indent}{content}")
            }
        }
    }

    /// Format this section as plain text.
    #[must_use]
    pub fn to_plain_text(&self, config: &ExplainConfig) -> String {
        match self {
            Self::Text(text) => {
                if config.indent > 0 {
                    let indent = " ".repeat(config.indent);
                    text.lines()
                        .map(|line| format!("{indent}{line}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    text.clone()
                }
            }
            Self::Bullets(items) => items
                .iter()
                .map(|item| {
                    if config.indent > 0 {
                        let indent = " ".repeat(config.indent);
                        format!("{indent}* {item}")
                    } else {
                        format!("* {item}")
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Code { code, language } => {
                let indent = if config.indent > 0 {
                    " ".repeat(config.indent)
                } else {
                    String::new()
                };
                let lang_prefix = if language.is_empty() {
                    String::new()
                } else {
                    format!("[{language}]\n")
                };
                let code_lines = code
                    .lines()
                    .map(|line| format!("{indent}    {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{indent}{lang_prefix}{code_lines}")
            }
            Self::Table { headers, rows } => {
                if headers.is_empty() {
                    return String::new();
                }
                let indent = if config.indent > 0 {
                    " ".repeat(config.indent)
                } else {
                    String::new()
                };

                // Calculate column widths
                let mut widths: Vec<usize> = headers.iter().map(String::len).collect();
                for row in rows {
                    for (i, cell) in row.iter().enumerate() {
                        if i < widths.len() {
                            widths[i] = widths[i].max(cell.len());
                        }
                    }
                }

                // Build header row
                let header_row: String = headers
                    .iter()
                    .enumerate()
                    .map(|(i, h)| pad_right(h, widths.get(i).copied().unwrap_or(0)))
                    .collect::<Vec<_>>()
                    .join(" | ");

                // Build separator
                let separator: String = widths
                    .iter()
                    .map(|&w| "-".repeat(w))
                    .collect::<Vec<_>>()
                    .join("-+-");

                // Build data rows
                let data_rows: String = rows
                    .iter()
                    .map(|row| {
                        let cells: String = row
                            .iter()
                            .enumerate()
                            .map(|(i, cell)| pad_right(cell, widths.get(i).copied().unwrap_or(0)))
                            .collect::<Vec<_>>()
                            .join(" | ");
                        format!("{indent}{cells}")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                if data_rows.is_empty() {
                    format!("{indent}{header_row}\n{indent}{separator}")
                } else {
                    format!("{indent}{header_row}\n{indent}{separator}\n{data_rows}")
                }
            }
            Self::Section { heading, content } => {
                let indent = if config.indent > 0 {
                    " ".repeat(config.indent)
                } else {
                    String::new()
                };
                format!("{indent}[{heading}]\n\n{indent}{content}")
            }
        }
    }
}

/// Configuration for explain output formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct ExplainConfig {
    /// Indentation level (number of spaces).
    pub indent: usize,
    /// Maximum line width for wrapping.
    pub line_width: usize,
    /// Enable ANSI color codes in output.
    pub color: bool,
}

impl Default for ExplainConfig {
    fn default() -> Self {
        Self {
            indent: 0,
            line_width: 80,
            color: false,
        }
    }
}

impl ExplainConfig {
    /// Create a new config with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            indent: 0,
            line_width: 80,
            color: false,
        }
    }

    /// Create a new config with the specified indent.
    #[must_use]
    pub const fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    /// Create a new config with the specified line width.
    #[must_use]
    pub const fn with_line_width(mut self, line_width: usize) -> Self {
        self.line_width = line_width;
        self
    }

    /// Create a new config with ANSI colors enabled/disabled.
    #[must_use]
    pub const fn with_color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }
}

/// Builder for creating human-readable explain output.
///
/// This builder provides a fluent interface for constructing formatted
/// explanations with support for titles, summaries, sections, bullet points,
/// code blocks, and tables.
///
/// # Example
///
/// ```
/// use lintdiff_explain_builder::ExplainBuilder;
///
/// let mut builder = ExplainBuilder::new();
/// builder
///     .with_title("My Title")
///     .with_summary("A brief summary")
///     .add_bullet("First point")
///     .add_bullet("Second point");
///
/// let output = builder.build();
/// assert!(output.contains("# My Title"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ExplainBuilder {
    title: Option<String>,
    summary: Option<String>,
    sections: Vec<ExplainSection>,
    config: ExplainConfig,
}

impl ExplainBuilder {
    /// Create a new empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new builder with the specified configuration.
    #[must_use]
    pub fn with_config(config: ExplainConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    /// Set the title for the explanation.
    ///
    /// The title is rendered as a level-1 heading in markdown.
    pub fn with_title(&mut self, title: &str) -> &mut Self {
        self.title = Some(title.to_string());
        self
    }

    /// Set the summary for the explanation.
    ///
    /// The summary appears after the title as an italicized paragraph.
    pub fn with_summary(&mut self, summary: &str) -> &mut Self {
        self.summary = Some(summary.to_string());
        self
    }

    /// Add a section with a heading and content.
    ///
    /// Sections are rendered as level-2 headings in markdown.
    pub fn add_section(&mut self, heading: &str, content: &str) -> &mut Self {
        self.sections
            .push(ExplainSection::section(heading, content));
        self
    }

    /// Add a bullet point to the explanation.
    ///
    /// Bullet points are collected and rendered as a list.
    pub fn add_bullet(&mut self, item: &str) -> &mut Self {
        // Find or create a Bullets section
        if let Some(ExplainSection::Bullets(items)) = self.sections.last_mut() {
            items.push(item.to_string());
            return self;
        }
        self.sections.push(ExplainSection::bullets(vec![item.to_string()]));
        self
    }

    /// Add a code block to the explanation.
    ///
    /// The language is used for syntax highlighting in markdown.
    pub fn add_code_block(&mut self, code: &str, language: &str) -> &mut Self {
        self.sections
            .push(ExplainSection::code(code, language));
        self
    }

    /// Add a table to the explanation.
    ///
    /// # Arguments
    ///
    /// * `headers` - Column headers
    /// * `rows` - Table rows (each row is a slice of cell values)
    pub fn add_table(&mut self, headers: &[&str], rows: &[&[&str]]) -> &mut Self {
        let headers: Vec<String> = headers.iter().map(|s| (*s).to_string()).collect();
        let rows: Vec<Vec<String>> = rows
            .iter()
            .map(|row| row.iter().map(|s| (*s).to_string()).collect())
            .collect();
        self.sections.push(ExplainSection::table(headers, rows));
        self
    }

    /// Add a raw text section.
    pub fn add_text(&mut self, text: &str) -> &mut Self {
        self.sections.push(ExplainSection::text(text));
        self
    }

    /// Add a pre-built section.
    pub fn add_section_item(&mut self, section: ExplainSection) -> &mut Self {
        self.sections.push(section);
        self
    }

    /// Set the configuration for output formatting.
    pub const fn set_config(&mut self, config: ExplainConfig) -> &mut Self {
        self.config = config;
        self
    }

    /// Get the current title.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Get the current summary.
    #[must_use]
    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }

    /// Get the sections.
    #[must_use]
    pub fn sections(&self) -> &[ExplainSection] {
        &self.sections
    }

    /// Get the configuration.
    #[must_use]
    pub const fn config(&self) -> &ExplainConfig {
        &self.config
    }

    /// Check if the builder is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.title.is_none() && self.summary.is_none() && self.sections.is_empty()
    }

    /// Clear all content from the builder.
    pub fn clear(&mut self) -> &mut Self {
        self.title = None;
        self.summary = None;
        self.sections.clear();
        self
    }

    /// Build the final markdown output.
    #[must_use]
    pub fn build(&self) -> String {
        self.build_markdown()
    }

    /// Build the final markdown output.
    #[must_use]
    pub fn build_markdown(&self) -> String {
        let mut output = String::new();

        // Add title
        if let Some(title) = &self.title {
            let title = if self.config.color {
                colorize_title(title)
            } else {
                title.clone()
            };
            output.push_str("# ");
            output.push_str(&title);
            output.push_str("\n\n");
        }

        // Add summary
        if let Some(summary) = &self.summary {
            let summary = if self.config.color {
                colorize_summary(summary)
            } else {
                summary.clone()
            };
            output.push('*');
            output.push_str(&summary);
            output.push_str("*\n\n");
        }

        // Add sections
        for section in &self.sections {
            let section_output = section.to_markdown(&self.config);
            if !section_output.is_empty() {
                if !output.is_empty() && !output.ends_with("\n\n") {
                    if output.ends_with('\n') {
                        output.push('\n');
                    } else {
                        output.push_str("\n\n");
                    }
                }
                output.push_str(&section_output);
            }
        }

        output.trim_end().to_string() + "\n"
    }

    /// Build the final plain text output.
    #[must_use]
    pub fn build_plain_text(&self) -> String {
        let mut output = String::new();

        // Add title
        if let Some(title) = &self.title {
            let title = if self.config.color {
                colorize_title(title)
            } else {
                title.clone()
            };
            output.push_str(&title);
            output.push('\n');
            output.push_str(&"=".repeat(title.len()));
            output.push_str("\n\n");
        }

        // Add summary
        if let Some(summary) = &self.summary {
            let summary = if self.config.color {
                colorize_summary(summary)
            } else {
                summary.clone()
            };
            output.push_str(&summary);
            output.push_str("\n\n");
        }

        // Add sections
        for section in &self.sections {
            let section_output = section.to_plain_text(&self.config);
            if !section_output.is_empty() {
                if !output.is_empty() && !output.ends_with("\n\n") {
                    if output.ends_with('\n') {
                        output.push('\n');
                    } else {
                        output.push_str("\n\n");
                    }
                }
                output.push_str(&section_output);
            }
        }

        output.trim_end().to_string() + "\n"
    }
}

/// Create a simple explanation with just a title and content.
///
/// This is a convenience function for creating quick explanations
/// without using the builder pattern.
#[must_use]
pub fn explain_simple(title: &str, content: &str) -> String {
    let mut builder = ExplainBuilder::new();
    builder.with_title(title).add_text(content);
    builder.build()
}

/// Format an `ExplainBuilder`'s content as markdown.
#[must_use]
pub fn format_as_markdown(builder: &ExplainBuilder) -> String {
    builder.build_markdown()
}

/// Format an `ExplainBuilder`'s content as plain text.
#[must_use]
pub fn format_as_plain_text(builder: &ExplainBuilder) -> String {
    builder.build_plain_text()
}

/// Pad a string on the right to the specified width.
fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}

/// Apply ANSI color to a title.
fn colorize_title(title: &str) -> String {
    // Bold cyan
    format!("\x1b[1;36m{title}\x1b[0m")
}

/// Apply ANSI color to a summary.
fn colorize_summary(summary: &str) -> String {
    // Italic gray
    format!("\x1b[3;90m{summary}\x1b[0m")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_section_text() {
        let section = ExplainSection::text("Hello, world!");
        assert_eq!(section, ExplainSection::Text("Hello, world!".to_string()));
    }

    #[test]
    fn test_explain_section_bullets() {
        let section = ExplainSection::bullets(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            section,
            ExplainSection::Bullets(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn test_explain_section_code() {
        let section = ExplainSection::code("fn main() {}", "rust");
        assert_eq!(
            section,
            ExplainSection::Code {
                code: "fn main() {}".to_string(),
                language: "rust".to_string(),
            }
        );
    }

    #[test]
    fn test_explain_section_table() {
        let section = ExplainSection::table(
            vec!["A".to_string(), "B".to_string()],
            vec![vec!["1".to_string(), "2".to_string()]],
        );
        assert_eq!(
            section,
            ExplainSection::Table {
                headers: vec!["A".to_string(), "B".to_string()],
                rows: vec![vec!["1".to_string(), "2".to_string()]],
            }
        );
    }

    #[test]
    fn test_explain_section_section() {
        let section = ExplainSection::section("Heading", "Content");
        assert_eq!(
            section,
            ExplainSection::Section {
                heading: "Heading".to_string(),
                content: "Content".to_string(),
            }
        );
    }

    #[test]
    fn test_explain_config_default() {
        let config = ExplainConfig::default();
        assert_eq!(config.indent, 0);
        assert_eq!(config.line_width, 80);
        assert!(!config.color);
    }

    #[test]
    fn test_explain_config_new() {
        let config = ExplainConfig::new();
        assert_eq!(config.indent, 0);
        assert_eq!(config.line_width, 80);
        assert!(!config.color);
    }

    #[test]
    fn test_explain_config_with_indent() {
        let config = ExplainConfig::new().with_indent(4);
        assert_eq!(config.indent, 4);
    }

    #[test]
    fn test_explain_config_with_line_width() {
        let config = ExplainConfig::new().with_line_width(120);
        assert_eq!(config.line_width, 120);
    }

    #[test]
    fn test_explain_config_with_color() {
        let config = ExplainConfig::new().with_color(true);
        assert!(config.color);
    }

    #[test]
    fn test_explain_builder_new() {
        let builder = ExplainBuilder::new();
        assert!(builder.is_empty());
        assert!(builder.title().is_none());
        assert!(builder.summary().is_none());
        assert!(builder.sections().is_empty());
    }

    #[test]
    fn test_explain_builder_with_title() {
        let mut builder = ExplainBuilder::new();
        builder.with_title("Test Title");
        assert_eq!(builder.title(), Some("Test Title"));
    }

    #[test]
    fn test_explain_builder_with_summary() {
        let mut builder = ExplainBuilder::new();
        builder.with_summary("Test summary");
        assert_eq!(builder.summary(), Some("Test summary"));
    }

    #[test]
    fn test_explain_builder_add_bullet() {
        let mut builder = ExplainBuilder::new();
        builder.add_bullet("Item 1").add_bullet("Item 2");
        assert_eq!(builder.sections().len(), 1);
    }

    #[test]
    fn test_explain_builder_add_section() {
        let mut builder = ExplainBuilder::new();
        builder.add_section("Heading", "Content");
        assert_eq!(builder.sections().len(), 1);
    }

    #[test]
    fn test_explain_builder_add_code_block() {
        let mut builder = ExplainBuilder::new();
        builder.add_code_block("fn main() {}", "rust");
        assert_eq!(builder.sections().len(), 1);
    }

    #[test]
    fn test_explain_builder_add_table() {
        let mut builder = ExplainBuilder::new();
        builder.add_table(&["A", "B"], &[&["1", "2"]]);
        assert_eq!(builder.sections().len(), 1);
    }

    #[test]
    fn test_explain_builder_clear() {
        let mut builder = ExplainBuilder::new();
        builder
            .with_title("Title")
            .with_summary("Summary")
            .add_bullet("Item");
        assert!(!builder.is_empty());
        builder.clear();
        assert!(builder.is_empty());
    }

    #[test]
    fn test_explain_simple() {
        let output = explain_simple("Title", "Content");
        assert!(output.contains("# Title"));
        assert!(output.contains("Content"));
    }

    #[test]
    fn test_format_as_markdown() {
        let mut builder = ExplainBuilder::new();
        builder.with_title("Title").add_text("Content");
        let output = format_as_markdown(&builder);
        assert!(output.contains("# Title"));
    }

    #[test]
    fn test_format_as_plain_text() {
        let mut builder = ExplainBuilder::new();
        builder.with_title("Title").add_text("Content");
        let output = format_as_plain_text(&builder);
        assert!(output.contains("Title"));
        assert!(output.contains("===="));
    }

    #[test]
    fn test_build_empty() {
        let builder = ExplainBuilder::new();
        let output = builder.build();
        assert!(output.is_empty() || output == "\n");
    }

    #[test]
    fn test_build_with_title() {
        let mut builder = ExplainBuilder::new();
        builder.with_title("My Title");
        let output = builder.build();
        assert!(output.contains("# My Title"));
    }

    #[test]
    fn test_build_with_summary() {
        let mut builder = ExplainBuilder::new();
        builder.with_summary("My summary");
        let output = builder.build();
        assert!(output.contains("*My summary*"));
    }

    #[test]
    fn test_build_with_bullets() {
        let mut builder = ExplainBuilder::new();
        builder.add_bullet("Item 1").add_bullet("Item 2");
        let output = builder.build();
        assert!(output.contains("- Item 1"));
        assert!(output.contains("- Item 2"));
    }

    #[test]
    fn test_build_with_code_block() {
        let mut builder = ExplainBuilder::new();
        builder.add_code_block("fn main() {}", "rust");
        let output = builder.build();
        assert!(output.contains("```rust"));
        assert!(output.contains("fn main() {}"));
    }

    #[test]
    fn test_build_with_table() {
        let mut builder = ExplainBuilder::new();
        builder.add_table(&["A", "B"], &[&["1", "2"], &["3", "4"]]);
        let output = builder.build();
        assert!(output.contains("| A |"));
        assert!(output.contains("| 1 |"));
    }

    #[test]
    fn test_build_plain_text_with_title() {
        let mut builder = ExplainBuilder::new();
        builder.with_title("Title");
        let output = builder.build_plain_text();
        assert!(output.contains("Title"));
        assert!(output.contains("====="));
    }

    #[test]
    fn test_build_plain_text_with_bullets() {
        let mut builder = ExplainBuilder::new();
        builder.add_bullet("Item 1").add_bullet("Item 2");
        let output = builder.build_plain_text();
        assert!(output.contains("* Item 1"));
        assert!(output.contains("* Item 2"));
    }

    #[test]
    fn test_build_plain_text_with_code_block() {
        let mut builder = ExplainBuilder::new();
        builder.add_code_block("fn main() {}", "rust");
        let output = builder.build_plain_text();
        assert!(output.contains("[rust]"));
        assert!(output.contains("fn main() {}"));
    }

    #[test]
    fn test_build_plain_text_with_table() {
        let mut builder = ExplainBuilder::new();
        builder.add_table(&["A", "B"], &[&["1", "2"]]);
        let output = builder.build_plain_text();
        assert!(output.contains("A | B"));
        assert!(output.contains("-+-"));
    }

    #[test]
    fn test_build_with_color() {
        let config = ExplainConfig::new().with_color(true);
        let mut builder = ExplainBuilder::with_config(config);
        builder.with_title("Title");
        let output = builder.build();
        assert!(output.contains("\x1b["));
    }

    #[test]
    fn test_section_to_markdown_text() {
        let section = ExplainSection::text("Hello");
        let config = ExplainConfig::default();
        let output = section.to_markdown(&config);
        assert_eq!(output, "Hello");
    }

    #[test]
    fn test_section_to_markdown_bullets() {
        let section = ExplainSection::bullets(vec!["a".to_string(), "b".to_string()]);
        let config = ExplainConfig::default();
        let output = section.to_markdown(&config);
        assert_eq!(output, "- a\n- b");
    }

    #[test]
    fn test_section_to_markdown_code() {
        let section = ExplainSection::code("code", "rust");
        let config = ExplainConfig::default();
        let output = section.to_markdown(&config);
        assert!(output.contains("```rust"));
        assert!(output.contains("code"));
    }

    #[test]
    fn test_section_to_markdown_table() {
        let section = ExplainSection::table(
            vec!["A".to_string()],
            vec![vec!["1".to_string()]],
        );
        let config = ExplainConfig::default();
        let output = section.to_markdown(&config);
        assert!(output.contains("| A |"));
        assert!(output.contains("| 1 |"));
    }

    #[test]
    fn test_section_to_plain_text_text() {
        let section = ExplainSection::text("Hello");
        let config = ExplainConfig::default();
        let output = section.to_plain_text(&config);
        assert_eq!(output, "Hello");
    }

    #[test]
    fn test_section_to_plain_text_bullets() {
        let section = ExplainSection::bullets(vec!["a".to_string(), "b".to_string()]);
        let config = ExplainConfig::default();
        let output = section.to_plain_text(&config);
        assert_eq!(output, "* a\n* b");
    }

    #[test]
    fn test_section_to_plain_text_code() {
        let section = ExplainSection::code("code", "rust");
        let config = ExplainConfig::default();
        let output = section.to_plain_text(&config);
        assert!(output.contains("[rust]"));
        assert!(output.contains("code"));
    }

    #[test]
    fn test_section_to_plain_text_table() {
        let section = ExplainSection::table(
            vec!["A".to_string()],
            vec![vec!["1".to_string()]],
        );
        let config = ExplainConfig::default();
        let output = section.to_plain_text(&config);
        assert!(output.contains("A"));
        assert!(output.contains("-"));
        assert!(output.contains("1"));
    }

    #[test]
    fn test_section_with_indent() {
        let section = ExplainSection::text("Hello");
        let config = ExplainConfig::new().with_indent(2);
        let output = section.to_markdown(&config);
        assert_eq!(output, "  Hello");
    }

    #[test]
    fn test_bullets_with_indent() {
        let section = ExplainSection::bullets(vec!["a".to_string()]);
        let config = ExplainConfig::new().with_indent(2);
        let output = section.to_markdown(&config);
        assert_eq!(output, "  - a");
    }

    #[test]
    fn test_empty_table() {
        let section = ExplainSection::table(vec![], vec![]);
        let config = ExplainConfig::default();
        let output = section.to_markdown(&config);
        assert!(output.is_empty());
    }

    #[test]
    fn test_empty_bullets() {
        let section = ExplainSection::bullets(vec![]);
        let config = ExplainConfig::default();
        let output = section.to_markdown(&config);
        assert!(output.is_empty());
    }

    #[test]
    fn test_builder_chaining() {
        let mut builder = ExplainBuilder::new();
        builder
            .with_title("Title")
            .with_summary("Summary")
            .add_section("Section", "Content")
            .add_bullet("Bullet")
            .add_code_block("code", "rust")
            .add_table(&["H"], &[&["D"]]);
        
        assert_eq!(builder.title(), Some("Title"));
        assert_eq!(builder.summary(), Some("Summary"));
        assert_eq!(builder.sections().len(), 4);
    }

    #[test]
    fn test_add_text() {
        let mut builder = ExplainBuilder::new();
        builder.add_text("Some text");
        assert_eq!(builder.sections().len(), 1);
    }

    #[test]
    fn test_add_section_item() {
        let mut builder = ExplainBuilder::new();
        builder.add_section_item(ExplainSection::text("Test"));
        assert_eq!(builder.sections().len(), 1);
    }

    #[test]
    fn test_set_config() {
        let mut builder = ExplainBuilder::new();
        let config = ExplainConfig::new().with_indent(4);
        builder.set_config(config.clone());
        assert_eq!(builder.config().indent, 4);
    }

    #[test]
    fn test_with_config() {
        let config = ExplainConfig::new().with_indent(4);
        let builder = ExplainBuilder::with_config(config);
        assert_eq!(builder.config().indent, 4);
    }

    #[test]
    fn test_multiple_bullet_groups() {
        let mut builder = ExplainBuilder::new();
        builder.add_bullet("A1");
        builder.add_bullet("A2");
        builder.add_text("Separator");
        builder.add_bullet("B1");
        
        // Should have 3 sections: Bullets, Text, Bullets
        assert_eq!(builder.sections().len(), 3);
    }

    #[test]
    fn test_table_with_empty_rows() {
        let mut builder = ExplainBuilder::new();
        builder.add_table(&["A", "B"], &[]);
        let output = builder.build();
        assert!(output.contains("| A |"));
    }

    #[test]
    fn test_code_block_empty_language() {
        let mut builder = ExplainBuilder::new();
        builder.add_code_block("code", "");
        let output = builder.build();
        assert!(output.contains("```")); // Should still have code fences
    }

    #[test]
    fn test_plain_text_code_empty_language() {
        let section = ExplainSection::code("code", "");
        let config = ExplainConfig::default();
        let output = section.to_plain_text(&config);
        assert!(!output.contains("[]")); // Should not have empty brackets
        assert!(output.contains("code"));
    }

    #[test]
    fn test_section_markdown() {
        let section = ExplainSection::section("Heading", "Content");
        let config = ExplainConfig::default();
        let output = section.to_markdown(&config);
        assert!(output.contains("## Heading"));
        assert!(output.contains("Content"));
    }

    #[test]
    fn test_section_plain_text() {
        let section = ExplainSection::section("Heading", "Content");
        let config = ExplainConfig::default();
        let output = section.to_plain_text(&config);
        assert!(output.contains("[Heading]"));
        assert!(output.contains("Content"));
    }

    #[test]
    fn test_multiline_text() {
        let section = ExplainSection::text("Line 1\nLine 2\nLine 3");
        let config = ExplainConfig::new().with_indent(2);
        let output = section.to_markdown(&config);
        assert!(output.contains("  Line 1"));
        assert!(output.contains("  Line 2"));
        assert!(output.contains("  Line 3"));
    }

    #[test]
    fn test_multiline_code() {
        let section = ExplainSection::code("fn a() {}\nfn b() {}", "rust");
        let config = ExplainConfig::default();
        let output = section.to_markdown(&config);
        assert!(output.contains("fn a() {}"));
        assert!(output.contains("fn b() {}"));
    }

    #[test]
    fn test_colorized_title() {
        let colored = colorize_title("Title");
        assert!(colored.contains("\x1b[1;36m"));
        assert!(colored.contains("Title"));
    }

    #[test]
    fn test_colorized_summary() {
        let colored = colorize_summary("Summary");
        assert!(colored.contains("\x1b[3;90m"));
        assert!(colored.contains("Summary"));
    }

    #[test]
    fn test_pad_right() {
        assert_eq!(pad_right("abc", 5), "abc  ");
        assert_eq!(pad_right("abc", 3), "abc");
        assert_eq!(pad_right("abc", 2), "abc");
    }

    #[test]
    fn test_table_column_widths() {
        let section = ExplainSection::table(
            vec!["A".to_string(), "BBB".to_string()],
            vec![vec!["CCCC".to_string(), "D".to_string()]],
        );
        let config = ExplainConfig::default();
        let output = section.to_markdown(&config);
        // First column should be width 4 (from CCCC)
        // Second column should be width 3 (from BBB)
        assert!(output.contains("CCCC"));
        assert!(output.contains("BBB"));
    }

    #[test]
    fn test_escaped_characters_in_markdown() {
        let mut builder = ExplainBuilder::new();
        builder.with_title("Title with <special> & \"chars\"");
        let output = builder.build();
        assert!(output.contains("Title with <special> & \"chars\""));
    }

    #[test]
    fn test_unicode_content() {
        let mut builder = ExplainBuilder::new();
        builder.with_title("日本語タイトル").add_bullet("項目１");
        let output = builder.build();
        assert!(output.contains("日本語タイトル"));
        assert!(output.contains("項目１"));
    }

    #[test]
    fn test_very_long_title() {
        let long_title = "A".repeat(1000);
        let mut builder = ExplainBuilder::new();
        builder.with_title(&long_title);
        let output = builder.build();
        assert!(output.contains(&long_title));
    }

    #[test]
    fn test_very_long_bullet() {
        let long_bullet = "B".repeat(1000);
        let mut builder = ExplainBuilder::new();
        builder.add_bullet(&long_bullet);
        let output = builder.build();
        assert!(output.contains(&long_bullet));
    }
}
