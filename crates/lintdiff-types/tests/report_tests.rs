//! Comprehensive tests for report structures.
//!
//! Tests cover:
//! - Report structure validation
//! - ToolInfo structure
//! - RunInfo structure
//! - HostInfo and GitInfo structures
//! - Verdict and VerdictStatus
//! - Finding and Severity
//! - Location structure
//! - DiagnosticDisposition and Disposition
//! - ExplainSummary
//! - Serialization format
//! - Field defaults

use lintdiff_types::*;

// =============================================================================
// Constants Tests
// =============================================================================

mod constants_tests {
    use super::*;

    #[test]
    fn schema_id_value() {
        assert_eq!(SCHEMA_ID, "lintdiff.report.v1");
    }

    #[test]
    fn tool_name_value() {
        assert_eq!(TOOL_NAME, "lintdiff");
    }

    #[test]
    fn check_diagnostics_on_diff_value() {
        assert_eq!(CHECK_DIAGNOSTICS_ON_DIFF, "diagnostics.on_diff");
    }
}

// =============================================================================
// Report Tests
// =============================================================================

mod report_tests {
    use super::*;
    use serde_json::Value;

    fn make_minimal_report() -> Report {
        Report {
            schema: SCHEMA_ID.to_string(),
            tool: ToolInfo {
                name: TOOL_NAME.to_string(),
                version: "1.0.0".to_string(),
                commit: None,
            },
            run: RunInfo {
                started_at: "2024-01-01T00:00:00Z".to_string(),
                ended_at: "2024-01-01T00:00:01Z".to_string(),
                duration_ms: Some(1000),
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
        }
    }

    #[test]
    fn minimal_report_serializable() {
        let report = make_minimal_report();
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("schema"));
        assert!(json.contains("tool"));
        assert!(json.contains("run"));
        assert!(json.contains("verdict"));
    }

    #[test]
    fn minimal_report_deserializable() {
        let json = r#"{
            "schema": "lintdiff.report.v1",
            "tool": {"name": "lintdiff", "version": "1.0.0"},
            "run": {
                "started_at": "2024-01-01T00:00:00Z",
                "ended_at": "2024-01-01T00:00:01Z"
            },
            "verdict": {
                "status": "pass",
                "counts": {"info": 0, "warn": 0, "error": 0}
            }
        }"#;
        let report: Report = serde_json::from_str(json).unwrap();
        assert_eq!(report.schema, SCHEMA_ID);
        assert_eq!(report.tool.name, TOOL_NAME);
    }

    #[test]
    fn report_with_findings() {
        let report = Report {
            findings: vec![Finding {
                severity: Severity::Error,
                code: "E001".to_string(),
                message: "test error".to_string(),
                location: None,
                check_id: None,
                help: None,
                url: None,
                fingerprint: None,
                data: None,
            }],
            ..make_minimal_report()
        };
        assert_eq!(report.findings.len(), 1);
    }

    #[test]
    fn report_with_data() {
        let data = serde_json::json!({"custom": "field"});
        let report = Report {
            data: Some(data.clone()),
            ..make_minimal_report()
        };
        assert_eq!(report.data, Some(data));
    }

    #[test]
    fn report_findings_default_empty() {
        let json = r#"{
            "schema": "lintdiff.report.v1",
            "tool": {"name": "lintdiff", "version": "1.0.0"},
            "run": {
                "started_at": "2024-01-01T00:00:00Z",
                "ended_at": "2024-01-01T00:00:01Z"
            },
            "verdict": {
                "status": "pass",
                "counts": {"info": 0, "warn": 0, "error": 0}
            }
        }"#;
        let report: Report = serde_json::from_str(json).unwrap();
        assert!(report.findings.is_empty());
    }

    #[test]
    fn report_data_skipped_when_none() {
        let report = make_minimal_report();
        let json = serde_json::to_string(&report).unwrap();
        // data should not appear when None
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed.as_object().unwrap().contains_key("data"));
    }

    #[test]
    fn clone() {
        let report = make_minimal_report();
        let cloned = report.clone();
        assert_eq!(report.schema, cloned.schema);
    }

    #[test]
    fn debug_format() {
        let report = make_minimal_report();
        let debug = format!("{:?}", report);
        assert!(debug.contains("Report"));
    }
}

// =============================================================================
// ToolInfo Tests
// =============================================================================

mod tool_info_tests {
    use super::*;

