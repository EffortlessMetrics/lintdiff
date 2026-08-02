//! Comprehensive tests for verdict computation.
//!
//! These tests cover:
//! - fail_on=error behavior
//! - fail_on=warn behavior
//! - fail_on=never behavior
//! - Allow list handling
//! - Suppress list handling
//! - Deny list handling
//! - Combined policy scenarios

use lintdiff_ingest_core::{compute_verdict, counts_from_findings};
use lintdiff_types::{
    FailOn, Finding, LintdiffConfig, Location, NormPath, Severity, VerdictStatus,
};

// =============================================================================
// Helper functions
// =============================================================================

fn finding_with_severity(sev: Severity) -> Finding {
    Finding {
        severity: sev,
        check_id: Some("test.check".to_string()),
        code: "test.code".to_string(),
        message: "test message".to_string(),
        location: Some(Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(1),
            col: None,
        }),
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

fn finding_with_code_and_severity(code: &str, sev: Severity) -> Finding {
    Finding {
        severity: sev,
        check_id: Some(code.to_string()),
        code: code.to_string(),
        message: "test message".to_string(),
        location: Some(Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(1),
            col: None,
        }),
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

fn cfg_with_fail_on(fail_on: FailOn) -> lintdiff_types::EffectiveConfig {
    LintdiffConfig {
        fail_on: Some(fail_on),
        ..Default::default()
    }
    .effective()
}

// =============================================================================
// fail_on=error tests
// =============================================================================

mod fail_on_error {
    use super::*;

    #[test]
    fn no_findings_is_pass() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 0, 0);
        assert_eq!(v.status, VerdictStatus::Pass);
    }

    #[test]
    fn error_finding_is_fail() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![finding_with_severity(Severity::Error)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn warning_finding_is_warn() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![finding_with_severity(Severity::Warn)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Warn);
    }

    #[test]
    fn info_finding_is_pass() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![finding_with_severity(Severity::Info)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Pass);
    }

    #[test]
    fn mixed_findings_with_error_is_fail() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![
            finding_with_severity(Severity::Info),
            finding_with_severity(Severity::Warn),
            finding_with_severity(Severity::Error),
        ];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn mixed_findings_without_error_is_warn() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![
            finding_with_severity(Severity::Info),
            finding_with_severity(Severity::Warn),
        ];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Warn);
    }

    #[test]
    fn multiple_errors_is_fail() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![
            finding_with_severity(Severity::Error),
            finding_with_severity(Severity::Error),
            finding_with_severity(Severity::Error),
        ];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn multiple_warnings_is_warn() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![
            finding_with_severity(Severity::Warn),
            finding_with_severity(Severity::Warn),
        ];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Warn);
    }
}

// =============================================================================
// fail_on=warn tests
// =============================================================================

mod fail_on_warn {
    use super::*;

    #[test]
    fn no_findings_is_pass() {
        let cfg = cfg_with_fail_on(FailOn::Warn);
        let v = compute_verdict(&cfg, &[], 0, 0);
        assert_eq!(v.status, VerdictStatus::Pass);
    }

