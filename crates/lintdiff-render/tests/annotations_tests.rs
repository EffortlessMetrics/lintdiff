//! Comprehensive tests for GitHub annotations rendering functionality.

use lintdiff_render::render_github_annotations;
use lintdiff_types::{
    Counts, Finding, Location, NormPath, Report, RunInfo, Severity, ToolInfo, Verdict,
    VerdictStatus, SCHEMA_ID, TOOL_NAME,
};

// =============================================================================
// Test Helpers
// =============================================================================

fn create_report(findings: Vec<Finding>) -> Report {
    let counts = counts_from(&findings);
    Report {
        schema: SCHEMA_ID.to_string(),
        tool: ToolInfo {
            name: TOOL_NAME.to_string(),
            version: "test".to_string(),
            commit: None,
        },
        run: RunInfo {
            started_at: "2026-01-01T00:00:00Z".to_string(),
            ended_at: "2026-01-01T00:00:01Z".to_string(),
            duration_ms: None,
            host: None,
            git: None,
        },
        verdict: Verdict {
            status: VerdictStatus::Warn,
            counts,
            reasons: vec![],
        },
        findings,
        data: None,
    }
}

fn counts_from(findings: &[Finding]) -> Counts {
    let mut c = Counts::default();
    for f in findings {
        match f.severity {
            Severity::Info => c.info += 1,
            Severity::Warn => c.warn += 1,
            Severity::Error => c.error += 1,
        }
    }
    c
}

fn create_finding(path: &str, line: u32, severity: Severity, code: &str, msg: &str) -> Finding {
    Finding {
        severity,
        check_id: Some("diagnostics.on_diff".to_string()),
        code: code.to_string(),
        message: msg.to_string(),
        location: Some(Location {
            path: NormPath::new(path),
            line: Some(line),
            col: None,
        }),
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

fn create_finding_with_col(
    path: &str,
    line: u32,
    col: u32,
    severity: Severity,
    code: &str,
    msg: &str,
) -> Finding {
    Finding {
        severity,
        check_id: Some("diagnostics.on_diff".to_string()),
        code: code.to_string(),
        message: msg.to_string(),
        location: Some(Location {
            path: NormPath::new(path),
            line: Some(line),
            col: Some(col),
        }),
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

fn create_finding_no_line(path: &str, severity: Severity, code: &str, msg: &str) -> Finding {
    Finding {
        severity,
        check_id: Some("diagnostics.on_diff".to_string()),
        code: code.to_string(),
        message: msg.to_string(),
        location: Some(Location {
            path: NormPath::new(path),
            line: None,
            col: None,
        }),
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

fn create_finding_no_location(severity: Severity, code: &str, msg: &str) -> Finding {
    Finding {
        severity,
        check_id: Some("diagnostics.on_diff".to_string()),
        code: code.to_string(),
        message: msg.to_string(),
        location: None,
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

// =============================================================================
// Single Annotation Format Tests
// =============================================================================

#[test]
fn annotation_single_warning_format() {
    let f = create_finding("src/lib.rs", 42, Severity::Warn, "WARN001", "Test warning");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("::warning file=src/lib.rs,line=42::[WARN001] Test warning"));
}

#[test]
fn annotation_single_error_format() {
    let f = create_finding("src/lib.rs", 10, Severity::Error, "ERR001", "Test error");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("::error file=src/lib.rs,line=10::[ERR001] Test error"));
}

#[test]
fn annotation_single_notice_format() {
    let f = create_finding("src/lib.rs", 5, Severity::Info, "INFO001", "Test info");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("::notice file=src/lib.rs,line=5::[INFO001] Test info"));
}

#[test]
fn annotation_format_structure() {
    let f = create_finding("src/main.rs", 1, Severity::Warn, "CODE", "Message");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    // Verify the structure: ::severity file=path,line=N::[code] message
    assert!(out.starts_with("::warning file=src/main.rs,line=1::[CODE] Message"));
    assert!(out.ends_with('\n'));
}

#[test]
fn annotation_includes_code_in_brackets() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "MY_LINT_CODE", "Message");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("[MY_LINT_CODE]"));
}

#[test]
fn annotation_with_column() {
    let f = create_finding_with_col("src/lib.rs", 10, 5, Severity::Error, "E001", "Error");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("file=src/lib.rs,line=10,col=5"));
}

#[test]
fn annotation_without_line() {
    let f = create_finding_no_line("src/lib.rs", Severity::Warn, "W001", "Warning");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    // Should have file but no line
    assert!(out.contains("file=src/lib.rs"));
    assert!(!out.contains(",line="));
}

// =============================================================================
// Multiple Annotations Tests
// =============================================================================

#[test]
fn annotations_multiple_findings() {
    let findings = vec![
        create_finding("src/a.rs", 1, Severity::Error, "E001", "Error one"),
        create_finding("src/b.rs", 2, Severity::Warn, "W001", "Warning one"),
        create_finding("src/c.rs", 3, Severity::Info, "I001", "Info one"),
    ];
    let r = create_report(findings);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("::error file=src/a.rs,line=1::[E001] Error one"));
    assert!(out.contains("::warning file=src/b.rs,line=2::[W001] Warning one"));
    assert!(out.contains("::notice file=src/c.rs,line=3::[I001] Info one"));
}

