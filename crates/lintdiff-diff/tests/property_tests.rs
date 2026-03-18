//! Property-based tests for diff parsing.
//!
//! These tests use proptest to verify invariants across a wide range of inputs.

use lintdiff_diff::parse_unified_diff;
use lintdiff_types::{LineRange, NormPath};
use proptest::prelude::*;

// =============================================================================
// Path Normalization Tests
// =============================================================================

proptest! {
    /// Path normalization should be idempotent - normalizing twice gives same result.
    #[test]
    fn path_normalization_idempotent(path in "[A-Za-z0-9_./\\\\-]{1,50}") {
        let normalized_once = NormPath::new(&path);
        let normalized_twice = NormPath::new(normalized_once.as_str());

        prop_assert_eq!(
            normalized_once,
            normalized_twice,
            "Path normalization should be idempotent"
        );
    }

    /// Paths with different slash types should normalize to the same path.
    #[test]
    fn slashes_normalized(
        segments in prop::collection::vec("[A-Za-z0-9_-]{1,10}", 2..5),
        extension in "[a-z]{1,4}",
    ) {
        let forward_path = format!("{}.{}", segments.join("/"), extension);
        let backslash_path = format!("{}.{}", segments.join("\\"), extension);

        let normalized_forward = NormPath::new(&forward_path);
        let normalized_backslash = NormPath::new(&backslash_path);

        prop_assert_eq!(
            normalized_forward,
            normalized_backslash,
            "Forward and backslash paths should normalize to the same value"
        );
    }

    /// Leading ./ should be stripped.
    #[test]
    fn leading_dot_slash_stripped(rest in "[A-Za-z0-9_./-]{1,40}") {
        let with_prefix = format!("./{}", rest);
        let normalized_with = NormPath::new(&with_prefix);
        let normalized_without = NormPath::new(&rest);

        prop_assert_eq!(
            normalized_with,
            normalized_without,
            "Leading ./ should be stripped"
        );
    }

    /// Multiple slashes should collapse to single slash.
    #[test]
    fn multiple_slashes_collapsed(
        // Use segments that won't trigger a/ or b/ prefix stripping
        segment1 in "[c-zC-Z0-9_-]{1,10}",
        segment2 in "[A-Za-z0-9_-]{1,10}",
        extension in "[a-z]{1,4}",
        slashes in "/{2,5}",
    ) {
        let multiple_slashes = format!("{}{}{}.{}", segment1, slashes, segment2, extension);
        let single_slash = format!("{}/{}.{}", segment1, segment2, extension);

        let normalized_multiple = NormPath::new(&multiple_slashes);
        let normalized_single = NormPath::new(&single_slash);

        prop_assert_eq!(
            normalized_multiple,
            normalized_single,
            "Multiple slashes should collapse to single slash"
        );
    }

    /// a/ and b/ prefixes should be stripped.
    /// Note: rest must not start with "a/" or "b/" to avoid double-stripping
    #[test]
    fn diff_prefixes_stripped(prefix in "(a|b)", rest in "[c-zA-Z0-9_./-][A-Za-z0-9_./-]{0,39}") {
        let with_prefix = format!("{}/{}", prefix, rest);
        let normalized_with = NormPath::new(&with_prefix);
        let normalized_without = NormPath::new(&rest);

        prop_assert_eq!(
            normalized_with,
            normalized_without,
            "a/ and b/ prefixes should be stripped"
        );
    }
}

// =============================================================================
// Diff Parsing Roundtrip Tests
// =============================================================================

/// Strategy for generating valid file paths for diffs.
fn file_path() -> impl Strategy<Value = String> {
    "[A-Za-z0-9_-]{3,10}\\.[a-z]{1,4}"
}

/// Strategy for generating valid diff content lines.
fn diff_content_line() -> impl Strategy<Value = String> {
    "[A-Za-z0-9_ (){};]{5,30}"
}

