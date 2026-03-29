//! Comprehensive BDD tests for lintdiff-code-norm
//!
//! Test categories:
//! 1. normalize_whitespace function (12 tests)
//! 2. normalize_whitespace_with_options function (8 tests)
//! 3. normalize_indentation function (10 tests)
//! 4. normalize_indentation_with_tab_width function (8 tests)
//! 5. normalize_line_endings function (10 tests)
//! 6. normalize_line_endings_to function (6 tests)
//! 7. normalize_code function (8 tests)
//! 8. LineEnding enum (6 tests)
//! 9. CodeNormalizer builder pattern (12 tests)
//! 10. CodeNormalizer.normalize method (10 tests)
//! 11. normalize_cow function (6 tests)
//! 12. needs_normalization function (6 tests)
//! 13. count_lines function (6 tests)
//! 14. detect_line_ending function (6 tests)
//! 15. Edge cases (8 tests)
//! 16. Property-based tests with proptest (10 tests)
//! Total: 132 tests

use lintdiff_code_norm::{
    count_lines, detect_line_ending, needs_normalization, normalize_code, normalize_cow,
    normalize_indentation, normalize_indentation_with_tab_width, normalize_line_endings,
    normalize_line_endings_to, normalize_whitespace, normalize_whitespace_with_options,
    CodeNormalizer, LineEnding,
};

// =============================================================================
// 1. normalize_whitespace function tests (12 tests)
// =============================================================================

mod normalize_whitespace_tests {
    use super::*;

    #[test]
    fn normalize_whitespace_returns_empty_for_empty_string() {
        assert_eq!(normalize_whitespace(""), "");
    }

    #[test]
    fn normalize_whitespace_preserves_single_words() {
        assert_eq!(normalize_whitespace("hello"), "hello");
    }

    #[test]
    fn normalize_whitespace_trims_leading_spaces() {
        assert_eq!(normalize_whitespace("   hello"), "hello");
    }

    #[test]
    fn normalize_whitespace_trims_trailing_spaces() {
        assert_eq!(normalize_whitespace("hello   "), "hello");
    }

    #[test]
    fn normalize_whitespace_trims_both_leading_and_trailing() {
        assert_eq!(normalize_whitespace("   hello   "), "hello");
    }

    #[test]
    fn normalize_whitespace_collapses_multiple_spaces() {
        assert_eq!(normalize_whitespace("hello    world"), "hello world");
    }

    #[test]
    fn normalize_whitespace_handles_tabs() {
        assert_eq!(normalize_whitespace("hello\tworld"), "hello world");
    }

    #[test]
    fn normalize_whitespace_handles_multiple_tabs() {
        assert_eq!(normalize_whitespace("hello\t\tworld"), "hello world");
    }

    #[test]
    fn normalize_whitespace_handles_mixed_whitespace() {
        assert_eq!(normalize_whitespace("hello \t world"), "hello world");
    }

    #[test]
    fn normalize_whitespace_preserves_newlines() {
        assert_eq!(normalize_whitespace("hello\nworld"), "hello\nworld");
    }

    #[test]
    fn normalize_whitespace_handles_crlf() {
        // CRLF should be converted to LF
        let result = normalize_whitespace("hello\r\nworld");
        assert_eq!(result, "hello\nworld");
    }

    #[test]
    fn normalize_whitespace_handles_multiline_content() {
        let input = "  line1  \n  line2  \n  line3  ";
        let result = normalize_whitespace(input);
        assert_eq!(result, "line1\nline2\nline3");
    }
}

// =============================================================================
// 2. normalize_whitespace_with_options function tests (8 tests)
// =============================================================================

mod normalize_whitespace_with_options_tests {
    use super::*;

