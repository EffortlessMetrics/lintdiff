//! Comprehensive BDD tests for lintdiff-diff-paths.
//!
//! These tests cover all public API functions and edge cases.

use lintdiff_diff_paths::{
    extract_normalized_paths, extract_paths_from_header, is_dev_null, normalize_path,
    normalize_path_owned, parse_new_line, parse_old_line, strip_diff_prefix, DiffPaths,
    DiffPathsError,
};

// ============================================================================
// DiffPathsError Tests
// ============================================================================

mod error_tests {
    use super::*;

    #[test]
    fn error_new_creates_message() {
        let err = DiffPathsError::new("something went wrong");
        assert_eq!(err.message(), "something went wrong");
        assert!(err.line().is_none());
    }

    #[test]
    fn error_with_line_stores_line() {
        let err = DiffPathsError::with_line("parse error", "--- invalid");
        assert_eq!(err.message(), "parse error");
        assert_eq!(err.line(), Some("--- invalid"));
    }

    #[test]
    fn error_display_without_line() {
        let err = DiffPathsError::new("test error");
        assert_eq!(format!("{}", err), "test error");
    }

    #[test]
    fn error_display_with_line() {
        let err = DiffPathsError::with_line("test error", "bad line");
        assert_eq!(format!("{}", err), "test error: \"bad line\"");
    }

    #[test]
    fn error_is_std_error() {
        let err = DiffPathsError::new("test");
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn error_clone() {
        let err = DiffPathsError::with_line("test", "line");
        let cloned = err.clone();
        assert_eq!(err.message(), cloned.message());
        assert_eq!(err.line(), cloned.line());
    }

    #[test]
    fn error_equality() {
        let err1 = DiffPathsError::new("test");
        let err2 = DiffPathsError::new("test");
        assert_eq!(err1, err2);
    }
}

// ============================================================================
// is_dev_null Tests
// ============================================================================

mod is_dev_null_tests {
    use super::*;

    #[test]
    fn recognizes_dev_null_with_leading_slash() {
        assert!(is_dev_null("/dev/null"));
    }

    #[test]
    fn recognizes_dev_null_without_leading_slash() {
        assert!(is_dev_null("dev/null"));
    }

    #[test]
    fn rejects_regular_path() {
        assert!(!is_dev_null("src/lib.rs"));
    }

    #[test]
    fn rejects_empty_string() {
        assert!(!is_dev_null(""));
    }

    #[test]
    fn rejects_dev_null_with_extra_chars() {
        assert!(!is_dev_null("/dev/nulls"));
        assert!(!is_dev_null("/dev/null/extra"));
    }

    #[test]
    fn rejects_partial_match() {
        assert!(!is_dev_null("dev/"));
        assert!(!is_dev_null("/null"));
    }
}

// ============================================================================
// strip_diff_prefix Tests
// ============================================================================

mod strip_diff_prefix_tests {
    use super::*;

    #[test]
    fn strips_a_prefix() {
        assert_eq!(strip_diff_prefix("a/src/lib.rs"), "src/lib.rs");
    }

    #[test]
    fn strips_b_prefix() {
        assert_eq!(strip_diff_prefix("b/src/lib.rs"), "src/lib.rs");
    }

    #[test]
    fn strips_i_prefix() {
        assert_eq!(strip_diff_prefix("i/src/lib.rs"), "src/lib.rs");
    }

    #[test]
    fn no_prefix_returns_unchanged() {
        assert_eq!(strip_diff_prefix("src/lib.rs"), "src/lib.rs");
    }

    #[test]
    fn preserves_dev_null() {
        assert_eq!(strip_diff_prefix("/dev/null"), "/dev/null");
    }

    #[test]
    fn only_a_slash_returns_empty() {
        assert_eq!(strip_diff_prefix("a/"), "");
    }

    #[test]
    fn nested_paths() {
        assert_eq!(
            strip_diff_prefix("a/deep/nested/path.rs"),
            "deep/nested/path.rs"
        );
    }

    #[test]
    fn does_not_strip_middle_a_prefix() {
        assert_eq!(strip_diff_prefix("src/a/file.rs"), "src/a/file.rs");
    }

    #[test]
    fn empty_string() {
        assert_eq!(strip_diff_prefix(""), "");
    }

    #[test]
    fn single_char() {
        assert_eq!(strip_diff_prefix("a"), "a");
    }

    #[test]
    fn a_without_slash_not_stripped() {
        assert_eq!(strip_diff_prefix("abc"), "abc");
    }
}

// ============================================================================
// normalize_path Tests
// ============================================================================

mod normalize_path_tests {
    use super::*;

    #[test]
    fn strips_a_prefix() {
        assert_eq!(normalize_path("a/src/lib.rs"), "src/lib.rs");
    }

    #[test]
    fn strips_b_prefix() {
        assert_eq!(normalize_path("b/src/lib.rs"), "src/lib.rs");
    }

    #[test]
    fn converts_backslashes() {
        assert_eq!(normalize_path("src\\lib.rs"), "src/lib.rs");
    }

