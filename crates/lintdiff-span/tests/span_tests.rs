//! Comprehensive tests for lintdiff-span crate.
//!
//! Test coverage:
//! 1. Position creation and methods (10 tests)
//! 2. Position comparisons (8 tests)
//! 3. Span creation methods (12 tests)
//! 4. Span contains_line/contains_position (8 tests)
//! 5. Span overlaps/intersection (10 tests)
//! 6. Span merge/expand (8 tests)
//! 7. Span display formatting (6 tests)
//! 8. FileSpan methods (8 tests)

use lintdiff_span::{FileSpan, Position, Span};
use std::cmp::Ordering;

// =============================================================================
// 1. Position creation and methods (10 tests)
// =============================================================================

#[test]
fn position_new_creates_position_with_given_values() {
    let pos = Position::new(42, 17);
    assert_eq!(pos.line, 42);
    assert_eq!(pos.column, 17);
}

#[test]
fn position_new_works_with_zero_values() {
    let pos = Position::new(0, 0);
    assert_eq!(pos.line, 0);
    assert_eq!(pos.column, 0);
}

#[test]
fn position_new_works_with_large_values() {
    let pos = Position::new(u32::MAX, u32::MAX);
    assert_eq!(pos.line, u32::MAX);
    assert_eq!(pos.column, u32::MAX);
}

#[test]
fn position_start_of_line_creates_position_at_column_1() {
    let pos = Position::start_of_line(10);
    assert_eq!(pos.line, 10);
    assert_eq!(pos.column, 1);
}

#[test]
fn position_start_of_line_works_for_line_1() {
    let pos = Position::start_of_line(1);
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
}

#[test]
fn position_start_creates_position_at_1_1() {
    let pos = Position::start();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
}

#[test]
fn position_is_start_of_line_returns_true_for_column_1() {
    let pos = Position::new(5, 1);
    assert!(pos.is_start_of_line());
}

#[test]
fn position_is_start_of_line_returns_false_for_other_columns() {
    let pos = Position::new(5, 2);
    assert!(!pos.is_start_of_line());
}

#[test]
fn position_is_start_of_line_returns_true_for_start() {
    let pos = Position::start();
    assert!(pos.is_start_of_line());
}

#[test]
fn position_is_start_of_line_returns_true_for_start_of_line() {
    let pos = Position::start_of_line(100);
    assert!(pos.is_start_of_line());
}

// =============================================================================
// 2. Position comparisons (8 tests)
// =============================================================================

#[test]
fn position_cmp_by_line_returns_less_for_smaller_line() {
    let pos1 = Position::new(3, 10);
    let pos2 = Position::new(5, 1);
    assert_eq!(pos1.cmp_by_line(&pos2), Ordering::Less);
}

#[test]
fn position_cmp_by_line_returns_greater_for_larger_line() {
    let pos1 = Position::new(10, 1);
    let pos2 = Position::new(5, 100);
    assert_eq!(pos1.cmp_by_line(&pos2), Ordering::Greater);
}

#[test]
fn position_cmp_by_line_returns_equal_for_same_line() {
    let pos1 = Position::new(5, 1);
    let pos2 = Position::new(5, 100);
    assert_eq!(pos1.cmp_by_line(&pos2), Ordering::Equal);
}

#[test]
fn position_partial_ord_compares_by_line_then_column() {
    let pos1 = Position::new(3, 5);
    let pos2 = Position::new(3, 10);
    assert!(pos1 < pos2);
}

#[test]
fn position_partial_ord_compares_by_line_first() {
    let pos1 = Position::new(3, 100);
    let pos2 = Position::new(4, 1);
    assert!(pos1 < pos2);
}

#[test]
fn position_equal_positions_are_equal() {
    let pos1 = Position::new(5, 10);
    let pos2 = Position::new(5, 10);
    assert_eq!(pos1, pos2);
}

#[test]
fn position_default_returns_start() {
    let pos = Position::default();
    assert_eq!(pos, Position::start());
}

#[test]
fn position_display_formats_correctly() {
    let pos = Position::new(42, 17);
    assert_eq!(format!("{pos}"), "42:17");
}

// =============================================================================
// 3. Span creation methods (12 tests)
// =============================================================================

#[test]
fn span_new_creates_span_with_given_positions() {
    let start = Position::new(1, 1);
    let end = Position::new(5, 10);
    let span = Span::new(start, end);
    assert_eq!(span.start, start);
    assert_eq!(span.end, end);
}

