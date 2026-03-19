//! Tests for stream handling in lintdiff-diagnostics.

use std::io::{BufReader, Cursor, Read};

use lintdiff_diagnostics::{parse_cargo_messages, DiagnosticLevel, DiagnosticsParseError};

// =============================================================================
// Basic Stream Handling
// =============================================================================

#[test]
fn reads_from_cursor() {
    let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"test"}}"#;
    let cursor = Cursor::new(input);

    let diags = parse_cargo_messages(cursor).unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].message, "test");
}

#[test]
fn reads_from_buf_reader() {
    let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"test"}}"#;
    let reader = BufReader::new(Cursor::new(input));

    let diags = parse_cargo_messages(reader).unwrap();
    assert_eq!(diags.len(), 1);
}

#[test]
fn reads_from_string_reader() {
    let input =
        r#"{"reason":"compiler-message","message":{"level":"error","message":"test"}}"#.to_string();
    let cursor = Cursor::new(input);

    let diags = parse_cargo_messages(cursor).unwrap();
    assert_eq!(diags.len(), 1);
}

#[test]
fn reads_from_bytes_reader() {
    let input = br#"{"reason":"compiler-message","message":{"level":"error","message":"test"}}"#;
    let cursor = Cursor::new(&input[..]);

    let diags = parse_cargo_messages(cursor).unwrap();
    assert_eq!(diags.len(), 1);
}

// =============================================================================
// Line Handling
// =============================================================================

#[test]
fn handles_unix_line_endings() {
    let input = "{\"reason\":\"compiler-message\",\"message\":{\"level\":\"error\",\"message\":\"first\"}}\n{\"reason\":\"compiler-message\",\"message\":{\"level\":\"warning\",\"message\":\"second\"}}\n";

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0].message, "first");
    assert_eq!(diags[1].message, "second");
}

#[test]
fn handles_windows_line_endings() {
    let input = "{\"reason\":\"compiler-message\",\"message\":{\"level\":\"error\",\"message\":\"first\"}}\r\n{\"reason\":\"compiler-message\",\"message\":{\"level\":\"warning\",\"message\":\"second\"}}\r\n";

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 2);
}

#[test]
fn handles_mixed_line_endings() {
    let input = "{\"reason\":\"compiler-message\",\"message\":{\"level\":\"error\",\"message\":\"first\"}}\n{\"reason\":\"compiler-message\",\"message\":{\"level\":\"warning\",\"message\":\"second\"}}\r\n{\"reason\":\"compiler-message\",\"message\":{\"level\":\"note\",\"message\":\"third\"}}\n";

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 3);
}

#[test]
fn handles_empty_lines() {
    let input = "\n{\"reason\":\"compiler-message\",\"message\":{\"level\":\"error\",\"message\":\"test\"}}\n\n\n";

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 1);
}

#[test]
fn handles_whitespace_only_lines() {
    let input = "   \t  \n{\"reason\":\"compiler-message\",\"message\":{\"level\":\"error\",\"message\":\"test\"}}\n   \n";

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 1);
}

#[test]
fn handles_no_trailing_newline() {
    let input =
        "{\"reason\":\"compiler-message\",\"message\":{\"level\":\"error\",\"message\":\"test\"}}";

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 1);
}

#[test]
fn handles_trailing_newline_only() {
    let input = "{\"reason\":\"compiler-message\",\"message\":{\"level\":\"error\",\"message\":\"test\"}}\n";

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 1);
}

// =============================================================================
// Error Reporting
// =============================================================================

#[test]
fn reports_line_number_on_invalid_json() {
    // First line is valid JSON but not a compiler-message (so ignored)
    // Second line is invalid JSON
    let input = "{\"reason\":\"build-script-executed\"}\n{\"bad json";

    let result = parse_cargo_messages(Cursor::new(input));
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        DiagnosticsParseError::InvalidJson { line, .. } => {
            assert_eq!(line, 2);
        }
        _ => panic!("Expected InvalidJson error"),
    }
}

#[test]
fn reports_line_number_on_missing_message_field() {
    let input = "{\"reason\":\"compiler-message\"}";

    let result = parse_cargo_messages(Cursor::new(input));
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        DiagnosticsParseError::InvalidShape { line, msg } => {
            assert_eq!(line, 1);
            assert!(msg.contains("message"));
        }
        _ => panic!("Expected InvalidShape error"),
    }
}

