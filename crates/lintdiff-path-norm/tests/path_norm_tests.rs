//! Comprehensive tests for lintdiff-path-norm.
//!
//! This test suite covers all public API functions and types.

use lintdiff_path_norm::{
    extension, file_name, join, normalize, normalize_owned, parent, paths_cmp, paths_eq,
    NormalizeConfig, NormalizedPath,
};

// =============================================================================
// normalize() basic cases (10 tests)
// =============================================================================

#[test]
fn test_normalize_simple_path() {
    assert_eq!(normalize("src/lib.rs"), "src/lib.rs");
}

#[test]
fn test_normalize_empty_path() {
    assert_eq!(normalize(""), "");
}

#[test]
fn test_normalize_single_component() {
    assert_eq!(normalize("lib.rs"), "lib.rs");
}

#[test]
fn test_normalize_deep_path() {
    assert_eq!(
        normalize("src/core/utils/helpers.rs"),
        "src/core/utils/helpers.rs"
    );
}

#[test]
fn test_normalize_trailing_slash() {
    assert_eq!(normalize("src/lib.rs/"), "src/lib.rs");
}

#[test]
fn test_normalize_trailing_slash_directory() {
    assert_eq!(normalize("src/foo/"), "src/foo");
}

#[test]
fn test_normalize_multiple_trailing_slashes() {
    // Duplicate slashes are collapsed first, then trailing slash is removed
    assert_eq!(normalize("src/foo//"), "src/foo");
}

#[test]
fn test_normalize_root_path() {
    assert_eq!(normalize("/"), "/");
}

#[test]
fn test_normalize_dot_slash() {
    assert_eq!(normalize("./src/lib.rs"), "src/lib.rs");
}

#[test]
fn test_normalize_dot_slash_only() {
    assert_eq!(normalize("./lib.rs"), "lib.rs");
}

// =============================================================================
// normalize() with diff prefixes (6 tests)
// =============================================================================

#[test]
fn test_normalize_diff_prefix_a() {
    assert_eq!(normalize("a/src/lib.rs"), "src/lib.rs");
}

#[test]
fn test_normalize_diff_prefix_b() {
    assert_eq!(normalize("b/src/lib.rs"), "src/lib.rs");
}

#[test]
fn test_normalize_diff_prefix_a_root() {
    assert_eq!(normalize("a/lib.rs"), "lib.rs");
}

#[test]
fn test_normalize_diff_prefix_b_root() {
    assert_eq!(normalize("b/lib.rs"), "lib.rs");
}

#[test]
fn test_normalize_diff_prefix_with_dot_slash() {
    // After stripping a/ prefix, we get ./src/lib.rs, then dot-slash is stripped
    assert_eq!(normalize("a/./src/lib.rs"), "src/lib.rs");
}

#[test]
fn test_normalize_diff_prefix_deep_path() {
    assert_eq!(
        normalize("a/src/core/utils/helpers.rs"),
        "src/core/utils/helpers.rs"
    );
}

// =============================================================================
// normalize() with Windows paths (6 tests)
// =============================================================================

#[test]
fn test_normalize_windows_single_backslash() {
    assert_eq!(normalize("src\\lib.rs"), "src/lib.rs");
}

#[test]
fn test_normalize_windows_multiple_backslashes() {
    assert_eq!(normalize("src\\foo\\bar.rs"), "src/foo/bar.rs");
}

#[test]
fn test_normalize_windows_deep_path() {
    assert_eq!(
        normalize("src\\core\\utils\\helpers.rs"),
        "src/core/utils/helpers.rs"
    );
}

#[test]
fn test_normalize_windows_root() {
    assert_eq!(normalize("\\"), "/");
}

#[test]
fn test_normalize_windows_trailing_backslash() {
    assert_eq!(normalize("src\\foo\\"), "src/foo");
}

#[test]
fn test_normalize_windows_mixed_slashes() {
    assert_eq!(normalize("src\\foo/bar.rs"), "src/foo/bar.rs");
}

