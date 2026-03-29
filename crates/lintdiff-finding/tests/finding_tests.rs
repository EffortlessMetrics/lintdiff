//! Tests for lintdiff-finding crate.

use lintdiff_finding::{Finding, Findings, Severity};

// =============================================================================
// 1. Finding Creation and Builder Methods (12 tests)
// =============================================================================

#[test]
fn test_finding_new_basic() {
    let finding = Finding::new("src/lib.rs", "unused variable `x`");
    assert_eq!(finding.path(), "src/lib.rs");
    assert_eq!(finding.message(), "unused variable `x`");
}

#[test]
fn test_finding_new_with_string() {
    let path = String::from("src/main.rs");
    let msg = String::from("unused import");
    let finding = Finding::new(path.clone(), msg.clone());
    assert_eq!(finding.path(), path);
    assert_eq!(finding.message(), msg);
}

#[test]
fn test_finding_with_line() {
    let finding = Finding::new("test.rs", "error").with_line(42);
    assert_eq!(finding.line(), Some(42));
}

#[test]
fn test_finding_with_column() {
    let finding = Finding::new("test.rs", "error").with_column(10);
    assert_eq!(finding.column(), Some(10));
}

#[test]
fn test_finding_with_line_range() {
    let finding = Finding::new("test.rs", "error").with_line_range(10, 20);
    assert_eq!(finding.line(), Some(10));
    assert_eq!(finding.end_line(), Some(20));
}

#[test]
fn test_finding_with_column_range() {
    let finding = Finding::new("test.rs", "error").with_column_range(5, 15);
    assert_eq!(finding.column(), Some(5));
    assert_eq!(finding.end_column(), Some(15));
}

#[test]
fn test_finding_with_code() {
    let finding = Finding::new("test.rs", "error").with_code("unused_variables");
    assert_eq!(finding.code(), Some("unused_variables"));
}

#[test]
fn test_finding_with_severity() {
    let finding = Finding::new("test.rs", "error").with_severity(Severity::Error);
    assert_eq!(finding.severity(), Severity::Error);
}

#[test]
fn test_finding_with_source() {
    let finding = Finding::new("test.rs", "error").with_source("clippy");
    assert_eq!(finding.source(), Some("clippy"));
}

#[test]
fn test_finding_with_suggestion() {
    let finding = Finding::new("test.rs", "error").with_suggestion("Remove unused variable");
    assert_eq!(finding.suggestion(), Some("Remove unused variable"));
}

#[test]
fn test_finding_builder_chaining() {
    let finding = Finding::new("src/lib.rs", "unused variable `x`")
        .with_line(42)
        .with_column(10)
        .with_code("unused_variables")
        .with_severity(Severity::Warning)
        .with_source("rustc")
        .with_suggestion("Consider using `_x`");

    assert_eq!(finding.path(), "src/lib.rs");
    assert_eq!(finding.message(), "unused variable `x`");
    assert_eq!(finding.line(), Some(42));
    assert_eq!(finding.column(), Some(10));
    assert_eq!(finding.code(), Some("unused_variables"));
    assert_eq!(finding.severity(), Severity::Warning);
    assert_eq!(finding.source(), Some("rustc"));
    assert_eq!(finding.suggestion(), Some("Consider using `_x`"));
}

#[test]
fn test_finding_with_line_and_column_range_combined() {
    let finding = Finding::new("test.rs", "multi-line error")
        .with_line_range(10, 15)
        .with_column_range(1, 80);

    assert_eq!(finding.line(), Some(10));
    assert_eq!(finding.end_line(), Some(15));
    assert_eq!(finding.column(), Some(1));
    assert_eq!(finding.end_column(), Some(80));
}

// =============================================================================
// 2. Finding Accessor Methods (10 tests)
// =============================================================================

#[test]
fn test_finding_path_accessor() {
    let finding = Finding::new("path/to/file.rs", "msg");
    assert_eq!(finding.path(), "path/to/file.rs");
}