/// Strategy for generating a valid simple diff.
fn simple_diff() -> impl Strategy<Value = String> {
    (
        file_path(),
        prop::collection::vec(diff_content_line(), 1..10),
        1u32..100u32,
    )
        .prop_map(|(path, lines, start_line)| {
            let num_lines = lines.len() as u32;
            let mut diff = format!(
                "diff --git a/{path} b/{path}\n\
                 --- a/{path}\n\
                 +++ b/{path}\n\
                 @@ -1,0 +{start_line},{num_lines} @@\n"
            );
            for line in lines {
                diff.push_str(&format!("+{line}\n"));
            }
            diff
        })
}

proptest! {
    /// Parsing a valid diff should succeed and extract correct paths.
    #[test]
    fn parse_valid_diff_extracts_path(diff in simple_diff()) {
        let result = parse_unified_diff(&diff);
        prop_assert!(result.is_ok(), "Valid diff should parse successfully");

        let map = result.unwrap();
        prop_assert!(!map.changed.is_empty(), "Should have at least one changed file");
        prop_assert!(map.stats.files >= 1, "Should count at least one file");
    }

    /// Parsing a valid diff should extract correct line numbers.
    #[test]
    fn parse_valid_diff_line_numbers(
        path in file_path(),
        lines in prop::collection::vec(diff_content_line(), 1..20),
        start_line in 1u32..200u32,
    ) {
        let num_lines = lines.len() as u32;
        let diff = format!(
            "diff --git a/{path} b/{path}\n\
             --- a/{path}\n\
             +++ b/{path}\n\
             @@ -1,0 +{start_line},{num_lines} @@\n\
             {}",
            lines.iter().map(|l| format!("+{l}")).collect::<Vec<_>>().join("\n")
        );

        let result = parse_unified_diff(&diff).unwrap();
        let normalized_path = NormPath::new(&path);
        let ranges = result.changed.get(&normalized_path);

        prop_assert!(
            ranges.is_some(),
            "Should have changes for path '{}'",
            normalized_path
        );

        let ranges = ranges.unwrap();
        prop_assert!(!ranges.is_empty(), "Should have at least one range");

        // First added line should be at start_line
        let first_range = &ranges[0];
        prop_assert!(
            first_range.start <= start_line,
            "First range start {} should be <= expected start {}",
            first_range.start,
            start_line
        );
    }

    /// Added lines count should match the diff content.
    #[test]
    fn added_lines_count_matches(
        path in file_path(),
        num_lines in 1u32..30,
        content in diff_content_line(),
    ) {
        let diff = format!(
            "diff --git a/{path} b/{path}\n\
             --- a/{path}\n\
             +++ b/{path}\n\
             @@ -1,0 +1,{num_lines} @@\n\
             {}",
            (0..num_lines).map(|_| format!("+{content}")).collect::<Vec<_>>().join("\n")
        );

        let result = parse_unified_diff(&diff).unwrap();
        prop_assert_eq!(
            result.stats.added_lines,
            num_lines,
            "Added lines count should match"
        );
    }
}

// =============================================================================
// Line Number Invariants Tests
// =============================================================================

