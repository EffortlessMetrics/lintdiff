//! Comprehensive BDD tests for lintdiff-slugify crate.
//!
//! These tests cover:
//! - Basic slugification
//! - Configuration options
//! - Special character handling
//! - Edge cases
//! - Property-based tests with proptest

use std::borrow::Cow;

use lintdiff_slugify::{
    slugify, slugify_cow, slugify_with_options, SlugOptions, Slugifier, SlugifierBuilder,
};

// =============================================================================
// Basic Slugification Tests
// =============================================================================

mod basic_slugify_tests {
    use super::*;

    #[test]
    fn test_simple_two_words() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn test_single_word() {
        assert_eq!(slugify("hello"), "hello");
    }

    #[test]
    fn test_uppercase_conversion() {
        assert_eq!(slugify("HELLO"), "hello");
        assert_eq!(slugify("HeLLo WoRLD"), "hello-world");
    }

    #[test]
    fn test_mixed_case() {
        assert_eq!(slugify("MyAwesomeTitle"), "myawesometitle");
    }

    #[test]
    fn test_numbers_preserved() {
        assert_eq!(slugify("Test123"), "test123");
        assert_eq!(slugify("2024Year"), "2024year");
    }

    #[test]
    fn test_numbers_with_spaces() {
        assert_eq!(slugify("Version 2 0"), "version-2-0");
    }

    #[test]
    fn test_already_slugified() {
        assert_eq!(slugify("already-slugified"), "already-slugified");
    }

    #[test]
    fn test_multiple_words() {
        assert_eq!(
            slugify("This is a long title"),
            "this-is-a-long-title"
        );
    }
}

// =============================================================================
// Empty String and Edge Cases
// =============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn test_only_whitespace() {
        assert_eq!(slugify("   "), "");
        assert_eq!(slugify("\t\n"), "");
    }

    #[test]
    fn test_only_special_chars() {
        assert_eq!(slugify("@#$%"), "");
        assert_eq!(slugify("!!!"), "");
    }

    #[test]
    fn test_leading_whitespace() {
        assert_eq!(slugify("  hello"), "hello");
        assert_eq!(slugify("\t\nhello"), "hello");
    }

    #[test]
    fn test_trailing_whitespace() {
        assert_eq!(slugify("hello  "), "hello");
        assert_eq!(slugify("hello\t\n"), "hello");
    }

    #[test]
    fn test_leading_and_trailing_whitespace() {
        assert_eq!(slugify("  hello  "), "hello");
    }

    #[test]
    fn test_leading_special_chars() {
        assert_eq!(slugify("@hello"), "hello");
        assert_eq!(slugify("!!!test"), "test");
    }

    #[test]
    fn test_trailing_special_chars() {
        assert_eq!(slugify("hello@"), "hello");
        assert_eq!(slugify("test!!!"), "test");
    }

    #[test]
    fn test_single_character() {
        assert_eq!(slugify("a"), "a");
        assert_eq!(slugify("A"), "a");
        assert_eq!(slugify("1"), "1");
    }

    #[test]
    fn test_single_special_char() {
        assert_eq!(slugify("@"), "");
        assert_eq!(slugify(" "), "");
    }
}

// =============================================================================
// Whitespace Handling Tests
// =============================================================================

mod whitespace_tests {
    use super::*;

    #[test]
    fn test_multiple_spaces_collapsed() {
        assert_eq!(slugify("Multiple   Spaces"), "multiple-spaces");
    }

    #[test]
    fn test_tabs_converted() {
        assert_eq!(slugify("hello\tworld"), "hello-world");
    }

    #[test]
    fn test_newlines_converted() {
        assert_eq!(slugify("hello\nworld"), "hello-world");
        assert_eq!(slugify("hello\r\nworld"), "hello-world");
    }

    #[test]
    fn test_mixed_whitespace() {
        assert_eq!(slugify("hello \t\n world"), "hello-world");
    }

    #[test]
    fn test_consecutive_whitespace_types() {
        assert_eq!(slugify("a  \t  \n  b"), "a-b");
    }
}

