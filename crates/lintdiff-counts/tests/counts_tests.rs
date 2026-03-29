//! Comprehensive tests for lintdiff-counts crate.

use lintdiff_counts::*;

// =============================================================================
// SeverityCounts Creation and Methods (12 tests)
// =============================================================================

#[test]
fn severity_counts_new_creates_zeroed_counts() {
    let counts = SeverityCounts::new();
    assert_eq!(counts.hints, 0);
    assert_eq!(counts.notes, 0);
    assert_eq!(counts.warnings, 0);
    assert_eq!(counts.errors, 0);
    assert_eq!(counts.fatals, 0);
}

#[test]
fn severity_counts_default_equals_new() {
    let counts_default = SeverityCounts::default();
    let counts_new = SeverityCounts::new();
    assert_eq!(counts_default, counts_new);
}

#[test]
fn severity_counts_from_values_sets_all_fields() {
    let counts = SeverityCounts::from_values(10, 20, 30, 40, 50);
    assert_eq!(counts.hints, 10);
    assert_eq!(counts.notes, 20);
    assert_eq!(counts.warnings, 30);
    assert_eq!(counts.errors, 40);
    assert_eq!(counts.fatals, 50);
}

#[test]
fn severity_counts_from_values_accepts_zeros() {
    let counts = SeverityCounts::from_values(0, 0, 0, 0, 0);
    assert!(counts.is_empty());
}

#[test]
fn severity_counts_total_sums_all_fields() {
    let counts = SeverityCounts::from_values(1, 2, 3, 4, 5);
    assert_eq!(counts.total(), 15);
}

#[test]
fn severity_counts_total_handles_large_values() {
    let counts = SeverityCounts::from_values(
        u64::MAX / 5,
        u64::MAX / 5,
        u64::MAX / 5,
        u64::MAX / 5,
        u64::MAX / 5,
    );
    assert!(counts.total() > 0);
}

#[test]
fn severity_counts_is_empty_true_when_all_zero() {
    let counts = SeverityCounts::new();
    assert!(counts.is_empty());
}

#[test]
fn severity_counts_is_empty_false_when_any_nonzero() {
    assert!(!SeverityCounts::from_values(1, 0, 0, 0, 0).is_empty());
    assert!(!SeverityCounts::from_values(0, 1, 0, 0, 0).is_empty());
    assert!(!SeverityCounts::from_values(0, 0, 1, 0, 0).is_empty());
    assert!(!SeverityCounts::from_values(0, 0, 0, 1, 0).is_empty());
    assert!(!SeverityCounts::from_values(0, 0, 0, 0, 1).is_empty());
}

#[test]
fn severity_counts_problems_returns_warning_plus_error_plus_fatal() {
    let counts = SeverityCounts::from_values(1, 2, 3, 4, 5);
    assert_eq!(counts.problems(), 3 + 4 + 5);
}

#[test]
fn severity_counts_blocking_returns_error_plus_fatal() {
    let counts = SeverityCounts::from_values(1, 2, 3, 4, 5);
    assert_eq!(counts.blocking(), 4 + 5);
}

#[test]
fn severity_counts_has_blocking_true_when_errors_or_fatals() {
    assert!(!SeverityCounts::from_values(0, 0, 5, 0, 0).has_blocking());
    assert!(SeverityCounts::from_values(0, 0, 0, 1, 0).has_blocking());
    assert!(SeverityCounts::from_values(0, 0, 0, 0, 1).has_blocking());
    assert!(SeverityCounts::from_values(0, 0, 0, 1, 1).has_blocking());
}

#[test]
fn severity_counts_has_problems_true_when_warnings_errors_or_fatals() {
    assert!(!SeverityCounts::from_values(5, 5, 0, 0, 0).has_problems());
    assert!(SeverityCounts::from_values(0, 0, 1, 0, 0).has_problems());
    assert!(SeverityCounts::from_values(0, 0, 0, 1, 0).has_problems());
    assert!(SeverityCounts::from_values(0, 0, 0, 0, 1).has_problems());
}

// =============================================================================
// SeverityCounts Arithmetic (8 tests)
// =============================================================================

