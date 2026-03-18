//! Comprehensive tests for filter compilation and matching.
//!
//! These tests cover:
//! - Include pattern compilation
//! - Exclude pattern compilation
//! - Glob pattern matching
//! - Empty filters
//! - Invalid patterns

use lintdiff_match::{compile_filters, path_allowed, Filters};
use lintdiff_types::LintdiffConfig;

/// Helper to create filters from include and exclude patterns.
fn filters_from_patterns(include: &[&str], exclude: &[&str]) -> Filters {
    let mut cfg = LintdiffConfig::default();
    cfg.filter.include_paths = include.iter().map(|s| s.to_string()).collect();
    cfg.filter.exclude_paths = exclude.iter().map(|s| s.to_string()).collect();
    compile_filters(&cfg.effective())
}

// Note: EffectiveConfig is not easily constructible directly,
// so we use the LintdiffConfig path via filters_from_patterns helper.

// =============================================================================
// Empty Filter Tests
// =============================================================================

mod empty_filters {
    use super::*;

    #[test]
    fn empty_filters_allow_all_paths() {
        let filters = filters_from_patterns(&[], &[]);

        assert!(path_allowed(&filters, "src/lib.rs"));
        assert!(path_allowed(&filters, "tests/integration.rs"));
        assert!(path_allowed(&filters, "any/path/file.txt"));
        assert!(path_allowed(&filters, "generated/code.rs"));
    }

    #[test]
    fn empty_filters_allow_absolute_paths() {
        let filters = filters_from_patterns(&[], &[]);

        assert!(path_allowed(&filters, "/absolute/path/file.rs"));
        assert!(path_allowed(&filters, "C:/windows/path/file.rs"));
    }

    #[test]
    fn empty_filters_allow_various_extensions() {
        let filters = filters_from_patterns(&[], &[]);

        assert!(path_allowed(&filters, "src/main.rs"));
        assert!(path_allowed(&filters, "src/lib.ts"));
        assert!(path_allowed(&filters, "docs/readme.md"));
        assert!(path_allowed(&filters, "config.toml"));
    }
}

// =============================================================================
// Include Pattern Tests
// =============================================================================

mod include_patterns {
    use super::*;

    #[test]
    fn single_include_pattern_matches() {
        let filters = filters_from_patterns(&["src/**/*.rs"], &[]);

        assert!(path_allowed(&filters, "src/lib.rs"));
        assert!(path_allowed(&filters, "src/module/sub.rs"));
        assert!(path_allowed(&filters, "src/deep/nested/path/mod.rs"));
    }

    #[test]
    fn single_include_pattern_rejects_non_matching() {
        let filters = filters_from_patterns(&["src/**/*.rs"], &[]);

        assert!(!path_allowed(&filters, "tests/lib.rs"));
        assert!(!path_allowed(&filters, "src/lib.ts"));
        assert!(!path_allowed(&filters, "lib.rs"));
    }

    #[test]
    fn multiple_include_patterns() {
        let filters = filters_from_patterns(&["src/**/*.rs", "tests/**/*.rs"], &[]);

        assert!(path_allowed(&filters, "src/lib.rs"));
        assert!(path_allowed(&filters, "tests/integration.rs"));
        assert!(path_allowed(&filters, "src/module/mod.rs"));
        assert!(path_allowed(&filters, "tests/unit/test.rs"));
        assert!(!path_allowed(&filters, "lib.rs"));
        assert!(!path_allowed(&filters, "build/output.rs"));
    }

    #[test]
    fn include_with_wildcard_extension() {
        let filters = filters_from_patterns(&["docs/**/*"], &[]);

        assert!(path_allowed(&filters, "docs/readme.md"));
        assert!(path_allowed(&filters, "docs/api/reference.html"));
        assert!(!path_allowed(&filters, "src/lib.rs"));
    }

    #[test]
    fn include_with_single_wildcard() {
        // Note: In globset, * matches any character including path separators
        // To match a single directory component, you'd need a different approach
        let filters = filters_from_patterns(&["src/*/mod.rs"], &[]);

        assert!(path_allowed(&filters, "src/core/mod.rs"));
        assert!(path_allowed(&filters, "src/utils/mod.rs"));
        // * also matches path separators in globset
        assert!(path_allowed(&filters, "src/deep/nested/mod.rs"));
        assert!(!path_allowed(&filters, "src/lib.rs"));
    }

