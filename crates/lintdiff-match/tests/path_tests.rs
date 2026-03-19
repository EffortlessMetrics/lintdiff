//! Comprehensive tests for path relativization and normalization.
//!
//! These tests cover:
//! - Path relativization
//! - Absolute to relative conversion
//! - Windows/Unix path handling
//! - Symlink resolution (if applicable)
//! - Path canonicalization

use lintdiff_match::relativize_span_path;
use lintdiff_types::NormPath;

// =============================================================================
// Basic Relativization Tests
// =============================================================================

mod basic_relativization {
    use super::*;

    #[test]
    fn relative_path_passes_through_unchanged() {
        let result = relativize_span_path(&NormPath::new("src/lib.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    #[test]
    fn simple_relative_path() {
        let result = relativize_span_path(&NormPath::new("lib.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "lib.rs");
    }

    #[test]
    fn nested_relative_path() {
        let result = relativize_span_path(&NormPath::new("src/core/utils/mod.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "src/core/utils/mod.rs");
    }

    #[test]
    fn path_with_extension() {
        let result = relativize_span_path(&NormPath::new("docs/api/reference.html"), None, true);
        assert_eq!(result.unwrap().as_str(), "docs/api/reference.html");
    }
}

// =============================================================================
// Absolute Path Conversion Tests
// =============================================================================

mod absolute_to_relative {
    use super::*;

    #[test]
    fn unix_absolute_path_with_repo_root() {
        let result = relativize_span_path(
            &NormPath::new("/home/user/project/src/lib.rs"),
            Some(&NormPath::new("/home/user/project")),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    #[test]
    fn unix_absolute_path_deep_nesting() {
        let result = relativize_span_path(
            &NormPath::new("/repo/src/deep/nested/module/mod.rs"),
            Some(&NormPath::new("/repo")),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "src/deep/nested/module/mod.rs");
    }

    #[test]
    fn repo_root_with_trailing_slash() {
        let result = relativize_span_path(
            &NormPath::new("/repo/src/lib.rs"),
            Some(&NormPath::new("/repo/")),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    #[test]
    fn repo_root_with_multiple_trailing_slashes() {
        // Note: trim_end_matches('/') removes ALL trailing slashes
        // So "/repo//" becomes "/repo"
        let result = relativize_span_path(
            &NormPath::new("/repo/src/lib.rs"),
            Some(&NormPath::new("/repo//")),
            true,
        );
        // After trimming all slashes, root becomes "/repo" which matches
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    #[test]
    fn exact_repo_root_path() {
        // Path that is exactly the repo root
        let result =
            relativize_span_path(&NormPath::new("/repo"), Some(&NormPath::new("/repo")), true);
        // After stripping, we get empty string, which returns None when workspace_only is true
        assert!(result.is_none());
    }

    #[test]
    fn path_starting_with_repo_name_as_prefix() {
        // Note: strip_prefix does simple string prefix matching
        // "/repo" IS a prefix of "/repo-other/src/lib.rs"
        let result = relativize_span_path(
            &NormPath::new("/repo-other/src/lib.rs"),
            Some(&NormPath::new("/repo")),
            true,
        );
        // strip_prefix("/repo") on "/repo-other/src/lib.rs" gives "-other/src/lib.rs"
        // After trim_start_matches('/'), we get "-other/src/lib.rs" (starts with '-')
        // This is returned as-is since it's not empty
        assert!(result.is_some());
    }
}

// =============================================================================
// Workspace-only Mode Tests
// =============================================================================

mod workspace_only_mode {
    use super::*;

    #[test]
    fn path_outside_root_returns_none_when_workspace_only() {
        let result = relativize_span_path(
            &NormPath::new("/other/src/lib.rs"),
            Some(&NormPath::new("/repo")),
            true,
        );
        assert!(result.is_none());
    }

    #[test]
    fn path_outside_root_returns_path_when_not_workspace_only() {
        let result = relativize_span_path(
            &NormPath::new("/other/src/lib.rs"),
            Some(&NormPath::new("/repo")),
            false,
        );
        assert!(result.is_some());
        // Path should be normalized but not relativized
        assert_eq!(result.unwrap().as_str(), "/other/src/lib.rs");
    }

    #[test]
    fn absolute_path_no_root_workspace_only_returns_none() {
        let result = relativize_span_path(&NormPath::new("/repo/src/lib.rs"), None, true);
        assert!(result.is_none());
    }

    #[test]
    fn absolute_path_no_root_not_workspace_only_returns_path() {
        let result = relativize_span_path(&NormPath::new("/repo/src/lib.rs"), None, false);
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), "/repo/src/lib.rs");
    }

    #[test]
    fn relative_path_ignores_workspace_only_flag() {
        // Relative paths should pass through regardless of workspace_only
        let result_with = relativize_span_path(
            &NormPath::new("src/lib.rs"),
            Some(&NormPath::new("/repo")),
            true,
        );
        let result_without = relativize_span_path(
            &NormPath::new("src/lib.rs"),
            Some(&NormPath::new("/repo")),
            false,
        );

        assert_eq!(result_with.unwrap().as_str(), "src/lib.rs");
        assert_eq!(result_without.unwrap().as_str(), "src/lib.rs");
    }
}

// =============================================================================
// Windows Path Handling Tests
// =============================================================================

mod windows_paths {
    use super::*;

    #[test]
    fn windows_relative_path_backslashes_normalized() {
        let result = relativize_span_path(&NormPath::new("src\\lib.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    #[test]
    fn windows_nested_path_backslashes_normalized() {
        let result = relativize_span_path(&NormPath::new("src\\core\\utils\\mod.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "src/core/utils/mod.rs");
    }

    #[test]
    fn windows_absolute_path_detected() {
        // Windows absolute paths should be detected
        let result = relativize_span_path(
            &NormPath::new("C:/Users/user/project/src/lib.rs"),
            None,
            true,
        );
        // With workspace_only=true and no repo_root, absolute path returns None
        assert!(result.is_none());
    }

    #[test]
    fn windows_absolute_path_with_repo_root() {
        let result = relativize_span_path(
            &NormPath::new("C:/project/src/lib.rs"),
            Some(&NormPath::new("C:/project")),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    #[test]
    fn windows_absolute_path_backslashes_in_repo_root() {
        let result = relativize_span_path(
            &NormPath::new("C:\\project\\src\\lib.rs"),
            Some(&NormPath::new("C:\\project")),
            true,
        );
        // Both should be normalized to forward slashes
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    #[test]
    fn mixed_slashes_and_backslashes() {
        let result = relativize_span_path(&NormPath::new("src\\nested/path\\file.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "src/nested/path/file.rs");
    }

    #[test]
    fn windows_drive_letter_various_cases() {
        // Lowercase drive letter
        assert!(relativize_span_path(&NormPath::new("c:/path/file.rs"), None, true,).is_none()); // None because absolute + workspace_only + no root

        // Uppercase drive letter
        assert!(relativize_span_path(&NormPath::new("D:/path/file.rs"), None, true,).is_none());
    }
}

// =============================================================================
// Unix Path Handling Tests
// =============================================================================

mod unix_paths {
    use super::*;

    #[test]
    fn unix_absolute_path_root() {
        let result = relativize_span_path(
            &NormPath::new("/src/lib.rs"),
            Some(&NormPath::new("/")),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    #[test]
    fn unix_absolute_path_deep_nesting() {
        let result = relativize_span_path(
            &NormPath::new("/home/user/projects/myproject/src/lib.rs"),
            Some(&NormPath::new("/home/user/projects/myproject")),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    #[test]
    fn unix_path_with_spaces() {
        let result = relativize_span_path(
            &NormPath::new("/home/user/my project/src/lib.rs"),
            Some(&NormPath::new("/home/user/my project")),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    #[test]
    fn unix_path_with_special_characters() {
        let result = relativize_span_path(
            &NormPath::new("/home/user/project-test/src/lib.rs"),
            Some(&NormPath::new("/home/user/project-test")),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }
}

// =============================================================================
// Edge Cases
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn empty_path() {
        let result = relativize_span_path(&NormPath::new(""), None, true);
        // Empty path is not absolute, so it passes through
        assert_eq!(result.unwrap().as_str(), "");
    }

    #[test]
    fn single_component_path() {
        let result = relativize_span_path(&NormPath::new("lib.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "lib.rs");
    }

    #[test]
    fn path_ending_with_slash() {
        let result = relativize_span_path(&NormPath::new("src/directory/"), None, true);
        assert_eq!(result.unwrap().as_str(), "src/directory/");
    }

    #[test]
    fn path_with_double_slashes() {
        // Note: NormPath normalizes double slashes to single slashes
        let result = relativize_span_path(&NormPath::new("src//lib.rs"), None, true);
        // NormPath normalizes the path
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    #[test]
    fn path_with_dot_components() {
        // Note: NormPath normalizes away leading ./
        let result = relativize_span_path(&NormPath::new("./src/lib.rs"), None, true);
        // NormPath normalizes the ./ prefix away
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    #[test]
    fn path_with_dot_dot_components() {
        // Paths with .. are not resolved by our implementation
        let result = relativize_span_path(&NormPath::new("../src/lib.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "../src/lib.rs");
    }

    #[test]
    fn very_long_path() {
        let long_path = format!("{}/lib.rs", "nested".repeat(100));
        let result = relativize_span_path(&NormPath::new(&long_path), None, true);
        assert_eq!(result.unwrap().as_str(), long_path);
    }

    #[test]
    fn unicode_path() {
        let result = relativize_span_path(&NormPath::new("src/日本語/файл.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "src/日本語/файл.rs");
    }

    #[test]
    fn path_with_multiple_extensions() {
        let result = relativize_span_path(&NormPath::new("src/lib.spec.ts"), None, true);
        assert_eq!(result.unwrap().as_str(), "src/lib.spec.ts");
    }
}

// =============================================================================
// Repo Root Edge Cases
// =============================================================================

mod repo_root_edge_cases {
    use super::*;

    #[test]
    fn repo_root_is_prefix_of_other_repo() {
        // Note: strip_prefix does simple string prefix matching
        // /repo IS a string prefix of /repo-other/src/lib.rs
        // This results in "-other/src/lib.rs" after stripping
        let result = relativize_span_path(
            &NormPath::new("/repo-other/src/lib.rs"),
            Some(&NormPath::new("/repo")),
            true,
        );
        // strip_prefix gives "-other/src/lib.rs", which is returned as-is
        assert!(result.is_some());
    }

    #[test]
    fn repo_root_exactly_matches_path() {
        let result =
            relativize_span_path(&NormPath::new("/repo"), Some(&NormPath::new("/repo")), true);
        // Empty result after stripping returns None when workspace_only
        assert!(result.is_none());
    }

    #[test]
    fn repo_root_exactly_matches_path_not_workspace_only() {
        let result = relativize_span_path(
            &NormPath::new("/repo"),
            Some(&NormPath::new("/repo")),
            false,
        );
        // Returns the original path when workspace_only is false
        assert!(result.is_some());
    }

    #[test]
    fn file_directly_in_repo_root() {
        let result = relativize_span_path(
            &NormPath::new("/repo/Cargo.toml"),
            Some(&NormPath::new("/repo")),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "Cargo.toml");
    }

    #[test]
    fn case_sensitivity() {
        // Unix paths are typically case-sensitive
        let result = relativize_span_path(
            &NormPath::new("/Repo/src/lib.rs"),
            Some(&NormPath::new("/repo")),
            true,
        );
        // Should not match due to case difference
        assert!(result.is_none());
    }
}

// =============================================================================
// Normalization Tests
// =============================================================================

mod normalization {
    use super::*;

    #[test]
    fn backslashes_converted_to_forward() {
        let result = relativize_span_path(&NormPath::new("src\\deep\\nested\\mod.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "src/deep/nested/mod.rs");
    }

    #[test]
    fn already_normalized_path_unchanged() {
        let result = relativize_span_path(&NormPath::new("src/deep/nested/mod.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "src/deep/nested/mod.rs");
    }

    #[test]
    fn absolute_path_normalized_after_relativization() {
        let result = relativize_span_path(
            &NormPath::new("C:\\project\\src\\lib.rs"),
            Some(&NormPath::new("C:\\project")),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }
}

// =============================================================================
// Integration-like Tests
// =============================================================================

mod integration {
    use super::*;

    #[test]
    fn typical_rust_project_scenario() {
        let repo_root = NormPath::new("/home/user/myproject");

        // Source file
        let result = relativize_span_path(
            &NormPath::new("/home/user/myproject/src/main.rs"),
            Some(&repo_root),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "src/main.rs");

        // Test file
        let result = relativize_span_path(
            &NormPath::new("/home/user/myproject/tests/integration.rs"),
            Some(&repo_root),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "tests/integration.rs");

        // Config file
        let result = relativize_span_path(
            &NormPath::new("/home/user/myproject/Cargo.toml"),
            Some(&repo_root),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "Cargo.toml");
    }

    #[test]
    fn monorepo_scenario() {
        let repo_root = NormPath::new("/workspace/monorepo");

        // Package source
        let result = relativize_span_path(
            &NormPath::new("/workspace/monorepo/packages/core/src/lib.rs"),
            Some(&repo_root),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "packages/core/src/lib.rs");

        // Shared utilities
        let result = relativize_span_path(
            &NormPath::new("/workspace/monorepo/shared/utils/src/lib.rs"),
            Some(&repo_root),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "shared/utils/src/lib.rs");
    }

    #[test]
    fn external_dependency_path_filtered_out() {
        // Paths from external dependencies should be filtered when workspace_only
        let result = relativize_span_path(
            &NormPath::new("/home/user/.cargo/registry/src/some-crate/src/lib.rs"),
            Some(&NormPath::new("/home/user/myproject")),
            true,
        );
        assert!(result.is_none());
    }
}