#[test]
fn span_new_accepts_equal_start_and_end() {
    let pos = Position::new(5, 10);
    let span = Span::new(pos, pos);
    assert_eq!(span.start, pos);
    assert_eq!(span.end, pos);
}

#[test]
#[should_panic(expected = "Span start must be <= end")]
fn span_new_panics_when_start_greater_than_end() {
    let start = Position::new(10, 5);
    let end = Position::new(5, 10);
    let _ = Span::new(start, end);
}

#[test]
#[should_panic(expected = "Span start must be <= end")]
fn span_new_panics_when_same_line_but_start_column_greater() {
    let start = Position::new(5, 20);
    let end = Position::new(5, 10);
    let _ = Span::new(start, end);
}

#[test]
fn span_single_line_creates_span_on_one_line() {
    let span = Span::single_line(10, 5, 20);
    assert_eq!(span.start.line, 10);
    assert_eq!(span.start.column, 5);
    assert_eq!(span.end.line, 10);
    assert_eq!(span.end.column, 20);
}

#[test]
fn span_full_line_creates_span_covering_entire_line() {
    let span = Span::full_line(15);
    assert_eq!(span.start.line, 15);
    assert_eq!(span.start.column, 1);
    assert_eq!(span.end.line, 15);
    assert_eq!(span.end.column, u32::MAX);
}

#[test]
fn span_full_lines_creates_span_covering_multiple_lines() {
    let span = Span::full_lines(5, 10);
    assert_eq!(span.start.line, 5);
    assert_eq!(span.start.column, 1);
    assert_eq!(span.end.line, 10);
    assert_eq!(span.end.column, u32::MAX);
}

#[test]
fn span_point_creates_zero_width_span() {
    let span = Span::point(10, 5);
    assert_eq!(span.start, span.end);
    assert_eq!(span.start.line, 10);
    assert_eq!(span.start.column, 5);
}

#[test]
fn span_empty_creates_point_at_1_1() {
    let span = Span::empty();
    assert!(span.is_point());
    assert_eq!(span.start, Position::start());
}

#[test]
fn span_default_returns_empty() {
    let span = Span::default();
    assert_eq!(span, Span::empty());
}

#[test]
fn span_is_point_returns_true_for_point_spans() {
    let span = Span::point(5, 10);
    assert!(span.is_point());
}

#[test]
fn span_is_point_returns_false_for_non_point_spans() {
    let span = Span::single_line(5, 1, 10);
    assert!(!span.is_point());
}

// =============================================================================
// 4. Span contains_line/contains_position (8 tests)
// =============================================================================

#[test]
fn span_contains_line_returns_true_for_line_within_span() {
    let span = Span::new(Position::new(3, 1), Position::new(7, 10));
    assert!(span.contains_line(3));
    assert!(span.contains_line(5));
    assert!(span.contains_line(7));
}

#[test]
fn span_contains_line_returns_false_for_line_outside_span() {
    let span = Span::new(Position::new(3, 1), Position::new(7, 10));
    assert!(!span.contains_line(2));
    assert!(!span.contains_line(8));
}

#[test]
fn span_contains_line_works_for_single_line_span() {
    let span = Span::single_line(5, 1, 10);
    assert!(span.contains_line(5));
    assert!(!span.contains_line(4));
    assert!(!span.contains_line(6));
}

#[test]
fn span_contains_line_works_for_point_span() {
    let span = Span::point(5, 10);
    assert!(span.contains_line(5));
    assert!(!span.contains_line(4));
    assert!(!span.contains_line(6));
}

#[test]
fn span_contains_position_returns_true_for_position_within_span() {
    let span = Span::new(Position::new(3, 5), Position::new(7, 10));
    assert!(span.contains_position(Position::new(3, 5)));
    assert!(span.contains_position(Position::new(5, 1)));
    assert!(span.contains_position(Position::new(7, 10)));
}

#[test]
fn span_contains_position_returns_false_for_position_outside_span() {
    let span = Span::new(Position::new(3, 5), Position::new(7, 10));
    assert!(!span.contains_position(Position::new(3, 4)));
    assert!(!span.contains_position(Position::new(7, 11)));
    assert!(!span.contains_position(Position::new(2, 10)));
    assert!(!span.contains_position(Position::new(8, 1)));
}

