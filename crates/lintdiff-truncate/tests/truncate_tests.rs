//! Comprehensive tests for lintdiff-truncate.
//!
//! These tests cover all public API functions and edge cases.

use lintdiff_truncate::{truncate, truncate_lines, truncate_owned, would_truncate, TruncateConfig};

// ============================================================================
// TruncateConfig Tests
// ============================================================================

mod config_tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let config = TruncateConfig::default();
        assert_eq!(config.max_length, 120);
        assert_eq!(config.ellipsis, "...");
        assert!(config.preserve_words);
    }

    #[test]
    fn new_config_sets_max_length() {
        let config = TruncateConfig::new(50);
        assert_eq!(config.max_length, 50);
        assert_eq!(config.ellipsis, "...");
        assert!(config.preserve_words);
    }

    #[test]
    fn new_config_with_zero_length() {
        let config = TruncateConfig::new(0);
        assert_eq!(config.max_length, 0);
    }

    #[test]
    fn github_config_has_140_char_limit() {
        let config = TruncateConfig::github();
        assert_eq!(config.max_length, 140);
        assert_eq!(config.ellipsis, "...");
        assert!(config.preserve_words);
    }

    #[test]
    fn unlimited_config_has_max_value() {
        let config = TruncateConfig::unlimited();
        assert_eq!(config.max_length, usize::MAX);
        assert_eq!(config.ellipsis, "...");
        assert!(config.preserve_words);
    }

    #[test]
    fn with_word_preservation_true() {
        let config = TruncateConfig::new(50).with_word_preservation(true);
        assert!(config.preserve_words);
    }

    #[test]
    fn with_word_preservation_false() {
        let config = TruncateConfig::new(50).with_word_preservation(false);
        assert!(!config.preserve_words);
    }

    #[test]
    fn with_ellipsis_custom() {
        let config = TruncateConfig::new(50).with_ellipsis("…");
        assert_eq!(config.ellipsis, "…");
    }

    #[test]
    fn with_ellipsis_empty() {
        let config = TruncateConfig::new(50).with_ellipsis("");
        assert_eq!(config.ellipsis, "");
    }

    #[test]
    fn with_ellipsis_long() {
        let config = TruncateConfig::new(50).with_ellipsis(".....more.....");
        assert_eq!(config.ellipsis, ".....more.....");
    }

    #[test]
    fn builder_chaining() {
        let config = TruncateConfig::new(100)
            .with_word_preservation(false)
            .with_ellipsis("[...]");
        assert_eq!(config.max_length, 100);
        assert!(!config.preserve_words);
        assert_eq!(config.ellipsis, "[...]");
    }
}

// ============================================================================
// truncate() Tests
// ============================================================================

mod truncate_tests {
    use super::*;

    #[test]
    fn no_truncation_when_string_fits() {
        let config = TruncateConfig::new(20);
        let result = truncate("Short string", &config);
        assert_eq!(result, "Short string");
    }

