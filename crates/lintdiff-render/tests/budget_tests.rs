//! Comprehensive tests for output budgeting and truncation functionality.

use lintdiff_render::{
    render_github_annotations, render_markdown, MarkdownOptions, DEFAULT_REPORT_PATH,
};
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
fn create_finding_with_location(path: &str, line: u32) -> Finding {
    Finding {
        severity: Severity::Warn,
        check_id: Some("diagnostics.on_diff".to_string()),
        code: "TEST_CODE".to_string(),
        message: "Test message".to_string(),
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

// =============================================================================
// Markdown Max Findings Limit Tests
// =============================================================================

#[test]
fn markdown_max_items_limits_output() {
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
    let r = create_report(VerdictStatus::Warn, findings);
    let opts = MarkdownOptions {
        max_items: 10,
        report_path: DEFAULT_REPORT_PATH.to_string(),
    };
    let md = render_markdown(&r, opts);

    // Count table rows (each finding is one row)
    let table_rows = md.matches("| warn |").count();
    assert_eq!(table_rows, 10);
}

#[test]
fn markdown_max_items_default_is_20() {
    let findings: Vec<Finding> = (1..=50)
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
    let md = render_markdown(&r, MarkdownOptions::default());

    // Default max is 20
    let table_rows = md.matches("| warn |").count();
    assert_eq!(table_rows, 20);
}

#[test]
fn markdown_max_items_shows_truncation_message() {
    let findings: Vec<Finding> = (1..=30)
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

    assert!(md.contains("And 20 more"));
}

#[test]
fn markdown_max_items_exact_no_truncation() {
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

    // No truncation message when exactly at limit
    assert!(!md.contains("And") || !md.contains("more"));
    assert!(md.contains("Full receipt:"));
}

#[test]
fn markdown_max_items_under_limit() {
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

    // All 5 should appear
    let table_rows = md.matches("| warn |").count();
    assert_eq!(table_rows, 5);
    assert!(!md.contains("more"));
}

#[test]
fn markdown_max_items_zero() {
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

    // Should show truncation message immediately
    assert!(md.contains("And 1 more"));
}

#[test]
fn markdown_max_items_one() {
    let findings = vec![
        create_finding("src/lib.rs", 1, Severity::Warn, "W001", "First"),
        create_finding("src/lib.rs", 2, Severity::Warn, "W002", "Second"),
    ];
    let r = create_report(VerdictStatus::Warn, findings);
    let opts = MarkdownOptions {
        max_items: 1,
        report_path: DEFAULT_REPORT_PATH.to_string(),
    };
    let md = render_markdown(&r, opts);

    // Only 1 row should appear
    let table_rows = md.matches("| warn |").count();
    assert_eq!(table_rows, 1);
    assert!(md.contains("And 1 more"));
}

// =============================================================================
// GitHub Annotations Max Limit Tests
// =============================================================================

#[test]
fn annotations_max_items_limits_output() {
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
    let r = create_report(VerdictStatus::Warn, findings);
    let out = render_github_annotations(&r, 10);

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 10);
}

#[test]
fn annotations_max_items_large_limit() {
    let findings: Vec<Finding> = (1..=50)
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
    let out = render_github_annotations(&r, 100);

    // All 50 should appear
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 50);
}

#[test]
fn annotations_max_items_zero() {
    let findings = vec![create_finding(
        "src/lib.rs",
        1,
        Severity::Warn,
        "W001",
        "Warning",
    )];
    let r = create_report(VerdictStatus::Warn, findings);
    let out = render_github_annotations(&r, 0);

    assert!(out.trim().is_empty());
}

#[test]
fn annotations_max_items_one() {
    let findings = vec![
        create_finding("src/a.rs", 1, Severity::Error, "E001", "First"),
        create_finding("src/b.rs", 2, Severity::Error, "E002", "Second"),
    ];
    let r = create_report(VerdictStatus::Fail, findings);
    let out = render_github_annotations(&r, 1);

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 1);
}

// =============================================================================
// Truncation Markers Tests
// =============================================================================

#[test]
fn markdown_truncation_marker_format() {
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
        max_items: 20,
        report_path: DEFAULT_REPORT_PATH.to_string(),
    };
    let md = render_markdown(&r, opts);

    // Should show "And X more…" format
    assert!(md.contains("_And 5 more"));
    assert!(md.contains("See full receipt:"));
}