#[test]
fn span_contains_position_works_for_same_line_span() {
    let span = Span::single_line(5, 10, 20);
    assert!(span.contains_position(Position::new(5, 10)));
    assert!(span.contains_position(Position::new(5, 15)));
    assert!(span.contains_position(Position::new(5, 20)));
    assert!(!span.contains_position(Position::new(5, 9)));
    assert!(!span.contains_position(Position::new(5, 21)));
}

#[test]
fn span_contains_position_works_for_point_span() {
    let span = Span::point(5, 10);
    assert!(span.contains_position(Position::new(5, 10)));
    assert!(!span.contains_position(Position::new(5, 9)));
    assert!(!span.contains_position(Position::new(5, 11)));
}

// =============================================================================
// 5. Span overlaps/intersection (10 tests)
// =============================================================================

#[test]
fn span_overlaps_returns_true_for_overlapping_spans() {
    let span1 = Span::new(Position::new(3, 1), Position::new(7, 10));
    let span2 = Span::new(Position::new(5, 1), Position::new(10, 1));
    assert!(span1.overlaps(&span2));
    assert!(span2.overlaps(&span1));
}

#[test]
fn span_overlaps_returns_true_for_contained_spans() {
    let outer = Span::new(Position::new(3, 1), Position::new(10, 1));
    let inner = Span::new(Position::new(5, 1), Position::new(7, 10));
    assert!(outer.overlaps(&inner));
    assert!(inner.overlaps(&outer));
}

#[test]
fn span_overlaps_returns_true_for_identical_spans() {
    let span1 = Span::new(Position::new(3, 1), Position::new(7, 10));
    let span2 = Span::new(Position::new(3, 1), Position::new(7, 10));
    assert!(span1.overlaps(&span2));
}

#[test]
fn span_overlaps_returns_false_for_non_overlapping_spans() {
    let span1 = Span::new(Position::new(3, 1), Position::new(5, 10));
    let span2 = Span::new(Position::new(7, 1), Position::new(10, 1));
    assert!(!span1.overlaps(&span2));
    assert!(!span2.overlaps(&span1));
}

#[test]
fn span_overlaps_returns_true_for_touching_endpoints() {
    let span1 = Span::new(Position::new(3, 1), Position::new(5, 10));
    let span2 = Span::new(Position::new(5, 10), Position::new(10, 1));
    assert!(span1.overlaps(&span2));
    assert!(span2.overlaps(&span1));
}

#[test]
fn span_intersection_returns_correct_overlap() {
    let span1 = Span::new(Position::new(3, 1), Position::new(7, 10));
    let span2 = Span::new(Position::new(5, 1), Position::new(10, 1));
    let intersection = span1.intersection(&span2);
    assert!(intersection.is_some());
    let inter = intersection.unwrap();
    assert_eq!(inter.start, Position::new(5, 1));
    assert_eq!(inter.end, Position::new(7, 10));
}

#[test]
fn span_intersection_returns_none_for_non_overlapping() {
    let span1 = Span::new(Position::new(3, 1), Position::new(5, 9));
    let span2 = Span::new(Position::new(5, 11), Position::new(10, 1));
    assert!(span1.intersection(&span2).is_none());
}

#[test]
fn span_intersection_returns_correct_for_contained_span() {
    let outer = Span::new(Position::new(3, 1), Position::new(10, 1));
    let inner = Span::new(Position::new(5, 1), Position::new(7, 10));
    let intersection = outer.intersection(&inner);
    assert!(intersection.is_some());
    let inter = intersection.unwrap();
    assert_eq!(inter, inner);
}

#[test]
fn span_intersection_returns_point_for_touching_spans() {
    let span1 = Span::new(Position::new(3, 1), Position::new(5, 10));
    let span2 = Span::new(Position::new(5, 10), Position::new(10, 1));
    let intersection = span1.intersection(&span2);
    assert!(intersection.is_some());
    let inter = intersection.unwrap();
    assert!(inter.is_point());
    assert_eq!(inter.start, Position::new(5, 10));
}

#[test]
fn span_intersection_is_symmetric() {
    let span1 = Span::new(Position::new(3, 1), Position::new(7, 10));
    let span2 = Span::new(Position::new(5, 1), Position::new(10, 1));
    assert_eq!(span1.intersection(&span2), span2.intersection(&span1));
}

// =============================================================================
// 6. Span merge/expand (8 tests)
// =============================================================================

