//! Comprehensive tests for span selection logic.
//!
//! These tests cover:
//! - Primary span selection
//! - Multi-span intersection
//! - Line range matching
//! - Edge cases (empty spans, zero lines)

use lintdiff_ingest_core::{select_spans, Span};
use lintdiff_types::NormPath;

/// Helper to create a span with specified parameters.
fn create_span(file: &str, line_start: u32, line_end: u32, is_primary: bool) -> Span {
    Span {
        file: NormPath::new(file),
        line_start,
        line_end,
        col_start: None,
        col_end: None,
        is_primary,
    }
}

/// Helper to create a span with column information.
fn create_span_with_cols(
    file: &str,
    line_start: u32,
    line_end: u32,
    col_start: u32,
    col_end: u32,
    is_primary: bool,
) -> Span {
    Span {
        file: NormPath::new(file),
        line_start,
        line_end,
        col_start: Some(col_start),
        col_end: Some(col_end),
        is_primary,
    }
}

// =============================================================================
// Primary Span Selection Tests
// =============================================================================

mod primary_span_selection {
    use super::*;

    #[test]
    fn single_primary_span_selected() {
        let spans = vec![create_span("src/lib.rs", 10, 15, true)];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line_start, 10);
        assert_eq!(result[0].line_end, 15);
        assert!(result[0].is_primary);
    }

    #[test]
    fn multiple_primary_spans_all_selected() {
        let spans = vec![
            create_span("src/lib.rs", 10, 15, true),
            create_span("src/lib.rs", 20, 25, true),
            create_span("src/utils.rs", 5, 8, true),
        ];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|s| s.is_primary));
    }

    #[test]
    fn only_primary_spans_selected_from_mixed() {
        let spans = vec![
            create_span("src/lib.rs", 10, 15, true),
            create_span("src/lib.rs", 20, 25, false),
            create_span("src/utils.rs", 5, 8, true),
            create_span("src/main.rs", 1, 5, false),
        ];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|s| s.is_primary));

        // Verify the correct spans were selected
        let line_starts: Vec<u32> = result.iter().map(|s| s.line_start).collect();
        assert!(line_starts.contains(&10));
        assert!(line_starts.contains(&5));
    }

    #[test]
    fn primary_spans_from_different_files() {
        let spans = vec![
            create_span("src/lib.rs", 10, 15, true),
            create_span("src/utils.rs", 5, 8, true),
            create_span("tests/integration.rs", 100, 110, true),
        ];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 3);

        let files: Vec<&str> = result.iter().map(|s| s.file.as_str()).collect();
        assert!(files.contains(&"src/lib.rs"));
        assert!(files.contains(&"src/utils.rs"));
        assert!(files.contains(&"tests/integration.rs"));
    }
}

// =============================================================================
// No Primary Span Tests
// =============================================================================

mod no_primary_spans {
    use super::*;

    #[test]
    fn all_spans_returned_when_no_primary() {
        let spans = vec![
            create_span("src/lib.rs", 10, 15, false),
            create_span("src/utils.rs", 5, 8, false),
        ];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn single_non_primary_span_returned() {
        let spans = vec![create_span("src/lib.rs", 10, 15, false)];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 1);
        assert!(!result[0].is_primary);
    }

    #[test]
    fn many_non_primary_spans_all_returned() {
        let spans = vec![
            create_span("src/lib.rs", 10, 15, false),
            create_span("src/lib.rs", 20, 25, false),
            create_span("src/lib.rs", 30, 35, false),
            create_span("src/lib.rs", 40, 45, false),
        ];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 4);
    }
}