#[test]
fn markdown_truncation_includes_report_path() {
    let findings: Vec<Finding> = (1..=30)
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
        report_path: "custom/path/report.json".to_string(),
    };
    let md = render_markdown(&r, opts);

    assert!(md.contains("custom/path/report.json"));
}

#[test]
fn markdown_data_truncated_flag() {
    let findings = vec![create_finding(
        "src/lib.rs",
        1,
        Severity::Warn,
        "W001",
        "Warning",
    )];
    let data = json!({
        "truncated": true
    });
    let r = create_report_with_data(VerdictStatus::Warn, findings, data);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("Output truncated"));
    assert!(md.contains("See full receipt"));
}

#[test]
fn markdown_data_truncated_false() {
    let findings = vec![create_finding(
        "src/lib.rs",
        1,
        Severity::Warn,
        "W001",
        "Warning",
    )];
    let data = json!({
        "truncated": false
    });
    let r = create_report_with_data(VerdictStatus::Warn, findings, data);
    let md = render_markdown(&r, MarkdownOptions::default());

    // Should not show truncation message when truncated is false
    assert!(!md.contains("Output truncated"));
}

#[test]
fn markdown_data_truncated_flag_with_explain_summary() {
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
    let data = json!({
        "truncated": true,
        "explain_summary": {
            "total": 100,
            "included": 5
        }
    });
    let r = create_report_with_data(VerdictStatus::Warn, findings, data);
    let md = render_markdown(&r, MarkdownOptions::default());

    // Both truncation marker and explain summary should appear
    assert!(md.contains("Output truncated"));
    assert!(md.contains("100 total: 5 matched"));
}

// =============================================================================
// Summary Counts Tests
// =============================================================================

#[test]
fn markdown_counts_match_findings() {
    let findings = vec![
        create_finding("src/a.rs", 1, Severity::Error, "E001", "Error"),
        create_finding("src/b.rs", 2, Severity::Error, "E002", "Error"),
        create_finding("src/c.rs", 3, Severity::Warn, "W001", "Warn"),
        create_finding("src/d.rs", 4, Severity::Info, "I001", "Info"),
    ];
    let r = create_report(VerdictStatus::Fail, findings);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("error 2 · warn 1 · info 1"));
}

#[test]
fn markdown_counts_independent_of_display_limit() {
    // Counts should reflect all findings, not just displayed ones
    let findings: Vec<Finding> = (1..=30)
        .enumerate()
        .map(|(i, _)| {
            let sev = if i < 10 {
                Severity::Error
            } else if i < 20 {
                Severity::Warn
            } else {
                Severity::Info
            };
            create_finding(
                "src/lib.rs",
                i as u32 + 1,
                sev,
                &format!("CODE_{:03}", i),
                "Message",
            )
        })
        .collect();
    let r = create_report(VerdictStatus::Fail, findings);
    let opts = MarkdownOptions {
        max_items: 5,
        report_path: DEFAULT_REPORT_PATH.to_string(),
    };
    let md = render_markdown(&r, opts);

    // Counts should show all 30 findings categorized
    assert!(md.contains("error 10 · warn 10 · info 10"));
}

#[test]
fn markdown_zero_counts() {
    let r = create_report(VerdictStatus::Pass, vec![]);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("error 0 · warn 0 · info 0"));
}

#[test]
fn markdown_only_errors() {
    let findings = vec![
        create_finding("src/a.rs", 1, Severity::Error, "E001", "Error"),
        create_finding("src/b.rs", 2, Severity::Error, "E002", "Error"),
    ];
    let r = create_report(VerdictStatus::Fail, findings);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("error 2 · warn 0 · info 0"));
}

#[test]
fn markdown_only_warnings() {
    let findings = vec![
        create_finding("src/a.rs", 1, Severity::Warn, "W001", "Warn"),
        create_finding("src/b.rs", 2, Severity::Warn, "W002", "Warn"),
        create_finding("src/c.rs", 3, Severity::Warn, "W003", "Warn"),
    ];
    let r = create_report(VerdictStatus::Warn, findings);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("error 0 · warn 3 · info 0"));
}

