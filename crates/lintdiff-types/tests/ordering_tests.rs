//! Comprehensive tests for finding ordering.
//!
//! Tests cover:
//! - Deterministic ordering contract
//! - Severity ranking (error > warn > info)
//! - Path ordering (ascending)
//! - Line ordering (ascending, missing last)
//! - Code ordering (ascending)
//! - Message ordering (ascending)
//! - Edge cases (equal findings, different fields)

use lintdiff_types::*;

// =============================================================================
// Helper Functions
// =============================================================================

fn make_finding(
    severity: Severity,
    path: &str,
    line: Option<u32>,
    code: &str,
    message: &str,
) -> Finding {
    Finding {
        severity,
        code: code.to_string(),
        message: message.to_string(),
        location: Some(Location {
            path: NormPath::new(path),
            line,
            col: None,
        }),
        check_id: None,
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

fn make_finding_no_location(severity: Severity, code: &str, message: &str) -> Finding {
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
// Severity Ordering Tests
// =============================================================================

mod severity_ordering_tests {
    use super::*;

    #[test]
    fn error_comes_before_warn() {
        let error = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "error");
        let warn = make_finding(Severity::Warn, "src/a.rs", Some(1), "E001", "error");

        let mut findings = vec![warn.clone(), error.clone()];
        sort_findings(&mut findings);

        assert_eq!(findings[0].severity, Severity::Error);
        assert_eq!(findings[1].severity, Severity::Warn);
    }

    #[test]
    fn warn_comes_before_info() {
        let warn = make_finding(Severity::Warn, "src/a.rs", Some(1), "E001", "msg");
        let info = make_finding(Severity::Info, "src/a.rs", Some(1), "E001", "msg");

        let mut findings = vec![info.clone(), warn.clone()];
        sort_findings(&mut findings);

        assert_eq!(findings[0].severity, Severity::Warn);
        assert_eq!(findings[1].severity, Severity::Info);
    }

    #[test]
    fn error_comes_before_info() {
        let error = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let info = make_finding(Severity::Info, "src/a.rs", Some(1), "E001", "msg");

        let mut findings = vec![info.clone(), error.clone()];
        sort_findings(&mut findings);

        assert_eq!(findings[0].severity, Severity::Error);
        assert_eq!(findings[1].severity, Severity::Info);
    }

    #[test]
    fn all_severities_sorted_correctly() {
        let error1 = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "error 1");
        let error2 = make_finding(Severity::Error, "src/b.rs", Some(1), "E002", "error 2");
        let warn1 = make_finding(Severity::Warn, "src/a.rs", Some(1), "W001", "warn 1");
        let warn2 = make_finding(Severity::Warn, "src/b.rs", Some(1), "W002", "warn 2");
        let info1 = make_finding(Severity::Info, "src/a.rs", Some(1), "I001", "info 1");
        let info2 = make_finding(Severity::Info, "src/b.rs", Some(1), "I002", "info 2");

        let mut findings = vec![
            info2.clone(),
            warn2.clone(),
            error2.clone(),
            info1.clone(),
            warn1.clone(),
            error1.clone(),
        ];
        sort_findings(&mut findings);

        // Errors first
        assert_eq!(findings[0].severity, Severity::Error);
        assert_eq!(findings[1].severity, Severity::Error);
        // Then warns
        assert_eq!(findings[2].severity, Severity::Warn);
        assert_eq!(findings[3].severity, Severity::Warn);
        // Then infos
        assert_eq!(findings[4].severity, Severity::Info);
        assert_eq!(findings[5].severity, Severity::Info);
    }
}

// =============================================================================
// Path Ordering Tests
// =============================================================================

mod path_ordering_tests {
    use super::*;

    #[test]
    fn path_ordered_ascending() {
        let a = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let b = make_finding(Severity::Error, "src/b.rs", Some(1), "E001", "msg");

        let mut findings = vec![b.clone(), a.clone()];
        sort_findings(&mut findings);

        assert_eq!(
            findings[0].location.as_ref().unwrap().path.as_str(),
            "src/a.rs"
        );
        assert_eq!(
            findings[1].location.as_ref().unwrap().path.as_str(),
            "src/b.rs"
        );
    }

    #[test]
    fn path_ordering_with_subdirectories() {
        let a = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let sub_a = make_finding(Severity::Error, "src/sub/a.rs", Some(1), "E001", "msg");
        let b = make_finding(Severity::Error, "src/b.rs", Some(1), "E001", "msg");

        let mut findings = vec![sub_a.clone(), b.clone(), a.clone()];
        sort_findings(&mut findings);

        // Alphabetical: src/a.rs < src/b.rs < src/sub/a.rs
        assert_eq!(
            findings[0].location.as_ref().unwrap().path.as_str(),
            "src/a.rs"
        );
        assert_eq!(
            findings[1].location.as_ref().unwrap().path.as_str(),
            "src/b.rs"
        );
        assert_eq!(
            findings[2].location.as_ref().unwrap().path.as_str(),
            "src/sub/a.rs"
        );
    }

    #[test]
    fn path_ordering_with_different_depths() {
        let root = make_finding(Severity::Error, "lib.rs", Some(1), "E001", "msg");
        let deep = make_finding(
            Severity::Error,
            "src/deep/nested/file.rs",
            Some(1),
            "E001",
            "msg",
        );

        let mut findings = vec![deep.clone(), root.clone()];
        sort_findings(&mut findings);

        // lib.rs < src/deep/nested/file.rs (alphabetically)
        assert_eq!(
            findings[0].location.as_ref().unwrap().path.as_str(),
            "lib.rs"
        );
        assert_eq!(
            findings[1].location.as_ref().unwrap().path.as_str(),
            "src/deep/nested/file.rs"
        );
    }

    #[test]
    fn missing_path_comes_first_in_path_order() {
        // Empty path "" should come before any actual path
        let no_loc = make_finding_no_location(Severity::Error, "E001", "msg");
        let with_path = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");

        let mut findings = vec![with_path.clone(), no_loc.clone()];
        sort_findings(&mut findings);

        // Empty path comes first alphabetically
        assert!(findings[0].location.is_none());
        assert!(findings[1].location.is_some());
    }
}

// =============================================================================
// Line Ordering Tests
// =============================================================================

mod line_ordering_tests {
    use super::*;

    #[test]
    fn line_ordered_ascending() {
        let line1 = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let line10 = make_finding(Severity::Error, "src/a.rs", Some(10), "E001", "msg");

        let mut findings = vec![line10.clone(), line1.clone()];
        sort_findings(&mut findings);

        assert_eq!(findings[0].location.as_ref().unwrap().line, Some(1));
        assert_eq!(findings[1].location.as_ref().unwrap().line, Some(10));
    }

    #[test]
    fn missing_line_goes_last() {
        let with_line = make_finding(Severity::Error, "src/a.rs", Some(100), "E001", "msg");
        let no_line = make_finding(Severity::Error, "src/a.rs", None, "E001", "msg");

        let mut findings = vec![no_line.clone(), with_line.clone()];
        sort_findings(&mut findings);

        // Finding with line comes first
        assert_eq!(findings[0].location.as_ref().unwrap().line, Some(100));
        assert_eq!(findings[1].location.as_ref().unwrap().line, None);
    }

    #[test]
    fn multiple_missing_lines_ordered_by_other_fields() {
        let no_line_a = make_finding(Severity::Error, "src/a.rs", None, "E001", "msg");
        let no_line_b = make_finding(Severity::Error, "src/b.rs", None, "E001", "msg");

        let mut findings = vec![no_line_b.clone(), no_line_a.clone()];
        sort_findings(&mut findings);

        // Both have missing lines, so ordered by path
        assert_eq!(
            findings[0].location.as_ref().unwrap().path.as_str(),
            "src/a.rs"
        );
        assert_eq!(
            findings[1].location.as_ref().unwrap().path.as_str(),
            "src/b.rs"
        );
    }

    #[test]
    fn line_zero_vs_missing() {
        // Line 0 is technically valid but unusual; missing goes after
        let line0 = make_finding(Severity::Error, "src/a.rs", Some(0), "E001", "msg");
        let no_line = make_finding(Severity::Error, "src/a.rs", None, "E001", "msg");

        let mut findings = vec![no_line.clone(), line0.clone()];
        sort_findings(&mut findings);

        assert_eq!(findings[0].location.as_ref().unwrap().line, Some(0));
        assert_eq!(findings[1].location.as_ref().unwrap().line, None);
    }
}

// =============================================================================
// Code Ordering Tests
// =============================================================================

mod code_ordering_tests {
    use super::*;

    #[test]
    fn code_ordered_ascending() {
        let a = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let b = make_finding(Severity::Error, "src/a.rs", Some(1), "E002", "msg");

        let mut findings = vec![b.clone(), a.clone()];
        sort_findings(&mut findings);

        assert_eq!(findings[0].code, "E001");
        assert_eq!(findings[1].code, "E002");
    }

    #[test]
    fn code_alphanumeric_ordering() {
        let e001 = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let e010 = make_finding(Severity::Error, "src/a.rs", Some(1), "E010", "msg");
        let e100 = make_finding(Severity::Error, "src/a.rs", Some(1), "E100", "msg");

        let mut findings = vec![e100.clone(), e001.clone(), e010.clone()];
        sort_findings(&mut findings);

        // String ordering: E001 < E010 < E100
        assert_eq!(findings[0].code, "E001");
        assert_eq!(findings[1].code, "E010");
        assert_eq!(findings[2].code, "E100");
    }

    #[test]
    fn code_case_sensitive() {
        let lower = make_finding(Severity::Error, "src/a.rs", Some(1), "e001", "msg");
        let upper = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");

        let mut findings = vec![lower.clone(), upper.clone()];
        sort_findings(&mut findings);

        // Uppercase comes before lowercase in ASCII
        assert_eq!(findings[0].code, "E001");
        assert_eq!(findings[1].code, "e001");
    }
}

// =============================================================================
// Message Ordering Tests
// =============================================================================

mod message_ordering_tests {
    use super::*;

    #[test]
    fn message_ordered_ascending() {
        let a = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "aaa");
        let b = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "bbb");

        let mut findings = vec![b.clone(), a.clone()];
        sort_findings(&mut findings);

        assert_eq!(findings[0].message, "aaa");
        assert_eq!(findings[1].message, "bbb");
    }

    #[test]
    fn message_is_final_tiebreaker() {
        // All other fields equal, message decides
        let a = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "apple");
        let b = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "banana");
        let c = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "cherry");

        let mut findings = vec![c.clone(), a.clone(), b.clone()];
        sort_findings(&mut findings);

        assert_eq!(findings[0].message, "apple");
        assert_eq!(findings[1].message, "banana");
        assert_eq!(findings[2].message, "cherry");
    }
}

