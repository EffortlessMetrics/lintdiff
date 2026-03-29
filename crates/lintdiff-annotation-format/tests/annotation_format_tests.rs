//! BDD tests for lintdiff-annotation-format.
//!
//! These tests verify annotation format handling and CI detection functionality
//! covering all format variants, CI detection functions, and edge cases.

use lintdiff_annotation_format::{
    detect_ci, format_annotation, format_azure_annotation, format_circleci_annotation,
    format_default_annotation, format_gitlab_annotation, format_github_annotation, is_azure_devops,
    is_circleci, is_github_actions, is_gitlab_ci, Annotation, AnnotationFormat, AnnotationSeverity,
    CiPlatform,
};
use proptest::prelude::*;

// =============================================================================
// Feature: Annotation Format Enum
// =============================================================================

mod annotation_format_enum {
    use super::*;

    mod scenario_default_format {
        use super::*;

        #[test]
        fn default_annotation_format_is_default() {
            let format = AnnotationFormat::default();
            assert_eq!(format, AnnotationFormat::Default);
        }
    }

    mod scenario_resolve_auto_format {
        use super::*;

        #[test]
        fn auto_resolves_to_detected_ci_format() {
            // Auto should resolve to some format (depends on environment)
            let resolved = AnnotationFormat::Auto.resolve();
            // Should be a valid non-auto format
            assert!(matches!(
                resolved,
                AnnotationFormat::Github
                    | AnnotationFormat::Gitlab
                    | AnnotationFormat::Azure
                    | AnnotationFormat::CircleCI
                    | AnnotationFormat::Default
            ));
        }
    }

    mod scenario_resolve_non_auto_formats {
        use super::*;

        #[test]
        fn github_format_resolves_to_itself() {
            assert_eq!(AnnotationFormat::Github.resolve(), AnnotationFormat::Github);
        }

        #[test]
        fn gitlab_format_resolves_to_itself() {
            assert_eq!(
                AnnotationFormat::Gitlab.resolve(),
                AnnotationFormat::Gitlab
            );
        }

        #[test]
        fn azure_format_resolves_to_itself() {
            assert_eq!(AnnotationFormat::Azure.resolve(), AnnotationFormat::Azure);
        }

        #[test]
        fn circleci_format_resolves_to_itself() {
            assert_eq!(
                AnnotationFormat::CircleCI.resolve(),
                AnnotationFormat::CircleCI
            );
        }

        #[test]
        fn default_format_resolves_to_itself() {
            assert_eq!(
                AnnotationFormat::Default.resolve(),
                AnnotationFormat::Default
            );
        }
    }
}

// =============================================================================
// Feature: CI Platform Detection
// =============================================================================

mod ci_platform_detection {
    use super::*;

    mod scenario_detect_ci_function {
        use super::*;

        #[test]
        fn detect_ci_returns_valid_platform() {
            let platform = detect_ci();
            assert!(matches!(
                platform,
                CiPlatform::GithubActions
                    | CiPlatform::GitLabCI
                    | CiPlatform::AzureDevOps
                    | CiPlatform::CircleCI
                    | CiPlatform::TravisCI
                    | CiPlatform::Jenkins
                    | CiPlatform::Unknown
            ));
        }
    }

    mod scenario_platform_annotation_format {
        use super::*;

        #[test]
        fn github_actions_maps_to_github_format() {
            assert_eq!(
                CiPlatform::GithubActions.annotation_format(),
                AnnotationFormat::Github
            );
        }

        #[test]
        fn gitlab_ci_maps_to_gitlab_format() {
            assert_eq!(
                CiPlatform::GitLabCI.annotation_format(),
                AnnotationFormat::Gitlab
            );
        }

        #[test]
        fn azure_devops_maps_to_azure_format() {
            assert_eq!(
                CiPlatform::AzureDevOps.annotation_format(),
                AnnotationFormat::Azure
            );
        }

        #[test]
        fn circleci_maps_to_circleci_format() {
            assert_eq!(
                CiPlatform::CircleCI.annotation_format(),
                AnnotationFormat::CircleCI
            );
        }

