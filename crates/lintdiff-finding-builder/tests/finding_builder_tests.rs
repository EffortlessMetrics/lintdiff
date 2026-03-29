//! Comprehensive tests for the FindingBuilder.
//!
//! Tests cover:
//! - Basic builder usage
//! - Missing required fields
//! - Optional fields
//! - Edge cases
//! - Integration with lintdiff-types

use lintdiff_finding_builder::{BuildError, FindingBuilder};
use lintdiff_types::{Finding, Location, NormPath, Severity};
use serde_json::json;

// =============================================================================
// Basic Builder Usage Tests
// =============================================================================

#[test]
fn new_creates_default_builder() {
    let builder = FindingBuilder::new();
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn minimal_valid_finding() {
    let finding = FindingBuilder::new()
        .with_code("TEST001")
        .with_message("Test message")
        .build()
        .unwrap();

    assert_eq!(finding.code, "TEST001");
    assert_eq!(finding.message, "Test message");
    assert_eq!(finding.severity, Severity::Warn);
    assert!(finding.location.is_none());
    assert!(finding.check_id.is_none());
    assert!(finding.help.is_none());
    assert!(finding.url.is_none());
    assert!(finding.fingerprint.is_none());
    assert!(finding.data.is_none());
}

#[test]
fn fully_configured_finding() {
    let finding = FindingBuilder::new()
        .with_code("CLIPPY001")
        .with_message("Unnecessary allocation")
        .with_severity(Severity::Error)
        .with_path("src/utils/string_helpers.rs")
        .with_line(42)
        .with_col(15)
        .with_check_id("clippy::unnecessary_allocation")
        .with_help("Use `&str` instead of `String`")
        .with_url(
            "https://rust-lang.github.io/rust-clippy/master/index.html#unnecessary_allocation",
        )
        .with_fingerprint("a1b2c3d4e5f6")
        .with_data(json!({ "suggestion": "&str", "complexity": "O(1)" }))
        .build()
        .unwrap();

    assert_eq!(finding.code, "CLIPPY001");
    assert_eq!(finding.message, "Unnecessary allocation");
    assert_eq!(finding.severity, Severity::Error);

    let loc = finding.location.as_ref().unwrap();
    assert_eq!(loc.path.as_str(), "src/utils/string_helpers.rs");
    assert_eq!(loc.line, Some(42));
    assert_eq!(loc.col, Some(15));

    assert_eq!(
        finding.check_id,
        Some("clippy::unnecessary_allocation".to_string())
    );
    assert_eq!(
        finding.help,
        Some("Use `&str` instead of `String`".to_string())
    );
    assert!(finding.url.is_some());
    assert_eq!(finding.fingerprint, Some("a1b2c3d4e5f6".to_string()));
    assert!(finding.data.is_some());
}

// =============================================================================
// Required Field Validation Tests
// =============================================================================

#[test]
fn missing_code_returns_error() {
    let result = FindingBuilder::new().with_message("Some message").build();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), BuildError::MissingCode);
}

#[test]
fn missing_message_returns_error() {
    let result = FindingBuilder::new().with_code("CODE001").build();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), BuildError::MissingMessage);
}

#[test]
fn missing_both_required_returns_code_error_first() {
    let result = FindingBuilder::new().build();
    assert_eq!(result.unwrap_err(), BuildError::MissingCode);
}

#[test]
fn empty_code_is_accepted() {
    // Empty string is still a valid code (validation is not the builder's job)
    let finding = FindingBuilder::new()
        .with_code("")
        .with_message("message")
        .build()
        .unwrap();

    assert_eq!(finding.code, "");
}

#[test]
fn empty_message_is_accepted() {
    // Empty string is still a valid message (validation is not the builder's job)
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("")
        .build()
        .unwrap();

    assert_eq!(finding.message, "");
}

// =============================================================================
// Severity Tests
// =============================================================================

#[test]
fn default_severity_is_warn() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .build()
        .unwrap();

    assert_eq!(finding.severity, Severity::Warn);
}

