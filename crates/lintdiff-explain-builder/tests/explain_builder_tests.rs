//! BDD tests for the lintdiff-explain-builder microcrate.
//!
//! These tests follow the Given-When-Then pattern to ensure comprehensive
//! coverage of the builder functionality.

use lintdiff_explain_builder::{
    explain_simple, format_as_markdown, format_as_plain_text, ExplainBuilder, ExplainConfig,
    ExplainSection,
};

// =============================================================================
// Feature: ExplainBuilder Creation
// =============================================================================

mod builder_creation {
    use super::*;

    #[test]
    fn given_new_builder_when_checked_then_is_empty() {
        // Given
        let builder = ExplainBuilder::new();

        // When
        let is_empty = builder.is_empty();

        // Then
        assert!(is_empty);
    }

    #[test]
    fn given_default_builder_when_checked_then_is_empty() {
        // Given
        let builder = ExplainBuilder::default();

        // When
        let is_empty = builder.is_empty();

        // Then
        assert!(is_empty);
    }

    #[test]
    fn given_builder_with_config_when_created_then_has_config() {
        // Given
        let config = ExplainConfig::new().with_indent(4);

        // When
        let builder = ExplainBuilder::with_config(config);

        // Then
        assert_eq!(builder.config().indent, 4);
    }

    #[test]
    fn given_new_builder_when_title_set_then_not_empty() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.with_title("Title");

        // Then
        assert!(!builder.is_empty());
    }

    #[test]
    fn given_new_builder_when_summary_set_then_not_empty() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.with_summary("Summary");

        // Then
        assert!(!builder.is_empty());
    }

    #[test]
    fn given_new_builder_when_bullet_added_then_not_empty() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.add_bullet("Item");

        // Then
        assert!(!builder.is_empty());
    }
}

// =============================================================================
// Feature: Title Management
// =============================================================================

mod title_management {
    use super::*;

    #[test]
    fn given_builder_when_title_set_then_can_retrieve() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.with_title("My Title");

        // Then
        assert_eq!(builder.title(), Some("My Title"));
    }

    #[test]
    fn given_builder_with_title_when_cleared_then_title_is_none() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.with_title("Title");

        // When
        builder.clear();

        // Then
        assert!(builder.title().is_none());
    }

    #[test]
    fn given_builder_when_title_overwritten_then_has_new_title() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.with_title("Old Title");

        // When
        builder.with_title("New Title");

        // Then
        assert_eq!(builder.title(), Some("New Title"));
    }

    #[test]
    fn given_empty_title_when_set_then_is_stored() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.with_title("");

        // Then
        assert_eq!(builder.title(), Some(""));
    }

    #[test]
    fn given_unicode_title_when_set_then_is_stored() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.with_title("日本語タイトル 🦀");

        // Then
        assert_eq!(builder.title(), Some("日本語タイトル 🦀"));
    }
}

// =============================================================================
// Feature: Summary Management
// =============================================================================

mod summary_management {
    use super::*;

    #[test]
    fn given_builder_when_summary_set_then_can_retrieve() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.with_summary("My summary text");

        // Then
        assert_eq!(builder.summary(), Some("My summary text"));
    }

    #[test]
    fn given_builder_with_summary_when_cleared_then_summary_is_none() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.with_summary("Summary");

        // When
        builder.clear();

        // Then
        assert!(builder.summary().is_none());
    }

    #[test]
    fn given_multiline_summary_when_set_then_is_stored() {
        // Given
        let mut builder = ExplainBuilder::new();
        let summary = "Line 1\nLine 2\nLine 3";

        // When
        builder.with_summary(summary);

        // Then
        assert_eq!(builder.summary(), Some(summary));
    }
}

// =============================================================================
// Feature: Bullet Points
// =============================================================================

mod bullet_points {
    use super::*;

    #[test]
    fn given_builder_when_single_bullet_added_then_has_one_section() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.add_bullet("Item 1");

        // Then
        assert_eq!(builder.sections().len(), 1);
    }

    #[test]
    fn given_builder_when_two_bullets_added_consecutively_then_single_bullets_section() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.add_bullet("Item 1").add_bullet("Item 2");

        // Then
        assert_eq!(builder.sections().len(), 1);
        if let ExplainSection::Bullets { items } = &builder.sections()[0] {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected Bullets section");
        }
    }

    #[test]
    fn given_bullets_when_separated_by_other_content_then_two_bullets_sections() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.add_bullet("A1");

        // When
        builder.add_text("Separator").add_bullet("B1");

        // Then
        assert_eq!(builder.sections().len(), 3);
    }

    #[test]
    fn given_empty_bullet_when_added_then_is_stored() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.add_bullet("");

        // Then
        if let ExplainSection::Bullets { items } = &builder.sections()[0] {
            assert_eq!(items[0], "");
        } else {
            panic!("Expected Bullets section");
        }
    }

    #[test]
    fn given_many_bullets_when_added_then_all_stored() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        for i in 0..100 {
            builder.add_bullet(&format!("Item {}", i));
        }

        // Then
        if let ExplainSection::Bullets { items } = &builder.sections()[0] {
            assert_eq!(items.len(), 100);
        } else {
            panic!("Expected Bullets section");
        }
    }
}