    #[test]
    fn strips_prefix_and_converts_backslashes() {
        assert_eq!(normalize_path("b\\src\\lib.rs"), "src/lib.rs");
    }

    #[test]
    fn preserves_dev_null() {
        assert_eq!(normalize_path("/dev/null"), "/dev/null");
    }

    #[test]
    fn no_changes_needed() {
        assert_eq!(normalize_path("src/lib.rs"), "src/lib.rs");
    }

    #[test]
    fn multiple_backslashes() {
        assert_eq!(normalize_path("src\\nested\\lib.rs"), "src/nested/lib.rs");
    }

    #[test]
    fn mixed_separators() {
        assert_eq!(normalize_path("a/src\\lib.rs"), "src/lib.rs");
    }
}

// ============================================================================
// normalize_path_owned Tests
// ============================================================================

mod normalize_path_owned_tests {
    use super::*;

    #[test]
    fn returns_owned_string() {
        let result = normalize_path_owned("a/src/lib.rs");
        assert_eq!(result, "src/lib.rs");
    }

    #[test]
    fn handles_backslashes() {
        let result = normalize_path_owned("src\\lib.rs");
        assert_eq!(result, "src/lib.rs");
    }
}

// ============================================================================
// DiffPaths Construction Tests
// ============================================================================

mod diff_paths_construction_tests {
    use super::*;

    #[test]
    fn new_creates_empty() {
        let paths = DiffPaths::new();
        assert!(paths.old_path.is_none());
        assert!(paths.new_path.is_none());
        assert!(paths.old_timestamp.is_none());
        assert!(paths.new_timestamp.is_none());
    }

    #[test]
    fn default_same_as_new() {
        let paths = DiffPaths::default();
        assert!(paths.old_path.is_none());
        assert!(paths.new_path.is_none());
    }

    #[test]
    fn creation_sets_new_path() {
        let paths = DiffPaths::creation("src/new.rs");
        assert!(paths.old_path.is_none());
        assert_eq!(paths.new_path, Some("src/new.rs".to_string()));
    }

    #[test]
    fn deletion_sets_old_path() {
        let paths = DiffPaths::deletion("src/old.rs");
        assert_eq!(paths.old_path, Some("src/old.rs".to_string()));
        assert!(paths.new_path.is_none());
    }

    #[test]
    fn modification_sets_both() {
        let paths = DiffPaths::modification("src/lib.rs");
        assert_eq!(paths.old_path, Some("src/lib.rs".to_string()));
        assert_eq!(paths.new_path, Some("src/lib.rs".to_string()));
    }

    #[test]
    fn rename_sets_different_paths() {
        let paths = DiffPaths::rename("src/old.rs", "src/new.rs");
        assert_eq!(paths.old_path, Some("src/old.rs".to_string()));
        assert_eq!(paths.new_path, Some("src/new.rs".to_string()));
    }
}

// ============================================================================
// DiffPaths::parse Tests - Standard Headers
// ============================================================================

mod parse_standard_tests {
    use super::*;

    #[test]
    fn parse_standard_git_header() {
        let header = "--- a/src/lib.rs\n+++ b/src/lib.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/src/lib.rs".to_string()));
        assert_eq!(paths.new_path, Some("b/src/lib.rs".to_string()));
    }

    #[test]
    fn parse_with_trailing_whitespace() {
        let header = "--- a/src/lib.rs  \n+++ b/src/lib.rs  \n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/src/lib.rs".to_string()));
        assert_eq!(paths.new_path, Some("b/src/lib.rs".to_string()));
    }

    #[test]
    fn parse_with_leading_whitespace() {
        let header = "  --- a/src/lib.rs\n  +++ b/src/lib.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/src/lib.rs".to_string()));
    }

    #[test]
    fn parse_without_trailing_newline() {
        let header = "--- a/src/lib.rs\n+++ b/src/lib.rs";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/src/lib.rs".to_string()));
        assert_eq!(paths.new_path, Some("b/src/lib.rs".to_string()));
    }

    #[test]
    fn parse_only_old_line() {
        let header = "--- a/src/lib.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/src/lib.rs".to_string()));
        assert!(paths.new_path.is_none());
    }

    #[test]
    fn parse_only_new_line() {
        let header = "+++ b/src/lib.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert!(paths.old_path.is_none());
        assert_eq!(paths.new_path, Some("b/src/lib.rs".to_string()));
    }

    #[test]
    fn parse_empty_string() {
        let result = DiffPaths::parse("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_no_header_lines() {
        let header = "some random text\nno headers here\n";
        let result = DiffPaths::parse(header).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_empty_lines_only() {
        let header = "\n\n\n";
        let result = DiffPaths::parse(header).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_windows_line_endings() {
        let header = "--- a/src/lib.rs\r\n+++ b/src/lib.rs\r\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/src/lib.rs".to_string()));
    }
}

// ============================================================================
// DiffPaths::parse Tests - Creation Detection
// ============================================================================