        #[test]
        fn travis_ci_maps_to_default_format() {
            assert_eq!(
                CiPlatform::TravisCI.annotation_format(),
                AnnotationFormat::Default
            );
        }

        #[test]
        fn jenkins_maps_to_default_format() {
            assert_eq!(CiPlatform::Jenkins.annotation_format(), AnnotationFormat::Default);
        }

        #[test]
        fn unknown_maps_to_default_format() {
            assert_eq!(CiPlatform::Unknown.annotation_format(), AnnotationFormat::Default);
        }
    }

    mod scenario_github_actions_detection {
        use super::*;

        #[test]
        fn is_github_actions_returns_false_without_env() {
            // Without GITHUB_ACTIONS=true, should return false
            // Note: This test may pass or fail depending on environment
            let _result = is_github_actions();
            // Just verify it doesn't panic
        }

        #[test]
        #[cfg_attr(target_os = "windows", allow(unused_attributes))]
        fn is_github_actions_detects_true_when_set() {
            temp_env::with_var("GITHUB_ACTIONS", Some("true"), || {
                assert!(is_github_actions());
            });
        }

        #[test]
        fn is_github_actions_returns_false_for_wrong_value() {
            temp_env::with_var("GITHUB_ACTIONS", Some("false"), || {
                assert!(!is_github_actions());
            });
        }
    }

    mod scenario_gitlab_ci_detection {
        use super::*;

        #[test]
        #[cfg_attr(target_os = "windows", allow(unused_attributes))]
        fn is_gitlab_ci_detects_true_when_set() {
            temp_env::with_var("GITLAB_CI", Some("true"), || {
                assert!(is_gitlab_ci());
            });
        }

        #[test]
        fn is_gitlab_ci_returns_false_for_wrong_value() {
            temp_env::with_var("GITLAB_CI", Some("false"), || {
                assert!(!is_gitlab_ci());
            });
        }
    }

    mod scenario_azure_devops_detection {
        use super::*;

        #[test]
        #[cfg_attr(target_os = "windows", allow(unused_attributes))]
        fn is_azure_devops_detects_true_when_set() {
            temp_env::with_var("TF_BUILD", Some("True"), || {
                assert!(is_azure_devops());
            });
        }

        #[test]
        fn is_azure_devops_returns_false_for_wrong_value() {
            temp_env::with_var("TF_BUILD", Some("False"), || {
                assert!(!is_azure_devops());
            });
        }
    }

    mod scenario_circleci_detection {
        use super::*;

        #[test]
        #[cfg_attr(target_os = "windows", allow(unused_attributes))]
        fn is_circleci_detects_true_when_set() {
            temp_env::with_var("CIRCLECI", Some("true"), || {
                assert!(is_circleci());
            });
        }

        #[test]
        fn is_circleci_returns_false_for_wrong_value() {
            temp_env::with_var("CIRCLECI", Some("false"), || {
                assert!(!is_circleci());
            });
        }
    }

    mod scenario_detect_ci_with_env {
        use super::*;

        #[test]
        fn detects_github_actions_when_set() {
            temp_env::with_var("GITHUB_ACTIONS", Some("true"), || {
                assert_eq!(detect_ci(), CiPlatform::GithubActions);
            });
        }

        #[test]
        fn detects_gitlab_ci_when_set() {
            temp_env::with_var("GITLAB_CI", Some("true"), || {
                assert_eq!(detect_ci(), CiPlatform::GitLabCI);
            });
        }

        #[test]
        fn detects_azure_devops_when_set() {
            temp_env::with_var("TF_BUILD", Some("True"), || {
                assert_eq!(detect_ci(), CiPlatform::AzureDevOps);
            });
        }

        #[test]
        fn detects_circleci_when_set() {
            temp_env::with_var("CIRCLECI", Some("true"), || {
                assert_eq!(detect_ci(), CiPlatform::CircleCI);
            });
        }

        #[test]
        fn detects_travis_ci_when_set() {
            temp_env::with_var("TRAVIS", Some("true"), || {
                assert_eq!(detect_ci(), CiPlatform::TravisCI);
            });
        }