// =============================================================================
// Feature: Code Blocks
// =============================================================================

mod code_blocks {
    use super::*;

    #[test]
    fn given_code_block_when_added_then_has_code_section() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.add_code_block("fn main() {}", "rust");

        // Then
        assert_eq!(builder.sections().len(), 1);
        if let ExplainSection::Code { code, language } = &builder.sections()[0] {
            assert_eq!(code, "fn main() {}");
            assert_eq!(language, "rust");
        } else {
            panic!("Expected Code section");
        }
    }

    #[test]
    fn given_code_block_empty_language_when_added_then_stored() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.add_code_block("code", "");

        // Then
        if let ExplainSection::Code { language, .. } = &builder.sections()[0] {
            assert_eq!(language, "");
        } else {
            panic!("Expected Code section");
        }
    }

    #[test]
    fn given_multiline_code_when_added_then_all_lines_preserved() {
        // Given
        let mut builder = ExplainBuilder::new();
        let code = "fn a() {}\nfn b() {}\nfn c() {}";

        // When
        builder.add_code_block(code, "rust");

        // Then
        if let ExplainSection::Code {
            code: stored_code, ..
        } = &builder.sections()[0]
        {
            assert!(stored_code.contains("fn a()"));
            assert!(stored_code.contains("fn b()"));
            assert!(stored_code.contains("fn c()"));
        } else {
            panic!("Expected Code section");
        }
    }

    #[test]
    fn given_multiple_code_blocks_when_added_then_separate_sections() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder
            .add_code_block("code1", "rust")
            .add_code_block("code2", "python");

        // Then
        assert_eq!(builder.sections().len(), 2);
    }
}

// =============================================================================
// Feature: Tables
// =============================================================================

mod tables {
    use super::*;

    #[test]
    fn given_table_when_added_then_has_table_section() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.add_table(&["A", "B"], &[&["1", "2"]]);

        // Then
        assert_eq!(builder.sections().len(), 1);
        if let ExplainSection::Table { headers, rows } = &builder.sections()[0] {
            assert_eq!(headers, &["A", "B"]);
            assert_eq!(rows, &[vec!["1", "2"]]);
        } else {
            panic!("Expected Table section");
        }
    }

    #[test]
    fn given_table_no_rows_when_added_then_headers_preserved() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.add_table(&["Col1", "Col2"], &[]);

        // Then
        if let ExplainSection::Table { headers, rows } = &builder.sections()[0] {
            assert_eq!(headers.len(), 2);
            assert!(rows.is_empty());
        } else {
            panic!("Expected Table section");
        }
    }

    #[test]
    fn given_table_no_headers_when_added_then_empty_table() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.add_table(&[], &[&["1", "2"]]);

        // Then
        if let ExplainSection::Table { headers, .. } = &builder.sections()[0] {
            assert!(headers.is_empty());
        } else {
            panic!("Expected Table section");
        }
    }

    #[test]
    fn given_table_many_rows_when_added_then_all_preserved() {
        // Given
        let mut builder = ExplainBuilder::new();
        let row_data: Vec<[String; 2]> = (0..50)
            .map(|i| [i.to_string(), "val".to_string()])
            .collect();
        let row_refs: Vec<[&str; 2]> = row_data
            .iter()
            .map(|r| [&r[0] as &str, &r[1] as &str])
            .collect();
        let row_slices: Vec<&[&str]> = row_refs.iter().map(|r| &r[..]).collect();

        // When
        builder.add_table(&["ID", "Value"], &row_slices);

        // Then
        if let ExplainSection::Table {
            rows: stored_rows, ..
        } = &builder.sections()[0]
        {
            assert_eq!(stored_rows.len(), 50);
        } else {
            panic!("Expected Table section");
        }
    }

    #[test]
    fn given_table_uneven_rows_when_added_then_preserved_as_is() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.add_table(&["A", "B", "C"], &[&["1", "2"]]);

        // Then
        if let ExplainSection::Table { rows, .. } = &builder.sections()[0] {
            assert_eq!(rows[0].len(), 2);
        } else {
            panic!("Expected Table section");
        }
    }
}