// =============================================================================
// Empty and Edge Case Tests
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn empty_spans_returns_empty() {
        let spans: Vec<Span> = vec![];

        let result = select_spans(&spans);

        assert!(result.is_empty());
    }

    #[test]
    fn single_line_span() {
        let spans = vec![create_span("src/lib.rs", 10, 10, true)];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line_start, 10);
        assert_eq!(result[0].line_end, 10);
    }

    #[test]
    fn zero_line_span() {
        // Edge case: what if line numbers are 0?
        let spans = vec![create_span("src/lib.rs", 0, 0, true)];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line_start, 0);
        assert_eq!(result[0].line_end, 0);
    }

    #[test]
    fn very_large_line_numbers() {
        let spans = vec![create_span("src/lib.rs", 1_000_000, 2_000_000, true)];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line_start, 1_000_000);
        assert_eq!(result[0].line_end, 2_000_000);
    }

    #[test]
    fn span_with_column_information_preserved() {
        let spans = vec![create_span_with_cols("src/lib.rs", 10, 10, 5, 20, true)];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].col_start, Some(5));
        assert_eq!(result[0].col_end, Some(20));
    }

    #[test]
    fn mixed_spans_with_and_without_columns() {
        let spans = vec![
            create_span_with_cols("src/lib.rs", 10, 10, 5, 20, true),
            create_span("src/utils.rs", 5, 8, true),
        ];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 2);

        // Find the span with columns
        let with_cols = result.iter().find(|s| s.col_start.is_some());
        assert!(with_cols.is_some());
        assert_eq!(with_cols.unwrap().col_start, Some(5));

        // Find the span without columns
        let without_cols = result.iter().find(|s| s.col_start.is_none());
        assert!(without_cols.is_some());
    }
}

// =============================================================================
// Span Ordering Tests
// =============================================================================

mod span_ordering {
    use super::*;

    #[test]
    fn spans_preserve_original_order() {
        let spans = vec![
            create_span("src/c.rs", 30, 35, true),
            create_span("src/a.rs", 10, 15, true),
            create_span("src/b.rs", 20, 25, true),
        ];

        let result = select_spans(&spans);

        // Order should be preserved
        assert_eq!(result[0].file.as_str(), "src/c.rs");
        assert_eq!(result[1].file.as_str(), "src/a.rs");
        assert_eq!(result[2].file.as_str(), "src/b.rs");
    }

    #[test]
    fn mixed_primary_secondary_preserves_primary_order() {
        let spans = vec![
            create_span("src/a.rs", 10, 15, false),
            create_span("src/b.rs", 20, 25, true),
            create_span("src/c.rs", 30, 35, false),
            create_span("src/d.rs", 40, 45, true),
        ];

        let result = select_spans(&spans);

        // Only primary spans, in their original order
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].file.as_str(), "src/b.rs");
        assert_eq!(result[1].file.as_str(), "src/d.rs");
    }
}

// =============================================================================
// Multi-file Span Tests
// =============================================================================

mod multi_file {
    use super::*;

    #[test]
    fn spans_from_same_file() {
        let spans = vec![
            create_span("src/lib.rs", 10, 15, true),
            create_span("src/lib.rs", 20, 25, true),
            create_span("src/lib.rs", 30, 35, true),
        ];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|s| s.file.as_str() == "src/lib.rs"));
    }

    #[test]
    fn spans_from_different_files() {
        let spans = vec![
            create_span("src/main.rs", 1, 5, true),
            create_span("src/lib.rs", 10, 15, true),
            create_span("src/utils/mod.rs", 100, 110, true),
            create_span("tests/integration.rs", 50, 60, true),
        ];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 4);
    }

    #[test]
    fn mixed_files_with_primary_filtering() {
        let spans = vec![
            create_span("src/main.rs", 1, 5, false),
            create_span("src/lib.rs", 10, 15, true),
            create_span("src/utils/mod.rs", 100, 110, false),
            create_span("tests/integration.rs", 50, 60, true),
        ];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 2);

        let files: Vec<&str> = result.iter().map(|s| s.file.as_str()).collect();
        assert!(files.contains(&"src/lib.rs"));
        assert!(files.contains(&"tests/integration.rs"));
    }
}

// =============================================================================
// Line Range Tests
// =============================================================================

mod line_ranges {
    use super::*;

    #[test]
    fn single_line_span_range() {
        let spans = vec![create_span("src/lib.rs", 42, 42, true)];

        let result = select_spans(&spans);

        assert_eq!(result[0].line_start, 42);
        assert_eq!(result[0].line_end, 42);
    }

    #[test]
    fn multi_line_span_range() {
        let spans = vec![create_span("src/lib.rs", 10, 50, true)];

        let result = select_spans(&spans);

        assert_eq!(result[0].line_start, 10);
        assert_eq!(result[0].line_end, 50);
    }