mod parse_creation_tests {
    use super::*;

    #[test]
    fn creation_with_dev_null() {
        let header = "--- /dev/null\n+++ b/new_file.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert!(paths.is_creation());
    }

    #[test]
    fn creation_with_dev_null_no_slash() {
        let header = "--- dev/null\n+++ b/new_file.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert!(paths.is_creation());
    }

    #[test]
    fn creation_missing_old_path() {
        let paths = DiffPaths {
            old_path: None,
            new_path: Some("new.rs".to_string()),
            old_timestamp: None,
            new_timestamp: None,
        };

        assert!(paths.is_creation());
    }

    #[test]
    fn creation_canonical_path_is_new() {
        let header = "--- /dev/null\n+++ b/new_file.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.canonical_path(), Some("b/new_file.rs"));
    }
}

// ============================================================================
// DiffPaths::parse Tests - Deletion Detection
// ============================================================================

mod parse_deletion_tests {
    use super::*;

    #[test]
    fn deletion_with_dev_null() {
        let header = "--- a/old_file.rs\n+++ /dev/null\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert!(paths.is_deletion());
    }

    #[test]
    fn deletion_with_dev_null_no_slash() {
        let header = "--- a/old_file.rs\n+++ dev/null\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert!(paths.is_deletion());
    }

    #[test]
    fn deletion_missing_new_path() {
        let paths = DiffPaths {
            old_path: Some("old.rs".to_string()),
            new_path: None,
            old_timestamp: None,
            new_timestamp: None,
        };

        assert!(paths.is_deletion());
    }

    #[test]
    fn deletion_canonical_path_is_old() {
        let header = "--- a/old_file.rs\n+++ /dev/null\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.canonical_path(), Some("a/old_file.rs"));
    }
}

// ============================================================================
// DiffPaths::parse Tests - Rename Detection
// ============================================================================

mod parse_rename_tests {
    use super::*;

    #[test]
    fn rename_different_paths() {
        let paths = DiffPaths::rename("src/old.rs", "src/new.rs");
        assert!(paths.is_rename());
    }

    #[test]
    fn rename_not_modification() {
        let paths = DiffPaths::rename("src/old.rs", "src/new.rs");
        assert!(!paths.is_modification());
    }

    #[test]
    fn rename_not_creation() {
        let paths = DiffPaths::rename("src/old.rs", "src/new.rs");
        assert!(!paths.is_creation());
    }

    #[test]
    fn rename_not_deletion() {
        let paths = DiffPaths::rename("src/old.rs", "src/new.rs");
        assert!(!paths.is_deletion());
    }

    #[test]
    fn same_path_not_rename() {
        let paths = DiffPaths::modification("src/lib.rs");
        assert!(!paths.is_rename());
    }

    #[test]
    fn creation_not_rename() {
        let paths = DiffPaths::creation("src/new.rs");
        assert!(!paths.is_rename());
    }

    #[test]
    fn deletion_not_rename() {
        let paths = DiffPaths::deletion("src/old.rs");
        assert!(!paths.is_rename());
    }

    #[test]
    fn rename_with_diff_prefixes() {
        let header = "--- a/src/old.rs\n+++ b/src/new.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert!(paths.is_rename());
    }

    #[test]
    fn canonical_path_prefers_new() {
        let paths = DiffPaths::rename("old.rs", "new.rs");
        assert_eq!(paths.canonical_path(), Some("new.rs"));
    }
}

// ============================================================================
// DiffPaths::parse Tests - Timestamp Handling
// ============================================================================

mod timestamp_tests {
    use super::*;

    #[test]
    fn parse_tab_separated_timestamp() {
        let header = "--- a/file.rs\t2024-01-01 12:00:00\n+++ b/file.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_timestamp, Some("2024-01-01 12:00:00".to_string()));
    }

    #[test]
    fn parse_both_timestamps() {
        let header = "--- a/file.rs\t2024-01-01 12:00:00\n+++ b/file.rs\t2024-01-02 13:00:00\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_timestamp, Some("2024-01-01 12:00:00".to_string()));
        assert_eq!(paths.new_timestamp, Some("2024-01-02 13:00:00".to_string()));
    }

    #[test]
    fn parse_double_space_timestamp() {
        let header = "--- a/file.rs  2024-01-01\n+++ b/file.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_timestamp, Some("2024-01-01".to_string()));
    }

    #[test]
    fn no_timestamp_returns_none() {
        let header = "--- a/file.rs\n+++ b/file.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert!(paths.old_timestamp.is_none());
        assert!(paths.new_timestamp.is_none());
    }

    #[test]
    fn timestamp_preserved_through_clone() {
        let paths = DiffPaths {
            old_path: Some("a.rs".to_string()),
            new_path: Some("b.rs".to_string()),
            old_timestamp: Some("2024-01-01".to_string()),
            new_timestamp: Some("2024-01-02".to_string()),
        };

        let cloned = paths.clone();
        assert_eq!(cloned.old_timestamp, paths.old_timestamp);
        assert_eq!(cloned.new_timestamp, paths.new_timestamp);
    }
}