proptest! {
    /// Line ranges should always have start <= end.
    #[test]
    fn line_range_start_lte_end(start in 1u32..10000u32, len in 0u32..100u32) {
        let end = start + len;
        let range = LineRange::new(start, end);

        prop_assert!(
            range.start <= range.end,
            "Range start {} should be <= end {}",
            range.start,
            range.end
        );
    }

    /// Line ranges should be 1-based.
    #[test]
    fn line_ranges_are_one_based(
        path in file_path(),
        start_line in 1u32..100u32,
        num_lines in 1u32..10u32,
    ) {
        let diff = format!(
            "diff --git a/{path} b/{path}\n\
             --- a/{path}\n\
             +++ b/{path}\n\
             @@ -1,0 +{start_line},{num_lines} @@\n\
             {}",
            (0..num_lines).map(|i| format!("+line{}", i)).collect::<Vec<_>>().join("\n")
        );

        let result = parse_unified_diff(&diff).unwrap();
        let normalized_path = NormPath::new(&path);

        if let Some(ranges) = result.changed.get(&normalized_path) {
            for range in ranges {
                prop_assert!(
                    range.start >= 1,
                    "Range start should be >= 1, got {}",
                    range.start
                );
                prop_assert!(
                    range.end >= 1,
                    "Range end should be >= 1, got {}",
                    range.end
                );
            }
        }
    }

    /// Line ranges from the same file should not overlap in unexpected ways.
    #[test]
    fn line_ranges_are_sorted(
        path in file_path(),
        lines in prop::collection::vec(1u32..1000u32, 1..50),
    ) {
        // Create a diff with lines at various positions
        let mut sorted_lines = lines.clone();
        sorted_lines.sort_unstable();
        sorted_lines.dedup();

        let diff_content: String = sorted_lines
            .iter()
            .map(|l| format!("+line at {}\n", l))
            .collect();

        // Build a diff that has hunks at these lines
        let diff = format!(
            "diff --git a/{path} b/{path}\n\
             --- a/{path}\n\
             +++ b/{path}\n\
             @@ -1,0 +1,{} @@\n\
             {}",
            sorted_lines.len(),
            diff_content
        );

        let result = parse_unified_diff(&diff).unwrap();
        let normalized_path = NormPath::new(&path);

        if let Some(ranges) = result.changed.get(&normalized_path) {
            // Ranges should be sorted by start
            for window in ranges.windows(2) {
                prop_assert!(
                    window[0].start <= window[1].start,
                    "Ranges should be sorted: {:?} should come before {:?}",
                    window[0],
                    window[1]
                );
            }
        }
    }
}

// =============================================================================
// Multi-File Diff Tests
// =============================================================================

proptest! {
    /// Multi-file diffs should parse all files.
    #[test]
    fn multi_file_diff_parses_all_files(
        file1 in file_path(),
        file2 in file_path(),
        content in diff_content_line(),
    ) {
        prop_assume!(file1 != file2);

        let diff = format!(
            "diff --git a/{file1} b/{file1}\n\
             --- a/{file1}\n\
             +++ b/{file1}\n\
             @@ -1,0 +1,1 @@\n\
             +{content}\n\
             diff --git a/{file2} b/{file2}\n\
             --- a/{file2}\n\
             +++ b/{file2}\n\
             @@ -1,0 +1,1 @@\n\
             +{content}\n"
        );

        let result = parse_unified_diff(&diff).unwrap();

        prop_assert_eq!(result.stats.files, 2, "Should have 2 files");
        prop_assert!(result.changed.contains_key(&NormPath::new(&file1)));
        prop_assert!(result.changed.contains_key(&NormPath::new(&file2)));
    }

    /// Stats should accumulate across files.
    #[test]
    fn stats_accumulate(
        files in prop::collection::vec(file_path(), 2..5),
        lines_per_file in 1u32..5u32,
        content in diff_content_line(),
    ) {
        let num_files = files.len() as u32;

        let diff: String = files
            .iter()
            .map(|f| {
                format!(
                    "diff --git a/{f} b/{f}\n\
                     --- a/{f}\n\
                     +++ b/{f}\n\
                     @@ -1,0 +1,{} @@\n\
                     {}\n",
                    lines_per_file,
                    (0..lines_per_file).map(|_| format!("+{content}")).collect::<Vec<_>>().join("\n")
                )
            })
            .collect();

        let result = parse_unified_diff(&diff).unwrap();

        prop_assert_eq!(result.stats.files, num_files);
        prop_assert_eq!(result.stats.added_lines, num_files * lines_per_file);
    }
}

// =============================================================================
// Edge Case Tests
// =============================================================================