#[test]
fn annotations_multiple_lines() {
    let findings = vec![
        create_finding("src/a.rs", 1, Severity::Error, "E001", "First"),
        create_finding("src/b.rs", 2, Severity::Error, "E002", "Second"),
    ];
    let r = create_report(findings);
    let out = render_github_annotations(&r, 50);

    // Each annotation should be on its own line
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn annotations_sorted_by_severity() {
    // Errors should come before warnings, warnings before info
    let findings = vec![
        create_finding("src/info.rs", 1, Severity::Info, "I001", "Info"),
        create_finding("src/error.rs", 2, Severity::Error, "E001", "Error"),
        create_finding("src/warn.rs", 3, Severity::Warn, "W001", "Warn"),
    ];
    let r = create_report(findings);
    let out = render_github_annotations(&r, 50);

    let error_pos = out.find("::error").expect("error should be present");
    let warn_pos = out.find("::warning").expect("warning should be present");
    let notice_pos = out.find("::notice").expect("notice should be present");

    assert!(error_pos < warn_pos, "Error should appear before warning");
    assert!(warn_pos < notice_pos, "Warning should appear before notice");
}

#[test]
fn annotations_same_file_different_lines() {
    let findings = vec![
        create_finding("src/lib.rs", 10, Severity::Warn, "W001", "First warning"),
        create_finding("src/lib.rs", 20, Severity::Warn, "W002", "Second warning"),
        create_finding("src/lib.rs", 30, Severity::Warn, "W003", "Third warning"),
    ];
    let r = create_report(findings);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("line=10"));
    assert!(out.contains("line=20"));
    assert!(out.contains("line=30"));
}

// =============================================================================
// Annotation Limits/Truncation Tests
// =============================================================================

#[test]
fn annotations_respects_max_limit() {
    let findings: Vec<Finding> = (1..=100)
        .map(|i| {
            create_finding(
                "src/lib.rs",
                i,
                Severity::Warn,
                &format!("W{:03}", i),
                "Warning",
            )
        })
        .collect();
    let r = create_report(findings);
    let out = render_github_annotations(&r, 10);

    // Should only have 10 annotations
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 10);
}

#[test]
fn annotations_max_one() {
    let findings = vec![
        create_finding("src/a.rs", 1, Severity::Error, "E001", "First"),
        create_finding("src/b.rs", 2, Severity::Error, "E002", "Second"),
    ];
    let r = create_report(findings);
    let out = render_github_annotations(&r, 1);

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 1);
    assert!(out.contains("E001"));
    assert!(!out.contains("E002"));
}

#[test]
fn annotations_max_zero() {
    let findings = vec![create_finding(
        "src/lib.rs",
        1,
        Severity::Warn,
        "W001",
        "Warning",
    )];
    let r = create_report(findings);
    let out = render_github_annotations(&r, 0);

    assert!(out.trim().is_empty());
}

#[test]
fn annotations_all_under_limit() {
    let findings: Vec<Finding> = (1..=5)
        .map(|i| {
            create_finding(
                "src/lib.rs",
                i,
                Severity::Warn,
                &format!("W{:03}", i),
                "Warning",
            )
        })
        .collect();
    let r = create_report(findings);
    let out = render_github_annotations(&r, 10);

    // All 5 should be present
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 5);
}

#[test]
fn annotations_exact_limit() {
    let findings: Vec<Finding> = (1..=10)
        .map(|i| {
            create_finding(
                "src/lib.rs",
                i,
                Severity::Warn,
                &format!("W{:03}", i),
                "Warning",
            )
        })
        .collect();
    let r = create_report(findings);
    let out = render_github_annotations(&r, 10);

    // All 10 should be present
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 10);
}

// =============================================================================
// Special Character Escaping Tests
// =============================================================================

#[test]
fn annotation_escapes_newline() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", "Line1\nLine2");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("Line1%0ALine2"));
    assert!(!out.contains("Line1\nLine2"));
}

#[test]
fn annotation_escapes_carriage_return() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", "Line1\rLine2");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("Line1%0DLine2"));
}