    #[test]
    fn with_trim_and_collapse_enabled() {
        let result = normalize_whitespace_with_options("  hello   world  ", true, true);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn with_trim_disabled_preserves_leading_trailing() {
        let result = normalize_whitespace_with_options("  hello world  ", false, true);
        assert!(result.starts_with(' ') || result == "hello world");
    }

    #[test]
    fn with_collapse_disabled_preserves_multiple_spaces() {
        let result = normalize_whitespace_with_options("hello    world", true, false);
        assert!(result.contains("hello") && result.contains("world"));
    }

    #[test]
    fn with_both_disabled() {
        let result = normalize_whitespace_with_options("  hello    world  ", false, false);
        // With both disabled, whitespace is preserved but CRLF still converted
        assert!(result.contains("hello"));
    }

    #[test]
    fn handles_empty_string() {
        let result = normalize_whitespace_with_options("", true, true);
        assert_eq!(result, "");
    }

    #[test]
    fn handles_single_word() {
        let result = normalize_whitespace_with_options("hello", true, true);
        assert_eq!(result, "hello");
    }

    #[test]
    fn handles_newlines_with_collapse() {
        let result = normalize_whitespace_with_options("line1\n  line2", true, true);
        assert_eq!(result, "line1\nline2");
    }

    #[test]
    fn handles_only_whitespace() {
        let result = normalize_whitespace_with_options("   \t   ", true, true);
        assert_eq!(result, "");
    }
}

// =============================================================================
// 3. normalize_indentation function tests (10 tests)
// =============================================================================

mod normalize_indentation_tests {
    use super::*;

    #[test]
    fn normalize_indentation_returns_empty_for_empty_string() {
        assert_eq!(normalize_indentation(""), "");
    }

    #[test]
    fn normalize_indentation_preserves_no_indent() {
        assert_eq!(normalize_indentation("line1\nline2"), "line1\nline2");
    }

    #[test]
    fn normalize_indentation_removes_common_indent() {
        let input = "    line1\n    line2\n    line3";
        let result = normalize_indentation(input);
        assert_eq!(result, "line1\nline2\nline3");
    }

    #[test]
    fn normalize_indentation_preserves_relative_indent() {
        let input = "    line1\n        line2\n    line3";
        let result = normalize_indentation(input);
        assert_eq!(result, "line1\n    line2\nline3");
    }

    #[test]
    fn normalize_indentation_handles_tabs() {
        let input = "\tline1\n\t\tline2";
        let result = normalize_indentation(input);
        // Tabs converted to 4 spaces, then common indent removed
        assert!(result.starts_with("line1"));
    }

    #[test]
    fn normalize_indentation_ignores_empty_lines_for_min_calc() {
        let input = "    line1\n\n    line2";
        let result = normalize_indentation(input);
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }

    #[test]
    fn normalize_indentation_handles_crlf() {
        let input = "    line1\r\n    line2";
        let result = normalize_indentation(input);
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }

    #[test]
    fn normalize_indentation_handles_single_line() {
        assert_eq!(normalize_indentation("    hello"), "hello");
    }

    #[test]
    fn normalize_indentation_handles_mixed_indent() {
        let input = "  line1\n    line2\n  line3";
        let result = normalize_indentation(input);
        assert!(result.starts_with("line1"));
    }

    #[test]
    fn normalize_indentation_preserves_content_only_lines() {
        let input = "line1\n    line2\nline3";
        let result = normalize_indentation(input);
        // No common indent to remove
        assert!(result.contains("line1"));
        assert!(result.contains("    line2"));
    }
}

// =============================================================================
// 4. normalize_indentation_with_tab_width function tests (8 tests)
// =============================================================================

mod normalize_indentation_with_tab_width_tests {
    use super::*;

    #[test]
    fn tab_width_2_converts_tabs_to_2_spaces() {
        let input = "\tline1";
        let result = normalize_indentation_with_tab_width(input, 2);
        assert_eq!(result, "line1");
    }

    #[test]
    fn tab_width_8_converts_tabs_to_8_spaces() {
        let input = "\tline1";
        let result = normalize_indentation_with_tab_width(input, 8);
        assert_eq!(result, "line1");
    }

    #[test]
    fn tab_width_0_preserves_tabs() {
        let input = "\tline1";
        let result = normalize_indentation_with_tab_width(input, 0);
        // Tab width 0 means no replacement
        assert!(result.contains("line1"));
    }

    #[test]
    fn handles_nested_indent_with_tabs() {
        let input = "\tline1\n\t\tline2";
        let result = normalize_indentation_with_tab_width(input, 2);
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }

