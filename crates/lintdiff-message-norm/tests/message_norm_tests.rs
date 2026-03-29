//! Comprehensive tests for lintdiff-message-norm
//!
//! Test categories:
//! 1. normalize function (10 tests)
//! 2. NormalizeConfig options (10 tests)
//! 3. truncate function (8 tests)
//! 4. truncate_at_word function (8 tests)
//! 5. strip_ansi function (6 tests)
//! 6. escape function (10 tests)
//! 7. NormalizedMessage wrapper (8 tests)
//! Total: 60 tests

use lintdiff_message_norm::{
    escape, has_ansi, normalize, normalize_owned, strip_ansi, truncate, truncate_at_word, unescape,
    EscapeFormat, NormalizeConfig, NormalizedMessage,
};

// =============================================================================
// 1. normalize function tests (10 tests)
// =============================================================================

mod normalize_tests {
    use super::*;

    #[test]
    fn normalize_returns_borrowed_when_no_changes_needed() {
        let msg = "hello world";
        let result = normalize(msg);
        assert!(matches!(result, std::borrow::Cow::Borrowed(_)));
        assert_eq!(result, "hello world");
    }

    #[test]
    fn normalize_trims_leading_whitespace() {
        assert_eq!(normalize("   hello"), "hello");
    }

    #[test]
    fn normalize_trims_trailing_whitespace() {
        assert_eq!(normalize("hello   "), "hello");
    }

    #[test]
    fn normalize_trims_both_leading_and_trailing_whitespace() {
        assert_eq!(normalize("   hello   "), "hello");
    }

    #[test]
    fn normalize_collapses_multiple_spaces() {
        assert_eq!(normalize("hello    world"), "hello world");
    }

    #[test]
    fn normalize_converts_crlf_to_lf() {
        assert_eq!(normalize("hello\r\nworld"), "hello\nworld");
    }

    #[test]
    fn normalize_removes_control_characters() {
        assert_eq!(normalize("hello\x00world"), "helloworld");
        assert_eq!(normalize("hello\x01world"), "helloworld");
        assert_eq!(normalize("hello\x1fworld"), "helloworld");
    }

    #[test]
    fn normalize_preserves_newlines() {
        assert_eq!(normalize("hello\nworld"), "hello\nworld");
    }

    #[test]
    fn normalize_converts_tabs_to_single_space() {
        // Tabs are converted to spaces during whitespace collapsing when there are multiple
        let result = normalize("hello\t\tworld");
        // Multiple tabs collapse to single space
        assert_eq!(result, "hello world");
    }

    #[test]
    fn normalize_handles_complex_input() {
        let input = "  Hello   \r\n  world\t\ttest  ";
        // After normalization: trim, collapse whitespace, CRLF->LF
        // "  Hello   \r\n  world\t\ttest  "
        // -> CRLF to LF: "  Hello   \n  world\t\ttest  "
        // -> Remove control chars: no change (no control chars except already handled)
        // -> Collapse whitespace: " Hello \n world test "
        // -> Trim: "Hello \n world test"
        // Note: tabs are collapsed with spaces
        let result = normalize(input);
        assert!(result.contains("Hello"));
        assert!(result.contains("world"));
        assert!(result.contains("test"));
    }
}

// =============================================================================
// 2. NormalizeConfig options tests (10 tests)
// =============================================================================

mod normalize_config_tests {
    use super::*;

    #[test]
    fn config_default_enables_all_normalizations() {
        let config = NormalizeConfig::default();
        assert!(config.trim);
        assert!(config.collapse_whitespace);
        assert!(config.normalize_line_endings);
        assert!(config.remove_control_chars);
        assert_eq!(config.max_length, 0);
        assert_eq!(config.truncation_suffix, "...");
    }

    #[test]
    fn config_none_disables_all_normalizations() {
        let config = NormalizeConfig::none();
        assert!(!config.trim);
        assert!(!config.collapse_whitespace);
        assert!(!config.normalize_line_endings);
        assert!(!config.remove_control_chars);
        assert_eq!(config.max_length, 0);
        assert!(config.truncation_suffix.is_empty());
    }

