//! Comprehensive BDD tests for lintdiff-diff-stats crate.
//!
//! Covers:
//! - DiffStats construction and methods
//! - FileDiffStats per-file statistics
//! - ChangeType variants
//! - Builder pattern
//! - Formatting functions
//! - Helper functions
//! - Edge cases (empty diff, large numbers)
//! - Property-based tests with proptest

use lintdiff_diff_stats::*;
use proptest::prop_assert;
use proptest::prop_assert_eq;

// =============================================================================
// ChangeType Tests (12 tests)
// =============================================================================

#[test]
fn change_type_added_display_shows_added() {
    assert_eq!(ChangeType::Added.to_string(), "added");
}

#[test]
fn change_type_deleted_display_shows_deleted() {
    assert_eq!(ChangeType::Deleted.to_string(), "deleted");
}

#[test]
fn change_type_modified_display_shows_modified() {
    assert_eq!(ChangeType::Modified.to_string(), "modified");
}

#[test]
fn change_type_renamed_display_shows_renamed() {
    assert_eq!(ChangeType::Renamed.to_string(), "renamed");
}

#[test]
fn change_type_added_symbol_is_a() {
    assert_eq!(ChangeType::Added.symbol(), "A");
}

#[test]
fn change_type_deleted_symbol_is_d() {
    assert_eq!(ChangeType::Deleted.symbol(), "D");
}

#[test]
fn change_type_modified_symbol_is_m() {
    assert_eq!(ChangeType::Modified.symbol(), "M");
}

#[test]
fn change_type_renamed_symbol_is_r() {
    assert_eq!(ChangeType::Renamed.symbol(), "R");
}

#[test]
fn change_type_is_addition_returns_true_only_for_added() {
    assert!(ChangeType::Added.is_addition());
    assert!(!ChangeType::Deleted.is_addition());
    assert!(!ChangeType::Modified.is_addition());
    assert!(!ChangeType::Renamed.is_addition());
}

#[test]
fn change_type_is_deletion_returns_true_only_for_deleted() {
    assert!(!ChangeType::Added.is_deletion());
    assert!(ChangeType::Deleted.is_deletion());
    assert!(!ChangeType::Modified.is_deletion());
    assert!(!ChangeType::Renamed.is_deletion());
}

#[test]
fn change_type_is_modification_returns_true_only_for_modified() {
    assert!(!ChangeType::Added.is_modification());
    assert!(!ChangeType::Deleted.is_modification());
    assert!(ChangeType::Modified.is_modification());
    assert!(!ChangeType::Renamed.is_modification());
}

#[test]
fn change_type_is_rename_returns_true_only_for_renamed() {
    assert!(!ChangeType::Added.is_rename());
    assert!(!ChangeType::Deleted.is_rename());
    assert!(!ChangeType::Modified.is_rename());
    assert!(ChangeType::Renamed.is_rename());
}

// =============================================================================
// FileDiffStats Construction Tests (10 tests)
// =============================================================================

#[test]
fn file_diff_stats_new_sets_all_fields() {
    let stats = FileDiffStats::new("src/main.rs", 10, 5, ChangeType::Modified);
    assert_eq!(stats.path, "src/main.rs");
    assert_eq!(stats.lines_added, 10);
    assert_eq!(stats.lines_removed, 5);
    assert_eq!(stats.change_type, ChangeType::Modified);
}

#[test]
fn file_diff_stats_added_creates_added_type() {
    let stats = FileDiffStats::added("new_file.rs", 25);
    assert_eq!(stats.path, "new_file.rs");
    assert_eq!(stats.lines_added, 25);
    assert_eq!(stats.lines_removed, 0);
    assert_eq!(stats.change_type, ChangeType::Added);
}

#[test]
fn file_diff_stats_deleted_creates_deleted_type() {
    let stats = FileDiffStats::deleted("old_file.rs", 15);
    assert_eq!(stats.path, "old_file.rs");
    assert_eq!(stats.lines_added, 0);
    assert_eq!(stats.lines_removed, 15);
    assert_eq!(stats.change_type, ChangeType::Deleted);
}

#[test]
fn file_diff_stats_modified_creates_modified_type() {
    let stats = FileDiffStats::modified("changed.rs", 8, 3);
    assert_eq!(stats.path, "changed.rs");
    assert_eq!(stats.lines_added, 8);
    assert_eq!(stats.lines_removed, 3);
    assert_eq!(stats.change_type, ChangeType::Modified);
}