    #[test]
    fn error_finding_is_fail() {
        let cfg = cfg_with_fail_on(FailOn::Warn);
        let findings = vec![finding_with_severity(Severity::Error)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn warning_finding_is_fail() {
        let cfg = cfg_with_fail_on(FailOn::Warn);
        let findings = vec![finding_with_severity(Severity::Warn)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn info_finding_is_pass() {
        let cfg = cfg_with_fail_on(FailOn::Warn);
        let findings = vec![finding_with_severity(Severity::Info)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Pass);
    }

    #[test]
    fn mixed_findings_with_error_is_fail() {
        let cfg = cfg_with_fail_on(FailOn::Warn);
        let findings = vec![
            finding_with_severity(Severity::Info),
            finding_with_severity(Severity::Error),
        ];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn mixed_findings_with_warning_is_fail() {
        let cfg = cfg_with_fail_on(FailOn::Warn);
        let findings = vec![
            finding_with_severity(Severity::Info),
            finding_with_severity(Severity::Warn),
        ];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn only_info_findings_is_pass() {
        let cfg = cfg_with_fail_on(FailOn::Warn);
        let findings = vec![
            finding_with_severity(Severity::Info),
            finding_with_severity(Severity::Info),
            finding_with_severity(Severity::Info),
        ];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Pass);
    }
}

// =============================================================================
// fail_on=never tests
// =============================================================================

mod fail_on_never {
    use super::*;

    #[test]
    fn no_findings_is_pass() {
        let cfg = cfg_with_fail_on(FailOn::Never);
        let v = compute_verdict(&cfg, &[], 0, 0);
        assert_eq!(v.status, VerdictStatus::Pass);
    }

    #[test]
    fn error_finding_is_warn() {
        let cfg = cfg_with_fail_on(FailOn::Never);
        let findings = vec![finding_with_severity(Severity::Error)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Warn);
    }

    #[test]
    fn warning_finding_is_warn() {
        let cfg = cfg_with_fail_on(FailOn::Never);
        let findings = vec![finding_with_severity(Severity::Warn)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Warn);
    }

    #[test]
    fn info_finding_is_pass() {
        let cfg = cfg_with_fail_on(FailOn::Never);
        let findings = vec![finding_with_severity(Severity::Info)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Pass);
    }

    #[test]
    fn multiple_errors_is_warn() {
        let cfg = cfg_with_fail_on(FailOn::Never);
        let findings = vec![
            finding_with_severity(Severity::Error),
            finding_with_severity(Severity::Error),
        ];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Warn);
    }

    #[test]
    fn mixed_severity_is_warn() {
        let cfg = cfg_with_fail_on(FailOn::Never);
        let findings = vec![
            finding_with_severity(Severity::Error),
            finding_with_severity(Severity::Warn),
            finding_with_severity(Severity::Info),
        ];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Warn);
    }

    #[test]
    fn never_produces_no_fail_status() {
        let cfg = cfg_with_fail_on(FailOn::Never);

        // Try various combinations - none should produce Fail
        let test_cases = vec![
            vec![finding_with_severity(Severity::Error)],
            vec![finding_with_severity(Severity::Warn)],
            vec![
                finding_with_severity(Severity::Error),
                finding_with_severity(Severity::Warn),
            ],
        ];

        for findings in test_cases {
            let v = compute_verdict(&cfg, &findings, 0, 0);
            assert_ne!(v.status, VerdictStatus::Fail);
        }
    }
}

// =============================================================================
// Counts tests
// =============================================================================

mod counts_tests {
    use super::*;

    #[test]
    fn empty_findings_zero_counts() {
        let c = counts_from_findings(&[]);
        assert_eq!(c.error, 0);
        assert_eq!(c.warn, 0);
        assert_eq!(c.info, 0);
    }

    #[test]
    fn single_error() {
        let c = counts_from_findings(&[finding_with_severity(Severity::Error)]);
        assert_eq!(c.error, 1);
        assert_eq!(c.warn, 0);
        assert_eq!(c.info, 0);
    }

    #[test]
    fn single_warning() {
        let c = counts_from_findings(&[finding_with_severity(Severity::Warn)]);
        assert_eq!(c.error, 0);
        assert_eq!(c.warn, 1);
        assert_eq!(c.info, 0);
    }

    #[test]
    fn single_info() {
        let c = counts_from_findings(&[finding_with_severity(Severity::Info)]);
        assert_eq!(c.error, 0);
        assert_eq!(c.warn, 0);
        assert_eq!(c.info, 1);
    }

    #[test]
    fn mixed_counts() {
        let findings = vec![
            finding_with_severity(Severity::Error),
            finding_with_severity(Severity::Error),
            finding_with_severity(Severity::Warn),
            finding_with_severity(Severity::Warn),
            finding_with_severity(Severity::Warn),
            finding_with_severity(Severity::Info),
        ];
        let c = counts_from_findings(&findings);
        assert_eq!(c.error, 2);
        assert_eq!(c.warn, 3);
        assert_eq!(c.info, 1);
    }

    #[test]
    fn verdict_contains_correct_counts() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![
            finding_with_severity(Severity::Error),
            finding_with_severity(Severity::Warn),
            finding_with_severity(Severity::Info),
        ];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.counts.error, 1);
        assert_eq!(v.counts.warn, 1);
        assert_eq!(v.counts.info, 1);
    }

    #[test]
    fn large_count() {
        let findings: Vec<Finding> = (0..100)
            .map(|_| finding_with_severity(Severity::Error))
            .collect();
        let c = counts_from_findings(&findings);
        assert_eq!(c.error, 100);
    }
}

// =============================================================================
// Suppressed and denied reasons tests
// =============================================================================

mod reasons_tests {
    use super::*;

