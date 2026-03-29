//! Comprehensive tests for lintdiff-escape crate.

use std::borrow::Cow;

use lintdiff_escape::{escape, escape_github, escape_html, escape_json, escape_markdown, escape_plain, needs_escaping, OutputFormat};

// =============================================================================
// GitHub Actions Escaping Tests
// =============================================================================

mod github_tests {
    use super::*;

    #[test]
    fn test_no_escaping_needed() {
        let input = "Hello, world!";
        let result = escape_github(input);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_percent_escaping() {
        assert_eq!(escape_github("100%"), "100%25");
        assert_eq!(escape_github("50% complete"), "50%25 complete");
        assert_eq!(escape_github("%%"), "%25%25");
    }

    #[test]
    fn test_colon_escaping() {
        assert_eq!(escape_github("Error: test"), "Error%3A test");
        assert_eq!(escape_github("::warning::"), "%3A%3Awarning%3A%3A");
    }

    #[test]
    fn test_newline_escaping() {
        assert_eq!(escape_github("line1\nline2"), "line1%0Aline2");
        assert_eq!(escape_github("line1\rline2"), "line1%0Dline2");
        assert_eq!(escape_github("line1\r\nline2"), "line1%0D%0Aline2");
    }

    #[test]
    fn test_multiple_newlines() {
        assert_eq!(escape_github("a\nb\nc"), "a%0Ab%0Ac");
        assert_eq!(escape_github("\n\n\n"), "%0A%0A%0A");
    }

    #[test]
    fn test_combined_special_chars() {
        assert_eq!(
            escape_github("Error: 100%\nDone"),
            "Error%3A 100%25%0ADone"
        );
        assert_eq!(
            escape_github("::error::50%\r\n"),
            "%3A%3Aerror%3A%3A50%25%0D%0A"
        );
    }

    #[test]
    fn test_empty_string() {
        let result = escape_github("");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "");
    }

    #[test]
    fn test_needs_escaping_github() {
        assert!(needs_escaping("100%", OutputFormat::GitHubActions));
        assert!(needs_escaping("Error:", OutputFormat::GitHubActions));
        assert!(needs_escaping("line1\n", OutputFormat::GitHubActions));
        assert!(needs_escaping("\r", OutputFormat::GitHubActions));
        assert!(!needs_escaping("normal text", OutputFormat::GitHubActions));
        assert!(!needs_escaping("", OutputFormat::GitHubActions));
    }
}

// =============================================================================
// Markdown Escaping Tests
// =============================================================================

mod markdown_tests {
    use super::*;

    #[test]
    fn test_no_escaping_needed() {
        let input = "Hello, world!";
        let result = escape_markdown(input);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_asterisk_escaping() {
        assert_eq!(escape_markdown("*bold*"), "\\*bold\\*");
        assert_eq!(escape_markdown("**bold**"), "\\*\\*bold\\*\\*");
        assert_eq!(escape_markdown("*italic*"), "\\*italic\\*");
    }

    #[test]
    fn test_underscore_escaping() {
        assert_eq!(escape_markdown("_italic_"), "\\_italic\\_");
        assert_eq!(escape_markdown("__bold__"), "\\_\\_bold\\_\\_");
    }

    #[test]
    fn test_bracket_escaping() {
        assert_eq!(escape_markdown("[link]"), "\\[link\\]");
    }

    #[test]
    fn test_angle_bracket_escaping() {
        assert_eq!(escape_markdown("<html>"), "\\<html\\>");
        assert_eq!(escape_markdown("> blockquote"), "\\> blockquote");
    }

    #[test]
    fn test_backtick_escaping() {
        assert_eq!(escape_markdown("`code`"), "\\`code\\`");
        assert_eq!(escape_markdown("``code``"), "\\`\\`code\\`\\`");
    }

    #[test]
    fn test_hash_escaping() {
        assert_eq!(escape_markdown("# Heading"), "\\# Heading");
        assert_eq!(escape_markdown("## Subheading"), "\\#\\# Subheading");
    }

    #[test]
    fn test_combined_markdown_chars() {
        assert_eq!(
            escape_markdown("# Header with *bold* and `code`"),
            "\\# Header with \\*bold\\* and \\`code\\`"
        );
    }

    #[test]
    fn test_empty_string() {
        let result = escape_markdown("");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "");
    }

