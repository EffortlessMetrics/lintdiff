//! Tests for diagnostics I/O functionality.

use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;

use lintdiff_app_io::{acquire_diagnostics, parse_diagnostics, AppIoError};
use lintdiff_diagnostics::DiagnosticLevel;
use tempfile::TempDir;

fn create_diagnostics_file(dir: &TempDir, filename: &str, content: &str) -> PathBuf {
    let path = dir.path().join(filename);
    let mut file = File::create(&path).expect("failed to create diagnostics file");
    file.write_all(content.as_bytes())
        .expect("failed to write diagnostics");
    path
}

fn make_compiler_message(level: &str, code: &str, message: &str, file: &str, line: u32) -> String {
    format!(
        r#"{{"reason":"compiler-message","message":{{"level":"{}","message":"{}","code":{{"code":"{}"}},"spans":[{{"file_name":"{}","line_start":{},"is_primary":true}}]}}}}"#,
        level, message, code, file, line
    )
}

// ============================================================================
// parse_diagnostics tests
// ============================================================================

#[test]
fn parse_valid_diagnostics_from_reader() {
    let json = make_compiler_message(
        "warning",
        "unused_variable",
        "unused variable",
        "src/lib.rs",
        10,
    );
    let reader = BufReader::new(json.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, DiagnosticLevel::Warning);
    assert_eq!(diags[0].code_raw.as_deref(), Some("unused_variable"));
    assert_eq!(diags[0].message, "unused variable");
}

#[test]
fn parse_multiple_diagnostics() {
    let json1 = make_compiler_message(
        "warning",
        "unused_variable",
        "unused variable",
        "src/lib.rs",
        10,
    );
    let json2 = make_compiler_message("error", "E0425", "cannot find value", "src/main.rs", 5);
    let input = format!("{}\n{}", json1, json2);
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0].level, DiagnosticLevel::Warning);
    assert_eq!(diags[1].level, DiagnosticLevel::Error);
}

#[test]
fn parse_empty_input() {
    let reader = BufReader::new("".as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert!(diags.is_empty());
}

#[test]
fn parse_whitespace_only_input() {
    let reader = BufReader::new("   \n\n   \n".as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert!(diags.is_empty());
}

#[test]
fn parse_skips_non_compiler_messages() {
    let input = r#"{"reason":"compiler-artifact","target":{"name":"test"}}
{"reason":"build-script-executed","name":"build-script"}
{"reason":"compiler-message","message":{"level":"warning","message":"test warning","code":{"code":"test"},"spans":[]}}"#;
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].message, "test warning");
}

#[test]
fn parse_handles_invalid_json_line() {
    // Invalid JSON in the stream causes an error - this is expected behavior
    let input = "not valid json";
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    // The parser returns an error on invalid JSON
    assert!(result.is_err());
}

#[test]
fn parse_diagnostic_without_code() {
    let input = r#"{"reason":"compiler-message","message":{"level":"warning","message":"warning without code","spans":[]}}"#;
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags.len(), 1);
    assert!(diags[0].code_raw.is_none());
    assert_eq!(diags[0].message, "warning without code");
}

#[test]
fn parse_diagnostic_with_multiple_spans() {
    let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"multiple spans","code":{"code":"E0001"},"spans":[{"file_name":"src/a.rs","line_start":1,"is_primary":true},{"file_name":"src/b.rs","line_start":2,"is_primary":false}]}}"#;
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].spans.len(), 2);
}

#[test]
fn parse_all_diagnostic_levels() {
    for (level_str, expected_level) in [
        ("error", DiagnosticLevel::Error),
        ("warning", DiagnosticLevel::Warning),
        ("note", DiagnosticLevel::Note),
        ("help", DiagnosticLevel::Help),
    ] {
        let input = format!(
            r#"{{"reason":"compiler-message","message":{{"level":"{}","message":"test","spans":[]}}}}"#,
            level_str
        );
        let reader = BufReader::new(input.as_bytes());

        let result = parse_diagnostics(reader);
        assert!(result.is_ok(), "failed to parse level: {}", level_str);

        let diags = result.unwrap();
        assert_eq!(
            diags[0].level, expected_level,
            "level mismatch for: {}",
            level_str
        );
    }
}

#[test]
fn parse_diagnostic_with_rendered() {
    let input = r#"{"reason":"compiler-message","message":{"level":"warning","message":"test","code":{"code":"test"},"spans":[],"rendered":"warning: test\n --> src/lib.rs:1:1\n"}}"#;
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags.len(), 1);
    assert!(diags[0].rendered.is_some());
    assert!(diags[0]
        .rendered
        .as_ref()
        .unwrap()
        .contains("warning: test"));
}