// ============================================================================
// DiffPaths Method Tests - is_modification
// ============================================================================

mod is_modification_tests {
    use super::*;

    #[test]
    fn same_path_is_modification() {
        let paths = DiffPaths::modification("src/lib.rs");
        assert!(paths.is_modification());
    }

    #[test]
    fn different_paths_not_modification() {
        let paths = DiffPaths::rename("old.rs", "new.rs");
        assert!(!paths.is_modification());
    }

    #[test]
    fn creation_not_modification() {
        let paths = DiffPaths::creation("new.rs");
        assert!(!paths.is_modification());
    }

    #[test]
    fn deletion_not_modification() {
        let paths = DiffPaths::deletion("old.rs");
        assert!(!paths.is_modification());
    }

    #[test]
    fn modification_with_dev_null_old_not_modification() {
        let paths = DiffPaths {
            old_path: Some("/dev/null".to_string()),
            new_path: Some("new.rs".to_string()),
            old_timestamp: None,
            new_timestamp: None,
        };
        assert!(!paths.is_modification());
    }

    #[test]
    fn modification_with_dev_null_new_not_modification() {
        let paths = DiffPaths {
            old_path: Some("old.rs".to_string()),
            new_path: Some("/dev/null".to_string()),
            old_timestamp: None,
            new_timestamp: None,
        };
        assert!(!paths.is_modification());
    }
}

// ============================================================================
// DiffPaths Method Tests - canonical_path
// ============================================================================

mod canonical_path_tests {
    use super::*;

    #[test]
    fn prefers_new_path() {
        let paths = DiffPaths::rename("old.rs", "new.rs");
        assert_eq!(paths.canonical_path(), Some("new.rs"));
    }

    #[test]
    fn falls_back_to_old_path() {
        let paths = DiffPaths::deletion("old.rs");
        assert_eq!(paths.canonical_path(), Some("old.rs"));
    }

    #[test]
    fn skips_dev_null_new() {
        let paths = DiffPaths {
            old_path: Some("old.rs".to_string()),
            new_path: Some("/dev/null".to_string()),
            old_timestamp: None,
            new_timestamp: None,
        };
        assert_eq!(paths.canonical_path(), Some("old.rs"));
    }

    #[test]
    fn skips_dev_null_old() {
        let paths = DiffPaths {
            old_path: Some("/dev/null".to_string()),
            new_path: Some("new.rs".to_string()),
            old_timestamp: None,
            new_timestamp: None,
        };
        assert_eq!(paths.canonical_path(), Some("new.rs"));
    }

    #[test]
    fn none_if_both_missing() {
        let paths = DiffPaths::new();
        assert!(paths.canonical_path().is_none());
    }
}

// ============================================================================
// DiffPaths Method Tests - normalized paths
// ============================================================================

mod normalized_paths_tests {
    use super::*;

    #[test]
    fn old_path_normalized_strips_prefix() {
        let paths = DiffPaths::parse("--- a/src/lib.rs\n").unwrap().unwrap();
        assert_eq!(paths.old_path_normalized(), Some("src/lib.rs"));
    }

    #[test]
    fn new_path_normalized_strips_prefix() {
        let paths = DiffPaths::parse("+++ b/src/lib.rs\n").unwrap().unwrap();
        assert_eq!(paths.new_path_normalized(), Some("src/lib.rs"));
    }

    #[test]
    fn canonical_path_normalized() {
        let header = "--- a/src/lib.rs\n+++ b/src/lib.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();
        assert_eq!(paths.canonical_path_normalized(), Some("src/lib.rs"));
    }

    #[test]
    fn normalized_handles_dev_null() {
        let paths = DiffPaths::parse("--- /dev/null\n+++ b/new.rs\n")
            .unwrap()
            .unwrap();
        assert_eq!(paths.old_path_normalized(), Some("/dev/null"));
    }

    #[test]
    fn normalized_handles_i_prefix() {
        let paths = DiffPaths::parse("--- i/src/lib.rs\n").unwrap().unwrap();
        assert_eq!(paths.old_path_normalized(), Some("src/lib.rs"));
    }
}

// ============================================================================
// DiffPaths Method Tests - strip_prefix
// ============================================================================

mod strip_prefix_tests {
    use super::*;

    #[test]
    fn strips_specified_prefix() {
        let paths = DiffPaths::parse("--- a/src/lib.rs\n+++ b/src/lib.rs\n")
            .unwrap()
            .unwrap();
        let stripped = paths.strip_prefix("a/");

        assert_eq!(stripped.old_path, Some("src/lib.rs".to_string()));
        assert_eq!(stripped.new_path, Some("b/src/lib.rs".to_string()));
    }

    #[test]
    fn preserves_dev_null() {
        let paths = DiffPaths::parse("--- /dev/null\n+++ b/new.rs\n")
            .unwrap()
            .unwrap();
        let stripped = paths.strip_prefix("a/");

        assert_eq!(stripped.old_path, Some("/dev/null".to_string()));
    }