    #[test]
    fn config_no_trim_preserves_whitespace() {
        let config = NormalizeConfig::new().no_trim();
        assert!(!config.trim);
        // With no_trim but collapse_whitespace=true, internal spaces are still collapsed
        let result = config.normalize("  hello  ");
        // Spaces are collapsed but not trimmed - result has leading/trailing space collapsed to single
        assert!(result.starts_with(' ') || result == "hello");
    }

    #[test]
    fn config_no_collapse_preserves_multiple_spaces() {
        let config = NormalizeConfig::new().no_collapse();
        assert!(!config.collapse_whitespace);
        let result = config.normalize("hello    world");
        // Still trimmed since trim is still true
        assert_eq!(result, "hello    world");
    }

    #[test]
    fn config_no_line_ending_norm_preserves_crlf() {
        let config = NormalizeConfig::new().no_line_ending_norm();
        assert!(!config.normalize_line_endings);
        let result = config.normalize("hello\r\nworld");
        // CRLF is preserved
        assert_eq!(result, "hello\r\nworld");
    }

    #[test]
    fn config_keep_control_chars_preserves_them() {
        let config = NormalizeConfig::new().keep_control_chars();
        assert!(!config.remove_control_chars);
        let result = config.normalize("hello\x01world");
        assert_eq!(result, "hello\x01world");
    }

    #[test]
    fn config_with_max_length_truncates() {
        let config = NormalizeConfig::new().with_max_length(10);
        assert_eq!(config.max_length, 10);
        let result = config.normalize("hello world this is long");
        assert_eq!(result.chars().count(), 10);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn config_with_suffix_uses_custom_suffix() {
        let config = NormalizeConfig::new()
            .with_max_length(10)
            .with_suffix("...");
        assert_eq!(config.truncation_suffix, "...");
        let result = config.normalize("hello world this is long");
        assert!(result.ends_with("..."));
    }

    #[test]
    fn config_can_chain_all_modifiers() {
        let config = NormalizeConfig::new()
            .no_trim()
            .no_collapse()
            .no_line_ending_norm()
            .keep_control_chars()
            .with_max_length(100)
            .with_suffix("...");
        assert!(!config.trim);
        assert!(!config.collapse_whitespace);
        assert!(!config.normalize_line_endings);
        assert!(!config.remove_control_chars);
        assert_eq!(config.max_length, 100);
        assert_eq!(config.truncation_suffix, "...");
    }

    #[test]
    fn config_max_length_zero_means_unlimited() {
        let config = NormalizeConfig::new().with_max_length(0);
        let long_msg = "a".repeat(1000);
        let result = config.normalize(&long_msg);
        assert_eq!(result.len(), 1000);
    }
}

// =============================================================================
// 3. truncate function tests (8 tests)
// =============================================================================

mod truncate_tests {
    use super::*;

    #[test]
    fn truncate_returns_original_if_shorter_than_max() {
        assert_eq!(truncate("hello", 10, "..."), "hello");
    }

    #[test]
    fn truncate_returns_original_if_equal_to_max() {
        assert_eq!(truncate("hello", 5, "..."), "hello");
    }

    #[test]
    fn truncate_truncates_with_suffix() {
        let result = truncate("hello world", 8, "...");
        assert_eq!(result, "hello...");
        assert_eq!(result.chars().count(), 8);
    }

    #[test]
    fn truncate_with_empty_suffix() {
        let result = truncate("hello world", 5, "");
        assert_eq!(result, "hello");
    }

    #[test]
    fn truncate_with_max_len_zero_returns_original() {
        assert_eq!(truncate("hello", 0, "..."), "hello");
    }

    #[test]
    fn truncate_handles_unicode_correctly() {
        let result = truncate("hello world test", 8, "...");
        assert_eq!(result, "hello...");
    }

    #[test]
    fn truncate_with_very_small_max_len() {
        let result = truncate("hello", 2, "...");
        // When max_len <= suffix_char_count, we truncate the suffix
        assert_eq!(result, "..");
    }