#[test]
fn test_normalize_windows_with_diff_prefix() {
    // Backslash converted to forward slash first, then a/ prefix is stripped
    assert_eq!(normalize("a\\src\\lib.rs"), "src/lib.rs");
}

// =============================================================================
// NormalizeConfig options (8 tests)
// =============================================================================

#[test]
fn test_config_default_all_enabled() {
    let config = NormalizeConfig::default();
    assert!(config.slash_normalize);
    assert!(config.strip_dot_slash);
    assert!(config.strip_diff_prefix);
    assert!(config.collapse_slashes);
    assert!(config.strip_trailing_slash);
}

#[test]
fn test_config_new() {
    let config = NormalizeConfig::new();
    assert!(config.slash_normalize);
    assert!(config.strip_dot_slash);
    assert!(config.strip_diff_prefix);
    assert!(config.collapse_slashes);
    assert!(config.strip_trailing_slash);
}

#[test]
fn test_config_keep_backslashes() {
    let config = NormalizeConfig::new().keep_backslashes();
    assert!(!config.slash_normalize);
    assert_eq!(config.normalize("src\\lib.rs"), "src\\lib.rs");
}

#[test]
fn test_config_keep_dot_slash() {
    let config = NormalizeConfig::new().keep_dot_slash();
    assert!(!config.strip_dot_slash);
    assert_eq!(config.normalize("./src/lib.rs"), "./src/lib.rs");
}

#[test]
fn test_config_keep_diff_prefix() {
    let config = NormalizeConfig::new().keep_diff_prefix();
    assert!(!config.strip_diff_prefix);
    assert_eq!(config.normalize("a/src/lib.rs"), "a/src/lib.rs");
}

#[test]
fn test_config_chained_options() {
    let config = NormalizeConfig::new()
        .keep_backslashes()
        .keep_dot_slash()
        .keep_diff_prefix();
    assert!(!config.slash_normalize);
    assert!(!config.strip_dot_slash);
    assert!(!config.strip_diff_prefix);
}

#[test]
fn test_config_normalize_with_collapse_disabled() {
    let config = NormalizeConfig {
        collapse_slashes: false,
        ..NormalizeConfig::new()
    };
    assert_eq!(config.normalize("src//lib.rs"), "src//lib.rs");
}

#[test]
fn test_config_normalize_with_trailing_slash_disabled() {
    let config = NormalizeConfig {
        strip_trailing_slash: false,
        ..NormalizeConfig::new()
    };
    assert_eq!(config.normalize("src/lib.rs/"), "src/lib.rs/");
}

// =============================================================================
// paths_eq and paths_cmp (8 tests)
// =============================================================================

#[test]
fn test_paths_eq_identical() {
    assert!(paths_eq("src/lib.rs", "src/lib.rs"));
}

#[test]
fn test_paths_eq_different_slashes() {
    assert!(paths_eq("src/lib.rs", "src\\lib.rs"));
}

#[test]
fn test_paths_eq_dot_slash() {
    assert!(paths_eq("./src/lib.rs", "src/lib.rs"));
}

#[test]
fn test_paths_eq_diff_prefix() {
    assert!(paths_eq("a/src/lib.rs", "b/src/lib.rs"));
}

#[test]
fn test_paths_eq_different_paths() {
    assert!(!paths_eq("src/lib.rs", "src/main.rs"));
}

#[test]
fn test_paths_cmp_equal() {
    assert_eq!(paths_cmp("a/lib.rs", "b/lib.rs"), std::cmp::Ordering::Equal);
}

#[test]
fn test_paths_cmp_less() {
    assert_eq!(paths_cmp("src/a.rs", "src/b.rs"), std::cmp::Ordering::Less);
}

#[test]
fn test_paths_cmp_greater() {
    assert_eq!(paths_cmp("src/b.rs", "src/a.rs"), std::cmp::Ordering::Greater);
}

// =============================================================================
// extension, file_name, parent (10 tests)
// =============================================================================

#[test]
fn test_extension_simple() {
    assert_eq!(extension("src/lib.rs"), Some("rs"));
}

