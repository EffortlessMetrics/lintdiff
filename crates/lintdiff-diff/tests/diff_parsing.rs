//! Comprehensive tests for diff parsing functionality.
//!
//! This module tests the `parse_unified_diff` function with various input patterns.

use lintdiff_diff::parse_unified_diff;
use lintdiff_types::{LineRange, NormPath};

// =============================================================================
// Basic diff parsing tests
// =============================================================================

#[test]
fn single_file_single_hunk() {
    let diff = r#"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,3 @@
+fn a() {}
+fn b() {}
+fn c() {}
"#;

    let map = parse_unified_diff(diff).unwrap();
    let ranges = map.changed.get(&NormPath::new("src/lib.rs")).unwrap();
    assert_eq!(ranges, &vec![LineRange::new(1, 3)]);
    assert_eq!(map.stats.files, 1);
    assert_eq!(map.stats.hunks, 1);
    assert_eq!(map.stats.added_lines, 3);
}

#[test]
fn single_file_multiple_hunks() {
    let diff = r#"
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,0 +10,2 @@
+// First addition
+fn first() {}
@@ -50,0 +52,3 @@
+// Second addition
+fn second() {}
+fn third() {}
"#;

    let map = parse_unified_diff(diff).unwrap();
    let ranges = map.changed.get(&NormPath::new("src/main.rs")).unwrap();

    // Should have two separate ranges
    assert_eq!(ranges.len(), 2);
    assert_eq!(ranges[0], LineRange::new(10, 11));
    assert_eq!(ranges[1], LineRange::new(52, 54));
    assert_eq!(map.stats.files, 1);
    assert_eq!(map.stats.hunks, 2);
    assert_eq!(map.stats.added_lines, 5);
}

#[test]
fn multiple_files() {
    let diff = r#"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,2 @@
+fn a() {}
+fn b() {}
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -5,0 +5,1 @@
+fn main() {}
"#;

    let map = parse_unified_diff(diff).unwrap();

    let lib_ranges = map.changed.get(&NormPath::new("src/lib.rs")).unwrap();
    assert_eq!(lib_ranges, &vec![LineRange::new(1, 2)]);

    let main_ranges = map.changed.get(&NormPath::new("src/main.rs")).unwrap();
    assert_eq!(main_ranges, &vec![LineRange::new(5, 5)]);

    assert_eq!(map.stats.files, 2);
    assert_eq!(map.stats.hunks, 2);
    assert_eq!(map.stats.added_lines, 3);
}

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
fn empty_diff() {
    let diff = "";
    let map = parse_unified_diff(diff).unwrap();
    assert!(map.changed.is_empty());
    assert_eq!(map.stats.files, 0);
    assert_eq!(map.stats.hunks, 0);
    assert_eq!(map.stats.added_lines, 0);
}

#[test]
fn diff_with_only_whitespace() {
    let diff = "   \n\n   \n";
    let map = parse_unified_diff(diff).unwrap();
    assert!(map.changed.is_empty());
}

#[test]
fn binary_files_are_skipped() {
    // Binary files typically show as:
    // diff --git a/binary.bin b/binary.bin
    // Binary files a/binary.bin and b/binary.bin differ
    let diff = r#"
diff --git a/image.png b/image.png
Binary files a/image.png and b/image.png differ
"#;

    let map = parse_unified_diff(diff).unwrap();
    // Binary files have no hunks, so no changed lines
    assert!(map.changed.is_empty() || !map.changed.contains_key(&NormPath::new("image.png")));
}

#[test]
fn deleted_file_no_new_side_lines() {
    let diff = r#"
diff --git a/deleted.rs b/deleted.rs
deleted file mode 100644
--- a/deleted.rs
+++ /dev/null
@@ -1,3 +0,0 @@
-fn a() {}
-fn b() {}
-fn c() {}
"#;

    let map = parse_unified_diff(diff).unwrap();
    // Deleted files have /dev/null as new path, so no changed lines on new side
    assert!(!map.changed.contains_key(&NormPath::new("deleted.rs")));
}

#[test]
fn new_file_creation() {
    let diff = r#"
diff --git a/new_file.rs b/new_file.rs
new file mode 100644
--- /dev/null
+++ b/new_file.rs
@@ -0,0 +1,3 @@
+fn new() {}
+fn functions() {}
+fn here() {}
"#;

    let map = parse_unified_diff(diff).unwrap();
    let ranges = map.changed.get(&NormPath::new("new_file.rs")).unwrap();
    assert_eq!(ranges, &vec![LineRange::new(1, 3)]);
}

#[test]
fn renamed_file() {
    let diff = r#"
diff --git a/old_name.rs b/new_name.rs
similarity index 100%
rename from old_name.rs
rename to new_name.rs
"#;

    let map = parse_unified_diff(diff).unwrap();

    // Should track the rename
    assert!(map.renames.contains_key(&NormPath::new("old_name.rs")));
    assert_eq!(
        map.renames.get(&NormPath::new("old_name.rs")),
        Some(&NormPath::new("new_name.rs"))
    );

    // No changed lines for pure renames
    assert!(!map.changed.contains_key(&NormPath::new("new_name.rs")));
}