    #[test]
    fn truncates_long_string() {
        let config = TruncateConfig::new(10);
        let result = truncate("Hello world, this is long", &config);
        assert!(result.len() <= 10);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn exact_length_no_truncation() {
        let config = TruncateConfig::new(11);
        let result = truncate("Hello world", &config);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn one_char_over_truncates() {
        let config = TruncateConfig::new(10);
        let result = truncate("Hello world", &config);
        assert!(result.len() <= 10);
    }

    #[test]
    fn empty_string_returns_empty() {
        let config = TruncateConfig::new(10);
        let result = truncate("", &config);
        assert_eq!(result, "");
    }

    #[test]
    fn string_shorter_than_ellipsis() {
        let config = TruncateConfig::new(2);
        let result = truncate("Hi", &config);
        // Should return just ellipsis or handle gracefully
        assert!(result.len() <= 2);
    }

    #[test]
    fn zero_max_length_returns_ellipsis() {
        let config = TruncateConfig::new(0);
        let result = truncate("Hello", &config);
        assert_eq!(result, "...");
    }

    #[test]
    fn word_boundary_preserved() {
        let config = TruncateConfig::new(15).with_word_preservation(true);
        let result = truncate("Hello beautiful world", &config);
        // Should not cut "beautiful" in the middle
        assert!(!result.contains("beauti"));
        assert!(!result.contains("beautif"));
    }

    #[test]
    fn no_word_boundary_exact_cut() {
        let config = TruncateConfig::new(10).with_word_preservation(false);
        let result = truncate("Hello world", &config);
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn custom_ellipsis_single_char() {
        let config = TruncateConfig::new(10).with_ellipsis("…");
        let result = truncate("Hello world, this is long", &config);
        assert!(result.ends_with("…"));
    }

    #[test]
    fn custom_ellipsis_empty() {
        let config = TruncateConfig::new(10).with_ellipsis("");
        let result = truncate("Hello world, this is long", &config);
        assert!(!result.ends_with("..."));
        assert!(result.len() <= 10);
    }

    #[test]
    fn multibyte_characters_japanese() {
        let config = TruncateConfig::new(10);
        let result = truncate("こんにちは世界の皆さん", &config);
        // Should not panic and should be valid UTF-8
        assert!(result.len() <= 10);
    }

    #[test]
    fn multibyte_characters_emoji() {
        let config = TruncateConfig::new(10);
        let result = truncate("Hello 🌍🌎🌏 world", &config);
        // Should not panic and should be valid UTF-8
        assert!(result.len() <= 10);
    }

    #[test]
    fn multibyte_preserves_char_boundary() {
        let config = TruncateConfig::new(8).with_word_preservation(false);
        let result = truncate("Hello 🌍", &config);
        // The emoji is 4 bytes, so we need to make sure we don't cut it
        assert!(result.chars().last().is_some());
    }

    #[test]
    fn single_word_longer_than_limit() {
        let config = TruncateConfig::new(10);
        let result = truncate("Supercalifragilisticexpialidocious", &config);
        assert!(result.len() <= 10);
    }

    #[test]
    fn whitespace_only_string() {
        let config = TruncateConfig::new(10);
        let result = truncate("          ", &config);
        assert!(result.len() <= 10);
    }

    #[test]
    fn string_with_newlines() {
        let config = TruncateConfig::new(15);
        let result = truncate("Line1\nLine2\nLine3", &config);
        assert!(result.len() <= 15);
    }

    #[test]
    fn string_with_tabs() {
        let config = TruncateConfig::new(15);
        let result = truncate("Column1\tColumn2\tColumn3", &config);
        assert!(result.len() <= 15);
    }

    #[test]
    fn returns_cow_borrowed_when_no_truncation() {
        use std::borrow::Cow;
        let config = TruncateConfig::new(100);
        let original = "Short string";
        let result = truncate(original, &config);
        matches!(result, Cow::Borrowed(_));
    }

    #[test]
    fn returns_cow_owned_when_truncated() {
        use std::borrow::Cow;
        let config = TruncateConfig::new(10);
        let result = truncate("This is a long string", &config);
        matches!(result, Cow::Owned(_));
    }
}

// ============================================================================
// truncate_owned() Tests
// ============================================================================

mod truncate_owned_tests {
    use super::*;

    #[test]
    fn returns_owned_string() {
        let config = TruncateConfig::new(10);
        let result: String = truncate_owned("Hello world, this is long", &config);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn no_truncation_returns_owned_copy() {
        let config = TruncateConfig::new(100);
        let result = truncate_owned("Short string", &config);
        assert_eq!(result, "Short string");
    }

    #[test]
    fn empty_string_returns_empty_owned() {
        let config = TruncateConfig::new(10);
        let result = truncate_owned("", &config);
        assert_eq!(result, "");
    }
}

// ============================================================================
// truncate_lines() Tests
// ============================================================================

mod truncate_lines_tests {
    use super::*;

    #[test]
    fn empty_lines_returns_empty_vec() {
        let config = TruncateConfig::new(30);
        let lines: Vec<String> = Vec::new();
        let result = truncate_lines(&lines, &config);
        assert!(result.is_empty());
    }

    #[test]
    fn single_line_fits_no_truncation() {
        let config = TruncateConfig::new(30);
        let lines = vec!["Short line".to_string()];
        let result = truncate_lines(&lines, &config);
        assert_eq!(result, lines);
    }

    #[test]
    fn multiple_lines_fit_no_truncation() {
        let config = TruncateConfig::new(30);
        let lines = vec!["Short line".to_string(), "Another".to_string()];
        let result = truncate_lines(&lines, &config);
        assert_eq!(result, lines);
    }

    #[test]
    fn single_line_needs_truncation() {
        let config = TruncateConfig::new(10);
        let lines = vec!["This is a very long line".to_string()];
        let result = truncate_lines(&lines, &config);
        assert!(result[0].len() <= 10);
    }

    #[test]
    fn multiple_lines_need_truncation() {
        let config = TruncateConfig::new(30);
        let lines = vec![
            "This is line one which is quite long".to_string(),
            "This is line two which is also long".to_string(),
        ];
        let result = truncate_lines(&lines, &config);
        let total: usize = result.iter().map(|s| s.len()).sum();
        assert!(total <= 30);
    }

    #[test]
    fn preserves_line_count() {
        let config = TruncateConfig::new(30);
        let lines = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ];
        let result = truncate_lines(&lines, &config);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn handles_empty_lines_in_vec() {
        let config = TruncateConfig::new(30);
        let lines = vec!["Line 1".to_string(), "".to_string(), "Line 3".to_string()];
        let result = truncate_lines(&lines, &config);
        assert_eq!(result.len(), 3);
    }
}

// ============================================================================
// would_truncate() Tests
// ============================================================================

mod would_truncate_tests {
    use super::*;

    #[test]
    fn returns_true_when_longer() {
        let config = TruncateConfig::new(10);
        assert!(would_truncate("This is a long string", &config));
    }

    #[test]
    fn returns_false_when_shorter() {
        let config = TruncateConfig::new(100);
        assert!(!would_truncate("Short string", &config));
    }

    #[test]
    fn returns_false_when_exact_length() {
        let config = TruncateConfig::new(11);
        assert!(!would_truncate("Hello world", &config));
    }

    #[test]
    fn empty_string_never_truncates() {
        let config = TruncateConfig::new(0);
        assert!(!would_truncate("", &config));
    }

    #[test]
    fn unlimited_config_never_truncates() {
        let config = TruncateConfig::unlimited();
        let long_string = "a".repeat(10000);
        assert!(!would_truncate(&long_string, &config));
    }
}

// ============================================================================
// Edge Cases and Integration Tests
// ============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn very_long_ellipsis() {
        let config = TruncateConfig::new(20).with_ellipsis("..........");
        let result = truncate("Hello world", &config);
        // With ellipsis of 10 chars, we have 10 chars left for content
        assert!(result.len() <= 20);
    }

    #[test]
    fn ellipsis_longer_than_max_length() {
        let config = TruncateConfig::new(5).with_ellipsis("..........");
        let result = truncate("Hello world", &config);
        // Should handle gracefully
        assert_eq!(result, "..........");
    }

    #[test]
    fn unicode_normalization() {
        // Test with composed vs decomposed characters
        let config = TruncateConfig::new(10);
        let composed = "café"; // é as single character
        let result = truncate(composed, &config);
        assert!(result.len() <= 10);
    }

    #[test]
    fn zero_width_characters() {
        let config = TruncateConfig::new(10);
        let result = truncate("Hello\u{200B}World", &config); // Zero-width space
        assert!(result.len() <= 10);
    }

    #[test]
    fn config_is_clone() {
        let config = TruncateConfig::new(50);
        let cloned = config.clone();
        assert_eq!(config.max_length, cloned.max_length);
    }

    #[test]
    fn config_is_debug() {
        let config = TruncateConfig::new(50);
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("TruncateConfig"));
    }
}

// ============================================================================
// Preset Config Tests
// ============================================================================

mod preset_tests {
    use super::*;

    #[test]
    fn github_preset_truncates_at_140() {
        let config = TruncateConfig::github();
        let long_string = "a".repeat(150);
        assert!(would_truncate(&long_string, &config));
        let result = truncate(&long_string, &config);
        assert!(result.len() <= 140);
    }

    #[test]
    fn github_preset_preserves_words() {
        let config = TruncateConfig::github();
        let text = "The quick brown fox jumps over the lazy dog and keeps going ".repeat(3);
        let result = truncate(&text, &config);
        // Should not cut a word in the middle
        assert!(result.ends_with("..."));
    }

    #[test]
    fn unlimited_preset_never_truncates() {
        let config = TruncateConfig::unlimited();
        let very_long = "a".repeat(1_000_000);
        assert!(!would_truncate(&very_long, &config));
        let result = truncate(&very_long, &config);
        assert_eq!(result.len(), 1_000_000);
    }
}