#[test]
fn severity_counts_add_combines_all_fields() {
    let a = SeverityCounts::from_values(1, 2, 3, 4, 5);
    let b = SeverityCounts::from_values(10, 20, 30, 40, 50);
    let c = a + b;
    assert_eq!(c.hints, 11);
    assert_eq!(c.notes, 22);
    assert_eq!(c.warnings, 33);
    assert_eq!(c.errors, 44);
    assert_eq!(c.fatals, 55);
}

#[test]
fn severity_counts_add_with_zero_is_identity() {
    let a = SeverityCounts::from_values(1, 2, 3, 4, 5);
    let zero = SeverityCounts::new();
    let result = a.clone() + zero;
    assert_eq!(result, a);
}

#[test]
fn severity_counts_add_is_commutative() {
    let a = SeverityCounts::from_values(1, 2, 3, 4, 5);
    let b = SeverityCounts::from_values(10, 20, 30, 40, 50);
    assert_eq!(a.clone() + b.clone(), b + a);
}

#[test]
fn severity_counts_add_assign_modifies_in_place() {
    let mut a = SeverityCounts::from_values(1, 2, 3, 4, 5);
    a += SeverityCounts::from_values(1, 1, 1, 1, 1);
    assert_eq!(a, SeverityCounts::from_values(2, 3, 4, 5, 6));
}

#[test]
fn severity_counts_add_assign_with_zero_is_noop() {
    let mut a = SeverityCounts::from_values(1, 2, 3, 4, 5);
    let expected = a.clone();
    a += SeverityCounts::new();
    assert_eq!(a, expected);
}

#[test]
fn severity_counts_increment_increases_correct_field() {
    let mut counts = SeverityCounts::new();
    counts.increment(SeverityLevel::Hint);
    assert_eq!(counts.hints, 1);
    counts.increment(SeverityLevel::Note);
    assert_eq!(counts.notes, 1);
    counts.increment(SeverityLevel::Warning);
    assert_eq!(counts.warnings, 1);
    counts.increment(SeverityLevel::Error);
    assert_eq!(counts.errors, 1);
    counts.increment(SeverityLevel::Fatal);
    assert_eq!(counts.fatals, 1);
}

#[test]
fn severity_counts_increment_multiple_times() {
    let mut counts = SeverityCounts::new();
    for _ in 0..10 {
        counts.increment(SeverityLevel::Warning);
    }
    assert_eq!(counts.warnings, 10);
    assert_eq!(counts.total(), 10);
}

#[test]
fn severity_counts_get_returns_correct_value() {
    let counts = SeverityCounts::from_values(100, 200, 300, 400, 500);
    assert_eq!(counts.get(SeverityLevel::Hint), 100);
    assert_eq!(counts.get(SeverityLevel::Note), 200);
    assert_eq!(counts.get(SeverityLevel::Warning), 300);
    assert_eq!(counts.get(SeverityLevel::Error), 400);
    assert_eq!(counts.get(SeverityLevel::Fatal), 500);
}

// =============================================================================
// SeverityCounts pass_rate (5 tests)
// =============================================================================

#[test]
fn pass_rate_returns_none_when_empty() {
    let counts = SeverityCounts::new();
    assert!(counts.pass_rate().is_none());
}

#[test]
fn pass_rate_is_one_when_only_non_problems() {
    let counts = SeverityCounts::from_values(5, 5, 0, 0, 0);
    assert_eq!(counts.pass_rate(), Some(1.0));
}

#[test]
fn pass_rate_is_zero_when_only_problems() {
    let counts = SeverityCounts::from_values(0, 0, 5, 5, 5);
    assert_eq!(counts.pass_rate(), Some(0.0));
}

#[test]
fn pass_rate_calculates_correctly_for_mixed() {
    // 2 hints + 2 notes = 4 non-problems, 10 total = 0.4
    let counts = SeverityCounts::from_values(2, 2, 2, 2, 2);
    let rate = counts.pass_rate().expect("should have rate");
    assert!((rate - 0.4).abs() < 1e-10);
}

#[test]
fn pass_rate_with_only_hints() {
    let counts = SeverityCounts::from_values(10, 0, 0, 0, 0);
    assert_eq!(counts.pass_rate(), Some(1.0));
}