#[test]
fn test_extension_no_extension() {
    assert_eq!(extension("src/lib"), None);
}

#[test]
fn test_extension_multiple_dots() {
    assert_eq!(extension("src/lib.test.rs"), Some("rs"));
}

#[test]
fn test_extension_hidden_file() {
    assert_eq!(extension(".gitignore"), None);
}

#[test]
fn test_extension_empty_path() {
    assert_eq!(extension(""), None);
}

#[test]
fn test_file_name_simple() {
    assert_eq!(file_name("src/lib.rs"), Some("lib.rs"));
}

#[test]
fn test_file_name_no_directory() {
    assert_eq!(file_name("lib.rs"), Some("lib.rs"));
}

#[test]
fn test_file_name_trailing_slash() {
    assert_eq!(file_name("src/"), None);
}

#[test]
fn test_parent_simple() {
    assert_eq!(parent("src/lib.rs"), Some("src"));
}

#[test]
fn test_parent_deep() {
    assert_eq!(parent("src/core/utils/helpers.rs"), Some("src/core/utils"));
}

#[test]
fn test_parent_no_parent() {
    assert_eq!(parent("lib.rs"), None);
}

#[test]
fn test_parent_trailing_slash() {
    assert_eq!(parent("src/foo/"), Some("src"));
}

// =============================================================================
// join function (5 tests)
// =============================================================================

#[test]
fn test_join_two_components() {
    assert_eq!(join(&["src", "lib.rs"]), "src/lib.rs");
}

#[test]
fn test_join_three_components() {
    assert_eq!(join(&["src", "foo", "bar.rs"]), "src/foo/bar.rs");
}

#[test]
fn test_join_single_component() {
    assert_eq!(join(&["lib.rs"]), "lib.rs");
}

#[test]
fn test_join_empty() {
    assert_eq!(join(&[]), "");
}

#[test]
fn test_join_multiple_components() {
    assert_eq!(
        join(&["a", "b", "c", "d", "e.rs"]),
        "a/b/c/d/e.rs"
    );
}

// =============================================================================
// NormalizedPath wrapper (7 tests)
// =============================================================================

#[test]
fn test_normalized_path_new() {
    let path = NormalizedPath::new("src\\lib.rs");
    assert_eq!(path.as_str(), "src/lib.rs");
}

#[test]
fn test_normalized_path_from_str() {
    let path: NormalizedPath = "src/lib.rs".into();
    assert_eq!(path.as_str(), "src/lib.rs");
}

#[test]
fn test_normalized_path_from_string() {
    let path: NormalizedPath = String::from("src/lib.rs").into();
    assert_eq!(path.as_str(), "src/lib.rs");
}

#[test]
fn test_normalized_path_extension() {
    let path = NormalizedPath::new("src/lib.rs");
    assert_eq!(path.extension(), Some("rs"));
}

#[test]
fn test_normalized_path_file_name() {
    let path = NormalizedPath::new("src/lib.rs");
    assert_eq!(path.file_name(), Some("lib.rs"));
}

#[test]
fn test_normalized_path_parent() {
    let path = NormalizedPath::new("src/lib.rs");
    assert_eq!(path.parent(), Some("src"));
}

#[test]
fn test_normalized_path_starts_with() {
    let path = NormalizedPath::new("src/core/lib.rs");
    assert!(path.starts_with("src"));
    assert!(path.starts_with("src/core"));
    assert!(!path.starts_with("lib"));
}

#[test]
fn test_normalized_path_ends_with() {
    let path = NormalizedPath::new("src/core/lib.rs");
    assert!(path.ends_with("lib.rs"));
    assert!(path.ends_with("core/lib.rs"));
    assert!(!path.ends_with("main.rs"));
}

#[test]
fn test_normalized_path_display() {
    let path = NormalizedPath::new("src/lib.rs");
    assert_eq!(format!("{}", path), "src/lib.rs");
}