    #[test]
    fn basic_tool_info() {
        let tool = ToolInfo {
            name: "lintdiff".to_string(),
            version: "1.0.0".to_string(),
            commit: None,
        };
        assert_eq!(tool.name, "lintdiff");
        assert_eq!(tool.version, "1.0.0");
        assert!(tool.commit.is_none());
    }

    #[test]
    fn tool_info_with_commit() {
        let tool = ToolInfo {
            name: "lintdiff".to_string(),
            version: "1.0.0".to_string(),
            commit: Some("abc123".to_string()),
        };
        assert_eq!(tool.commit, Some("abc123".to_string()));
    }

    #[test]
    fn serialize() {
        let tool = ToolInfo {
            name: "lintdiff".to_string(),
            version: "1.0.0".to_string(),
            commit: None,
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("lintdiff"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn serialize_with_commit() {
        let tool = ToolInfo {
            name: "lintdiff".to_string(),
            version: "1.0.0".to_string(),
            commit: Some("abc123".to_string()),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("commit"));
        assert!(json.contains("abc123"));
    }

    #[test]
    fn deserialize() {
        let json = r#"{"name": "lintdiff", "version": "1.0.0"}"#;
        let tool: ToolInfo = serde_json::from_str(json).unwrap();
        assert_eq!(tool.name, "lintdiff");
        assert_eq!(tool.version, "1.0.0");
        assert!(tool.commit.is_none());
    }

    #[test]
    fn commit_skipped_when_none() {
        let tool = ToolInfo {
            name: "lintdiff".to_string(),
            version: "1.0.0".to_string(),
            commit: None,
        };
        let json = serde_json::to_string(&tool).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed.as_object().unwrap().contains_key("commit"));
    }

    #[test]
    fn clone() {
        let tool = ToolInfo {
            name: "lintdiff".to_string(),
            version: "1.0.0".to_string(),
            commit: None,
        };
        let cloned = tool.clone();
        assert_eq!(tool.name, cloned.name);
    }

    #[test]
    fn debug_format() {
        let tool = ToolInfo {
            name: "lintdiff".to_string(),
            version: "1.0.0".to_string(),
            commit: None,
        };
        let debug = format!("{:?}", tool);
        assert!(debug.contains("ToolInfo"));
    }
}

// =============================================================================
// RunInfo Tests
// =============================================================================

mod run_info_tests {
    use super::*;

    fn make_minimal_run_info() -> RunInfo {
        RunInfo {
            started_at: "2024-01-01T00:00:00Z".to_string(),
            ended_at: "2024-01-01T00:00:01Z".to_string(),
            duration_ms: None,
            host: None,
            git: None,
        }
    }

    #[test]
    fn basic_run_info() {
        let run = make_minimal_run_info();
        assert_eq!(run.started_at, "2024-01-01T00:00:00Z");
        assert_eq!(run.ended_at, "2024-01-01T00:00:01Z");
    }

    #[test]
    fn run_info_with_duration() {
        let run = RunInfo {
            duration_ms: Some(1000),
            ..make_minimal_run_info()
        };
        assert_eq!(run.duration_ms, Some(1000));
    }

    #[test]
    fn run_info_with_host() {
        let host = HostInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        let run = RunInfo {
            host: Some(host.clone()),
            ..make_minimal_run_info()
        };
        assert_eq!(run.host, Some(host));
    }

    #[test]
    fn run_info_with_git() {
        let git = GitInfo {
            repo: Some("https://github.com/example/repo".to_string()),
            base_ref: None,
            head_ref: None,
            base_sha: None,
            head_sha: None,
            merge_base: None,
        };
        let run = RunInfo {
            git: Some(git.clone()),
            ..make_minimal_run_info()
        };
        assert_eq!(run.git, Some(git));
    }

    #[test]
    fn serialize() {
        let run = make_minimal_run_info();
        let json = serde_json::to_string(&run).unwrap();
        assert!(json.contains("started_at"));
        assert!(json.contains("ended_at"));
    }

    #[test]
    fn deserialize() {
        let json = r#"{
            "started_at": "2024-01-01T00:00:00Z",
            "ended_at": "2024-01-01T00:00:01Z"
        }"#;
        let run: RunInfo = serde_json::from_str(json).unwrap();
        assert_eq!(run.started_at, "2024-01-01T00:00:00Z");
        assert!(run.duration_ms.is_none());
    }

    #[test]
    fn optional_fields_skipped_when_none() {
        let run = make_minimal_run_info();
        let json = serde_json::to_string(&run).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed.as_object().unwrap().contains_key("duration_ms"));
        assert!(!parsed.as_object().unwrap().contains_key("host"));
        assert!(!parsed.as_object().unwrap().contains_key("git"));
    }