// =============================================================================
// Feature: Sections
// =============================================================================

mod sections {
    use super::*;

    #[test]
    fn given_section_when_added_then_has_section_item() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.add_section("Heading", "Content");

        // Then
        if let ExplainSection::Section { heading, content } = &builder.sections()[0] {
            assert_eq!(heading, "Heading");
            assert_eq!(content, "Content");
        } else {
            panic!("Expected Section");
        }
    }

    #[test]
    fn given_section_empty_content_when_added_then_stored() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder.add_section("Heading", "");

        // Then
        if let ExplainSection::Section { content, .. } = &builder.sections()[0] {
            assert_eq!(content, "");
        } else {
            panic!("Expected Section");
        }
    }

    #[test]
    fn given_multiple_sections_when_added_then_all_preserved() {
        // Given
        let mut builder = ExplainBuilder::new();

        // When
        builder
            .add_section("First", "Content 1")
            .add_section("Second", "Content 2");

        // Then
        assert_eq!(builder.sections().len(), 2);
    }
}

// =============================================================================
// Feature: Markdown Output
// =============================================================================

mod markdown_output {
    use super::*;

    #[test]
    fn given_title_when_build_markdown_then_has_h1() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.with_title("My Title");

        // When
        let output = builder.build_markdown();

        // Then
        assert!(output.contains("# My Title"));
    }

    #[test]
    fn given_summary_when_build_markdown_then_is_italic() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.with_summary("Summary text");

        // When
        let output = builder.build_markdown();

        // Then
        assert!(output.contains("*Summary text*"));
    }

    #[test]
    fn given_bullets_when_build_markdown_then_has_list() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.add_bullet("Item 1").add_bullet("Item 2");

        // When
        let output = builder.build_markdown();

        // Then
        assert!(output.contains("- Item 1"));
        assert!(output.contains("- Item 2"));
    }

    #[test]
    fn given_code_block_when_build_markdown_then_has_fence() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.add_code_block("fn main() {}", "rust");

        // When
        let output = builder.build_markdown();

        // Then
        assert!(output.contains("```rust"));
        assert!(output.contains("fn main() {}"));
        assert!(output.matches("```").count() >= 2);
    }

    #[test]
    fn given_table_when_build_markdown_then_has_table_format() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.add_table(&["A", "B"], &[&["1", "2"]]);

        // When
        let output = builder.build_markdown();

        // Then
        assert!(output.contains("| A |"));
        assert!(output.contains("| 1 |"));
    }

    #[test]
    fn given_section_when_build_markdown_then_has_h2() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.add_section("Heading", "Content");

        // When
        let output = builder.build_markdown();

        // Then
        assert!(output.contains("## Heading"));
        assert!(output.contains("Content"));
    }

    #[test]
    fn given_full_builder_when_build_markdown_then_complete_output() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder
            .with_title("Title")
            .with_summary("Summary")
            .add_section("Section", "Content")
            .add_bullet("Bullet")
            .add_code_block("code", "rust")
            .add_table(&["H"], &[&["D"]]);

        // When
        let output = builder.build_markdown();

        // Then
        assert!(output.contains("# Title"));
        assert!(output.contains("*Summary*"));
        assert!(output.contains("## Section"));
        assert!(output.contains("- Bullet"));
        assert!(output.contains("```rust"));
        assert!(output.contains("| H |"));
    }

    #[test]
    fn given_empty_builder_when_build_markdown_then_empty_or_newline() {
        // Given
        let builder = ExplainBuilder::new();

        // When
        let output = builder.build_markdown();

        // Then
        assert!(output.is_empty() || output == "\n");
    }

    #[test]
    fn given_format_as_markdown_when_called_then_same_as_build_markdown() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.with_title("Title");

        // When
        let output1 = builder.build_markdown();
        let output2 = format_as_markdown(&builder);

        // Then
        assert_eq!(output1, output2);
    }
}

// =============================================================================
// Feature: Plain Text Output
// =============================================================================

mod plain_text_output {
    use super::*;

    #[test]
    fn given_title_when_build_plain_text_then_has_underline() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.with_title("Title");

        // When
        let output = builder.build_plain_text();