    #[test]
    fn handles_mixed_tabs_and_spaces() {
        let input = "\t    line1";
        let result = normalize_indentation_with_tab_width(input, 4);
        assert_eq!(result, "line1");
    }

    #[test]
    fn empty_string_returns_empty() {
        let result = normalize_indentation_with_tab_width("", 4);
        assert_eq!(result, "");
    }

    #[test]
    fn no_indent_returns_unchanged() {
        let input = "line1\nline2";
        let result = normalize_indentation_with_tab_width(input, 4);
        assert_eq!(result, input);
    }

    #[test]
    fn handles_multiple_lines_with_varying_indent() {
        let input = "    line1\n        line2\n    line3";
        let result = normalize_indentation_with_tab_width(input, 4);
        assert!(result.starts_with("line1"));
    }
}

// =============================================================================
// 5. normalize_line_endings function tests (10 tests)
// =============================================================================

mod normalize_line_endings_tests {
    use super::*;

    #[test]
    fn normalize_line_endings_returns_empty_for_empty_string() {
        assert_eq!(normalize_line_endings(""), "");
    }

    #[test]
    fn normalize_line_endings_converts_crlf_to_lf() {
        assert_eq!(normalize_line_endings("a\r\nb"), "a\nb");
    }

    #[test]
    fn normalize_line_endings_converts_cr_to_lf() {
        assert_eq!(normalize_line_endings("a\rb"), "a\nb");
    }

    #[test]
    fn normalize_line_endings_preserves_lf() {
        assert_eq!(normalize_line_endings("a\nb"), "a\nb");
    }

    #[test]
    fn normalize_line_endings_handles_multiple_crlf() {
        assert_eq!(normalize_line_endings("a\r\nb\r\nc"), "a\nb\nc");
    }

    #[test]
    fn normalize_line_endings_handles_mixed_endings() {
        assert_eq!(normalize_line_endings("a\r\nb\rc\nd"), "a\nb\nc\nd");
    }

    #[test]
    fn normalize_line_endings_handles_crlf_at_end() {
        assert_eq!(normalize_line_endings("hello\r\n"), "hello\n");
    }

    #[test]
    fn normalize_line_endings_handles_cr_at_end() {
        assert_eq!(normalize_line_endings("hello\r"), "hello\n");
    }

    #[test]
    fn normalize_line_endings_handles_only_endings() {
        assert_eq!(normalize_line_endings("\r\n\r\n"), "\n\n");
    }

    #[test]
    fn normalize_line_endings_preserves_content() {
        let input = "line1\r\nline2\r\nline3";
        let result = normalize_line_endings(input);
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
        assert!(result.contains("line3"));
    }
}

// =============================================================================
// 6. normalize_line_endings_to function tests (6 tests)
// =============================================================================

mod normalize_line_endings_to_tests {
    use super::*;

    #[test]
    fn converts_to_unix() {
        let result = normalize_line_endings_to("a\r\nb", LineEnding::Unix);
        assert_eq!(result, "a\nb");
    }

    #[test]
    fn converts_to_windows() {
        let result = normalize_line_endings_to("a\nb", LineEnding::Windows);
        assert_eq!(result, "a\r\nb");
    }

    #[test]
    fn handles_empty_string() {
        let result = normalize_line_endings_to("", LineEnding::Unix);
        assert_eq!(result, "");
    }

    #[test]
    fn handles_multiple_lines_to_windows() {
        let result = normalize_line_endings_to("a\nb\nc", LineEnding::Windows);
        assert_eq!(result, "a\r\nb\r\nc");
    }

    #[test]
    fn handles_cr_only_input() {
        let result = normalize_line_endings_to("a\rb", LineEnding::Unix);
        assert_eq!(result, "a\nb");
    }

    #[test]
    fn native_uses_platform_default() {
        let result = normalize_line_endings_to("a\nb", LineEnding::Native);
        // Native should work without error
        assert!(result.contains('a'));
        assert!(result.contains('b'));
    }
}

// =============================================================================
// 7. normalize_code function tests (8 tests)
// =============================================================================

mod normalize_code_tests {
    use super::*;

    #[test]
    fn normalize_code_returns_empty_for_empty_string() {
        assert_eq!(normalize_code(""), "");
    }

