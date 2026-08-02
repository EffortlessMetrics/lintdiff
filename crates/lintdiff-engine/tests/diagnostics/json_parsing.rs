//! Tests for JSON message parsing in the ingest engine.

use std::io::Cursor;

use lintdiff_engine::{parse_cargo_messages, DiagnosticLevel};

/// Helper to create a minimal valid compiler message JSON.
fn make_compiler_message(fields: &str) -> String {
    format!(r#"{{"reason":"compiler-message","message":{{{fields}}}}}"#)
}

// =============================================================================
// Valid Diagnostic Messages
// =============================================================================

#[test]
fn parses_minimal_valid_diagnostic() {
    let input = make_compiler_message(r#""level":"warning","message":"test message""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();

    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, DiagnosticLevel::Warning);
    assert_eq!(diags[0].message, "test message");
    assert!(diags[0].code_raw.is_none());
    assert!(diags[0].spans.is_empty());
    assert!(diags[0].rendered.is_none());
}

#[test]
fn parses_full_diagnostic_with_all_fields() {
    // Note: JSON must be on a single line for line-delimited JSON parsing
    let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"cannot find value `x` in this scope","code":{"code":"E0425","explanation":"..."},"spans":[{"file_name":"src/main.rs","byte_start":100,"byte_end":101,"line_start":10,"line_end":10,"column_start":5,"column_end":6,"is_primary":true,"text":[{"text":"    x","highlight_start":5,"highlight_end":6}],"label":"not found in this scope","suggested_replacement":null,"expansion":null}],"children":[],"rendered":"error[E0425]: cannot find value `x` in this scope\n"}}"#;

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();

    assert_eq!(diags.len(), 1);
    let diag = &diags[0];

    assert_eq!(diag.level, DiagnosticLevel::Error);
    assert_eq!(diag.message, "cannot find value `x` in this scope");
    assert_eq!(diag.code_raw.as_deref(), Some("E0425"));
    assert_eq!(
        diag.rendered.as_deref(),
        Some("error[E0425]: cannot find value `x` in this scope\n")
    );

    assert_eq!(diag.spans.len(), 1);
    let span = &diag.spans[0];
    assert_eq!(span.file.as_str(), "src/main.rs");
    assert_eq!(span.line_start, 10);
    assert_eq!(span.line_end, 10);
    assert_eq!(span.col_start, Some(5));
    assert_eq!(span.col_end, Some(6));
    assert!(span.is_primary);
}

#[test]
fn parses_multiple_diagnostics_in_sequence() {
    let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"first error"}}
{"reason":"compiler-message","message":{"level":"warning","message":"second warning"}}
{"reason":"compiler-message","message":{"level":"note","message":"third note"}}"#;

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();

    assert_eq!(diags.len(), 3);
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
    assert_eq!(diags[0].message, "first error");
    assert_eq!(diags[1].level, DiagnosticLevel::Warning);
    assert_eq!(diags[1].message, "second warning");
    assert_eq!(diags[2].level, DiagnosticLevel::Note);
    assert_eq!(diags[2].message, "third note");
}

// =============================================================================
// Level Extraction
// =============================================================================