        // Then
        assert!(output.contains("Title"));
        assert!(output.contains("====="));
    }

    #[test]
    fn given_summary_when_build_plain_text_then_is_plain() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.with_summary("Summary");

        // When
        let output = builder.build_plain_text();

        // Then
        assert!(output.contains("Summary"));
        assert!(!output.contains("*Summary*"));
    }

    #[test]
    fn given_bullets_when_build_plain_text_then_has_asterisks() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.add_bullet("Item");

        // When
        let output = builder.build_plain_text();

        // Then
        assert!(output.contains("* Item"));
        assert!(!output.contains("- Item"));
    }

    #[test]
    fn given_code_block_when_build_plain_text_then_has_language_tag() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.add_code_block("code", "rust");

        // When
        let output = builder.build_plain_text();

        // Then
        assert!(output.contains("[rust]"));
        assert!(output.contains("code"));
    }

    #[test]
    fn given_code_block_no_language_when_build_plain_text_then_no_empty_brackets() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.add_code_block("code", "");

        // When
        let output = builder.build_plain_text();

        // Then
        assert!(!output.contains("[]"));
    }

    #[test]
    fn given_table_when_build_plain_text_then_has_text_format() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.add_table(&["A", "B"], &[&["1", "2"]]);

        // When
        let output = builder.build_plain_text();

        // Then
        assert!(output.contains("A | B"));
        assert!(output.contains("-+-"));
    }

    #[test]
    fn given_section_when_build_plain_text_then_has_brackets() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.add_section("Heading", "Content");

        // When
        let output = builder.build_plain_text();

        // Then
        assert!(output.contains("[Heading]"));
        assert!(output.contains("Content"));
    }

    #[test]
    fn given_format_as_plain_text_when_called_then_same_as_build_plain_text() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.with_title("Title");

        // When
        let output1 = builder.build_plain_text();
        let output2 = format_as_plain_text(&builder);

        // Then
        assert_eq!(output1, output2);
    }
}

// =============================================================================
// Feature: Configuration
// =============================================================================

mod configuration {
    use super::*;

    #[test]
    fn given_default_config_when_created_then_has_defaults() {
        // Given/When
        let config = ExplainConfig::default();

        // Then
        assert_eq!(config.indent, 0);
        assert_eq!(config.line_width, 80);
        assert!(!config.color);
    }

    #[test]
    fn given_config_with_indent_when_used_then_output_is_indented() {
        // Given
        let config = ExplainConfig::new().with_indent(4);
        let mut builder = ExplainBuilder::with_config(config);
        builder.add_text("Text");

        // When
        let output = builder.build_markdown();

        // Then
        assert!(output.contains("    Text"));
    }

    #[test]
    fn given_config_with_color_when_used_then_has_ansi_codes() {
        // Given
        let config = ExplainConfig::new().with_color(true);
        let mut builder = ExplainBuilder::with_config(config);
        builder.with_title("Title");

        // When
        let output = builder.build_markdown();

        // Then
        assert!(output.contains("\x1b["));
    }

    #[test]
    fn given_config_without_color_when_used_then_no_ansi_codes() {
        // Given
        let config = ExplainConfig::new().with_color(false);
        let mut builder = ExplainBuilder::with_config(config);
        builder.with_title("Title");

        // When
        let output = builder.build_markdown();

        // Then
        assert!(!output.contains("\x1b["));
    }

    #[test]
    fn given_builder_when_set_config_then_config_updated() {
        // Given
        let mut builder = ExplainBuilder::new();
        let config = ExplainConfig::new().with_indent(8);

        // When
        builder.set_config(config);

        // Then
        assert_eq!(builder.config().indent, 8);
    }

    #[test]
    fn test_config_line_width() {
        let config = ExplainConfig::new().with_line_width(120);
        assert_eq!(config.line_width, 120);
    }
}

// =============================================================================
// Feature: Indentation
// =============================================================================

mod indentation {
    use super::*;

    #[test]
    fn given_indent_when_text_section_then_indented() {
        // Given
        let section = ExplainSection::text("Hello");
        let config = ExplainConfig::new().with_indent(2);

        // When
        let output = section.to_markdown(&config);

        // Then
        assert_eq!(output, "  Hello");
    }

    #[test]
    fn given_indent_when_bullets_section_then_indented() {
        // Given
        let section = ExplainSection::bullets(vec!["Item".to_string()]);
        let config = ExplainConfig::new().with_indent(3);

        // When
        let output = section.to_markdown(&config);

        // Then
        assert_eq!(output, "   - Item");
    }

    #[test]
    fn given_indent_when_code_section_then_indented() {
        // Given
        let section = ExplainSection::code("code", "rust");
        let config = ExplainConfig::new().with_indent(4);

        // When
        let output = section.to_markdown(&config);

        // Then
        assert!(output.contains("    ```rust"));
        assert!(output.contains("    code"));
    }