#[test]
fn test_finding_message_accessor() {
    let finding = Finding::new("test.rs", "this is a diagnostic message");
    assert_eq!(finding.message(), "this is a diagnostic message");
}

#[test]
fn test_finding_line_accessor_none() {
    let finding = Finding::new("test.rs", "msg");
    assert_eq!(finding.line(), None);
}

#[test]
fn test_finding_column_accessor_none() {
    let finding = Finding::new("test.rs", "msg");
    assert_eq!(finding.column(), None);
}

#[test]
fn test_finding_end_line_accessor() {
    let finding = Finding::new("test.rs", "msg").with_line_range(5, 10);
    assert_eq!(finding.end_line(), Some(10));
}

#[test]
fn test_finding_end_column_accessor() {
    let finding = Finding::new("test.rs", "msg").with_column_range(1, 20);
    assert_eq!(finding.end_column(), Some(20));
}

#[test]
fn test_finding_code_accessor_none() {
    let finding = Finding::new("test.rs", "msg");
    assert_eq!(finding.code(), None);
}

#[test]
fn test_finding_source_accessor_none() {
    let finding = Finding::new("test.rs", "msg");
    assert_eq!(finding.source(), None);
}

#[test]
fn test_finding_suggestion_accessor_none() {
    let finding = Finding::new("test.rs", "msg");
    assert_eq!(finding.suggestion(), None);
}

#[test]
fn test_finding_severity_default() {
    let finding = Finding::new("test.rs", "msg");
    assert_eq!(finding.severity(), Severity::Warning);
}

// =============================================================================
// 3. Finding Classification Methods (8 tests)
// =============================================================================

#[test]
fn test_finding_is_multiline_true() {
    let finding = Finding::new("test.rs", "msg").with_line_range(10, 15);
    assert!(finding.is_multiline());
}

#[test]
fn test_finding_is_multiline_false_same_lines() {
    let finding = Finding::new("test.rs", "msg").with_line_range(10, 10);
    assert!(!finding.is_multiline());
}

#[test]
fn test_finding_is_multiline_false_no_end_line() {
    let finding = Finding::new("test.rs", "msg").with_line(10);
    assert!(!finding.is_multiline());
}

#[test]
fn test_finding_has_span_true() {
    let finding = Finding::new("test.rs", "msg").with_line(42);
    assert!(finding.has_span());
}

#[test]
fn test_finding_has_span_false() {
    let finding = Finding::new("test.rs", "msg");
    assert!(!finding.has_span());
}

#[test]
fn test_finding_is_error_true_for_error() {
    let finding = Finding::new("test.rs", "msg").with_severity(Severity::Error);
    assert!(finding.is_error());
}

#[test]
fn test_finding_is_error_true_for_fatal() {
    let finding = Finding::new("test.rs", "msg").with_severity(Severity::Fatal);
    assert!(finding.is_error());
}

#[test]
fn test_finding_is_warning() {
    let finding = Finding::new("test.rs", "msg").with_severity(Severity::Warning);
    assert!(finding.is_warning());
    assert!(!finding.is_error());
}

// =============================================================================
// 4. Finding Display (5 tests)
// =============================================================================

#[test]
fn test_finding_display_path_only() {
    let finding = Finding::new("src/lib.rs", "error message");
    assert_eq!(format!("{}", finding), "src/lib.rs: error message");
}

#[test]
fn test_finding_display_with_line() {
    let finding = Finding::new("src/lib.rs", "error message").with_line(42);
    assert_eq!(format!("{}", finding), "src/lib.rs:42: error message");
}

#[test]
fn test_finding_display_with_line_and_column() {
    let finding = Finding::new("src/lib.rs", "error message")
        .with_line(42)
        .with_column(10);
    assert_eq!(format!("{}", finding), "src/lib.rs:42:10: error message");
}