// =============================================================================
// Stability Tests
// =============================================================================

mod stability_tests {
    use super::*;

    #[test]
    fn sort_is_stable_for_identical_findings() {
        let a = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let b = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");

        let mut findings = vec![a.clone(), b.clone()];
        sort_findings(&mut findings);

        // Both are equal, order doesn't matter but should not panic
        assert_eq!(findings.len(), 2);
    }

    #[test]
    fn empty_findings_list() {
        let mut findings: Vec<Finding> = vec![];
        sort_findings(&mut findings);
        assert!(findings.is_empty());
    }

    #[test]
    fn single_finding() {
        let finding = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let mut findings = vec![finding.clone()];
        sort_findings(&mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "E001");
    }

    #[test]
    fn many_findings_sorted_deterministically() {
        let mut findings: Vec<Finding> = (0..100)
            .map(|i| {
                let severity = match i % 3 {
                    0 => Severity::Error,
                    1 => Severity::Warn,
                    _ => Severity::Info,
                };
                make_finding(
                    severity,
                    &format!("src/file{}.rs", i),
                    Some(i),
                    &format!("CODE{:03}", i),
                    &format!("Message {}", i),
                )
            })
            .collect();

        // Reverse the order
        findings.reverse();

        // Sort
        sort_findings(&mut findings);

        // Verify errors come first
        let error_count = findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count();
        let warn_count = findings
            .iter()
            .filter(|f| f.severity == Severity::Warn)
            .count();
        let _info_count = findings
            .iter()
            .filter(|f| f.severity == Severity::Info)
            .count();

        // Errors should be in the first third (roughly)
        for finding in findings.iter().take(error_count) {
            assert_eq!(finding.severity, Severity::Error);
        }

        // Warns in the middle third
        for finding in findings.iter().skip(error_count).take(warn_count) {
            assert_eq!(finding.severity, Severity::Warn);
        }

        // Infos in the last third
        for finding in findings.iter().skip(error_count + warn_count) {
            assert_eq!(finding.severity, Severity::Info);
        }
    }
}