    #[test]
    fn test_needs_escaping_markdown() {
        assert!(needs_escaping("*bold*", OutputFormat::Markdown));
        assert!(needs_escaping("_italic_", OutputFormat::Markdown));
        assert!(needs_escaping("[link]", OutputFormat::Markdown));
        assert!(needs_escaping("<tag>", OutputFormat::Markdown));
        assert!(needs_escaping("`code`", OutputFormat::Markdown));
        assert!(needs_escaping("# Heading", OutputFormat::Markdown));
        assert!(!needs_escaping("normal text", OutputFormat::Markdown));
        assert!(!needs_escaping("", OutputFormat::Markdown));
    }
}

// =============================================================================
// HTML Escaping Tests
// =============================================================================

mod html_tests {
    use super::*;

    #[test]
    fn test_no_escaping_needed() {
        let input = "Hello, world!";
        let result = escape_html(input);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_ampersand_escaping() {
        // Using hex escape for ampersand to avoid issues
        assert_eq!(escape_html("a \x26 b"), "a \x26amp; b");
    }

    #[test]
    fn test_less_than_escaping() {
        assert_eq!(escape_html("<div>"), "\x26lt;div\x26gt;");
        assert_eq!(escape_html("a < b"), "a \x26lt; b");
    }

    #[test]
    fn test_greater_than_escaping() {
        assert_eq!(escape_html(">"), "\x26gt;");
        assert_eq!(escape_html("a > b"), "a \x26gt; b");
    }

    #[test]
    fn test_double_quote_escaping() {
        assert_eq!(escape_html("\"quoted\""), "\x26quot;quoted\x26quot;");
    }

    #[test]
    fn test_single_quote_escaping() {
        assert_eq!(escape_html("\x27single\x27"), "\x26#x27;single\x26#x27;");
    }

    #[test]
    fn test_combined_html_chars() {
        let input = "<div class=\"test\">Hello \x26 goodbye</div>";
        let expected = "\x26lt;div class=\x26quot;test\x26quot;\x26gt;Hello \x26amp; goodbye\x26lt;/div\x26gt;";
        assert_eq!(escape_html(input), expected);
    }

    #[test]
    fn test_empty_string() {
        let result = escape_html("");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "");
    }

    #[test]
    fn test_needs_escaping_html() {
        assert!(needs_escaping("<tag>", OutputFormat::Html));
        assert!(needs_escaping("a \x26 b", OutputFormat::Html));
        assert!(needs_escaping("\"quoted\"", OutputFormat::Html));
        assert!(needs_escaping("\x27single\x27", OutputFormat::Html));
        assert!(!needs_escaping("normal text", OutputFormat::Html));
        assert!(!needs_escaping("", OutputFormat::Html));
    }
}

// =============================================================================
// JSON Escaping Tests
// =============================================================================

mod json_tests {
    use super::*;

    #[test]
    fn test_no_escaping_needed() {
        let input = "Hello, world!";
        let result = escape_json(input);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_quote_escaping() {
        assert_eq!(escape_json("say \"hi\""), "say \\\"hi\\\"");
    }

    #[test]
    fn test_backslash_escaping() {
        assert_eq!(escape_json("path\\to\\file"), "path\\\\to\\\\file");
        assert_eq!(escape_json("\\"), "\\\\");
    }

    #[test]
    fn test_newline_escaping() {
        assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_carriage_return_escaping() {
        assert_eq!(escape_json("line1\rline2"), "line1\\rline2");
    }

    #[test]
    fn test_tab_escaping() {
        assert_eq!(escape_json("col1\tcol2"), "col1\\tcol2");
    }

    #[test]
    fn test_control_characters() {
        // Null byte
        assert_eq!(escape_json("\x00"), "\\u0000");
        // Bell
        assert_eq!(escape_json("\x07"), "\\u0007");
    }

    #[test]
    fn test_combined_json_chars() {
        assert_eq!(
            escape_json("He said \"Hello\"\nNew line"),
            "He said \\\"Hello\\\"\\nNew line"
        );
    }

    #[test]
    fn test_empty_string() {
        let result = escape_json("");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "");
    }

    #[test]
    fn test_needs_escaping_json() {
        assert!(needs_escaping("\"quoted\"", OutputFormat::Json));
        assert!(needs_escaping("back\\slash", OutputFormat::Json));
        assert!(needs_escaping("new\nline", OutputFormat::Json));
        assert!(needs_escaping("\x00", OutputFormat::Json)); // Control char
        assert!(!needs_escaping("normal text", OutputFormat::Json));
        assert!(!needs_escaping("", OutputFormat::Json));
    }

    #[test]
    fn test_unicode_preserved() {
        // Unicode characters should pass through unchanged
        assert_eq!(escape_json("Hello 世界"), "Hello 世界");
        assert_eq!(escape_json("café"), "café");
    }
}

// =============================================================================
// Plain Text Tests
// =============================================================================

mod plain_text_tests {
    use super::*;