    #[test]
    fn clone() {
        let run = make_minimal_run_info();
        let cloned = run.clone();
        assert_eq!(run.started_at, cloned.started_at);
    }

    #[test]
    fn debug_format() {
        let run = make_minimal_run_info();
        let debug = format!("{:?}", run);
        assert!(debug.contains("RunInfo"));
    }
}

// =============================================================================
// HostInfo Tests
// =============================================================================

mod host_info_tests {
    use super::*;

    #[test]
    fn basic_host_info() {
        let host = HostInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        assert_eq!(host.os, "linux");
        assert_eq!(host.arch, "x86_64");
    }

    #[test]
    fn serialize() {
        let host = HostInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        let json = serde_json::to_string(&host).unwrap();
        assert!(json.contains("linux"));
        assert!(json.contains("x86_64"));
    }

    #[test]
    fn deserialize() {
        let json = r#"{"os": "windows", "arch": "aarch64"}"#;
        let host: HostInfo = serde_json::from_str(json).unwrap();
        assert_eq!(host.os, "windows");
        assert_eq!(host.arch, "aarch64");
    }

    #[test]
    fn clone() {
        let host = HostInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        let cloned = host.clone();
        assert_eq!(host.os, cloned.os);
    }

    #[test]
    fn debug_format() {
        let host = HostInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        let debug = format!("{:?}", host);
        assert!(debug.contains("HostInfo"));
    }
}

// =============================================================================
// GitInfo Tests
// =============================================================================

mod git_info_tests {
    use super::*;

    fn make_git_info() -> GitInfo {
        GitInfo {
            repo: None,
            base_ref: None,
            head_ref: None,
            base_sha: None,
            head_sha: None,
            merge_base: None,
        }
    }

    #[test]
    fn empty_git_info() {
        let git = make_git_info();
        assert!(git.repo.is_none());
        assert!(git.base_ref.is_none());
        assert!(git.head_ref.is_none());
        assert!(git.base_sha.is_none());
        assert!(git.head_sha.is_none());
        assert!(git.merge_base.is_none());
    }

    #[test]
    fn full_git_info() {
        let git = GitInfo {
            repo: Some("https://github.com/example/repo".to_string()),
            base_ref: Some("main".to_string()),
            head_ref: Some("feature".to_string()),
            base_sha: Some("abc123".to_string()),
            head_sha: Some("def456".to_string()),
            merge_base: Some("abc123".to_string()),
        };
        assert_eq!(
            git.repo,
            Some("https://github.com/example/repo".to_string())
        );
        assert_eq!(git.base_ref, Some("main".to_string()));
        assert_eq!(git.head_ref, Some("feature".to_string()));
    }

    #[test]
    fn serialize_empty() {
        let git = make_git_info();
        let json = serde_json::to_string(&git).unwrap();
        // All fields are None, so should be empty object
        assert_eq!(json, "{}");
    }

    #[test]
    fn serialize_with_fields() {
        let git = GitInfo {
            repo: Some("https://github.com/example/repo".to_string()),
            base_ref: Some("main".to_string()),
            head_ref: None,
            base_sha: None,
            head_sha: None,
            merge_base: None,
        };
        let json = serde_json::to_string(&git).unwrap();
        assert!(json.contains("repo"));
        assert!(json.contains("base_ref"));
        assert!(!json.contains("head_ref"));
    }

    #[test]
    fn deserialize() {
        let json = r#"{
            "repo": "https://github.com/example/repo",
            "base_ref": "main",
            "head_sha": "def456"
        }"#;
        let git: GitInfo = serde_json::from_str(json).unwrap();
        assert_eq!(
            git.repo,
            Some("https://github.com/example/repo".to_string())
        );
        assert_eq!(git.base_ref, Some("main".to_string()));
        assert_eq!(git.head_sha, Some("def456".to_string()));
        assert!(git.head_ref.is_none());
    }

    #[test]
    fn clone() {
        let git = GitInfo {
            repo: Some("test".to_string()),
            ..make_git_info()
        };
        let cloned = git.clone();
        assert_eq!(git.repo, cloned.repo);
    }

    #[test]
    fn debug_format() {
        let git = make_git_info();
        let debug = format!("{:?}", git);
        assert!(debug.contains("GitInfo"));
    }
}