// =============================================================================
// Comparator Function Tests
// =============================================================================

mod comparator_tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn comparator_error_less_than_warn() {
        let error = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let warn = make_finding(Severity::Warn, "src/a.rs", Some(1), "E001", "msg");

        assert_eq!(sort_findings_cmp(&error, &warn), Ordering::Less);
        assert_eq!(sort_findings_cmp(&warn, &error), Ordering::Greater);
    }

    #[test]
    fn comparator_equal_findings() {
        let a = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let b = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");

        assert_eq!(sort_findings_cmp(&a, &b), Ordering::Equal);
    }

    #[test]
    fn comparator_uses_all_fields() {
        // Different severity
        let error = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let warn = make_finding(Severity::Warn, "src/a.rs", Some(1), "E001", "msg");
        assert_eq!(sort_findings_cmp(&error, &warn), Ordering::Less);

        // Same severity, different path
        let path_a = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let path_b = make_finding(Severity::Error, "src/b.rs", Some(1), "E001", "msg");
        assert_eq!(sort_findings_cmp(&path_a, &path_b), Ordering::Less);

        // Same path, different line
        let line_1 = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");
        let line_2 = make_finding(Severity::Error, "src/a.rs", Some(2), "E001", "msg");
        assert_eq!(sort_findings_cmp(&line_1, &line_2), Ordering::Less);

        // Same line, different code
        let code_a = make_finding(Severity::Error, "src/a.rs", Some(1), "A", "msg");
        let code_b = make_finding(Severity::Error, "src/a.rs", Some(1), "B", "msg");
        assert_eq!(sort_findings_cmp(&code_a, &code_b), Ordering::Less);

        // Same code, different message
        let msg_a = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "a");
        let msg_b = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "b");
        assert_eq!(sort_findings_cmp(&msg_a, &msg_b), Ordering::Less);
    }
}

