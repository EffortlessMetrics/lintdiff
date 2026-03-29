//! Integration tests for lintdiff-render-annotations.
//!
//! These tests verify the GitHub Actions annotation rendering functionality
//! including format compliance, severity filtering, and configuration options.

use lintdiff_render_annotations::{
    render_annotations, render_finding_annotation, AnnotationsConfig,
};
use lintdiff_types::{Finding, Location, NormPath, Severity};

/// Helper to create a test finding with all fields.
fn create_finding(
    severity: Severity,
    code: &str,
    message: &str,
    path: &str,
    line: Option<u32>,
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

/// Helper to create a test finding without a location.
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

mod empty_findings {
    use super::*;

    #[test]
    fn returns_empty_string_for_empty_input() {
        let findings: Vec<Finding> = vec![];
        let config = AnnotationsConfig::default();
        let result = render_annotations(&findings, &config);
        assert!(result.is_empty());
    }

    #[test]
    fn returns_empty_string_when_all_filtered_out() {
        let findings = vec![create_finding_no_location(Severity::Info, "I001", "info")];
        let config = AnnotationsConfig::default(); // include_notes: false by default
        let result = render_annotations(&findings, &config);
        assert!(result.is_empty());
    }
}

mod single_finding_rendering {
    use super::*;

    #[test]
    fn renders_error_with_correct_level() {
        let finding = create_finding(
            Severity::Error,
            "E001",
            "error message",
            "src/lib.rs",
            Some(10),
        );
        let annotation = render_finding_annotation(&finding);

        assert!(annotation.starts_with("::error"));
        assert!(annotation.contains("file=src/lib.rs"));
        assert!(annotation.contains("line=10"));
        assert!(annotation.contains("title=E001"));
        assert!(annotation.ends_with("error message"));
    }

    #[test]
    fn renders_warning_with_correct_level() {
        let finding = create_finding(
            Severity::Warn,
            "W001",
            "warning message",
            "src/main.rs",
            Some(42),
        );
        let annotation = render_finding_annotation(&finding);

        assert!(annotation.starts_with("::warning"));
        assert!(annotation.contains("file=src/main.rs"));
        assert!(annotation.contains("line=42"));
    }

    #[test]
    fn renders_info_as_notice() {
        let finding = create_finding(
            Severity::Info,
            "I001",
            "info message",
            "docs/README.md",
            Some(1),
        );
        let annotation = render_finding_annotation(&finding);

        assert!(annotation.starts_with("::notice"));
        assert!(annotation.contains("file=docs/README.md"));
        assert!(annotation.contains("line=1"));
    }

    #[test]
    fn renders_finding_without_location() {
        let finding = create_finding_no_location(Severity::Error, "E002", "no location");
        let annotation = render_finding_annotation(&finding);

        assert!(annotation.starts_with("::error"));
        assert!(annotation.contains("title=E002"));
        assert!(!annotation.contains("file="));
        assert!(!annotation.contains("line="));
    }

    #[test]
    fn uses_default_line_when_missing() {
        let finding = create_finding(Severity::Error, "E003", "no line", "src/lib.rs", None);
        let annotation = render_finding_annotation(&finding);

        // Should use line 1 as default
        assert!(annotation.contains("line=1"));
    }
}

mod multiple_findings {
    use super::*;

    #[test]
    fn renders_all_findings_on_separate_lines() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "first", "src/a.rs", Some(1)),
            create_finding(Severity::Warn, "W001", "second", "src/b.rs", Some(2)),
            create_finding(Severity::Error, "E002", "third", "src/c.rs", Some(3)),
        ];
        let config = AnnotationsConfig::default();
        let result = render_annotations(&findings, &config);

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("E001"));
        assert!(lines[1].contains("W001"));
        assert!(lines[2].contains("E002"));
    }

    #[test]
    fn preserves_order_of_findings() {
        let findings = vec![
            create_finding(Severity::Error, "FIRST", "first", "src/lib.rs", Some(1)),
            create_finding(Severity::Error, "SECOND", "second", "src/lib.rs", Some(2)),
            create_finding(Severity::Error, "THIRD", "third", "src/lib.rs", Some(3)),
        ];
        let config = AnnotationsConfig::default();
        let result = render_annotations(&findings, &config);

        let first_line = result.lines().next().unwrap();
        assert!(first_line.contains("FIRST"));
    }
}