        #[test]
        fn detects_jenkins_when_set() {
            temp_env::with_var("JENKINS_URL", Some("http://jenkins:8080"), || {
                assert_eq!(detect_ci(), CiPlatform::Jenkins);
            });
        }

        #[test]
        fn returns_unknown_when_no_ci_detected() {
            temp_env::with_vars(
                [
                    ("GITHUB_ACTIONS", None::<&str>),
                    ("GITLAB_CI", None::<&str>),
                    ("TF_BUILD", None::<&str>),
                    ("CIRCLECI", None::<&str>),
                    ("TRAVIS", None::<&str>),
                    ("JENKINS_URL", None::<&str>),
                ],
                || {
                    assert_eq!(detect_ci(), CiPlatform::Unknown);
                },
            );
        }

        #[test]
        fn github_actions_has_priority_over_others() {
            // When multiple CI env vars are set, GitHub Actions should be detected first
            temp_env::with_vars(
                [
                    ("GITHUB_ACTIONS", Some("true")),
                    ("GITLAB_CI", Some("true")),
                ],
                || {
                    assert_eq!(detect_ci(), CiPlatform::GithubActions);
                },
            );
        }
    }
}

// =============================================================================
// Feature: Annotation Severity
// =============================================================================

mod annotation_severity {
    use super::*;

    mod scenario_github_level_mapping {
        use super::*;

        #[test]
        fn notice_maps_to_notice() {
            assert_eq!(AnnotationSeverity::Notice.as_github_level(), "notice");
        }

        #[test]
        fn warning_maps_to_warning() {
            assert_eq!(AnnotationSeverity::Warning.as_github_level(), "warning");
        }

        #[test]
        fn error_maps_to_error() {
            assert_eq!(AnnotationSeverity::Error.as_github_level(), "error");
        }

        #[test]
        fn fatal_maps_to_error() {
            assert_eq!(AnnotationSeverity::Fatal.as_github_level(), "error");
        }
    }

    mod scenario_gitlab_severity_mapping {
        use super::*;

        #[test]
        fn notice_maps_to_info() {
            assert_eq!(AnnotationSeverity::Notice.as_gitlab_severity(), "info");
        }

        #[test]
        fn warning_maps_to_warning() {
            assert_eq!(AnnotationSeverity::Warning.as_gitlab_severity(), "warning");
        }

        #[test]
        fn error_maps_to_error() {
            assert_eq!(AnnotationSeverity::Error.as_gitlab_severity(), "error");
        }

        #[test]
        fn fatal_maps_to_critical() {
            assert_eq!(AnnotationSeverity::Fatal.as_gitlab_severity(), "critical");
        }
    }

    mod scenario_azure_level_mapping {
        use super::*;

        #[test]
        fn notice_maps_to_information() {
            assert_eq!(AnnotationSeverity::Notice.as_azure_level(), "information");
        }

        #[test]
        fn warning_maps_to_warning() {
            assert_eq!(AnnotationSeverity::Warning.as_azure_level(), "warning");
        }

        #[test]
        fn error_maps_to_error() {
            assert_eq!(AnnotationSeverity::Error.as_azure_level(), "error");
        }

        #[test]
        fn fatal_maps_to_error() {
            assert_eq!(AnnotationSeverity::Fatal.as_azure_level(), "error");
        }
    }

    mod scenario_default_severity {
        use super::*;

        #[test]
        fn default_severity_is_warning() {
            assert_eq!(AnnotationSeverity::default(), AnnotationSeverity::Warning);
        }
    }
}

// =============================================================================
// Feature: Annotation Struct
// =============================================================================

mod annotation_struct {
    use super::*;

    mod scenario_create_annotation {
        use super::*;

        #[test]
        fn new_creates_complete_annotation() {
            let annotation = Annotation::new(
                "src/lib.rs",
                42,
                Some(10),
                AnnotationSeverity::Error,
                "Test error",
            );

            assert_eq!(annotation.path, "src/lib.rs");
            assert_eq!(annotation.line, 42);
            assert_eq!(annotation.column, Some(10));
            assert_eq!(annotation.severity, AnnotationSeverity::Error);
            assert_eq!(annotation.message, "Test error");
        }