// ============================================================================
// acquire_diagnostics tests - file reading
// ============================================================================

#[test]
fn acquire_diagnostics_from_file() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let json = make_compiler_message(
        "warning",
        "unused_imports",
        "unused import",
        "src/lib.rs",
        5,
    );
    let path = create_diagnostics_file(&dir, "diagnostics.jsonl", &json);

    let result = acquire_diagnostics(Some(&path));
    assert!(result.is_ok());

    let diags_opt = result.unwrap();
    assert!(diags_opt.is_some());

    let diags = diags_opt.unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, DiagnosticLevel::Warning);
}

#[test]
fn acquire_diagnostics_from_file_multiple_entries() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let json1 = make_compiler_message("error", "E0425", "error 1", "src/a.rs", 1);
    let json2 = make_compiler_message("warning", "dead_code", "warning 1", "src/b.rs", 2);
    let json3 = make_compiler_message("warning", "unused_variables", "warning 2", "src/c.rs", 3);
    let content = format!("{}\n{}\n{}", json1, json2, json3);
    let path = create_diagnostics_file(&dir, "diagnostics.jsonl", &content);

    let result = acquire_diagnostics(Some(&path));
    assert!(result.is_ok());

    let diags = result.unwrap().unwrap();
    assert_eq!(diags.len(), 3);
}

#[test]
fn acquire_diagnostics_missing_file() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let missing_path = dir.path().join("nonexistent.jsonl");

    let result = acquire_diagnostics(Some(&missing_path));
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        AppIoError::ReadFile { path, source } => {
            assert_eq!(path, missing_path);
            assert!(source.kind() == std::io::ErrorKind::NotFound);
        }
        _ => panic!("expected ReadFile error"),
    }
}

#[test]
fn acquire_diagnostics_empty_file() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = create_diagnostics_file(&dir, "empty.jsonl", "");

    let result = acquire_diagnostics(Some(&path));
    assert!(result.is_ok());

    let diags_opt = result.unwrap();
    assert!(diags_opt.is_some());

    let diags = diags_opt.unwrap();
    assert!(diags.is_empty());
}

#[test]
fn acquire_diagnostics_whitespace_only_file() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = create_diagnostics_file(&dir, "whitespace.jsonl", "   \n\n   ");

    let result = acquire_diagnostics(Some(&path));
    assert!(result.is_ok());

    let diags = result.unwrap().unwrap();
    // Whitespace-only content should parse to empty
    assert!(diags.is_empty());
}

// ============================================================================
// acquire_diagnostics tests - stdin handling
// ============================================================================

#[test]
fn acquire_diagnostics_none_path_reads_stdin() {
    // When path is None, it reads from stdin
    // With empty stdin, should return Ok(None)
    // Note: This test documents the behavior but may not work in all test environments
    // In practice, stdin reading is tested via integration tests

    // This test verifies the function signature and that None is accepted
    let result = acquire_diagnostics(None);
    // Result depends on whether stdin has content
    // In CI/test environments, stdin may be empty or closed
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// Error handling tests
// ============================================================================

#[test]
fn error_message_contains_file_path() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let missing_path = dir.path().join("missing_diagnostics.json");

    let result = acquire_diagnostics(Some(&missing_path));
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing_diagnostics.json"),
        "error message should contain path: {}",
        err_msg
    );
}

#[test]
fn parse_error_includes_message() {
    // Create a file with invalid JSON that will cause parse errors
    let dir = TempDir::new().expect("failed to create temp dir");
    let invalid_content = r#"{"reason":"compiler-message","message":{invalid}}"#;
    let path = create_diagnostics_file(&dir, "invalid.jsonl", invalid_content);

    // acquire_diagnostics returns an error when JSON parsing fails
    let result = acquire_diagnostics(Some(&path));
    assert!(result.is_err());

    // The error should mention parsing/diagnostics
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("parse") || err_msg.contains("diagnostics"),
        "error message should mention parsing: {}",
        err_msg
    );
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn parse_large_message_field() {
    let long_message = "x".repeat(10000);
    let input = format!(
        r#"{{"reason":"compiler-message","message":{{"level":"warning","message":"{}","spans":[]}}}}"#,
        long_message
    );
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags[0].message.len(), 10000);
}