// =============================================================================
// Special Character Handling Tests
// =============================================================================

mod special_char_tests {
    use super::*;

    #[test]
    fn test_at_symbol() {
        assert_eq!(slugify("test@example"), "test-example");
    }

    #[test]
    fn test_hash_symbol() {
        assert_eq!(slugify("tag#123"), "tag-123");
    }

    #[test]
    fn test_dollar_symbol() {
        assert_eq!(slugify("price$100"), "price-100");
    }

    #[test]
    fn test_percent_symbol() {
        assert_eq!(slugify("100%complete"), "100-complete");
    }

    #[test]
    fn test_ampersand() {
        assert_eq!(slugify("foo&bar"), "foo-bar");
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("What? Why!"), "what-why");
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(slugify("(test)"), "test");
        assert_eq!(slugify("func(arg)"), "func-arg");
    }

    #[test]
    fn test_brackets() {
        assert_eq!(slugify("[test]"), "test");
        assert_eq!(slugify("array[0]"), "array-0");
    }

    #[test]
    fn test_braces() {
        assert_eq!(slugify("{test}"), "test");
    }

    #[test]
    fn test_slashes() {
        assert_eq!(slugify("path/to/file"), "path-to-file");
        assert_eq!(slugify("back\\slash"), "back-slash");
    }

    #[test]
    fn test_colons_and_semicolons() {
        assert_eq!(slugify("error: test"), "error-test");
        assert_eq!(slugify("a;b"), "a-b");
    }

    #[test]
    fn test_quotes() {
        assert_eq!(slugify("\"quoted\""), "quoted");
        assert_eq!(slugify("'single'"), "single");
    }

    #[test]
    fn test_math_symbols() {
        assert_eq!(slugify("2+2=4"), "2-2-4");
        assert_eq!(slugify("x*y"), "x-y");
    }

    #[test]
    fn test_multiple_special_chars_consecutive() {
        assert_eq!(slugify("a!!!b"), "a-b");
        assert_eq!(slugify("test@@@example"), "test-example");
    }

    #[test]
    fn test_special_between_words() {
        assert_eq!(slugify("foo@bar"), "foo-bar");
        assert_eq!(slugify("hello#world"), "hello-world");
    }
}

// =============================================================================
// Preserve Special Characters Option Tests
// =============================================================================

mod preserve_special_tests {
    use super::*;

    fn preserve_options() -> SlugOptions {
        SlugOptions::new().with_preserve_special(true)
    }

    #[test]
    fn test_preserve_at_symbol() {
        let options = preserve_options();
        assert_eq!(slugify_with_options("test@example", &options), "test@example");
    }

    #[test]
    fn test_preserve_hash_symbol() {
        let options = preserve_options();
        assert_eq!(slugify_with_options("tag#123", &options), "tag#123");
    }

    #[test]
    fn test_preserve_dollar_symbol() {
        let options = preserve_options();
        assert_eq!(slugify_with_options("price$100", &options), "price$100");
    }

    #[test]
    fn test_preserve_percent_symbol() {
        let options = preserve_options();
        assert_eq!(slugify_with_options("100%", &options), "100%");
    }

    #[test]
    fn test_preserve_ampersand() {
        let options = preserve_options();
        assert_eq!(slugify_with_options("foo&bar", &options), "foo&bar");
    }

    #[test]
    fn test_preserve_plus() {
        let options = preserve_options();
        assert_eq!(slugify_with_options("c++", &options), "c++");
    }

    #[test]
    fn test_preserve_equals() {
        let options = preserve_options();
        assert_eq!(slugify_with_options("a=b", &options), "a=b");
    }

    #[test]
    fn test_preserve_tilde() {
        let options = preserve_options();
        assert_eq!(slugify_with_options("~home", &options), "~home");
    }

    #[test]
    fn test_preserve_backtick() {
        let options = preserve_options();
        assert_eq!(slugify_with_options("`code`", &options), "`code`");
    }