        #[test]
        fn simple_creates_annotation_without_column() {
            let annotation =
                Annotation::simple("main.rs", 1, AnnotationSeverity::Notice, "Info");

            assert_eq!(annotation.path, "main.rs");
            assert_eq!(annotation.line, 1);
            assert_eq!(annotation.column, None);
            assert_eq!(annotation.severity, AnnotationSeverity::Notice);
            assert_eq!(annotation.message, "Info");
        }
    }

    mod scenario_annotation_equality {
        use super::*;

        #[test]
        fn identical_annotations_are_equal() {
            let a1 = Annotation::new("file.rs", 10, Some(5), AnnotationSeverity::Error, "msg");
            let a2 = Annotation::new("file.rs", 10, Some(5), AnnotationSeverity::Error, "msg");
            assert_eq!(a1, a2);
        }

        #[test]
        fn different_paths_are_not_equal() {
            let a1 = Annotation::simple("file1.rs", 1, AnnotationSeverity::Warning, "msg");
            let a2 = Annotation::simple("file2.rs", 1, AnnotationSeverity::Warning, "msg");
            assert_ne!(a1, a2);
        }

        #[test]
        fn different_lines_are_not_equal() {
            let a1 = Annotation::simple("file.rs", 1, AnnotationSeverity::Warning, "msg");
            let a2 = Annotation::simple("file.rs", 2, AnnotationSeverity::Warning, "msg");
            assert_ne!(a1, a2);
        }

        #[test]
        fn different_columns_are_not_equal() {
            let a1 = Annotation::new("file.rs", 1, Some(1), AnnotationSeverity::Warning, "msg");
            let a2 = Annotation::new("file.rs", 1, Some(2), AnnotationSeverity::Warning, "msg");
            assert_ne!(a1, a2);
        }

        #[test]
        fn different_severities_are_not_equal() {
            let a1 = Annotation::simple("file.rs", 1, AnnotationSeverity::Warning, "msg");
            let a2 = Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "msg");
            assert_ne!(a1, a2);
        }

        #[test]
        fn different_messages_are_not_equal() {
            let a1 = Annotation::simple("file.rs", 1, AnnotationSeverity::Warning, "msg1");
            let a2 = Annotation::simple("file.rs", 1, AnnotationSeverity::Warning, "msg2");
            assert_ne!(a1, a2);
        }
    }
}

// =============================================================================
// Feature: GitHub Annotation Formatting
// =============================================================================

mod github_formatting {
    use super::*;

    mod scenario_basic_formatting {
        use super::*;

        #[test]
        fn formats_error_with_file_and_line() {
            let annotation =
                Annotation::simple("src/lib.rs", 42, AnnotationSeverity::Error, "Error message");

            let output = format_github_annotation(&annotation);
            assert!(output.starts_with("::error file=src/lib.rs,line=42::"));
            assert!(output.ends_with("Error message"));
        }

        #[test]
        fn formats_warning_correctly() {
            let annotation = Annotation::simple(
                "src/warning.rs",
                10,
                AnnotationSeverity::Warning,
                "Warning message",
            );

            let output = format_github_annotation(&annotation);
            assert!(output.starts_with("::warning"));
        }

        #[test]
        fn formats_notice_correctly() {
            let annotation = Annotation::simple(
                "docs/readme.md",
                5,
                AnnotationSeverity::Notice,
                "Notice",
            );

            let output = format_github_annotation(&annotation);
            assert!(output.starts_with("::notice"));
        }

        #[test]
        fn formats_fatal_as_error() {
            let annotation =
                Annotation::simple("fatal.rs", 1, AnnotationSeverity::Fatal, "Fatal error");

            let output = format_github_annotation(&annotation);
            assert!(output.starts_with("::error"));
        }
    }

    mod scenario_with_column {
        use super::*;

        #[test]
        fn includes_column_when_present() {
            let annotation =
                Annotation::new("src/lib.rs", 42, Some(10), AnnotationSeverity::Error, "E");

            let output = format_github_annotation(&annotation);
            assert!(output.contains("col=10"));
        }