    #[test]
    fn no_match_returns_unchanged() {
        let paths = DiffPaths::parse("--- src/lib.rs\n").unwrap().unwrap();
        let stripped = paths.strip_prefix("a/");

        assert_eq!(stripped.old_path, Some("src/lib.rs".to_string()));
    }

    #[test]
    fn preserves_timestamps() {
        let paths = DiffPaths {
            old_path: Some("a/file.rs".to_string()),
            new_path: Some("b/file.rs".to_string()),
            old_timestamp: Some("2024-01-01".to_string()),
            new_timestamp: Some("2024-01-02".to_string()),
        };

        let stripped = paths.strip_prefix("a/");
        assert_eq!(stripped.old_timestamp, Some("2024-01-01".to_string()));
        assert_eq!(stripped.new_timestamp, Some("2024-01-02".to_string()));
    }
}

// ============================================================================
// DiffPaths Method Tests - strip_diff_prefixes
// ============================================================================

mod strip_diff_prefixes_tests {
    use super::*;

    #[test]
    fn strips_all_diff_prefixes() {
        let paths = DiffPaths::parse("--- a/src/lib.rs\n+++ b/src/lib.rs\n")
            .unwrap()
            .unwrap();
        let normalized = paths.strip_diff_prefixes();

        assert_eq!(normalized.old_path, Some("src/lib.rs".to_string()));
        assert_eq!(normalized.new_path, Some("src/lib.rs".to_string()));
    }

    #[test]
    fn handles_rename() {
        let paths = DiffPaths::parse("--- a/old.rs\n+++ b/new.rs\n")
            .unwrap()
            .unwrap();
        let normalized = paths.strip_diff_prefixes();

        assert_eq!(normalized.old_path, Some("old.rs".to_string()));
        assert_eq!(normalized.new_path, Some("new.rs".to_string()));
    }

    #[test]
    fn preserves_dev_null() {
        let paths = DiffPaths::parse("--- /dev/null\n+++ b/new.rs\n")
            .unwrap()
            .unwrap();
        let normalized = paths.strip_diff_prefixes();

        assert_eq!(normalized.old_path, Some("/dev/null".to_string()));
    }

    #[test]
    fn handles_backslashes() {
        let paths = DiffPaths {
            old_path: Some("a\\src\\lib.rs".to_string()),
            new_path: Some("b\\src\\lib.rs".to_string()),
            old_timestamp: None,
            new_timestamp: None,
        };

        let normalized = paths.strip_diff_prefixes();
        assert_eq!(normalized.old_path, Some("src/lib.rs".to_string()));
        assert_eq!(normalized.new_path, Some("src/lib.rs".to_string()));
    }
}

// ============================================================================
// DiffPaths Method Tests - as_tuple
// ============================================================================

mod as_tuple_tests {
    use super::*;

    #[test]
    fn returns_both_paths() {
        let paths = DiffPaths::rename("old.rs", "new.rs");
        let (old, new) = paths.as_tuple();

        assert_eq!(old, Some("old.rs"));
        assert_eq!(new, Some("new.rs"));
    }

    #[test]
    fn handles_missing_old() {
        let paths = DiffPaths::creation("new.rs");
        let (old, new) = paths.as_tuple();

        assert!(old.is_none());
        assert_eq!(new, Some("new.rs"));
    }

    #[test]
    fn handles_missing_new() {
        let paths = DiffPaths::deletion("old.rs");
        let (old, new) = paths.as_tuple();

        assert_eq!(old, Some("old.rs"));
        assert!(new.is_none());
    }

    #[test]
    fn as_tuple_normalized() {
        let paths = DiffPaths::parse("--- a/old.rs\n+++ b/new.rs\n")
            .unwrap()
            .unwrap();
        let (old, new) = paths.as_tuple_normalized();

        assert_eq!(old, Some("old.rs"));
        assert_eq!(new, Some("new.rs"));
    }
}

// ============================================================================
// DiffPaths Method Tests - matches_path
// ============================================================================

mod matches_path_tests {
    use super::*;

    #[test]
    fn matches_old_path() {
        let paths = DiffPaths::modification("src/lib.rs");
        assert!(paths.matches_path("src/lib.rs"));
    }

    #[test]
    fn matches_new_path() {
        let paths = DiffPaths::rename("old.rs", "new.rs");
        assert!(paths.matches_path("new.rs"));
    }

    #[test]
    fn matches_with_diff_prefix() {
        let paths = DiffPaths::parse("--- a/src/lib.rs\n+++ b/src/lib.rs\n")
            .unwrap()
            .unwrap();
        assert!(paths.matches_path("src/lib.rs"));
    }

    #[test]
    fn does_not_match_different_path() {
        let paths = DiffPaths::modification("src/lib.rs");
        assert!(!paths.matches_path("other.rs"));
    }