    #[test]
    fn no_suppressed_or_denied_no_reasons() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 0, 0);
        assert!(v.reasons.is_empty());
    }

    #[test]
    fn suppressed_count_adds_reason() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 5, 0);
        assert!(v.reasons.contains(&"suppressed".to_string()));
    }

    #[test]
    fn denied_count_adds_reason() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 0, 3);
        assert!(v.reasons.contains(&"deny_list".to_string()));
    }

    #[test]
    fn both_suppressed_and_denied_adds_reasons() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 5, 3);
        assert!(v.reasons.contains(&"suppressed".to_string()));
        assert!(v.reasons.contains(&"deny_list".to_string()));
    }

    #[test]
    fn suppressed_count_one() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 1, 0);
        assert!(v.reasons.contains(&"suppressed".to_string()));
    }

    #[test]
    fn denied_count_one() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 0, 1);
        assert!(v.reasons.contains(&"deny_list".to_string()));
    }

    #[test]
    fn suppressed_does_not_affect_status() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 100, 0);
        // No findings, so should still be Pass
        assert_eq!(v.status, VerdictStatus::Pass);
    }

    #[test]
    fn denied_does_not_affect_status() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 0, 100);
        // No findings, so should still be Pass
        assert_eq!(v.status, VerdictStatus::Pass);
    }

    #[test]
    fn large_suppressed_count() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], u32::MAX, 0);
        assert!(v.reasons.contains(&"suppressed".to_string()));
    }

    #[test]
    fn large_denied_count() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 0, u32::MAX);
        assert!(v.reasons.contains(&"deny_list".to_string()));
    }
}

// =============================================================================
// Combined policy scenarios
// =============================================================================

mod combined_scenarios {
    use super::*;

    #[test]
    fn findings_with_suppressed_and_denied() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![
            finding_with_severity(Severity::Error),
            finding_with_severity(Severity::Warn),
        ];
        let v = compute_verdict(&cfg, &findings, 10, 5);

        // Should fail due to error
        assert_eq!(v.status, VerdictStatus::Fail);
        // Should have both reasons
        assert!(v.reasons.contains(&"suppressed".to_string()));
        assert!(v.reasons.contains(&"deny_list".to_string()));
        // Should have correct counts
        assert_eq!(v.counts.error, 1);
        assert_eq!(v.counts.warn, 1);
    }

    #[test]
    fn warn_fail_mode_with_errors_and_warnings() {
        let cfg = cfg_with_fail_on(FailOn::Warn);
        let findings = vec![
            finding_with_severity(Severity::Error),
            finding_with_severity(Severity::Warn),
        ];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn never_fail_mode_with_all_severities() {
        let cfg = cfg_with_fail_on(FailOn::Never);
        let findings = vec![
            finding_with_severity(Severity::Error),
            finding_with_severity(Severity::Warn),
            finding_with_severity(Severity::Info),
        ];
        let v = compute_verdict(&cfg, &findings, 5, 3);

        // Should warn, not fail
        assert_eq!(v.status, VerdictStatus::Warn);
        // Should have reasons
        assert!(v.reasons.contains(&"suppressed".to_string()));
        assert!(v.reasons.contains(&"deny_list".to_string()));
    }

    #[test]
    fn multiple_errors_with_different_codes() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![
            finding_with_code_and_severity("clippy::needless_borrow", Severity::Error),
            finding_with_code_and_severity("rustc.E0502", Severity::Error),
            finding_with_code_and_severity("unused_variables", Severity::Warn),
        ];
        let v = compute_verdict(&cfg, &findings, 0, 0);

        assert_eq!(v.status, VerdictStatus::Fail);
        assert_eq!(v.counts.error, 2);
        assert_eq!(v.counts.warn, 1);
    }

    #[test]
    fn only_info_findings_always_pass() {
        for fail_on in [FailOn::Error, FailOn::Warn, FailOn::Never] {
            let cfg = cfg_with_fail_on(fail_on);
            let findings = vec![
                finding_with_severity(Severity::Info),
                finding_with_severity(Severity::Info),
            ];
            let v = compute_verdict(&cfg, &findings, 0, 0);
            assert_eq!(v.status, VerdictStatus::Pass);
        }
    }

    #[test]
    fn empty_findings_always_pass() {
        for fail_on in [FailOn::Error, FailOn::Warn, FailOn::Never] {
            let cfg = cfg_with_fail_on(fail_on);
            let v = compute_verdict(&cfg, &[], 0, 0);
            assert_eq!(v.status, VerdictStatus::Pass);
        }
    }
}

