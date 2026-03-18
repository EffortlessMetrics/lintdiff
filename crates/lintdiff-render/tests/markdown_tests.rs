//! Comprehensive tests for markdown rendering functionality.

use lintdiff_render::{render_markdown, MarkdownOptions, DEFAULT_REPORT_PATH};
use lintdiff_types::{
    Counts, Finding, Location, NormPath, Report, RunInfo, Severity, ToolInfo, Verdict,
    VerdictStatus, SCHEMA_ID, TOOL_NAME,
};
use serde_json::json;

// =============================================================================
// Test Helpers
// =============================================================================

fn create_report(status: VerdictStatus, findings: Vec<Finding>) -> Report {
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
            status,
            counts,
            reasons: vec![],
        },
        findings,
        data: None,
    }
}

fn create_report_with_data(
    status: VerdictStatus,
    findings: Vec<Finding>,
    data: serde_json::Value,
) -> Report {
    let mut report = create_report(status, findings);
    report.data = Some(data);
    report
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

#[allow(dead_code)]
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

// =============================================================================
// Basic Report to Markdown Tests
// =============================================================================

#[test]
fn markdown_empty_report_shows_pass_status() {
    let r = create_report(VerdictStatus::Pass, vec![]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("### lintdiff"));
    assert!(md.contains("**Status:** `PASS`"));
    assert!(md.contains("No diagnostics matched"));
}

#[test]
fn markdown_header_always_present() {
    let r = create_report(VerdictStatus::Pass, vec![]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.starts_with("### lintdiff\n\n"));
}

#[test]
fn markdown_includes_counts_section() {
    let r = create_report(VerdictStatus::Pass, vec![]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("**Counts:** error 0 · warn 0 · info 0"));
}

#[test]
fn markdown_includes_report_path() {
    let r = create_report(VerdictStatus::Pass, vec![]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains(DEFAULT_REPORT_PATH));
}

#[test]
fn markdown_custom_report_path() {
    let r = create_report(VerdictStatus::Pass, vec![]);
    let opts = MarkdownOptions {
        max_items: 20,
        report_path: "custom/path/report.json".to_string(),
    };
    let md = render_markdown(&r, opts);

    assert!(md.contains("custom/path/report.json"));
}

// =============================================================================
// Multiple Findings Tests
// =============================================================================

#[test]
fn markdown_single_finding_renders_table() {
    let f = create_finding("src/lib.rs", 10, Severity::Warn, "WARN001", "Test warning");
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("| Sev | Location | Code | Message |"));
    assert!(md.contains("| --- | --- | --- | --- |"));
    assert!(md.contains("src/lib.rs:10"));
    assert!(md.contains("WARN001"));
    assert!(md.contains("Test warning"));
}

#[test]
fn markdown_multiple_findings_renders_all() {
    let findings = vec![
        create_finding("src/a.rs", 1, Severity::Error, "E001", "Error one"),
        create_finding("src/b.rs", 2, Severity::Warn, "W001", "Warning one"),
        create_finding("src/c.rs", 3, Severity::Info, "I001", "Info one"),
    ];
    let r = create_report(VerdictStatus::Fail, findings);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("src/a.rs:1"));
    assert!(md.contains("src/b.rs:2"));
    assert!(md.contains("src/c.rs:3"));
    assert!(md.contains("E001"));
    assert!(md.contains("W001"));
    assert!(md.contains("I001"));
}

#[test]
fn markdown_many_findings_sorted_by_severity() {
    // Errors should appear before warnings, warnings before info
    let findings = vec![
        create_finding("src/info.rs", 1, Severity::Info, "I001", "Info"),
        create_finding("src/error.rs", 2, Severity::Error, "E001", "Error"),
        create_finding("src/warn.rs", 3, Severity::Warn, "W001", "Warn"),
    ];
    let r = create_report(VerdictStatus::Fail, findings);
    let md = render_markdown(&r, MarkdownOptions::default());

    // Find positions
    let error_pos = md.find("E001").expect("E001 should be present");
    let warn_pos = md.find("W001").expect("W001 should be present");
    let info_pos = md.find("I001").expect("I001 should be present");

    // Error should come before warn, warn before info
    assert!(error_pos < warn_pos, "Error should appear before warning");
    assert!(warn_pos < info_pos, "Warning should appear before info");
}