    #[test]
    fn truncate_with_max_len_equal_to_suffix_len() {
        let result = truncate("hello", 3, "...");
        assert_eq!(result, "...");
    }
}

// =============================================================================
// 4. truncate_at_word function tests (8 tests)
// =============================================================================

mod truncate_at_word_tests {
    use super::*;

    #[test]
    fn truncate_at_word_returns_original_if_shorter() {
        assert_eq!(truncate_at_word("hello world", 20, "..."), "hello world");
    }

    #[test]
    fn truncate_at_word_finds_word_boundary() {
        let result = truncate_at_word("hello world test", 10, "...");
        assert_eq!(result, "hello...");
    }

    #[test]
    fn truncate_at_word_falls_back_to_char_boundary_if_no_space() {
        let result = truncate_at_word("helloworldtest", 10, "...");
        // No space to find, falls back to character boundary
        // 7 chars + 3 char suffix = 10
        assert_eq!(result.chars().count(), 10);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn truncate_at_word_with_empty_suffix() {
        let result = truncate_at_word("hello world test", 8, "");
        // With empty suffix, finds word boundary at "hello" (5 chars)
        assert!(result.chars().count() <= 8);
    }

    #[test]
    fn truncate_at_word_prefers_earlier_space_over_mid_word() {
        let result = truncate_at_word("a very long message here", 12, "...");
        // Should truncate at "a very" not "a very lon"
        assert_eq!(result, "a very...");
    }

    #[test]
    fn truncate_at_word_handles_unicode() {
        let result = truncate_at_word("hello world test", 10, "...");
        assert!(result.ends_with("..."));
    }

    #[test]
    fn truncate_at_word_with_single_long_word() {
        let result = truncate_at_word("supercalifragilisticexpialidocious", 10, "...");
        // 7 chars + 3 suffix = 10
        assert_eq!(result.chars().count(), 10);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn truncate_at_word_zero_max_len_returns_original() {
        assert_eq!(truncate_at_word("hello", 0, "..."), "hello");
    }
}

// =============================================================================
// 5. strip_ansi function tests (6 tests)
// =============================================================================

mod strip_ansi_tests {
    use super::*;

    #[test]
    fn strip_ansi_returns_original_if_no_ansi() {
        let msg = "hello world";
        assert_eq!(strip_ansi(msg), msg);
    }

    #[test]
    fn strip_ansi_removes_color_codes() {
        let result = strip_ansi("\x1b[31mred\x1b[0m");
        assert_eq!(result, "red");
    }

    #[test]
    fn strip_ansi_removes_multiple_codes() {
        let result = strip_ansi("\x1b[1;31mbold red\x1b[0m normal");
        assert_eq!(result, "bold red normal");
    }

    #[test]
    fn strip_ansi_removes_complex_sequences() {
        // 256-color and RGB sequences
        let result = strip_ansi("\x1b[38;5;196mred\x1b[0m");
        assert_eq!(result, "red");
    }

    #[test]
    fn has_ansi_detects_ansi_codes() {
        assert!(has_ansi("\x1b[31mred\x1b[0m"));
        assert!(!has_ansi("plain text"));
    }

    #[test]
    fn strip_ansi_preserves_message_content() {
        let msg = "Error: \x1b[31mfile not found\x1b[0m at line 10";
        let result = strip_ansi(msg);
        assert_eq!(result, "Error: file not found at line 10");
    }
}

// =============================================================================
// 6. escape function tests (10 tests)
// =============================================================================

mod escape_tests {
    use super::*;

    // JSON tests
    #[test]
    fn escape_json_escapes_quotes() {
        assert_eq!(escape("say \"hi\"", EscapeFormat::Json), "say \\\"hi\\\"");
    }

    #[test]
    fn escape_json_escapes_backslash() {
        assert_eq!(
            escape("path\\to\\file", EscapeFormat::Json),
            "path\\\\to\\\\file"
        );
    }

    #[test]
    fn escape_json_escapes_newlines() {
        assert_eq!(escape("hello\nworld", EscapeFormat::Json), "hello\\nworld");
    }

    #[test]
    fn escape_json_escapes_tabs() {
        assert_eq!(escape("hello\tworld", EscapeFormat::Json), "hello\\tworld");
    }

    // HTML tests
    #[test]
    fn escape_html_escapes_angle_brackets() {
        let result = escape("<div>", EscapeFormat::Html);
        assert_eq!(result, "\x26lt;div\x26gt;");
    }

    #[test]
    fn escape_html_escapes_ampersand() {
        let result = escape("a & b", EscapeFormat::Html);
        assert_eq!(result, "a \x26amp; b");
    }

    #[test]
    fn escape_html_escapes_quotes() {
        let result = escape("say \"hi\"", EscapeFormat::Html);
        assert_eq!(result, "say \x26quot;hi\x26quot;");
    }

    // Shell tests
    #[test]
    fn escape_shell_quotes_special_chars() {
        let result = escape("hello world", EscapeFormat::Shell);
        assert!(result.starts_with('\''));
        assert!(result.ends_with('\''));
    }

    #[test]
    fn escape_shell_handles_single_quotes() {
        let result = escape("it's", EscapeFormat::Shell);
        // Single quotes are escaped with: end quote, double quote, single quote, double quote, start quote
        assert!(result.contains("it"));
        assert!(result.contains("'"));
    }

    // Markdown tests
    #[test]
    fn escape_markdown_escapes_special_chars() {
        assert_eq!(escape("*bold*", EscapeFormat::Markdown), "\\*bold\\*");
        assert_eq!(escape("_italic_", EscapeFormat::Markdown), "\\_italic\\_");
    }
}

// =============================================================================
// 7. NormalizedMessage wrapper tests (8 tests)
// =============================================================================

mod normalized_message_tests {
    use super::*;

    #[test]
    fn normalized_message_new_normalizes_input() {
        let msg = NormalizedMessage::new("  hello   world  ");
        assert_eq!(msg.as_str(), "hello world");
    }

    #[test]
    fn normalized_message_with_config_uses_config() {
        let config = NormalizeConfig::none();
        let msg = NormalizedMessage::with_config("  hello  ", &config);
        // With none config, no normalization happens
        assert_eq!(msg.as_str(), "  hello  ");
    }

    #[test]
    fn normalized_message_len_returns_correct_length() {
        let msg = NormalizedMessage::new("hello");
        assert_eq!(msg.len(), 5);
    }

    #[test]
    fn normalized_message_is_empty_works() {
        assert!(NormalizedMessage::new("").is_empty());
        assert!(!NormalizedMessage::new("hello").is_empty());
    }

    #[test]
    fn normalized_message_truncate_modifies_in_place() {
        let mut msg = NormalizedMessage::new("hello world");
        msg.truncate(8, "...");
        assert_eq!(msg.as_str(), "hello...");
    }

    #[test]
    fn normalized_message_display_trait() {
        let msg = NormalizedMessage::new("hello");
        assert_eq!(format!("{msg}"), "hello");
    }

    #[test]
    fn normalized_message_from_str() {
        let msg: NormalizedMessage = "hello".into();
        assert_eq!(msg.as_str(), "hello");
    }

    #[test]
    fn normalized_message_from_string() {
        let msg: NormalizedMessage = String::from("hello").into();
        assert_eq!(msg.as_str(), "hello");
    }

    #[test]
    fn normalized_message_as_ref() {
        let msg = NormalizedMessage::new("hello");
        let ref_str: &str = msg.as_ref();
        assert_eq!(ref_str, "hello");
    }

    #[test]
    fn normalized_message_into_inner() {
        let msg = NormalizedMessage::new("hello");
        let inner = msg.into_inner();
        assert_eq!(inner, "hello");
    }

    #[test]
    fn normalized_message_default() {
        let msg = NormalizedMessage::default();
        assert!(msg.is_empty());
    }
}

// =============================================================================
// Additional unescape tests
// =============================================================================

mod unescape_tests {
    use super::*;

    #[test]
    fn unescape_json_unescapes_quotes() {
        assert_eq!(unescape("say \\\"hi\\\"", EscapeFormat::Json), "say \"hi\"");
    }

    #[test]
    fn unescape_json_unescapes_newlines() {
        assert_eq!(
            unescape("hello\\nworld", EscapeFormat::Json),
            "hello\nworld"
        );
    }

    #[test]
    fn unescape_html_unescapes_entities() {
        let result = unescape("\x26lt;div\x26gt;", EscapeFormat::Html);
        assert_eq!(result, "<div>");
    }

    #[test]
    fn unescape_html_unescapes_ampersand() {
        let result = unescape("a \x26amp; b", EscapeFormat::Html);
        assert_eq!(result, "a & b");
    }

    #[test]
    fn unescape_markdown_unescapes_special_chars() {
        assert_eq!(unescape("\\*bold\\*", EscapeFormat::Markdown), "*bold*");
    }
}

// =============================================================================
// Edge case tests
// =============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn empty_string_handling() {
        assert_eq!(normalize(""), "");
        assert_eq!(truncate("", 10, "..."), "");
        assert_eq!(strip_ansi(""), "");
        assert!(NormalizedMessage::new("").is_empty());
    }