// =============================================================================
// Verdict Tests
// =============================================================================

mod verdict_tests {
    use super::*;

    fn make_verdict() -> Verdict {
        Verdict {
            status: VerdictStatus::Pass,
            counts: Counts::default(),
            reasons: vec![],
        }
    }

    #[test]
    fn basic_verdict() {
        let verdict = make_verdict();
        assert_eq!(verdict.status, VerdictStatus::Pass);
        assert_eq!(verdict.counts.info, 0);
        assert_eq!(verdict.counts.warn, 0);
        assert_eq!(verdict.counts.error, 0);
        assert!(verdict.reasons.is_empty());
    }

    #[test]
    fn verdict_with_reasons() {
        let verdict = Verdict {
            reasons: vec!["reason 1".to_string(), "reason 2".to_string()],
            ..make_verdict()
        };
        assert_eq!(verdict.reasons.len(), 2);
    }

    #[test]
    fn serialize() {
        let verdict = make_verdict();
        let json = serde_json::to_string(&verdict).unwrap();
        assert!(json.contains("status"));
        assert!(json.contains("counts"));
    }

    #[test]
    fn serialize_with_reasons() {
        let verdict = Verdict {
            reasons: vec!["reason 1".to_string()],
            ..make_verdict()
        };
        let json = serde_json::to_string(&verdict).unwrap();
        assert!(json.contains("reasons"));
    }

    #[test]
    fn reasons_skipped_when_empty() {
        let verdict = make_verdict();
        let json = serde_json::to_string(&verdict).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed.as_object().unwrap().contains_key("reasons"));
    }

    #[test]
    fn deserialize() {
        let json = r#"{
            "status": "fail",
            "counts": {"info": 1, "warn": 2, "error": 3}
        }"#;
        let verdict: Verdict = serde_json::from_str(json).unwrap();
        assert_eq!(verdict.status, VerdictStatus::Fail);
        assert_eq!(verdict.counts.info, 1);
        assert_eq!(verdict.counts.warn, 2);
        assert_eq!(verdict.counts.error, 3);
    }

    #[test]
    fn clone() {
        let verdict = make_verdict();
        let cloned = verdict.clone();
        assert_eq!(verdict.status, cloned.status);
    }

    #[test]
    fn debug_format() {
        let verdict = make_verdict();
        let debug = format!("{:?}", verdict);
        assert!(debug.contains("Verdict"));
    }
}

// =============================================================================
// VerdictStatus Tests
// =============================================================================

mod verdict_status_tests {
    use super::*;

    #[test]
    fn serialize_lowercase() {
        assert_eq!(
            serde_json::to_string(&VerdictStatus::Pass).unwrap(),
            r#""pass""#
        );
        assert_eq!(
            serde_json::to_string(&VerdictStatus::Warn).unwrap(),
            r#""warn""#
        );
        assert_eq!(
            serde_json::to_string(&VerdictStatus::Fail).unwrap(),
            r#""fail""#
        );
        assert_eq!(
            serde_json::to_string(&VerdictStatus::Skip).unwrap(),
            r#""skip""#
        );
    }