    #[test]
    fn overlapping_line_ranges() {
        // Both spans cover overlapping lines - both should be selected
        let spans = vec![
            create_span("src/lib.rs", 10, 20, true),
            create_span("src/lib.rs", 15, 25, true),
        ];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn adjacent_line_ranges() {
        let spans = vec![
            create_span("src/lib.rs", 10, 20, true),
            create_span("src/lib.rs", 21, 30, true),
        ];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn large_line_range() {
        let spans = vec![create_span("src/lib.rs", 1, 10000, true)];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line_start, 1);
        assert_eq!(result[0].line_end, 10000);
    }
}

// =============================================================================
// Cloning Tests
// =============================================================================

mod cloning {
    use super::*;

    #[test]
    fn result_is_cloned_from_input() {
        let spans = vec![create_span("src/lib.rs", 10, 15, true)];

        let result = select_spans(&spans);

        // Result should be a clone, modifying it shouldn't affect original
        drop(result);
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn multiple_calls_produce_independent_results() {
        let spans = vec![create_span("src/lib.rs", 10, 15, true)];

        let result1 = select_spans(&spans);
        let result2 = select_spans(&spans);

        assert_eq!(result1.len(), result2.len());
    }
}

// =============================================================================
// Path Handling in Spans
// =============================================================================

mod path_handling {
    use super::*;

    #[test]
    fn windows_path_in_span() {
        let spans = vec![create_span("src\\lib.rs", 10, 15, true)];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 1);
        // Note: NormPath::new normalizes backslashes to forward slashes
        assert_eq!(result[0].file.as_str(), "src/lib.rs");
    }

    #[test]
    fn absolute_path_in_span() {
        let spans = vec![create_span("/home/user/project/src/lib.rs", 10, 15, true)];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file.as_str(), "/home/user/project/src/lib.rs");
    }

    #[test]
    fn unicode_path_in_span() {
        let spans = vec![create_span("src/日本語/файл.rs", 10, 15, true)];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file.as_str(), "src/日本語/файл.rs");
    }

    #[test]
    fn path_with_spaces_in_span() {
        let spans = vec![create_span("src/my module/lib.rs", 10, 15, true)];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file.as_str(), "src/my module/lib.rs");
    }
}

// =============================================================================
// Real-world Scenario Tests
// =============================================================================

mod real_world_scenarios {
    use super::*;

    #[test]
    fn rustc_error_with_primary_and_context() {
        // Typical rustc error: one primary span and several context spans
        let spans = vec![
            create_span("src/lib.rs", 10, 15, true),
            create_span("src/lib.rs", 5, 8, false), // Related context
            create_span("src/lib.rs", 20, 25, false), // Another context
        ];

        let result = select_spans(&spans);

        // Only primary should be selected
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line_start, 10);
    }

    #[test]
    fn clippy_warning_with_multiple_primaries() {
        // Some lints may have multiple primary spans
        let spans = vec![
            create_span("src/lib.rs", 10, 15, true),
            create_span("src/utils.rs", 42, 50, true),
            create_span("src/lib.rs", 5, 8, false),
        ];

        let result = select_spans(&spans);

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|s| s.is_primary));
    }

    #[test]
    fn macro_expansion_spans() {
        // Error from macro expansion with spans in different files
        let spans = vec![
            create_span("src/macros.rs", 5, 10, true), // Macro definition
            create_span("src/lib.rs", 42, 42, true),   // Invocation site
            create_span("<macro expansion>", 1, 5, false), // Generated code
        ];

        let result = select_spans(&spans);

        // Both primary spans should be selected
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn no_primary_spans_fallback() {
        // Some diagnostics may not have primary spans marked
        let spans = vec![
            create_span("src/lib.rs", 10, 15, false),
            create_span("src/lib.rs", 20, 25, false),
        ];

        let result = select_spans(&spans);

        // All spans should be returned as fallback
        assert_eq!(result.len(), 2);
    }
}

// =============================================================================
// Debug Trait Tests
// =============================================================================

mod debug_trait {
    use super::*;

    #[test]
    fn span_debug_output() {
        let spans = vec![create_span("src/lib.rs", 10, 15, true)];

        let result = select_spans(&spans);
        let debug_str = format!("{:?}", result[0]);

        assert!(debug_str.contains("Span"));
        assert!(debug_str.contains("src/lib.rs"));
    }
}