    #[test]
    fn matches_with_query_prefix() {
        let paths = DiffPaths::parse("--- a/file.rs\n").unwrap().unwrap();
        assert!(paths.matches_path("a/file.rs"));
    }
}

// ============================================================================
// DiffPaths Method Tests - path_ends_with
// ============================================================================

mod path_ends_with_tests {
    use super::*;

    #[test]
    fn matches_extension() {
        let paths = DiffPaths::modification("src/lib.rs");
        assert!(paths.path_ends_with(".rs"));
    }

    #[test]
    fn matches_directory() {
        let paths = DiffPaths::modification("src/module/lib.rs");
        assert!(paths.path_ends_with("module/lib.rs"));
    }

    #[test]
    fn does_not_match_different_suffix() {
        let paths = DiffPaths::modification("src/lib.rs");
        assert!(!paths.path_ends_with(".txt"));
    }

    #[test]
    fn matches_new_path() {
        let paths = DiffPaths::rename("old.txt", "new.rs");
        assert!(paths.path_ends_with(".rs"));
    }

    #[test]
    fn matches_old_path() {
        let paths = DiffPaths::rename("old.rs", "new.txt");
        assert!(paths.path_ends_with(".rs"));
    }
}

// ============================================================================
// DiffPaths Method Tests - is_binary
// ============================================================================

mod is_binary_tests {
    use super::*;

    #[test]
    fn is_binary_always_false() {
        let paths = DiffPaths::modification("image.png");
        assert!(!paths.is_binary());
    }
}

// ============================================================================
// Convenience Function Tests - extract_paths_from_header
// ============================================================================

mod extract_paths_from_header_tests {
    use super::*;

    #[test]
    fn extracts_paths() {
        let header = "--- a/src/lib.rs\n+++ b/src/lib.rs\n";
        let paths = extract_paths_from_header(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/src/lib.rs".to_string()));
        assert_eq!(paths.new_path, Some("b/src/lib.rs".to_string()));
    }

    #[test]
    fn returns_none_for_no_headers() {
        let result = extract_paths_from_header("no headers here").unwrap();
        assert!(result.is_none());
    }
}

// ============================================================================
// Convenience Function Tests - extract_normalized_paths
// ============================================================================

mod extract_normalized_paths_tests {
    use super::*;

    #[test]
    fn extracts_and_normalizes() {
        let header = "--- a/src/lib.rs\n+++ b/src/lib.rs\n";
        let paths = extract_normalized_paths(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("src/lib.rs".to_string()));
        assert_eq!(paths.new_path, Some("src/lib.rs".to_string()));
    }

    #[test]
    fn handles_creation() {
        let header = "--- /dev/null\n+++ b/new.rs\n";
        let paths = extract_normalized_paths(header).unwrap().unwrap();

        assert!(paths.is_creation());
        assert_eq!(paths.new_path, Some("new.rs".to_string()));
    }
}

// ============================================================================
// Convenience Function Tests - parse_old_line
// ============================================================================

mod parse_old_line_tests {
    use super::*;

    #[test]
    fn parses_standard_line() {
        let (path, ts) = parse_old_line("--- a/src/lib.rs");
        assert_eq!(path, Some("a/src/lib.rs"));
        assert!(ts.is_none());
    }

    #[test]
    fn parses_with_timestamp() {
        let (path, ts) = parse_old_line("--- a/file.rs\t2024-01-01");
        assert_eq!(path, Some("a/file.rs"));
        assert_eq!(ts, Some("2024-01-01"));
    }

    #[test]
    fn returns_none_for_non_header() {
        let (path, ts) = parse_old_line("not a header");
        assert!(path.is_none());
        assert!(ts.is_none());
    }

    #[test]
    fn handles_whitespace() {
        let (path, _) = parse_old_line("  --- a/file.rs  ");
        assert_eq!(path, Some("a/file.rs"));
    }

    #[test]
    fn empty_after_prefix() {
        let (path, ts) = parse_old_line("---");
        assert!(path.is_none());
        assert!(ts.is_none());
    }
}

// ============================================================================
// Convenience Function Tests - parse_new_line
// ============================================================================

mod parse_new_line_tests {
    use super::*;

    #[test]
    fn parses_standard_line() {
        let (path, ts) = parse_new_line("+++ b/src/lib.rs");
        assert_eq!(path, Some("b/src/lib.rs"));
        assert!(ts.is_none());
    }

    #[test]
    fn parses_with_timestamp() {
        let (path, ts) = parse_new_line("+++ b/file.rs\t2024-01-01");
        assert_eq!(path, Some("b/file.rs"));
        assert_eq!(ts, Some("2024-01-01"));
    }

    #[test]
    fn returns_none_for_non_header() {
        let (path, ts) = parse_new_line("not a header");
        assert!(path.is_none());
        assert!(ts.is_none());
    }

    #[test]
    fn handles_whitespace() {
        let (path, _) = parse_new_line("  +++ b/file.rs  ");
        assert_eq!(path, Some("b/file.rs"));
    }
}

// ============================================================================
// Edge Cases - Special Characters
// ============================================================================

mod special_characters_tests {
    use super::*;