    #[test]
    fn deserialize_lowercase() {
        let pass: VerdictStatus = serde_json::from_str(r#""pass""#).unwrap();
        assert_eq!(pass, VerdictStatus::Pass);

        let warn: VerdictStatus = serde_json::from_str(r#""warn""#).unwrap();
        assert_eq!(warn, VerdictStatus::Warn);

        let fail: VerdictStatus = serde_json::from_str(r#""fail""#).unwrap();
        assert_eq!(fail, VerdictStatus::Fail);

        let skip: VerdictStatus = serde_json::from_str(r#""skip""#).unwrap();
        assert_eq!(skip, VerdictStatus::Skip);
    }

    #[test]
    fn equality() {
        assert_eq!(VerdictStatus::Pass, VerdictStatus::Pass);
        assert_ne!(VerdictStatus::Pass, VerdictStatus::Fail);
    }

    #[test]
    fn clone() {
        let status = VerdictStatus::Fail;
        let cloned = status;
        assert_eq!(status, cloned);
    }

    #[test]
    fn copy() {
        let status = VerdictStatus::Warn;
        let copied = status;
        let _still_valid = status; // Copy trait allows this
        assert_eq!(copied, VerdictStatus::Warn);
    }

    #[test]
    fn debug_format() {
        let debug = format!("{:?}", VerdictStatus::Pass);
        assert!(debug.contains("Pass"));
    }
}

// =============================================================================
// Counts Tests
// =============================================================================

mod counts_tests {
    use super::*;

    #[test]
    fn default_is_zero() {
        let counts = Counts::default();
        assert_eq!(counts.info, 0);
        assert_eq!(counts.warn, 0);
        assert_eq!(counts.error, 0);
    }

    #[test]
    fn custom_values() {
        let counts = Counts {
            info: 1,
            warn: 2,
            error: 3,
        };
        assert_eq!(counts.info, 1);
        assert_eq!(counts.warn, 2);
        assert_eq!(counts.error, 3);
    }

    #[test]
    fn serialize() {
        let counts = Counts {
            info: 1,
            warn: 2,
            error: 3,
        };
        let json = serde_json::to_string(&counts).unwrap();
        assert!(json.contains("\"info\":1"));
        assert!(json.contains("\"warn\":2"));
        assert!(json.contains("\"error\":3"));
    }

    #[test]
    fn deserialize() {
        let json = r#"{"info": 5, "warn": 10, "error": 15}"#;
        let counts: Counts = serde_json::from_str(json).unwrap();
        assert_eq!(counts.info, 5);
        assert_eq!(counts.warn, 10);
        assert_eq!(counts.error, 15);
    }

    #[test]
    fn clone() {
        let counts = Counts {
            info: 1,
            warn: 2,
            error: 3,
        };
        let cloned = counts.clone();
        assert_eq!(counts.info, cloned.info);
    }

    #[test]
    fn debug_format() {
        let counts = Counts::default();
        let debug = format!("{:?}", counts);
        assert!(debug.contains("Counts"));
    }
}

// =============================================================================
// Finding Tests
// =============================================================================

mod finding_tests {
    use super::*;

    fn make_minimal_finding() -> Finding {
        Finding {
            severity: Severity::Error,
            code: "E001".to_string(),
            message: "test error".to_string(),
            location: None,
            check_id: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        }
    }

    #[test]
    fn minimal_finding() {
        let finding = make_minimal_finding();
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.code, "E001");
        assert_eq!(finding.message, "test error");
        assert!(finding.location.is_none());
    }

    #[test]
    fn finding_with_location() {
        let location = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(10),
            col: Some(5),
        };
        let finding = Finding {
            location: Some(location),
            ..make_minimal_finding()
        };
        assert!(finding.location.is_some());
        let loc = finding.location.unwrap();
        assert_eq!(loc.path.as_str(), "src/lib.rs");
        assert_eq!(loc.line, Some(10));
        assert_eq!(loc.col, Some(5));
    }

    #[test]
    fn finding_with_all_fields() {
        let finding = Finding {
            severity: Severity::Warn,
            check_id: Some("check-001".to_string()),
            code: "W001".to_string(),
            message: "warning message".to_string(),
            location: Some(Location {
                path: NormPath::new("src/lib.rs"),
                line: Some(1),
                col: None,
            }),
            help: Some("try this".to_string()),
            url: Some("https://example.com".to_string()),
            fingerprint: Some("abc123".to_string()),
            data: Some(serde_json::json!({"extra": "data"})),
        };
        assert_eq!(finding.check_id, Some("check-001".to_string()));
        assert_eq!(finding.help, Some("try this".to_string()));
        assert_eq!(finding.url, Some("https://example.com".to_string()));
        assert_eq!(finding.fingerprint, Some("abc123".to_string()));
        assert!(finding.data.is_some());
    }

    #[test]
    fn serialize_minimal() {
        let finding = make_minimal_finding();
        let json = serde_json::to_string(&finding).unwrap();
        assert!(json.contains("severity"));
        assert!(json.contains("code"));
        assert!(json.contains("message"));
    }

    #[test]
    fn deserialize_minimal() {
        let json = r#"{
            "severity": "error",
            "code": "E001",
            "message": "test error"
        }"#;
        let finding: Finding = serde_json::from_str(json).unwrap();
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.code, "E001");
        assert!(finding.location.is_none());
    }

    #[test]
    fn optional_fields_skipped_when_none() {
        let finding = make_minimal_finding();
        let json = serde_json::to_string(&finding).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed.as_object().unwrap().contains_key("location"));
        assert!(!parsed.as_object().unwrap().contains_key("help"));
        assert!(!parsed.as_object().unwrap().contains_key("url"));
    }

    #[test]
    fn clone() {
        let finding = make_minimal_finding();
        let cloned = finding.clone();
        assert_eq!(finding.code, cloned.code);
    }

    #[test]
    fn debug_format() {
        let finding = make_minimal_finding();
        let debug = format!("{:?}", finding);
        assert!(debug.contains("Finding"));
    }
}