#[test]
fn reports_correct_line_number_after_empty_lines() {
    let input = "\n\n\n{\"reason\":\"compiler-message\"}";

    let result = parse_cargo_messages(Cursor::new(input));
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        DiagnosticsParseError::InvalidShape { line, .. } => {
            // Line 4 (after 3 empty lines)
            assert_eq!(line, 4);
        }
        _ => panic!("Expected InvalidShape error"),
    }
}

#[test]
fn continues_after_non_compiler_message() {
    let input = r#"{"reason":"build-script-executed"}
{"reason":"compiler-message","message":{"level":"error","message":"found"}}"#;

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].message, "found");
}

// =============================================================================
// Large Stream Handling
// =============================================================================

#[test]
fn handles_large_stream() {
    let mut input = String::new();
    for i in 0..1000 {
        input.push_str(&format!(
            r#"{{"reason":"compiler-message","message":{{"level":"warning","message":"msg {}"}}}}
"#,
            i
        ));
    }

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 1000);
    assert_eq!(diags[0].message, "msg 0");
    assert_eq!(diags[999].message, "msg 999");
}

#[test]
fn handles_large_individual_message() {
    let large_message = "x".repeat(100_000);
    let input = format!(
        r#"{{"reason":"compiler-message","message":{{"level":"error","message":"{}"}}}}"#,
        large_message
    );

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags[0].message.len(), 100_000);
}

#[test]
fn handles_many_spans_per_message() {
    let mut spans = String::from("[");
    for i in 0..100 {
        if i > 0 {
            spans.push(',');
        }
        spans.push_str(&format!(r#"{{"file_name":"{}.rs","line_start":{}}}"#, i, i));
    }
    spans.push(']');

    let input = format!(
        r#"{{"reason":"compiler-message","message":{{"level":"error","message":"test","spans":{}}}}}"#,
        spans
    );

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert_eq!(diags[0].spans.len(), 100);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn handles_empty_stream() {
    let input = "";
    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert!(diags.is_empty());
}

#[test]
fn handles_stream_with_only_whitespace() {
    let input = "   \n\t\n   \n";
    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert!(diags.is_empty());
}

#[test]
fn handles_single_diagnostic() {
    let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"single"}}"#;
    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 1);
}

#[test]
fn preserves_diagnostic_order() {
    let mut input = String::new();
    for i in 0..10 {
        input.push_str(&format!(
            r#"{{"reason":"compiler-message","message":{{"level":"error","message":"order_{}"}}}}
"#,
            i
        ));
    }

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    for (i, diag) in diags.iter().enumerate() {
        assert_eq!(diag.message, format!("order_{}", i));
    }
}

// =============================================================================
// Real-World Cargo Output Simulation
// =============================================================================

#[test]
fn simulates_typical_cargo_check_output() {
    let input = r#"{"reason":"compiler-artifact","package_id":"test 0.1.0","target":{"name":"test","kind":["lib"]},"profile":{"test":false},"features":[],"filenames":["target/debug/libtest.rlib"],"executable":null,"fresh":false}
{"reason":"compiler-message","message":{"level":"warning","message":"unused variable: `x`","code":{"code":"unused_variables"},"spans":[{"file_name":"src/lib.rs","line_start":1,"line_end":1,"column_start":9,"column_end":10,"is_primary":true}],"children":[{"level":"note","message":"`#[warn(unused_variables)]` on by default","spans":[]}],"rendered":"warning: unused variable: `x`\n --> src/lib.rs:1:9\n  |\n1 |     let x = 1;\n  |         ^ help: try ignoring the field: `_, `\n  |\n  = note: `#[warn(unused_variables)]` on by default\n\n"}}
{"reason":"build-finished","success":true}"#;

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, DiagnosticLevel::Warning);
    assert_eq!(diags[0].code_raw.as_deref(), Some("unused_variables"));
    assert_eq!(diags[0].spans.len(), 1);
}

#[test]
fn simulates_cargo_build_with_errors() {
    let input = r#"{"reason":"compiler-artifact","package_id":"test 0.1.0","target":{"name":"test","kind":["lib"]}}
{"reason":"compiler-message","message":{"level":"error","message":"cannot find value `undefined` in this scope","code":{"code":"E0425"},"spans":[{"file_name":"src/main.rs","line_start":5,"line_end":5,"column_start":5,"column_end":14,"is_primary":true,"label":"not found in this scope"}]}}
{"reason":"build-finished","success":false}"#;

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
}

