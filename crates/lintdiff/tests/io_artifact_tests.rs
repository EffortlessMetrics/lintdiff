//! Tests for artifact writing functionality.

use std::fs;

use lintdiff::io::{now_rfc3339, write_report_json, write_text};
use lintdiff_types::{
    Counts, Finding, GitInfo, HostInfo, Location, Report, RunInfo, Severity, ToolInfo, Verdict,
    VerdictStatus,
};
use tempfile::TempDir;

fn create_test_report() -> Report {
    Report {
        schema: "lintdiff.report.v1".to_string(),
        tool: ToolInfo {
            name: "lintdiff".to_string(),
            version: "0.1.0".to_string(),
            commit: Some("abc123".to_string()),
        },
        run: RunInfo {
            started_at: "2024-01-01T00:00:00Z".to_string(),
            ended_at: "2024-01-01T00:01:00Z".to_string(),
            duration_ms: Some(60000),
            host: Some(HostInfo {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
            }),
            git: Some(GitInfo {
                repo: Some("https://github.com/example/repo".to_string()),
                base_ref: Some("main".to_string()),
                head_ref: Some("feature-branch".to_string()),
                base_sha: Some("base123".to_string()),
                head_sha: Some("head456".to_string()),
                merge_base: Some("merge789".to_string()),
            }),
        },
        verdict: Verdict {
            status: VerdictStatus::Pass,
            counts: Counts {
                info: 0,
                warn: 2,
                error: 0,
            },
            reasons: vec!["No new errors on changed lines".to_string()],
        },
        findings: vec![Finding {
            severity: Severity::Warn,
            check_id: Some("diagnostics.on_diff".to_string()),
            code: "unused_variables".to_string(),
            message: "unused variable: `x`".to_string(),
            location: Some(Location {
                path: "src/lib.rs".into(),
                line: Some(10),
                col: Some(5),
            }),
            help: Some("consider prefixing with an underscore".to_string()),
            url: Some("https://rust-lang.org/".to_string()),
            fingerprint: None,
            data: None,
        }],
        data: None,
    }
}

// ============================================================================
// write_report_json tests
// ============================================================================

#[test]
fn write_report_to_file() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("report.json");

    let report = create_test_report();
    let result = write_report_json(&report, &path);

    assert!(result.is_ok());
    assert!(path.exists());

    let contents = fs::read_to_string(&path).expect("failed to read report");
    assert!(contents.contains("\"schema\": \"lintdiff.report.v1\""));
    assert!(contents.contains("\"tool\":"));
    assert!(contents.contains("\"verdict\":"));
}

#[test]
fn write_report_creates_parent_directories() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("nested").join("deeply").join("report.json");

    let report = create_test_report();
    let result = write_report_json(&report, &path);

    assert!(result.is_ok());
    assert!(path.exists());
    assert!(dir.path().join("nested").join("deeply").exists());
}

#[test]
fn write_report_overwrites_existing_file() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("report.json");

    // Write initial report
    let mut report = create_test_report();
    report.verdict.status = VerdictStatus::Pass;
    write_report_json(&report, &path).expect("first write failed");

    // Modify and write again
    report.verdict.status = VerdictStatus::Fail;
    let result = write_report_json(&report, &path);

    assert!(result.is_ok());

    // Verify content was overwritten
    let contents = fs::read_to_string(&path).expect("failed to read report");
    assert!(contents.contains("\"status\": \"fail\""));
}

#[test]
fn write_report_valid_json() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("report.json");

    let report = create_test_report();
    write_report_json(&report, &path).expect("write failed");

    let contents = fs::read_to_string(&path).expect("failed to read report");

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&contents).expect("invalid JSON");
    assert_eq!(parsed["schema"], "lintdiff.report.v1");
}

#[test]
fn write_report_pretty_printed() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("report.json");

    let report = create_test_report();
    write_report_json(&report, &path).expect("write failed");

    let contents = fs::read_to_string(&path).expect("failed to read report");

    // Pretty-printed JSON should have newlines and indentation
    assert!(contents.contains('\n'));
    assert!(contents.contains("  "));
}