mod severity_filtering {
    use super::*;

    #[test]
    fn filters_out_errors_when_disabled() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "error", "src/lib.rs", Some(1)),
            create_finding(Severity::Warn, "W001", "warning", "src/lib.rs", Some(2)),
        ];
        let config = AnnotationsConfig {
            include_errors: false,
            include_warnings: true,
            include_notes: false,
            max_annotations: 50,
        };
        let result = render_annotations(&findings, &config);

        assert!(!result.contains("E001"));
        assert!(result.contains("W001"));
    }

    #[test]
    fn filters_out_warnings_when_disabled() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "error", "src/lib.rs", Some(1)),
            create_finding(Severity::Warn, "W001", "warning", "src/lib.rs", Some(2)),
        ];
        let config = AnnotationsConfig {
            include_errors: true,
            include_warnings: false,
            include_notes: false,
            max_annotations: 50,
        };
        let result = render_annotations(&findings, &config);

        assert!(result.contains("E001"));
        assert!(!result.contains("W001"));
    }

    #[test]
    fn includes_notes_when_enabled() {
        let findings = vec![create_finding(
            Severity::Info,
            "I001",
            "info",
            "src/lib.rs",
            Some(1),
        )];
        let config = AnnotationsConfig {
            include_errors: true,
            include_warnings: true,
            include_notes: true,
            max_annotations: 50,
        };
        let result = render_annotations(&findings, &config);

        assert!(result.contains("I001"));
        assert!(result.contains("::notice"));
    }

    #[test]
    fn filters_all_when_none_enabled() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "error", "src/lib.rs", Some(1)),
            create_finding(Severity::Warn, "W001", "warning", "src/lib.rs", Some(2)),
            create_finding(Severity::Info, "I001", "info", "src/lib.rs", Some(3)),
        ];
        let config = AnnotationsConfig {
            include_errors: false,
            include_warnings: false,
            include_notes: false,
            max_annotations: 50,
        };
        let result = render_annotations(&findings, &config);

        assert!(result.is_empty());
    }
}

mod max_annotations_limit {
    use super::*;

    #[test]
    fn limits_output_to_max_annotations() {
        let findings: Vec<Finding> = (0..100)
            .map(|i| {
                create_finding(
                    Severity::Error,
                    &format!("E{i:03}"),
                    "error",
                    "src/lib.rs",
                    Some(i),
                )
            })
            .collect();

        let config = AnnotationsConfig {
            max_annotations: 10,
            ..Default::default()
        };
        let result = render_annotations(&findings, &config);

        assert_eq!(result.lines().count(), 10);
    }

    #[test]
    fn takes_first_n_findings() {
        let findings: Vec<Finding> = (0..5)
            .map(|i| {
                create_finding(
                    Severity::Error,
                    &format!("E{i}"),
                    "error",
                    "src/lib.rs",
                    Some(i),
                )
            })
            .collect();

        let config = AnnotationsConfig {
            max_annotations: 2,
            ..Default::default()
        };
        let result = render_annotations(&findings, &config);

        assert!(result.contains("E0"));
        assert!(result.contains("E1"));
        assert!(!result.contains("E2"));
        assert!(!result.contains("E3"));
        assert!(!result.contains("E4"));
    }

    #[test]
    fn zero_max_annotations_produces_empty_output() {
        let findings = vec![create_finding(
            Severity::Error,
            "E001",
            "error",
            "src/lib.rs",
            Some(1),
        )];
        let config = AnnotationsConfig {
            max_annotations: 0,
            ..Default::default()
        };
        let result = render_annotations(&findings, &config);

        assert!(result.is_empty());
    }
}

mod github_annotation_format {
    use super::*;

    #[test]
    fn follows_github_workflow_command_format() {
        let finding = create_finding(
            Severity::Error,
            "clippy::unwrap_used",
            "used unwrap()",
            "src/lib.rs",
            Some(42),
        );
        let annotation = render_finding_annotation(&finding);

        // Format: ::{level} file={file},line={line},title={title}::{message}
        // Note: colons in title are escaped for GitHub Actions
        assert!(annotation.starts_with("::error "));
        assert!(annotation.contains(" file=src/lib.rs"));
        assert!(annotation.contains(",line=42"));
        assert!(annotation.contains(",title=clippy%3A%3Aunwrap_used"));
        assert!(annotation.contains("::used unwrap()"));
    }