// =============================================================================
// FileCounts Operations (10 tests)
// =============================================================================

#[test]
fn file_counts_new_is_empty() {
    let fc = FileCounts::new();
    assert!(fc.is_empty());
    assert_eq!(fc.file_count(), 0);
    assert_eq!(fc.total().total(), 0);
}

#[test]
fn file_counts_default_equals_new() {
    let fc_default = FileCounts::default();
    let fc_new = FileCounts::new();
    assert_eq!(fc_default.file_count(), fc_new.file_count());
    assert_eq!(fc_default.total(), fc_new.total());
}

#[test]
fn file_counts_add_file_increases_file_count() {
    let mut fc = FileCounts::new();
    fc.add_file("src/main.rs", SeverityCounts::from_values(1, 2, 3, 4, 5));
    assert_eq!(fc.file_count(), 1);
}

#[test]
fn file_counts_add_file_updates_total() {
    let mut fc = FileCounts::new();
    fc.add_file("src/main.rs", SeverityCounts::from_values(1, 2, 3, 4, 5));
    assert_eq!(fc.total().total(), 15);
}

#[test]
fn file_counts_add_same_file_accumulates() {
    let mut fc = FileCounts::new();
    fc.add_file("src/main.rs", SeverityCounts::from_values(1, 0, 0, 0, 0));
    fc.add_file("src/main.rs", SeverityCounts::from_values(0, 2, 0, 0, 0));
    assert_eq!(fc.file_count(), 1);
    let counts = fc.get("src/main.rs").expect("file should exist");
    assert_eq!(counts.hints, 1);
    assert_eq!(counts.notes, 2);
}

#[test]
fn file_counts_increment_single_severity() {
    let mut fc = FileCounts::new();
    fc.increment("src/lib.rs", SeverityLevel::Error);
    assert_eq!(fc.total().errors, 1);
    let counts = fc.get("src/lib.rs").expect("file should exist");
    assert_eq!(counts.errors, 1);
}

#[test]
fn file_counts_get_returns_none_for_missing_file() {
    let fc = FileCounts::new();
    assert!(fc.get("nonexistent.rs").is_none());
}

#[test]
fn file_counts_files_iterator_yields_all_files() {
    let mut fc = FileCounts::new();
    fc.add_file("a.rs", SeverityCounts::new());
    fc.add_file("b.rs", SeverityCounts::new());
    fc.add_file("c.rs", SeverityCounts::new());
    let files: Vec<_> = fc.files().map(|(path, _)| path).collect();
    assert_eq!(files.len(), 3);
    assert!(files.contains(&&String::from("a.rs")));
    assert!(files.contains(&&String::from("b.rs")));
    assert!(files.contains(&&String::from("c.rs")));
}

#[test]
fn file_counts_multiple_files_with_different_counts() {
    let mut fc = FileCounts::new();
    fc.add_file("a.rs", SeverityCounts::from_values(1, 0, 0, 0, 0));
    fc.add_file("b.rs", SeverityCounts::from_values(0, 2, 0, 0, 0));
    fc.add_file("c.rs", SeverityCounts::from_values(0, 0, 3, 0, 0));

    assert_eq!(fc.file_count(), 3);
    assert_eq!(fc.total().hints, 1);
    assert_eq!(fc.total().notes, 2);
    assert_eq!(fc.total().warnings, 3);
}

#[test]
fn file_counts_is_empty_false_after_add() {
    let mut fc = FileCounts::new();
    fc.add_file("a.rs", SeverityCounts::from_values(1, 0, 0, 0, 0));
    assert!(!fc.is_empty());
}

#[test]
fn file_counts_clone_preserves_data() {
    let mut fc = FileCounts::new();
    fc.add_file("a.rs", SeverityCounts::from_values(1, 2, 3, 4, 5));
    let fc2 = fc.clone();
    assert_eq!(fc.file_count(), fc2.file_count());
    assert_eq!(fc.total(), fc2.total());
}

// =============================================================================
// CategoryCounts Operations (10 tests)
// =============================================================================