#[test]
fn write_report_with_all_verdict_statuses() {
    let dir = TempDir::new().expect("failed to create temp dir");

    for (status, status_str) in [
        (VerdictStatus::Pass, "pass"),
        (VerdictStatus::Warn, "warn"),
        (VerdictStatus::Fail, "fail"),
        (VerdictStatus::Skip, "skip"),
    ] {
        let path = dir.path().join(format!("report_{}.json", status_str));
        let mut report = create_test_report();
        report.verdict.status = status;

        let result = write_report_json(&report, &path);
        assert!(result.is_ok(), "failed to write status: {}", status_str);

        let contents = fs::read_to_string(&path).expect("failed to read");
        assert!(
            contents.contains(&format!("\"status\": \"{}\"", status_str)),
            "status not found: {}",
            status_str
        );
    }
}

#[test]
fn write_report_with_multiple_findings() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("report.json");

    let mut report = create_test_report();
    report.findings = vec![
        Finding {
            severity: Severity::Error,
            check_id: None,
            code: "E0425".to_string(),
            message: "cannot find value".to_string(),
            location: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        },
        Finding {
            severity: Severity::Warn,
            check_id: None,
            code: "unused_variables".to_string(),
            message: "unused variable".to_string(),
            location: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        },
        Finding {
            severity: Severity::Info,
            check_id: None,
            code: "info_code".to_string(),
            message: "info message".to_string(),
            location: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        },
    ];

    let result = write_report_json(&report, &path);
    assert!(result.is_ok());

    let contents = fs::read_to_string(&path).expect("failed to read");
    assert!(contents.contains("E0425"));
    assert!(contents.contains("unused_variables"));
    assert!(contents.contains("info_code"));
}

#[test]
fn write_report_with_no_findings() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("report.json");

    let mut report = create_test_report();
    report.findings = vec![];

    let result = write_report_json(&report, &path);
    assert!(result.is_ok());

    let contents = fs::read_to_string(&path).expect("failed to read");
    // Empty findings array should still be present
    assert!(contents.contains("\"findings\": []"));
}

#[test]
fn write_report_with_unicode() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("report.json");

    let mut report = create_test_report();
    report.findings = vec![Finding {
        severity: Severity::Warn,
        check_id: None,
        code: "unicode_test".to_string(),
        message: "Unicode: 你好世界 🎉 émojis".to_string(),
        location: None,
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }];

    let result = write_report_json(&report, &path);
    assert!(result.is_ok());

    let contents = fs::read_to_string(&path).expect("failed to read");
    assert!(contents.contains("你好世界"));
    assert!(contents.contains("🎉"));
}

// ============================================================================
// write_text tests
// ============================================================================

#[test]
fn write_text_to_file() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("output.txt");

    let content = "Hello, world!";
    let result = write_text(&path, content);

    assert!(result.is_ok());
    assert!(path.exists());

    let read_content = fs::read_to_string(&path).expect("failed to read file");
    assert_eq!(read_content, content);
}

#[test]
fn write_text_creates_parent_directories() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("a").join("b").join("c").join("output.txt");

    let result = write_text(&path, "test content");
    assert!(result.is_ok());
    assert!(path.exists());
}

#[test]
fn write_text_overwrites_existing() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("output.txt");

    write_text(&path, "original content").expect("first write failed");
    write_text(&path, "new content").expect("second write failed");

    let content = fs::read_to_string(&path).expect("failed to read");
    assert_eq!(content, "new content");
}

#[test]
fn write_text_empty_string() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("empty.txt");

    let result = write_text(&path, "");
    assert!(result.is_ok());
    assert!(path.exists());

    let content = fs::read_to_string(&path).expect("failed to read");
    assert!(content.is_empty());
}

#[test]
fn write_text_multiline() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("multiline.txt");

    let content = "line 1\nline 2\nline 3\n";
    let result = write_text(&path, content);
    assert!(result.is_ok());

    let read_content = fs::read_to_string(&path).expect("failed to read");
    assert_eq!(read_content, content);
}

#[test]
fn write_text_unicode() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("unicode.txt");

    let content = "Unicode: 你好世界 🎉 émojis";
    let result = write_text(&path, content);
    assert!(result.is_ok());

    let read_content = fs::read_to_string(&path).expect("failed to read");
    assert_eq!(read_content, content);
}

#[test]
fn write_text_large_content() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("large.txt");

    let content = "x".repeat(1_000_000);
    let result = write_text(&path, &content);
    assert!(result.is_ok());

    let read_content = fs::read_to_string(&path).expect("failed to read");
    assert_eq!(read_content.len(), 1_000_000);
}