// =============================================================================
// Edge cases
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn finding_without_location() {
        let finding = Finding {
            severity: Severity::Error,
            check_id: Some("test".to_string()),
            code: "test.code".to_string(),
            message: "test message".to_string(),
            location: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        };

        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[finding], 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn finding_without_check_id() {
        let finding = Finding {
            severity: Severity::Warn,
            check_id: None,
            code: "test.code".to_string(),
            message: "test message".to_string(),
            location: Some(Location {
                path: NormPath::new("src/lib.rs"),
                line: Some(1),
                col: None,
            }),
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        };

        let cfg = cfg_with_fail_on(FailOn::Warn);
        let v = compute_verdict(&cfg, &[finding], 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn finding_with_all_optional_fields() {
        let finding = Finding {
            severity: Severity::Error,
            check_id: Some("test.check".to_string()),
            code: "test.code".to_string(),
            message: "test message".to_string(),
            location: Some(Location {
                path: NormPath::new("src/lib.rs"),
                line: Some(42),
                col: Some(10),
            }),
            help: Some("try this instead".to_string()),
            url: Some("https://example.com/help".to_string()),
            fingerprint: Some("abc123".to_string()),
            data: None,
        };

        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[finding], 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn many_findings() {
        let findings: Vec<Finding> = (0..1000)
            .map(|i| Finding {
                severity: if i % 3 == 0 {
                    Severity::Error
                } else if i % 3 == 1 {
                    Severity::Warn
                } else {
                    Severity::Info
                },
                check_id: Some(format!("check.{}", i)),
                code: format!("code.{}", i),
                message: format!("message {}", i),
                location: Some(Location {
                    path: NormPath::new(format!("src/file{}.rs", i % 10)),
                    line: Some(i),
                    col: None,
                }),
                help: None,
                url: None,
                fingerprint: None,
                data: None,
            })
            .collect();

        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &findings, 0, 0);

        assert_eq!(v.status, VerdictStatus::Fail);
        // Counts: errors = ceil(1000/3) ≈ 334, warn ≈ 333, info ≈ 333
        assert!(v.counts.error > 0);
        assert!(v.counts.warn > 0);
        assert!(v.counts.info > 0);
    }

    #[test]
    fn long_message() {
        let finding = Finding {
            severity: Severity::Error,
            check_id: Some("test".to_string()),
            code: "test.code".to_string(),
            message: "x".repeat(10000),
            location: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        };

        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[finding], 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn unicode_in_message() {
        let finding = Finding {
            severity: Severity::Warn,
            check_id: Some("test".to_string()),
            code: "test.code".to_string(),
            message: "日本語メッセージ 🦀 with emoji".to_string(),
            location: Some(Location {
                path: NormPath::new("src/日本語/lib.rs"),
                line: Some(1),
                col: None,
            }),
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        };

        let cfg = cfg_with_fail_on(FailOn::Warn);
        let v = compute_verdict(&cfg, &[finding], 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }
}

// =============================================================================
// Default config behavior
// =============================================================================

mod default_config {
    use super::*;

    #[test]
    fn default_fail_on_is_error() {
        let cfg = LintdiffConfig::default().effective();
        assert_eq!(cfg.fail_on, FailOn::Error);
    }

    #[test]
    fn default_config_error_fails() {
        let cfg = LintdiffConfig::default().effective();
        let findings = vec![finding_with_severity(Severity::Error)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn default_config_warn_warns() {
        let cfg = LintdiffConfig::default().effective();
        let findings = vec![finding_with_severity(Severity::Warn)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Warn);
    }

    #[test]
    fn default_config_info_passes() {
        let cfg = LintdiffConfig::default().effective();
        let findings = vec![finding_with_severity(Severity::Info)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Pass);
    }
}