    #[test]
    fn test_no_escaping_always() {
        assert_eq!(escape_plain("Hello, world!"), "Hello, world!");
        assert_eq!(escape_plain("<script>"), "<script>");
        assert_eq!(escape_plain("*bold*"), "*bold*");
        assert_eq!(escape_plain("100%"), "100%");
    }

    #[test]
    fn test_always_returns_borrowed() {
        let s = String::from("any string");
        let result = escape_plain(&s);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_needs_escaping_plain() {
        // Plain text never needs escaping
        assert!(!needs_escaping("anything", OutputFormat::PlainText));
        assert!(!needs_escaping("", OutputFormat::PlainText));
    }
}

// =============================================================================
// Generic escape() Function Tests
// =============================================================================

mod escape_dispatch_tests {
    use super::*;

    #[test]
    fn test_dispatch_github_actions() {
        assert_eq!(escape("100%", OutputFormat::GitHubActions), "100%25");
        assert_eq!(escape("Error:", OutputFormat::GitHubActions), "Error%3A");
    }

    #[test]
    fn test_dispatch_markdown() {
        assert_eq!(escape("*bold*", OutputFormat::Markdown), "\\*bold\\*");
        assert_eq!(escape("# Heading", OutputFormat::Markdown), "\\# Heading");
    }

    #[test]
    fn test_dispatch_html() {
        assert_eq!(escape("<div>", OutputFormat::Html), "\x26lt;div\x26gt;");
    }

    #[test]
    fn test_dispatch_json() {
        assert_eq!(escape("\"hi\"", OutputFormat::Json), "\\\"hi\\\"");
        assert_eq!(escape("a\nb", OutputFormat::Json), "a\\nb");
    }

    #[test]
    fn test_dispatch_plain_text() {
        assert_eq!(escape("anything", OutputFormat::PlainText), "anything");
    }
}

// =============================================================================
// Zero-Copy Optimization Tests
// =============================================================================

mod zero_copy_tests {
    use super::*;