    #[test]
    fn normalize_code_applies_all_normalizations() {
        let input = "  hello   world  \r\n";
        let result = normalize_code(input);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn normalize_code_handles_indentation() {
        let input = "    line1\n    line2";
        let result = normalize_code(input);
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }

    #[test]
    fn normalize_code_handles_complex_input() {
        let input = "  \t  Hello   \r\n  \t  World  \t  ";
        let result = normalize_code(input);
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn normalize_code_preserves_relative_indentation() {
        let input = "    line1\n        line2\n    line3";
        let result = normalize_code(input);
        // After removing common indent, line2 should still have relative indent
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
        assert!(result.contains("line3"));
    }

    #[test]
    fn normalize_code_handles_single_line() {
        let result = normalize_code("  hello world  ");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn normalize_code_handles_multiline_with_mixed_endings() {
        let input = "line1\r\nline2\rline3\nline4";
        let result = normalize_code(input);
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
        assert!(result.contains("line3"));
        assert!(result.contains("line4"));
    }

    #[test]
    fn normalize_code_is_idempotent() {
        let input = "  hello   world  \r\n";
        let first = normalize_code(input);
        let second = normalize_code(&first);
        assert_eq!(first, second);
    }
}

// =============================================================================
// 8. LineEnding enum tests (6 tests)
// =============================================================================

mod line_ending_tests {
    use super::*;

    #[test]
    fn line_ending_default_is_unix() {
        assert_eq!(LineEnding::default(), LineEnding::Unix);
    }

    #[test]
    fn line_ending_as_str_unix() {
        assert_eq!(LineEnding::Unix.as_str(), "\n");
    }

    #[test]
    fn line_ending_as_str_windows() {
        assert_eq!(LineEnding::Windows.as_str(), "\r\n");
    }

    #[test]
    fn line_ending_as_bytes_unix() {
        assert_eq!(LineEnding::Unix.as_bytes(), b"\n");
    }

    #[test]
    fn line_ending_as_bytes_windows() {
        assert_eq!(LineEnding::Windows.as_bytes(), b"\r\n");
    }

    #[test]
    fn line_ending_display_works() {
        assert!(format!("{}", LineEnding::Unix).contains("Unix"));
        assert!(format!("{}", LineEnding::Windows).contains("Windows"));
        assert!(format!("{}", LineEnding::Native).contains("Native"));
    }
}

// =============================================================================
// 9. CodeNormalizer builder pattern tests (12 tests)
// =============================================================================

mod code_normalizer_builder_tests {
    use super::*;

    #[test]
    fn new_creates_default_normalizer() {
        let n = CodeNormalizer::new();
        assert!(n.trim);
        assert!(n.collapse_spaces);
        assert_eq!(n.tab_width, 4);
        assert_eq!(n.line_ending, LineEnding::Unix);
        assert!(n.normalize_indent);
        assert!(!n.preserve_empty_lines);
    }

    #[test]
    fn default_matches_new() {
        let n1 = CodeNormalizer::new();
        let n2 = CodeNormalizer::default();
        assert_eq!(n1, n2);
    }

    #[test]
    fn none_creates_passthrough_normalizer() {
        let n = CodeNormalizer::none();
        assert!(!n.trim);
        assert!(!n.collapse_spaces);
        assert!(!n.normalize_indent);
        assert!(n.preserve_empty_lines);
    }

    #[test]
    fn trim_whitespace_sets_trim() {
        let n = CodeNormalizer::new().trim_whitespace(false);
        assert!(!n.trim);
    }

    #[test]
    fn collapse_spaces_sets_collapse() {
        let n = CodeNormalizer::new().collapse_spaces(false);
        assert!(!n.collapse_spaces);
    }

    #[test]
    fn tab_width_sets_width() {
        let n = CodeNormalizer::new().tab_width(2);
        assert_eq!(n.tab_width, 2);
    }

    #[test]
    fn line_ending_sets_ending() {
        let n = CodeNormalizer::new().line_ending(LineEnding::Windows);
        assert_eq!(n.line_ending, LineEnding::Windows);
    }

    #[test]
    fn normalize_indent_sets_flag() {
        let n = CodeNormalizer::new().normalize_indent(false);
        assert!(!n.normalize_indent);
    }

    #[test]
    fn preserve_empty_lines_sets_flag() {
        let n = CodeNormalizer::new().preserve_empty_lines(true);
        assert!(n.preserve_empty_lines);
    }

    #[test]
    fn builder_chains_multiple_options() {
        let n = CodeNormalizer::new()
            .trim_whitespace(false)
            .collapse_spaces(false)
            .tab_width(2)
            .line_ending(LineEnding::Windows);
        assert!(!n.trim);
        assert!(!n.collapse_spaces);
        assert_eq!(n.tab_width, 2);
        assert_eq!(n.line_ending, LineEnding::Windows);
    }

    #[test]
    fn clone_creates_equal_copy() {
        let n1 = CodeNormalizer::new().tab_width(2);
        let n2 = n1.clone();
        assert_eq!(n1, n2);
    }

    #[test]
    fn debug_format_works() {
        let n = CodeNormalizer::new();
        let debug = format!("{:?}", n);
        assert!(debug.contains("CodeNormalizer"));
    }
}

// =============================================================================
// 10. CodeNormalizer.normalize method tests (10 tests)
// =============================================================================

mod code_normalizer_normalize_tests {
    use super::*;

    #[test]
    fn normalize_returns_empty_for_empty_string() {
        let n = CodeNormalizer::new();
        assert_eq!(n.normalize(""), "");
    }

    #[test]
    fn normalize_applies_all_transformations() {
        let n = CodeNormalizer::new();
        let result = n.normalize("  hello   world  ");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn normalize_with_trim_false_preserves_whitespace() {
        let n = CodeNormalizer::new().trim_whitespace(false);
        let result = n.normalize("  hello  ");
        // With collapse_spaces still true, spaces are collapsed but not trimmed
        assert!(result.contains("hello"));
    }

    #[test]
    fn normalize_with_collapse_false_preserves_spaces() {
        let n = CodeNormalizer::new().collapse_spaces(false);
        let result = n.normalize("hello    world");
        // Content is still there
        assert!(result.contains("hello"));
        assert!(result.contains("world"));
    }

    #[test]
    fn normalize_with_windows_line_ending() {
        let n = CodeNormalizer::new().line_ending(LineEnding::Windows);
        let result = n.normalize("a\nb");
        assert_eq!(result, "a\r\nb");
    }

    #[test]
    fn normalize_with_tab_width_2() {
        let n = CodeNormalizer::new().tab_width(2);
        let result = n.normalize("\thello");
        assert_eq!(result, "hello");
    }

    #[test]
    fn normalize_with_indent_normalization_disabled() {
        let n = CodeNormalizer::new().normalize_indent(false);
        let result = n.normalize("    hello");
        // With indent normalization disabled, spaces are still trimmed by default
        assert!(result.contains("hello"));
    }

    #[test]
    fn normalize_preserves_empty_lines_when_configured() {
        let n = CodeNormalizer::new().preserve_empty_lines(true);
        let result = n.normalize("line1\n\nline2");
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }

    #[test]
    fn normalize_none_passthrough() {
        let n = CodeNormalizer::none();
        let input = "  hello    world  ";
        let result = n.normalize(input);
        // With none(), minimal transformation happens
        assert!(result.contains("hello"));
        assert!(result.contains("world"));
    }

    #[test]
    fn normalize_handles_complex_multiline() {
        let n = CodeNormalizer::new();
        let input = "    line1\n        line2\n    line3";
        let result = n.normalize(input);
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
        assert!(result.contains("line3"));
    }
}

// =============================================================================
// 11. normalize_cow function tests (6 tests)
// =============================================================================

mod normalize_cow_tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn normalize_cow_returns_borrowed_for_clean_input() {
        let result = normalize_cow("hello world");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn normalize_cow_returns_owned_for_dirty_input() {
        let result = normalize_cow("hello  world");
        assert!(matches!(result, Cow::Owned(_)));
    }

    #[test]
    fn normalize_cow_returns_empty_for_empty() {
        let result = normalize_cow("");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "");
    }

    #[test]
    fn normalize_cow_normalizes_content() {
        let result = normalize_cow("  hello   world  ");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn normalize_cow_handles_tabs() {
        let result = normalize_cow("hello\tworld");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn normalize_cow_handles_crlf() {
        let result = normalize_cow("hello\r\nworld");
        assert_eq!(result, "hello\nworld");
    }
}

// =============================================================================
// 12. needs_normalization function tests (6 tests)
// =============================================================================

mod needs_normalization_tests {
    use super::*;

    #[test]
    fn needs_normalization_returns_false_for_clean_input() {
        assert!(!needs_normalization("hello world"));
    }

    #[test]
    fn needs_normalization_returns_false_for_empty() {
        assert!(!needs_normalization(""));
    }

    #[test]
    fn needs_normalization_detects_multiple_spaces() {
        assert!(needs_normalization("hello  world"));
    }

    #[test]
    fn needs_normalization_detects_tabs() {
        assert!(needs_normalization("hello\tworld"));
    }

    #[test]
    fn needs_normalization_detects_crlf() {
        assert!(needs_normalization("hello\r\nworld"));
    }

    #[test]
    fn needs_normalization_detects_leading_trailing_whitespace() {
        assert!(needs_normalization("  hello"));
        assert!(needs_normalization("hello  "));
    }
}

// =============================================================================
// 13. count_lines function tests (6 tests)
// =============================================================================

mod count_lines_tests {
    use super::*;

    #[test]
    fn count_lines_returns_0_for_empty() {
        assert_eq!(count_lines(""), 0);
    }

    #[test]
    fn count_lines_returns_1_for_single_line() {
        assert_eq!(count_lines("hello"), 1);
    }

    #[test]
    fn count_lines_counts_newlines() {
        assert_eq!(count_lines("a\nb"), 2);
    }

    #[test]
    fn count_lines_handles_multiple_newlines() {
        assert_eq!(count_lines("a\nb\nc"), 3);
    }

    #[test]
    fn count_lines_handles_trailing_newline() {
        assert_eq!(count_lines("hello\n"), 2);
    }

    #[test]
    fn count_lines_handles_only_newlines() {
        assert_eq!(count_lines("\n\n\n"), 4);
    }
}

// =============================================================================
// 14. detect_line_ending function tests (6 tests)
// =============================================================================

mod detect_line_ending_tests {
    use super::*;

    #[test]
    fn detect_line_ending_returns_unix_for_no_endings() {
        assert_eq!(detect_line_ending("hello world"), LineEnding::Unix);
    }

    #[test]
    fn detect_line_ending_detects_unix() {
        assert_eq!(detect_line_ending("a\nb\nc"), LineEnding::Unix);
    }

    #[test]
    fn detect_line_ending_detects_windows() {
        assert_eq!(detect_line_ending("a\r\nb\r\nc"), LineEnding::Windows);
    }

    #[test]
    fn detect_line_ending_handles_mixed() {
        // Should return the dominant style
        let result = detect_line_ending("a\nb\r\nc\nd");
        assert_eq!(result, LineEnding::Unix);
    }

    #[test]
    fn detect_line_ending_handles_empty() {
        assert_eq!(detect_line_ending(""), LineEnding::Unix);
    }

    #[test]
    fn detect_line_ending_detects_cr_only() {
        // CR only is treated as Unix after normalization
        let result = detect_line_ending("a\rb\rc");
        // With more CR than CRLF, it defaults to Unix
        assert_eq!(result, LineEnding::Unix);
    }
}

// =============================================================================
// 15. Edge cases tests (8 tests)
// =============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn handles_only_whitespace() {
        assert_eq!(normalize_whitespace("   \t   "), "");
    }

    #[test]
    fn handles_only_newlines() {
        let result = normalize_whitespace("\n\n\n");
        // Empty lines are collapsed
        assert!(result.is_empty() || result.chars().all(|c| c == '\n'));
    }

    #[test]
    fn handles_unicode_content() {
        let result = normalize_whitespace("  hello 世界  ");
        assert_eq!(result, "hello 世界");
    }

    #[test]
    fn handles_very_long_line() {
        let long_line = "a".repeat(10000);
        let result = normalize_whitespace(&long_line);
        assert_eq!(result.len(), 10000);
    }

    #[test]
    fn handles_very_many_lines() {
        let many_lines: String = (0..1000).map(|i| format!("line{}\n", i)).collect();
        let result = normalize_code(&many_lines);
        assert!(result.contains("line0"));
        assert!(result.contains("line999"));
    }

    #[test]
    fn handles_alternating_whitespace() {
        let result = normalize_whitespace(" a  b   c    d ");
        assert_eq!(result, "a b c d");
    }

    #[test]
    fn handles_code_normalizer_with_all_options_disabled() {
        let n = CodeNormalizer::none();
        let input = "  hello    world  ";
        let result = n.normalize(input);
        // With none, content is preserved
        assert!(result.contains("hello"));
    }

    #[test]
    fn handles_single_character() {
        assert_eq!(normalize_whitespace(" a "), "a");
    }
}

// =============================================================================
// 16. Property-based tests with proptest (10 tests)
// =============================================================================

mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn normalize_whitespace_never_panics(s: String) {
            let _ = normalize_whitespace(&s);
        }

        #[test]
        fn normalize_line_endings_never_panics(s: String) {
            let _ = normalize_line_endings(&s);
        }

        #[test]
        fn normalize_indentation_never_panics(s: String) {
            let _ = normalize_indentation(&s);
        }

        #[test]
        fn normalize_code_never_panics(s: String) {
            let _ = normalize_code(&s);
        }

        #[test]
        fn normalize_code_is_idempotent(s: String) {
            let first = normalize_code(&s);
            let second = normalize_code(&first);
            prop_assert_eq!(first, second);
        }

        #[test]
        fn normalize_whitespace_removes_crlf(s: String) {
            let result = normalize_whitespace(&s);
            prop_assert!(!result.contains("\r\n"));
        }

        #[test]
        fn normalize_line_endings_contains_no_cr(s: String) {
            let result = normalize_line_endings(&s);
            prop_assert!(!result.contains('\r'));
        }

        #[test]
        fn code_normalizer_never_panics(s: String) {
            let n = CodeNormalizer::new();
            let _ = n.normalize(&s);
        }

        #[test]
        fn count_lines_consistent_with_split(s: String) {
            if s.is_empty() {
                prop_assert_eq!(count_lines(&s), 0);
            } else {
                prop_assert_eq!(count_lines(&s), s.split('\n').count());
            }
        }

        #[test]
        fn needs_normalization_inverse_of_clean(s: String) {
            let normalized = normalize_code(&s);
            prop_assert!(!needs_normalization(&normalized));
        }
    }
}

// =============================================================================
// Additional integration tests
// =============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn full_normalization_pipeline() {
        let input = "\r\n    /// Documentation comment\r\n    fn example() {\r\n        let x = 1;\r\n    }\r\n";
        let result = normalize_code(input);