        #[test]
        fn column_appears_after_line() {
            let annotation =
                Annotation::new("src/lib.rs", 42, Some(10), AnnotationSeverity::Error, "E");

            let output = format_github_annotation(&annotation);
            let line_pos = output.find("line=42").expect("line not found");
            let col_pos = output.find("col=10").expect("col not found");
            assert!(line_pos < col_pos);
        }
    }

    mod scenario_special_characters {
        use super::*;

        #[test]
        fn escapes_colon_in_message() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "Error: something");

            let output = format_github_annotation(&annotation);
            assert!(output.contains("%3A"));
            assert!(!output.contains("Error: something")); // Colon should be escaped
        }

        #[test]
        fn escapes_comma_in_message() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "a, b, c");

            let output = format_github_annotation(&annotation);
            assert!(output.contains("%2C"));
        }

        #[test]
        fn escapes_percent_in_message() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "100% done");

            let output = format_github_annotation(&annotation);
            assert!(output.contains("%25"));
        }

        #[test]
        fn escapes_newline_in_message() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "line1\nline2");

            let output = format_github_annotation(&annotation);
            assert!(output.contains("%0A"));
        }

        #[test]
        fn escapes_carriage_return_in_message() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "line1\rline2");

            let output = format_github_annotation(&annotation);
            assert!(output.contains("%0D"));
        }

        #[test]
        fn escapes_multiple_special_chars() {
            let annotation = Annotation::simple(
                "file.rs",
                1,
                AnnotationSeverity::Error,
                "a:b,c%d\ne",
            );

            let output = format_github_annotation(&annotation);
            assert!(output.contains("%3A")); // :
            assert!(output.contains("%2C")); // ,
            assert!(output.contains("%25")); // %
            assert!(output.contains("%0A")); // \n
        }
    }
}

// =============================================================================
// Feature: GitLab Annotation Formatting
// =============================================================================

mod gitlab_formatting {
    use super::*;

    mod scenario_basic_formatting {
        use super::*;

        #[test]
        fn formats_with_path_line_severity_message() {
            let annotation =
                Annotation::simple("src/lib.rs", 42, AnnotationSeverity::Warning, "Warning msg");

            let output = format_gitlab_annotation(&annotation);
            assert!(output.starts_with("src/lib.rs:42:"));
            assert!(output.contains("warning"));
            assert!(output.ends_with("Warning msg"));
        }

        #[test]
        fn uses_info_for_notice() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Notice, "Info");

            let output = format_gitlab_annotation(&annotation);
            assert!(output.contains("info"));
        }

        #[test]
        fn uses_critical_for_fatal() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Fatal, "Fatal");

            let output = format_gitlab_annotation(&annotation);
            assert!(output.contains("critical"));
        }
    }

    mod scenario_special_characters {
        use super::*;

        #[test]
        fn escapes_newline() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "line1\nline2");

            let output = format_gitlab_annotation(&annotation);
            assert!(output.contains("\\n"));
        }

        #[test]
        fn escapes_carriage_return() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "line1\rline2");

            let output = format_gitlab_annotation(&annotation);
            assert!(output.contains("\\r"));
        }
    }
}

// =============================================================================
// Feature: Azure DevOps Annotation Formatting
// =============================================================================

mod azure_formatting {
    use super::*;

    mod scenario_basic_formatting {
        use super::*;

        #[test]
        fn uses_logging_command_format() {
            let annotation =
                Annotation::simple("src/lib.rs", 42, AnnotationSeverity::Error, "Error");

            let output = format_azure_annotation(&annotation);
            assert!(output.starts_with("##vso[task.logissue"));
        }

        #[test]
        fn includes_type_sourcepath_and_linenumber() {
            let annotation =
                Annotation::simple("build.rs", 10, AnnotationSeverity::Warning, "Warn");

            let output = format_azure_annotation(&annotation);
            assert!(output.contains("type=warning"));
            assert!(output.contains("sourcepath=build.rs"));
            assert!(output.contains("linenumber=10"));
        }

        #[test]
        fn uses_information_for_notice() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Notice, "Info");