#[test]
fn span_merge_returns_merged_span_for_overlapping() {
    let span1 = Span::new(Position::new(3, 1), Position::new(7, 10));
    let span2 = Span::new(Position::new(5, 1), Position::new(10, 1));
    let merged = span1.merge(&span2);
    assert!(merged.is_some());
    let m = merged.unwrap();
    assert_eq!(m.start, Position::new(3, 1));
    assert_eq!(m.end, Position::new(10, 1));
}

#[test]
fn span_merge_returns_merged_span_for_adjacent() {
    let span1 = Span::new(Position::new(3, 1), Position::new(5, 10));
    let span2 = Span::new(Position::new(5, 10), Position::new(10, 1));
    let merged = span1.merge(&span2);
    assert!(merged.is_some());
    let m = merged.unwrap();
    assert_eq!(m.start, Position::new(3, 1));
    assert_eq!(m.end, Position::new(10, 1));
}

#[test]
fn span_merge_returns_none_for_non_adjacent_non_overlapping() {
    let span1 = Span::new(Position::new(3, 1), Position::new(5, 9));
    let span2 = Span::new(Position::new(5, 11), Position::new(10, 1));
    assert!(span1.merge(&span2).is_none());
}

#[test]
fn span_merge_is_symmetric_for_overlapping() {
    let span1 = Span::new(Position::new(3, 1), Position::new(7, 10));
    let span2 = Span::new(Position::new(5, 1), Position::new(10, 1));
    assert_eq!(span1.merge(&span2), span2.merge(&span1));
}

#[test]
fn span_is_adjacent_returns_true_for_adjacent_spans() {
    let span1 = Span::new(Position::new(3, 1), Position::new(5, 10));
    let span2 = Span::new(Position::new(5, 10), Position::new(10, 1));
    assert!(span1.is_adjacent(&span2));
    assert!(span2.is_adjacent(&span1));
}

#[test]
fn span_is_adjacent_returns_false_for_non_adjacent_spans() {
    let span1 = Span::new(Position::new(3, 1), Position::new(5, 9));
    let span2 = Span::new(Position::new(5, 11), Position::new(10, 1));
    assert!(!span1.is_adjacent(&span2));
}

#[test]
fn span_expand_to_include_extends_span() {
    let mut span1 = Span::new(Position::new(3, 1), Position::new(7, 10));
    let span2 = Span::new(Position::new(1, 1), Position::new(10, 5));
    span1.expand_to_include(&span2);
    assert_eq!(span1.start, Position::new(1, 1));
    assert_eq!(span1.end, Position::new(10, 5));
}

#[test]
fn span_expand_to_include_works_when_already_contained() {
    let mut span1 = Span::new(Position::new(1, 1), Position::new(10, 10));
    let span2 = Span::new(Position::new(3, 5), Position::new(7, 8));
    span1.expand_to_include(&span2);
    assert_eq!(span1.start, Position::new(1, 1));
    assert_eq!(span1.end, Position::new(10, 10));
}

// =============================================================================
// 7. Span display formatting (6 tests)
// =============================================================================

#[test]
fn span_display_formats_point_as_single_position() {
    let span = Span::point(5, 10);
    assert_eq!(format!("{span}"), "5:10");
}

#[test]
fn span_display_formats_single_line_span_correctly() {
    let span = Span::single_line(5, 1, 10);
    assert_eq!(format!("{span}"), "5:1-10");
}

#[test]
fn span_display_formats_multi_line_span_correctly() {
    let span = Span::new(Position::new(3, 5), Position::new(7, 10));
    assert_eq!(format!("{span}"), "3:5-7:10");
}

#[test]
fn span_display_formats_full_line_correctly() {
    let span = Span::full_line(5);
    // full_line is single-line, so format is "line:start_col-end_col"
    assert_eq!(format!("{span}"), "5:1-4294967295");
}

#[test]
fn span_display_formats_empty_span_correctly() {
    let span = Span::empty();
    assert_eq!(format!("{span}"), "1:1");
}

#[test]
fn span_display_formats_full_lines_correctly() {
    let span = Span::full_lines(3, 7);
    assert_eq!(format!("{span}"), "3:1-7:4294967295");
}

// =============================================================================
// 8. FileSpan methods (8 tests)
// =============================================================================

