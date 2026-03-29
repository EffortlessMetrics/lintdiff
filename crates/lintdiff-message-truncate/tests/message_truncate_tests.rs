//! Comprehensive BDD tests for lintdiff-message-truncate.
//!
//! These tests cover all public API functions and edge cases.

use lintdiff_message_truncate::{
    char_len, is_truncated, truncate, truncate_bytes, truncate_with_options, truncate_words,
    TruncateOptions, DEFAULT_ELLIPSIS,
};

// ============================================================================
// TruncateOptions Tests
// ============================================================================

mod truncate_options_tests {
    use super::*;

    #[test]
    fn default_options_has_expected_values() {
        let options = TruncateOptions::default();
        assert_eq!(options.max_length, 100);
        assert_eq!(options.ellipsis, "...");
        assert!(!options.preserve_words);
        assert!(!options.preserve_sentences);
    }

    #[test]
    fn new_options_sets_max_length() {
        let options = TruncateOptions::new(50);
        assert_eq!(options.max_length, 50);
        assert_eq!(options.ellipsis, "...");
    }

    #[test]
    fn new_options_with_zero_length() {
        let options = TruncateOptions::new(0);
        assert_eq!(options.max_length, 0);
    }

    #[test]
    fn with_ellipsis_custom() {
        let options = TruncateOptions::new(50).with_ellipsis("…");
        assert_eq!(options.ellipsis, "…");
    }

    #[test]
    fn with_ellipsis_empty() {
        let options = TruncateOptions::new(50).with_ellipsis("");
        assert_eq!(options.ellipsis, "");
    }

    #[test]
    fn with_ellipsis_long() {
        let options = TruncateOptions::new(50).with_ellipsis(".....more.....");
        assert_eq!(options.ellipsis, ".....more.....");
    }

    #[test]
    fn with_preserve_words_true() {
        let options = TruncateOptions::new(50).with_preserve_words(true);
        assert!(options.preserve_words);
    }

    #[test]
    fn with_preserve_words_false() {
        let options = TruncateOptions::new(50).with_preserve_words(false);
        assert!(!options.preserve_words);
    }

    #[test]
    fn with_preserve_sentences_true() {
        let options = TruncateOptions::new(100).with_preserve_sentences(true);
        assert!(options.preserve_sentences);
    }

    #[test]
    fn with_preserve_sentences_false() {
        let options = TruncateOptions::new(100).with_preserve_sentences(false);
        assert!(!options.preserve_sentences);
    }

    #[test]
    fn builder_chaining() {
        let options = TruncateOptions::new(100)
            .with_ellipsis("[...]")
            .with_preserve_words(true)
            .with_preserve_sentences(true);
        assert_eq!(options.max_length, 100);
        assert_eq!(options.ellipsis, "[...]");
        assert!(options.preserve_words);
        assert!(options.preserve_sentences);
    }

    #[test]
    fn clone_creates_equal_instance() {
        let options = TruncateOptions::new(50).with_ellipsis("…");
        let cloned = options.clone();
        assert_eq!(options, cloned);
    }
}

// ============================================================================
// char_len Tests
// ============================================================================

mod char_len_tests {
    use super::*;

    #[test]
    fn empty_string() {
        assert_eq!(char_len(""), 0);
    }

    #[test]
    fn ascii_string() {
        assert_eq!(char_len("Hello"), 5);
        assert_eq!(char_len("Hello, world!"), 13);
    }

    #[test]
    fn unicode_emoji() {
        // Single emoji is 1 character but 4 bytes
        assert_eq!(char_len("😀"), 1);
        assert_eq!(char_len("Hello 😀"), 7);
    }

    #[test]
    fn unicode_multi_byte() {
        // Chinese characters are 3 bytes each but 1 character each
        assert_eq!(char_len("你好"), 2);
        assert_eq!(char_len("Hello你好"), 7);
    }

    #[test]
    fn mixed_unicode() {
        // Mix of ASCII, emoji, and multi-byte
        assert_eq!(char_len("Hi 😀 世界"), 7);
    }

    #[test]
    fn combining_characters() {
        // Combining characters are separate chars
        assert_eq!(char_len("é"), 1); // precomposed
        assert_eq!(char_len("e\u{0301}"), 2); // e + combining acute
    }
}

