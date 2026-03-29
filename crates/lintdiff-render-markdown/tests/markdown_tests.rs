//! Comprehensive tests for markdown rendering functionality.

use lintdiff_render_markdown::{
    render_finding_markdown, render_markdown, render_summary, MarkdownConfig,
};
use lintdiff_stats::Stats;
use lintdiff_types::{Finding, Location, NormPath, Severity};

// =============================================================================
// Test Helpers
// =============================================================================

fn create_finding(severity: Severity, path: &str, line: u32, code: &str, message: &str) -> Finding {
    Finding {
        severity,
        code: code.to_string(),
        message: message.to_string(),
        location: Some(Location {
            path: NormPath::new(path),
            line: Some(line),
            col: None,
        }),
        check_id: None,
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

fn create_finding_with_col(
    severity: Severity,
    path: &str,
    line: u32,
    col: u32,
    code: &str,
    message: &str,
) -> Finding {
    Finding {
        severity,
        code: code.to_string(),
        message: message.to_string(),
        location: Some(Location {
            path: NormPath::new(path),
            line: Some(line),
            col: Some(col),
        }),
        check_id: None,
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

fn create_finding_no_location(severity: Severity, code: &str, message: &str) -> Finding {
    Finding {
        severity,
        code: code.to_string(),
        message: message.to_string(),
        location: None,
        check_id: None,
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

// =============================================================================
// Empty Findings Tests
// =============================================================================

#[test]
fn empty_findings_returns_message() {
    let findings: Vec<Finding> = vec![];
    let md = render_markdown(&findings, &MarkdownConfig::default());
    assert!(md.contains("No findings"));
}

#[test]
fn empty_findings_has_no_table() {
    let findings: Vec<Finding> = vec![];
    let md = render_markdown(&findings, &MarkdownConfig::default());
    assert!(!md.contains("| Sev |"));
}

// =============================================================================
// Single Finding Tests
// =============================================================================

#[test]
fn single_error_renders_correctly() {
    let findings = vec![create_finding(
        Severity::Error,
        "src/lib.rs",
        10,
        "E001",
        "Test error message",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    assert!(md.contains("| Sev | Location | Code | Message |"));
    assert!(md.contains("error"));
    assert!(md.contains("src/lib.rs:10"));
    assert!(md.contains("`E001`"));
    assert!(md.contains("Test error message"));
}

#[test]
fn single_warning_renders_correctly() {
    let findings = vec![create_finding(
        Severity::Warn,
        "src/warn.rs",
        20,
        "W001",
        "Test warning message",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    assert!(md.contains("warn"));
    assert!(md.contains("src/warn.rs:20"));
}

#[test]
fn single_info_renders_correctly() {
    let findings = vec![create_finding(
        Severity::Info,
        "src/info.rs",
        30,
        "I001",
        "Test info message",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    assert!(md.contains("info"));
    assert!(md.contains("src/info.rs:30"));
}

#[test]
fn finding_with_column_in_location() {
    let findings = vec![create_finding_with_col(
        Severity::Error,
        "src/lib.rs",
        10,
        5,
        "E001",
        "Error with column",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    // Column is not shown in the location format (only path:line)
    assert!(md.contains("src/lib.rs:10"));
}

#[test]
fn finding_without_location_shows_dash() {
    let findings = vec![create_finding_no_location(
        Severity::Error,
        "E001",
        "No location",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    assert!(md.contains("`-`"));
}

// =============================================================================
// Multiple Findings Tests
// =============================================================================

#[test]
fn multiple_findings_all_rendered() {
    let findings = vec![
        create_finding(Severity::Error, "src/a.rs", 1, "E001", "Error 1"),
        create_finding(Severity::Warn, "src/b.rs", 2, "W001", "Warning 1"),
        create_finding(Severity::Info, "src/c.rs", 3, "I001", "Info 1"),
    ];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    assert!(md.contains("E001"));
    assert!(md.contains("W001"));
    assert!(md.contains("I001"));
}

#[test]
fn findings_sorted_by_severity() {
    let findings = vec![
        create_finding(Severity::Info, "src/a.rs", 1, "I001", "Info"),
        create_finding(Severity::Error, "src/b.rs", 2, "E001", "Error"),
        create_finding(Severity::Warn, "src/c.rs", 3, "W001", "Warning"),
    ];
    let md = render_markdown(&findings, &MarkdownConfig::default());
    let lines: Vec<&str> = md.lines().collect();

    // After header (2 lines), error should be first
    assert!(lines[2].contains("error"));
    assert!(lines[3].contains("warn"));
    assert!(lines[4].contains("info"));
}

#[test]
fn findings_sorted_by_path_when_same_severity() {
    let findings = vec![
        create_finding(Severity::Warn, "src/z.rs", 1, "W001", "Warning Z"),
        create_finding(Severity::Warn, "src/a.rs", 2, "W002", "Warning A"),
    ];
    let md = render_markdown(&findings, &MarkdownConfig::default());
    let lines: Vec<&str> = md.lines().collect();

    // 'a' should come before 'z' alphabetically
    assert!(lines[2].contains("src/a.rs"));
    assert!(lines[3].contains("src/z.rs"));
}

// =============================================================================
// Config Options Tests
// =============================================================================

#[test]
fn config_max_line_length_truncates_long_messages() {
    let long_message = "x".repeat(200);
    let findings = vec![create_finding(
        Severity::Warn,
        "src/lib.rs",
        1,
        "W001",
        &long_message,
    )];

    let config = MarkdownConfig {
        max_line_length: 50,
        ..Default::default()
    };
    let md = render_markdown(&findings, &config);

    assert!(md.contains("..."));
}

#[test]
fn config_max_line_length_default_preserves_normal_messages() {
    let normal_message = "This is a normal message";
    let findings = vec![create_finding(
        Severity::Warn,
        "src/lib.rs",
        1,
        "W001",
        normal_message,
    )];

    let config = MarkdownConfig::default();
    let md = render_markdown(&findings, &config);

    assert!(md.contains(normal_message));
    assert!(!md.contains("..."));
}

#[test]
fn config_include_snippets_true_adds_backticks() {
    let findings = vec![create_finding(
        Severity::Error,
        "src/lib.rs",
        1,
        "E001",
        "Error",
    )];

    let config = MarkdownConfig {
        include_snippets: true,
        ..Default::default()
    };
    let md = render_markdown(&findings, &config);

    assert!(md.contains("`E001`"));
}

#[test]
fn config_include_snippets_false_no_backticks() {
    let findings = vec![create_finding(
        Severity::Error,
        "src/lib.rs",
        1,
        "E001",
        "Error",
    )];

    let config = MarkdownConfig {
        include_snippets: false,
        ..Default::default()
    };
    let md = render_markdown(&findings, &config);

    // Should contain the code but not wrapped in backticks
    assert!(md.contains("E001"));
    // Check that E001 is not wrapped in backticks by checking the pattern
    assert!(!md.contains("| `E001` |"));
}

#[test]
fn config_default_values() {
    let config = MarkdownConfig::default();

    assert_eq!(config.max_line_length, 120);
    assert!(config.include_snippets);
    assert!(config.gfm);
}

// =============================================================================
// Escape and Special Character Tests
// =============================================================================

#[test]
fn pipe_character_escaped() {
    let findings = vec![create_finding(
        Severity::Warn,
        "src/lib.rs",
        1,
        "W001",
        "Message with | pipe",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    assert!(md.contains("Message with \\| pipe"));
}

#[test]
fn multiple_pipes_escaped() {
    let findings = vec![create_finding(
        Severity::Warn,
        "src/lib.rs",
        1,
        "W001",
        "a | b | c",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    assert!(md.contains("a \\| b \\| c"));
}

#[test]
fn newline_replaced_with_space() {
    let findings = vec![create_finding(
        Severity::Warn,
        "src/lib.rs",
        1,
        "W001",
        "Line 1\nLine 2",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    assert!(md.contains("Line 1 Line 2"));
}

#[test]
fn multiple_newlines_replaced() {
    let findings = vec![create_finding(
        Severity::Warn,
        "src/lib.rs",
        1,
        "W001",
        "Line 1\nLine 2\nLine 3",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    assert!(md.contains("Line 1 Line 2 Line 3"));
}

#[test]
fn carriage_return_removed() {
    let findings = vec![create_finding(
        Severity::Warn,
        "src/lib.rs",
        1,
        "W001",
        "Line 1\r\nLine 2",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    // Carriage return should be removed, newline replaced with space
    assert!(md.contains("Line 1 Line 2"));
}

// =============================================================================
// render_finding_markdown Tests
// =============================================================================

#[test]
fn render_finding_markdown_returns_table_row() {
    let finding = create_finding(Severity::Error, "src/test.rs", 42, "E001", "Error message");
    let md = render_finding_markdown(&finding, &MarkdownConfig::default());

    assert!(md.starts_with('|'));
    assert!(md.ends_with('\n'));
}

#[test]
fn render_finding_markdown_contains_all_fields() {
    let finding = create_finding(Severity::Warn, "src/warn.rs", 100, "WARN_CODE", "Warning!");
    let md = render_finding_markdown(&finding, &MarkdownConfig::default());

    assert!(md.contains("warn"));
    assert!(md.contains("src/warn.rs:100"));
    assert!(md.contains("WARN_CODE"));
    assert!(md.contains("Warning!"));
}

// =============================================================================
// render_summary Tests
// =============================================================================

#[test]
fn summary_empty_stats() {
    let stats = Stats::new();
    let summary = render_summary(&stats, &MarkdownConfig::default());

    assert!(summary.contains("Summary"));
    assert!(summary.contains("Total: 0"));
    assert!(summary.contains("Matched: 0"));
    assert!(summary.contains("Files: 0"));
}

#[test]
fn summary_with_counts() {
    let mut stats = Stats::new();
    stats.total_diagnostics = 100;
    stats.matched_diagnostics = 50;
    stats.files_affected = 10;

    let summary = render_summary(&stats, &MarkdownConfig::default());

    assert!(summary.contains("Total: 100"));
    assert!(summary.contains("Matched: 50"));
    assert!(summary.contains("Files: 10"));
}

#[test]
fn summary_with_severity_breakdown() {
    let mut stats = Stats::new();
    stats.by_severity.insert("error".to_string(), 5);
    stats.by_severity.insert("warning".to_string(), 10);
    stats.by_severity.insert("info".to_string(), 2);

    let summary = render_summary(&stats, &MarkdownConfig::default());

    assert!(summary.contains("By Severity"));
}

#[test]
fn summary_with_code_breakdown() {
    let mut stats = Stats::new();
    stats.by_code.insert("clippy::unwrap_used".to_string(), 10);
    stats.by_code.insert("clippy::map_identity".to_string(), 5);

    let summary = render_summary(&stats, &MarkdownConfig::default());

    assert!(summary.contains("Top Codes"));
    assert!(summary.contains("clippy::unwrap_used"));
}

#[test]
fn summary_codes_sorted_by_count() {
    let mut stats = Stats::new();
    stats.by_code.insert("LOW_COUNT".to_string(), 1);
    stats.by_code.insert("HIGH_COUNT".to_string(), 100);
    stats.by_code.insert("MID_COUNT".to_string(), 50);

    let summary = render_summary(&stats, &MarkdownConfig::default());

    // HIGH_COUNT should appear before MID_COUNT and LOW_COUNT
    let high_pos = summary
        .find("HIGH_COUNT")
        .expect("HIGH_COUNT should be present");
    let mid_pos = summary
        .find("MID_COUNT")
        .expect("MID_COUNT should be present");
    let low_pos = summary
        .find("LOW_COUNT")
        .expect("LOW_COUNT should be present");

    assert!(high_pos < mid_pos);
    assert!(mid_pos < low_pos);
}

#[test]
fn summary_limited_codes_with_many_entries() {
    let mut stats = Stats::new();
    for i in 0..20 {
        stats.by_code.insert(format!("CODE_{i:02}"), 1);
    }

    let config = MarkdownConfig {
        max_line_length: 120,
        ..Default::default()
    };
    let summary = render_summary(&stats, &config);

    // Should show "and X more" for codes beyond the limit
    assert!(summary.contains("... and"));
}

#[test]
fn summary_snippets_config_affects_code_display() {
    let mut stats = Stats::new();
    stats.by_code.insert("TEST_CODE".to_string(), 5);

    let config_with_snippets = MarkdownConfig {
        include_snippets: true,
        ..Default::default()
    };
    let summary = render_summary(&stats, &config_with_snippets);
    assert!(summary.contains("`TEST_CODE`"));

    let config_no_snippets = MarkdownConfig {
        include_snippets: false,
        ..Default::default()
    };
    let summary = render_summary(&stats, &config_no_snippets);
    // Code should still appear but without backticks in the list format
    assert!(summary.contains("TEST_CODE"));
}

// =============================================================================
// Path Normalization Tests
// =============================================================================

#[test]
fn backslash_path_normalized() {
    let finding = Finding {
        severity: Severity::Error,
        code: "E001".to_string(),
        message: "Error".to_string(),
        location: Some(Location {
            path: NormPath::new("src\\lib.rs"),
            line: Some(10),
            col: None,
        }),
        check_id: None,
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    };

    let md = render_finding_markdown(&finding, &MarkdownConfig::default());
    // NormPath converts backslashes to forward slashes
    assert!(md.contains("src/lib.rs"));
}

#[test]
fn dot_slash_path_normalized() {
    let finding = Finding {
        severity: Severity::Error,
        code: "E001".to_string(),
        message: "Error".to_string(),
        location: Some(Location {
            path: NormPath::new("./src/lib.rs"),
            line: Some(10),
            col: None,
        }),
        check_id: None,
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    };

    let md = render_finding_markdown(&finding, &MarkdownConfig::default());
    // NormPath removes leading ./
    assert!(md.contains("src/lib.rs"));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn empty_message_renders() {
    let findings = vec![create_finding(Severity::Warn, "src/lib.rs", 1, "W001", "")];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    // Should still render the row
    assert!(md.contains("W001"));
}

#[test]
fn empty_code_renders() {
    let findings = vec![create_finding(
        Severity::Warn,
        "src/lib.rs",
        1,
        "",
        "Message",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    // Should still render the row
    assert!(md.contains("Message"));
}

#[test]
fn unicode_in_message_renders() {
    let findings = vec![create_finding(
        Severity::Warn,
        "src/lib.rs",
        1,
        "W001",
        "Unicode: 日本語 🎉 émoji",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    assert!(md.contains("Unicode: 日本語 🎉 émoji"));
}

#[test]
fn very_long_path_renders() {
    let long_path = format!("src/{}/lib.rs", "a".repeat(100));
    let findings = vec![create_finding(
        Severity::Error,
        &long_path,
        1,
        "E001",
        "Error",
    )];
    let md = render_markdown(&findings, &MarkdownConfig::default());

    assert!(md.contains(&long_path));
}

#[test]
fn location_without_line_number() {
    let finding = Finding {
        severity: Severity::Error,
        code: "E001".to_string(),
        message: "Error".to_string(),
        location: Some(Location {
            path: NormPath::new("src/lib.rs"),
            line: None,
            col: None,
        }),
        check_id: None,
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    };

    let md = render_finding_markdown(&finding, &MarkdownConfig::default());
    // Should show just the path without line number
    assert!(md.contains("`src/lib.rs`"));
    assert!(!md.contains("src/lib.rs:"));
}