#[test]
fn file_span_new_creates_file_span_with_given_values() {
    let span = Span::single_line(5, 1, 10);
    let file_span = FileSpan::new("src/main.rs", span);
    assert_eq!(file_span.path, "src/main.rs");
    assert_eq!(file_span.span, span);
}

#[test]
fn file_span_new_accepts_string_ref() {
    let span = Span::single_line(5, 1, 10);
    let path = String::from("src/main.rs");
    let file_span = FileSpan::new(&path, span);
    assert_eq!(file_span.path, "src/main.rs");
}

#[test]
fn file_span_single_line_creates_correct_span() {
    let file_span = FileSpan::single_line("lib.rs", 10, 5, 20);
    assert_eq!(file_span.path, "lib.rs");
    assert_eq!(file_span.span.start.line, 10);
    assert_eq!(file_span.span.start.column, 5);
    assert_eq!(file_span.span.end.line, 10);
    assert_eq!(file_span.span.end.column, 20);
}

#[test]
fn file_span_full_line_creates_correct_span() {
    let file_span = FileSpan::full_line("test.rs", 15);
    assert_eq!(file_span.path, "test.rs");
    assert!(file_span.span.is_full_lines());
    assert_eq!(file_span.span.start.line, 15);
}

#[test]
fn file_span_point_creates_correct_span() {
    let file_span = FileSpan::point("config.toml", 42, 17);
    assert_eq!(file_span.path, "config.toml");
    assert!(file_span.span.is_point());
    assert_eq!(file_span.span.start.line, 42);
    assert_eq!(file_span.span.start.column, 17);
}

#[test]
fn file_span_path_returns_path_reference() {
    let file_span = FileSpan::full_line("src/lib.rs", 10);
    assert_eq!(file_span.path(), "src/lib.rs");
}

#[test]
fn file_span_span_returns_span_reference() {
    let span = Span::full_line(10);
    let file_span = FileSpan::new("src/lib.rs", span);
    assert_eq!(*file_span.span(), span);
}

#[test]
fn file_span_display_formats_correctly() {
    let span = Span::single_line(5, 1, 10);
    let file_span = FileSpan::new("src/main.rs", span);
    assert_eq!(format!("{file_span}"), "src/main.rs:5:1-10");
}

// =============================================================================
// Additional tests to reach 70+ total
// =============================================================================

#[test]
fn span_line_count_returns_1_for_single_line() {
    let span = Span::single_line(5, 1, 10);
    assert_eq!(span.line_count(), 1);
}

#[test]
fn span_line_count_returns_correct_count_for_multi_line() {
    let span = Span::new(Position::new(1, 1), Position::new(5, 10));
    assert_eq!(span.line_count(), 5);
}

#[test]
fn span_line_count_returns_1_for_point() {
    let span = Span::point(100, 50);
    assert_eq!(span.line_count(), 1);
}

#[test]
fn span_is_full_lines_returns_true_for_full_line() {
    let span = Span::full_line(5);
    assert!(span.is_full_lines());
}

#[test]
fn span_is_full_lines_returns_true_for_full_lines() {
    let span = Span::full_lines(3, 7);
    assert!(span.is_full_lines());
}

#[test]
fn span_is_full_lines_returns_false_for_partial_line() {
    let span = Span::single_line(5, 2, 10);
    assert!(!span.is_full_lines());
}

#[test]
fn span_is_full_lines_returns_false_when_end_column_not_max() {
    let span = Span::new(Position::start_of_line(5), Position::new(5, 100));
    assert!(!span.is_full_lines());
}

#[test]
fn span_accessors_return_correct_values() {
    let span = Span::new(Position::new(3, 5), Position::new(7, 10));
    assert_eq!(span.start_line(), 3);
    assert_eq!(span.end_line(), 7);
    assert_eq!(span.start_column(), 5);
    assert_eq!(span.end_column(), 10);
}

#[test]
fn span_to_line_range_returns_correct_tuple() {
    let span = Span::new(Position::new(3, 5), Position::new(7, 10));
    assert_eq!(span.to_line_range(), (3, 7));
}

#[test]
fn span_to_line_range_works_for_single_line() {
    let span = Span::single_line(5, 1, 10);
    assert_eq!(span.to_line_range(), (5, 5));
}

#[test]
fn position_ord_trait_works_correctly() {
    let mut positions = vec![
        Position::new(5, 10),
        Position::new(3, 20),
        Position::new(5, 5),
        Position::new(1, 1),
    ];
    positions.sort();
    assert_eq!(
        positions,
        vec![
            Position::new(1, 1),
            Position::new(3, 20),
            Position::new(5, 5),
            Position::new(5, 10),
        ]
    );
}