#[test]
fn severity_info() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_severity(Severity::Info)
        .build()
        .unwrap();

    assert_eq!(finding.severity, Severity::Info);
}

#[test]
fn severity_error() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_severity(Severity::Error)
        .build()
        .unwrap();

    assert_eq!(finding.severity, Severity::Error);
}

// =============================================================================
// Location Tests
// =============================================================================

#[test]
fn location_created_when_path_set() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_path("src/main.rs")
        .build()
        .unwrap();

    assert!(finding.location.is_some());
    let loc = finding.location.unwrap();
    assert_eq!(loc.path.as_str(), "src/main.rs");
    assert!(loc.line.is_none());
    assert!(loc.col.is_none());
}

#[test]
fn location_with_line() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_path("src/main.rs")
        .with_line(100)
        .build()
        .unwrap();

    let loc = finding.location.unwrap();
    assert_eq!(loc.line, Some(100));
    assert!(loc.col.is_none());
}

#[test]
fn location_with_line_and_col() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_path("src/main.rs")
        .with_line(50)
        .with_col(20)
        .build()
        .unwrap();

    let loc = finding.location.unwrap();
    assert_eq!(loc.line, Some(50));
    assert_eq!(loc.col, Some(20));
}

#[test]
fn line_without_path_does_not_create_location() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_line(42)
        .build()
        .unwrap();

    assert!(finding.location.is_none());
}

#[test]
fn col_without_path_does_not_create_location() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_col(10)
        .build()
        .unwrap();

    assert!(finding.location.is_none());
}

#[test]
fn path_normalizes_backslashes() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_path("src\\nested\\file.rs")
        .build()
        .unwrap();

    let loc = finding.location.unwrap();
    assert_eq!(loc.path.as_str(), "src/nested/file.rs");
}

#[test]
fn path_normalizes_leading_dot() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_path("./src/lib.rs")
        .build()
        .unwrap();

    let loc = finding.location.unwrap();
    assert_eq!(loc.path.as_str(), "src/lib.rs");
}

// =============================================================================
// Optional Fields Tests
// =============================================================================

#[test]
fn check_id_is_optional() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_check_id("custom.check.id")
        .build()
        .unwrap();

    assert_eq!(finding.check_id, Some("custom.check.id".to_string()));
}

#[test]
fn help_is_optional() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_help("Try doing X instead")
        .build()
        .unwrap();

    assert_eq!(finding.help, Some("Try doing X instead".to_string()));
}

#[test]
fn url_is_optional() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_url("https://docs.example.com/errors/CODE")
        .build()
        .unwrap();

    assert_eq!(
        finding.url,
        Some("https://docs.example.com/errors/CODE".to_string())
    );
}

#[test]
fn fingerprint_is_optional() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_fingerprint("sha256:abc123")
        .build()
        .unwrap();

    assert_eq!(finding.fingerprint, Some("sha256:abc123".to_string()));
}

#[test]
fn data_is_optional() {
    let data = json!({
        "key": "value",
        "nested": {
            "array": [1, 2, 3]
        }
    });

    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_data(data.clone())
        .build()
        .unwrap();

    assert_eq!(finding.data, Some(data));
}

// =============================================================================
// Edge Cases Tests
// =============================================================================

#[test]
fn line_zero_is_valid() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_path("file.rs")
        .with_line(0)
        .build()
        .unwrap();

    assert_eq!(finding.location.unwrap().line, Some(0));
}

#[test]
fn col_zero_is_valid() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_path("file.rs")
        .with_col(0)
        .build()
        .unwrap();

    assert_eq!(finding.location.unwrap().col, Some(0));
}

#[test]
fn max_line_number() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_path("file.rs")
        .with_line(u32::MAX)
        .build()
        .unwrap();

    assert_eq!(finding.location.unwrap().line, Some(u32::MAX));
}