// =============================================================================
// Severity Levels Tests
// =============================================================================

#[test]
fn markdown_error_severity_badge() {
    let f = create_finding("src/lib.rs", 1, Severity::Error, "E001", "Error message");
    let r = create_report(VerdictStatus::Fail, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("| error |"));
}

#[test]
fn markdown_warn_severity_badge() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "W001", "Warning message");
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("| warn |"));
}

#[test]
fn markdown_info_severity_badge() {
    let f = create_finding("src/lib.rs", 1, Severity::Info, "I001", "Info message");
    let r = create_report(VerdictStatus::Pass, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("| info |"));
}

#[test]
fn markdown_counts_reflect_severities() {
    let findings = vec![
        create_finding("src/a.rs", 1, Severity::Error, "E001", "Error"),
        create_finding("src/b.rs", 2, Severity::Error, "E002", "Error"),
        create_finding("src/c.rs", 3, Severity::Warn, "W001", "Warn"),
        create_finding("src/d.rs", 4, Severity::Info, "I001", "Info"),
        create_finding("src/e.rs", 5, Severity::Info, "I002", "Info"),
        create_finding("src/f.rs", 6, Severity::Info, "I003", "Info"),
    ];
    let r = create_report(VerdictStatus::Fail, findings);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("error 2 · warn 1 · info 3"));
}

// =============================================================================
// Code Formatting Tests
// =============================================================================

#[test]
fn markdown_code_is_backtick_formatted() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "MY_LINT_CODE", "Message");
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("`MY_LINT_CODE`"));
}

#[test]
fn markdown_location_is_backtick_formatted() {
    let f = create_finding("src/lib.rs", 42, Severity::Warn, "CODE", "Message");
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("`src/lib.rs:42`"));
}

#[test]
fn markdown_location_without_line() {
    let f = create_finding_no_line("src/lib.rs", Severity::Warn, "CODE", "Message");
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("`src/lib.rs`"));
    assert!(!md.contains("`src/lib.rs:`"));
}

#[test]
fn markdown_finding_without_location() {
    let f = create_finding_no_location(Severity::Warn, "CODE", "Message");
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("`-`"));
}

// =============================================================================
// Links to Files Tests
// =============================================================================

#[test]
fn markdown_path_preserved_in_location() {
    let f = create_finding(
        "deeply/nested/path/to/file.rs",
        100,
        Severity::Error,
        "E001",
        "Error",
    );
    let r = create_report(VerdictStatus::Fail, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("deeply/nested/path/to/file.rs:100"));
}

#[test]
fn markdown_windows_style_paths() {
    let f = create_finding(
        "src\\windows\\path.rs",
        10,
        Severity::Warn,
        "W001",
        "Warning",
    );
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    // Path should be preserved as-is (NormPath normalizes internally)
    assert!(md.contains("W001"));
}

#[test]
fn markdown_special_characters_in_path() {
    let f = create_finding("src/special-file_v2.rs", 1, Severity::Info, "I001", "Info");
    let r = create_report(VerdictStatus::Pass, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("special-file_v2.rs"));
}

// =============================================================================
// Escape Table Tests
// =============================================================================

#[test]
fn markdown_escapes_pipe_character() {
    let f = create_finding(
        "src/lib.rs",
        1,
        Severity::Warn,
        "CODE",
        "Message with | pipe",
    );
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("Message with \\| pipe"));
    assert!(!md.contains("Message with | pipe"));
}

#[test]
fn markdown_escapes_multiple_pipes() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", "a | b | c");
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("a \\| b \\| c"));
}

#[test]
fn markdown_converts_newlines_to_spaces() {
    let f = create_finding(
        "src/lib.rs",
        1,
        Severity::Warn,
        "CODE",
        "Line 1\nLine 2\nLine 3",
    );
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("Line 1 Line 2 Line 3"));
    // Should not contain literal newlines in the table cell
    let table_section = md.split("| Sev |").nth(1).unwrap_or("");
    // Within table rows, there should be no unescaped newlines in message cell
    assert!(!table_section.contains("Line 1\n"));
}