#[test]
fn simulates_clippy_output() {
    let input = r#"{"reason":"compiler-message","message":{"level":"warning","message":"this expression creates a reference to a reference","code":{"code":"clippy::needless_borrow"},"spans":[{"file_name":"src/lib.rs","line_start":10,"line_end":10,"column_start":5,"column_end":10,"is_primary":true}],"children":[{"level":"help","message":"try removing the `&`","spans":[{"file_name":"src/lib.rs","line_start":10,"line_end":10,"column_start":5,"column_end":6,"is_primary":true,"suggested_replacement":"","suggestion_applicability":"MachineApplicable"}]}]}}"#;

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(
        diags[0].code_raw.as_deref(),
        Some("clippy::needless_borrow")
    );
}

// =============================================================================
// IO Error Simulation
// =============================================================================

/// A reader that always returns an error after some bytes.
struct FailingReader {
    bytes_to_read: usize,
}

impl Read for FailingReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.bytes_to_read == 0 {
            return Err(std::io::Error::other("simulated IO error"));
        }
        let to_read = buf.len().min(self.bytes_to_read);
        for b in &mut buf[..to_read] {
            *b = b'x';
        }
        self.bytes_to_read -= to_read;
        Ok(to_read)
    }
}

#[test]
fn handles_io_error_gracefully() {
    let reader = FailingReader { bytes_to_read: 0 };
    let result = parse_cargo_messages(BufReader::new(reader));
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        DiagnosticsParseError::InvalidShape { msg, .. } => {
            assert!(msg.contains("io error"));
        }
        _ => panic!("Expected InvalidShape error with io error message"),
    }
}

// =============================================================================
// Unicode and Special Characters in Stream
// =============================================================================

#[test]
fn handles_unicode_in_messages() {
    let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"エラー: 日本語のメッセージ"}}"#;

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert_eq!(diags[0].message, "エラー: 日本語のメッセージ");
}

#[test]
fn handles_emoji_in_messages() {
    let input = r#"{"reason":"compiler-message","message":{"level":"warning","message":"Warning! ⚠️ Check this! 🔍"}}"#;

    let diags = parse_cargo_messages(Cursor::new(input)).unwrap();
    assert!(diags[0].message.contains("⚠️"));
    assert!(diags[0].message.contains("🔍"));
}

#[test]
fn handles_multiline_rendered_output() {
    let rendered = "error[E0308]: mismatched types\n --> src/main.rs:4:5\n  |\n4 |     x\n  |     ^ expected integer, found &str\n";
    let input = format!(
        r#"{{"reason":"compiler-message","message":{{"level":"error","message":"mismatched types","rendered":"{}"}}}}"#,
        rendered.replace('\n', "\\n").replace('"', "\\\"")
    );

    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();
    assert!(diags[0]
        .rendered
        .as_ref()
        .unwrap()
        .contains("mismatched types"));
}

// =============================================================================
// Concurrent/Streaming Scenarios
// =============================================================================

#[test]
fn handles_incremental_output_simulation() {
    // Simulates how cargo might output messages incrementally
    let lines = [
        r#"{"reason":"compiler-artifact","package_id":"dep1"}"#,
        r#"{"reason":"compiler-artifact","package_id":"dep2"}"#,
        r#"{"reason":"compiler-message","message":{"level":"warning","message":"first warning"}}"#,
        r#"{"reason":"compiler-artifact","package_id":"main"}"#,
        r#"{"reason":"compiler-message","message":{"level":"error","message":"first error"}}"#,
        r#"{"reason":"compiler-message","message":{"level":"warning","message":"second warning"}}"#,
        r#"{"reason":"build-finished","success":false}"#,
    ];

    let input = lines.join("\n");
    let diags = parse_cargo_messages(Cursor::new(&input)).unwrap();

    assert_eq!(diags.len(), 3);
    assert_eq!(diags[0].level, DiagnosticLevel::Warning);
    assert_eq!(diags[1].level, DiagnosticLevel::Error);
    assert_eq!(diags[2].level, DiagnosticLevel::Warning);
}