    #[test]
    fn escapes_colons_in_code() {
        let finding = create_finding(
            Severity::Error,
            "clippy::unwrap_used",
            "message",
            "src/lib.rs",
            Some(1),
        );
        let annotation = render_finding_annotation(&finding);

        // The title should have colons escaped
        assert!(annotation.contains("title=clippy%3A%3Aunwrap_used"));
    }

    #[test]
    fn escapes_colons_in_message() {
        let finding = create_finding(
            Severity::Error,
            "E001",
            "error: something went wrong",
            "src/lib.rs",
            Some(1),
        );
        let annotation = render_finding_annotation(&finding);

        assert!(annotation.contains("error%3A something went wrong"));
    }

    #[test]
    fn escapes_commas_in_message() {
        let finding = create_finding(Severity::Error, "E001", "a, b, c", "src/lib.rs", Some(1));
        let annotation = render_finding_annotation(&finding);

        assert!(annotation.contains("a%2C b%2C c"));
    }

    #[test]
    fn escapes_newlines_in_message() {
        let finding = create_finding(
            Severity::Error,
            "E001",
            "line1\nline2",
            "src/lib.rs",
            Some(1),
        );
        let annotation = render_finding_annotation(&finding);

        assert!(annotation.contains("line1%0Aline2"));
    }

    #[test]
    fn escapes_carriage_returns_in_message() {
        let finding = create_finding(
            Severity::Error,
            "E001",
            "line1\r\nline2",
            "src/lib.rs",
            Some(1),
        );
        let annotation = render_finding_annotation(&finding);

        assert!(annotation.contains("line1%0D%0Aline2"));
    }

    #[test]
    fn escapes_percent_in_message() {
        let finding = create_finding(
            Severity::Error,
            "E001",
            "100% complete",
            "src/lib.rs",
            Some(1),
        );
        let annotation = render_finding_annotation(&finding);

        assert!(annotation.contains("100%25 complete"));
    }

    #[test]
    fn handles_complex_paths() {
        let finding = create_finding(
            Severity::Error,
            "E001",
            "error",
            "crates/lintdiff-render-annotations/src/lib.rs",
            Some(100),
        );
        let annotation = render_finding_annotation(&finding);

        assert!(annotation.contains("file=crates/lintdiff-render-annotations/src/lib.rs"));
    }
}

mod config_defaults {
    use super::*;

    #[test]
    fn default_max_annotations_is_50() {
        let config = AnnotationsConfig::default();
        assert_eq!(config.max_annotations, 50);
    }

    #[test]
    fn default_includes_errors() {
        let config = AnnotationsConfig::default();
        assert!(config.include_errors);
    }

    #[test]
    fn default_includes_warnings() {
        let config = AnnotationsConfig::default();
        assert!(config.include_warnings);
    }

    #[test]
    fn default_excludes_notes() {
        let config = AnnotationsConfig::default();
        assert!(!config.include_notes);
    }
}

mod path_normalization {
    use super::*;

    #[test]
    fn handles_windows_style_paths() {
        // NormPath normalizes backslashes to forward slashes
        let finding = Finding {
            severity: Severity::Error,
            code: "E001".to_string(),
            message: "error".to_string(),
            location: Some(Location {
                path: NormPath::new("src\\lib.rs"),
                line: Some(1),
                col: None,
            }),
            check_id: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        };
        let annotation = render_finding_annotation(&finding);

        // Path should be normalized to forward slashes
        assert!(annotation.contains("file=src/lib.rs"));
    }

    #[test]
    fn handles_dot_slash_prefix() {
        let finding = Finding {
            severity: Severity::Error,
            code: "E001".to_string(),
            message: "error".to_string(),
            location: Some(Location {
                path: NormPath::new("./src/lib.rs"),
                line: Some(1),
                col: None,
            }),
            check_id: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        };
        let annotation = render_finding_annotation(&finding);

        // Path should be normalized
        assert!(annotation.contains("file=src/lib.rs"));
    }
}