    #[test]
    fn given_indent_when_table_section_then_indented() {
        // Given
        let section = ExplainSection::table(vec!["A".to_string()], vec![]);
        let config = ExplainConfig::new().with_indent(2);

        // When
        let output = section.to_markdown(&config);

        // Then
        assert!(output.starts_with("  |"));
    }

    #[test]
    fn given_indent_when_section_then_indented() {
        // Given
        let section = ExplainSection::section("H", "C");
        let config = ExplainConfig::new().with_indent(2);

        // When
        let output = section.to_markdown(&config);

        // Then
        assert!(output.contains("  ## H"));
    }

    #[test]
    fn given_indent_when_multiline_text_then_all_lines_indented() {
        // Given
        let section = ExplainSection::text("Line1\nLine2\nLine3");
        let config = ExplainConfig::new().with_indent(2);

        // When
        let output = section.to_markdown(&config);

        // Then
        assert!(output.contains("  Line1"));
        assert!(output.contains("  Line2"));
        assert!(output.contains("  Line3"));
    }
}

// =============================================================================
// Feature: Convenience Functions
// =============================================================================

mod convenience_functions {
    use super::*;

    #[test]
    fn given_explain_simple_when_called_then_has_title_and_content() {
        // When
        let output = explain_simple("Title", "Content");

        // Then
        assert!(output.contains("# Title"));
        assert!(output.contains("Content"));
    }

    #[test]
    fn given_explain_simple_empty_content_then_has_title() {
        // When
        let output = explain_simple("Title", "");

        // Then
        assert!(output.contains("# Title"));
    }
}

// =============================================================================
// Feature: Edge Cases
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn given_special_characters_when_building_then_preserved() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.with_title("Title with <special> & \"chars\"");

        // When
        let output = builder.build();

        // Then
        assert!(output.contains("<special>"));
        assert!(output.contains("&"));
        assert!(output.contains("\"chars\""));
    }

    #[test]
    fn given_unicode_content_when_building_then_preserved() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.with_title("日本語").add_bullet("項目");

        // When
        let output = builder.build();

        // Then
        assert!(output.contains("日本語"));
        assert!(output.contains("項目"));
    }

    #[test]
    fn given_very_long_title_when_building_then_not_truncated() {
        // Given
        let mut builder = ExplainBuilder::new();
        let long_title = "A".repeat(1000);
        builder.with_title(&long_title);

        // When
        let output = builder.build();

        // Then
        assert!(output.contains(&long_title));
    }

    #[test]
    fn given_very_long_bullet_when_building_then_not_truncated() {
        // Given
        let mut builder = ExplainBuilder::new();
        let long_bullet = "B".repeat(1000);
        builder.add_bullet(&long_bullet);

        // When
        let output = builder.build();

        // Then
        assert!(output.contains(&long_bullet));
    }

    #[test]
    fn given_empty_table_headers_when_build_then_empty_output() {
        // Given
        let section = ExplainSection::table(vec![], vec![]);
        let config = ExplainConfig::default();

        // When
        let output = section.to_markdown(&config);

        // Then
        assert!(output.is_empty());
    }

    #[test]
    fn given_newlines_in_content_when_build_then_preserved() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.add_text("Line1\nLine2\nLine3");

        // When
        let output = builder.build();

        // Then
        assert!(output.contains("Line1"));
        assert!(output.contains("Line2"));
        assert!(output.contains("Line3"));
    }

    #[test]
    fn given_builder_when_cleared_then_can_reuse() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder.with_title("Old").add_bullet("Old Item");

        // When
        builder.clear();
        builder.with_title("New").add_bullet("New Item");

        // Then
        let output = builder.build();
        assert!(output.contains("# New"));
        assert!(output.contains("- New Item"));
        assert!(!output.contains("Old"));
    }

    #[test]
    fn given_many_sections_when_build_then_all_included() {
        // Given
        let mut builder = ExplainBuilder::new();
        for i in 0..100 {
            builder.add_section(&format!("Section {}", i), &format!("Content {}", i));
        }

        // When
        let output = builder.build();

        // Then
        assert!(output.contains("Section 0"));
        assert!(output.contains("Section 99"));
    }
}

// =============================================================================
// Feature: Section Types
// =============================================================================

mod section_types {
    use super::*;

    #[test]
    fn given_text_section_when_to_markdown_then_plain_text() {
        // Given
        let section = ExplainSection::text("Hello");
        let config = ExplainConfig::default();

        // When
        let output = section.to_markdown(&config);

        // Then
        assert_eq!(output, "Hello");
    }

    #[test]
    fn given_bullets_section_when_to_markdown_then_markdown_list() {
        // Given
        let section = ExplainSection::bullets(vec!["A".to_string(), "B".to_string()]);
        let config = ExplainConfig::default();

        // When
        let output = section.to_markdown(&config);

        // Then
        assert_eq!(output, "- A\n- B");
    }

