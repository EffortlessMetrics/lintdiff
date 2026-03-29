//! Comprehensive tests for lintdiff-jsonl crate.

use lintdiff_jsonl::{parse_jsonl, to_jsonl, JsonlBuilder, JsonlError, JsonlParser};
use serde_json::json;

// =============================================================================
// JsonlParser::new and from_string tests (8 tests)
// =============================================================================

#[test]
fn test_parser_new_with_cursor() {
    let data = "{\"test\":1}";
    let cursor = std::io::Cursor::new(data);
    let mut parser = JsonlParser::new(cursor);
    let result = parser.next().unwrap();
    assert!(result.is_some());
}

#[test]
fn test_parser_from_string_empty() {
    let mut parser = JsonlParser::from_string("");
    let result = parser.next().unwrap();
    assert!(result.is_none());
}

#[test]
fn test_parser_from_string_single_line() {
    let data = "{\"type\":\"diagnostic\"}";
    let mut parser = JsonlParser::from_string(data);
    let result = parser.next().unwrap();
    assert_eq!(result, Some(json!({"type": "diagnostic"})));
}

#[test]
fn test_parser_from_string_multiple_lines() {
    let data = "{\"a\":1}\n{\"b\":2}\n{\"c\":3}";
    let mut parser = JsonlParser::from_string(data);

    assert_eq!(parser.next().unwrap(), Some(json!({"a": 1})));
    assert_eq!(parser.next().unwrap(), Some(json!({"b": 2})));
    assert_eq!(parser.next().unwrap(), Some(json!({"c": 3})));
    assert!(parser.next().unwrap().is_none());
}

#[test]
fn test_parser_from_string_with_trailing_newline() {
    let data = "{\"x\":10}\n";
    let mut parser = JsonlParser::from_string(data);

    assert_eq!(parser.next().unwrap(), Some(json!({"x": 10})));
    assert!(parser.next().unwrap().is_none());
}

#[test]
fn fn_test_parser_from_string_with_leading_newline() {
    let data = "\n{\"y\":20}";
    let mut parser = JsonlParser::from_string(data);

    // Empty lines should be skipped
    assert_eq!(parser.next().unwrap(), Some(json!({"y": 20})));
    assert!(parser.next().unwrap().is_none());
}

#[test]
fn test_parser_from_string_with_multiple_empty_lines() {
    let data = "{\"a\":1}\n\n\n{\"b\":2}";
    let mut parser = JsonlParser::from_string(data);

    assert_eq!(parser.next().unwrap(), Some(json!({"a": 1})));
    assert_eq!(parser.next().unwrap(), Some(json!({"b": 2})));
    assert!(parser.next().unwrap().is_none());
}

#[test]
fn test_parser_from_string_with_windows_line_endings() {
    let data = "{\"a\":1}\r\n{\"b\":2}\r\n";
    let mut parser = JsonlParser::from_string(data);

    assert_eq!(parser.next().unwrap(), Some(json!({"a": 1})));
    assert_eq!(parser.next().unwrap(), Some(json!({"b": 2})));
    assert!(parser.next().unwrap().is_none());
}

// =============================================================================
// JsonlParser::next tests (10 tests)
// =============================================================================

#[test]
fn test_next_returns_none_after_exhausted() {
    let data = "{\"a\":1}";
    let mut parser = JsonlParser::from_string(data);

    let _ = parser.next();
    let _ = parser.next(); // Should return None
    let result = parser.next().unwrap();
    assert!(result.is_none());
}

#[test]
fn test_next_parses_simple_object() {
    let data = "{\"name\":\"test\"}";
    let mut parser = JsonlParser::from_string(data);

    let result = parser.next().unwrap();
    assert_eq!(result, Some(json!({"name": "test"})));
}

#[test]
fn test_next_parses_nested_object() {
    let data = "{\"outer\":{\"inner\":\"value\"}}";
    let mut parser = JsonlParser::from_string(data);

    let result = parser.next().unwrap();
    assert_eq!(result, Some(json!({"outer": {"inner": "value"}})));
}

#[test]
fn test_next_parses_array() {
    let data = "{\"items\":[1,2,3]}";
    let mut parser = JsonlParser::from_string(data);

    let result = parser.next().unwrap();
    assert_eq!(result, Some(json!({"items": [1, 2, 3]})));
}

#[test]
fn test_next_parses_null_value() {
    let data = "{\"value\":null}";
    let mut parser = JsonlParser::from_string(data);

    let result = parser.next().unwrap();
    assert_eq!(result, Some(json!({"value": null})));
}

