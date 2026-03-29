//! Comprehensive tests for lintdiff-render-utils.
//!
//! This test suite covers:
//! 1. IndentConfig methods (8 tests)
//! 2. WrapConfig methods (8 tests)
//! 3. pluralize function (5 tests)
//! 4. bullet_list and numbered_list (6 tests)
//! 5. table function (5 tests)
//! 6. truncate_with_ellipsis (5 tests)
//! 7. pad_left, pad_right, center (8 tests)
//! 8. TextBuilder methods (5 tests)

use lintdiff_render_utils::*;

// =============================================================================
// IndentConfig Tests (8 tests)
// =============================================================================

#[test]
fn test_indent_config_default_values() {
    let config = IndentConfig::default();
    assert_eq!(config.spaces, 2);
    assert_eq!(config.char, ' ');
}

#[test]
fn test_indent_config_new_custom_spaces() {
    let config = IndentConfig::new(4);
    assert_eq!(config.spaces, 4);
    assert_eq!(config.char, ' ');
}

#[test]
fn test_indent_config_indent_level_zero() {
    let config = IndentConfig::default();
    let result = config.indent(0);
    assert!(result.is_empty());
}

#[test]
fn test_indent_config_indent_level_one() {
    let config = IndentConfig::default();
    let result = config.indent(1);
    assert_eq!(result, "  ");
}

#[test]
fn test_indent_config_indent_multiple_levels() {
    let config = IndentConfig::new(4);
    assert_eq!(config.indent(1), "    ");
    assert_eq!(config.indent(2), "        ");
    assert_eq!(config.indent(3), "            ");
}

#[test]
fn test_indent_config_indent_string_single_line() {
    let config = IndentConfig::default();
    let result = config.indent_string("hello", 1);
    assert_eq!(result, "  hello");
}

#[test]
fn test_indent_config_indent_string_multi_line() {
    let config = IndentConfig::default();
    let input = "line1\nline2\nline3";
    let result = config.indent_string(input, 1);
    assert_eq!(result, "  line1\n  line2\n  line3");
}

#[test]
fn test_indent_config_indent_string_empty() {
    let config = IndentConfig::default();
    let result = config.indent_string("", 1);
    // Empty string has no lines, so result is empty
    assert_eq!(result, "");
}

// =============================================================================
// WrapConfig Tests (8 tests)
// =============================================================================

#[test]
fn test_wrap_config_default_values() {
    let config = WrapConfig::default();
    assert_eq!(config.width, 80);
    assert!(config.initial_indent.is_empty());
    assert!(config.subsequent_indent.is_empty());
    assert!(!config.break_long_words);
}