    #[test]
    fn given_code_section_when_to_markdown_then_fenced_code() {
        // Given
        let section = ExplainSection::code("code", "rust");
        let config = ExplainConfig::default();

        // When
        let output = section.to_markdown(&config);

        // Then
        assert!(output.starts_with("```rust"));
        assert!(output.ends_with("```"));
        assert!(output.contains("code"));
    }

    #[test]
    fn given_table_section_when_to_markdown_then_markdown_table() {
        // Given
        let section = ExplainSection::table(vec!["H".to_string()], vec![vec!["D".to_string()]]);
        let config = ExplainConfig::default();

        // When
        let output = section.to_markdown(&config);

        // Then
        assert!(output.contains("| H |"));
        assert!(output.contains("| D |"));
    }

    #[test]
    fn given_section_when_to_markdown_then_h2_and_content() {
        // Given
        let section = ExplainSection::section("Head", "Body");
        let config = ExplainConfig::default();

        // When
        let output = section.to_markdown(&config);

        // Then
        assert!(output.contains("## Head"));
        assert!(output.contains("Body"));
    }

    #[test]
    fn given_text_section_when_to_plain_text_then_plain_text() {
        // Given
        let section = ExplainSection::text("Hello");
        let config = ExplainConfig::default();

        // When
        let output = section.to_plain_text(&config);

        // Then
        assert_eq!(output, "Hello");
    }

    #[test]
    fn given_bullets_section_when_to_plain_text_then_asterisk_list() {
        // Given
        let section = ExplainSection::bullets(vec!["A".to_string(), "B".to_string()]);
        let config = ExplainConfig::default();

        // When
        let output = section.to_plain_text(&config);

        // Then
        assert_eq!(output, "* A\n* B");
    }

    #[test]
    fn given_code_section_when_to_plain_text_then_indented_with_lang() {
        // Given
        let section = ExplainSection::code("code", "rust");
        let config = ExplainConfig::default();

        // When
        let output = section.to_plain_text(&config);

        // Then
        assert!(output.contains("[rust]"));
        assert!(output.contains("    code"));
    }

    #[test]
    fn given_table_section_when_to_plain_text_then_text_table() {
        // Given
        let section = ExplainSection::table(vec!["H".to_string()], vec![vec!["D".to_string()]]);
        let config = ExplainConfig::default();

        // When
        let output = section.to_plain_text(&config);

        // Then
        assert!(output.contains("H"));
        assert!(output.contains("-"));
        assert!(output.contains("D"));
    }

    #[test]
    fn given_section_when_to_plain_text_then_bracket_heading() {
        // Given
        let section = ExplainSection::section("Head", "Body");
        let config = ExplainConfig::default();

        // When
        let output = section.to_plain_text(&config);

        // Then
        assert!(output.contains("[Head]"));
        assert!(output.contains("Body"));
    }
}

// =============================================================================
// Feature: Builder Chaining
// =============================================================================

mod builder_chaining {
    use super::*;

    #[test]
    fn given_builder_when_chained_then_all_operations_applied() {
        // Given/When
        let mut builder = ExplainBuilder::new();
        builder
            .with_title("T")
            .with_summary("S")
            .add_section("H", "C")
            .add_bullet("B")
            .add_code_block("code", "rust")
            .add_table(&["H"], &[&["D"]])
            .add_text("T");

        // Then
        assert_eq!(builder.title(), Some("T"));
        assert_eq!(builder.summary(), Some("S"));
        assert_eq!(builder.sections().len(), 5);
    }

    #[test]
    fn given_builder_when_chained_then_output_complete() {
        // Given
        let mut builder = ExplainBuilder::new();
        builder
            .with_title("Title")
            .with_summary("Summary")
            .add_bullet("Bullet");

        // When
        let output = builder.build();

        // Then
        assert!(output.contains("# Title"));
        assert!(output.contains("*Summary*"));
        assert!(output.contains("- Bullet"));
    }
}

// =============================================================================
// Feature: Serde Support (optional)
// =============================================================================

#[cfg(feature = "serde")]
mod serde_support {
    use super::*;

    #[test]
    fn given_explain_config_when_serialized_then_valid_json() {
        // Given
        let config = ExplainConfig::new().with_indent(4).with_color(true);

        // When
        let json = serde_json::to_string(&config).unwrap();

        // Then
        assert!(json.contains("\"indent\":4"));
        assert!(json.contains("\"color\":true"));
    }

