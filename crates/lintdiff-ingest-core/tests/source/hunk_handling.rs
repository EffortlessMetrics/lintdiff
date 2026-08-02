//! Comprehensive tests for hunk header parsing and handling.
//!
//! This module tests the `parse_hunk_header` function and related hunk processing logic.

use lintdiff_ingest_core::parse_unified_diff;
use lintdiff_types::{LineRange, NormPath};

// =============================================================================
// Hunk header parsing tests - testing the internal parse_hunk_header function
// =============================================================================

/// Helper to extract hunk ranges from a diff for testing
fn get_changed_ranges(diff: &str, path: &str) -> Vec<LineRange> {
    let map = parse_unified_diff(diff).unwrap();
    map.changed
        .get(&NormPath::new(path))
        .cloned()
        .unwrap_or_default()
}

// =============================================================================
// Standard hunk header formats
// =============================================================================

#[test]
fn standard_hunk_header_with_counts() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,5 +1,10 @@
 unchanged
+added
 unchanged
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    assert!(!ranges.is_empty());
    // The added line should be at line 2
    assert!(ranges.iter().any(|r| r.contains_line(2)));
}

#[test]
fn hunk_header_without_old_count() {
    // When old count is 1, it may be omitted
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1 +1,2 @@
 unchanged
+added
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    assert!(ranges.iter().any(|r| r.contains_line(2)));
}

#[test]
fn hunk_header_without_new_count() {
    // When new count is 1, it may be omitted
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,2 +1 @@
 unchanged
-deleted
"#;

    let map = parse_unified_diff(diff).unwrap();
    // No added lines, so no changed ranges
    assert!(!map.changed.contains_key(&NormPath::new("test.rs")));
}

#[test]
fn hunk_header_with_context_label() {
    // Hunk headers can have optional context after the numbers
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,3 +1,4 @@ fn main() {
 existing
+added
 existing
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    assert!(ranges.iter().any(|r| r.contains_line(2)));
}

#[test]
fn hunk_header_with_function_context() {
    // Git can include function context in hunk headers
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -10,6 +10,7 @@ pub fn example() {
     let x = 1;
+    let y = 2;
     let z = 3;
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    assert!(ranges.iter().any(|r| r.contains_line(11)));
}

// =============================================================================
// Edge cases for line numbers
// =============================================================================

#[test]
fn hunk_starting_at_line_zero_becomes_one() {
    // Line numbers should be clamped to minimum 1
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -0,0 +1,1 @@
+first line
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    assert_eq!(ranges, vec![LineRange::new(1, 1)]);
}

#[test]
fn hunk_with_large_line_numbers() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -999999,1 +999999,1 @@
+large line number
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    assert_eq!(ranges, vec![LineRange::new(999999, 999999)]);
}

#[test]
fn multiple_hunks_with_gap() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -10,0 +10,1 @@
+hunk 1
@@ -100,0 +101,1 @@
+hunk 2
@@ -500,0 +502,1 @@
+hunk 3
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    assert_eq!(ranges.len(), 3);
    assert!(ranges.contains(&LineRange::new(10, 10)));
    assert!(ranges.contains(&LineRange::new(101, 101)));
    assert!(ranges.contains(&LineRange::new(502, 502)));
}

// =============================================================================
// Hunk content handling
// =============================================================================

#[test]
fn hunk_with_only_additions() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -0,0 +1,5 @@
+line 1
+line 2
+line 3
+line 4
+line 5
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    assert_eq!(ranges, vec![LineRange::new(1, 5)]);
}

#[test]
fn hunk_with_only_deletions() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,3 +0,0 @@
-line 1
-line 2
-line 3
"#;

    let map = parse_unified_diff(diff).unwrap();
    // Deletions only - no new-side changed lines
    assert!(!map.changed.contains_key(&NormPath::new("test.rs")));
}

#[test]
fn hunk_with_only_context() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,3 +1,3 @@
 context 1
 context 2
 context 3
"#;

    let map = parse_unified_diff(diff).unwrap();
    // No additions - no changed lines
    assert!(!map.changed.contains_key(&NormPath::new("test.rs")));
}

#[test]
fn hunk_with_mixed_content() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,6 +1,6 @@
 context before
-deleted line
+added line 1
+added line 2
 context middle
-deleted line 2
+added line 3
 context after
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    // Tracing new-side line numbers:
    // - context before: old=1, new=1 (context, both advance)
    // - deleted line: old=2 (deletion, only old advances)
    // - added line 1: new=2 (addition, only new advances)
    // - added line 2: new=3 (addition, only new advances)
    // - context middle: old=3, new=4 (context, both advance)
    // - deleted line 2: old=4 (deletion, only old advances)
    // - added line 3: new=5 (addition, only new advances)
    // - context after: old=5, new=6 (context, both advance)
    //
    // Added lines are at new positions 2, 3, and 5
    assert!(ranges.iter().any(|r| r.contains_line(2)));
    assert!(ranges.iter().any(|r| r.contains_line(3)));
    assert!(ranges.iter().any(|r| r.contains_line(5)));
}

// =============================================================================
// Special hunk markers
// =============================================================================

#[test]
fn no_newline_marker_ignored() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,0 +1,1 @@
+content without newline
\ No newline at end of file
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    assert_eq!(ranges, vec![LineRange::new(1, 1)]);
}