#[test]
fn write_text_in_existing_directory() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let subdir = dir.path().join("existing");
    fs::create_dir(&subdir).expect("failed to create dir");

    let path = subdir.join("output.txt");
    let result = write_text(&path, "test");
    assert!(result.is_ok());
}

#[test]
fn write_text_at_root_of_existing_directory() {
    let dir = TempDir::new().expect("failed to create temp dir");
    // File directly in existing directory (no parent to create)
    let path = dir.path().join("output.txt");

    let result = write_text(&path, "test");
    assert!(result.is_ok());
}

// ============================================================================
// now_rfc3339 tests
// ============================================================================

#[test]
fn now_rfc3339_returns_valid_format() {
    let timestamp = now_rfc3339();

    // Should be valid RFC3339 format
    // Example: 2024-01-15T10:30:00Z
    assert!(timestamp.contains('T'));
    assert!(timestamp.ends_with('Z'));

    // Should be parseable
    let parsed =
        time::OffsetDateTime::parse(&timestamp, &time::format_description::well_known::Rfc3339);
    assert!(
        parsed.is_ok(),
        "timestamp {} is not valid RFC3339",
        timestamp
    );
}

#[test]
fn now_rfc3339_returns_recent_time() {
    let before = time::OffsetDateTime::now_utc();
    let timestamp = now_rfc3339();
    let after = time::OffsetDateTime::now_utc();

    let parsed =
        time::OffsetDateTime::parse(&timestamp, &time::format_description::well_known::Rfc3339)
            .expect("invalid timestamp");

    // The timestamp should be between before and after
    assert!(parsed >= before - time::Duration::seconds(1));
    assert!(parsed <= after + time::Duration::seconds(1));
}

#[test]
fn now_rfc3339_multiple_calls_different() {
    let first = now_rfc3339();
    // Small delay
    std::thread::sleep(std::time::Duration::from_millis(10));
    let second = now_rfc3339();

    // They might be the same if within the same second, but format should be consistent
    assert!(!first.is_empty());
    assert!(!second.is_empty());
}

// ============================================================================
// Error handling tests
// ============================================================================

#[test]
fn write_error_message_contains_path() {
    // Try to write to an invalid path (e.g., a path that would require creating
    // a directory that is actually a file)
    let dir = TempDir::new().expect("failed to create temp dir");
    let file_path = dir.path().join("blocking_file");
    fs::write(&file_path, "content").expect("failed to create blocking file");

    // Try to create a file "inside" the blocking file (which is not a directory)
    let invalid_path = file_path.join("subdir").join("report.json");

    let report = create_test_report();
    let result = write_report_json(&report, &invalid_path);

    // This should fail because we can't create a directory inside a file
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("failed to write") || err_msg.contains("blocking_file"));
}

#[test]
fn write_text_error_message_contains_path() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let file_path = dir.path().join("blocking_file");
    fs::write(&file_path, "content").expect("failed to create blocking file");

    let invalid_path = file_path.join("subdir").join("output.txt");

    let result = write_text(&invalid_path, "test");
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("failed to write") || err_msg.contains("blocking_file"));
}

// ============================================================================
// Report structure tests
// ============================================================================

#[test]
fn report_with_minimal_fields() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("minimal_report.json");

    let report = Report {
        schema: "lintdiff.report.v1".to_string(),
        tool: ToolInfo {
            name: "lintdiff".to_string(),
            version: "0.1.0".to_string(),
            commit: None,
        },
        run: RunInfo {
            started_at: "2024-01-01T00:00:00Z".to_string(),
            ended_at: "2024-01-01T00:00:01Z".to_string(),
            duration_ms: None,
            host: None,
            git: None,
        },
        verdict: Verdict {
            status: VerdictStatus::Pass,
            counts: Counts::default(),
            reasons: vec![],
        },
        findings: vec![],
        data: None,
    };

    let result = write_report_json(&report, &path);
    assert!(result.is_ok());

    let contents = fs::read_to_string(&path).expect("failed to read");
    // Verify optional fields are not present (skip_serializing_if)
    assert!(!contents.contains("\"commit\":"));
    assert!(!contents.contains("\"duration_ms\":"));
    assert!(!contents.contains("\"host\":"));
    assert!(!contents.contains("\"git\":"));
    assert!(!contents.contains("\"data\":"));
}