// =============================================================================
// Edge Cases
// =============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn unicode_in_path() {
        let a = make_finding(Severity::Error, "src/日本語.rs", Some(1), "E001", "msg");
        let b = make_finding(Severity::Error, "src/中文.rs", Some(1), "E001", "msg");

        let mut findings = vec![a.clone(), b.clone()];
        sort_findings(&mut findings);

        // Should not panic; ordering based on Unicode code points
        assert_eq!(findings.len(), 2);
    }

    #[test]
    fn unicode_in_message() {
        let a = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "Error: 错误");
        let b = make_finding(
            Severity::Error,
            "src/a.rs",
            Some(1),
            "E001",
            "Error: エラー",
        );

        let mut findings = vec![a.clone(), b.clone()];
        sort_findings(&mut findings);

        assert_eq!(findings.len(), 2);
    }

    #[test]
    fn very_long_paths() {
        let long_path = "src/".repeat(100) + "file.rs";
        let a = make_finding(Severity::Error, &long_path, Some(1), "E001", "msg");
        let b = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "msg");

        let mut findings = vec![a.clone(), b.clone()];
        sort_findings(&mut findings);

        // Shorter path comes first
        assert_eq!(
            findings[0].location.as_ref().unwrap().path.as_str(),
            "src/a.rs"
        );
    }

    #[test]
    fn very_long_messages() {
        let long_msg = "x".repeat(10000);
        let a = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", &long_msg);
        let b = make_finding(Severity::Error, "src/a.rs", Some(1), "E001", "short");

        let mut findings = vec![a.clone(), b.clone()];
        sort_findings(&mut findings);

        // "short" < "xxx..."
        assert_eq!(findings[0].message, "short");
    }

    #[test]
    fn special_characters_in_code() {
        let a = make_finding(Severity::Error, "src/a.rs", Some(1), "E-001", "msg");
        let b = make_finding(Severity::Error, "src/a.rs", Some(1), "E_001", "msg");

        let mut findings = vec![a.clone(), b.clone()];
        sort_findings(&mut findings);

        // '-' < '_' in ASCII
        assert_eq!(findings[0].code, "E-001");
        assert_eq!(findings[1].code, "E_001");
    }

    #[test]
    fn max_line_value() {
        let max_line = make_finding(Severity::Error, "src/a.rs", Some(u32::MAX), "E001", "msg");
        let no_line = make_finding(Severity::Error, "src/a.rs", None, "E001", "msg");

        let mut findings = vec![no_line.clone(), max_line.clone()];
        sort_findings(&mut findings);

        // Both use u32::MAX internally for missing line, so they're equal by line
        // Order determined by next field (code), which is the same
        // Then by message, which is the same
        assert_eq!(findings.len(), 2);
    }
}