#[test]
fn unicode_in_message() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("Error: 你好世界 🌍")
        .build()
        .unwrap();

    assert_eq!(finding.message, "Error: 你好世界 🌍");
}

#[test]
fn unicode_in_path() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("message")
        .with_path("src/文件/测试.rs")
        .build()
        .unwrap();

    assert_eq!(finding.location.unwrap().path.as_str(), "src/文件/测试.rs");
}

#[test]
fn unicode_in_code() {
    let finding = FindingBuilder::new()
        .with_code("错误-001")
        .with_message("message")
        .build()
        .unwrap();

    assert_eq!(finding.code, "错误-001");
}

#[test]
fn very_long_message() {
    let long_msg = "x".repeat(10000);
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message(long_msg.clone())
        .build()
        .unwrap();

    assert_eq!(finding.message.len(), 10000);
}

#[test]
fn multiline_message() {
    let multiline = "Line 1\nLine 2\nLine 3";
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message(multiline)
        .build()
        .unwrap();

    assert!(finding.message.contains('\n'));
}

// =============================================================================
// Builder Reuse Tests
// =============================================================================

#[test]
fn builder_can_be_cloned() {
    let base = FindingBuilder::new()
        .with_code("BASE")
        .with_message("Base message");

    let clone = base.clone();
    let finding1 = base.build().unwrap();
    let finding2 = clone.build().unwrap();

    assert_eq!(finding1.code, finding2.code);
    assert_eq!(finding1.message, finding2.message);
}

#[test]
fn builder_chain_is_idiomatic() {
    // Test that the builder pattern allows fluent chaining
    let finding = FindingBuilder::new()
        // Required fields
        .with_code("CODE")
        .with_message("Message")
        // Optional fields in any order
        .with_severity(Severity::Error)
        .with_path("file.rs")
        .with_line(1)
        .with_col(1)
        .with_check_id("check")
        .with_help("help")
        .with_url("url")
        .with_fingerprint("fp")
        .with_data(json!({}))
        .build()
        .unwrap();

    assert_eq!(finding.code, "CODE");
}

// =============================================================================
// Error Display Tests
// =============================================================================

#[test]
fn build_error_missing_code_display() {
    let err = BuildError::MissingCode;
    let display = format!("{}", err);
    assert!(display.contains("code"));
}

#[test]
fn build_error_missing_message_display() {
    let err = BuildError::MissingMessage;
    let display = format!("{}", err);
    assert!(display.contains("message"));
}

#[test]
fn build_error_is_std_error() {
    fn takes_error(_err: &dyn std::error::Error) {}

    let err = BuildError::MissingCode;
    takes_error(&err);
}

// =============================================================================
// Integration with lintdiff-types Tests
// =============================================================================

#[test]
fn finding_is_compatible_with_lintdiff_types() {
    let finding = FindingBuilder::new()
        .with_code("TEST")
        .with_message("Test")
        .with_severity(Severity::Error)
        .with_path("src/lib.rs")
        .with_line(10)
        .build()
        .unwrap();

    // Ensure the built Finding matches the expected type
    let _: Finding = finding.clone();

    // Verify it can be used as a lintdiff_types::Finding
    assert_eq!(finding.severity, Severity::Error);
}

#[test]
fn location_is_compatible_with_lintdiff_types() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("msg")
        .with_path("test.rs")
        .with_line(5)
        .with_col(3)
        .build()
        .unwrap();

    let loc: Location = finding.location.unwrap();

    // Verify Location fields
    assert_eq!(loc.path.as_str(), "test.rs");
    assert_eq!(loc.line, Some(5));
    assert_eq!(loc.col, Some(3));
}

#[test]
fn norm_path_integration() {
    let finding = FindingBuilder::new()
        .with_code("CODE")
        .with_message("msg")
        .with_path("a/b/../c/./d.rs")
        .build()
        .unwrap();

    let loc = finding.location.unwrap();
    let _: NormPath = loc.path;

    // NormPath normalizes paths
    assert!(loc.path.as_str().contains("c"));
}