    #[test]
    fn include_exact_match() {
        let filters = filters_from_patterns(&["src/lib.rs"], &[]);

        assert!(path_allowed(&filters, "src/lib.rs"));
        assert!(!path_allowed(&filters, "src/lib.ts"));
        assert!(!path_allowed(&filters, "src/other.rs"));
    }
}

// =============================================================================
// Exclude Pattern Tests
// =============================================================================

mod exclude_patterns {
    use super::*;

    #[test]
    fn single_exclude_pattern_blocks() {
        let filters = filters_from_patterns(&[], &["src/lib.rs"]);

        assert!(!path_allowed(&filters, "src/lib.rs"));
        assert!(path_allowed(&filters, "src/main.rs"));
        assert!(path_allowed(&filters, "tests/lib.rs"));
    }

    #[test]
    fn exclude_with_double_star() {
        let filters = filters_from_patterns(&[], &["**/generated/**"]);

        assert!(!path_allowed(&filters, "generated/mod.rs"));
        assert!(!path_allowed(&filters, "src/generated/api.rs"));
        assert!(!path_allowed(&filters, "deep/nested/generated/file.rs"));
        assert!(path_allowed(&filters, "src/lib.rs"));
        assert!(path_allowed(&filters, "src/handwritten.rs"));
    }

    #[test]
    fn multiple_exclude_patterns() {
        let filters = filters_from_patterns(&[], &["*.generated.rs", "target/**", "build/**"]);

        assert!(!path_allowed(&filters, "src/api.generated.rs"));
        assert!(!path_allowed(&filters, "target/debug/main"));
        assert!(!path_allowed(&filters, "build/release/output"));
        assert!(path_allowed(&filters, "src/lib.rs"));
        assert!(path_allowed(&filters, "tests/integration.rs"));
    }

    #[test]
    fn exclude_with_extension_wildcard() {
        let filters = filters_from_patterns(&[], &["vendor/**/*"]);

        assert!(!path_allowed(&filters, "vendor/lib.rs"));
        assert!(!path_allowed(&filters, "vendor/sub/lib.c"));
        assert!(path_allowed(&filters, "src/vendor.rs"));
    }

    #[test]
    fn exclude_with_single_wildcard() {
        let filters = filters_from_patterns(&[], &["*.min.js"]);

        assert!(!path_allowed(&filters, "bundle.min.js"));
        assert!(!path_allowed(&filters, "src/app.min.js"));
        assert!(path_allowed(&filters, "src/app.js"));
        assert!(path_allowed(&filters, "src/app.ts"));
    }
}

// =============================================================================
// Combined Include/Exclude Tests
// =============================================================================

mod combined_patterns {
    use super::*;

    #[test]
    fn exclude_takes_precedence_over_include() {
        let filters = filters_from_patterns(&["src/**"], &["src/lib.rs"]);

        // Include would allow, but exclude blocks
        assert!(!path_allowed(&filters, "src/lib.rs"));
        // Include allows and exclude doesn't block
        assert!(path_allowed(&filters, "src/main.rs"));
        assert!(path_allowed(&filters, "src/module/mod.rs"));
    }

    #[test]
    fn include_limits_and_exclude_further_restricts() {
        let filters =
            filters_from_patterns(&["src/**/*.rs"], &["src/generated/**", "src/test_*.rs"]);

        // In include, not in exclude
        assert!(path_allowed(&filters, "src/lib.rs"));
        assert!(path_allowed(&filters, "src/module/mod.rs"));

        // In include, but also in exclude
        assert!(!path_allowed(&filters, "src/generated/api.rs"));
        assert!(!path_allowed(&filters, "src/test_utils.rs"));

        // Not in include
        assert!(!path_allowed(&filters, "tests/integration.rs"));
    }