    #[test]
    fn test_preserve_multiple_special() {
        let options = preserve_options();
        assert_eq!(
            slugify_with_options("test@#$value", &options),
            "test@#$value"
        );
    }

    #[test]
    fn test_preserve_with_spaces() {
        let options = preserve_options();
        assert_eq!(
            slugify_with_options("hello @ world", &options),
            "hello-@-world"
        );
    }

    #[test]
    fn test_preserve_leads_to_separator() {
        let options = preserve_options();
        // Special char at start followed by space should not create double separator
        assert_eq!(slugify_with_options("@ test", &options), "@-test");
    }
}

// =============================================================================
// Separator Option Tests
// =============================================================================

mod separator_tests {
    use super::*;

    #[test]
    fn test_underscore_separator() {
        let options = SlugOptions::new().with_separator('_');
        assert_eq!(slugify_with_options("Hello World", &options), "hello_world");
    }

    #[test]
    fn test_dot_separator() {
        let options = SlugOptions::new().with_separator('.');
        assert_eq!(slugify_with_options("Hello World", &options), "hello.world");
    }

    #[test]
    fn test_tilde_separator() {
        let options = SlugOptions::new().with_separator('~');
        assert_eq!(slugify_with_options("Hello World", &options), "hello~world");
    }

    #[test]
    fn test_plus_separator() {
        let options = SlugOptions::new().with_separator('+');
        assert_eq!(slugify_with_options("Hello World", &options), "hello+world");
    }

    #[test]
    fn test_multiple_words_with_custom_separator() {
        let options = SlugOptions::new().with_separator('_');
        assert_eq!(
            slugify_with_options("One Two Three Four", &options),
            "one_two_three_four"
        );
    }

    #[test]
    fn test_separator_with_max_length() {
        let options = SlugOptions::new()
            .with_separator('_')
            .with_max_length(15);
        assert_eq!(
            slugify_with_options("One Two Three Four Five", &options),
            "one_two_three"
        );
    }
}

// =============================================================================
// Max Length Option Tests
// =============================================================================

mod max_length_tests {
    use super::*;

    #[test]
    fn test_short_max_length() {
        let options = SlugOptions::new().with_max_length(5);
        assert_eq!(slugify_with_options("Hello World", &options), "hello");
    }

    #[test]
    fn test_exact_max_length() {
        let options = SlugOptions::new().with_max_length(11);
        assert_eq!(slugify_with_options("Hello World", &options), "hello-world");
    }

    #[test]
    fn test_max_length_truncation() {
        let options = SlugOptions::new().with_max_length(10);
        assert_eq!(slugify_with_options("Very Long String", &options), "very-long");
    }

    #[test]
    fn test_max_length_zero() {
        let options = SlugOptions::new().with_max_length(0);
        assert_eq!(slugify_with_options("Hello World", &options), "");
    }

    #[test]
    fn test_max_length_one() {
        let options = SlugOptions::new().with_max_length(1);
        assert_eq!(slugify_with_options("Hello World", &options), "h");
    }

    #[test]
    fn test_max_length_preserves_word_boundary() {
        let options = SlugOptions::new().with_max_length(10);
        // Should truncate at word boundary when possible
        let result = slugify_with_options("Hello Beautiful World", &options);
        assert!(result.len() <= 10);
        assert!(!result.ends_with('-'));
    }

    #[test]
    fn test_max_length_long_string() {
        let options = SlugOptions::new().with_max_length(20);
        let long_input = "This is a very long string that should be truncated";
        let result = slugify_with_options(long_input, &options);
        assert!(result.len() <= 20);
    }

    #[test]
    fn test_max_length_no_truncation_needed() {
        let options = SlugOptions::new().with_max_length(100);
        assert_eq!(
            slugify_with_options("Short", &options),
            "short"
        );
    }
}

// =============================================================================
// Lowercase Option Tests
// =============================================================================

mod lowercase_tests {
    use super::*;

    #[test]
    fn test_lowercase_true() {
        let options = SlugOptions::new().with_lowercase(true);
        assert_eq!(slugify_with_options("HELLO WORLD", &options), "hello-world");
    }