// =============================================================================
// Severity Tests
// =============================================================================

mod severity_tests {
    use super::*;

    #[test]
    fn serialize_lowercase() {
        assert_eq!(serde_json::to_string(&Severity::Info).unwrap(), r#""info""#);
        assert_eq!(serde_json::to_string(&Severity::Warn).unwrap(), r#""warn""#);
        assert_eq!(
            serde_json::to_string(&Severity::Error).unwrap(),
            r#""error""#
        );
    }

    #[test]
    fn deserialize_lowercase() {
        let info: Severity = serde_json::from_str(r#""info""#).unwrap();
        assert_eq!(info, Severity::Info);

        let warn: Severity = serde_json::from_str(r#""warn""#).unwrap();
        assert_eq!(warn, Severity::Warn);

        let error: Severity = serde_json::from_str(r#""error""#).unwrap();
        assert_eq!(error, Severity::Error);
    }

    #[test]
    fn equality() {
        assert_eq!(Severity::Error, Severity::Error);
        assert_ne!(Severity::Error, Severity::Warn);
    }

    #[test]
    fn clone() {
        let severity = Severity::Error;
        let cloned = severity;
        assert_eq!(severity, cloned);
    }

    #[test]
    fn copy() {
        let severity = Severity::Warn;
        let copied = severity;
        let _still_valid = severity;
        assert_eq!(copied, Severity::Warn);
    }

    #[test]
    fn debug_format() {
        let debug = format!("{:?}", Severity::Error);
        assert!(debug.contains("Error"));
    }
}

// =============================================================================
// Location Tests
// =============================================================================

mod location_tests {
    use super::*;

    #[test]
    fn location_with_path_only() {
        let location = Location {
            path: NormPath::new("src/lib.rs"),
            line: None,
            col: None,
        };
        assert_eq!(location.path.as_str(), "src/lib.rs");
        assert!(location.line.is_none());
        assert!(location.col.is_none());
    }

    #[test]
    fn location_with_line_only() {
        let location = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(10),
            col: None,
        };
        assert_eq!(location.line, Some(10));
        assert!(location.col.is_none());
    }

    #[test]
    fn location_with_all_fields() {
        let location = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(10),
            col: Some(5),
        };
        assert_eq!(location.path.as_str(), "src/lib.rs");
        assert_eq!(location.line, Some(10));
        assert_eq!(location.col, Some(5));
    }

    #[test]
    fn serialize() {
        let location = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(10),
            col: Some(5),
        };
        let json = serde_json::to_string(&location).unwrap();
        assert!(json.contains("path"));
        assert!(json.contains("line"));
        assert!(json.contains("col"));
    }

    #[test]
    fn deserialize() {
        let json = r#"{"path": "src/lib.rs", "line": 10, "col": 5}"#;
        let location: Location = serde_json::from_str(json).unwrap();
        assert_eq!(location.path.as_str(), "src/lib.rs");
        assert_eq!(location.line, Some(10));
        assert_eq!(location.col, Some(5));
    }

    #[test]
    fn optional_fields_skipped_when_none() {
        let location = Location {
            path: NormPath::new("src/lib.rs"),
            line: None,
            col: None,
        };
        let json = serde_json::to_string(&location).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed.as_object().unwrap().contains_key("line"));
        assert!(!parsed.as_object().unwrap().contains_key("col"));
    }

    #[test]
    fn clone() {
        let location = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(10),
            col: None,
        };
        let cloned = location.clone();
        assert_eq!(location.path, cloned.path);
    }

    #[test]
    fn debug_format() {
        let location = Location {
            path: NormPath::new("src/lib.rs"),
            line: None,
            col: None,
        };
        let debug = format!("{:?}", location);
        assert!(debug.contains("Location"));
    }
}

// =============================================================================
// DiagnosticDisposition Tests
// =============================================================================

mod diagnostic_disposition_tests {
    use super::*;

    fn make_disposition() -> DiagnosticDisposition {
        DiagnosticDisposition {
            code: "E001".to_string(),
            message_preview: "error message...".to_string(),
            file: None,
            line: None,
            disposition: Disposition::Included,
            fingerprint: None,
        }
    }

    #[test]
    fn basic_disposition() {
        let disp = make_disposition();
        assert_eq!(disp.code, "E001");
        assert_eq!(disp.message_preview, "error message...");
        assert_eq!(disp.disposition, Disposition::Included);
    }