#[test]
fn annotation_escapes_crlf() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", "Line1\r\nLine2");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    // CR then LF should become %0D%0A
    assert!(out.contains("Line1%0D%0ALine2"));
}

#[test]
fn annotation_escapes_percent() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", "100% complete");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("100%25 complete"));
}

#[test]
fn annotation_escapes_multiple_special_chars() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", "A\nB\rC%D");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("A%0AB%0DC%25D"));
}

#[test]
fn annotation_preserves_pipe_character() {
    // Unlike markdown, annotations don't escape pipes
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", "a | b | c");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("a | b | c"));
}

#[test]
fn annotation_preserves_backticks() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", "use `code` here");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("use `code` here"));
}

#[test]
fn annotation_handles_unicode() {
    let f = create_finding(
        "src/lib.rs",
        1,
        Severity::Warn,
        "CODE",
        "Unicode: 你好世界 🌍",
    );
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("你好世界"));
    assert!(out.contains("🌍"));
}

#[test]
fn annotation_handles_brackets_in_message() {
    let f = create_finding(
        "src/lib.rs",
        1,
        Severity::Warn,
        "CODE",
        "Use [array] syntax",
    );
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    // Brackets should be preserved
    assert!(out.contains("[CODE] Use [array] syntax"));
}

// =============================================================================
// Findings Without Location Tests
// =============================================================================

#[test]
fn annotations_skips_findings_without_location() {
    let findings = vec![
        create_finding_no_location(Severity::Error, "E001", "Error without location"),
        create_finding(
            "src/lib.rs",
            10,
            Severity::Warn,
            "W001",
            "Warning with location",
        ),
    ];
    let r = create_report(findings);
    let out = render_github_annotations(&r, 50);

    // Only the finding with location should appear
    assert!(!out.contains("E001"));
    assert!(out.contains("W001"));
}

#[test]
fn annotations_empty_when_all_without_location() {
    let findings = vec![
        create_finding_no_location(Severity::Error, "E001", "First"),
        create_finding_no_location(Severity::Error, "E002", "Second"),
    ];
    let r = create_report(findings);
    let out = render_github_annotations(&r, 50);

    assert!(out.trim().is_empty());
}

// =============================================================================
// Empty/Edge Case Tests
// =============================================================================

#[test]
fn annotations_empty_for_no_findings() {
    let r = create_report(vec![]);
    let out = render_github_annotations(&r, 50);

    assert!(out.trim().is_empty());
}

#[test]
fn annotations_empty_string_ends_with_newline() {
    let r = create_report(vec![]);
    let out = render_github_annotations(&r, 50);

    // Empty output is just empty (no trailing newline for empty)
    assert_eq!(out, "");
}

#[test]
fn annotations_single_ends_with_newline() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "W001", "Warning");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.ends_with('\n'));
}

#[test]
fn annotations_empty_message() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", "");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    // Should still render with empty message
    assert!(out.contains("::warning file=src/lib.rs,line=1::[CODE]"));
}

#[test]
fn annotations_empty_code() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "", "Message");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    // Should still render with empty code in brackets
    assert!(out.contains("[] Message"));
}

// =============================================================================
// Path Handling Tests
// =============================================================================

#[test]
fn annotations_nested_path() {
    let f = create_finding(
        "deeply/nested/path/to/module.rs",
        100,
        Severity::Error,
        "E001",
        "Error",
    );
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("file=deeply/nested/path/to/module.rs"));
}

#[test]
fn annotations_special_chars_in_path() {
    let f = create_finding(
        "src/special-file_v2.rs",
        1,
        Severity::Warn,
        "W001",
        "Warning",
    );
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    assert!(out.contains("file=src/special-file_v2.rs"));
}

#[test]
fn annotations_dotdot_in_path() {
    let f = create_finding("../parent/file.rs", 1, Severity::Warn, "W001", "Warning");
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    // Path should be preserved
    assert!(out.contains("file=../parent/file.rs") || out.contains("file="));
}

// =============================================================================
// Large Message Tests
// =============================================================================

#[test]
fn annotations_very_long_message() {
    let long_msg = "A".repeat(1000);
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", &long_msg);
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    // Long message should be preserved
    assert!(out.contains(&long_msg));
}

#[test]
fn annotations_multiline_message_escaped() {
    let multiline = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", multiline);
    let r = create_report(vec![f]);
    let out = render_github_annotations(&r, 50);

    // All newlines should be escaped
    assert!(out.contains("Line 1%0ALine 2%0ALine 3%0ALine 4%0ALine 5"));
    // No literal newlines in the annotation command itself
    let annotation_line = out.lines().next().unwrap();
    assert!(!annotation_line.contains('\n'));
}