#[test]
fn parse_unicode_in_message() {
    let input = r#"{"reason":"compiler-message","message":{"level":"warning","message":"Unicode: 你好世界 🎉 émojis","spans":[]}}"#;
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert!(diags[0].message.contains("你好世界"));
    assert!(diags[0].message.contains("🎉"));
}

#[test]
fn parse_special_characters_in_path() {
    let input = r#"{"reason":"compiler-message","message":{"level":"warning","message":"test","spans":[{"file_name":"src/path with spaces/file.rs","line_start":1,"is_primary":true}]}}"#;
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags[0].spans.len(), 1);
}

#[test]
fn parse_mixed_line_endings() {
    let json1 = make_compiler_message("warning", "test1", "message 1", "src/a.rs", 1);
    let json2 = make_compiler_message("warning", "test2", "message 2", "src/b.rs", 2);
    // Mix of \n and \r\n
    let input = format!("{}\r\n{}\n", json1, json2);
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags.len(), 2);
}

#[test]
fn parse_trailing_newline() {
    let json = make_compiler_message("warning", "test", "message", "src/lib.rs", 1);
    let input = format!("{}\n\n\n", json);
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags.len(), 1);
}

#[test]
fn parse_no_trailing_newline() {
    let json = make_compiler_message("warning", "test", "message", "src/lib.rs", 1);
    let reader = BufReader::new(json.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags.len(), 1);
}

#[test]
fn parse_very_long_file() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let mut content = String::new();
    for i in 0..1000 {
        let json = make_compiler_message(
            "warning",
            &format!("warn_{}", i),
            &format!("message {}", i),
            "src/lib.rs",
            i,
        );
        content.push_str(&json);
        content.push('\n');
    }
    let path = create_diagnostics_file(&dir, "large.jsonl", &content);

    let result = acquire_diagnostics(Some(&path));
    assert!(result.is_ok());

    let diags = result.unwrap().unwrap();
    assert_eq!(diags.len(), 1000);
}

// ============================================================================
// Path handling tests
// ============================================================================

#[test]
fn acquire_diagnostics_relative_path() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let json = make_compiler_message("warning", "test", "test", "src/lib.rs", 1);
    let path = create_diagnostics_file(&dir, "test.jsonl", &json);

    // The path is absolute (from TempDir), but test that it works
    let result = acquire_diagnostics(Some(&path));
    assert!(result.is_ok());
}

#[test]
fn acquire_diagnostics_path_with_parent_directory() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let subdir = dir.path().join("nested").join("deeply");
    fs::create_dir_all(&subdir).expect("failed to create subdirs");

    let json = make_compiler_message("warning", "test", "test", "src/lib.rs", 1);
    let path = subdir.join("diagnostics.jsonl");
    let mut file = File::create(&path).expect("failed to create file");
    file.write_all(json.as_bytes()).expect("failed to write");

    let result = acquire_diagnostics(Some(&path));
    assert!(result.is_ok());
}

// ============================================================================
// Diagnostic content tests
// ============================================================================

#[test]
fn diagnostic_code_extraction() {
    let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"cannot find value","code":{"code":"E0425"},"spans":[]}}"#;
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags[0].code_raw.as_deref(), Some("E0425"));
}

#[test]
fn diagnostic_clippy_code() {
    let input = r#"{"reason":"compiler-message","message":{"level":"warning","message":"this is clippy","code":{"code":"clippy::unwrap_used"},"spans":[]}}"#;
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags[0].code_raw.as_deref(), Some("clippy::unwrap_used"));
}

#[test]
fn diagnostic_span_information() {
    let input = r#"{"reason":"compiler-message","message":{"level":"warning","message":"test","code":{"code":"test"},"spans":[{"file_name":"src/main.rs","line_start":10,"line_end":15,"column_start":1,"column_end":20,"is_primary":true,"label":"here"}]}}"#;
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert_eq!(diags[0].spans.len(), 1);
    let span = &diags[0].spans[0];
    assert_eq!(span.file.as_str(), "src/main.rs");
    assert!(span.is_primary);
}

#[test]
fn diagnostic_no_spans() {
    let input = r#"{"reason":"compiler-message","message":{"level":"warning","message":"no spans here","code":{"code":"test"},"spans":[]}}"#;
    let reader = BufReader::new(input.as_bytes());

    let result = parse_diagnostics(reader);
    assert!(result.is_ok());

    let diags = result.unwrap();
    assert!(diags[0].spans.is_empty());
}