#[test]
fn category_counts_new_is_empty() {
    let cc = CategoryCounts::new();
    assert!(cc.is_empty());
    assert_eq!(cc.category_count(), 0);
    assert_eq!(cc.total().total(), 0);
}

#[test]
fn category_counts_default_equals_new() {
    let cc_default = CategoryCounts::default();
    let cc_new = CategoryCounts::new();
    assert_eq!(cc_default.category_count(), cc_new.category_count());
    assert_eq!(cc_default.total(), cc_new.total());
}

#[test]
fn category_counts_add_category_increases_count() {
    let mut cc = CategoryCounts::new();
    cc.add_category("clippy::style", SeverityCounts::from_values(1, 2, 3, 4, 5));
    assert_eq!(cc.category_count(), 1);
}

#[test]
fn category_counts_add_category_updates_total() {
    let mut cc = CategoryCounts::new();
    cc.add_category("clippy::style", SeverityCounts::from_values(1, 2, 3, 4, 5));
    assert_eq!(cc.total().total(), 15);
}

#[test]
fn category_counts_add_same_category_accumulates() {
    let mut cc = CategoryCounts::new();
    cc.add_category("clippy::style", SeverityCounts::from_values(1, 0, 0, 0, 0));
    cc.add_category("clippy::style", SeverityCounts::from_values(0, 2, 0, 0, 0));
    assert_eq!(cc.category_count(), 1);
    let counts = cc.get("clippy::style").expect("category should exist");
    assert_eq!(counts.hints, 1);
    assert_eq!(counts.notes, 2);
}

#[test]
fn category_counts_increment_single_severity() {
    let mut cc = CategoryCounts::new();
    cc.increment("rustc::unused", SeverityLevel::Warning);
    assert_eq!(cc.total().warnings, 1);
    let counts = cc.get("rustc::unused").expect("category should exist");
    assert_eq!(counts.warnings, 1);
}

#[test]
fn category_counts_get_returns_none_for_missing_category() {
    let cc = CategoryCounts::new();
    assert!(cc.get("nonexistent").is_none());
}

#[test]
fn category_counts_categories_iterator_yields_all() {
    let mut cc = CategoryCounts::new();
    cc.add_category("cat1", SeverityCounts::new());
    cc.add_category("cat2", SeverityCounts::new());
    cc.add_category("cat3", SeverityCounts::new());
    let cats: Vec<_> = cc.categories().map(|(name, _)| name).collect();
    assert_eq!(cats.len(), 3);
    assert!(cats.contains(&&String::from("cat1")));
    assert!(cats.contains(&&String::from("cat2")));
    assert!(cats.contains(&&String::from("cat3")));
}

#[test]
fn category_counts_multiple_categories_with_different_counts() {
    let mut cc = CategoryCounts::new();
    cc.add_category("cat1", SeverityCounts::from_values(1, 0, 0, 0, 0));
    cc.add_category("cat2", SeverityCounts::from_values(0, 2, 0, 0, 0));
    cc.add_category("cat3", SeverityCounts::from_values(0, 0, 3, 0, 0));

    assert_eq!(cc.category_count(), 3);
    assert_eq!(cc.total().hints, 1);
    assert_eq!(cc.total().notes, 2);
    assert_eq!(cc.total().warnings, 3);
}

#[test]
fn category_counts_is_empty_false_after_add() {
    let mut cc = CategoryCounts::new();
    cc.add_category("cat", SeverityCounts::from_values(1, 0, 0, 0, 0));
    assert!(!cc.is_empty());
}

#[test]
fn category_counts_clone_preserves_data() {
    let mut cc = CategoryCounts::new();
    cc.add_category("cat", SeverityCounts::from_values(1, 2, 3, 4, 5));
    let cc2 = cc.clone();
    assert_eq!(cc.category_count(), cc2.category_count());
    assert_eq!(cc.total(), cc2.total());
}

// =============================================================================
// CountSummary Operations (10 tests)
// =============================================================================

#[test]
fn count_summary_new_is_empty() {
    let summary = CountSummary::new();
    assert!(summary.is_empty());
    assert_eq!(summary.total(), 0);
    assert!(!summary.has_blocking());
}