#[test]
fn renamed_file_with_changes() {
    let diff = r#"
diff --git a/old.rs b/new.rs
rename from old.rs
rename to new.rs
--- a/old.rs
+++ b/new.rs
@@ -5,0 +5,1 @@
+// Added comment
"#;

    let map = parse_unified_diff(diff).unwrap();

    // Should track the rename
    assert_eq!(
        map.renames.get(&NormPath::new("old.rs")),
        Some(&NormPath::new("new.rs"))
    );

    // Should also track the changes under the new path
    let ranges = map.changed.get(&NormPath::new("new.rs")).unwrap();
    assert_eq!(ranges, &vec![LineRange::new(5, 5)]);
}

#[test]
fn special_characters_in_paths() {
    // Note: Git quotes paths with special characters, but our parser doesn't unquote them.
    // This test uses a path without quotes that still has special characters.
    let diff = r#"
diff --git a/src/file-with-dash.rs b/src/file-with-dash.rs
--- a/src/file-with-dash.rs
+++ b/src/file-with-dash.rs
@@ -1,0 +1,1 @@
+fn test() {}
"#;

    let map = parse_unified_diff(diff).unwrap();
    // Path normalization should handle paths with dashes
    assert!(map
        .changed
        .contains_key(&NormPath::new("src/file-with-dash.rs")));
}

#[test]
fn unicode_in_paths() {
    let diff = r#"
diff --git a/src/日本語.rs b/src/日本語.rs
--- a/src/日本語.rs
+++ b/src/日本語.rs
@@ -1,0 +1,1 @@
+fn test() {}
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert!(map.changed.contains_key(&NormPath::new("src/日本語.rs")));
}

#[test]
fn deeply_nested_paths() {
    let diff = r#"
diff --git a/src/deep/nested/path/to/module.rs b/src/deep/nested/path/to/module.rs
--- a/src/deep/nested/path/to/module.rs
+++ b/src/deep/nested/path/to/module.rs
@@ -1,0 +1,1 @@
+fn deep() {}
"#;

    let map = parse_unified_diff(diff).unwrap();
    // Path after stripping a/ prefix
    assert!(map
        .changed
        .contains_key(&NormPath::new("src/deep/nested/path/to/module.rs")));
}

// =============================================================================
// Line range calculation tests
// =============================================================================

#[test]
fn context_lines_not_counted_as_changed() {
    let diff = r#"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,3 +1,4 @@
 existing line 1
 existing line 2
+new line
 existing line 3
"#;

    let map = parse_unified_diff(diff).unwrap();
    let ranges = map.changed.get(&NormPath::new("src/lib.rs")).unwrap();

    // Only the added line should be in the range
    assert_eq!(ranges, &vec![LineRange::new(3, 3)]);
}

#[test]
fn mixed_additions_and_deletions() {
    let diff = r#"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,4 +1,4 @@
-old line 1
 unchanged 1
+new line 1
 unchanged 2
-old line 2
+new line 2
"#;

    let map = parse_unified_diff(diff).unwrap();
    let ranges = map.changed.get(&NormPath::new("src/lib.rs")).unwrap();

    // Tracing the new-side line numbers:
    // - old line 1: old_line++, new_line unchanged (deletion)
    // - unchanged 1: old_line++, new_line++ (now at new line 1)
    // - new line 1: new_line++ (added at line 2)
    // - unchanged 2: old_line++, new_line++ (now at new line 3)
    // - old line 2: old_line++, new_line unchanged (deletion)
    // - new line 2: new_line++ (added at line 4)
    //
    // So added lines are at positions 2 and 4 (non-contiguous = 2 ranges)
    assert_eq!(ranges.len(), 2);
    assert!(ranges.contains(&LineRange::new(2, 2)));
    assert!(ranges.contains(&LineRange::new(4, 4)));
}

#[test]
fn contiguous_additions_merge_to_single_range() {
    let diff = r#"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,5 @@
+line 1
+line 2
+line 3
+line 4
+line 5
"#;

    let map = parse_unified_diff(diff).unwrap();
    let ranges = map.changed.get(&NormPath::new("src/lib.rs")).unwrap();

    // All 5 contiguous lines should be a single range
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0], LineRange::new(1, 5));
}

#[test]
fn non_contiguous_additions_create_separate_ranges() {
    let diff = r#"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,3 +1,5 @@
+new at start
 existing 1
 existing 2
+new in middle
 existing 3
+new at end
"#;

    let map = parse_unified_diff(diff).unwrap();
    let ranges = map.changed.get(&NormPath::new("src/lib.rs")).unwrap();

    // Lines 1, 4, and 6 should be in separate ranges (non-contiguous)
    assert_eq!(ranges.len(), 3);
    assert_eq!(ranges[0], LineRange::new(1, 1));
    assert_eq!(ranges[1], LineRange::new(4, 4));
    assert_eq!(ranges[2], LineRange::new(6, 6));
}

