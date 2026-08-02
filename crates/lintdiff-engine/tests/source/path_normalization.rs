//! Comprehensive tests for file path handling and normalization in diffs.
//!
//! This module tests how the diff parser handles various path formats and normalization.

use lintdiff_engine::parse_unified_diff;
use lintdiff_types::{LineRange, NormPath};

// =============================================================================
// Basic path extraction tests
// =============================================================================

#[test]
fn standard_diff_git_paths() {
    let diff = r#"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map.changed.contains_key(&NormPath::new("src/lib.rs")));
}

#[test]
fn paths_with_a_and_b_prefixes_stripped() {
    let diff = r#"
diff --git a/path/to/file.rs b/path/to/file.rs
--- a/path/to/file.rs
+++ b/path/to/file.rs
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    // The a/ and b/ prefixes should be stripped
    assert!(map.changed.contains_key(&NormPath::new("path/to/file.rs")));
}

#[test]
fn paths_from_diff_header_used_as_fallback() {
    // When --- and +++ lines are missing, paths come from diff --git line
    let diff = r#"
diff --git a/fallback.rs b/fallback.rs
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map.changed.contains_key(&NormPath::new("fallback.rs")));
}

// =============================================================================
// Path normalization tests
// =============================================================================

#[test]
fn backslashes_converted_to_forward_slashes() {
    // Windows-style paths should be normalized
    let diff = r#"
diff --git a/src\windows\path.rs b/src\windows\path.rs
--- a/src\windows\path.rs
+++ b/src\windows\path.rs
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    // Should normalize to forward slashes
    assert!(map
        .changed
        .contains_key(&NormPath::new("src/windows/path.rs")));
}

#[test]
fn leading_dot_slash_stripped() {
    let diff = r#"
diff --git a/./relative/path.rs b/./relative/path.rs
--- a/./relative/path.rs
+++ b/./relative/path.rs
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    // The ./ should be stripped during normalization
    assert!(map.changed.contains_key(&NormPath::new("relative/path.rs")));
}

#[test]
fn double_slashes_collapsed() {
    let diff = r#"
diff --git a/src//double///slash.rs b/src//double///slash.rs
--- a/src//double///slash.rs
+++ b/src//double///slash.rs
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    // Multiple slashes should be collapsed
    assert!(map
        .changed
        .contains_key(&NormPath::new("src/double/slash.rs")));
}

#[test]
fn whitespace_trimmed_from_paths() {
    let diff = r#"
diff --git a/  spaced/path.rs  b/  spaced/path.rs  
--- a/  spaced/path.rs  
+++ b/  spaced/path.rs  
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    // Whitespace should be trimmed
    assert!(map.changed.contains_key(&NormPath::new("spaced/path.rs")));
}

// =============================================================================
// Special path cases
// =============================================================================

#[test]
fn dev_null_for_new_file() {
    let diff = r#"
diff --git a/new_file.rs b/new_file.rs
new file mode 100644
--- /dev/null
+++ b/new_file.rs
@@ -0,0 +1,1 @@
+new content
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map.changed.contains_key(&NormPath::new("new_file.rs")));
}

#[test]
fn dev_null_for_deleted_file() {
    let diff = r#"
diff --git a/deleted_file.rs b/deleted_file.rs
deleted file mode 100644
--- a/deleted_file.rs
+++ /dev/null
@@ -1,1 +0,0 @@
-old content
"#;

    let map = parse_unified_diff(diff).unwrap();
    // Deleted files have no new-side changes
    assert!(!map.changed.contains_key(&NormPath::new("deleted_file.rs")));
}

#[test]
fn file_in_root_directory() {
    let diff = r#"
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map.changed.contains_key(&NormPath::new("Cargo.toml")));
}

#[test]
fn hidden_files() {
    let diff = r#"
diff --git a/.gitignore b/.gitignore
--- a/.gitignore
+++ b/.gitignore
@@ -1,0 +1,1 @@
+*.rs.bk
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map.changed.contains_key(&NormPath::new(".gitignore")));
}

#[test]
fn dotfiles_in_subdirectories() {
    let diff = r#"
diff --git a/config/.env b/config/.env
--- a/config/.env
+++ b/config/.env
@@ -1,0 +1,1 @@
+KEY=value
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map.changed.contains_key(&NormPath::new("config/.env")));
}

// =============================================================================
// Rename handling tests
// =============================================================================