#[test]
fn test_finding_display_with_code() {
    let finding = Finding::new("src/lib.rs", "error message").with_code("E0001");
    assert_eq!(format!("{}", finding), "src/lib.rs: error message [E0001]");
}

#[test]
fn test_finding_display_full() {
    let finding = Finding::new("src/lib.rs", "unused variable")
        .with_line(42)
        .with_column(10)
        .with_code("unused_variables");
    assert_eq!(
        format!("{}", finding),
        "src/lib.rs:42:10: unused variable [unused_variables]"
    );
}

// =============================================================================
// 5. Severity Parsing and Methods (8 tests)
// =============================================================================

#[test]
fn test_severity_parse_hint() {
    assert_eq!(Severity::parse("hint").unwrap(), Severity::Hint);
    assert_eq!(Severity::parse("HINT").unwrap(), Severity::Hint);
    assert_eq!(Severity::parse("Hint").unwrap(), Severity::Hint);
}

#[test]
fn test_severity_parse_info_aliases() {
    assert_eq!(Severity::parse("info").unwrap(), Severity::Hint);
    assert_eq!(Severity::parse("information").unwrap(), Severity::Hint);
}

#[test]
fn test_severity_parse_note() {
    assert_eq!(Severity::parse("note").unwrap(), Severity::Note);
    assert_eq!(Severity::parse("suggestion").unwrap(), Severity::Note);
}

#[test]
fn test_severity_parse_warning() {
    assert_eq!(Severity::parse("warning").unwrap(), Severity::Warning);
    assert_eq!(Severity::parse("warn").unwrap(), Severity::Warning);
    assert_eq!(Severity::parse("WARNING").unwrap(), Severity::Warning);
}

#[test]
fn test_severity_parse_error() {
    assert_eq!(Severity::parse("error").unwrap(), Severity::Error);
    assert_eq!(Severity::parse("err").unwrap(), Severity::Error);
    assert_eq!(Severity::parse("ERROR").unwrap(), Severity::Error);
}

#[test]
fn test_severity_parse_fatal() {
    assert_eq!(Severity::parse("fatal").unwrap(), Severity::Fatal);
    assert_eq!(Severity::parse("critical").unwrap(), Severity::Fatal);
}

#[test]
fn test_severity_parse_unknown() {
    let result = Severity::parse("unknown");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("unknown"));
}

#[test]
fn test_severity_as_str() {
    assert_eq!(Severity::Hint.as_str(), "hint");
    assert_eq!(Severity::Note.as_str(), "note");
    assert_eq!(Severity::Warning.as_str(), "warning");
    assert_eq!(Severity::Error.as_str(), "error");
    assert_eq!(Severity::Fatal.as_str(), "fatal");
}

#[test]
fn test_severity_display() {
    assert_eq!(format!("{}", Severity::Hint), "hint");
    assert_eq!(format!("{}", Severity::Note), "note");
    assert_eq!(format!("{}", Severity::Warning), "warning");
    assert_eq!(format!("{}", Severity::Error), "error");
    assert_eq!(format!("{}", Severity::Fatal), "fatal");
}

#[test]
fn test_severity_ordering() {
    assert!(Severity::Hint < Severity::Note);
    assert!(Severity::Note < Severity::Warning);
    assert!(Severity::Warning < Severity::Error);
    assert!(Severity::Error < Severity::Fatal);
}

#[test]
fn test_severity_is_problem() {
    assert!(!Severity::Hint.is_problem());
    assert!(!Severity::Note.is_problem());
    assert!(Severity::Warning.is_problem());
    assert!(Severity::Error.is_problem());
    assert!(Severity::Fatal.is_problem());
}

#[test]
fn test_severity_is_blocking() {
    assert!(!Severity::Hint.is_blocking());
    assert!(!Severity::Note.is_blocking());
    assert!(!Severity::Warning.is_blocking());
    assert!(Severity::Error.is_blocking());
    assert!(Severity::Fatal.is_blocking());
}

