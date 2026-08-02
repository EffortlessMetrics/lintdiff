//! Tests for span extraction in the ingest engine.

use std::io::Cursor;

use lintdiff_engine::parse_cargo_messages;

/// Helper to create a compiler message with spans.
fn make_message_with_spans(message_fields: &str, spans_json: &str) -> String {
    format!(
        r#"{{"reason":"compiler-message","message":{{"spans":[{spans_json}],{message_fields}}}}}"#
    )
}

/// Helper to create a single span JSON.
fn make_span(fields: &str) -> String {
    format!(r#"{{{fields}}}"#)
}

// =============================================================================
// Basic Span Extraction
// =============================================================================

#[test]
fn extracts_single_span() {
    let span = make_span(
        r#""file_name":"src/lib.rs","line_start":10,"line_end":10,"column_start":1,"column_end":5,"is_primary":true"#,
    );
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans.len(), 1);

    let s = &diags[0].spans[0];
    assert_eq!(s.file.as_str(), "src/lib.rs");
    assert_eq!(s.line_start, 10);
    assert_eq!(s.line_end, 10);
    assert_eq!(s.col_start, Some(1));
    assert_eq!(s.col_end, Some(5));
    assert!(s.is_primary);
}

#[test]
fn extracts_span_with_minimal_fields() {
    let span = make_span(r#""file_name":"main.rs""#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans.len(), 1);

    let s = &diags[0].spans[0];
    assert_eq!(s.file.as_str(), "main.rs");
    assert_eq!(s.line_start, 1); // Default adjusted to 1 (from 0)
    assert_eq!(s.line_end, 1); // Defaults to line_start
    assert_eq!(s.col_start, None);
    assert_eq!(s.col_end, None);
    assert!(!s.is_primary); // Defaults to false
}

// =============================================================================
// Primary Spans
// =============================================================================

#[test]
fn identifies_primary_span() {
    let span = make_span(r#""file_name":"src/lib.rs","line_start":5,"is_primary":true"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert!(diags[0].spans[0].is_primary);
}

#[test]
fn identifies_non_primary_span() {
    let span = make_span(r#""file_name":"src/lib.rs","line_start":5,"is_primary":false"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert!(!diags[0].spans[0].is_primary);
}

#[test]
fn defaults_to_non_primary() {
    let span = make_span(r#""file_name":"src/lib.rs","line_start":5"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert!(!diags[0].spans[0].is_primary);
}

// =============================================================================
// Multiple Spans
// =============================================================================

#[test]
fn extracts_multiple_spans() {
    let span1 = make_span(r#""file_name":"a.rs","line_start":1,"is_primary":true"#);
    let span2 = make_span(r#""file_name":"b.rs","line_start":2,"is_primary":false"#);
    let span3 = make_span(r#""file_name":"c.rs","line_start":3,"is_primary":false"#);
    let spans = format!("{},{},{}", span1, span2, span3);

    let input = make_message_with_spans(r#""level":"error","message":"test""#, &spans);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();

    assert_eq!(diags[0].spans.len(), 3);
    assert_eq!(diags[0].spans[0].file.as_str(), "a.rs");
    assert!(diags[0].spans[0].is_primary);
    assert_eq!(diags[0].spans[1].file.as_str(), "b.rs");
    assert!(!diags[0].spans[1].is_primary);
    assert_eq!(diags[0].spans[2].file.as_str(), "c.rs");
    assert!(!diags[0].spans[2].is_primary);
}

#[test]
fn handles_multiple_primary_spans() {
    // Rustc typically only has one primary, but we should handle multiple
    let span1 = make_span(r#""file_name":"a.rs","line_start":1,"is_primary":true"#);
    let span2 = make_span(r#""file_name":"b.rs","line_start":2,"is_primary":true"#);
    let spans = format!("{},{}", span1, span2);

    let input = make_message_with_spans(r#""level":"error","message":"test""#, &spans);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();

    assert_eq!(diags[0].spans.len(), 2);
    assert!(diags[0].spans[0].is_primary);
    assert!(diags[0].spans[1].is_primary);
}

#[test]
fn handles_many_spans() {
    let spans: Vec<String> = (1..=50)
        .map(|i| make_span(&format!(r#""file_name":"file{}.rs","line_start":{}"#, i, i)))
        .collect();
    let spans_json = spans.join(",");

    let input = make_message_with_spans(r#""level":"error","message":"test""#, &spans_json);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();

    assert_eq!(diags[0].spans.len(), 50);
}

// =============================================================================
// Line Number Handling
// =============================================================================

#[test]
fn extracts_line_start_and_end() {
    let span = make_span(r#""file_name":"src/lib.rs","line_start":10,"line_end":15"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].line_start, 10);
    assert_eq!(diags[0].spans[0].line_end, 15);
}

#[test]
fn defaults_line_end_to_line_start() {
    let span = make_span(r#""file_name":"src/lib.rs","line_start":42"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].line_start, 42);
    assert_eq!(diags[0].spans[0].line_end, 42);
}

#[test]
fn handles_zero_line_start_as_one() {
    // rustc uses 1-based lines; 0 is invalid and adjusted to 1
    let span = make_span(r#""file_name":"src/lib.rs","line_start":0,"line_end":0"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].line_start, 1);
    assert_eq!(diags[0].spans[0].line_end, 1);
}

#[test]
fn ensures_line_end_at_least_line_start() {
    // If line_end < line_start, it should be adjusted
    let span = make_span(r#""file_name":"src/lib.rs","line_start":10,"line_end":5"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].line_start, 10);
    assert_eq!(diags[0].spans[0].line_end, 10); // Adjusted to line_start
}

#[test]
fn handles_large_line_numbers() {
    let span = make_span(r#""file_name":"src/lib.rs","line_start":1000000,"line_end":1000005"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].line_start, 1000000);
    assert_eq!(diags[0].spans[0].line_end, 1000005);
}

// =============================================================================
// Column Handling
// =============================================================================

#[test]
fn extracts_column_start_and_end() {
    let span =
        make_span(r#""file_name":"src/lib.rs","line_start":1,"column_start":5,"column_end":10"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].col_start, Some(5));
    assert_eq!(diags[0].spans[0].col_end, Some(10));
}

#[test]
fn handles_missing_columns() {
    let span = make_span(r#""file_name":"src/lib.rs","line_start":1"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].col_start, None);
    assert_eq!(diags[0].spans[0].col_end, None);
}

#[test]
fn handles_only_column_start() {
    let span = make_span(r#""file_name":"src/lib.rs","line_start":1,"column_start":5"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].col_start, Some(5));
    assert_eq!(diags[0].spans[0].col_end, None);
}

#[test]
fn handles_zero_column_values() {
    // Column 0 becomes Some(0) - the parser doesn't adjust columns like it does lines
    let span =
        make_span(r#""file_name":"src/lib.rs","line_start":1,"column_start":0,"column_end":0"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].col_start, Some(0));
    assert_eq!(diags[0].spans[0].col_end, Some(0));
}

// =============================================================================
// File Path Handling
// =============================================================================

#[test]
fn extracts_simple_file_path() {
    let span = make_span(r#""file_name":"main.rs","line_start":1"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].file.as_str(), "main.rs");
}

#[test]
fn extracts_path_with_directory() {
    let span = make_span(r#""file_name":"src/lib.rs","line_start":1"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].file.as_str(), "src/lib.rs");
}

#[test]
fn extracts_nested_path() {
    let span = make_span(r#""file_name":"src/deeply/nested/module.rs","line_start":1"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(
        diags[0].spans[0].file.as_str(),
        "src/deeply/nested/module.rs"
    );
}

#[test]
fn handles_absolute_path() {
    let span = make_span(r#""file_name":"/home/user/project/src/main.rs","line_start":1"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(
        diags[0].spans[0].file.as_str(),
        "/home/user/project/src/main.rs"
    );
}

#[test]
fn handles_windows_style_path() {
    // Note: NormPath normalizes backslashes to forward slashes
    let span = make_span(r#""file_name":"C:\\Users\\project\\src\\main.rs","line_start":1"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    // NormPath normalizes to forward slashes
    assert_eq!(
        diags[0].spans[0].file.as_str(),
        "C:/Users/project/src/main.rs"
    );
}

#[test]
fn handles_missing_file_name() {
    let span = make_span(r#""line_start":1"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].file.as_str(), "");
}

#[test]
fn handles_unicode_in_file_path() {
    let span = make_span(r#""file_name":"src/日本語/モジュール.rs","line_start":1"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].file.as_str(), "src/日本語/モジュール.rs");
}

// =============================================================================
// Expansion Info (Macro Expansion)
// =============================================================================

#[test]
fn handles_span_with_expansion() {
    // Spans can have expansion info from macros - we just need to parse without error
    // Note: JSON must be single-line for line-delimited parsing
    let span = r#"{"file_name":"src/lib.rs","line_start":5,"line_end":5,"column_start":1,"column_end":10,"is_primary":true,"expansion":{"span":{"file_name":"<macro>","line_start":1,"line_end":1,"column_start":1,"column_end":1,"is_primary":false,"expansion":null},"macro_decl_name":"println!"}}"#;

    let input = format!(
        r#"{{"reason":"compiler-message","message":{{"level":"error","message":"test","spans":[{}]}}}}"#,
        span
    );

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans.len(), 1);
    assert_eq!(diags[0].spans[0].file.as_str(), "src/lib.rs");
}

#[test]
fn handles_nested_expansions() {
    // Deeply nested macro expansions
    let span = r#"{"file_name":"src/lib.rs","line_start":5,"is_primary":true,"expansion":{"span":{"file_name":"<macro>","line_start":1,"is_primary":false,"expansion":{"span":{"file_name":"<macro>","line_start":1,"is_primary":false,"expansion":null}}}}}"#;

    let input = format!(
        r#"{{"reason":"compiler-message","message":{{"level":"error","message":"test","spans":[{}]}}}}"#,
        span
    );

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans.len(), 1);
}

// =============================================================================
// Other Span Fields (Ignored but should parse)
// =============================================================================

#[test]
fn handles_span_with_text_array() {
    let span = r#"{"file_name":"src/lib.rs","line_start":5,"is_primary":true,"text":[{"text":"    let x = 5;","highlight_start":5,"highlight_end":6}]}"#;

    let input = format!(
        r#"{{"reason":"compiler-message","message":{{"level":"error","message":"test","spans":[{}]}}}}"#,
        span
    );

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans.len(), 1);
}

#[test]
fn handles_span_with_label() {
    let span = r#"{"file_name":"src/lib.rs","line_start":5,"is_primary":true,"label":"expected i32, found String"}"#;

    let input = format!(
        r#"{{"reason":"compiler-message","message":{{"level":"error","message":"test","spans":[{}]}}}}"#,
        span
    );

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans.len(), 1);
}

#[test]
fn handles_span_with_suggested_replacement() {
    let span = r#"{"file_name":"src/lib.rs","line_start":5,"is_primary":true,"suggested_replacement":"foo","suggestion_applicability":"MaybeIncorrect"}"#;

    let input = format!(
        r#"{{"reason":"compiler-message","message":{{"level":"error","message":"test","spans":[{}]}}}}"#,
        span
    );

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans.len(), 1);
}

#[test]
fn handles_span_with_byte_positions() {
    let span = r#"{"file_name":"src/lib.rs","line_start":5,"is_primary":true,"byte_start":100,"byte_end":105}"#;

    let input = format!(
        r#"{{"reason":"compiler-message","message":{{"level":"error","message":"test","spans":[{}]}}}}"#,
        span
    );

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans.len(), 1);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn handles_span_with_null_expansion() {
    let span =
        make_span(r#""file_name":"src/lib.rs","line_start":5,"is_primary":true,"expansion":null"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans.len(), 1);
}

#[test]
fn handles_span_in_generated_file() {
    let span = make_span(r#""file_name":"<generated>","line_start":1,"is_primary":true"#);
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans[0].file.as_str(), "<generated>");
}

#[test]
fn handles_span_in_cargo_registry() {
    let span = make_span(
        r#""file_name":"/home/user/.cargo/registry/src/index.crates.io-6f17d22bba15001f/serde-1.0.188/src/lib.rs","line_start":100,"is_primary":false"#,
    );
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert!(diags[0].spans[0].file.as_str().contains(".cargo"));
}

#[test]
fn handles_multiline_span() {
    let span = make_span(
        r#""file_name":"src/lib.rs","line_start":10,"line_end":20,"column_start":1,"column_end":5"#,
    );
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    let s = &diags[0].spans[0];
    assert_eq!(s.line_start, 10);
    assert_eq!(s.line_end, 20);
}

#[test]
fn handles_single_character_span() {
    let span = make_span(
        r#""file_name":"src/lib.rs","line_start":10,"line_end":10,"column_start":5,"column_end":6"#,
    );
    let input = make_message_with_spans(r#""level":"error","message":"test""#, &span);

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    let s = &diags[0].spans[0];
    assert_eq!(s.line_start, s.line_end);
    assert_eq!(s.col_start, Some(5));
    assert_eq!(s.col_end, Some(6));
}