    #[test]
    fn test_lowercase_false() {
        let options = SlugOptions::new().with_lowercase(false);
        assert_eq!(slugify_with_options("Hello World", &options), "Hello-World");
    }

    #[test]
    fn test_lowercase_false_preserves_case() {
        let options = SlugOptions::new().with_lowercase(false);
        assert_eq!(
            slugify_with_options("MyAwesomeTitle", &options),
            "MyAwesomeTitle"
        );
    }

    #[test]
    fn test_lowercase_false_with_special_chars() {
        let options = SlugOptions::new()
            .with_lowercase(false)
            .with_preserve_special(true);
        assert_eq!(
            slugify_with_options("Test@EXAMPLE", &options),
            "Test@EXAMPLE"
        );
    }

    #[test]
    fn test_lowercase_default_is_true() {
        let options = SlugOptions::new();
        assert!(options.lowercase);
    }
}

// =============================================================================
// Slugifier Tests
// =============================================================================

mod slugifier_tests {
    use super::*;

    #[test]
    fn test_slugifier_basic() {
        let slugifier = Slugifier::new(SlugOptions::new());
        assert_eq!(slugifier.slugify("Hello World"), "hello-world");
    }

    #[test]
    fn test_slugifier_options() {
        let options = SlugOptions::new().with_separator('_');
        let slugifier = Slugifier::new(options.clone());
        assert_eq!(slugifier.options().separator, '_');
    }

    #[test]
    fn test_slugifier_reusable() {
        let slugifier = Slugifier::new(SlugOptions::new());
        assert_eq!(slugifier.slugify("First Test"), "first-test");
        assert_eq!(slugifier.slugify("Second Test"), "second-test");
        assert_eq!(slugifier.slugify("Third Test"), "third-test");
    }

    #[test]
    fn test_slugifier_clone() {
        let slugifier = Slugifier::new(SlugOptions::new().with_separator('_'));
        let cloned = slugifier.clone();
        assert_eq!(cloned.slugify("Test String"), "test_string");
    }
}

// =============================================================================
// SlugifierBuilder Tests
// =============================================================================

mod builder_tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let slugifier = SlugifierBuilder::new().build();
        assert_eq!(slugifier.slugify("Hello World"), "hello-world");
    }

    #[test]
    fn test_builder_with_lowercase() {
        let slugifier = SlugifierBuilder::new()
            .with_lowercase(false)
            .build();
        assert_eq!(slugifier.slugify("Hello World"), "Hello-World");
    }

    #[test]
    fn test_builder_with_separator() {
        let slugifier = SlugifierBuilder::new()
            .with_separator('_')
            .build();
        assert_eq!(slugifier.slugify("Hello World"), "hello_world");
    }

    #[test]
    fn test_builder_with_max_length() {
        let slugifier = SlugifierBuilder::new()
            .with_max_length(10)
            .build();
        assert_eq!(slugifier.slugify("Very Long String"), "very-long");
    }

    #[test]
    fn test_builder_with_preserve_special() {
        let slugifier = SlugifierBuilder::new()
            .with_preserve_special(true)
            .build();
        assert_eq!(slugifier.slugify("test@example"), "test@example");
    }

    #[test]
    fn test_builder_chained_options() {
        let slugifier = SlugifierBuilder::new()
            .with_lowercase(false)
            .with_separator('_')
            .with_max_length(20)
            .build();
        assert_eq!(
            slugifier.slugify("Hello World Test"),
            "Hello_World_Test"
        );
    }

    #[test]
    fn test_builder_all_options() {
        let slugifier = SlugifierBuilder::new()
            .with_lowercase(true)
            .with_preserve_special(true)
            .with_preserve_alphanumeric(true)
            .with_max_length(50)
            .with_separator('-')
            .build();
        assert_eq!(slugifier.slugify("Test@Example"), "test@example");
    }

    #[test]
    fn test_builder_reusable() {
        let builder = SlugifierBuilder::new().with_separator('_');
        let s1 = builder.clone().build();
        let s2 = builder.build();
        assert_eq!(s1.slugify("Test"), s2.slugify("Test"));
    }
}