    #[test]
    fn complex_combined_scenario() {
        // Real-world scenario: include all source files but exclude generated and vendored
        let filters = filters_from_patterns(
            &["src/**/*.rs", "lib/**/*.rs"],
            &["**/generated/**", "**/vendor/**", "**/*.generated.rs"],
        );

        // Allowed: source files
        assert!(path_allowed(&filters, "src/main.rs"));
        assert!(path_allowed(&filters, "lib/core/mod.rs"));

        // Blocked: generated files
        assert!(!path_allowed(&filters, "src/generated/proto.rs"));
        assert!(!path_allowed(&filters, "src/api.generated.rs"));

        // Blocked: vendor files
        assert!(!path_allowed(&filters, "src/vendor/third_party.rs"));

        // Blocked: not in include
        assert!(!path_allowed(&filters, "tests/integration.rs"));
    }
}

// =============================================================================
// Glob Pattern Edge Cases
// =============================================================================

mod glob_edge_cases {
    use super::*;

    #[test]
    fn pattern_with_special_characters() {
        let filters = filters_from_patterns(&["src/[test]/**"], &[]);

        // Glob patterns with brackets have special meaning
        // This tests that the glob library handles them correctly
        assert!(path_allowed(&filters, "src/t/file.rs")); // 't' matches [test]
    }

    #[test]
    fn pattern_with_question_mark() {
        let filters = filters_from_patterns(&["src/test_?.rs"], &[]);

        assert!(path_allowed(&filters, "src/test_a.rs"));
        assert!(path_allowed(&filters, "src/test_1.rs"));
        assert!(!path_allowed(&filters, "src/test_ab.rs"));
        assert!(!path_allowed(&filters, "src/test.rs"));
    }

    #[test]
    fn double_star_at_start() {
        let filters = filters_from_patterns(&["**/test.rs"], &[]);

        assert!(path_allowed(&filters, "test.rs"));
        assert!(path_allowed(&filters, "src/test.rs"));
        assert!(path_allowed(&filters, "deep/nested/path/test.rs"));
    }

    #[test]
    fn double_star_at_end() {
        let filters = filters_from_patterns(&["src/**"], &[]);

        assert!(path_allowed(&filters, "src/lib.rs"));
        assert!(path_allowed(&filters, "src/deep/nested/mod.rs"));
        assert!(path_allowed(&filters, "src/"));
    }

    #[test]
    fn double_star_in_middle() {
        let filters = filters_from_patterns(&["src/**/mod.rs"], &[]);

        assert!(path_allowed(&filters, "src/mod.rs"));
        assert!(path_allowed(&filters, "src/core/mod.rs"));
        assert!(path_allowed(&filters, "src/deep/nested/mod.rs"));
        assert!(!path_allowed(&filters, "src/lib.rs"));
    }

    #[test]
    fn empty_string_pattern() {
        // Empty patterns should be handled gracefully
        let _filters = filters_from_patterns(&[""], &[]);

        // Empty pattern matches nothing meaningful
        // Behavior depends on glob library implementation
    }
}

// =============================================================================
// Invalid Pattern Tests
// =============================================================================

mod invalid_patterns {
    use super::*;

    #[test]
    fn invalid_glob_pattern_compiles_but_matches_nothing() {
        // Invalid patterns are silently skipped during compilation
        // When all include patterns are invalid, the resulting empty GlobSet matches nothing
        // This means no paths are allowed (which is reasonable behavior)
        let filters = filters_from_patterns(&["[invalid"], &[]);

        // Empty include GlobSet matches nothing, so no paths are allowed
        assert!(!path_allowed(&filters, "src/lib.rs"));
        assert!(!path_allowed(&filters, "any/path.rs"));
    }

    #[test]
    fn unclosed_bracket_pattern_compiles_but_matches_nothing() {
        // Invalid patterns are skipped, resulting in empty include GlobSet
        let filters = filters_from_patterns(&["src/[unclosed/**"], &[]);

        // Empty include GlobSet matches nothing
        assert!(!path_allowed(&filters, "src/lib.rs"));
    }