#[test]
fn test_next_parses_boolean_values() {
    let data = "{\"a\":true,\"b\":false}";
    let mut parser = JsonlParser::from_string(data);

    let result = parser.next().unwrap();
    assert_eq!(result, Some(json!({"a": true, "b": false})));
}

#[test]
fn test_next_parses_numeric_values() {
    let data = "{\"int\":42,\"float\":3.14,\"negative\":-10,\"exp\":1e5}";
    let mut parser = JsonlParser::from_string(data);

    let result = parser.next().unwrap();
    assert_eq!(
        result,
        Some(json!({"int": 42, "float": 3.14, "negative": -10, "exp": 1e5}))
    );
}

#[test]
fn test_next_parses_string_with_special_chars() {
    let data = "{\"msg\":\"hello\\nworld\"}";
    let mut parser = JsonlParser::from_string(data);

    let result = parser.next().unwrap();
    assert_eq!(result, Some(json!({"msg": "hello\nworld"})));
}

#[test]
fn test_next_handles_empty_object() {
    let data = "{}";
    let mut parser = JsonlParser::from_string(data);

    let result = parser.next().unwrap();
    assert_eq!(result, Some(json!({})));
}

#[test]
fn test_next_returns_error_on_invalid_json() {
    let data = "{invalid json}";
    let mut parser = JsonlParser::from_string(data);

    let result = parser.next();
    assert!(result.is_err());
    match result.unwrap_err() {
        JsonlError::Parse { line, details } => {
            assert!(line.contains("invalid"));
            assert!(!details.is_empty());
        }
        _ => panic!("Expected Parse error"),
    }
}

// =============================================================================
// JsonlParser::collect tests (8 tests)
// =============================================================================

#[test]
fn test_collect_empty_input() {
    let mut parser = JsonlParser::from_string("");
    let result = parser.collect().unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_collect_single_object() {
    let mut parser = JsonlParser::from_string("{\"a\":1}");
    let result = parser.collect().unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], json!({"a": 1}));
}

#[test]
fn test_collect_multiple_objects() {
    let mut parser = JsonlParser::from_string("{\"a\":1}\n{\"b\":2}\n{\"c\":3}");
    let result = parser.collect().unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], json!({"a": 1}));
    assert_eq!(result[1], json!({"b": 2}));
    assert_eq!(result[2], json!({"c": 3}));
}

#[test]
fn test_collect_with_empty_lines() {
    let mut parser = JsonlParser::from_string("{\"a\":1}\n\n{\"b\":2}\n\n\n{\"c\":3}");
    let result = parser.collect().unwrap();
    assert_eq!(result.len(), 3);
}

#[test]
fn test_collect_large_input() {
    let mut data = String::new();
    for i in 0..100 {
        if i > 0 {
            data.push('\n');
        }
        data.push_str(&format!("{{\"id\":{}}}", i));
    }

    let mut parser = JsonlParser::from_string(&data);
    let result = parser.collect().unwrap();
    assert_eq!(result.len(), 100);
    assert_eq!(result[0], json!({"id": 0}));
    assert_eq!(result[99], json!({"id": 99}));
}

#[test]
fn test_collect_preserves_order() {
    let mut parser = JsonlParser::from_string("{\"order\":1}\n{\"order\":2}\n{\"order\":3}");
    let result = parser.collect().unwrap();

    assert_eq!(result[0]["order"], 1);
    assert_eq!(result[1]["order"], 2);
    assert_eq!(result[2]["order"], 3);
}

#[test]
fn test_collect_with_complex_objects() {
    let data = r#"{"type":"error","message":"test","span":{"start":1,"end":10}}
{"type":"warning","message":"warn","span":{"start":20,"end":30}}"#;
    let mut parser = JsonlParser::from_string(data);
    let result = parser.collect().unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0]["type"], "error");
    assert_eq!(result[1]["type"], "warning");
}

#[test]
fn test_collect_returns_error_on_invalid_line() {
    let data = "{\"a\":1}\n{invalid}\n{\"b\":2}";
    let mut parser = JsonlParser::from_string(data);
    let result = parser.collect();
    assert!(result.is_err());
}

// =============================================================================
// JsonlBuilder tests (10 tests)
// =============================================================================

#[test]
fn test_builder_new_creates_empty_buffer() {
    let builder = JsonlBuilder::new();
    assert!(builder.is_empty());
    assert_eq!(builder.len(), 0);
}

#[test]
fn test_builder_default_same_as_new() {
    let builder = JsonlBuilder::default();
    assert!(builder.is_empty());
}

#[test]
fn test_builder_push_single_value() {
    let mut builder = JsonlBuilder::new();
    builder.push(&json!({"a": 1})).unwrap();

    assert!(!builder.is_empty());
    assert!(builder.len() > 0);
}