    #[test]
    fn disposition_with_file_and_line() {
        let disp = DiagnosticDisposition {
            file: Some("src/lib.rs".to_string()),
            line: Some(10),
            ..make_disposition()
        };
        assert_eq!(disp.file, Some("src/lib.rs".to_string()));
        assert_eq!(disp.line, Some(10));
    }

    #[test]
    fn disposition_with_fingerprint() {
        let disp = DiagnosticDisposition {
            fingerprint: Some("abc123".to_string()),
            ..make_disposition()
        };
        assert_eq!(disp.fingerprint, Some("abc123".to_string()));
    }

    #[test]
    fn serialize() {
        let disp = make_disposition();
        let json = serde_json::to_string(&disp).unwrap();
        assert!(json.contains("code"));
        assert!(json.contains("message_preview"));
        assert!(json.contains("disposition"));
    }

    #[test]
    fn deserialize() {
        let json = r#"{
            "code": "E001",
            "message_preview": "error...",
            "disposition": "included"
        }"#;
        let disp: DiagnosticDisposition = serde_json::from_str(json).unwrap();
        assert_eq!(disp.code, "E001");
        assert_eq!(disp.disposition, Disposition::Included);
    }

    #[test]
    fn clone() {
        let disp = make_disposition();
        let cloned = disp.clone();
        assert_eq!(disp.code, cloned.code);
    }

    #[test]
    fn debug_format() {
        let disp = make_disposition();
        let debug = format!("{:?}", disp);
        assert!(debug.contains("DiagnosticDisposition"));
    }
}

// =============================================================================
// Disposition Tests
// =============================================================================

mod disposition_enum_tests {
    use super::*;

    #[test]
    fn serialize_snake_case() {
        assert_eq!(
            serde_json::to_string(&Disposition::Included).unwrap(),
            r#""included""#
        );
        assert_eq!(
            serde_json::to_string(&Disposition::DroppedNoSpan).unwrap(),
            r#""dropped_no_span""#
        );
        assert_eq!(
            serde_json::to_string(&Disposition::DroppedOutsideDiff).unwrap(),
            r#""dropped_outside_diff""#
        );
        assert_eq!(
            serde_json::to_string(&Disposition::DroppedByPathFilter).unwrap(),
            r#""dropped_by_path_filter""#
        );
        assert_eq!(
            serde_json::to_string(&Disposition::SuppressedByCode).unwrap(),
            r#""suppressed_by_code""#
        );
        assert_eq!(
            serde_json::to_string(&Disposition::CutByBudget).unwrap(),
            r#""cut_by_budget""#
        );
    }