#[test]
fn rename_from_to_captures_paths() {
    let diff = r#"
diff --git a/old_name.rs b/new_name.rs
similarity index 100%
rename from old_name.rs
rename to new_name.rs
"#;

    let map = parse_unified_diff(diff).unwrap();

    // Should track the rename mapping
    assert_eq!(
        map.renames.get(&NormPath::new("old_name.rs")),
        Some(&NormPath::new("new_name.rs"))
    );
}

#[test]
fn rename_with_content_changes() {
    let diff = r#"
diff --git a/old.rs b/new.rs
rename from old.rs
rename to new.rs
--- a/old.rs
+++ b/new.rs
@@ -5,0 +5,1 @@
+// New comment
"#;

    let map = parse_unified_diff(diff).unwrap();

    // Should track both the rename and the changes
    assert_eq!(
        map.renames.get(&NormPath::new("old.rs")),
        Some(&NormPath::new("new.rs"))
    );

    let ranges = map.changed.get(&NormPath::new("new.rs")).unwrap();
    assert_eq!(ranges, &vec![LineRange::new(5, 5)]);
}

#[test]
fn rename_paths_override_diff_header_paths() {
    let diff = r#"
diff --git a/ambiguous b/ambiguous
rename from actual_old_name.txt
rename to actual_new_name.txt
"#;

    let map = parse_unified_diff(diff).unwrap();

    // Rename paths should be used over diff --git paths
    assert_eq!(
        map.renames.get(&NormPath::new("actual_old_name.txt")),
        Some(&NormPath::new("actual_new_name.txt"))
    );
}

// =============================================================================
// Complex path scenarios
// =============================================================================

#[test]
fn paths_with_spaces_not_supported() {
    // Note: Git quotes paths with spaces, but our parser doesn't unquote them.
    // This is a known limitation - paths with spaces require special handling.
    // The parser will still process the file, but the path extraction may not be perfect.
    let diff = r#"
diff --git "a/src/my file.rs" "b/src/my file.rs"
--- "a/src/my file.rs"
+++ "b/src/my file.rs"
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    // The parser processes the file (stats should show 1 file)
    // but the path extraction with quotes is a known limitation
    assert!(map.stats.files >= 1);
}

#[test]
fn paths_with_dashes_and_underscores() {
    // Test paths with common special characters that don't require quoting
    let diff = r#"
diff --git a/src/file-name_1.rs b/src/file-name_1.rs
--- a/src/file-name_1.rs
+++ b/src/file-name_1.rs
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map
        .changed
        .contains_key(&NormPath::new("src/file-name_1.rs")));
}

#[test]
fn paths_with_unicode() {
    // Unicode paths that don't require quoting work fine
    let diff = r#"
diff --git a/src/日本語/ファイル.rs b/src/日本語/ファイル.rs
--- a/src/日本語/ファイル.rs
+++ b/src/日本語/ファイル.rs
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map
        .changed
        .contains_key(&NormPath::new("src/日本語/ファイル.rs")));
}

#[test]
fn paths_with_at_symbol() {
    // Test paths with @ symbol (common in code)
    let diff = r#"
diff --git a/src/@types/index.d.ts b/src/@types/index.d.ts
--- a/src/@types/index.d.ts
+++ b/src/@types/index.d.ts
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map
        .changed
        .contains_key(&NormPath::new("src/@types/index.d.ts")));
}

#[test]
fn very_long_paths() {
    let long_path = "a/".repeat(50) + "file.rs";
    let diff = format!(
        r#"
diff --git "a/{path}" "b/{path}"
--- "a/{path}"
+++ "b/{path}"
@@ -1,0 +1,1 @@
+content
"#,
        path = long_path
    );

    let map = parse_unified_diff(&diff).unwrap();
    // Should handle very long paths - just verify it doesn't crash
    assert!(map.stats.files >= 1);
}

// =============================================================================
// Multiple files with various path formats
// =============================================================================

#[test]
fn multiple_files_with_different_path_depths() {
    let diff = r#"
diff --git a/root.txt b/root.txt
--- a/root.txt
+++ b/root.txt
@@ -1,0 +1,1 @@
+root
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,1 @@
+lib
diff --git a/src/deep/nested/module.rs b/src/deep/nested/module.rs
--- a/src/deep/nested/module.rs
+++ b/src/deep/nested/module.rs
@@ -1,0 +1,1 @@
+module
"#;

    let map = parse_unified_diff(diff).unwrap();

    assert!(map.changed.contains_key(&NormPath::new("root.txt")));
    assert!(map.changed.contains_key(&NormPath::new("src/lib.rs")));
    assert!(map
        .changed
        .contains_key(&NormPath::new("src/deep/nested/module.rs")));

    assert_eq!(map.stats.files, 3);
}