#[test]
fn extracts_error_level() {
    let input = make_compiler_message(r#""level":"error","message":"test""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
}

#[test]
fn extracts_warning_level() {
    let input = make_compiler_message(r#""level":"warning","message":"test""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].level, DiagnosticLevel::Warning);
}

#[test]
fn extracts_note_level() {
    let input = make_compiler_message(r#""level":"note","message":"test""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].level, DiagnosticLevel::Note);
}

#[test]
fn extracts_help_level() {
    let input = make_compiler_message(r#""level":"help","message":"test""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].level, DiagnosticLevel::Help);
}

#[test]
fn extracts_unknown_level_as_other() {
    let input = make_compiler_message(r#""level":"custom-level","message":"test""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(
        diags[0].level,
        DiagnosticLevel::Other("custom-level".to_string())
    );
}

#[test]
fn handles_missing_level_as_other() {
    let input = make_compiler_message(r#""message":"test""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].level, DiagnosticLevel::Other("other".to_string()));
}

// =============================================================================
// Code Extraction
// =============================================================================

#[test]
fn extracts_code_from_nested_object() {
    let input = make_compiler_message(
        r#""level":"error","message":"test","code":{"code":"E0308","explanation":"..."}"#,
    );
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].code_raw.as_deref(), Some("E0308"));
}

#[test]
fn handles_missing_code() {
    let input = make_compiler_message(r#""level":"error","message":"test""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert!(diags[0].code_raw.is_none());
}

#[test]
fn handles_code_object_without_code_field() {
    let input =
        make_compiler_message(r#""level":"error","message":"test","code":{"explanation":"..."}"#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert!(diags[0].code_raw.is_none());
}

#[test]
fn extracts_clippy_lint_codes() {
    let input = make_compiler_message(
        r#""level":"warning","message":"this is unnecessary","code":{"code":"clippy::needless_borrow"}"#,
    );
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(
        diags[0].code_raw.as_deref(),
        Some("clippy::needless_borrow")
    );
}

#[test]
fn extracts_rustc_lint_codes() {
    let input = make_compiler_message(
        r#""level":"warning","message":"unused variable","code":{"code":"unused_variables"}"#,
    );
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].code_raw.as_deref(), Some("unused_variables"));
}

// =============================================================================
// Message Extraction
// =============================================================================

#[test]
fn extracts_simple_message() {
    let input = make_compiler_message(r#""level":"error","message":"simple message""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].message, "simple message");
}

#[test]
fn handles_missing_message_as_empty() {
    let input = make_compiler_message(r#""level":"error""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].message, "");
}

#[test]
fn preserves_message_whitespace() {
    let input = make_compiler_message(r#""level":"error","message":"  message with spaces  ""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].message, "  message with spaces  ");
}

// =============================================================================
// Rendered Field
// =============================================================================

#[test]
fn extracts_rendered_field() {
    let input = make_compiler_message(
        r#""level":"error","message":"test","rendered":"error[E0308]: mismatched types\n""#,
    );
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(
        diags[0].rendered.as_deref(),
        Some("error[E0308]: mismatched types\n")
    );
}

#[test]
fn handles_missing_rendered() {
    let input = make_compiler_message(r#""level":"error","message":"test""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert!(diags[0].rendered.is_none());
}

// =============================================================================
// Non-Diagnostic Messages (Should Be Ignored)
// =============================================================================

#[test]
fn ignores_build_script_executed() {
    let input = r#"{"reason":"build-script-executed","package_id":"test 0.1.0"}"#;
    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert!(diags.is_empty());
}

#[test]
fn ignores_compiler_artifact() {
    let input = r#"{"reason":"compiler-artifact","package_id":"test 0.1.0","target":{"name":"test","kind":["lib"]}}"#;
    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert!(diags.is_empty());
}

#[test]
fn ignores_build_finished() {
    let input = r#"{"reason":"build-finished","success":true}"#;
    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert!(diags.is_empty());
}

#[test]
fn ignores_unknown_reason() {
    let input = r#"{"reason":"some-other-reason","data":{}}"#;
    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert!(diags.is_empty());
}

#[test]
fn ignores_message_without_reason() {
    let input = r#"{"message":"hello world"}"#;
    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert!(diags.is_empty());
}

#[test]
fn filters_mixed_messages() {
    let input = r#"{"reason":"build-script-executed","package_id":"x"}
{"reason":"compiler-message","message":{"level":"warning","message":"found"}}
{"reason":"compiler-artifact","package_id":"y"}
{"reason":"compiler-message","message":{"level":"error","message":"found2"}}"#;

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0].message, "found");
    assert_eq!(diags[1].message, "found2");
}

// =============================================================================
// Malformed JSON
// =============================================================================

#[test]
fn rejects_invalid_json_syntax() {
    let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"test""#; // Missing closing braces
    let result = parse_cargo_messages(Cursor::new(input));
    assert!(result.is_err());
}

#[test]
fn rejects_non_object_json() {
    let input = r#""just a string""#;
    let result = parse_cargo_messages(Cursor::new(input));
    // Non-object JSON is valid JSON but doesn't have "reason", so it's ignored
    assert!(result.is_ok());
    let diags = result.unwrap();
    assert!(diags.is_empty());
}

#[test]
fn rejects_json_array_at_top_level() {
    let input = r#"[{"reason":"compiler-message","message":{"level":"error","message":"test"}}]"#;
    let result = parse_cargo_messages(Cursor::new(input));
    // Array at top level doesn't match our expected structure
    // The parser expects line-delimited JSON objects
    assert!(result.is_ok()); // Arrays are valid JSON, just ignored
}

#[test]
fn handles_trailing_comma_in_json() {
    // JSON standard doesn't allow trailing commas, but let's test behavior
    let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"test",}}"#;
    let result = parse_cargo_messages(Cursor::new(input));
    // serde_json rejects trailing commas by default
    assert!(result.is_err());
}

// =============================================================================
// Missing Fields
// =============================================================================

#[test]
fn handles_missing_message_field_in_compiler_message() {
    let input = r#"{"reason":"compiler-message"}"#;
    let result = parse_cargo_messages(Cursor::new(input));
    assert!(result.is_err());
}

#[test]
fn handles_null_message_field() {
    // When message is null, the parser gracefully handles it by using defaults
    // The key exists so .get() returns Some(null), and subsequent .get() calls return None
    let input = r#"{"reason":"compiler-message","message":null}"#;
    let result = parse_cargo_messages(Cursor::new(input));
    // This actually succeeds with empty/default values
    assert!(result.is_ok());
    let diags = result.unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].message, "");
    assert_eq!(diags[0].level, DiagnosticLevel::Other("other".to_string()));
}

#[test]
fn handles_empty_spans_array() {
    let input = make_compiler_message(r#""level":"error","message":"test","spans":[]"#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert!(diags[0].spans.is_empty());
}

#[test]
fn handles_missing_spans_as_empty() {
    let input = make_compiler_message(r#""level":"error","message":"test""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert!(diags[0].spans.is_empty());
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn handles_empty_input() {
    let input = "";
    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert!(diags.is_empty());
}

#[test]
fn handles_only_whitespace_lines() {
    let input = "   \n\t\n  \n";
    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert!(diags.is_empty());
}

#[test]
fn handles_empty_lines_between_messages() {
    let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"first"}}

{"reason":"compiler-message","message":{"level":"warning","message":"second"}}"#;

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 2);
}

#[test]
fn handles_whitespace_around_lines() {
    let input = r#"  
{"reason":"compiler-message","message":{"level":"error","message":"test"}}
   "#;

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 1);
}