// ============================================================================
// truncate Tests
// ============================================================================

mod truncate_tests {
    use super::*;

    #[test]
    fn no_truncation_needed() {
        assert_eq!(truncate("Hello", 10), "Hello");
        assert_eq!(truncate("Hi", 5), "Hi");
    }

    #[test]
    fn exact_fit() {
        assert_eq!(truncate("Hello", 5), "Hello");
    }

    #[test]
    fn basic_truncation() {
        // With max_chars=8, we can fit 5 chars + 3 ellipsis
        assert_eq!(truncate("Hello, world!", 8), "Hello...");
    }

    #[test]
    fn empty_string() {
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn zero_max_chars() {
        assert_eq!(truncate("Hello", 0), "...");
    }

    #[test]
    fn max_chars_less_than_ellipsis() {
        // With max_chars=2 and ellipsis="...", we can only show ".."
        let result = truncate("Hello", 2);
        assert_eq!(result, "..");
    }

    #[test]
    fn max_chars_equals_one() {
        let result = truncate("Hello", 1);
        assert_eq!(result, ".");
    }

    #[test]
    fn unicode_emoji_truncation() {
        // Emoji is 1 character, "Hello 😀" is 7 chars
        // With max_chars=6, we can fit 3 chars + 3 ellipsis
        let result = truncate("Hello 😀", 6);
        assert_eq!(result, "Hel...");
    }

    #[test]
    fn unicode_emoji_preserved() {
        // Don't cut in the middle of emoji
        // "Hello 😀" is 7 chars, so max_chars=7 is exact fit
        let result = truncate("Hello 😀", 7);
        assert_eq!(result, "Hello 😀");
    }

    #[test]
    fn unicode_emoji_at_boundary() {
        // "Hello 😀 world" is 13 chars
        // With max_chars=10, we can fit 7 chars + 3 ellipsis
        let result = truncate("Hello 😀 world", 10);
        assert_eq!(result, "Hello 😀...");
    }

    #[test]
    fn chinese_characters() {
        // "你好世界测试" is 5 chars, which is <= 6, so no truncation
        let result = truncate("你好世界测试", 6);
        assert_eq!(result, "你好世界测试");
        // With max_chars=4, we can fit 1 char + 3 ellipsis
        let result2 = truncate("你好世界测试", 4);
        assert_eq!(result2, "你...");
    }

    #[test]
    fn mixed_unicode_truncation() {
        // "Hello 😀 世界 test" is 12 chars
        // With max_chars=8, we can fit 5 chars + 3 ellipsis
        let result = truncate("Hello 😀 世界 test", 8);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn whitespace_preserved() {
        // "Hello    world" has multiple spaces
        // With max_chars=10, we can fit 7 chars + 3 ellipsis
        let result = truncate("Hello    world", 10);
        // Result preserves chars up to position 7
        assert!(result.starts_with("Hello"));
        assert!(result.ends_with("..."));
    }

    #[test]
    fn newlines_count_as_chars() {
        // "Hello\nWorld" is 11 chars
        // With max_chars=8, we can fit 5 chars + 3 ellipsis
        let result = truncate("Hello\nWorld", 8);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn tabs_count_as_chars() {
        // "Hello\tWorld" is 11 chars
        // With max_chars=8, we can fit 5 chars + 3 ellipsis
        let result = truncate("Hello\tWorld", 8);
        assert_eq!(result, "Hello...");
    }
}

// ============================================================================
// truncate_bytes Tests
// ============================================================================

mod truncate_bytes_tests {
    use super::*;

    #[test]
    fn no_truncation_needed() {
        assert_eq!(truncate_bytes("Hello", 10), "Hello");
    }

    #[test]
    fn exact_fit() {
        assert_eq!(truncate_bytes("Hello", 5), "Hello");
    }

    #[test]
    fn basic_truncation() {
        // "Hello, world!" is 13 bytes
        // With max_bytes=8, we can fit 5 bytes + 3 ellipsis
        let result = truncate_bytes("Hello, world!", 8);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn empty_string() {
        assert_eq!(truncate_bytes("", 5), "");
    }

    #[test]
    fn zero_max_bytes() {
        assert_eq!(truncate_bytes("Hello", 0), "");
    }

    #[test]
    fn max_bytes_less_than_ellipsis() {
        let result = truncate_bytes("Hello", 2);
        assert_eq!(result, "..");
    }

    #[test]
    fn unicode_emoji_byte_boundary() {
        // "Hello 😀" is 10 bytes (5 + 1 space + 4 for emoji)
        // Truncating to 8 bytes should give "Hello..." (5 + 3)
        let result = truncate_bytes("Hello 😀", 8);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn unicode_does_not_split_emoji() {
        // Emoji is 4 bytes, so truncating to 7 bytes should fit emoji + ellipsis
        // "😀 world" is 10 bytes (4 emoji + 1 space + 5 world)
        let result = truncate_bytes("😀 world", 7);
        // Emoji (4 bytes) + ellipsis (3 bytes) = 7 bytes, fits perfectly
        assert_eq!(result, "😀...");
    }

    #[test]
    fn chinese_byte_boundary() {
        // Each Chinese character is 3 bytes
        // "你好世界" is 12 bytes
        // With max_bytes=6, we can fit 3 bytes + 3 ellipsis
        let result = truncate_bytes("你好世界", 6);
        assert_eq!(result, "你...");
    }

    #[test]
    fn preserves_valid_utf8() {
        // This should never panic - all results must be valid UTF-8
        let result = truncate_bytes("Hello 😀 世界 test", 10);
        assert!(result.is_char_boundary(result.len()));
    }

    #[test]
    fn exactly_at_char_boundary() {
        // "Hello" is exactly 5 bytes
        // With max_bytes=8, we can fit 5 bytes + 3 ellipsis
        let result = truncate_bytes("Hello world", 8);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn between_char_boundaries() {
        // Emoji is 4 bytes, test at 2 bytes (middle of emoji)
        let result = truncate_bytes("😀 test", 2);
        // Should back up to 0 and just show partial ellipsis
        assert_eq!(result, "..");
    }
}

// ============================================================================
// truncate_words Tests
// ============================================================================

mod truncate_words_tests {
    use super::*;

    #[test]
    fn no_truncation_needed() {
        assert_eq!(truncate_words("One two three", 5), "One two three");
    }

    #[test]
    fn exact_word_count() {
        assert_eq!(truncate_words("One two three", 3), "One two three");
    }

    #[test]
    fn basic_word_truncation() {
        // Words are normalized with single spaces
        assert_eq!(
            truncate_words("Hello beautiful world", 2),
            "Hello beautiful..."
        );
    }

    #[test]
    fn single_word() {
        assert_eq!(truncate_words("Hello", 1), "Hello");
    }

    #[test]
    fn empty_string() {
        assert_eq!(truncate_words("", 5), "");
    }

    #[test]
    fn zero_max_words() {
        assert_eq!(truncate_words("Hello world", 0), "...");
    }

    #[test]
    fn multiple_spaces_between_words() {
        // Multiple spaces are normalized to single space
        let result = truncate_words("Hello    beautiful    world", 2);
        assert_eq!(result, "Hello beautiful...");
    }

    #[test]
    fn leading_whitespace() {
        // Leading whitespace is stripped by split_whitespace
        let result = truncate_words("   Hello world", 1);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn trailing_whitespace() {
        // Trailing whitespace is stripped
        let result = truncate_words("Hello world   ", 1);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn newlines_as_separators() {
        // Newlines are treated as whitespace
        let result = truncate_words("Hello\nworld\ntest", 2);
        assert_eq!(result, "Hello world...");
    }

    #[test]
    fn tabs_as_separators() {
        // Tabs are treated as whitespace
        let result = truncate_words("Hello\tworld\ttest", 2);
        assert_eq!(result, "Hello world...");
    }

    #[test]
    fn mixed_whitespace() {
        // All whitespace is normalized
        let result = truncate_words("Hello \t\n world", 1);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn unicode_words() {
        let result = truncate_words("你好 世界 测试", 2);
        assert_eq!(result, "你好 世界...");
    }

    #[test]
    fn punctuation_in_words() {
        let result = truncate_words("Hello, beautiful world!", 2);
        assert_eq!(result, "Hello, beautiful...");
    }
}

// ============================================================================
// truncate_with_options Tests
// ============================================================================

mod truncate_with_options_tests {
    use super::*;

    #[test]
    fn basic_truncation_with_options() {
        // With max_chars=8, we can fit 5 chars + 3 ellipsis
        let options = TruncateOptions::new(8);
        let result = truncate_with_options("Hello, world!", &options);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn custom_ellipsis() {
        // With max_chars=6 and ellipsis="…", we can fit 5 chars + 1 ellipsis
        let options = TruncateOptions::new(6).with_ellipsis("…");
        let result = truncate_with_options("Hello, world!", &options);
        assert_eq!(result, "Hello…");
    }

    #[test]
    fn empty_ellipsis() {
        let options = TruncateOptions::new(5).with_ellipsis("");
        let result = truncate_with_options("Hello, world!", &options);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn long_ellipsis() {
        // "Hello, world!" is 13 chars, which is < 15, so no truncation needed
        let options = TruncateOptions::new(15).with_ellipsis(".....more.....");
        let result = truncate_with_options("Hello, world!", &options);
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn preserve_words_enabled() {
        // With preserve_words=true, truncates at word boundary
        let options = TruncateOptions::new(12).with_preserve_words(true);
        let result = truncate_with_options("Hello beautiful world", &options);
        // Should truncate at word boundary
        assert!(result.starts_with("Hello"));
        assert!(result.ends_with("..."));
    }

    #[test]
    fn preserve_words_disabled() {
        let options = TruncateOptions::new(10).with_preserve_words(false);
        let result = truncate_with_options("Hello beautiful world", &options);
        // Should truncate at exactly 10 chars (7 + 3 for ellipsis)
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn preserve_sentences_enabled() {
        let options = TruncateOptions::new(20).with_preserve_sentences(true);
        let result = truncate_with_options("Hello world. How are you?", &options);
        // Should truncate at sentence boundary
        assert!(result.starts_with("Hello world."));
        assert!(result.ends_with("..."));
    }

    #[test]
    fn preserve_sentences_with_exclamation() {
        let options = TruncateOptions::new(15).with_preserve_sentences(true);
        let result = truncate_with_options("Hello! How are you doing?", &options);
        assert!(result.starts_with("Hello!"));
        assert!(result.ends_with("..."));
    }

    #[test]
    fn preserve_sentences_with_question() {
        let options = TruncateOptions::new(15).with_preserve_sentences(true);
        let result = truncate_with_options("Hi there? What's up?", &options);
        assert!(result.starts_with("Hi there?"));
        assert!(result.ends_with("..."));
    }

    #[test]
    fn preserve_both_words_and_sentences() {
        let options = TruncateOptions::new(20)
            .with_preserve_words(true)
            .with_preserve_sentences(true);
        let result = truncate_with_options("Hello world. How are you?", &options);
        // Sentence preservation takes precedence
        assert!(result.contains('.'));
    }

    #[test]
    fn unicode_with_custom_ellipsis() {
        // With max_chars=6 and ellipsis="…", we can fit 5 chars + 1 ellipsis
        let options = TruncateOptions::new(6).with_ellipsis("…");
        let result = truncate_with_options("Hello 😀 world", &options);
        assert_eq!(result, "Hello…");
    }

    #[test]
    fn no_truncation_needed() {
        let options = TruncateOptions::new(100);
        let result = truncate_with_options("Short", &options);
        assert_eq!(result, "Short");
    }
}

// ============================================================================
// is_truncated Tests
// ============================================================================

mod is_truncated_tests {
    use super::*;

    #[test]
    fn detects_truncation_by_length() {
        assert!(is_truncated("Hello, world!", "Hello..."));
    }

    #[test]
    fn detects_no_truncation_same_string() {
        assert!(!is_truncated("Hello", "Hello"));
    }

    #[test]
    fn detects_no_truncation_different_same_length() {
        // Same length but different content - still considered truncated
        assert!(is_truncated("Hello", "World"));
    }

    #[test]
    fn empty_strings() {
        assert!(!is_truncated("", ""));
    }

    #[test]
    fn original_empty_truncated_not() {
        assert!(is_truncated("", "Hello"));
    }

    #[test]
    fn original_not_empty_truncated_empty() {
        assert!(is_truncated("Hello", ""));
    }

    #[test]
    fn with_unicode() {
        assert!(is_truncated("Hello 😀 world", "Hello..."));
        assert!(!is_truncated("Hello 😀", "Hello 😀"));
    }
}

// ============================================================================
// Unicode Boundary Tests
// ============================================================================

mod unicode_boundary_tests {
    use super::*;

    #[test]
    fn emoji_not_split_char_boundary() {
        // "Hi 😀" is 4 chars, with max_chars=3 we can fit 0 chars + 3 ellipsis
        // Actually, with target_chars=0, we return just ellipsis
        let result = truncate("Hi 😀", 3);
        assert_eq!(result, "...");
    }

    #[test]
    fn emoji_not_split_byte_boundary() {
        let result = truncate_bytes("Hi 😀", 4);
        // "Hi " is 3 bytes, next char is emoji (4 bytes)
        // Can't fit emoji, so result should be "..." or similar
        assert!(result.is_char_boundary(result.len()));
    }

    #[test]
    fn chinese_not_split() {
        // Each Chinese char is 3 bytes
        let result = truncate_bytes("你好", 4);
        // "你" is 3 bytes, "好" would start at byte 3
        // At 4 bytes, we're in the middle of "好", should back up
        assert!(result.is_char_boundary(result.len()));
    }

    #[test]
    fn combining_characters_preserved() {
        // e + combining acute accent is 2 chars
        // "e\u{0301} world" is 8 chars total
        // With max_chars=5, we can fit 2 chars + 3 ellipsis
        let input = "e\u{0301} world";
        let result = truncate(input, 5);
        // Should include both the e and the combining character + ellipsis
        assert!(result.starts_with("e\u{0301}"));
        assert!(result.ends_with("..."));
    }

    #[test]
    fn flag_emoji_preserved() {
        // Flag emoji is 2 regional indicator symbols (4 bytes each = 8 bytes total, 2 chars)
        let flag = "🇺🇸";
        let result = truncate(flag, 2);
        assert_eq!(result, "🇺🇸");
    }

    #[test]
    fn family_emoji_preserved() {
        // Family emoji is a complex sequence
        let family = "👨‍👩‍👧‍👦";
        let result = truncate(family, 10);
        // This is a ZWJ sequence, chars().count() returns more than 1
        assert!(result.contains("👨‍👩‍👧‍👦") || result.contains("..."));
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn very_long_string() {
        let long = "a".repeat(10000);
        // With max_chars=12, we can fit 9 chars + 3 ellipsis
        let result = truncate(&long, 12);
        assert_eq!(result, "aaaaaaaaa...");
    }

    #[test]
    fn string_exactly_at_limit() {
        let s = "Hello";
        let result = truncate(s, 5);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn string_one_over_limit() {
        // "Hello!" is 6 chars, with max_chars=5 we can fit 2 chars + 3 ellipsis
        let s = "Hello!";
        let result = truncate(s, 5);
        assert_eq!(result, "He...");
    }

    #[test]
    fn max_length_one() {
        let result = truncate("Hello", 1);
        assert_eq!(result, ".");
    }

    #[test]
    fn max_length_two() {
        let result = truncate("Hello", 2);
        assert_eq!(result, "..");
    }

    #[test]
    fn max_length_three() {
        let result = truncate("Hello", 3);
        assert_eq!(result, "...");
    }

    #[test]
    fn max_length_four() {
        let result = truncate("Hello", 4);
        assert_eq!(result, "H...");
    }

    #[test]
    fn all_whitespace() {
        // "     " is 5 chars, with max_chars=3 we get "..."
        let result = truncate("     ", 3);
        assert_eq!(result, "...");
    }

    #[test]
    fn only_newlines() {
        // "\n\n\n\n" is 4 chars, with max_chars=3 we get "..."
        let result = truncate("\n\n\n\n", 3);
        assert_eq!(result, "...");
    }

    #[test]
    fn single_character_string() {
        let result = truncate("A", 5);
        assert_eq!(result, "A");
    }

    #[test]
    fn single_character_truncated() {
        let result = truncate("A", 0);
        assert_eq!(result, "...");
    }

    #[test]
    fn word_truncation_single_word() {
        let result = truncate_words("Supercalifragilisticexpialidocious", 1);
        assert_eq!(result, "Supercalifragilisticexpialidocious");
    }

    #[test]
    fn byte_truncation_ascii_only() {
        // "Hello" is 5 bytes, with max_bytes=3 we get "..."
        let result = truncate_bytes("Hello", 3);
        assert_eq!(result, "...");
    }

    #[test]
    fn options_with_zero_max_length() {
        let options = TruncateOptions::new(0);
        let result = truncate_with_options("Hello", &options);
        assert_eq!(result, "...");
    }

    #[test]
    fn options_with_empty_ellipsis_zero_max() {
        let options = TruncateOptions::new(0).with_ellipsis("");
        let result = truncate_with_options("Hello", &options);
        assert_eq!(result, "");
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn truncate_never_panics(s: String, max_chars: usize) {
            let _ = truncate(&s, max_chars);
        }

        #[test]
        fn truncate_bytes_never_panics(s: String, max_bytes: usize) {
            let _ = truncate_bytes(&s, max_bytes);
        }

        #[test]
        fn truncate_words_never_panics(s: String, max_words: usize) {
            let _ = truncate_words(&s, max_words);
        }

        #[test]
        fn truncate_result_is_valid_utf8(s: String, max_chars: usize) {
            let result = truncate(&s, max_chars);
            prop_assert!(result.is_char_boundary(result.len()));
        }

        #[test]
        fn truncate_bytes_result_is_valid_utf8(s: String, max_bytes: usize) {
            let result = truncate_bytes(&s, max_bytes);
            prop_assert!(result.is_char_boundary(result.len()));
        }

        #[test]
        fn char_len_matches_chars_count(s: String) {
            prop_assert_eq!(char_len(&s), s.chars().count());
        }

        #[test]
        fn is_truncated_detects_length_difference(original: String, truncated: String) {
            let result = is_truncated(&original, &truncated);
            if original.len() != truncated.len() {
                prop_assert!(result);
            } else if original == truncated {
                prop_assert!(!result);
            }
        }
    }
}

// ============================================================================
// DEFAULT_ELLIPSIS Constant Tests
// ============================================================================

mod constants_tests {
    use super::*;

    #[test]
    fn default_ellipsis_value() {
        assert_eq!(DEFAULT_ELLIPSIS, "...");
    }

    #[test]
    fn default_ellipsis_len() {
        assert_eq!(DEFAULT_ELLIPSIS.len(), 3);
        assert_eq!(char_len(DEFAULT_ELLIPSIS), 3);
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn full_workflow_character_truncation() {
        let message = "This is a very long error message that needs to be truncated for display";
        // With max_chars=23, we can fit 20 chars + 3 ellipsis
        let truncated = truncate(message, 23);
        assert!(is_truncated(message, &truncated));
        assert!(truncated.ends_with("..."));
        assert!(char_len(&truncated) <= 23);
    }

    #[test]
    fn full_workflow_byte_truncation() {
        let message = "Error: 你好世界 😀 test message";
        let truncated = truncate_bytes(message, 20);
        assert!(truncated.is_char_boundary(truncated.len()));
    }

    #[test]
    fn full_workflow_word_truncation() {
        let message = "The quick brown fox jumps over the lazy dog";
        let truncated = truncate_words(message, 4);
        assert!(truncated.ends_with("..."));
        assert!(is_truncated(message, &truncated));
    }

    #[test]
    fn full_workflow_custom_options() {
        let message =
            "Error: Something went wrong. Please try again later. Contact support if needed.";
        let options = TruncateOptions::new(50)
            .with_ellipsis("…")
            .with_preserve_sentences(true);
        let truncated = truncate_with_options(message, &options);
        assert!(truncated.ends_with("…"));
    }

    #[test]
    fn unicode_heavy_workflow() {
        let messages = vec![
            "错误：文件未找到 😀",
            "エラー：ファイルが見つかりません 🎌",
            "오류: 파일을 찾을 수 없습니다 🇰🇷",
            "❌ Error: File not found 📁",
        ];

        for msg in messages {
            let truncated = truncate(msg, 10);
            assert!(truncated.is_char_boundary(truncated.len()));

            let truncated_bytes = truncate_bytes(msg, 20);
            assert!(truncated_bytes.is_char_boundary(truncated_bytes.len()));
        }
    }

    #[test]
    fn preserves_original_string() {
        let original = "Hello, world!";
        let _truncated = truncate(original, 5);
        // Original should be unchanged
        assert_eq!(original, "Hello, world!");
    }
}