#[test]
fn large_line_numbers() {
    let diff = r#"
diff --git a/large.rs b/large.rs
--- a/large.rs
+++ b/large.rs
@@ -10000,0 +10000,2 @@
+fn large() {}
+fn file() {}
"#;

    let map = parse_unified_diff(diff).unwrap();
    let ranges = map.changed.get(&NormPath::new("large.rs")).unwrap();
    assert_eq!(ranges, &vec![LineRange::new(10000, 10001)]);
}

// =============================================================================
// Error handling tests
// =============================================================================

#[test]
fn malformed_hunk_header_missing_minus_segment() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ +1,2 @@
+line 1
"#;

    let result = parse_unified_diff(diff);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("missing '-' segment"));
}

#[test]
fn malformed_hunk_header_missing_plus_segment() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,2 @@
+line 1
"#;

    let result = parse_unified_diff(diff);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("missing '+' segment"));
}

#[test]
fn malformed_hunk_header_invalid_line_number() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -abc,1 +1,1 @@
+line 1
"#;

    let result = parse_unified_diff(diff);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("invalid old_start"));
}

#[test]
fn truncated_hunk_still_parses() {
    // A hunk that ends abruptly - parser should be forgiving
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,0 +1,2 @@
+line 1
"#;

    // Parser should still succeed with partial data
    let map = parse_unified_diff(diff).unwrap();
    let ranges = map.changed.get(&NormPath::new("test.rs")).unwrap();
    assert_eq!(ranges, &vec![LineRange::new(1, 1)]);
}

#[test]
fn no_newline_at_end_of_file_marker() {
    let diff = "diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,0 +1,1 @@
+line 1
\\ No newline at end of file";

    let map = parse_unified_diff(diff).unwrap();
    let ranges = map.changed.get(&NormPath::new("test.rs")).unwrap();
    assert_eq!(ranges, &vec![LineRange::new(1, 1)]);
}

#[test]
fn leading_junk_before_diff_is_ignored() {
    let diff = r#"
Some random output from git
More output
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,0 +1,1 @@
+line 1
"#;

    let map = parse_unified_diff(diff).unwrap();
    let ranges = map.changed.get(&NormPath::new("test.rs")).unwrap();
    assert_eq!(ranges, &vec![LineRange::new(1, 1)]);
}

// =============================================================================
// Stats tracking tests
// =============================================================================

#[test]
fn stats_track_multiple_files_correctly() {
    let diff = r#"
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,0 +1,1 @@
+a
diff --git a/b.rs b/b.rs
--- b/b.rs
+++ b/b.rs
@@ -1,0 +1,2 @@
+b1
+b2
diff --git a/c.rs b/c.rs
--- c/c.rs
+++ c/c.rs
@@ -1,0 +1,3 @@
+c1
+c2
+c3
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert_eq!(map.stats.files, 3);
    assert_eq!(map.stats.hunks, 3);
    assert_eq!(map.stats.added_lines, 6);
}

#[test]
fn stats_track_multiple_hunks_per_file() {
    let diff = r#"
diff --git a/multi.rs b/multi.rs
--- a/multi.rs
+++ b/multi.rs
@@ -10,0 +10,1 @@
+hunk1
@@ -20,0 +21,1 @@
+hunk2
@@ -30,0 +32,1 @@
+hunk3
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert_eq!(map.stats.files, 1);
    assert_eq!(map.stats.hunks, 3);
    assert_eq!(map.stats.added_lines, 3);
}

// =============================================================================
// Mode change tests
// =============================================================================

#[test]
fn mode_change_without_content() {
    let diff = r#"
diff --git a/script.sh b/script.sh
old mode 100644
new mode 100755
"#;

    let map = parse_unified_diff(diff).unwrap();
    // Mode changes without content changes have no hunks
    assert!(!map.changed.contains_key(&NormPath::new("script.sh")));
}

// =============================================================================
// Complex real-world-like diffs
// =============================================================================

#[test]
fn typical_feature_branch_diff() {
    let diff = r#"
diff --git a/src/lib.rs b/src/lib.rs
index 1234567..abcdefg 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -5,6 +5,10 @@ mod submodule;
 
 use std::collections::HashMap;
 
+/// New configuration struct
+pub struct Config {
+    pub debug: bool,
+}
+
 fn main() {
     println!("Hello");
 }
diff --git a/src/submodule.rs b/src/submodule.rs
--- a/src/submodule.rs
+++ b/src/submodule.rs
@@ -1,0 +1,5 @@
+//! Submodule documentation
+
+pub fn helper() -> i32 {
+    42
+}
"#;

    let map = parse_unified_diff(diff).unwrap();

    // lib.rs should have lines 8-11 (the new Config struct)
    let lib_ranges = map.changed.get(&NormPath::new("src/lib.rs")).unwrap();
    assert!(lib_ranges.iter().any(|r| r.start <= 11 && r.end >= 8));

    // submodule.rs should have lines 1-5
    let sub_ranges = map.changed.get(&NormPath::new("src/submodule.rs")).unwrap();
    assert_eq!(sub_ranges, &vec![LineRange::new(1, 5)]);

    assert_eq!(map.stats.files, 2);
}