    #[test]
    fn deserialize_snake_case() {
        let included: Disposition = serde_json::from_str(r#""included""#).unwrap();
        assert_eq!(included, Disposition::Included);

        let dropped: Disposition = serde_json::from_str(r#""dropped_no_span""#).unwrap();
        assert_eq!(dropped, Disposition::DroppedNoSpan);

        let cut: Disposition = serde_json::from_str(r#""cut_by_budget""#).unwrap();
        assert_eq!(cut, Disposition::CutByBudget);
    }

    #[test]
    fn equality() {
        assert_eq!(Disposition::Included, Disposition::Included);
        assert_ne!(Disposition::Included, Disposition::CutByBudget);
    }

    #[test]
    fn clone() {
        let disp = Disposition::Included;
        let cloned = disp.clone();
        assert_eq!(disp, cloned);
    }

    #[test]
    fn debug_format() {
        let debug = format!("{:?}", Disposition::Included);
        assert!(debug.contains("Included"));
    }
}

// =============================================================================
// ExplainSummary Tests
// =============================================================================

mod explain_summary_tests {
    use super::*;

    #[test]
    fn default_is_zero() {
        let summary = ExplainSummary::default();
        assert_eq!(summary.total, 0);
        assert_eq!(summary.included, 0);
        assert_eq!(summary.dropped_no_span, 0);
        assert_eq!(summary.dropped_outside_diff, 0);
        assert_eq!(summary.dropped_by_path_filter, 0);
        assert_eq!(summary.suppressed_by_code, 0);
        assert_eq!(summary.cut_by_budget, 0);
    }

    #[test]
    fn custom_values() {
        let summary = ExplainSummary {
            total: 100,
            included: 50,
            dropped_no_span: 10,
            dropped_outside_diff: 15,
            dropped_by_path_filter: 10,
            suppressed_by_code: 5,
            cut_by_budget: 10,
        };
        assert_eq!(summary.total, 100);
        assert_eq!(summary.included, 50);
    }

    #[test]
    fn serialize() {
        let summary = ExplainSummary {
            total: 10,
            included: 5,
            dropped_no_span: 1,
            dropped_outside_diff: 1,
            dropped_by_path_filter: 1,
            suppressed_by_code: 1,
            cut_by_budget: 1,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("total"));
        assert!(json.contains("included"));
        assert!(json.contains("dropped_no_span"));
    }

    #[test]
    fn deserialize() {
        let json = r#"{
            "total": 100,
            "included": 50,
            "dropped_no_span": 10,
            "dropped_outside_diff": 10,
            "dropped_by_path_filter": 10,
            "suppressed_by_code": 10,
            "cut_by_budget": 10
        }"#;
        let summary: ExplainSummary = serde_json::from_str(json).unwrap();
        assert_eq!(summary.total, 100);
        assert_eq!(summary.included, 50);
    }

    #[test]
    fn clone() {
        let summary = ExplainSummary {
            total: 10,
            ..Default::default()
        };
        let cloned = summary.clone();
        assert_eq!(summary.total, cloned.total);
    }

    #[test]
    fn debug_format() {
        let summary = ExplainSummary::default();
        let debug = format!("{:?}", summary);
        assert!(debug.contains("ExplainSummary"));
    }
}

// =============================================================================
// Round-trip Serialization Tests
// =============================================================================

mod round_trip_tests {
    use super::*;

    #[test]
    fn report_round_trip() {
        let original = Report {
            schema: SCHEMA_ID.to_string(),
            tool: ToolInfo {
                name: TOOL_NAME.to_string(),
                version: "1.0.0".to_string(),
                commit: Some("abc123".to_string()),
            },
            run: RunInfo {
                started_at: "2024-01-01T00:00:00Z".to_string(),
                ended_at: "2024-01-01T00:00:01Z".to_string(),
                duration_ms: Some(1000),
                host: Some(HostInfo {
                    os: "linux".to_string(),
                    arch: "x86_64".to_string(),
                }),
                git: Some(GitInfo {
                    repo: Some("https://github.com/example/repo".to_string()),
                    base_ref: Some("main".to_string()),
                    head_ref: Some("feature".to_string()),
                    base_sha: Some("abc123".to_string()),
                    head_sha: Some("def456".to_string()),
                    merge_base: Some("abc123".to_string()),
                }),
            },
            verdict: Verdict {
                status: VerdictStatus::Fail,
                counts: Counts {
                    info: 1,
                    warn: 2,
                    error: 3,
                },
                reasons: vec!["Found 3 errors".to_string()],
            },
            findings: vec![Finding {
                severity: Severity::Error,
                check_id: Some("check-001".to_string()),
                code: "E001".to_string(),
                message: "test error".to_string(),
                location: Some(Location {
                    path: NormPath::new("src/lib.rs"),
                    line: Some(10),
                    col: Some(5),
                }),
                help: Some("fix this".to_string()),
                url: Some("https://example.com".to_string()),
                fingerprint: Some("fp123".to_string()),
                data: Some(serde_json::json!({"key": "value"})),
            }],
            data: Some(serde_json::json!({"custom": "data"})),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: Report = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.schema, original.schema);
        assert_eq!(parsed.tool.name, original.tool.name);
        assert_eq!(parsed.tool.version, original.tool.version);
        assert_eq!(parsed.tool.commit, original.tool.commit);
        assert_eq!(parsed.run.started_at, original.run.started_at);
        assert_eq!(parsed.verdict.status, original.verdict.status);
        assert_eq!(parsed.findings.len(), original.findings.len());
    }

    #[test]
    fn finding_round_trip() {
        let original = Finding {
            severity: Severity::Warn,
            check_id: Some("check-001".to_string()),
            code: "W001".to_string(),
            message: "warning message".to_string(),
            location: Some(Location {
                path: NormPath::new("src/lib.rs"),
                line: Some(10),
                col: None,
            }),
            help: Some("try this".to_string()),
            url: None,
            fingerprint: Some("abc123".to_string()),
            data: None,
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: Finding = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.severity, original.severity);
        assert_eq!(parsed.code, original.code);
        assert_eq!(parsed.message, original.message);
        assert_eq!(parsed.location.unwrap().path.as_str(), "src/lib.rs");
    }
}