#[test]
fn test_severity_default() {
    assert_eq!(Severity::default(), Severity::Warning);
}

// =============================================================================
// 6. Findings Collection Methods (12 tests)
// =============================================================================

#[test]
fn test_findings_new() {
    let findings = Findings::new();
    assert!(findings.is_empty());
    assert_eq!(findings.len(), 0);
}

#[test]
fn test_findings_push() {
    let mut findings = Findings::new();
    findings.push(Finding::new("test.rs", "error1"));
    findings.push(Finding::new("test.rs", "error2"));
    assert_eq!(findings.len(), 2);
}

#[test]
fn test_findings_from_vec() {
    let vec = vec![
        Finding::new("test.rs", "error1"),
        Finding::new("test.rs", "error2"),
    ];
    let findings = Findings::from_vec(vec);
    assert_eq!(findings.len(), 2);
}

#[test]
fn test_findings_iter() {
    let mut findings = Findings::new();
    findings.push(Finding::new("a.rs", "error"));
    findings.push(Finding::new("b.rs", "error"));

    let paths: Vec<&str> = findings.iter().map(|f| f.path()).collect();
    assert_eq!(paths, vec!["a.rs", "b.rs"]);
}

#[test]
fn test_findings_filter_by_severity() {
    let mut findings = Findings::new();
    findings.push(Finding::new("test.rs", "warn1").with_severity(Severity::Warning));
    findings.push(Finding::new("test.rs", "error1").with_severity(Severity::Error));
    findings.push(Finding::new("test.rs", "warn2").with_severity(Severity::Warning));

    let warnings = findings.filter_by_severity(Severity::Warning);
    assert_eq!(warnings.len(), 2);

    let errors = findings.filter_by_severity(Severity::Error);
    assert_eq!(errors.len(), 1);
}

#[test]
fn test_findings_filter_by_path() {
    let mut findings = Findings::new();
    findings.push(Finding::new("src/lib.rs", "error"));
    findings.push(Finding::new("src/main.rs", "error"));
    findings.push(Finding::new("tests/test.rs", "error"));

    let src_findings = findings.filter_by_path("src/");
    assert_eq!(src_findings.len(), 2);

    let tests_findings = findings.filter_by_path("tests/");
    assert_eq!(tests_findings.len(), 1);
}

#[test]
fn test_findings_count_by_severity() {
    let mut findings = Findings::new();
    findings.push(Finding::new("test.rs", "warn1").with_severity(Severity::Warning));
    findings.push(Finding::new("test.rs", "warn2").with_severity(Severity::Warning));
    findings.push(Finding::new("test.rs", "error1").with_severity(Severity::Error));

    assert_eq!(findings.count_by_severity(Severity::Warning), 2);
    assert_eq!(findings.count_by_severity(Severity::Error), 1);
    assert_eq!(findings.count_by_severity(Severity::Hint), 0);
}

#[test]
fn test_findings_error_count() {
    let mut findings = Findings::new();
    findings.push(Finding::new("test.rs", "warn").with_severity(Severity::Warning));
    findings.push(Finding::new("test.rs", "error1").with_severity(Severity::Error));
    findings.push(Finding::new("test.rs", "fatal").with_severity(Severity::Fatal));

    assert_eq!(findings.error_count(), 2);
}

#[test]
fn test_findings_warning_count() {
    let mut findings = Findings::new();
    findings.push(Finding::new("test.rs", "warn1").with_severity(Severity::Warning));
    findings.push(Finding::new("test.rs", "warn2").with_severity(Severity::Warning));
    findings.push(Finding::new("test.rs", "error").with_severity(Severity::Error));

    assert_eq!(findings.warning_count(), 2);
}

#[test]
fn test_findings_from_iterator() {
    let findings: Findings = vec![
        Finding::new("a.rs", "error1"),
        Finding::new("b.rs", "error2"),
    ]
    .into_iter()
    .collect();

    assert_eq!(findings.len(), 2);
}