    #[test]
    fn handles_spaces_in_path() {
        let header = "--- a/src/my file.rs\n+++ b/src/my file.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/src/my file.rs".to_string()));
    }

    #[test]
    fn handles_unicode_in_path() {
        let header = "--- a/src/日本語.rs\n+++ b/src/日本語.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/src/日本語.rs".to_string()));
    }

    #[test]
    fn handles_dots_in_path() {
        let header = "--- a/src/../lib.rs\n+++ b/src/../lib.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/src/../lib.rs".to_string()));
    }

    #[test]
    fn handles_special_regex_chars() {
        let header = "--- a/file[test].rs\n+++ b/file[test].rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/file[test].rs".to_string()));
    }

    #[test]
    fn handles_parentheses() {
        let header = "--- a/file (copy).rs\n+++ b/file (copy).rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/file (copy).rs".to_string()));
    }
}

// ============================================================================
// Edge Cases - Empty and Minimal
// ============================================================================

mod edge_cases_tests {
    use super::*;

    #[test]
    fn empty_path_after_prefix() {
        let header = "--- a/\n+++ b/\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/".to_string()));
        assert_eq!(paths.new_path, Some("b/".to_string()));
    }

    #[test]
    fn single_character_filename() {
        let header = "--- a/a\n+++ b/b\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path, Some("a/a".to_string()));
    }

    #[test]
    fn deeply_nested_path() {
        let header = "--- a/a/b/c/d/e/f/g/h/file.rs\n+++ b/a/b/c/d/e/f/g/h/file.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert_eq!(paths.old_path_normalized(), Some("a/b/c/d/e/f/g/h/file.rs"));
    }

    #[test]
    fn very_long_path() {
        let long_path = "a/".repeat(100) + "file.rs";
        let header = format!("--- {}\n+++ {}\n", long_path, long_path);
        let paths = DiffPaths::parse(&header).unwrap().unwrap();

        assert!(paths.old_path.is_some());
    }
}

// ============================================================================
// Clone and Debug Tests
// ============================================================================

mod trait_tests {
    use super::*;

    #[test]
    fn clone_works() {
        let paths = DiffPaths::modification("src/lib.rs");
        let cloned = paths.clone();

        assert_eq!(paths, cloned);
    }

    #[test]
    fn debug_works() {
        let paths = DiffPaths::modification("src/lib.rs");
        let debug_str = format!("{:?}", paths);

        assert!(debug_str.contains("src/lib.rs"));
    }