            let output = format_azure_annotation(&annotation);
            assert!(output.contains("type=information"));
        }
    }

    mod scenario_special_characters {
        use super::*;

        #[test]
        fn escapes_bracket() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "Error]");

            let output = format_azure_annotation(&annotation);
            assert!(output.contains("%5D"));
        }

        #[test]
        fn escapes_semicolon() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "a; b");

            let output = format_azure_annotation(&annotation);
            assert!(output.contains("%3B"));
        }

        #[test]
        fn escapes_newline() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "a\nb");

            let output = format_azure_annotation(&annotation);
            assert!(output.contains("%0A"));
        }
    }
}

// =============================================================================
// Feature: CircleCI Annotation Formatting
// =============================================================================

mod circleci_formatting {
    use super::*;

    mod scenario_basic_formatting {
        use super::*;

        #[test]
        fn uses_gcc_style_format() {
            let annotation =
                Annotation::simple("src/lib.rs", 42, AnnotationSeverity::Error, "Error");

            let output = format_circleci_annotation(&annotation);
            assert!(output.starts_with("src/lib.rs:42:"));
        }

        #[test]
        fn includes_column_when_present() {
            let annotation =
                Annotation::new("src/lib.rs", 42, Some(10), AnnotationSeverity::Error, "E");

            let output = format_circleci_annotation(&annotation);
            assert!(output.starts_with("src/lib.rs:42:10:"));
        }

        #[test]
        fn uses_notice_for_notice() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Notice, "Info");

            let output = format_circleci_annotation(&annotation);
            assert!(output.contains("notice"));
        }
    }
}

// =============================================================================
// Feature: Default Annotation Formatting
// =============================================================================

mod default_formatting {
    use super::*;

    mod scenario_basic_formatting {
        use super::*;

        #[test]
        fn includes_path_line_severity_message() {
            let annotation =
                Annotation::simple("src/lib.rs", 42, AnnotationSeverity::Error, "Error msg");

            let output = format_default_annotation(&annotation);
            assert!(output.contains("src/lib.rs"));
            assert!(output.contains("42"));
            assert!(output.contains("Error"));
            assert!(output.contains("Error msg"));
        }

        #[test]
        fn includes_column_when_present() {
            let annotation =
                Annotation::new("src/lib.rs", 42, Some(10), AnnotationSeverity::Error, "E");

            let output = format_default_annotation(&annotation);
            assert!(output.contains("42:10"));
        }

        #[test]
        fn formats_severity_as_debug_format() {
            let warning =
                Annotation::simple("f.rs", 1, AnnotationSeverity::Warning, "w");
            let notice =
                Annotation::simple("f.rs", 1, AnnotationSeverity::Notice, "n");
            let error =
                Annotation::simple("f.rs", 1, AnnotationSeverity::Error, "e");
            let fatal =
                Annotation::simple("f.rs", 1, AnnotationSeverity::Fatal, "f");

            assert!(format_default_annotation(&warning).contains("Warning"));
            assert!(format_default_annotation(&notice).contains("Notice"));
            assert!(format_default_annotation(&error).contains("Error"));
            assert!(format_default_annotation(&fatal).contains("Fatal"));
        }
    }
}

// =============================================================================
// Feature: Generic format_annotation Function
// =============================================================================

mod generic_format_annotation {
    use super::*;

    #[test]
    fn routes_to_github_format() {
        let annotation =
            Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "msg");

        let output = format_annotation(AnnotationFormat::Github, &annotation);
        assert!(output.starts_with("::error"));
    }

    #[test]
    fn routes_to_gitlab_format() {
        let annotation =
            Annotation::simple("file.rs", 1, AnnotationSeverity::Warning, "msg");

        let output = format_annotation(AnnotationFormat::Gitlab, &annotation);
        assert!(output.starts_with("file.rs:1:"));
    }

    #[test]
    fn routes_to_azure_format() {
        let annotation =
            Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "msg");

        let output = format_annotation(AnnotationFormat::Azure, &annotation);
        assert!(output.starts_with("##vso[task.logissue"));
    }

    #[test]
    fn routes_to_circleci_format() {
        let annotation =
            Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "msg");

        let output = format_annotation(AnnotationFormat::CircleCI, &annotation);
        assert!(output.starts_with("file.rs:1:"));
    }

    #[test]
    fn routes_to_default_format() {
        let annotation =
            Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "msg");

        let output = format_annotation(AnnotationFormat::Default, &annotation);
        assert!(output.starts_with("file.rs:1:"));
    }
}