// =============================================================================
// Cow (Zero-Copy) Tests
// =============================================================================

mod cow_tests {
    use super::*;

    #[test]
    fn test_cow_borrowed_no_change_needed() {
        let result = slugify_cow("already-slug");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "already-slug");
    }

    #[test]
    fn test_cow_borrowed_simple() {
        let result = slugify_cow("simple");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_cow_owned_uppercase() {
        let result = slugify_cow("UPPERCASE");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "uppercase");
    }

    #[test]
    fn test_cow_owned_spaces() {
        let result = slugify_cow("has spaces");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "has-spaces");
    }

    #[test]
    fn test_cow_owned_special_chars() {
        let result = slugify_cow("special@chars");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "special-chars");
    }

    #[test]
    fn test_cow_empty_string() {
        let result = slugify_cow("");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "");
    }

    #[test]
    fn test_cow_numbers_only() {
        let result = slugify_cow("12345");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_cow_with_hyphen() {
        let result = slugify_cow("already-hyphenated");
        assert!(matches!(result, Cow::Borrowed(_)));
    }
}

// =============================================================================
// Unicode and Internationalization Tests
// =============================================================================

mod unicode_tests {
    use super::*;

    #[test]
    fn test_unicode_replaced() {
        // Non-ASCII characters should be replaced with separator
        let result = slugify("café");
        assert!(result.contains("caf"));
    }

    #[test]
    fn test_emoji_replaced() {
        let result = slugify("hello 🌍 world");
        assert_eq!(result, "hello-world");
    }

    #[test]
    fn test_chinese_replaced() {
        let result = slugify("hello世界world");
        assert_eq!(result, "hello-world");
    }

    #[test]
    fn test_japanese_replaced() {
        let result = slugify("testテストtest");
        assert_eq!(result, "test-test");
    }

    #[test]
    fn test_cyrillic_replaced() {
        let result = slugify("helloприветworld");
        assert_eq!(result, "hello-world");
    }

    #[test]
    fn test_accented_chars_replaced() {
        let result = slugify("naïve résumé");
        assert!(result.starts_with("na"));
        assert!(result.contains("r"));
    }
}

// =============================================================================
// Combination Tests
// =============================================================================

mod combination_tests {
    use super::*;

    #[test]
    fn test_all_options_combined() {
        let options = SlugOptions::new()
            .with_lowercase(true)
            .with_preserve_special(true)
            .with_separator('_')
            .with_max_length(30);

        let result = slugify_with_options("Hello World @ Test String Long", &options);
        assert!(result.len() <= 30);
        assert!(result.contains('_'));
    }

    #[test]
    fn test_preserve_special_no_lowercase() {
        let options = SlugOptions::new()
            .with_lowercase(false)
            .with_preserve_special(true);

        assert_eq!(
            slugify_with_options("Test@EXAMPLE#123", &options),
            "Test@EXAMPLE#123"
        );
    }

    #[test]
    fn test_custom_separator_with_special_chars() {
        let options = SlugOptions::new()
            .with_separator('.')
            .with_preserve_special(false);

        assert_eq!(
            slugify_with_options("Hello@World Test", &options),
            "hello.world.test"
        );
    }

    #[test]
    fn test_complex_string_all_features() {
        let options = SlugOptions::new()
            .with_lowercase(true)
            .with_separator('-')
            .with_max_length(50);

        let input = "This is a COMPLEX Test String!!! With @ Special # Characters $";
        let result = slugify_with_options(input, &options);
        assert!(result.len() <= 50);
        assert!(result.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
    }
}

// =============================================================================
// Real-World Use Case Tests
// =============================================================================

mod real_world_tests {
    use super::*;

    #[test]
    fn test_blog_title() {
        assert_eq!(
            slugify("My Awesome Blog Post Title"),
            "my-awesome-blog-post-title"
        );
    }