#[test]
fn count_summary_default_equals_new() {
    let summary_default = CountSummary::default();
    let summary_new = CountSummary::new();
    assert_eq!(summary_default.total(), summary_new.total());
    assert_eq!(summary_default.is_empty(), summary_new.is_empty());
}

#[test]
fn count_summary_record_increases_total() {
    let mut summary = CountSummary::new();
    summary.record("a.rs", "cat", SeverityLevel::Warning);
    assert_eq!(summary.total(), 1);
}

#[test]
fn count_summary_record_updates_all_components() {
    let mut summary = CountSummary::new();
    summary.record("a.rs", "cat", SeverityLevel::Error);

    // Check severity counts
    assert_eq!(summary.severity.errors, 1);

    // Check file counts
    assert_eq!(summary.by_file.file_count(), 1);
    let file_counts = summary.by_file.get("a.rs").expect("file should exist");
    assert_eq!(file_counts.errors, 1);

    // Check category counts
    assert_eq!(summary.by_category.category_count(), 1);
    let cat_counts = summary
        .by_category
        .get("cat")
        .expect("category should exist");
    assert_eq!(cat_counts.errors, 1);
}

#[test]
fn count_summary_record_multiple_same_file() {
    let mut summary = CountSummary::new();
    summary.record("a.rs", "cat1", SeverityLevel::Hint);
    summary.record("a.rs", "cat2", SeverityLevel::Warning);

    assert_eq!(summary.total(), 2);
    assert_eq!(summary.by_file.file_count(), 1);
    assert_eq!(summary.by_category.category_count(), 2);
}

#[test]
fn count_summary_has_blocking_true_after_error() {
    let mut summary = CountSummary::new();
    assert!(!summary.has_blocking());
    summary.record("a.rs", "cat", SeverityLevel::Error);
    assert!(summary.has_blocking());
}

#[test]
fn count_summary_has_blocking_true_after_fatal() {
    let mut summary = CountSummary::new();
    summary.record("a.rs", "cat", SeverityLevel::Fatal);
    assert!(summary.has_blocking());
}

#[test]
fn count_summary_has_blocking_false_with_only_warnings() {
    let mut summary = CountSummary::new();
    summary.record("a.rs", "cat", SeverityLevel::Warning);
    assert!(!summary.has_blocking());
}

#[test]
fn count_summary_merge_combines_all_data() {
    let mut a = CountSummary::new();
    a.record("a.rs", "cat1", SeverityLevel::Hint);

    let mut b = CountSummary::new();
    b.record("b.rs", "cat2", SeverityLevel::Error);

    a.merge(b);

    assert_eq!(a.total(), 2);
    assert_eq!(a.by_file.file_count(), 2);
    assert_eq!(a.by_category.category_count(), 2);
    assert!(a.has_blocking());
}

#[test]
fn count_summary_merge_same_file_accumulates() {
    let mut a = CountSummary::new();
    a.record("a.rs", "cat1", SeverityLevel::Hint);

    let mut b = CountSummary::new();
    b.record("a.rs", "cat2", SeverityLevel::Error);

    a.merge(b);

    assert_eq!(a.by_file.file_count(), 1);
    let file_counts = a.by_file.get("a.rs").expect("file should exist");
    assert_eq!(file_counts.hints, 1);
    assert_eq!(file_counts.errors, 1);
}

// =============================================================================
// Display Formatting (5 tests)
// =============================================================================

#[test]
fn severity_counts_display_formats_correctly() {
    let counts = SeverityCounts::from_values(1, 2, 3, 4, 5);
    let s = format!("{counts}");
    assert_eq!(s, "1 hints, 2 notes, 3 warnings, 4 errors, 5 fatals");
}

#[test]
fn severity_counts_display_with_zeros() {
    let counts = SeverityCounts::new();
    let s = format!("{counts}");
    assert_eq!(s, "0 hints, 0 notes, 0 warnings, 0 errors, 0 fatals");
}

#[test]
fn count_summary_display_formats_correctly() {
    let mut summary = CountSummary::new();
    summary.record("a.rs", "cat1", SeverityLevel::Hint);
    summary.record("b.rs", "cat2", SeverityLevel::Error);

    let s = format!("{summary}");
    assert_eq!(
        s,
        "2 files, 2 categories, 1 hints, 0 notes, 0 warnings, 1 errors, 0 fatals"
    );
}

