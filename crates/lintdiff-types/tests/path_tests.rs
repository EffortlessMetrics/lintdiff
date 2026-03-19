//! Comprehensive tests for path normalization.
//!
//! Tests cover:
//! - Path normalization (backslash to forward slash)
//! - Diff prefix stripping (a/, b/)
//! - Leading ./ stripping
//! - Multiple slash collapsing
//! - LineRange operations
//! - Edge cases (empty paths, unicode, whitespace)

use lintdiff_types::*;

// =============================================================================
// NormPath Tests
// =============================================================================

mod norm_path_tests {
    use super::*;

    #[test]
    fn new_creates_normalized_path() {
        let path = NormPath::new("src/lib.rs");
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn as_str_returns_inner() {
        let path = NormPath::new("src/lib.rs");
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn into_string_consumes() {
        let path = NormPath::new("src/lib.rs");
        let s = path.into_string();
        assert_eq!(s, "src/lib.rs");
    }

    #[test]
    fn display_format() {
        let path = NormPath::new("src/lib.rs");
        assert_eq!(format!("{}", path), "src/lib.rs");
    }

    #[test]
    fn from_string_trait() {
        let path = NormPath::from("src/lib.rs".to_string());
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn from_str_trait() {
        let path = NormPath::from("src/lib.rs");
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn clone() {
        let path = NormPath::new("src/lib.rs");
        let cloned = path.clone();
        assert_eq!(path.as_str(), cloned.as_str());
    }

    #[test]
    fn eq() {
        let a = NormPath::new("src/lib.rs");
        let b = NormPath::new("src/lib.rs");
        assert_eq!(a, b);
    }

    #[test]
    fn ord() {
        let a = NormPath::new("src/a.rs");
        let b = NormPath::new("src/b.rs");
        assert!(a < b);
    }

    #[test]
    fn hash_consistency() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(NormPath::new("src/lib.rs"));
        set.insert(NormPath::new("src/lib.rs"));

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn serialize() {
        let path = NormPath::new("src/lib.rs");
        let json = serde_json::to_string(&path).unwrap();
        assert_eq!(json, r#""src/lib.rs""#);
    }

    #[test]
    fn deserialize() {
        let path: NormPath = serde_json::from_str(r#""src/lib.rs""#).unwrap();
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn debug_format() {
        let path = NormPath::new("src/lib.rs");
        let debug = format!("{:?}", path);
        assert!(debug.contains("NormPath"));
    }
}

// =============================================================================
// Path Normalization Tests
// =============================================================================

mod normalize_path_tests {
    use super::*;

    #[test]
    fn simple_path_unchanged() {
        let path = normalize_path("src/lib.rs");
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn backslash_converted_to_forward_slash() {
        let path = normalize_path("src\\lib.rs");
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn multiple_backslashes_converted() {
        let path = normalize_path("src\\nested\\deep\\lib.rs");
        assert_eq!(path.as_str(), "src/nested/deep/lib.rs");
    }

    #[test]
    fn mixed_slashes_normalized() {
        let path = normalize_path("src\\nested/deep\\lib.rs");
        assert_eq!(path.as_str(), "src/nested/deep/lib.rs");
    }

    #[test]
    fn diff_prefix_a_stripped() {
        let path = normalize_path("a/src/lib.rs");
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn diff_prefix_b_stripped() {
        let path = normalize_path("b/src/lib.rs");
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn diff_prefix_only_stripped_at_start() {
        // "a/" in the middle should not be stripped
        let path = normalize_path("src/a/lib.rs");
        assert_eq!(path.as_str(), "src/a/lib.rs");
    }

    #[test]
    fn leading_dot_slash_stripped() {
        let path = normalize_path("./src/lib.rs");
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn multiple_leading_dot_slash_stripped() {
        let path = normalize_path("./././src/lib.rs");
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn double_slash_collapsed() {
        let path = normalize_path("src//lib.rs");
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn multiple_double_slashes_collapsed() {
        let path = normalize_path("src//nested///lib.rs");
        assert_eq!(path.as_str(), "src/nested/lib.rs");
    }

    #[test]
    fn all_normalizations_combined() {
        // Windows path with diff prefix and ./
        let path = normalize_path("a/.\\src\\nested//file.rs");
        assert_eq!(path.as_str(), "src/nested/file.rs");
    }

    #[test]
    fn empty_path() {
        let path = normalize_path("");
        assert_eq!(path.as_str(), "");
    }

    #[test]
    fn whitespace_only() {
        let path = normalize_path("   ");
        assert_eq!(path.as_str(), "");
    }

    #[test]
    fn whitespace_trimmed() {
        let path = normalize_path("  src/lib.rs  ");
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn just_dot_slash() {
        let path = normalize_path("./");
        assert_eq!(path.as_str(), "");
    }

    #[test]
    fn just_a_slash() {
        let path = normalize_path("a/");
        assert_eq!(path.as_str(), "");
    }

    #[test]
    fn just_b_slash() {
        let path = normalize_path("b/");
        assert_eq!(path.as_str(), "");
    }

    #[test]
    fn root_path() {
        let path = normalize_path("/");
        assert_eq!(path.as_str(), "/");
    }

    #[test]
    fn filename_only() {
        let path = normalize_path("lib.rs");
        assert_eq!(path.as_str(), "lib.rs");
    }

    #[test]
    fn hidden_file() {
        let path = normalize_path(".hidden");
        assert_eq!(path.as_str(), ".hidden");
    }

    #[test]
    fn hidden_file_in_directory() {
        let path = normalize_path("src/.hidden");
        assert_eq!(path.as_str(), "src/.hidden");
    }
}

// =============================================================================
// Cross-Platform Tests
// =============================================================================

mod cross_platform_tests {
    use super::*;

    #[test]
    fn windows_absolute_path_normalized() {
        // C:\path\to\file.rs -> C:/path/to/file.rs
        let path = normalize_path("C:\\path\\to\\file.rs");
        assert_eq!(path.as_str(), "C:/path/to/file.rs");
    }

    #[test]
    fn windows_unc_path_normalized() {
        // \\server\share\file.rs -> /server/share/file.rs
        // (double slashes are collapsed)
        let path = normalize_path("\\\\server\\share\\file.rs");
        assert_eq!(path.as_str(), "/server/share/file.rs");
    }

    #[test]
    fn unix_absolute_path_unchanged() {
        let path = normalize_path("/usr/local/src/lib.rs");
        assert_eq!(path.as_str(), "/usr/local/src/lib.rs");
    }

    #[test]
    fn relative_path_parent_directory() {
        let path = normalize_path("../src/lib.rs");
        assert_eq!(path.as_str(), "../src/lib.rs");
    }

    #[test]
    fn relative_path_multiple_parents() {
        let path = normalize_path("../../src/lib.rs");
        assert_eq!(path.as_str(), "../../src/lib.rs");
    }

    #[test]
    fn windows_relative_with_backslash() {
        let path = normalize_path("..\\src\\lib.rs");
        assert_eq!(path.as_str(), "../src/lib.rs");
    }
}

// =============================================================================
// Unicode Tests
// =============================================================================

mod unicode_tests {
    use super::*;

    #[test]
    fn unicode_filename() {
        let path = normalize_path("src/日本語.rs");
        assert_eq!(path.as_str(), "src/日本語.rs");
    }

    #[test]
    fn unicode_directory() {
        let path = normalize_path("文件夹/文件.rs");
        assert_eq!(path.as_str(), "文件夹/文件.rs");
    }

    #[test]
    fn emoji_in_path() {
        let path = normalize_path("src/🎉/lib.rs");
        assert_eq!(path.as_str(), "src/🎉/lib.rs");
    }

    #[test]
    fn unicode_with_backslash() {
        let path = normalize_path("文件夹\\文件.rs");
        assert_eq!(path.as_str(), "文件夹/文件.rs");
    }

    #[test]
    fn mixed_unicode_and_ascii() {
        let path = normalize_path("src/日本語/lib.rs");
        assert_eq!(path.as_str(), "src/日本語/lib.rs");
    }

    #[test]
    fn unicode_preserved_through_normalization() {
        let original = "src/αβγ/δ.rs";
        let path = normalize_path(original);
        assert_eq!(path.as_str(), original);
    }
}

// =============================================================================
// Edge Cases
// =============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn very_long_path() {
        let long_path = "src/".repeat(100) + "lib.rs";
        let path = normalize_path(&long_path);
        assert!(path.as_str().ends_with("lib.rs"));
    }

    #[test]
    fn path_with_spaces() {
        let path = normalize_path("src/my file.rs");
        assert_eq!(path.as_str(), "src/my file.rs");
    }

    #[test]
    fn path_with_special_characters() {
        let path = normalize_path("src/file-name_1.0.rs");
        assert_eq!(path.as_str(), "src/file-name_1.0.rs");
    }

    #[test]
    fn path_with_parentheses() {
        let path = normalize_path("src/file (copy).rs");
        assert_eq!(path.as_str(), "src/file (copy).rs");
    }

    #[test]
    fn path_with_brackets() {
        let path = normalize_path("src/file[1].rs");
        assert_eq!(path.as_str(), "src/file[1].rs");
    }

    #[test]
    fn path_with_at_sign() {
        let path = normalize_path("src/@scope/file.rs");
        assert_eq!(path.as_str(), "src/@scope/file.rs");
    }

    #[test]
    fn path_with_hash() {
        let path = normalize_path("src/#file.rs");
        assert_eq!(path.as_str(), "src/#file.rs");
    }

    #[test]
    fn path_with_percent() {
        let path = normalize_path("src/100%.rs");
        assert_eq!(path.as_str(), "src/100%.rs");
    }

    #[test]
    fn multiple_consecutive_slashes() {
        let path = normalize_path("src////lib.rs");
        assert_eq!(path.as_str(), "src/lib.rs");
    }

    #[test]
    fn trailing_slash() {
        let path = normalize_path("src/lib.rs/");
        assert_eq!(path.as_str(), "src/lib.rs/");
    }

    #[test]
    fn trailing_backslash() {
        let path = normalize_path("src\\lib.rs\\");
        assert_eq!(path.as_str(), "src/lib.rs/");
    }

    #[test]
    fn dot_in_filename() {
        let path = normalize_path("src/lib.test.rs");
        assert_eq!(path.as_str(), "src/lib.test.rs");
    }

    #[test]
    fn no_extension() {
        let path = normalize_path("src/Makefile");
        assert_eq!(path.as_str(), "src/Makefile");
    }

    #[test]
    fn multiple_extensions() {
        let path = normalize_path("src/archive.tar.gz");
        assert_eq!(path.as_str(), "src/archive.tar.gz");
    }

    #[test]
    fn diff_prefix_after_normalization() {
        // If the path starts with "a/" after backslash conversion
        let path = normalize_path("a\\src\\lib.rs");
        // First backslash converts to forward, then a/ is stripped
        // But actually, a\src\lib.rs -> a/src/lib.rs -> src/lib.rs
        assert_eq!(path.as_str(), "src/lib.rs");
    }
}

// =============================================================================
// LineRange Tests
// =============================================================================

mod line_range_tests {
    use super::*;

    #[test]
    fn new_creates_range() {
        let range = LineRange::new(1, 10);
        assert_eq!(range.start, 1);
        assert_eq!(range.end, 10);
    }

    #[test]
    fn intersects_overlapping() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(5, 15);
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn intersects_touching() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(10, 20);
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn intersects_non_overlapping() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(11, 20);
        assert!(!a.intersects(&b));
        assert!(!b.intersects(&a));
    }

    #[test]
    fn intersects_same_range() {
        let range = LineRange::new(1, 10);
        assert!(range.intersects(&range));
    }

    #[test]
    fn intersects_contained() {
        let outer = LineRange::new(1, 20);
        let inner = LineRange::new(5, 10);
        assert!(outer.intersects(&inner));
        assert!(inner.intersects(&outer));
    }

    #[test]
    fn contains_line_within() {
        let range = LineRange::new(1, 10);
        assert!(range.contains_line(1));
        assert!(range.contains_line(5));
        assert!(range.contains_line(10));
    }

    #[test]
    fn contains_line_outside() {
        let range = LineRange::new(1, 10);
        assert!(!range.contains_line(0));
        assert!(!range.contains_line(11));
    }

    #[test]
    fn contains_line_at_boundaries() {
        let range = LineRange::new(5, 10);
        assert!(range.contains_line(5)); // start
        assert!(range.contains_line(10)); // end
        assert!(!range.contains_line(4)); // before start
        assert!(!range.contains_line(11)); // after end
    }

    #[test]
    fn single_line_range() {
        let range = LineRange::new(5, 5);
        assert!(range.contains_line(5));
        assert!(!range.contains_line(4));
        assert!(!range.contains_line(6));
        assert!(range.intersects(&range));
    }

    #[test]
    fn clone() {
        let range = LineRange::new(1, 10);
        let cloned = range;
        assert_eq!(range.start, cloned.start);
        assert_eq!(range.end, cloned.end);
    }

    #[test]
    fn eq() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(1, 10);
        assert_eq!(a, b);
    }

    #[test]
    fn ord() {
        let a = LineRange::new(1, 10);
        let b = LineRange::new(2, 10);
        assert!(a < b);
    }

    #[test]
    fn hash_consistency() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(LineRange::new(1, 10));
        set.insert(LineRange::new(1, 10));

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn serialize() {
        let range = LineRange::new(1, 10);
        let json = serde_json::to_string(&range).unwrap();
        assert!(json.contains("\"start\":1"));
        assert!(json.contains("\"end\":10"));
    }

    #[test]
    fn deserialize() {
        let json = r#"{"start":1,"end":10}"#;
        let range: LineRange = serde_json::from_str(json).unwrap();
        assert_eq!(range.start, 1);
        assert_eq!(range.end, 10);
    }

    #[test]
    fn debug_format() {
        let range = LineRange::new(1, 10);
        let debug = format!("{:?}", range);
        assert!(debug.contains("LineRange"));
    }

    #[test]
    fn copy_trait() {
        let range = LineRange::new(1, 10);
        let copied = range; // Copy, not move
        let _still_valid = range; // Should still work
        assert_eq!(copied.start, 1);
    }
}

// =============================================================================
// Integration Tests
// =============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn norm_path_equality_after_different_normalizations() {
        // These should all be equal after normalization
        let paths = [
            normalize_path("src/lib.rs"),
            normalize_path("src\\lib.rs"),
            normalize_path("./src/lib.rs"),
            normalize_path("src//lib.rs"),
        ];

        let first = &paths[0];
        for path in &paths[1..] {
            assert_eq!(first, path, "Normalized paths should be equal");
        }
    }

    #[test]
    fn norm_path_in_hashmap() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert(NormPath::new("src/lib.rs"), 1);
        map.insert(NormPath::new("src\\lib.rs"), 2); // Same path, different normalization

        // Both insertions should result in a single entry
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&NormPath::new("src/lib.rs")), Some(&2));
    }

    #[test]
    fn norm_path_in_hashset() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(NormPath::new("src/lib.rs"));
        set.insert(NormPath::new("src\\lib.rs"));
        set.insert(NormPath::new("./src/lib.rs"));

        // All should be the same after normalization
        assert_eq!(set.len(), 1);
    }
}