    #[test]
    fn given_json_when_deserialized_then_valid_config() {
        // Given
        let json = r#"{"indent":2,"line_width":100,"color":true}"#;

        // When
        let config: ExplainConfig = serde_json::from_str(json).unwrap();

        // Then
        assert_eq!(config.indent, 2);
        assert_eq!(config.line_width, 100);
        assert!(config.color);
    }

    #[test]
    fn given_explain_section_text_when_serialized_then_valid_json() {
        // Given
        let section = ExplainSection::text("Hello");

        // When
        let json = serde_json::to_string(&section).unwrap();

        // Then
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn given_explain_section_bullets_when_serialized_then_valid_json() {
        // Given
        let section = ExplainSection::bullets(vec!["A".to_string(), "B".to_string()]);

        // When
        let json = serde_json::to_string(&section).unwrap();

        // Then
        assert!(json.contains("\"type\":\"bullets\""));
        assert!(json.contains("A"));
        assert!(json.contains("B"));
    }

    #[test]
    fn given_explain_section_code_when_serialized_then_valid_json() {
        // Given
        let section = ExplainSection::code("fn main() {}", "rust");

        // When
        let json = serde_json::to_string(&section).unwrap();

        // Then
        assert!(json.contains("\"type\":\"code\""));
        assert!(json.contains("fn main() {}"));
        assert!(json.contains("rust"));
    }

    #[test]
    fn given_explain_section_table_when_serialized_then_valid_json() {
        // Given
        let section = ExplainSection::table(vec!["A".to_string()], vec![vec!["1".to_string()]]);

        // When
        let json = serde_json::to_string(&section).unwrap();

        // Then
        assert!(json.contains("\"type\":\"table\""));
        assert!(json.contains("headers"));
        assert!(json.contains("rows"));
    }
}

// =============================================================================
// Property-Based Tests
// =============================================================================

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_title_preserved(title in ".*") {
            let mut builder = ExplainBuilder::new();
            builder.with_title(&title);
            prop_assert_eq!(builder.title(), Some(title.as_str()));
        }

        #[test]
        fn prop_summary_preserved(summary in ".*") {
            let mut builder = ExplainBuilder::new();
            builder.with_summary(&summary);
            prop_assert_eq!(builder.summary(), Some(summary.as_str()));
        }

        #[test]
        fn prop_bullet_preserved(bullet in "[a-zA-Z0-9]+( [a-zA-Z0-9]+)*") {
            let mut builder = ExplainBuilder::new();
            builder.add_bullet(&bullet);
            let output = builder.build();
            prop_assert!(output.contains(&bullet));
        }

        #[test]
        fn prop_text_section_preserved(text in ".*") {
            let section = ExplainSection::text(&text);
            let config = ExplainConfig::default();
            let output = section.to_markdown(&config);
            prop_assert_eq!(output, text);
        }

        #[test]
        fn prop_indent_non_negative(indent in 0usize..100) {
            let config = ExplainConfig::new().with_indent(indent);
            prop_assert_eq!(config.indent, indent);
        }

        #[test]
        fn prop_line_width_non_negative(width in 1usize..1000) {
            let config = ExplainConfig::new().with_line_width(width);
            prop_assert_eq!(config.line_width, width);
        }

        #[test]
        fn prop_build_never_panics(
            title: Option<String>,
            summary: Option<String>,
            bullets: Vec<String>
        ) {
            let mut builder = ExplainBuilder::new();
            if let Some(t) = title {
                builder.with_title(&t);
            }
            if let Some(s) = summary {
                builder.with_summary(&s);
            }
            for bullet in bullets {
                builder.add_bullet(&bullet);
            }
            let _ = builder.build();
            let _ = builder.build_markdown();
            let _ = builder.build_plain_text();
        }

        #[test]
        fn prop_clear_resets_builder(
            title: String,
            summary: String,
            bullets: Vec<String>
        ) {
            let mut builder = ExplainBuilder::new();
            builder.with_title(&title).with_summary(&summary);
            for bullet in &bullets {
                builder.add_bullet(bullet);
            }
            builder.clear();
            prop_assert!(builder.is_empty());
        }

        #[test]
        fn prop_code_block_contains_code(code in ".*", language in "[a-z]*") {
            let mut builder = ExplainBuilder::new();
            builder.add_code_block(&code, &language);
            let output = builder.build();
            prop_assert!(output.contains(&code) || code.is_empty());
        }

        #[test]
        fn prop_multiple_sections_all_included(sections: Vec<String>) {
            let mut builder = ExplainBuilder::new();
            for section in &sections {
                builder.add_section(section, section);
            }
            let output = builder.build();
            for section in &sections {
                prop_assert!(output.contains(section) || section.is_empty());
            }
        }
    }
}