#[test]
fn test_wrap_config_new_custom_width() {
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
fn test_wrap_config_wrap_short_text() {
    let config = WrapConfig::new(80);
    let text = "This is a short line.";
    let result = config.wrap(text);
    assert_eq!(result, text);
}

#[test]
fn test_wrap_config_wrap_long_text() {
    let config = WrapConfig::new(20);
    let text = "This is a longer line that should be wrapped";
    let result = config.wrap(text);
    // The text should be split into multiple lines
    assert!(result.contains('\n'));
}

#[test]
fn test_wrap_config_wrap_empty_text() {
    let config = WrapConfig::new(80);
    let result = config.wrap("");
    assert!(result.is_empty());
}

#[test]
fn test_wrap_config_wrap_with_indents() {
    let config = WrapConfig::new(20)
        .with_initial_indent("> ")
        .with_subsequent_indent("  ");
    let text = "This is text that will wrap";
    let result = config.wrap(text);
    // First line should start with "> "
    assert!(result.starts_with("> "));
}

// =============================================================================
// pluralize Tests (5 tests)
// =============================================================================

#[test]
fn test_pluralize_count_one() {
    let result = pluralize(1, "item", None);
    assert_eq!(result, "1 item");
}

#[test]
fn test_pluralize_count_multiple() {
    let result = pluralize(5, "item", None);
    assert_eq!(result, "5 items");
}

#[test]
fn test_pluralize_count_zero() {
    let result = pluralize(0, "item", None);
    assert_eq!(result, "0 items");
}

#[test]
fn test_pluralize_custom_plural() {
    let result = pluralize(2, "child", Some("children"));
    assert_eq!(result, "2 children");
}

#[test]
fn test_pluralize_custom_plural_single() {
    let result = pluralize(1, "child", Some("children"));
    assert_eq!(result, "1 child");
}

// =============================================================================
// bullet_list and numbered_list Tests (6 tests)
// =============================================================================

#[test]
fn test_bullet_list_basic() {
    let items = vec!["apple", "banana", "cherry"];
    let result = bullet_list(&items, "-");
    assert_eq!(result, "- apple\n- banana\n- cherry");
}

#[test]
fn test_bullet_list_custom_bullet() {
    let items = vec!["first", "second"];
    let result = bullet_list(&items, "*");
    assert_eq!(result, "* first\n* second");
}

#[test]
fn test_bullet_list_empty() {
    let items: Vec<&str> = vec![];
    let result = bullet_list(&items, "-");
    assert!(result.is_empty());
}

#[test]
fn test_numbered_list_basic() {
    let items = vec!["one", "two", "three"];
    let result = numbered_list(&items);
    assert_eq!(result, "1. one\n2. two\n3. three");
}

#[test]
fn test_numbered_list_single_item() {
    let items = vec!["only"];
    let result = numbered_list(&items);
    assert_eq!(result, "1. only");
}

#[test]
fn test_numbered_list_empty() {
    let items: Vec<&str> = vec![];
    let result = numbered_list(&items);
    assert!(result.is_empty());
}

// =============================================================================
// table Tests (5 tests)
// =============================================================================

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
fn test_table_multiple_rows() {
    let headers = vec!["ID", "Value"];
    let rows = vec![vec!["1", "a"], vec!["2", "b"], vec!["3", "c"]];
    let result = table(&headers, &rows);

    assert!(result.contains("1"));
    assert!(result.contains("2"));
    assert!(result.contains("3"));
}

#[test]
fn test_table_empty_rows() {
    let headers = vec!["Col1", "Col2"];
    let rows: Vec<Vec<&str>> = vec![];
    let result = table(&headers, &rows);

    // Should still have header and separator
    assert!(result.contains("Col1"));
    assert!(result.contains('|'));
}

#[test]
fn test_table_column_width_alignment() {
    let headers = vec!["Short", "Very Long Header"];
    let rows = vec![vec!["x", "y"]];
    let result = table(&headers, &rows);

    // Check that separator line is properly formed
    let lines: Vec<&str> = result.lines().collect();
    assert!(lines.len() >= 2);
}

#[test]
fn test_table_empty_headers() {
    let headers: Vec<&str> = vec![];
    let rows = vec![vec!["a", "b"]];
    let result = table(&headers, &rows);

    assert!(result.is_empty());
}

// =============================================================================
// truncate_with_ellipsis Tests (5 tests)
// =============================================================================

#[test]
fn test_truncate_no_truncation_needed() {
    let result = truncate_with_ellipsis("hello", 10);
    assert_eq!(result, "hello");
}

#[test]
fn test_truncate_exact_length() {
    let result = truncate_with_ellipsis("hello", 5);
    assert_eq!(result, "hello");
}

#[test]
fn test_truncate_truncation_occurs() {
    let result = truncate_with_ellipsis("hello world", 8);
    assert_eq!(result, "hello...");
}

#[test]
fn test_truncate_very_short_max_len() {
    let result = truncate_with_ellipsis("hello", 2);
    assert_eq!(result, "..");
}

#[test]
fn test_truncate_max_len_three() {
    let result = truncate_with_ellipsis("hello", 3);
    assert_eq!(result, "...");
}

// =============================================================================
// pad_left, pad_right, center Tests (8 tests)
// =============================================================================

#[test]
fn test_pad_left_basic() {
    let result = pad_left("42", 5, '0');
    assert_eq!(result, "00042");
}

#[test]
fn test_pad_left_no_padding_needed() {
    let result = pad_left("hello", 3, ' ');
    assert_eq!(result, "hello");
}

#[test]
fn test_pad_left_exact_width() {
    let result = pad_left("hi", 2, '-');
    assert_eq!(result, "hi");
}

#[test]
fn test_pad_right_basic() {
    let result = pad_right("hello", 10, '.');
    assert_eq!(result, "hello.....");
}

#[test]
fn test_pad_right_no_padding_needed() {
    let result = pad_right("hello", 3, ' ');
    assert_eq!(result, "hello");
}

#[test]
fn test_center_even_padding() {
    let result = center("hi", 6, '-');
    assert_eq!(result, "--hi--");
}

#[test]
fn test_center_odd_padding() {
    let result = center("hi", 5, '-');
    // Left gets less padding when odd
    assert_eq!(result, "-hi--");
}

#[test]
fn test_center_no_padding_needed() {
    let result = center("hello", 3, ' ');
    assert_eq!(result, "hello");
}

// =============================================================================
// TextBuilder Tests (5 tests)
// =============================================================================

#[test]
fn test_text_builder_empty() {
    let builder = TextBuilder::new();
    assert!(builder.build().is_empty());
}

#[test]
fn test_text_builder_single_line() {
    let mut builder = TextBuilder::new();
    builder.line("hello");
    assert_eq!(builder.build(), "hello");
}

#[test]
fn test_text_builder_multiple_lines() {
    let mut builder = TextBuilder::new();
    builder.line("line1").line("line2").line("line3");
    assert_eq!(builder.build(), "line1\nline2\nline3");
}

#[test]
fn test_text_builder_with_indentation() {
    let mut builder = TextBuilder::new().with_indent(IndentConfig::new(2));
    builder
        .line("root")
        .indent()
        .line("child")
        .dedent()
        .line("back");
    assert_eq!(builder.build(), "root\n  child\nback");
}

#[test]
fn test_text_builder_empty_line() {
    let mut builder = TextBuilder::new();
    builder.line("first").empty_line().line("second");
    assert_eq!(builder.build(), "first\n\nsecond");
}

// =============================================================================
// Additional Edge Case Tests
// =============================================================================

#[test]
fn test_indent_config_clone() {
    let config = IndentConfig::new(4);
    let cloned = config.clone();
    assert_eq!(config, cloned);
}

#[test]
fn test_indent_config_partial_eq() {
    let config1 = IndentConfig::new(4);
    let config2 = IndentConfig::new(4);
    let config3 = IndentConfig::new(2);

    assert_eq!(config1, config2);
    assert_ne!(config1, config3);
}

#[test]
fn test_wrap_config_clone() {
    let config = WrapConfig::new(40);
    let cloned = config.clone();
    assert_eq!(config, cloned);
}

#[test]
fn test_wrap_config_partial_eq() {
    let config1 = WrapConfig::new(40);
    let config2 = WrapConfig::new(40);
    let config3 = WrapConfig::new(80);

    assert_eq!(config1, config2);
    assert_ne!(config1, config3);
}

#[test]
fn test_text_builder_display_trait() {
    let mut builder = TextBuilder::new();
    builder.line("hello").line("world");
    let display = format!("{builder}");
    assert_eq!(display, "hello\nworld");
}

#[test]
fn test_text_builder_clone() {
    let mut builder = TextBuilder::new();
    builder.line("test");
    let cloned = builder.clone();
    assert_eq!(builder.build(), cloned.build());
}

#[test]
fn test_text_builder_default() {
    let builder1 = TextBuilder::new();
    let builder2 = TextBuilder::default();
    assert_eq!(builder1.build(), builder2.build());
}

#[test]
fn test_bullet_list_single_item() {
    let items = vec!["only"];
    let result = bullet_list(&items, "*");
    assert_eq!(result, "* only");
}

#[test]
fn test_numbered_list_many_items() {
    let items = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
    let result = numbered_list(&items);
    assert!(result.contains("10. j"));
}

#[test]
fn test_truncate_empty_string() {
    let result = truncate_with_ellipsis("", 5);
    assert_eq!(result, "");
}

#[test]
fn test_truncate_zero_max_len() {
    let result = truncate_with_ellipsis("hello", 0);
    assert_eq!(result, "");
}

#[test]
fn test_pad_left_empty_string() {
    let result = pad_left("", 5, 'x');
    assert_eq!(result, "xxxxx");
}

#[test]
fn test_pad_right_empty_string() {
    let result = pad_right("", 5, 'x');
    assert_eq!(result, "xxxxx");
}

#[test]
fn test_center_empty_string() {
    let result = center("", 5, 'x');
    assert_eq!(result, "xxxxx");
}

#[test]
fn test_indent_config_tab_char() {
    let config = IndentConfig {
        spaces: 1,
        char: '\t',
    };
    assert_eq!(config.indent(2), "\t\t");
}

#[test]
fn test_text_builder_nested_indent() {
    let mut builder = TextBuilder::new().with_indent(IndentConfig::new(2));
    builder
        .line("0")
        .indent()
        .line("1")
        .indent()
        .line("2")
        .dedent()
        .dedent()
        .line("0");
    assert_eq!(builder.build(), "0\n  1\n    2\n0");
}

#[test]
fn test_text_builder_dedent_at_zero() {
    let mut builder = TextBuilder::new();
    builder.dedent(); // Should not panic
    builder.line("test");
    assert_eq!(builder.build(), "test");
}

#[test]
fn test_table_row_fewer_columns() {
    let headers = vec!["A", "B", "C"];
    let rows = vec![vec!["1", "2"]]; // Only 2 columns
    let result = table(&headers, &rows);
    // Should handle gracefully
    assert!(result.contains("A"));
    assert!(result.contains("B"));
    assert!(result.contains("C"));
}

#[test]
fn test_wrap_single_word() {
    let config = WrapConfig::new(10);
    let result = config.wrap("hello");
    assert_eq!(result, "hello");
}

#[test]
fn test_wrap_multiple_short_words() {
    let config = WrapConfig::new(20);
    let result = config.wrap("a b c d e f g h i j");
    assert_eq!(result, "a b c d e f g h i j");
}

#[test]
fn test_pluralize_large_number() {
    let result = pluralize(1000, "error", None);
    assert_eq!(result, "1000 errors");
}

#[test]
fn test_pluralize_irregular_plural() {
    let result = pluralize(2, "person", Some("people"));
    assert_eq!(result, "2 people");
}

#[test]
fn test_indent_string_preserves_empty_lines() {
    let config = IndentConfig::default();
    let input = "line1\n\nline3";
    let result = config.indent_string(input, 1);
    assert_eq!(result, "  line1\n  \n  line3");
}

#[test]
fn test_center_single_char() {
    let result = center("x", 5, '-');
    assert_eq!(result, "--x--");
}

#[test]
fn test_pad_left_various_chars() {
    let result = pad_left("test", 8, '.');
    assert_eq!(result, "....test");
}

#[test]
fn test_pad_right_various_chars() {
    let result = pad_right("test", 8, '_');
    assert_eq!(result, "test____");
}

#[test]
fn test_text_builder_chaining() {
    let mut builder = TextBuilder::new();
    builder
        .line("first")
        .empty_line()
        .indent()
        .line("indented")
        .dedent()
        .line("last");
    assert_eq!(builder.build(), "first\n\n  indented\nlast");
}

#[test]
fn test_bullet_list_different_bullets() {
    let items = vec!["item"];

    assert_eq!(bullet_list(&items, "-"), "- item");
    assert_eq!(bullet_list(&items, "*"), "* item");
    assert_eq!(bullet_list(&items, "+"), "+ item");
    assert_eq!(bullet_list(&items, ">"), "> item");
}

#[test]
fn test_table_special_characters() {
    let headers = vec!["Col1", "Col2"];
    let rows = vec![vec!["a|b", "c&d"]];
    let result = table(&headers, &rows);
    // Should contain the special chars
    assert!(result.contains("a|b"));
    assert!(result.contains("c&d"));
}