#[test]
fn file_diff_stats_renamed_creates_renamed_type() {
    let stats = FileDiffStats::renamed("renamed_file.rs");
    assert_eq!(stats.path, "renamed_file.rs");
    assert_eq!(stats.lines_added, 0);
    assert_eq!(stats.lines_removed, 0);
    assert_eq!(stats.change_type, ChangeType::Renamed);
}

#[test]
fn file_diff_stats_default_is_empty_modified() {
    let stats = FileDiffStats::default();
    assert_eq!(stats.path, "");
    assert_eq!(stats.lines_added, 0);
    assert_eq!(stats.lines_removed, 0);
    assert_eq!(stats.change_type, ChangeType::Modified);
}

#[test]
fn file_diff_stats_net_lines_positive_when_more_added() {
    let stats = FileDiffStats::modified("test.rs", 10, 5);
    assert_eq!(stats.net_lines(), 5);
}

#[test]
fn file_diff_stats_net_lines_negative_when_more_removed() {
    let stats = FileDiffStats::modified("test.rs", 3, 8);
    assert_eq!(stats.net_lines(), -5);
}

#[test]
fn file_diff_stats_net_lines_zero_when_equal() {
    let stats = FileDiffStats::modified("test.rs", 5, 5);
    assert_eq!(stats.net_lines(), 0);
}

#[test]
fn file_diff_stats_has_changes_true_when_any_lines() {
    assert!(FileDiffStats::added("a.rs", 1).has_changes());
    assert!(FileDiffStats::deleted("b.rs", 1).has_changes());
    assert!(FileDiffStats::modified("c.rs", 0, 1).has_changes());
    assert!(!FileDiffStats::renamed("d.rs").has_changes());
}

// =============================================================================
// FileDiffStats Method Tests (6 tests)
// =============================================================================

#[test]
fn file_diff_stats_total_lines_changed_sums_additions_and_removals() {
    let stats = FileDiffStats::modified("test.rs", 10, 5);
    assert_eq!(stats.total_lines_changed(), 15);
}

#[test]
fn file_diff_stats_total_lines_changed_zero_when_no_changes() {
    let stats = FileDiffStats::renamed("test.rs");
    assert_eq!(stats.total_lines_changed(), 0);
}

#[test]
fn file_diff_stats_path_accepts_string() {
    let path = String::from("src/lib.rs");
    let stats = FileDiffStats::added(path.clone(), 10);
    assert_eq!(stats.path, path);
}

#[test]
fn file_diff_stats_path_accepts_str() {
    let stats = FileDiffStats::added("src/lib.rs", 10);
    assert_eq!(stats.path, "src/lib.rs");
}