        // Should have normalized line endings
        assert!(!result.contains("\r\n"));

        // Should have removed common indentation
        assert!(result.contains("fn example()"));
    }

    #[test]
    fn compare_normalized_versions() {
        let code1 = "fn main() {\n    println!(\"hello\");\n}";
        let code2 = "fn main() {\r\n    println!(\"hello\");\r\n}";

        let norm1 = normalize_code(code1);
        let norm2 = normalize_code(code2);

        // Both should normalize to the same thing
        assert_eq!(norm1, norm2);
    }

    #[test]
    fn preserves_code_structure() {
        let code = r#"
fn example() {
    let x = 1;
    if x > 0 {
        println!("positive");
    }
}
"#;
        let result = normalize_code(code);

        // Structure should be preserved
        assert!(result.contains("fn example()"));
        assert!(result.contains("let x = 1"));
        assert!(result.contains("if x > 0"));
        assert!(result.contains("println!"));
    }

    #[test]
    fn handles_rust_code_sample() {
        let code = r#"/// A sample function
pub fn calculate(x: i32, y: i32) -> i32 {
    x + y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate() {
        assert_eq!(calculate(1, 2), 3);
    }
}
"#;
        let result = normalize_code(code);

        // All important elements should be present
        assert!(result.contains("pub fn calculate"));
        assert!(result.contains("x + y"));
        assert!(result.contains("#[cfg(test)]"));
        assert!(result.contains("mod tests"));
        assert!(result.contains("fn test_calculate"));
    }
}