// =============================================================================
// Additional Coverage Tests
// =============================================================================

mod additional_coverage {
    use super::*;

    #[test]
    fn test_add_section_item() {
        let mut builder = ExplainBuilder::new();
        builder.add_section_item(ExplainSection::text("Test"));
        assert_eq!(builder.sections().len(), 1);
    }

    #[test]
    fn test_config_access() {
        let config = ExplainConfig::new().with_indent(4);
        let builder = ExplainBuilder::with_config(config);
        assert_eq!(builder.config().indent, 4);
    }

    #[test]
    fn test_sections_access() {
        let mut builder = ExplainBuilder::new();
        builder.add_text("A").add_text("B");
        assert_eq!(builder.sections().len(), 2);
    }

    #[test]
    fn test_build_default() {
        let builder = ExplainBuilder::default();
        let output = builder.build();
        assert!(output.is_empty() || output == "\n");
    }

    #[test]
    fn test_plain_text_multiline_code() {
        let section = ExplainSection::code("line1\nline2", "rust");
        let config = ExplainConfig::default();
        let output = section.to_plain_text(&config);
        assert!(output.contains("    line1"));
        assert!(output.contains("    line2"));
    }

    #[test]
    fn test_markdown_table_separator() {
        let section = ExplainSection::table(vec!["A".to_string(), "B".to_string()], vec![]);
        let config = ExplainConfig::default();
        let output = section.to_markdown(&config);
        // Should have header row and separator
        assert!(output.contains("| A |"));
        assert!(output.contains("---"));
    }

    #[test]
    fn test_plain_text_table_separator() {
        let section = ExplainSection::table(vec!["A".to_string(), "B".to_string()], vec![]);
        let config = ExplainConfig::default();
        let output = section.to_plain_text(&config);
        assert!(output.contains("-+-"));
    }

    #[test]
    fn test_table_row_with_fewer_columns() {
        let section = ExplainSection::table(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![vec!["1".to_string()]],
        );
        let config = ExplainConfig::default();
        let output = section.to_markdown(&config);
        // Should still render without panic
        assert!(output.contains("1"));
    }

    #[test]
    fn test_indent_plain_text_section() {
        let section = ExplainSection::section("H", "C");
        let config = ExplainConfig::new().with_indent(2);
        let output = section.to_plain_text(&config);
        assert!(output.contains("  [H]"));
    }

    #[test]
    fn test_indent_plain_text_table() {
        let section = ExplainSection::table(vec!["A".to_string()], vec![vec!["1".to_string()]]);
        let config = ExplainConfig::new().with_indent(2);
        let output = section.to_plain_text(&config);
        assert!(output.starts_with("  "));
    }

    #[test]
    fn test_indent_plain_text_code() {
        let section = ExplainSection::code("code", "rust");
        let config = ExplainConfig::new().with_indent(2);
        let output = section.to_plain_text(&config);
        assert!(output.contains("  [rust]"));
    }

    #[test]
    fn test_color_summary() {
        let config = ExplainConfig::new().with_color(true);
        let mut builder = ExplainBuilder::with_config(config);
        builder.with_summary("Summary");
        let output = builder.build();
        assert!(output.contains("\x1b[3;90m"));
    }

    #[test]
    fn test_empty_bullets_output() {
        let section = ExplainSection::bullets(vec![]);
        let config = ExplainConfig::default();
        let md = section.to_markdown(&config);
        let plain = section.to_plain_text(&config);
        assert!(md.is_empty());
        assert!(plain.is_empty());
    }

    #[test]
    fn test_builder_after_multiple_clears() {
        let mut builder = ExplainBuilder::new();
        builder.with_title("A");
        builder.clear();
        builder.with_title("B");
        builder.clear();
        builder.with_title("C");
        assert_eq!(builder.title(), Some("C"));
    }

    #[test]
    fn test_mixed_content_order_preserved() {
        let mut builder = ExplainBuilder::new();
        builder
            .with_title("Title")
            .add_bullet("B1")
            .add_text("Text")
            .add_bullet("B2")
            .add_code_block("code", "rust")
            .add_bullet("B3");

        let output = builder.build();
        let b1_pos = output.find("B1").unwrap();
        let text_pos = output.find("Text").unwrap();
        let b2_pos = output.find("B2").unwrap();
        let code_pos = output.find("code").unwrap();
        let b3_pos = output.find("B3").unwrap();

        assert!(b1_pos < text_pos);
        assert!(text_pos < b2_pos);
        assert!(b2_pos < code_pos);
        assert!(code_pos < b3_pos);
    }
}