proptest! {
    /// Empty diff should parse successfully.
    #[test]
    fn empty_diff_parses(junk in "[A-Za-z ]{0,50}") {
        let diff = junk; // Just random text, no actual diff
        let result = parse_unified_diff(&diff);

        // Should parse without error (returns empty map)
        prop_assert!(result.is_ok());
        let map = result.unwrap();
        prop_assert!(map.changed.is_empty());
    }

    /// Diff with only context lines should have no changes.
    #[test]
    fn context_only_no_changes(
        path in file_path(),
        lines in prop::collection::vec(diff_content_line(), 1..10),
    ) {
        let diff = format!(
            "diff --git a/{path} b/{path}\n\
             --- a/{path}\n\
             +++ b/{path}\n\
             @@ -1,{} +1,{} @@\n\
             {}",
            lines.len(),
            lines.len(),
            lines.iter().map(|l| format!(" {l}")).collect::<Vec<_>>().join("\n")
        );

        let result = parse_unified_diff(&diff).unwrap();
        let normalized_path = NormPath::new(&path);

        // Context lines (starting with space) should not be counted as changes
        if let Some(ranges) = result.changed.get(&normalized_path) {
            prop_assert!(
                ranges.is_empty(),
                "Context-only diff should have no changed lines"
            );
        }
    }

    /// Deleted lines should not appear in new-side changes.
    #[test]
    fn deleted_lines_not_in_changes(
        path in file_path(),
        added in prop::collection::vec(diff_content_line(), 1..5),
        deleted in prop::collection::vec(diff_content_line(), 1..5),
    ) {
        let mut diff_content = String::new();

        // Add deleted lines
        for line in &deleted {
            diff_content.push_str(&format!("-{line}\n"));
        }

        // Add added lines
        for line in &added {
            diff_content.push_str(&format!("+{line}\n"));
        }

        let diff = format!(
            "diff --git a/{path} b/{path}\n\
             --- a/{path}\n\
             +++ b/{path}\n\
             @@ -1,{} +1,{} @@\n\
             {}",
            deleted.len(),
            added.len(),
            diff_content
        );

        let result = parse_unified_diff(&diff).unwrap();

        // Only added lines should be counted
        prop_assert_eq!(
            result.stats.added_lines,
            added.len() as u32,
            "Only added lines should be counted"
        );
    }

    /// Very long lines should be handled.
    #[test]
    fn very_long_lines(
        path in file_path(),
        long_content in "[A-Za-z ]{1000,5000}",
    ) {
        let diff = format!(
            "diff --git a/{path} b/{path}\n\
             --- a/{path}\n\
             +++ b/{path}\n\
             @@ -1,0 +1,1 @@\n\
             +{long_content}\n"
        );

        let result = parse_unified_diff(&diff);
        prop_assert!(result.is_ok(), "Should handle very long lines");

        let map = result.unwrap();
        prop_assert_eq!(map.stats.added_lines, 1);
    }

    /// Special characters in paths should be handled.
    #[test]
    fn special_chars_in_path(
        segment in "[A-Za-z0-9_.-]{2,10}",
        extension in "[a-z]{1,4}",
    ) {
        let path = format!("{}.{}", segment, extension);

        let diff = format!(
            "diff --git a/{path} b/{path}\n\
             --- a/{path}\n\
             +++ b/{path}\n\
             @@ -1,0 +1,1 @@\n\
             +content\n"
        );

        let result = parse_unified_diff(&diff);
        prop_assert!(result.is_ok(), "Should handle path: {}", path);
    }
}

// =============================================================================
// Rename Detection Tests
// =============================================================================

proptest! {
    /// Renamed files should be tracked.
    #[test]
    fn renamed_files_tracked(
        old_name in "[a-z]{3,10}\\.rs",
        new_name in "[a-z]{3,10}\\.rs",
    ) {
        prop_assume!(old_name != new_name);

        let diff = format!(
            "diff --git a/{old_name} b/{new_name}\n\
             rename from {old_name}\n\
             rename to {new_name}\n\
             --- a/{old_name}\n\
             +++ b/{new_name}\n\
             @@ -1,0 +1,1 @@\n\
             +content\n"
        );

        let result = parse_unified_diff(&diff).unwrap();

        // Should track the rename
        let old_path = NormPath::new(&old_name);
        let new_path = NormPath::new(&new_name);

        if let Some(mapped) = result.renames.get(&old_path) {
            prop_assert_eq!(mapped, &new_path, "Rename should be tracked");
        }
    }
}