    #[test]
    fn test_github_zero_copy() {
        let s = "no special chars here";
        let result = escape_github(s);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_markdown_zero_copy() {
        let s = "no special chars here";
        let result = escape_markdown(s);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_html_zero_copy() {
        let s = "no special chars here";
        let result = escape_html(s);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_json_zero_copy() {
        let s = "no special chars here";
        let result = escape_json(s);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_plain_always_zero_copy() {
        let s = "any string at all";
        let result = escape_plain(s);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_escape_function_zero_copy() {
        let s = "no special chars";
        assert!(matches!(escape(s, OutputFormat::GitHubActions), Cow::Borrowed(_)));
        assert!(matches!(escape(s, OutputFormat::Markdown), Cow::Borrowed(_)));
        assert!(matches!(escape(s, OutputFormat::Html), Cow::Borrowed(_)));
        assert!(matches!(escape(s, OutputFormat::Json), Cow::Borrowed(_)));
        assert!(matches!(escape(s, OutputFormat::PlainText), Cow::Borrowed(_)));
    }
}

// =============================================================================
// OutputFormat Enum Tests
// =============================================================================

mod output_format_tests {
    use super::*;

    #[test]
    fn test_debug_impl() {
        assert_eq!(format!("{:?}", OutputFormat::GitHubActions), "GitHubActions");
        assert_eq!(format!("{:?}", OutputFormat::Markdown), "Markdown");
        assert_eq!(format!("{:?}", OutputFormat::PlainText), "PlainText");
        assert_eq!(format!("{:?}", OutputFormat::Html), "Html");
        assert_eq!(format!("{:?}", OutputFormat::Json), "Json");
    }

    #[test]
    fn test_clone_impl() {
        let format = OutputFormat::Html;
        let cloned = format.clone();
        assert_eq!(format, cloned);
    }

    #[test]
    fn test_copy_impl() {
        let format = OutputFormat::Json;
        let copied: OutputFormat = format;
        assert_eq!(format, copied);
    }

    #[test]
    fn test_partial_eq_impl() {
        assert_eq!(OutputFormat::Html, OutputFormat::Html);
        assert_ne!(OutputFormat::Html, OutputFormat::Json);
    }

    #[test]
    fn test_hash_impl() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(OutputFormat::Html);
        assert!(set.contains(&OutputFormat::Html));
        assert!(!set.contains(&OutputFormat::Json));
    }
}

// =============================================================================
// Edge Cases and Boundary Conditions
// =============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_string_all_formats() {
        assert_eq!(escape("", OutputFormat::GitHubActions), "");
        assert_eq!(escape("", OutputFormat::Markdown), "");
        assert_eq!(escape("", OutputFormat::PlainText), "");
        assert_eq!(escape("", OutputFormat::Html), "");
        assert_eq!(escape("", OutputFormat::Json), "");
    }

    #[test]
    fn test_only_special_chars() {
        // GitHub
        assert_eq!(escape_github(":"), "%3A");
        assert_eq!(escape_github("%"), "%25");
        assert_eq!(escape_github("\n"), "%0A");
        assert_eq!(escape_github("\r"), "%0D");

        // Markdown
        assert_eq!(escape_markdown("*"), "\\*");
        assert_eq!(escape_markdown("_"), "\\_");
        assert_eq!(escape_markdown("["), "\\[");
        assert_eq!(escape_markdown("]"), "\\]");
        assert_eq!(escape_markdown("<"), "\\<");
        assert_eq!(escape_markdown(">"), "\\>");
        assert_eq!(escape_markdown("`"), "\\`");
        assert_eq!(escape_markdown("#"), "\\#");

        // HTML - using hex escapes
        assert_eq!(escape_html("<"), "\x26lt;");
        assert_eq!(escape_html(">"), "\x26gt;");
        assert_eq!(escape_html("\x26"), "\x26amp;");
        assert_eq!(escape_html("\""), "\x26quot;");
        assert_eq!(escape_html("\x27"), "\x26#x27;");

        // JSON
        assert_eq!(escape_json("\""), "\\\"");
        assert_eq!(escape_json("\\"), "\\\\");
    }

    #[test]
    fn test_long_string() {
        let long = "a".repeat(10000);
        let result = escape_html(&long);
        assert_eq!(result.len(), 10000);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_long_string_with_special_chars() {
        let long = "a".repeat(5000) + "<" + &"b".repeat(5000);
        let result = escape_html(&long);
        assert!(result.contains("\x26lt;"));
        assert!(matches!(result, Cow::Owned(_)));
    }

    #[test]
    fn test_unicode_handling() {
        // GitHub Actions - unicode passes through
        assert_eq!(escape_github("Hello 世界"), "Hello 世界");

        // Markdown - unicode passes through
        assert_eq!(escape_markdown("Hello 世界"), "Hello 世界");

        // HTML - unicode passes through
        assert_eq!(escape_html("Hello 世界"), "Hello 世界");

        // JSON - unicode passes through
        assert_eq!(escape_json("Hello 世界"), "Hello 世界");
    }

    #[test]
    fn test_mixed_content() {
        let mixed = "Error: 50% complete\n**bold** <script>";
        assert_eq!(
            escape_github(mixed),
            "Error%3A 50%25 complete%0A**bold** <script>"
        );
        assert_eq!(
            escape_markdown(mixed),
            "Error: 50% complete\n\\*\\*bold\\*\\* \\<script\\>"
        );
        // HTML uses hex escapes for entities
        assert!(escape_html(mixed).contains("\x26lt;"));
        assert_eq!(
            escape_json(mixed),
            "Error: 50% complete\\n**bold** <script>"
        );
    }
}