    #[test]
    fn test_file_name() {
        assert_eq!(
            slugify("Document (Final) [v2].pdf"),
            "document-final-v2-pdf"
        );
    }

    #[test]
    fn test_url_path() {
        assert_eq!(
            slugify("/api/v1/users/123"),
            "api-v1-users-123"
        );
    }

    #[test]
    fn test_email_local_part() {
        assert_eq!(
            slugify("user.name+tag@domain.com"),
            "user-name-tag-domain-com"
        );
    }

    #[test]
    fn test_code_identifier() {
        assert_eq!(
            slugify("myFunctionName"),
            "myfunctionname"
        );
    }

    #[test]
    fn test_error_message() {
        assert_eq!(
            slugify("Error: Failed to connect (timeout)"),
            "error-failed-to-connect-timeout"
        );
    }

    #[test]
    fn test_markdown_header() {
        assert_eq!(
            slugify("## Introduction to Rust"),
            "introduction-to-rust"
        );
    }

    #[test]
    fn test_version_string() {
        assert_eq!(
            slugify("v1.2.3-beta.1"),
            "v1-2-3-beta-1"
        );
    }

    #[test]
    fn test_commit_message() {
        assert_eq!(
            slugify("feat: add new feature (#123)"),
            "feat-add-new-feature-123"
        );
    }

    #[test]
    fn test_package_name() {
        assert_eq!(
            slugify("@scope/package-name"),
            "scope-package-name"
        );
    }
}

// =============================================================================
// Property-Based Tests with proptest
// =============================================================================

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_slugify_never_contains_whitespace(s in ".*") {
            let result = slugify(&s);
            prop_assert!(!result.contains(char::is_whitespace));
        }

        #[test]
        fn test_slugify_never_has_consecutive_separators(s in ".*") {
            let result = slugify(&s);
            prop_assert!(!result.contains("--"));
        }

        #[test]
        fn test_slugify_never_starts_with_separator(s in ".*") {
            let result = slugify(&s);
            prop_assert!(!result.starts_with('-') || result.is_empty());
        }

        #[test]
        fn test_slugify_never_ends_with_separator(s in ".*") {
            let result = slugify(&s);
            prop_assert!(!result.ends_with('-') || result.is_empty());
        }

        #[test]
        fn test_slugify_is_deterministic(s in ".*") {
            let result1 = slugify(&s);
            let result2 = slugify(&s);
            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn test_slugify_preserves_alphanumeric(s in "[a-zA-Z0-9]+") {
            let result = slugify(&s);
            let expected = s.to_lowercase();
            prop_assert_eq!(result, expected);
        }

        #[test]
        fn test_slugify_max_length_respected(s in ".*", max in 0usize..100usize) {
            let options = SlugOptions::new().with_max_length(max);
            let result = slugify_with_options(&s, &options);
            prop_assert!(result.len() <= max);
        }

        #[test]
        fn test_slugify_empty_stays_empty(s in "") {
            let result = slugify(&s);
            prop_assert!(result.is_empty());
        }

        #[test]
        fn test_slugify_ascii_printable(s in "[ -~]+") {
            // All ASCII printable characters should produce valid slugs
            let result = slugify(&s);
            // Result should only contain lowercase alphanumeric and hyphens
            for c in result.chars() {
                prop_assert!(
                    c.is_ascii_lowercase() ||
                    c.is_ascii_digit() ||
                    c == '-'
                );
            }
        }

        #[test]
        fn test_slugify_with_separator_only_alnum_and_separator(s in ".*", sep in any::<char>()) {
            // Skip control characters and whitespace for separator
            prop_assume!(!sep.is_control() && !sep.is_whitespace());

            let options = SlugOptions::new().with_separator(sep);
            let result = slugify_with_options(&s, &options);

            // Result should only contain lowercase alphanumeric and the separator
            for c in result.chars() {
                prop_assert!(
                    c.is_ascii_lowercase() ||
                    c.is_ascii_digit() ||
                    c == sep
                );
            }
        }

        #[test]
        fn test_slugify_cow_borrowed_for_valid_slugs(s in "[a-z0-9-]+") {
            let result = slugify_cow(&s);
            prop_assert!(matches!(result, Cow::Borrowed(_)));
        }

        #[test]
        fn test_slugify_numbers_preserved(s in "[0-9]+") {
            let result = slugify(&s);
            prop_assert_eq!(result, s);
        }

        #[test]
        fn test_builder_produces_consistent_results(s in ".*") {
            let slugifier1 = SlugifierBuilder::new().build();
            let slugifier2 = SlugifierBuilder::new().build();

            prop_assert_eq!(slugifier1.slugify(&s), slugifier2.slugify(&s));
        }
    }
}