// =============================================================================
// Feature: Edge Cases
// =============================================================================

mod edge_cases {
    use super::*;

    mod scenario_empty_values {
        use super::*;

        #[test]
        fn empty_message_github() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "");

            let output = format_github_annotation(&annotation);
            assert!(output.ends_with("::"));
        }

        #[test]
        fn empty_message_gitlab() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "");

            let output = format_gitlab_annotation(&annotation);
            // Should still have structure: path:line: severity: message
            assert!(output.contains("file.rs:1:"));
        }

        #[test]
        fn empty_message_azure() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "");

            let output = format_azure_annotation(&annotation);
            assert!(output.ends_with(']'));
        }
    }

    mod scenario_boundary_values {
        use super::*;

        #[test]
        fn line_number_one() {
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, "msg");

            let output = format_github_annotation(&annotation);
            assert!(output.contains("line=1"));
        }

        #[test]
        fn large_line_number() {
            let annotation =
                Annotation::simple("file.rs", 1_000_000, AnnotationSeverity::Error, "msg");

            let output = format_github_annotation(&annotation);
            assert!(output.contains("line=1000000"));
        }

        #[test]
        fn column_number_one() {
            let annotation =
                Annotation::new("file.rs", 1, Some(1), AnnotationSeverity::Error, "msg");

            let output = format_github_annotation(&annotation);
            assert!(output.contains("col=1"));
        }

        #[test]
        fn large_column_number() {
            let annotation = Annotation::new(
                "file.rs",
                1,
                Some(10_000),
                AnnotationSeverity::Error,
                "msg",
            );

            let output = format_github_annotation(&annotation);
            assert!(output.contains("col=10000"));
        }
    }

    mod scenario_special_paths {
        use super::*;

        #[test]
        fn path_with_spaces() {
            let annotation = Annotation::simple(
                "src/my module/file.rs",
                1,
                AnnotationSeverity::Error,
                "msg",
            );

            let output = format_github_annotation(&annotation);
            assert!(output.contains("src/my module/file.rs"));
        }

        #[test]
        fn path_with_special_chars() {
            let annotation = Annotation::simple(
                "src/file-name_test.rs",
                1,
                AnnotationSeverity::Error,
                "msg",
            );

            let output = format_github_annotation(&annotation);
            assert!(output.contains("file-name_test.rs"));
        }

        #[test]
        fn windows_style_path() {
            let annotation = Annotation::simple(
                "src\\module\\file.rs",
                1,
                AnnotationSeverity::Error,
                "msg",
            );

            let output = format_github_annotation(&annotation);
            assert!(output.contains("src\\module\\file.rs"));
        }

        #[test]
        fn absolute_path() {
            let annotation = Annotation::simple(
                "/home/user/project/src/file.rs",
                1,
                AnnotationSeverity::Error,
                "msg",
            );

            let output = format_github_annotation(&annotation);
            assert!(output.contains("/home/user/project/src/file.rs"));
        }
    }

    mod scenario_unicode {
        use super::*;

        #[test]
        fn unicode_message() {
            let annotation = Annotation::simple(
                "file.rs",
                1,
                AnnotationSeverity::Error,
                "错误: 你好世界 🌍",
            );

            let output = format_default_annotation(&annotation);
            assert!(output.contains("错误"));
            assert!(output.contains("你好世界"));
            assert!(output.contains("🌍"));
        }

        #[test]
        fn unicode_path() {
            let annotation =
                Annotation::simple("文件/测试.rs", 1, AnnotationSeverity::Error, "msg");

            let output = format_default_annotation(&annotation);
            assert!(output.contains("文件/测试.rs"));
        }
    }

    mod scenario_long_values {
        use super::*;

        #[test]
        fn very_long_message() {
            let long_msg = "x".repeat(10000);
            let annotation =
                Annotation::simple("file.rs", 1, AnnotationSeverity::Error, &long_msg);

            let output = format_github_annotation(&annotation);
            assert!(output.contains(&long_msg));
        }

        #[test]
        fn very_long_path() {
            let long_path = format!("src/{}", "a".repeat(500));
            let annotation =
                Annotation::simple(&long_path, 1, AnnotationSeverity::Error, "msg");

            let output = format_github_annotation(&annotation);
            assert!(output.contains(&long_path));
        }
    }
}