#[test]
fn test_builder_push_multiple_values() {
    let mut builder = JsonlBuilder::new();
    builder.push(&json!({"a": 1})).unwrap();
    builder.push(&json!({"b": 2})).unwrap();
    builder.push(&json!({"c": 3})).unwrap();

    assert_eq!(builder.len(), 24); // Each line is 8 chars: {"a":1}\n
}

#[test]
fn test_builder_build_empty() {
    let builder = JsonlBuilder::new();
    let result = builder.build();
    assert!(result.is_empty());
}

#[test]
fn test_builder_build_single_value() {
    let mut builder = JsonlBuilder::new();
    builder.push(&json!({"a": 1})).unwrap();
    let result = builder.build();

    assert_eq!(result, "{\"a\":1}\n");
}

#[test]
fn test_builder_build_multiple_values() {
    let mut builder = JsonlBuilder::new();
    builder.push(&json!({"a": 1})).unwrap();
    builder.push(&json!({"b": 2})).unwrap();
    let result = builder.build();

    assert_eq!(result, "{\"a\":1}\n{\"b\":2}\n");
}

#[test]
fn test_builder_push_array() {
    let mut builder = JsonlBuilder::new();
    builder.push(&json!([1, 2, 3])).unwrap();
    let result = builder.build();

    assert_eq!(result, "[1,2,3]\n");
}

#[test]
fn test_builder_push_null() {
    let mut builder = JsonlBuilder::new();
    builder.push(&json!(null)).unwrap();
    let result = builder.build();

    assert_eq!(result, "null\n");
}

#[test]
fn test_builder_push_complex_nested_structure() {
    let mut builder = JsonlBuilder::new();
    let complex = json!({
        "type": "diagnostic",
        "data": {
            "nested": {
                "deep": [1, 2, {"key": "value"}]
            }
        }
    });
    builder.push(&complex).unwrap();
    let result = builder.build();

    assert!(result.contains("diagnostic"));
    assert!(result.contains("nested"));
    assert!(result.ends_with('\n'));
}

// =============================================================================
// parse_jsonl function tests (8 tests)
// =============================================================================

#[test]
fn test_parse_jsonl_empty_string() {
    let result = parse_jsonl("").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_parse_jsonl_single_line() {
    let result = parse_jsonl("{\"test\":1}").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], json!({"test": 1}));
}

#[test]
fn test_parse_jsonl_multiple_lines() {
    let result = parse_jsonl("{\"a\":1}\n{\"b\":2}\n{\"c\":3}").unwrap();
    assert_eq!(result.len(), 3);
}

#[test]
fn test_parse_jsonl_with_trailing_newline() {
    let result = parse_jsonl("{\"a\":1}\n").unwrap();
    assert_eq!(result.len(), 1);
}

#[test]
fn test_parse_jsonl_with_empty_lines() {
    let result = parse_jsonl("{\"a\":1}\n\n{\"b\":2}").unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_parse_jsonl_diagnostic_format() {
    let data = r#"{"type":"error","message":"unused variable","file":"src/lib.rs","line":42}
{"type":"warning","message":"unused import","file":"src/main.rs","line":10}"#;
    let result = parse_jsonl(data).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0]["type"], "error");
    assert_eq!(result[1]["type"], "warning");
}

#[test]
fn test_parse_jsonl_returns_error_on_invalid() {
    let result = parse_jsonl("{\"valid\":1}\n{invalid}");
    assert!(result.is_err());
}

#[test]
fn test_parse_jsonl_handles_unicode() {
    let data = "{\"msg\":\"Hello 世界\"}\n{\"emoji\":\"🎉\"}";
    let result = parse_jsonl(data).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0]["msg"], "Hello 世界");
    assert_eq!(result[1]["emoji"], "🎉");
}

// =============================================================================
// to_jsonl function tests (6 tests)
// =============================================================================