#[test]
fn markdown_handles_complex_message() {
    let f = create_finding(
        "src/lib.rs",
        1,
        Severity::Error,
        "E001",
        "Error: expected `|` but found `||`\nTry using | instead",
    );
    let r = create_report(VerdictStatus::Fail, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("\\|"));
    assert!(md.contains("Try using"));
}

// =============================================================================
// Verdict Status Tests
// =============================================================================

#[test]
fn markdown_pass_status() {
    let r = create_report(VerdictStatus::Pass, vec![]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("**Status:** `PASS`"));
}

#[test]
fn markdown_warn_status() {
    let r = create_report(VerdictStatus::Warn, vec![]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("**Status:** `WARN`"));
}

#[test]
fn markdown_fail_status() {
    let r = create_report(VerdictStatus::Fail, vec![]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("**Status:** `FAIL`"));
}

#[test]
fn markdown_skip_status() {
    let mut r = create_report(VerdictStatus::Skip, vec![]);
    r.verdict.reasons = vec![
        "No diff file found".to_string(),
        "No diagnostics provided".to_string(),
    ];
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("**Status:** `SKIP`"));
    assert!(md.contains("lintdiff skipped"));
    assert!(md.contains("No diff file found"));
    assert!(md.contains("No diagnostics provided"));
}

#[test]
fn markdown_skip_no_table() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "W001", "Should not appear");
    let mut r = create_report(VerdictStatus::Skip, vec![f]);
    r.verdict.reasons = vec!["Skipped".to_string()];
    let md = render_markdown(&r, MarkdownOptions::default());

    // Skip status should not render the findings table
    assert!(!md.contains("| Sev | Location | Code | Message |"));
    assert!(md.contains("lintdiff skipped"));
}

// =============================================================================
// Truncation and Budget Tests
// =============================================================================

#[test]
fn markdown_truncates_at_max_items() {
    let findings: Vec<Finding> = (1..=25)
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
    let r = create_report(VerdictStatus::Warn, findings);
    let opts = MarkdownOptions {
        max_items: 10,
        report_path: DEFAULT_REPORT_PATH.to_string(),
    };
    let md = render_markdown(&r, opts);

    assert!(md.contains("And 15 more"));
}

#[test]
fn markdown_shows_all_when_under_limit() {
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
    let r = create_report(VerdictStatus::Warn, findings);
    let opts = MarkdownOptions {
        max_items: 10,
        report_path: DEFAULT_REPORT_PATH.to_string(),
    };
    let md = render_markdown(&r, opts);

    assert!(!md.contains("more"));
    assert!(md.contains("Full receipt:"));
}

#[test]
fn markdown_exact_max_items_no_truncation() {
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
    let r = create_report(VerdictStatus::Warn, findings);
    let opts = MarkdownOptions {
        max_items: 10,
        report_path: DEFAULT_REPORT_PATH.to_string(),
    };
    let md = render_markdown(&r, opts);

    // Exactly 10 items should not show truncation
    assert!(!md.contains("And") || !md.contains("more"));
}

#[test]
fn markdown_truncated_data_flag() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "W001", "Warning");
    let data = json!({
        "truncated": true
    });
    let r = create_report_with_data(VerdictStatus::Warn, vec![f], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("Output truncated"));
}

#[test]
fn markdown_not_truncated_data_flag() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "W001", "Warning");
    let data = json!({
        "truncated": false
    });
    let r = create_report_with_data(VerdictStatus::Warn, vec![f], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(!md.contains("Output truncated"));
}

// =============================================================================
// Explain Summary Tests
// =============================================================================

#[test]
fn markdown_explain_summary_basic() {
    let data = json!({
        "explain_summary": {
            "total": 100,
            "included": 50
        }
    });
    let r = create_report_with_data(VerdictStatus::Warn, vec![], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("**Diagnostics:** 100 total: 50 matched"));
}