    #[test]
    fn mixed_valid_and_invalid_patterns() {
        // Mix of valid and invalid patterns - valid ones are still added
        let filters = filters_from_patterns(
            &["src/**/*.rs", "[invalid", "tests/**/*.rs"],
            &["**/generated/**", "[also-invalid"],
        );

        // Valid patterns should still work
        assert!(path_allowed(&filters, "src/lib.rs"));
        assert!(path_allowed(&filters, "tests/integration.rs"));
        // Invalid exclude pattern is skipped, but valid one still works
        assert!(!path_allowed(&filters, "src/generated/api.rs"));
    }

    #[test]
    fn invalid_exclude_with_no_include_allows_all() {
        // When only exclude patterns are invalid, include is None (allow all)
        let filters = filters_from_patterns(&[], &["[invalid"]);

        // No valid exclude patterns, so all paths are allowed
        assert!(path_allowed(&filters, "src/lib.rs"));
        assert!(path_allowed(&filters, "any/path.rs"));
    }
}

// =============================================================================
// Path Format Tests
// =============================================================================

mod path_formats {
    use super::*;

    #[test]
    fn windows_style_paths() {
        let filters = filters_from_patterns(&["src/**/*.rs"], &[]);

        // Windows-style paths with backslashes should be normalized
        // Note: globset typically expects forward slashes
        assert!(path_allowed(&filters, "src\\lib.rs"));
        assert!(path_allowed(&filters, "src\\module\\mod.rs"));
    }

    #[test]
    fn absolute_unix_paths() {
        let filters = filters_from_patterns(&["src/**/*.rs"], &[]);

        // Absolute paths typically won't match relative patterns
        assert!(!path_allowed(&filters, "/home/user/project/src/lib.rs"));
    }

    #[test]
    fn absolute_windows_paths() {
        let filters = filters_from_patterns(&["src/**/*.rs"], &[]);

        assert!(!path_allowed(&filters, "C:/Users/user/project/src/lib.rs"));
    }

    #[test]
    fn paths_with_dots() {
        let filters = filters_from_patterns(&["src/**/*.rs"], &[]);

        // Paths with . or .. are typically not resolved by the filter
        assert!(!path_allowed(&filters, "./src/lib.rs"));
        assert!(!path_allowed(&filters, "../project/src/lib.rs"));
    }
}

// =============================================================================
// Filter Structure Tests
// =============================================================================

mod filter_structure {
    use super::*;

    #[test]
    fn filters_debug_impl() {
        let filters = filters_from_patterns(&["src/**/*.rs"], &["target/**"]);

        // Filters should implement Debug
        let debug_str = format!("{:?}", filters);
        assert!(debug_str.contains("Filters"));
    }

    #[test]
    fn filters_clone_impl() {
        let filters = filters_from_patterns(&["src/**/*.rs"], &["target/**"]);

        // Filters should implement Clone
        let cloned = filters.clone();
        assert!(path_allowed(&cloned, "src/lib.rs"));
        assert!(!path_allowed(&cloned, "target/debug/main"));
    }
}

// =============================================================================
// Performance-related Tests
// =============================================================================

mod performance {
    use super::*;

    #[test]
    fn many_patterns_performance() {
        // Create filters with many patterns to test compilation doesn't timeout
        let include: Vec<String> = (0..100)
            .map(|i| format!("src/module{}/**/*.rs", i))
            .collect();
        let exclude: Vec<String> = (0..50).map(|i| format!("**/generated{}/**", i)).collect();

        let mut cfg = LintdiffConfig::default();
        cfg.filter.include_paths = include;
        cfg.filter.exclude_paths = exclude;

        let filters = compile_filters(&cfg.effective());

        // Should still work correctly
        assert!(path_allowed(&filters, "src/module0/lib.rs"));
        assert!(path_allowed(&filters, "src/module99/deep/mod.rs"));
        assert!(!path_allowed(&filters, "src/module0/generated0/api.rs"));
    }

    #[test]
    fn long_path_handling() {
        let filters = filters_from_patterns(&["src/**/*.rs"], &[]);

        // Very long path
        let long_path = format!("src/{}", "nested/".repeat(100).trim_end_matches('/'));
        let long_path = format!("{}/mod.rs", long_path);

        // Should handle without issues
        assert!(path_allowed(&filters, &long_path));
    }
}