#[test]
fn count_summary_display_empty() {
    let summary = CountSummary::new();
    let s = format!("{summary}");
    assert_eq!(
        s,
        "0 files, 0 categories, 0 hints, 0 notes, 0 warnings, 0 errors, 0 fatals"
    );
}

#[test]
fn severity_level_debug_formats_correctly() {
    assert_eq!(format!("{:?}", SeverityLevel::Hint), "Hint");
    assert_eq!(format!("{:?}", SeverityLevel::Note), "Note");
    assert_eq!(format!("{:?}", SeverityLevel::Warning), "Warning");
    assert_eq!(format!("{:?}", SeverityLevel::Error), "Error");
    assert_eq!(format!("{:?}", SeverityLevel::Fatal), "Fatal");
}

// =============================================================================
// Additional Edge Cases and Property Tests
// =============================================================================

#[test]
fn severity_level_copy_trait_works() {
    let level = SeverityLevel::Warning;
    let level_copy = level;
    assert_eq!(level, level_copy);
}

#[test]
fn severity_level_clone_works() {
    let level = SeverityLevel::Error;
    let level_clone = level.clone();
    assert_eq!(level, level_clone);
}

#[test]
fn severity_level_equality() {
    assert_eq!(SeverityLevel::Hint, SeverityLevel::Hint);
    assert_eq!(SeverityLevel::Note, SeverityLevel::Note);
    assert_eq!(SeverityLevel::Warning, SeverityLevel::Warning);
    assert_eq!(SeverityLevel::Error, SeverityLevel::Error);
    assert_eq!(SeverityLevel::Fatal, SeverityLevel::Fatal);

    assert_ne!(SeverityLevel::Hint, SeverityLevel::Note);
    assert_ne!(SeverityLevel::Warning, SeverityLevel::Error);
}