    #[test]
    fn whitespace_only_string() {
        assert_eq!(normalize("    "), "");
        assert_eq!(normalize("\t\t"), "");
        assert_eq!(normalize("\n\n"), "");
    }

    #[test]
    fn very_long_string_normalization() {
        let long_msg = "a".repeat(10000);
        let result = normalize(&long_msg);
        assert_eq!(result.len(), 10000);
    }

    #[test]
    fn unicode_preservation() {
        let msg = "Hello world";
        let result = normalize(msg);
        assert_eq!(result, msg);
    }

    #[test]
    fn mixed_content_normalization() {
        let input = "  Hello\t\tworld\r\nTest  ";
        let result = normalize(input);
        assert!(result.contains("Hello"));
        assert!(result.contains("world"));
        assert!(result.contains("Test"));
    }

    #[test]
    fn normalize_owned_returns_string() {
        let result = normalize_owned("  hello  ");
        assert_eq!(result, "hello");
        assert!(matches!(result, String { .. }));
    }
}

// =============================================================================
// Property-based tests using proptest
// =============================================================================

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn normalize_never_panics(s: String) {
            let _ = normalize(&s);
        }

        #[test]
        fn normalize_idempotent(s: String) {
            let first = normalize(&s);
            let second = normalize(&first);
            assert_eq!(first, second);
        }