#[test]
fn span_hash_allows_use_in_hashset() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let span1 = Span::single_line(5, 1, 10);
    let span2 = Span::single_line(5, 1, 10);
    let span3 = Span::single_line(5, 1, 11);
    set.insert(span1);
    assert!(set.contains(&span2));
    assert!(!set.contains(&span3));
}

#[test]
fn position_hash_allows_use_in_hashset() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let pos1 = Position::new(5, 10);
    let pos2 = Position::new(5, 10);
    let pos3 = Position::new(5, 11);
    set.insert(pos1);
    assert!(set.contains(&pos2));
    assert!(!set.contains(&pos3));
}

#[test]
fn file_span_hash_allows_use_in_hashset() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let fs1 = FileSpan::single_line("test.rs", 5, 1, 10);
    let fs2 = FileSpan::single_line("test.rs", 5, 1, 10);
    let fs3 = FileSpan::single_line("other.rs", 5, 1, 10);
    set.insert(fs1);
    assert!(set.contains(&fs2));
    assert!(!set.contains(&fs3));
}

#[test]
fn position_clone_works_correctly() {
    let pos1 = Position::new(5, 10);
    #[allow(clippy::clone_on_copy)]
    let pos2 = pos1.clone();
    assert_eq!(pos1, pos2);
}

#[test]
fn span_clone_works_correctly() {
    let span1 = Span::single_line(5, 1, 10);
    #[allow(clippy::clone_on_copy)]
    let span2 = span1.clone();
    assert_eq!(span1, span2);
}

#[test]
fn file_span_clone_works_correctly() {
    let fs1 = FileSpan::single_line("test.rs", 5, 1, 10);
    let fs2 = fs1.clone();
    assert_eq!(fs1, fs2);
}

#[test]
fn span_new_unchecked_creates_span_without_validation() {
    let start = Position::new(1, 1);
    let end = Position::new(5, 10);
    let span = Span::new_unchecked(start, end);
    assert_eq!(span.start, start);
    assert_eq!(span.end, end);
}

#[test]
fn span_debug_format_includes_all_fields() {
    let span = Span::single_line(5, 1, 10);
    let debug_str = format!("{span:?}");
    assert!(debug_str.contains("start"));
    assert!(debug_str.contains("end"));
}

#[test]
fn position_debug_format_includes_all_fields() {
    let pos = Position::new(5, 10);
    let debug_str = format!("{pos:?}");
    assert!(debug_str.contains("line"));
    assert!(debug_str.contains("column"));
}

#[test]
fn file_span_debug_format_includes_all_fields() {
    let fs = FileSpan::single_line("test.rs", 5, 1, 10);
    let debug_str = format!("{fs:?}");
    assert!(debug_str.contains("path"));
    assert!(debug_str.contains("span"));
}

#[test]
fn span_equality_works_correctly() {
    let span1 = Span::single_line(5, 1, 10);
    let span2 = Span::single_line(5, 1, 10);
    let span3 = Span::single_line(5, 1, 11);
    assert_eq!(span1, span2);
    assert_ne!(span1, span3);
}

#[test]
fn position_equality_works_correctly() {
    let pos1 = Position::new(5, 10);
    let pos2 = Position::new(5, 10);
    let pos3 = Position::new(5, 11);
    assert_eq!(pos1, pos2);
    assert_ne!(pos1, pos3);
}

#[test]
fn file_span_equality_works_correctly() {
    let fs1 = FileSpan::single_line("test.rs", 5, 1, 10);
    let fs2 = FileSpan::single_line("test.rs", 5, 1, 10);
    let fs3 = FileSpan::single_line("other.rs", 5, 1, 10);
    let fs4 = FileSpan::single_line("test.rs", 5, 1, 11);
    assert_eq!(fs1, fs2);
    assert_ne!(fs1, fs3);
    assert_ne!(fs1, fs4);
}

#[test]
fn span_copy_trait_works() {
    let span1 = Span::single_line(5, 1, 10);
    let span2 = span1; // Copy
    assert_eq!(span1, span2);
}

#[test]
fn position_copy_trait_works() {
    let pos1 = Position::new(5, 10);
    let pos2 = pos1; // Copy
    assert_eq!(pos1, pos2);
}