#[test]
fn test_normalized_path_as_ref_str() {
    let path = NormalizedPath::new("src/lib.rs");
    let s: &str = path.as_ref();
    assert_eq!(s, "src/lib.rs");
}

#[test]
fn test_normalized_path_as_ref_path() {
    let path = NormalizedPath::new("src/lib.rs");
    let p: &std::path::Path = path.as_ref();
    assert_eq!(p.to_str(), Some("src/lib.rs"));
}

#[test]
fn test_normalized_path_equality() {
    let path1 = NormalizedPath::new("src\\lib.rs");
    let path2 = NormalizedPath::new("src/lib.rs");
    assert_eq!(path1, path2);
}

#[test]
fn test_normalized_path_ordering() {
    let path1 = NormalizedPath::new("src/a.rs");
    let path2 = NormalizedPath::new("src/b.rs");
    assert!(path1 < path2);
}

#[test]
fn test_normalized_path_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(NormalizedPath::new("src\\lib.rs"));
    assert!(set.contains(&NormalizedPath::new("src/lib.rs")));
}

// =============================================================================
// Additional edge case tests
// =============================================================================

#[test]
fn test_normalize_owned_returns_string() {
    let result = normalize_owned("src\\lib.rs");
    assert_eq!(result, "src/lib.rs");
}

#[test]
fn test_normalize_collapse_multiple_slashes() {
    assert_eq!(normalize("src///lib.rs"), "src/lib.rs");
}

#[test]
fn test_normalize_collapse_leading_slashes() {
    assert_eq!(normalize("//src/lib.rs"), "/src/lib.rs");
}

#[test]
fn test_paths_eq_empty() {
    assert!(paths_eq("", ""));
}

#[test]
fn test_paths_cmp_empty() {
    assert_eq!(paths_cmp("", ""), std::cmp::Ordering::Equal);
}

#[test]
fn test_extension_with_windows_path() {
    assert_eq!(extension("src\\lib.rs"), Some("rs"));
}

#[test]
fn test_file_name_with_windows_path() {
    assert_eq!(file_name("src\\lib.rs"), Some("lib.rs"));
}

#[test]
fn test_parent_with_windows_path() {
    assert_eq!(parent("src\\lib.rs"), Some("src"));
}

#[test]
fn test_normalize_preserves_case() {
    assert_eq!(normalize("SRC/Lib.RS"), "SRC/Lib.RS");
}

#[test]
fn test_normalize_with_spaces() {
    assert_eq!(normalize("src/my file.rs"), "src/my file.rs");
}

#[test]
fn test_normalize_special_chars() {
    assert_eq!(normalize("src/foo-bar_baz.rs"), "src/foo-bar_baz.rs");
}

#[test]
fn test_parent_root_file() {
    assert_eq!(parent("/lib.rs"), Some(""));
}

#[test]
fn test_file_name_root_file() {
    assert_eq!(file_name("/lib.rs"), Some("lib.rs"));
}

#[test]
fn test_extension_tar_gz() {
    assert_eq!(extension("archive.tar.gz"), Some("gz"));
}

#[test]
fn test_extension_double_dot() {
    assert_eq!(extension("file..rs"), Some("rs"));
}

#[test]
fn test_normalize_diff_prefix_preserves_a_directory() {
    // If the actual directory is named "a", it should be preserved after "a/" prefix strip
    assert_eq!(normalize("a/a/file.rs"), "a/file.rs");
}

#[test]
fn test_config_clone() {
    let config = NormalizeConfig::new();
    let cloned = config.clone();
    assert_eq!(config, cloned);
}

#[test]
fn test_config_debug() {
    let config = NormalizeConfig::new();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("NormalizeConfig"));
}

#[test]
fn test_normalized_path_clone() {
    let path = NormalizedPath::new("src/lib.rs");
    let cloned = path.clone();
    assert_eq!(path, cloned);
}

#[test]
fn test_normalized_path_debug() {
    let path = NormalizedPath::new("src/lib.rs");
    let debug_str = format!("{:?}", path);
    assert!(debug_str.contains("NormalizedPath"));
}