#[test]
fn markdown_only_info() {
    let findings = vec![create_finding(
        "src/a.rs",
        1,
        Severity::Info,
        "I001",
        "Info",
    )];
    let r = create_report(VerdictStatus::Pass, findings);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("error 0 · warn 0 · info 1"));
}

// =============================================================================
// Explain Summary Budget Tests
// =============================================================================

#[test]
fn explain_summary_included_count() {
    let data = json!({
        "explain_summary": {
            "total": 100,
            "included": 42
        }
    });
    let r = create_report_with_data(VerdictStatus::Warn, vec![], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    assert!(md.contains("42 matched"));
}

#[test]
fn explain_summary_dropped_counts() {
    let data = json!({
        "explain_summary": {
            "total": 100,
            "included": 40,
            "dropped_outside_diff": 30,
            "dropped_no_span": 10,
            "dropped_by_path_filter": 15,
            "suppressed_by_code": 5
        }
    });
    let r = create_report_with_data(VerdictStatus::Warn, vec![], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    // All drop reasons should be shown
    assert!(md.contains("30 outside diff"));
    assert!(md.contains("10 no span"));
    assert!(md.contains("15 filtered by path"));
    assert!(md.contains("5 suppressed"));
}

#[test]
fn explain_summary_zero_drops_not_shown() {
    let data = json!({
        "explain_summary": {
            "total": 10,
            "included": 10,
            "dropped_outside_diff": 0,
            "dropped_no_span": 0,
            "dropped_by_path_filter": 0,
            "suppressed_by_code": 0
        }
    });
    let r = create_report_with_data(VerdictStatus::Pass, vec![], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    // Zero drop counts should not appear
    assert!(md.contains("10 total: 10 matched"));
    assert!(!md.contains("0 outside diff"));
    assert!(!md.contains("0 no span"));
    assert!(!md.contains("0 filtered"));
    assert!(!md.contains("0 suppressed"));
}

#[test]
fn explain_summary_partial_drops() {
    let data = json!({
        "explain_summary": {
            "total": 100,
            "included": 80,
            "dropped_outside_diff": 20,
            "dropped_no_span": 0,
            "dropped_by_path_filter": 0,
            "suppressed_by_code": 0
        }
    });
    let r = create_report_with_data(VerdictStatus::Warn, vec![], data);
    let md = render_markdown(&r, MarkdownOptions::default());

    // Only non-zero drops should appear
    assert!(md.contains("20 outside diff"));
    assert!(!md.contains("no span"));
    assert!(!md.contains("filtered"));
    assert!(!md.contains("suppressed"));
}

// =============================================================================
// Sorting with Budget Tests
// =============================================================================

#[test]
fn markdown_truncation_preserves_sorting() {
    // Create findings in random order
    let findings = vec![
        create_finding("src/info.rs", 1, Severity::Info, "I001", "Info"),
        create_finding("src/error.rs", 2, Severity::Error, "E001", "Error"),
        create_finding("src/warn.rs", 3, Severity::Warn, "W001", "Warn"),
        create_finding("src/info2.rs", 4, Severity::Info, "I002", "Info2"),
        create_finding("src/error2.rs", 5, Severity::Error, "E002", "Error2"),
    ];
    let r = create_report(VerdictStatus::Fail, findings);
    let opts = MarkdownOptions {
        max_items: 3,
        report_path: DEFAULT_REPORT_PATH.to_string(),
    };
    let md = render_markdown(&r, opts);

    // First 3 displayed should be errors (2) then warnings (1)
    // E001, E002, W001 should be shown; I001, I002 truncated
    let e001_pos = md.find("E001").unwrap_or(usize::MAX);
    let e002_pos = md.find("E002").unwrap_or(usize::MAX);
    let w001_pos = md.find("W001").unwrap_or(usize::MAX);

    // Verify errors come before warning
    assert!(e001_pos < w001_pos);
    assert!(e002_pos < w001_pos);

    // Info findings should be in "more" count
    assert!(md.contains("And 2 more"));
}

#[test]
fn annotations_truncation_preserves_sorting() {
    let findings = vec![
        create_finding("src/info.rs", 1, Severity::Info, "I001", "Info"),
        create_finding("src/error.rs", 2, Severity::Error, "E001", "Error"),
        create_finding("src/warn.rs", 3, Severity::Warn, "W001", "Warn"),
    ];
    let r = create_report(VerdictStatus::Fail, findings);
    let out = render_github_annotations(&r, 2);

    // Error should be first, then warning; info should be truncated
    let error_pos = out.find("::error").unwrap_or(usize::MAX);
    let warn_pos = out.find("::warning").unwrap_or(usize::MAX);

    assert!(error_pos < warn_pos);
    assert!(!out.contains("::notice"));
}

// =============================================================================
// Edge Cases with Budget
// =============================================================================

#[test]
fn markdown_large_finding_count() {
    let findings: Vec<Finding> = (1..=1000)
        .map(|i| {
            create_finding(
                "src/lib.rs",
                i,
                Severity::Warn,
                &format!("W{:04}", i),
                "Warning",
            )
        })
        .collect();
    let r = create_report(VerdictStatus::Warn, findings);
    let opts = MarkdownOptions {
        max_items: 20,
        report_path: DEFAULT_REPORT_PATH.to_string(),
    };
    let md = render_markdown(&r, opts);

    // Should handle large counts
    assert!(md.contains("And 980 more"));
    assert!(md.contains("warn 1000"));
}

#[test]
fn annotations_large_finding_count() {
    let findings: Vec<Finding> = (1..=1000)
        .map(|i| {
            create_finding(
                "src/lib.rs",
                i,
                Severity::Warn,
                &format!("W{:04}", i),
                "Warning",
            )
        })
        .collect();
    let r = create_report(VerdictStatus::Warn, findings);
    let out = render_github_annotations(&r, 50);

    // Should limit to 50
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 50);
}

#[test]
fn markdown_mixed_severities_large_count() {
    let mut findings = Vec::new();
    for i in 1..=100 {
        findings.push(create_finding(
            "src/e.rs",
            i,
            Severity::Error,
            &format!("E{:03}", i),
            "Error",
        ));
    }
    for i in 1..=100 {
        findings.push(create_finding(
            "src/w.rs",
            i,
            Severity::Warn,
            &format!("W{:03}", i),
            "Warn",
        ));
    }
    for i in 1..=100 {
        findings.push(create_finding(
            "src/i.rs",
            i,
            Severity::Info,
            &format!("I{:03}", i),
            "Info",
        ));
    }

    let r = create_report(VerdictStatus::Fail, findings);
    let opts = MarkdownOptions {
        max_items: 50,
        report_path: DEFAULT_REPORT_PATH.to_string(),
    };
    let md = render_markdown(&r, opts);

    // Counts should reflect all 300 findings
    assert!(md.contains("error 100 · warn 100 · info 100"));

    // Should show truncation
    assert!(md.contains("And 250 more"));

    // Table should have 50 rows
    let error_rows = md.matches("| error |").count();
    assert_eq!(error_rows, 50); // Only errors fit in first 50
}

// =============================================================================
// Report Path in Truncation Messages
// =============================================================================

#[test]
fn markdown_truncated_shows_report_path() {
    let findings: Vec<Finding> = (1..=30)
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
        report_path: "my/custom/path.json".to_string(),
    };
    let md = render_markdown(&r, opts);

    // Truncation message should include the path
    assert!(md.contains("my/custom/path.json"));
}

#[test]
fn markdown_no_findings_shows_report_path() {
    let r = create_report(VerdictStatus::Pass, vec![]);
    let opts = MarkdownOptions {
        max_items: 10,
        report_path: "reports/lintdiff.json".to_string(),
    };
    let md = render_markdown(&r, opts);

    assert!(md.contains("reports/lintdiff.json"));
}

#[test]
fn markdown_all_findings_shown_includes_report_path() {
    let findings = vec![create_finding(
        "src/lib.rs",
        1,
        Severity::Warn,
        "W001",
        "Warning",
    )];
    let r = create_report(VerdictStatus::Warn, findings);
    let opts = MarkdownOptions {
        max_items: 10,
        report_path: "output/report.json".to_string(),
    };
    let md = render_markdown(&r, opts);

    assert!(md.contains("Full receipt: `output/report.json`"));
}