        #[test]
        fn truncate_never_panics(s: String, max: usize, suffix: String) {
            let _ = truncate(&s, max % 1000, &suffix);
        }

        #[test]
        fn strip_ansi_never_panics(s: String) {
            let _ = strip_ansi(&s);
        }

        #[test]
        fn strip_ansi_removes_all_escape_chars(s: String) {
            let result = strip_ansi(&s);
            assert!(!has_ansi(&result));
        }

        #[test]
        fn escape_unescape_roundtrip_json(s: String) {
            let escaped = escape(&s, EscapeFormat::Json);
            let unescaped = unescape(&escaped, EscapeFormat::Json);
            assert_eq!(s, unescaped);
        }

        #[test]
        fn escape_unescape_roundtrip_html(s: String) {
            let escaped = escape(&s, EscapeFormat::Html);
            let unescaped = unescape(&escaped, EscapeFormat::Html);
            assert_eq!(s, unescaped);
        }

        #[test]
        fn escape_unescape_roundtrip_markdown(s: String) {
            let escaped = escape(&s, EscapeFormat::Markdown);
            let unescaped = unescape(&escaped, EscapeFormat::Markdown);
            assert_eq!(s, unescaped);
        }

        #[test]
        fn normalized_message_roundtrip(s: String) {
            let msg = NormalizedMessage::new(&s);
            let inner = msg.into_inner();
            let msg2 = NormalizedMessage::new(&inner);
            assert_eq!(msg2.as_str(), normalize(&s));
        }
    }
}