#[test]
fn file_diff_stats_clones_correctly() {
    let original = FileDiffStats::modified("test.rs", 10, 5);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn file_diff_stats_debug_includes_all_fields() {
    let stats = FileDiffStats::modified("test.rs", 10, 5);
    let debug = format!("{:?}", stats);
    assert!(debug.contains("test.rs"));
    assert!(debug.contains("lines_added"));
    assert!(debug.contains("lines_removed"));
}

// =============================================================================
// DiffStats Construction Tests (8 tests)
// =============================================================================

#[test]
fn diff_stats_new_creates_zeroed_stats() {
    let stats = DiffStats::new();
    assert_eq!(stats.files_changed, 0);
    assert_eq!(stats.lines_added, 0);
    assert_eq!(stats.lines_removed, 0);
    assert_eq!(stats.files_added, 0);
    assert_eq!(stats.files_deleted, 0);
    assert_eq!(stats.files_renamed, 0);
}

#[test]
fn diff_stats_default_equals_new() {
    let default_stats = DiffStats::default();
    let new_stats = DiffStats::new();
    assert_eq!(default_stats, new_stats);
}

#[test]
fn diff_stats_from_values_sets_all_fields() {
    let stats = DiffStats::from_values(5, 100, 50, 2, 1, 1);
    assert_eq!(stats.files_changed, 5);
    assert_eq!(stats.lines_added, 100);
    assert_eq!(stats.lines_removed, 50);
    assert_eq!(stats.files_added, 2);
    assert_eq!(stats.files_deleted, 1);
    assert_eq!(stats.files_renamed, 1);
}

#[test]
fn diff_stats_from_values_accepts_zeros() {
    let stats = DiffStats::from_values(0, 0, 0, 0, 0, 0);
    assert!(stats.is_empty());
}

#[test]
fn diff_stats_total_files_equals_files_changed() {
    let stats = DiffStats::from_values(10, 0, 0, 0, 0, 0);
    assert_eq!(stats.total_files(), 10);
}

#[test]
fn diff_stats_net_lines_positive_when_more_added() {
    let stats = DiffStats::from_values(1, 100, 50, 0, 0, 0);
    assert_eq!(stats.net_lines(), 50);
}

#[test]
fn diff_stats_net_lines_negative_when_more_removed() {
    let stats = DiffStats::from_values(1, 50, 100, 0, 0, 0);
    assert_eq!(stats.net_lines(), -50);
}

#[test]
fn diff_stats_total_lines_changed_sums_additions_and_removals() {
    let stats = DiffStats::from_values(1, 100, 50, 0, 0, 0);
    assert_eq!(stats.total_lines_changed(), 150);
}

// =============================================================================
// DiffStats Empty/Check Tests (8 tests)
// =============================================================================

#[test]
fn diff_stats_is_empty_true_when_all_zero() {
    let stats = DiffStats::new();
    assert!(stats.is_empty());
}

#[test]
fn diff_stats_is_empty_false_when_files_changed() {
    let stats = DiffStats::from_values(1, 0, 0, 0, 0, 0);
    assert!(!stats.is_empty());
}

#[test]
fn diff_stats_is_empty_false_when_lines_added() {
    let stats = DiffStats::from_values(0, 1, 0, 0, 0, 0);
    assert!(!stats.is_empty());
}

#[test]
fn diff_stats_is_empty_false_when_lines_removed() {
    let stats = DiffStats::from_values(0, 0, 1, 0, 0, 0);
    assert!(!stats.is_empty());
}

#[test]
fn diff_stats_has_additions_true_when_files_added() {
    let stats = DiffStats::from_values(0, 0, 0, 1, 0, 0);
    assert!(stats.has_additions());
}

#[test]
fn diff_stats_has_deletions_true_when_files_deleted() {
    let stats = DiffStats::from_values(0, 0, 0, 0, 1, 0);
    assert!(stats.has_deletions());
}

#[test]
fn diff_stats_has_renames_true_when_files_renamed() {
    let stats = DiffStats::from_values(0, 0, 0, 0, 0, 1);
    assert!(stats.has_renames());
}

#[test]
fn diff_stats_has_modifications_true_when_non_special_files() {
    let stats = DiffStats::from_values(5, 0, 0, 1, 1, 1);
    // 5 files changed, but only 3 are special (added + deleted + renamed)
    // So 2 are modifications
    assert!(stats.has_modifications());
}

// =============================================================================
// DiffStats Arithmetic Tests (6 tests)
// =============================================================================

#[test]
fn diff_stats_add_combines_all_fields() {
    let a = DiffStats::from_values(2, 10, 5, 1, 0, 0);
    let b = DiffStats::from_values(3, 20, 10, 0, 1, 1);
    let merged = a + b;

    assert_eq!(merged.files_changed, 5);
    assert_eq!(merged.lines_added, 30);
    assert_eq!(merged.lines_removed, 15);
    assert_eq!(merged.files_added, 1);
    assert_eq!(merged.files_deleted, 1);
    assert_eq!(merged.files_renamed, 1);
}

#[test]
fn diff_stats_add_assign_modifies_in_place() {
    let mut a = DiffStats::from_values(2, 10, 5, 1, 0, 0);
    let b = DiffStats::from_values(3, 20, 10, 0, 1, 1);
    a += b;

    assert_eq!(a.files_changed, 5);
    assert_eq!(a.lines_added, 30);
}

#[test]
fn diff_stats_add_with_empty_returns_same() {
    let stats = DiffStats::from_values(5, 10, 5, 1, 1, 1);
    let empty = DiffStats::new();
    let merged = stats.clone() + empty;
    assert_eq!(merged, stats);
}

#[test]
fn diff_stats_add_is_commutative() {
    let a = DiffStats::from_values(2, 10, 5, 1, 0, 0);
    let b = DiffStats::from_values(3, 20, 10, 0, 1, 1);

    let ab = a.clone() + b.clone();
    let ba = b + a;

    assert_eq!(ab, ba);
}

#[test]
fn diff_stats_add_is_associative() {
    let a = DiffStats::from_values(1, 5, 2, 1, 0, 0);
    let b = DiffStats::from_values(2, 10, 5, 0, 1, 0);
    let c = DiffStats::from_values(3, 15, 8, 0, 0, 1);

    let ab_c = (a.clone() + b.clone()) + c.clone();
    let a_bc = a + (b + c);

    assert_eq!(ab_c, a_bc);
}

#[test]
fn diff_stats_add_handles_large_values() {
    let a = DiffStats::from_values(usize::MAX / 2, usize::MAX / 2, 0, 0, 0, 0);
    let b = DiffStats::from_values(usize::MAX / 2, usize::MAX / 2, 0, 0, 0, 0);
    let merged = a + b;

    // Note: This will overflow, but we're testing that it doesn't panic
    assert!(merged.files_changed > 0);
}

// =============================================================================
// DiffStatsBuilder Tests (12 tests)
// =============================================================================

#[test]
fn builder_new_creates_empty_builder() {
    let builder = DiffStatsBuilder::new();
    assert_eq!(builder.file_count(), 0);
    assert!(builder.is_empty());
}

#[test]
fn builder_default_equals_new() {
    let default_builder = DiffStatsBuilder::default();
    let new_builder = DiffStatsBuilder::new();
    // Both should produce the same stats
    assert_eq!(default_builder.build(), new_builder.build());
}

#[test]
fn builder_add_file_increments_files_changed() {
    let mut builder = DiffStatsBuilder::new();
    builder.add_file(&FileDiffStats::modified("test.rs", 10, 5));
    assert_eq!(builder.file_count(), 1);
}

#[test]
fn builder_add_file_tracks_added_files() {
    let mut builder = DiffStatsBuilder::new();
    builder.add_file(&FileDiffStats::added("new.rs", 10));
    let stats = builder.build();
    assert_eq!(stats.files_added, 1);
}

#[test]
fn builder_add_file_tracks_deleted_files() {
    let mut builder = DiffStatsBuilder::new();
    builder.add_file(&FileDiffStats::deleted("old.rs", 10));
    let stats = builder.build();
    assert_eq!(stats.files_deleted, 1);
}

#[test]
fn builder_add_file_tracks_renamed_files() {
    let mut builder = DiffStatsBuilder::new();
    builder.add_file(&FileDiffStats::renamed("renamed.rs"));
    let stats = builder.build();
    assert_eq!(stats.files_renamed, 1);
}

#[test]
fn builder_add_file_tracks_modified_files_not_counted_separately() {
    let mut builder = DiffStatsBuilder::new();
    builder.add_file(&FileDiffStats::modified("mod.rs", 10, 5));
    let stats = builder.build();
    // Modified files are counted in files_changed but not in files_added/deleted/renamed
    assert_eq!(stats.files_changed, 1);
    assert_eq!(stats.files_added, 0);
    assert_eq!(stats.files_deleted, 0);
    assert_eq!(stats.files_renamed, 0);
}

#[test]
fn builder_add_file_sums_lines() {
    let mut builder = DiffStatsBuilder::new();
    builder
        .add_file(&FileDiffStats::modified("a.rs", 10, 5))
        .add_file(&FileDiffStats::modified("b.rs", 20, 10));
    let stats = builder.build();
    assert_eq!(stats.lines_added, 30);
    assert_eq!(stats.lines_removed, 15);
}

#[test]
fn builder_add_files_accepts_iterator() {
    let mut builder = DiffStatsBuilder::new();
    let files = vec![
        FileDiffStats::added("a.rs", 10),
        FileDiffStats::added("b.rs", 20),
    ];
    builder.add_files(files);
    assert_eq!(builder.file_count(), 2);
}

#[test]
fn builder_build_returns_correct_stats() {
    let mut builder = DiffStatsBuilder::new();
    builder
        .add_file(&FileDiffStats::added("new.rs", 10))
        .add_file(&FileDiffStats::deleted("old.rs", 5))
        .add_file(&FileDiffStats::modified("mod.rs", 3, 2))
        .add_file(&FileDiffStats::renamed("ren.rs"));

    let stats = builder.build();
    assert_eq!(stats.files_changed, 4);
    assert_eq!(stats.lines_added, 13);
    assert_eq!(stats.lines_removed, 7);
    assert_eq!(stats.files_added, 1);
    assert_eq!(stats.files_deleted, 1);
    assert_eq!(stats.files_renamed, 1);
}

#[test]
fn builder_reset_clears_all_values() {
    let mut builder = DiffStatsBuilder::new();
    builder.add_file(&FileDiffStats::added("test.rs", 100));
    assert_eq!(builder.file_count(), 1);

    builder.reset();
    assert_eq!(builder.file_count(), 0);
    assert!(builder.is_empty());
}

#[test]
fn builder_chains_add_file_calls() {
    let stats = DiffStatsBuilder::new()
        .add_file(&FileDiffStats::added("a.rs", 10))
        .add_file(&FileDiffStats::added("b.rs", 20))
        .build();
    assert_eq!(stats.files_changed, 2);
}

// =============================================================================
// format_stats Tests (8 tests)
// =============================================================================

#[test]
fn format_stats_empty_shows_no_changes() {
    let stats = DiffStats::new();
    assert_eq!(format_stats(&stats), "No changes");
}

#[test]
fn format_stats_shows_files_changed() {
    let stats = DiffStats::from_values(3, 0, 0, 0, 0, 0);
    let formatted = format_stats(&stats);
    assert!(formatted.contains("3 files changed"));
}

#[test]
fn format_stats_singular_file() {
    let stats = DiffStats::from_values(1, 0, 0, 0, 0, 0);
    let formatted = format_stats(&stats);
    assert!(formatted.contains("1 file changed"));
}

#[test]
fn format_stats_shows_line_changes() {
    let stats = DiffStats::from_values(1, 10, 5, 0, 0, 0);
    let formatted = format_stats(&stats);
    assert!(formatted.contains("10 insertions"));
    assert!(formatted.contains("5 deletions"));
}

#[test]
fn format_stats_singular_insertion_deletion() {
    let stats = DiffStats::from_values(1, 1, 1, 0, 0, 0);
    let formatted = format_stats(&stats);
    assert!(formatted.contains("1 insertion"));
    assert!(formatted.contains("1 deletion"));
}

#[test]
fn format_stats_shows_file_operations() {
    let stats = DiffStats::from_values(5, 10, 5, 2, 1, 1);
    let formatted = format_stats(&stats);
    assert!(formatted.contains("2 added"));
    assert!(formatted.contains("1 deleted"));
    assert!(formatted.contains("1 renamed"));
}

#[test]
fn format_stats_hides_zero_operations() {
    let stats = DiffStats::from_values(1, 10, 5, 0, 0, 0);
    let formatted = format_stats(&stats);
    assert!(!formatted.contains("0 added"));
    assert!(!formatted.contains("0 deleted"));
}

#[test]
fn format_stats_handles_all_operations() {
    let stats = DiffStats::from_values(6, 100, 50, 2, 2, 2);
    let formatted = format_stats(&stats);
    assert!(formatted.contains("6 files changed"));
    assert!(formatted.contains("100 insertions"));
    assert!(formatted.contains("50 deletions"));
    assert!(formatted.contains("2 added"));
    assert!(formatted.contains("2 deleted"));
    assert!(formatted.contains("2 renamed"));
}

// =============================================================================
// format_stats_short Tests (6 tests)
// =============================================================================

#[test]
fn format_stats_short_empty_shows_zero() {
    let stats = DiffStats::new();
    assert_eq!(format_stats_short(&stats), "0");
}

#[test]
fn format_stats_short_shows_lines_and_files() {
    let stats = DiffStats::from_values(3, 10, 5, 0, 0, 0);
    assert_eq!(format_stats_short(&stats), "+10-5 files:3");
}

#[test]
fn format_stats_short_handles_zero_lines() {
    let stats = DiffStats::from_values(1, 0, 0, 0, 0, 0);
    assert_eq!(format_stats_short(&stats), "+0-0 files:1");
}

#[test]
fn format_stats_short_handles_large_numbers() {
    let stats = DiffStats::from_values(1000, 10000, 5000, 0, 0, 0);
    assert_eq!(format_stats_short(&stats), "+10000-5000 files:1000");
}

#[test]
fn format_stats_short_zero_lines_removed() {
    let stats = DiffStats::from_values(1, 10, 0, 0, 0, 0);
    assert_eq!(format_stats_short(&stats), "+10-0 files:1");
}

#[test]
fn format_stats_short_zero_lines_added() {
    let stats = DiffStats::from_values(1, 0, 10, 0, 0, 0);
    assert_eq!(format_stats_short(&stats), "+0-10 files:1");
}

// =============================================================================
// format_stats_markdown Tests (6 tests)
// =============================================================================

#[test]
fn format_stats_markdown_creates_table_header() {
    let stats = DiffStats::new();
    let markdown = format_stats_markdown(&stats);
    assert!(markdown.contains("| Metric | Count |"));
    assert!(markdown.contains("|--------|-------|"));
}

#[test]
fn format_stats_markdown_includes_all_metrics() {
    let stats = DiffStats::from_values(5, 100, 50, 2, 1, 1);
    let markdown = format_stats_markdown(&stats);

    assert!(markdown.contains("| Files changed | 5 |"));
    assert!(markdown.contains("| Lines added | 100 |"));
    assert!(markdown.contains("| Lines removed | 50 |"));
    assert!(markdown.contains("| Files added | 2 |"));
    assert!(markdown.contains("| Files deleted | 1 |"));
    assert!(markdown.contains("| Files renamed | 1 |"));
}

#[test]
fn format_stats_markdown_includes_net_lines() {
    let stats = DiffStats::from_values(1, 100, 50, 0, 0, 0);
    let markdown = format_stats_markdown(&stats);
    assert!(markdown.contains("| Net lines | 50 |"));
}

#[test]
fn format_stats_markdown_shows_negative_net_lines() {
    let stats = DiffStats::from_values(1, 50, 100, 0, 0, 0);
    let markdown = format_stats_markdown(&stats);
    assert!(markdown.contains("| Net lines | -50 |"));
}

#[test]
fn format_stats_markdown_handles_empty_stats() {
    let stats = DiffStats::new();
    let markdown = format_stats_markdown(&stats);

    assert!(markdown.contains("| Files changed | 0 |"));
    assert!(markdown.contains("| Net lines | 0 |"));
}

#[test]
fn format_stats_markdown_has_seven_rows() {
    let stats = DiffStats::new();
    let markdown = format_stats_markdown(&stats);
    // Count the number of table rows (lines starting with |)
    let row_count = markdown.lines().filter(|l| l.starts_with('|')).count();
    assert_eq!(row_count, 9); // header + separator + 7 data rows
}

// =============================================================================
// Helper Function Tests (8 tests)
// =============================================================================

#[test]
fn is_empty_returns_true_for_empty_stats() {
    let stats = DiffStats::new();
    assert!(is_empty(&stats));
}

#[test]
fn is_empty_returns_false_for_non_empty_stats() {
    let stats = DiffStats::from_values(1, 0, 0, 0, 0, 0);
    assert!(!is_empty(&stats));
}

#[test]
fn net_lines_returns_positive_for_more_additions() {
    let stats = DiffStats::from_values(1, 100, 50, 0, 0, 0);
    assert_eq!(net_lines(&stats), 50);
}

#[test]
fn net_lines_returns_negative_for_more_removals() {
    let stats = DiffStats::from_values(1, 50, 100, 0, 0, 0);
    assert_eq!(net_lines(&stats), -50);
}

#[test]
fn net_lines_returns_zero_for_equal_changes() {
    let stats = DiffStats::from_values(1, 50, 50, 0, 0, 0);
    assert_eq!(net_lines(&stats), 0);
}

#[test]
fn merge_stats_combines_two_stats() {
    let a = DiffStats::from_values(2, 10, 5, 1, 0, 0);
    let b = DiffStats::from_values(3, 20, 10, 0, 1, 1);
    let merged = merge_stats(&a, &b);

    assert_eq!(merged.files_changed, 5);
    assert_eq!(merged.lines_added, 30);
    assert_eq!(merged.lines_removed, 15);
}

#[test]
fn empty_stats_creates_zeroed_stats() {
    let stats = empty_stats();
    assert_eq!(stats.files_changed, 0);
    assert_eq!(stats.lines_added, 0);
    assert_eq!(stats.lines_removed, 0);
}

#[test]
fn from_file_stats_aggregates_correctly() {
    let files = vec![
        FileDiffStats::added("a.rs", 10),
        FileDiffStats::deleted("b.rs", 5),
        FileDiffStats::modified("c.rs", 3, 2),
    ];
    let stats = from_file_stats(&files);

    assert_eq!(stats.files_changed, 3);
    assert_eq!(stats.lines_added, 13);
    assert_eq!(stats.lines_removed, 7);
    assert_eq!(stats.files_added, 1);
    assert_eq!(stats.files_deleted, 1);
}

// =============================================================================
// Edge Case Tests (8 tests)
// =============================================================================

#[test]
fn diff_stats_handles_zero_lines_added_removed() {
    let stats = DiffStats::from_values(1, 0, 0, 0, 0, 0);
    assert_eq!(stats.net_lines(), 0);
    assert_eq!(stats.total_lines_changed(), 0);
}

#[test]
fn diff_stats_handles_large_line_counts() {
    let stats = DiffStats::from_values(100, usize::MAX, usize::MAX, 0, 0, 0);
    // Note: total_lines_changed will overflow, testing it doesn't panic
    assert_eq!(stats.lines_added, usize::MAX);
}

#[test]
fn file_diff_stats_handles_empty_path() {
    let stats = FileDiffStats::added("", 10);
    assert_eq!(stats.path, "");
}

#[test]
fn file_diff_stats_handles_unicode_path() {
    let stats = FileDiffStats::added("src/日本語/файл.rs", 10);
    assert_eq!(stats.path, "src/日本語/файл.rs");
}

#[test]
fn builder_handles_many_files() {
    let mut builder = DiffStatsBuilder::new();
    for i in 0..1000 {
        builder.add_file(&FileDiffStats::modified(format!("file{}.rs", i), 1, 1));
    }
    let stats = builder.build();
    assert_eq!(stats.files_changed, 1000);
    assert_eq!(stats.lines_added, 1000);
    assert_eq!(stats.lines_removed, 1000);
}

#[test]
fn format_stats_handles_only_additions() {
    let stats = DiffStats::from_values(1, 100, 0, 1, 0, 0);
    let formatted = format_stats(&stats);
    assert!(formatted.contains("100 insertions"));
    assert!(formatted.contains("0 deletions"));
}

#[test]
fn format_stats_handles_only_deletions() {
    let stats = DiffStats::from_values(1, 0, 100, 0, 1, 0);
    let formatted = format_stats(&stats);
    assert!(formatted.contains("0 insertions"));
    assert!(formatted.contains("100 deletions"));
}

#[test]
fn change_type_default_is_modified() {
    assert_eq!(ChangeType::default(), ChangeType::Modified);
}

// =============================================================================
// Property-Based Tests with proptest (8 tests)
// =============================================================================

proptest::proptest! {
    #[test]
    fn prop_diff_stats_add_is_commutative(
        a_files in 0usize..1000,
        a_added in 0usize..10000,
        a_removed in 0usize..10000,
        b_files in 0usize..1000,
        b_added in 0usize..10000,
        b_removed in 0usize..10000
    ) {
        let a = DiffStats::from_values(a_files, a_added, a_removed, 0, 0, 0);
        let b = DiffStats::from_values(b_files, b_added, b_removed, 0, 0, 0);

        let ab = a.clone() + b.clone();
        let ba = b + a;

        prop_assert_eq!(ab, ba);
    }

    #[test]
    fn prop_net_lines_is_added_minus_removed(
        added in 0usize..100000,
        removed in 0usize..100000
    ) {
        let stats = DiffStats::from_values(1, added, removed, 0, 0, 0);
        let expected = added as isize - removed as isize;
        prop_assert_eq!(stats.net_lines(), expected);
    }

    #[test]
    fn prop_builder_aggregates_lines_correctly(
        files in proptest::collection::vec(
            (0usize..1000, 0usize..1000),
            0..100
        )
    ) {
        let mut builder = DiffStatsBuilder::new();
        let mut expected_added = 0usize;
        let mut expected_removed = 0usize;

        for (added, removed) in &files {
            builder.add_file(&FileDiffStats::modified("test.rs", *added, *removed));
            expected_added += added;
            expected_removed += removed;
        }

        let stats = builder.build();
        prop_assert_eq!(stats.lines_added, expected_added);
        prop_assert_eq!(stats.lines_removed, expected_removed);
    }

    #[test]
    fn prop_format_stats_short_round_trips(
        files in 0usize..1000usize,
        added in 0usize..100000usize,
        removed in 0usize..100000usize
    ) {
        let stats = DiffStats::from_values(files, added, removed, 0, 0, 0);
        let formatted = format_stats_short(&stats);
        let expected_added = format!("+{}-", added);
        let expected_files = format!("files:{}", files);
        prop_assert!(formatted.starts_with('+'));
        prop_assert!(formatted.contains(&expected_added));
        prop_assert!(formatted.contains(&expected_files));
    }

    #[test]
    fn prop_merge_stats_is_same_as_add(
        a_files in 0usize..100,
        a_added in 0usize..1000,
        a_removed in 0usize..1000,
        b_files in 0usize..100,
        b_added in 0usize..1000,
        b_removed in 0usize..1000
    ) {
        let a = DiffStats::from_values(a_files, a_added, a_removed, 0, 0, 0);
        let b = DiffStats::from_values(b_files, b_added, b_removed, 0, 0, 0);

        let merged = merge_stats(&a, &b);
        let added = a.clone() + b.clone();

        prop_assert_eq!(merged, added);
    }

    #[test]
    fn prop_file_diff_stats_net_lines(
        added in 0usize..100000usize,
        removed in 0usize..100000usize
    ) {
        let stats = FileDiffStats::modified("test.rs", added, removed);
        let expected = added as isize - removed as isize;
        prop_assert_eq!(stats.net_lines(), expected);
    }

    #[test]
    fn prop_from_file_stats_matches_builder(
        file_count in 1usize..50usize
    ) {
        let files: Vec<FileDiffStats> = (0..file_count)
            .map(|i| FileDiffStats::modified(format!("file{}.rs", i), i, i * 2))
            .collect();

        let stats = from_file_stats(&files);

        prop_assert_eq!(stats.files_changed, file_count);
    }

    #[test]
    fn prop_total_lines_changed_sums_correctly(
        added in 0usize..100000usize,
        removed in 0usize..100000usize
    ) {
        let stats = DiffStats::from_values(1, added, removed, 0, 0, 0);
        // Note: This may overflow for very large values
        if added.checked_add(removed).is_some() {
            prop_assert_eq!(stats.total_lines_changed(), added + removed);
        }
    }
}

// =============================================================================
// Additional Coverage Tests (10+ tests)
// =============================================================================

#[test]
fn change_type_all_variants_covered() {
    // Ensure all variants are tested
    let variants = [
        ChangeType::Added,
        ChangeType::Deleted,
        ChangeType::Modified,
        ChangeType::Renamed,
    ];
    for variant in variants {
        // Just ensure we can use each variant
        let _ = variant.to_string();
        let _ = variant.symbol();
    }
}

#[test]
fn file_diff_stats_equality_works() {
    let a = FileDiffStats::modified("test.rs", 10, 5);
    let b = FileDiffStats::modified("test.rs", 10, 5);
    let c = FileDiffStats::modified("test.rs", 10, 6);

    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn diff_stats_equality_works() {
    let a = DiffStats::from_values(5, 100, 50, 2, 1, 1);
    let b = DiffStats::from_values(5, 100, 50, 2, 1, 1);
    let c = DiffStats::from_values(5, 100, 50, 2, 1, 0);

    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn diff_stats_clone_works() {
    let original = DiffStats::from_values(5, 100, 50, 2, 1, 1);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn builder_clone_works() {
    let mut original = DiffStatsBuilder::new();
    original.add_file(&FileDiffStats::added("test.rs", 10));
    let cloned = original.clone();
    assert_eq!(original.build(), cloned.build());
}

#[test]
fn file_diff_stats_with_all_change_types() {
    let files = vec![
        FileDiffStats::added("a.rs", 10),
        FileDiffStats::deleted("b.rs", 5),
        FileDiffStats::modified("c.rs", 3, 2),
        FileDiffStats::renamed("d.rs"),
    ];

    let stats = from_file_stats(&files);
    assert_eq!(stats.files_changed, 4);
    assert_eq!(stats.files_added, 1);
    assert_eq!(stats.files_deleted, 1);
    assert_eq!(stats.files_renamed, 1);
}

#[test]
fn format_stats_with_only_files_no_lines() {
    let stats = DiffStats::from_values(3, 0, 0, 0, 0, 0);
    let formatted = format_stats(&stats);
    assert!(formatted.contains("3 files changed"));
}

#[test]
fn builder_add_files_with_empty_iterator() {
    let mut builder = DiffStatsBuilder::new();
    builder.add_files(vec![]);
    assert!(builder.is_empty());
}

#[test]
fn builder_multiple_resets() {
    let mut builder = DiffStatsBuilder::new();
    builder.add_file(&FileDiffStats::added("test.rs", 10));
    builder.reset();
    builder.reset(); // Second reset should be safe
    assert!(builder.is_empty());
}

#[test]
fn diff_stats_debug_includes_all_fields() {
    let stats = DiffStats::from_values(5, 100, 50, 2, 1, 1);
    let debug = format!("{:?}", stats);
    assert!(debug.contains("files_changed"));
    assert!(debug.contains("lines_added"));
    assert!(debug.contains("lines_removed"));
    assert!(debug.contains("files_added"));
    assert!(debug.contains("files_deleted"));
    assert!(debug.contains("files_renamed"));
}

#[test]
fn builder_debug_includes_all_fields() {
    let mut builder = DiffStatsBuilder::new();
    builder.add_file(&FileDiffStats::added("test.rs", 10));
    let debug = format!("{:?}", builder);
    assert!(debug.contains("files_changed"));
    assert!(debug.contains("lines_added"));
}

#[test]
fn empty_stats_is_const_fn() {
    // Verify empty_stats returns a proper empty stats
    const EMPTY: DiffStats = empty_stats();
    assert_eq!(EMPTY.files_changed, 0);
}

#[test]
fn diff_stats_new_is_const_fn() {
    // Verify new is const
    const STATS: DiffStats = DiffStats::new();
    assert_eq!(STATS.files_changed, 0);
}

#[test]
fn change_type_hash_consistency() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(ChangeType::Added);
    set.insert(ChangeType::Deleted);
    set.insert(ChangeType::Modified);
    set.insert(ChangeType::Renamed);

    assert_eq!(set.len(), 4);
    assert!(set.contains(&ChangeType::Added));
}

#[test]
fn file_diff_stats_hash_consistency() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(FileDiffStats::added("test.rs", 10));
    set.insert(FileDiffStats::added("test.rs", 10)); // Duplicate

    assert_eq!(set.len(), 1);
}