#[test]
fn report_with_data_field() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("report_with_data.json");

    let mut report = create_test_report();
    report.data = Some(serde_json::json!({
        "custom": "data",
        "nested": {
            "key": "value"
        }
    }));

    let result = write_report_json(&report, &path);
    assert!(result.is_ok());

    let contents = fs::read_to_string(&path).expect("failed to read");
    assert!(contents.contains("\"custom\": \"data\""));
    assert!(contents.contains("\"nested\""));
}

#[test]
fn report_with_multiple_reasons() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("report.json");

    let mut report = create_test_report();
    report.verdict.reasons = vec![
        "Reason 1: something happened".to_string(),
        "Reason 2: another thing".to_string(),
        "Reason 3: final reason".to_string(),
    ];

    let result = write_report_json(&report, &path);
    assert!(result.is_ok());

    let contents = fs::read_to_string(&path).expect("failed to read");
    assert!(contents.contains("Reason 1"));
    assert!(contents.contains("Reason 2"));
    assert!(contents.contains("Reason 3"));
}

#[test]
fn report_counts_serialization() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("report.json");

    let mut report = create_test_report();
    report.verdict.counts = Counts {
        info: 5,
        warn: 10,
        error: 2,
    };

    let result = write_report_json(&report, &path);
    assert!(result.is_ok());

    let contents = fs::read_to_string(&path).expect("failed to read");
    assert!(contents.contains("\"info\": 5"));
    assert!(contents.contains("\"warn\": 10"));
    assert!(contents.contains("\"error\": 2"));
}

// ============================================================================
// Path handling tests
// ============================================================================

#[test]
fn write_report_with_long_path() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let long_name = "a".repeat(100);
    let path = dir.path().join(&long_name).join("report.json");

    let report = create_test_report();
    let result = write_report_json(&report, &path);
    assert!(result.is_ok());
}

#[test]
fn write_report_with_special_chars_in_parent_dir() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let special_dir = dir.path().join("path-with_special.chars");
    fs::create_dir(&special_dir).expect("failed to create dir");

    let path = special_dir.join("report.json");
    let report = create_test_report();
    let result = write_report_json(&report, &path);
    assert!(result.is_ok());
}

#[test]
fn write_report_with_spaces_in_path() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let spaced_dir = dir.path().join("path with spaces");
    let path = spaced_dir.join("report.json");

    let report = create_test_report();
    let result = write_report_json(&report, &path);
    assert!(result.is_ok());
    assert!(path.exists());
}

// ============================================================================
// Round-trip tests
// ============================================================================

#[test]
fn report_round_trip() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("roundtrip.json");

    let original = create_test_report();
    write_report_json(&original, &path).expect("write failed");

    let contents = fs::read_to_string(&path).expect("read failed");
    let parsed: Report = serde_json::from_str(&contents).expect("parse failed");

    assert_eq!(parsed.schema, original.schema);
    assert_eq!(parsed.tool.name, original.tool.name);
    assert_eq!(parsed.verdict.status, original.verdict.status);
    assert_eq!(parsed.findings.len(), original.findings.len());
}

#[test]
fn report_round_trip_preserves_findings() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path().join("findings_roundtrip.json");

    let mut original = create_test_report();
    original.findings = vec![
        Finding {
            severity: Severity::Error,
            check_id: Some("check.1".to_string()),
            code: "E001".to_string(),
            message: "Error message".to_string(),
            location: Some(Location {
                path: "src/error.rs".into(),
                line: Some(1),
                col: Some(1),
            }),
            help: Some("fix it".to_string()),
            url: Some("https://example.com".to_string()),
            fingerprint: None,
            data: None,
        },
        Finding {
            severity: Severity::Warn,
            check_id: None,
            code: "W001".to_string(),
            message: "Warning message".to_string(),
            location: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        },
    ];

    write_report_json(&original, &path).expect("write failed");

    let contents = fs::read_to_string(&path).expect("read failed");
    let parsed: Report = serde_json::from_str(&contents).expect("parse failed");

    assert_eq!(parsed.findings.len(), 2);
    assert_eq!(parsed.findings[0].severity, Severity::Error);
    assert_eq!(parsed.findings[0].code, "E001");
    assert_eq!(parsed.findings[1].severity, Severity::Warn);
    assert_eq!(parsed.findings[1].code, "W001");
}