#[test]
fn files_with_similar_names_in_different_directories() {
    let diff = r#"
diff --git a/mod.rs b/mod.rs
--- a/mod.rs
+++ b/mod.rs
@@ -1,0 +1,1 @@
+root mod
diff --git a/src/mod.rs b/src/mod.rs
--- a/src/mod.rs
+++ b/src/mod.rs
@@ -1,0 +1,1 @@
+src mod
diff --git a/src/sub/mod.rs b/src/sub/mod.rs
--- a/src/sub/mod.rs
+++ b/src/sub/mod.rs
@@ -1,0 +1,1 @@
+sub mod
"#;

    let map = parse_unified_diff(diff).unwrap();

    // Each should be tracked separately
    assert!(map.changed.contains_key(&NormPath::new("mod.rs")));
    assert!(map.changed.contains_key(&NormPath::new("src/mod.rs")));
    assert!(map.changed.contains_key(&NormPath::new("src/sub/mod.rs")));
}

// =============================================================================
// Path consistency tests
// =============================================================================

#[test]
fn same_path_different_prefixes_normalized_equally() {
    // Both a/ and b/ prefixed paths should normalize to the same thing
    let norm1 = NormPath::new("a/src/file.rs");
    let norm2 = NormPath::new("b/src/file.rs");
    let norm3 = NormPath::new("src/file.rs");

    assert_eq!(norm1, norm3);
    assert_eq!(norm2, norm3);
}

#[test]
fn path_normalization_is_idempotent() {
    let path = "src///complex//path.rs";
    let norm1 = NormPath::new(path);
    let norm2 = NormPath::new(norm1.as_str());

    assert_eq!(norm1, norm2);
}

// =============================================================================
// Edge cases for path handling
// =============================================================================

#[test]
fn empty_path_components_handled() {
    let diff = r#"
diff --git a/src//file.rs b/src//file.rs
--- a/src//file.rs
+++ b/src//file.rs
@@ -1,0 +1,1 @@
+content
"#;

    let map = parse_unified_diff(diff).unwrap();
    // Should normalize the path
    assert!(map.changed.contains_key(&NormPath::new("src/file.rs")));
}

#[test]
fn file_extension_with_multiple_dots() {
    let diff = r#"
diff --git a/config.local.json b/config.local.json
--- a/config.local.json
+++ b/config.local.json
@@ -1,0 +1,1 @@
+{"key": "value"}
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map
        .changed
        .contains_key(&NormPath::new("config.local.json")));
}

#[test]
fn no_file_extension() {
    let diff = r#"
diff --git a/Makefile b/Makefile
--- a/Makefile
+++ b/Makefile
@@ -1,0 +1,1 @@
+build:
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map.changed.contains_key(&NormPath::new("Makefile")));
}

#[test]
fn filename_starting_with_dot() {
    let diff = r#"
diff --git a/.hidden b/.hidden
--- a/.hidden
+++ b/.hidden
@@ -1,0 +1,1 @@
+hidden content
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map.changed.contains_key(&NormPath::new(".hidden")));
}

#[test]
fn directory_starting_with_dot() {
    let diff = r#"
diff --git a/.github/workflows/ci.yml b/.github/workflows/ci.yml
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ -1,0 +1,1 @@
+name: CI
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map
        .changed
        .contains_key(&NormPath::new(".github/workflows/ci.yml")));
}

// =============================================================================
// Rename edge cases
// =============================================================================

#[test]
fn rename_to_same_name_not_tracked() {
    // If old and new paths are the same, it shouldn't be in renames
    let diff = r#"
diff --git a/file.rs b/file.rs
rename from file.rs
rename to file.rs
"#;

    let map = parse_unified_diff(diff).unwrap();
    // Same path shouldn't be in renames
    assert!(!map.renames.contains_key(&NormPath::new("file.rs")));
}

#[test]
fn rename_directory_change() {
    let diff = r#"
diff --git a/old_dir/file.rs b/new_dir/file.rs
rename from old_dir/file.rs
rename to new_dir/file.rs
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert_eq!(
        map.renames.get(&NormPath::new("old_dir/file.rs")),
        Some(&NormPath::new("new_dir/file.rs"))
    );
}

#[test]
fn rename_with_extension_change() {
    let diff = r#"
diff --git a/file.txt b/file.md
rename from file.txt
rename to file.md
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert_eq!(
        map.renames.get(&NormPath::new("file.txt")),
        Some(&NormPath::new("file.md"))
    );
}