// =============================================================================
// SlugOptions Tests
// =============================================================================

mod options_tests {
    use super::*;

    #[test]
    fn test_options_default() {
        let options = SlugOptions::default();
        assert!(options.lowercase);
        assert!(!options.preserve_special);
        assert!(options.preserve_alphanumeric);
        assert!(options.max_length.is_none());
        assert_eq!(options.separator, '-');
    }

    #[test]
    fn test_options_new() {
        let options = SlugOptions::new();
        assert!(options.lowercase);
        assert_eq!(options.separator, '-');
    }

    #[test]
    fn test_options_chained_builders() {
        let options = SlugOptions::new()
            .with_lowercase(false)
            .with_separator('_')
            .with_max_length(100)
            .with_preserve_special(true);

        assert!(!options.lowercase);
        assert_eq!(options.separator, '_');
        assert_eq!(options.max_length, Some(100));
        assert!(options.preserve_special);
    }

    #[test]
    fn test_options_clone() {
        let options = SlugOptions::new().with_separator('_');
        let cloned = options.clone();
        assert_eq!(options.separator, cloned.separator);
    }

    #[test]
    fn test_options_debug() {
        let options = SlugOptions::new();
        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("lowercase"));
        assert!(debug_str.contains("separator"));
    }
}

// =============================================================================
// Truncate at Boundary Tests
// =============================================================================

mod truncate_boundary_tests {
    use super::*;

    #[test]
    fn test_truncate_at_word_boundary() {
        let options = SlugOptions::new().with_max_length(15);
        // "hello-beautiful-world" is 22 chars
        // Should truncate at a word boundary before position 15
        let result = slugify_with_options("hello beautiful world", &options);
        assert!(result.len() <= 15);
        assert!(!result.ends_with('-'));
    }

    #[test]
    fn test_truncate_preserves_readability() {
        let options = SlugOptions::new().with_max_length(20);
        let result = slugify_with_options("This is a very long sentence that needs truncation", &options);
        assert!(result.len() <= 20);
        // Should not end with a separator
        assert!(!result.ends_with('-'));
    }

    #[test]
    fn test_truncate_single_long_word() {
        let options = SlugOptions::new().with_max_length(5);
        let result = slugify_with_options("supercalifragilistic", &options);
        assert_eq!(result, "super");
    }
}

// =============================================================================
// Serde Tests (feature-gated)
// =============================================================================

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    #[test]
    fn test_options_serialize() {
        let options = SlugOptions::new().with_max_length(50);
        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("max_length"));
    }

    #[test]
    fn test_options_deserialize() {
        let json = r#"{"lowercase":true,"preserve_special":false,"preserve_alphanumeric":true,"max_length":50,"separator":"-"}"#;
        let options: SlugOptions = serde_json::from_str(json).unwrap();
        assert_eq!(options.max_length, Some(50));
        assert_eq!(options.separator, '-');
    }

    #[test]
    fn test_options_roundtrip() {
        let original = SlugOptions::new()
            .with_lowercase(false)
            .with_separator('_')
            .with_max_length(100);

        let json = serde_json::to_string(&original).unwrap();
        let restored: SlugOptions = serde_json::from_str(&json).unwrap();

        assert_eq!(original.lowercase, restored.lowercase);
        assert_eq!(original.separator, restored.separator);
        assert_eq!(original.max_length, restored.max_length);
    }
}