#[test]
fn no_newline_marker_in_old_file() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1 +1 @@
-old without newline
\ No newline at end of file
+new with newline
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    assert_eq!(ranges, vec![LineRange::new(1, 1)]);
}

// =============================================================================
// Error cases for hunk headers
// =============================================================================

#[test]
fn invalid_hunk_header_missing_at_at() {
    // This should be treated as non-hunk content, not an error
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
-1,2 +1,2 @@
+line
"#;

    // Parser should not error, but also not find any hunks
    let map = parse_unified_diff(diff).unwrap();
    assert!(!map.changed.contains_key(&NormPath::new("test.rs")));
}

#[test]
fn hunk_header_missing_minus_segment_errors() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ +1,2 +1,2 @@
+line
"#;

    let result = parse_unified_diff(diff);
    assert!(result.is_err());
}

#[test]
fn hunk_header_missing_plus_segment_errors() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,2 -1,2 @@
+line
"#;

    let result = parse_unified_diff(diff);
    assert!(result.is_err());
}

#[test]
fn hunk_header_non_numeric_start_errors() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -abc +1,2 @@
+line
"#;

    let result = parse_unified_diff(diff);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("invalid"));
}

// =============================================================================
// Hunk boundary detection
// =============================================================================

#[test]
fn hunk_ends_at_next_diff_header() {
    let diff = r#"
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,0 +1,1 @@
+content in a
diff --git a/b.rs b/b.rs
--- b/b.rs
+++ b/b.rs
@@ -1,0 +1,1 @@
+content in b
"#;

    let map = parse_unified_diff(diff).unwrap();

    let a_ranges = map.changed.get(&NormPath::new("a.rs")).unwrap();
    assert_eq!(a_ranges.clone(), vec![LineRange::new(1, 1)]);

    let b_ranges = map.changed.get(&NormPath::new("b.rs")).unwrap();
    assert_eq!(b_ranges.clone(), vec![LineRange::new(1, 1)]);
}

#[test]
fn hunk_ends_at_metadata_line() {
    // When we encounter what looks like metadata after a hunk, the hunk ends
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,0 +1,1 @@
+added line
index 1234567..abcdefg 100644
"#;

    let map = parse_unified_diff(diff).unwrap();
    let ranges = map.changed.get(&NormPath::new("test.rs")).unwrap();
    assert_eq!(ranges.clone(), vec![LineRange::new(1, 1)]);
}

// =============================================================================
// Line counting within hunks
// =============================================================================

#[test]
fn line_counting_tracks_old_and_new_separately() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -10,5 +10,5 @@
 context at 10
-deleted at 11
+added at 11
 context at 12
 context at 13
-deleted at 14
+added at 14
 context at 15
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    // New lines should be at positions 11 and 14 in the new file
    assert_eq!(ranges.len(), 2);
    assert!(ranges.contains(&LineRange::new(11, 11)));
    assert!(ranges.contains(&LineRange::new(14, 14)));
}

#[test]
fn additions_shift_subsequent_line_numbers() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,3 +1,5 @@
 original 1
+inserted 1
+inserted 2
 original 2
 original 3
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    // The inserted lines should be at positions 2 and 3
    assert_eq!(ranges, vec![LineRange::new(2, 3)]);
}

#[test]
fn deletions_shift_subsequent_line_numbers() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,5 +1,3 @@
 original 1
-deleted 1
-deleted 2
 original 2
 original 3
"#;

    let map = parse_unified_diff(diff).unwrap();
    // No additions, so no changed ranges
    assert!(!map.changed.contains_key(&NormPath::new("test.rs")));
}

// =============================================================================
// Complex hunk scenarios
// =============================================================================

#[test]
fn overlapping_hunk_ranges() {
    // Multiple hunks that would have overlapping ranges after line shifts
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -10,2 +10,3 @@
 ctx
+add1
 ctx
@@ -10,2 +13,3 @@
 ctx
+add2
 ctx
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    // Both additions should be captured
    assert!(ranges.iter().any(|r| r.contains_line(11)));
    assert!(ranges.iter().any(|r| r.contains_line(14)));
}

#[test]
fn adjacent_hunks() {
    // Hunks that are immediately adjacent
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,2 +1,3 @@
 line 1
+added after 1
 line 2
@@ -3,1 +4,2 @@
 line 3
+added after 3
"#;

    let ranges = get_changed_ranges(diff, "test.rs");
    // Should have additions at lines 2 and 5
    assert!(ranges.iter().any(|r| r.contains_line(2)));
    assert!(ranges.iter().any(|r| r.contains_line(5)));
}

// =============================================================================
// Stats verification for hunks
// =============================================================================

#[test]
fn hunk_count_in_stats() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,0 +1,1 @@
+a
@@ -10,0 +11,1 @@
+b
@@ -20,0 +22,1 @@
+c
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert_eq!(map.stats.hunks, 3);
}

#[test]
fn added_lines_count_in_stats() {
    let diff = r#"
diff --git a/test.rs b/test.rs
--- a/test.rs
+++ b/test.rs
@@ -1,0 +1,3 @@
+a
+b
+c
"#;

    let map = parse_unified_diff(diff).unwrap();
    assert_eq!(map.stats.added_lines, 3);
}