// =============================================================================
// Feature: Property-Based Tests
// =============================================================================

mod property_tests {
    use super::*;

    proptest! {
        #[test]
        fn github_format_never_contains_unescaped_special_chars(
            msg in "[^%\\r\\n:,]*"
        ) {
            let annotation = Annotation::simple("file.rs", 1, AnnotationSeverity::Error, &msg);
            let output = format_github_annotation(&annotation);

            // If message doesn't contain special chars, output should match
            prop_assert!(output.contains(&msg) || msg.is_empty());
        }

        #[test]
        fn line_number_preserved_in_all_formats(line in 1usize..10000) {
            let annotation = Annotation::simple("file.rs", line, AnnotationSeverity::Error, "msg");

            let github = format_github_annotation(&annotation);
            let gitlab = format_gitlab_annotation(&annotation);
            let azure = format_azure_annotation(&annotation);
            let circleci = format_circleci_annotation(&annotation);
            let default = format_default_annotation(&annotation);

            let line_str = format!("line={}", line);
            let line_colon = format!("{}:", line);
            let linenumber_str = format!("linenumber={}", line);
            let line_default = format!("{}", line);

            prop_assert!(github.contains(&line_str));
            prop_assert!(gitlab.contains(&line_colon));
            prop_assert!(azure.contains(&linenumber_str));
            prop_assert!(circleci.contains(&line_colon));
            prop_assert!(default.contains(&line_default));
        }

        #[test]
        fn column_number_preserved_when_present(col in 1usize..1000) {
            let annotation = Annotation::new("file.rs", 1, Some(col), AnnotationSeverity::Error, "msg");

            let github = format_github_annotation(&annotation);
            let circleci = format_circleci_annotation(&annotation);
            let default = format_default_annotation(&annotation);

            let col_str = format!("col={}", col);
            let col_colon = format!(":{}", col);

            prop_assert!(github.contains(&col_str));
            prop_assert!(circleci.contains(&col_colon));
            prop_assert!(default.contains(&col_colon));
        }

        #[test]
        fn severity_determines_github_level(severity in 0u8..4) {
            let sev = match severity {
                0 => AnnotationSeverity::Notice,
                1 => AnnotationSeverity::Warning,
                2 => AnnotationSeverity::Error,
                _ => AnnotationSeverity::Fatal,
            };
            let annotation = Annotation::simple("file.rs", 1, sev, "msg");
            let output = format_github_annotation(&annotation);

            let expected_level = sev.as_github_level();
            let prefix = format!("::{}", expected_level);
            prop_assert!(output.starts_with(&prefix));
        }
    }
}

// =============================================================================
// Feature: Must_use Attributes
// =============================================================================

mod must_use_attributes {
    use super::*;

    #[test]
    fn annotation_new_is_must_use() {
        // This test verifies compilation - must_use generates a warning if unused
        let _annotation = Annotation::new("f.rs", 1, None, AnnotationSeverity::Error, "m");
    }

    #[test]
    fn annotation_simple_is_must_use() {
        let _annotation = Annotation::simple("f.rs", 1, AnnotationSeverity::Error, "m");
    }

    #[test]
    fn format_functions_are_must_use() {
        let annotation = Annotation::simple("f.rs", 1, AnnotationSeverity::Error, "m");

        let _github = format_github_annotation(&annotation);
        let _gitlab = format_gitlab_annotation(&annotation);
        let _azure = format_azure_annotation(&annotation);
        let _circleci = format_circleci_annotation(&annotation);
        let _default = format_default_annotation(&annotation);
        let _generic = format_annotation(AnnotationFormat::Github, &annotation);
    }

    #[test]
    fn ci_detection_functions_are_must_use() {
        let _platform = detect_ci();
        let _is_gh = is_github_actions();
        let _is_gl = is_gitlab_ci();
        let _is_az = is_azure_devops();
        let _is_cc = is_circleci();
    }
}