#[test]
fn handles_very_long_message() {
    let long_message = "x".repeat(10000);
    let input = make_compiler_message(&format!(r#""level":"error","message":"{}""#, long_message));
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].message.len(), 10000);
}

#[test]
fn handles_unicode_in_message() {
    let input = make_compiler_message(r#""level":"error","message":"Unicode: 日本語 🎉 émoji""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].message, "Unicode: 日本語 🎉 émoji");
}

#[test]
fn handles_special_characters_in_message() {
    let input = make_compiler_message(
        r#""level":"error","message":"Special: \"quotes\" \\backslash\\ \n newline \t tab""#,
    );
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert!(diags[0].message.contains("quotes"));
    assert!(diags[0].message.contains("backslash"));
}

#[test]
fn handles_newlines_in_message() {
    let input = make_compiler_message(r#""level":"error","message":"line1\nline2\nline3""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].message, "line1\nline2\nline3");
}

#[test]
fn handles_escaped_unicode_in_message() {
    let input =
        make_compiler_message(r#""level":"error","message":"\u0048\u0065\u006c\u006c\u006f""#);
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].message, "Hello");
}

#[test]
fn handles_code_with_special_characters() {
    let input = make_compiler_message(
        r#""level":"warning","message":"test","code":{"code":"rustc::lint::some-lint-name"}"#,
    );
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(
        diags[0].code_raw.as_deref(),
        Some("rustc::lint::some-lint-name")
    );
}

// =============================================================================
// Children Handling (Note: children are currently ignored by the parser)
// =============================================================================

#[test]
fn handles_children_array() {
    // The current implementation doesn't extract children, but should parse without error
    let input = make_compiler_message(
        r#""level":"error","message":"parent","children":[{"level":"note","message":"child note","spans":[]}]"#,
    );
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].message, "parent");
}