    #[test]
    fn partial_eq_works() {
        let paths1 = DiffPaths::modification("src/lib.rs");
        let paths2 = DiffPaths::modification("src/lib.rs");
        let paths3 = DiffPaths::modification("other.rs");

        assert_eq!(paths1, paths2);
        assert_ne!(paths1, paths3);
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn strip_prefix_never_panics(path in ".*", _prefix in ".*") {
            let _ = strip_diff_prefix(&path);
        }

        #[test]
        fn is_dev_null_consistent(s in ".*") {
            let result = is_dev_null(&s);
            prop_assert_eq!(result, s == "/dev/null" || s == "dev/null");
        }

        #[test]
        fn normalize_path_never_panics(path in ".*") {
            let _ = normalize_path(&path);
        }

        #[test]
        fn parse_never_panics(header in ".*") {
            let _ = DiffPaths::parse(&header);
        }

        #[test]
        fn strip_diff_prefix_idempotent_after_first_strip(path in "[^abi].*") {
            // If path doesn't start with a/, b/, or i/, stripping is idempotent
            let first = strip_diff_prefix(&path);
            let second = strip_diff_prefix(first);
            prop_assert_eq!(first, second);
        }

        #[test]
        fn creation_is_not_deletion(path in ".*") {
            let paths = DiffPaths::creation(&path);
            prop_assert!(paths.is_creation());
            prop_assert!(!paths.is_deletion());
        }

        #[test]
        fn deletion_is_not_creation(path in ".*") {
            let paths = DiffPaths::deletion(&path);
            prop_assert!(paths.is_deletion());
            prop_assert!(!paths.is_creation());
        }

        #[test]
        fn modification_is_not_rename_nor_create_nor_delete(path in ".*") {
            let paths = DiffPaths::modification(&path);
            prop_assert!(paths.is_modification());
            prop_assert!(!paths.is_rename());
            prop_assert!(!paths.is_creation());
            prop_assert!(!paths.is_deletion());
        }

        #[test]
        fn canonical_path_returns_some_for_valid_paths(old in ".*", new in ".*") {
            let paths = DiffPaths {
                old_path: if old.is_empty() { None } else { Some(old.clone()) },
                new_path: if new.is_empty() { None } else { Some(new.clone()) },
                old_timestamp: None,
                new_timestamp: None,
            };

            if paths.old_path.is_some() || paths.new_path.is_some() {
                // Should have a canonical path unless both are dev/null
                let has_canonical = paths.canonical_path().is_some()
                    || (paths.old_path.as_deref() == Some("/dev/null")
                        && paths.new_path.as_deref() == Some("/dev/null"));
                prop_assert!(has_canonical || paths.canonical_path().is_none());
            }
        }

        #[test]
        fn strip_diff_prefixes_is_idempotent_on_normalized(
            old_path in "a/.*",
            new_path in "b/.*"
        ) {
            let paths = DiffPaths {
                old_path: Some(old_path),
                new_path: Some(new_path),
                old_timestamp: None,
                new_timestamp: None,
            };

            let normalized = paths.strip_diff_prefixes();
            let double_normalized = normalized.strip_diff_prefixes();

            prop_assert_eq!(normalized.old_path, double_normalized.old_path);
            prop_assert_eq!(normalized.new_path, double_normalized.new_path);
        }
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn full_workflow_modification() {
        let header = "--- a/src/lib.rs\n+++ b/src/lib.rs\n";
        let paths = extract_normalized_paths(header).unwrap().unwrap();

        assert!(paths.is_modification());
        assert!(!paths.is_creation());
        assert!(!paths.is_deletion());
        assert!(!paths.is_rename());
        assert_eq!(paths.canonical_path(), Some("src/lib.rs"));
        assert!(paths.matches_path("src/lib.rs"));
        assert!(paths.path_ends_with(".rs"));
    }

    #[test]
    fn full_workflow_creation() {
        let header = "--- /dev/null\n+++ b/src/new_file.rs\n";
        let paths = extract_normalized_paths(header).unwrap().unwrap();

        assert!(paths.is_creation());
        assert!(!paths.is_modification());
        assert_eq!(paths.canonical_path(), Some("src/new_file.rs"));
    }

    #[test]
    fn full_workflow_deletion() {
        let header = "--- a/src/deleted_file.rs\n+++ /dev/null\n";
        let paths = extract_normalized_paths(header).unwrap().unwrap();

        assert!(paths.is_deletion());
        assert!(!paths.is_modification());
        assert_eq!(paths.canonical_path(), Some("src/deleted_file.rs"));
    }

    #[test]
    fn full_workflow_rename() {
        let header = "--- a/src/old_name.rs\n+++ b/src/new_name.rs\n";
        let paths = extract_normalized_paths(header).unwrap().unwrap();

        assert!(paths.is_rename());
        assert!(!paths.is_modification());
        assert_eq!(paths.canonical_path(), Some("src/new_name.rs"));
        let (old, new) = paths.as_tuple();
        assert_eq!(old, Some("src/old_name.rs"));
        assert_eq!(new, Some("src/new_name.rs"));
    }

    #[test]
    fn real_world_git_diff_header() {
        // Real git diff format
        let header = concat!(
            "--- a/crates/lintdiff-diff-paths/src/lib.rs\n",
            "+++ b/crates/lintdiff-diff-paths/src/lib.rs\n"
        );
        let paths = extract_normalized_paths(header).unwrap().unwrap();

        assert!(paths.is_modification());
        assert_eq!(
            paths.canonical_path(),
            Some("crates/lintdiff-diff-paths/src/lib.rs")
        );
    }

    #[test]
    fn real_world_git_diff_with_timestamps() {
        // Git diff with commit timestamps
        let header = concat!(
            "--- a/file.rs\t2024-01-15 10:30:00.000000000 +0000\n",
            "+++ b/file.rs\t2024-01-15 10:35:00.000000000 +0000\n"
        );
        let paths = DiffPaths::parse(header).unwrap().unwrap();

        assert!(paths.is_modification());
        assert!(paths.old_timestamp.is_some());
        assert!(paths.new_timestamp.is_some());
    }
}

// ============================================================================
// Must Use Tests (compile-time check)
// ============================================================================

mod must_use_tests {
    use super::*;

    #[test]
    fn diff_paths_methods_are_must_use() {
        // These would produce warnings if not marked #[must_use]
        let paths = DiffPaths::modification("test.rs");
        let _ = paths.is_creation();
        let _ = paths.is_deletion();
        let _ = paths.is_rename();
        let _ = paths.is_modification();
        let _ = paths.canonical_path();
        let _ = paths.old_path_normalized();
        let _ = paths.new_path_normalized();
        let _ = paths.canonical_path_normalized();
        let _ = paths.strip_prefix("a/");
        let _ = paths.strip_diff_prefixes();
        let _ = paths.as_tuple();
        let _ = paths.as_tuple_normalized();
        let _ = paths.matches_path("test");
        let _ = paths.path_ends_with(".rs");
        let _ = paths.is_binary();
    }

    #[test]
    fn functions_are_must_use() {
        let _ = is_dev_null("/dev/null");
        let _ = strip_diff_prefix("a/test");
        let _ = normalize_path("a/test");
        let _ = normalize_path_owned("a/test");
        let _ = parse_old_line("--- a/test");
        let _ = parse_new_line("+++ b/test");
    }
}