#[test]
fn severity_counts_equality() {
    let a = SeverityCounts::from_values(1, 2, 3, 4, 5);
    let b = SeverityCounts::from_values(1, 2, 3, 4, 5);
    let c = SeverityCounts::from_values(5, 4, 3, 2, 1);

    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn severity_counts_clone() {
    let original = SeverityCounts::from_values(1, 2, 3, 4, 5);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn file_counts_clone() {
    let mut original = FileCounts::new();
    original.add_file("a.rs", SeverityCounts::from_values(1, 2, 3, 4, 5));
    let cloned = original.clone();
    assert_eq!(original.file_count(), cloned.file_count());
    assert_eq!(original.total(), cloned.total());
}

#[test]
fn category_counts_clone() {
    let mut original = CategoryCounts::new();
    original.add_category("cat", SeverityCounts::from_values(1, 2, 3, 4, 5));
    let cloned = original.clone();
    assert_eq!(original.category_count(), cloned.category_count());
    assert_eq!(original.total(), cloned.total());
}

#[test]
fn count_summary_clone() {
    let mut original = CountSummary::new();
    original.record("a.rs", "cat", SeverityLevel::Warning);
    let cloned = original.clone();
    assert_eq!(original.total(), cloned.total());
    assert_eq!(original.has_blocking(), cloned.has_blocking());
}

#[test]
fn large_counts_handling() {
    let counts = SeverityCounts::from_values(1_000_000, 2_000_000, 3_000_000, 4_000_000, 5_000_000);
    assert_eq!(counts.total(), 15_000_000);
    assert_eq!(counts.problems(), 12_000_000);
    assert_eq!(counts.blocking(), 9_000_000);
}

#[test]
fn file_counts_with_path_containing_special_chars() {
    let mut fc = FileCounts::new();
    fc.add_file(
        "src/path/with spaces/file.rs",
        SeverityCounts::from_values(1, 0, 0, 0, 0),
    );
    fc.add_file(
        "src/unicode/日本語.rs",
        SeverityCounts::from_values(1, 0, 0, 0, 0),
    );
    assert_eq!(fc.file_count(), 2);
}

#[test]
fn category_counts_with_special_names() {
    let mut cc = CategoryCounts::new();
    cc.add_category(
        "clippy::result_unit_err",
        SeverityCounts::from_values(1, 0, 0, 0, 0),
    );
    cc.add_category("rustc::E0433", SeverityCounts::from_values(0, 1, 0, 0, 0));
    assert_eq!(cc.category_count(), 2);
}

#[test]
fn count_summary_complex_scenario() {
    let mut summary = CountSummary::new();

    // Record multiple findings across files and categories
    summary.record("src/main.rs", "clippy::style", SeverityLevel::Warning);
    summary.record("src/main.rs", "clippy::style", SeverityLevel::Warning);
    summary.record("src/main.rs", "clippy::correctness", SeverityLevel::Error);
    summary.record("src/lib.rs", "rustc::unused", SeverityLevel::Warning);
    summary.record("src/lib.rs", "rustc::unused", SeverityLevel::Hint);
    summary.record("src/utils.rs", "clippy::perf", SeverityLevel::Fatal);

    assert_eq!(summary.total(), 6);
    assert_eq!(summary.by_file.file_count(), 3);
    assert_eq!(summary.by_category.category_count(), 4);
    assert!(summary.has_blocking());
    assert_eq!(summary.severity.warnings, 3);
    assert_eq!(summary.severity.errors, 1);
    assert_eq!(summary.severity.fatals, 1);
    assert_eq!(summary.severity.hints, 1);
}

#[test]
fn count_summary_merge_empty() {
    let mut a = CountSummary::new();
    a.record("a.rs", "cat", SeverityLevel::Hint);

    let b = CountSummary::new();

    a.merge(b);

    assert_eq!(a.total(), 1);
}

#[test]
fn count_summary_merge_into_empty() {
    let mut a = CountSummary::new();

    let mut b = CountSummary::new();
    b.record("b.rs", "cat", SeverityLevel::Error);

    a.merge(b);

    assert_eq!(a.total(), 1);
    assert!(a.has_blocking());
}

#[test]
fn file_counts_increment_multiple_severities() {
    let mut fc = FileCounts::new();
    fc.increment("a.rs", SeverityLevel::Hint);
    fc.increment("a.rs", SeverityLevel::Hint);
    fc.increment("a.rs", SeverityLevel::Warning);
    fc.increment("a.rs", SeverityLevel::Error);

    let counts = fc.get("a.rs").expect("file should exist");
    assert_eq!(counts.hints, 2);
    assert_eq!(counts.warnings, 1);
    assert_eq!(counts.errors, 1);
    assert_eq!(fc.total().total(), 4);
}

#[test]
fn category_counts_increment_multiple_severities() {
    let mut cc = CategoryCounts::new();
    cc.increment("cat", SeverityLevel::Note);
    cc.increment("cat", SeverityLevel::Note);
    cc.increment("cat", SeverityLevel::Warning);
    cc.increment("cat", SeverityLevel::Fatal);

    let counts = cc.get("cat").expect("category should exist");
    assert_eq!(counts.notes, 2);
    assert_eq!(counts.warnings, 1);
    assert_eq!(counts.fatals, 1);
    assert_eq!(cc.total().total(), 4);
}

#[test]
fn severity_counts_get_all_levels() {
    let counts = SeverityCounts::from_values(11, 22, 33, 44, 55);
    assert_eq!(counts.get(SeverityLevel::Hint), 11);
    assert_eq!(counts.get(SeverityLevel::Note), 22);
    assert_eq!(counts.get(SeverityLevel::Warning), 33);
    assert_eq!(counts.get(SeverityLevel::Error), 44);
    assert_eq!(counts.get(SeverityLevel::Fatal), 55);
}

#[test]
fn pass_rate_precision() {
    // Test with values that could expose floating point issues
    let counts = SeverityCounts::from_values(3, 1, 0, 0, 0); // 4/4 = 1.0
    let rate = counts.pass_rate().expect("should have rate");
    assert!((rate - 1.0).abs() < f64::EPSILON);

    let counts2 = SeverityCounts::from_values(1, 1, 1, 1, 0); // 2/4 = 0.5
    let rate2 = counts2.pass_rate().expect("should have rate");
    assert!((rate2 - 0.5).abs() < f64::EPSILON);
}