#[test]
fn test_findings_is_empty() {
    let findings = Findings::new();
    assert!(findings.is_empty());

    let mut findings_with_items = Findings::new();
    findings_with_items.push(Finding::new("test.rs", "error"));
    assert!(!findings_with_items.is_empty());
}

#[test]
fn test_findings_clone() {
    let mut findings = Findings::new();
    findings.push(Finding::new("test.rs", "error"));

    let cloned = findings.clone();
    assert_eq!(cloned.len(), 1);
    assert_eq!(cloned.iter().next().unwrap().path(), "test.rs");
}

// =============================================================================
// Additional Tests for Complete Coverage
// =============================================================================

#[test]
fn test_finding_location_path_only() {
    let finding = Finding::new("src/lib.rs", "msg");
    assert_eq!(finding.location(), "src/lib.rs");
}

#[test]
fn test_finding_location_with_line() {
    let finding = Finding::new("src/lib.rs", "msg").with_line(42);
    assert_eq!(finding.location(), "src/lib.rs:42");
}

#[test]
fn test_finding_location_with_line_and_column() {
    let finding = Finding::new("src/lib.rs", "msg")
        .with_line(42)
        .with_column(10);
    assert_eq!(finding.location(), "src/lib.rs:42:10");
}

#[test]
fn test_finding_clone() {
    let finding = Finding::new("test.rs", "msg")
        .with_line(10)
        .with_code("E001");

    let cloned = finding.clone();
    assert_eq!(cloned.path(), "test.rs");
    assert_eq!(cloned.message(), "msg");
    assert_eq!(cloned.line(), Some(10));
    assert_eq!(cloned.code(), Some("E001"));
}

#[test]
fn test_finding_equality() {
    let f1 = Finding::new("test.rs", "msg").with_line(10);
    let f2 = Finding::new("test.rs", "msg").with_line(10);
    let f3 = Finding::new("test.rs", "msg").with_line(20);

    assert_eq!(f1, f2);
    assert_ne!(f1, f3);
}

#[test]
fn test_severity_equality() {
    assert_eq!(Severity::Warning, Severity::Warning);
    assert_ne!(Severity::Warning, Severity::Error);
}

#[test]
fn test_severity_clone() {
    let severity = Severity::Error;
    let cloned = severity.clone();
    assert_eq!(severity, cloned);
}

#[test]
fn test_severity_copy() {
    let severity = Severity::Warning;
    let copied: Severity = severity;
    assert_eq!(severity, copied);
}

#[test]
fn test_severity_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Severity::Warning);
    set.insert(Severity::Error);
    set.insert(Severity::Warning);

    assert_eq!(set.len(), 2);
}

#[test]
fn test_severity_repr_u8() {
    assert_eq!(Severity::Hint as u8, 0);
    assert_eq!(Severity::Note as u8, 1);
    assert_eq!(Severity::Warning as u8, 2);
    assert_eq!(Severity::Error as u8, 3);
    assert_eq!(Severity::Fatal as u8, 4);
}

#[test]
fn test_severity_parse_error_unknown() {
    let err = Severity::parse("invalid").unwrap_err();
    assert!(err.to_string().contains("invalid"));
}

#[test]
fn test_findings_default() {
    let findings = Findings::default();
    assert!(findings.is_empty());
}

#[test]
fn test_finding_is_error_false_for_warning() {
    let finding = Finding::new("test.rs", "msg").with_severity(Severity::Warning);
    assert!(!finding.is_error());
}

#[test]
fn test_finding_is_warning_false_for_error() {
    let finding = Finding::new("test.rs", "msg").with_severity(Severity::Error);
    assert!(!finding.is_warning());
}

#[test]
fn test_finding_is_warning_false_for_hint() {
    let finding = Finding::new("test.rs", "msg").with_severity(Severity::Hint);
    assert!(!finding.is_warning());
}