#[test]
fn markdown_explain_summary_with_outside_diff() {
    let data = json!({
        "explain_summary": {
            "total": 100,
            "included": 50,
            "dropped_outside_diff": 30
        }
    });
    let r = create_report_with_data(VerdictStatus::Warn, vec![], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("30 outside diff"));
}

#[test]
fn markdown_explain_summary_with_no_span() {
    let data = json!({
        "explain_summary": {
            "total": 100,
            "included": 50,
            "dropped_no_span": 10
        }
    });
    let r = create_report_with_data(VerdictStatus::Warn, vec![], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("10 no span"));
}

#[test]
fn markdown_explain_summary_with_path_filter() {
    let data = json!({
        "explain_summary": {
            "total": 100,
            "included": 50,
            "dropped_by_path_filter": 5
        }
    });
    let r = create_report_with_data(VerdictStatus::Warn, vec![], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("5 filtered by path"));
}

#[test]
fn markdown_explain_summary_with_suppressed() {
    let data = json!({
        "explain_summary": {
            "total": 100,
            "included": 50,
            "suppressed_by_code": 15
        }
    });
    let r = create_report_with_data(VerdictStatus::Warn, vec![], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("15 suppressed"));
}

#[test]
fn markdown_explain_summary_all_fields() {
    let data = json!({
        "explain_summary": {
            "total": 100,
            "included": 40,
            "dropped_outside_diff": 30,
            "dropped_no_span": 10,
            "dropped_by_path_filter": 5,
            "suppressed_by_code": 15
        }
    });
    let r = create_report_with_data(VerdictStatus::Warn, vec![], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("100 total: 40 matched"));
    assert!(md.contains("30 outside diff"));
    assert!(md.contains("10 no span"));
    assert!(md.contains("5 filtered by path"));
    assert!(md.contains("15 suppressed"));
}

#[test]
fn markdown_explain_summary_zero_total_not_shown() {
    let data = json!({
        "explain_summary": {
            "total": 0,
            "included": 0
        }
    });
    let r = create_report_with_data(VerdictStatus::Pass, vec![], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    // When total is 0, the diagnostics summary line should not appear
    assert!(!md.contains("**Diagnostics:**"));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn markdown_empty_code_field() {
    let mut f = create_finding("src/lib.rs", 1, Severity::Warn, "", "Message");
    f.code = "".to_string();
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    // Empty code should still render backticks
    assert!(md.contains("`` |"));
}

#[test]
fn markdown_empty_message_field() {
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", "");
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    // Empty message should still render the table row
    assert!(md.contains("| warn |"));
}

#[test]
fn markdown_unicode_in_message() {
    let f = create_finding(
        "src/lib.rs",
        1,
        Severity::Warn,
        "CODE",
        "Unicode: 你好世界 🌍",
    );
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("你好世界"));
    assert!(md.contains("🌍"));
}

#[test]
fn markdown_very_long_message() {
    let long_msg = "A".repeat(500);
    let f = create_finding("src/lib.rs", 1, Severity::Warn, "CODE", &long_msg);
    let r = create_report(VerdictStatus::Warn, vec![f]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains(&long_msg));
}

#[test]
fn markdown_zero_max_items() {
    let findings = vec![create_finding(
        "src/lib.rs",
        1,
        Severity::Warn,
        "W001",
        "Warning",
    )];
    let r = create_report(VerdictStatus::Warn, findings);
    let opts = MarkdownOptions {
        max_items: 0,
        report_path: DEFAULT_REPORT_PATH.to_string(),
    };
    let md = render_markdown(&r, opts);

    // Should show "And 1 more" since 0 items are displayed
    assert!(md.contains("And 1 more"));
}

// =============================================================================
// Default Options Tests
// =============================================================================

#[test]
fn markdown_options_default_max_items() {
    let opts = MarkdownOptions::default();
    assert_eq!(opts.max_items, 20);
}

#[test]
fn markdown_options_default_report_path() {
    let opts = MarkdownOptions::default();
    assert_eq!(opts.report_path, DEFAULT_REPORT_PATH);
}

#[test]
fn default_report_path_constant() {
    assert_eq!(DEFAULT_REPORT_PATH, "artifacts/lintdiff/report.json");
}