#[test]
fn test_to_jsonl_empty_slice() {
    let values: Vec<serde_json::Value> = vec![];
    let result = to_jsonl(&values).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_to_jsonl_single_value() {
    let values = vec![json!({"a": 1})];
    let result = to_jsonl(&values).unwrap();
    assert_eq!(result, "{\"a\":1}\n");
}

#[test]
fn test_to_jsonl_multiple_values() {
    let values = vec![json!({"a": 1}), json!({"b": 2}), json!({"c": 3})];
    let result = to_jsonl(&values).unwrap();
    assert_eq!(result, "{\"a\":1}\n{\"b\":2}\n{\"c\":3}\n");
}

#[test]
fn test_to_jsonl_with_arrays() {
    let values = vec![json!([1, 2, 3]), json!(["a", "b", "c"])];
    let result = to_jsonl(&values).unwrap();
    assert!(result.contains("[1,2,3]"));
    assert!(result.contains("[\"a\",\"b\",\"c\"]"));
}

#[test]
fn test_to_jsonl_with_primitives() {
    let values = vec![json!(42), json!("hello"), json!(true), json!(null)];
    let result = to_jsonl(&values).unwrap();

    assert!(result.contains("42"));
    assert!(result.contains("\"hello\""));
    assert!(result.contains("true"));
    assert!(result.contains("null"));
}

#[test]
fn test_to_jsonl_roundtrip() {
    let original = vec![
        json!({"type": "error", "code": "E0001"}),
        json!({"type": "warning", "code": "W0001"}),
    ];

    let jsonl = to_jsonl(&original).unwrap();
    let parsed = parse_jsonl(&jsonl).unwrap();

    assert_eq!(original, parsed);
}

// =============================================================================
// Error cases tests (5 tests)
// =============================================================================

#[test]
fn test_error_parse_invalid_json_object() {
    let data = "{not valid}";
    let result = parse_jsonl(data);
    assert!(result.is_err());

    match result.unwrap_err() {
        JsonlError::Parse { line, details } => {
            assert!(line.contains("not valid"));
            assert!(!details.is_empty());
        }
        _ => panic!("Expected Parse error"),
    }
}

#[test]
fn test_error_parse_truncated_json() {
    let data = "{\"a\":";
    let result = parse_jsonl(data);
    assert!(result.is_err());
}

#[test]
fn test_error_parse_unclosed_bracket() {
    let data = "{\"a\":[1,2,3";
    let result = parse_jsonl(data);
    assert!(result.is_err());
}

#[test]
fn test_error_parse_invalid_escape() {
    let data = "{\"msg\":\"bad\\escape\"}";
    let result = parse_jsonl(data);
    assert!(result.is_err());
}

#[test]
fn test_error_parse_mixed_valid_invalid() {
    let data = "{\"valid\":1}\n{\"also\":\"valid\"}\n{broken}";
    let mut parser = JsonlParser::from_string(data);

    // First two should succeed
    assert!(parser.next().unwrap().is_some());
    assert!(parser.next().unwrap().is_some());

    // Third should fail
    let result = parser.next();
    assert!(result.is_err());
}

// =============================================================================
// Additional edge case tests
// =============================================================================

#[test]
fn test_parser_static_lifetime() {
    // Test that parser can work with 'static data
    let data: &'static str = "{\"test\":1}";
    let mut parser = JsonlParser::from_string(data);
    let result = parser.next().unwrap();
    assert!(result.is_some());
}

#[test]
fn test_builder_len_updates_correctly() {
    let mut builder = JsonlBuilder::new();
    assert_eq!(builder.len(), 0);

    builder.push(&json!({"a": 1})).unwrap();
    let len_after_first = builder.len();

    builder.push(&json!({"b": 2})).unwrap();
    let len_after_second = builder.len();

    assert!(len_after_second > len_after_first);
}

#[test]
fn test_parse_jsonl_whitespace_only_lines() {
    let data = "{\"a\":1}\n   \n{\"b\":2}";
    let result = parse_jsonl(data).unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_parser_with_vec_u8_reader() {
    let data = b"{\"bytes\":true}";
    let mut parser = JsonlParser::new(&data[..]);
    let result = parser.next().unwrap();
    assert_eq!(result, Some(json!({"bytes": true})));
}

#[test]
fn test_builder_with_empty_object() {
    let mut builder = JsonlBuilder::new();
    builder.push(&json!({})).unwrap();
    let result = builder.build();
    assert_eq!(result, "{}\n");
}

#[test]
fn test_parser_parse_static_method() {
    let data = "{\"static\":true}";
    let result = JsonlParser::parse(data).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], json!({"static": true}));
}

#[test]
fn test_parse_jsonl_with_very_long_line() {
    let long_value = "x".repeat(10000);
    let data = format!("{{\"long\":\"{}\"}}", long_value);
    let result = parse_jsonl(&data).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0]["long"], long_value);
}

#[test]
fn test_to_jsonl_with_nested_objects() {
    let values = vec![json!({
        "level1": {
            "level2": {
                "level3": "deep"
            }
        }
    })];
    let result = to_jsonl(&values).unwrap();
    assert!(result.contains("level1"));
    assert!(result.contains("level2"));
    assert!(result.contains("level3"));
    assert!(result.contains("deep"));
}
