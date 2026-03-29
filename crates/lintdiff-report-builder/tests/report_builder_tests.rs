//! BDD tests for lintdiff-report-builder.
//!
//! These tests follow the Given-When-Then pattern to describe behavior.

use lintdiff_report_builder::{
    quick_report, FileResult, Finding, GitInfo, ReportBuilder, ReportBuilderError, ReportSummary,
    Severity, ToolInfo,
};
use proptest::prelude::*;

// =============================================================================
// Feature: ReportBuilder Construction
// =============================================================================

mod report_builder_construction {
    use super::*;

    // Scenario: Creating a new builder and building empty report
    #[test]
    fn given_nothing_when_new_builder_created_then_can_build_minimal_report() {
        // Given: No preconditions
        // When
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .build();

        // Then
        assert!(report.is_ok());
        let report = report.unwrap();
        assert!(report.files.is_empty());
        assert!(report.git.is_none());
    }

    // Scenario: Setting tool info
    #[test]
    fn given_new_builder_when_with_tool_info_then_tool_info_in_report() {
        // Given
        let builder = ReportBuilder::new();

        // When
        let report = builder
            .with_tool_info("mylint", "2.0.0")
            .with_timestamp("2024-03-15T10:30:00Z")
            .build()
            .unwrap();

        // Then
        assert_eq!(report.tool.name, "mylint");
        assert_eq!(report.tool.version, "2.0.0");
    }

    // Scenario: Setting timestamp
    #[test]
    fn given_new_builder_when_with_timestamp_then_timestamp_in_report() {
        // Given
        let builder = ReportBuilder::new();

        // When
        let report = builder
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-03-15T10:30:00Z")
            .build()
            .unwrap();

        // Then
        assert_eq!(report.timestamp, "2024-03-15T10:30:00Z");
    }

    // Scenario: Setting git info with ref
    #[test]
    fn given_new_builder_when_with_git_info_with_ref_then_git_in_report() {
        // Given
        let builder = ReportBuilder::new();

        // When
        let report = builder
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .with_git_info("abc123def", Some("feature-branch"))
            .build()
            .unwrap();

        // Then
        assert!(report.git.is_some());
        let git = report.git.unwrap();
        assert_eq!(git.sha, "abc123def");
        assert_eq!(git.ref_name, Some("feature-branch".to_string()));
    }

    // Scenario: Setting git info without ref
    #[test]
    fn given_new_builder_when_with_git_info_no_ref_then_git_has_no_ref() {
        // Given
        let builder = ReportBuilder::new();

        // When
        let report = builder
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .with_git_info("abc123def", None)
            .build()
            .unwrap();

        // Then
        let git = report.git.unwrap();
        assert_eq!(git.sha, "abc123def");
        assert_eq!(git.ref_name, None);
    }

    // Scenario: Builder method chaining
    #[test]
    fn given_builder_when_chaining_methods_then_all_values_in_report() {
        // Given/When
        let report = ReportBuilder::new()
            .with_tool_info("tool", "1.0")
            .with_timestamp("2024-01-01T00:00:00Z")
            .with_git_info("sha123", Some("main"))
            .build()
            .unwrap();

        // Then
        assert_eq!(report.tool.name, "tool");
        assert_eq!(report.tool.version, "1.0");
        assert_eq!(report.timestamp, "2024-01-01T00:00:00Z");
        assert!(report.git.is_some());
    }
}

// =============================================================================
// Feature: FileResult Management
// =============================================================================

mod file_result_management {
    use super::*;

    // Scenario: Creating empty file result
    #[test]
    fn given_path_when_file_result_new_then_empty_result_created() {
        // Given
        let path = "src/main.rs";

        // When
        let result = FileResult::new(path);

        // Then
        assert_eq!(result.path, path);
        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
        assert!(result.unchanged.is_empty());
        assert!(!result.has_changes());
    }

    // Scenario: Adding findings to different lists
    #[test]
    fn given_file_result_when_adding_findings_then_lists_populated() {
        // Given
        let mut result = FileResult::new("test.rs");

        // When
        result.add_added(Finding::error("error1"));
        result.add_added(Finding::warning("warning1"));
        result.add_removed(Finding::hint("fixed1"));
        result.add_unchanged(Finding::note("unchanged1"));

        // Then
        assert_eq!(result.added.len(), 2);
        assert_eq!(result.removed.len(), 1);
        assert_eq!(result.unchanged.len(), 1);
        assert!(result.has_changes());
    }

    // Scenario: Counting total findings
    #[test]
    fn given_file_result_with_findings_when_total_count_then_returns_sum() {
        // Given
        let mut result = FileResult::new("test.rs");
        result.add_added(Finding::error("e1"));
        result.add_added(Finding::error("e2"));
        result.add_removed(Finding::warning("w1"));
        result.add_unchanged(Finding::hint("h1"));
        result.add_unchanged(Finding::hint("h2"));
        result.add_unchanged(Finding::hint("h3"));

        // When
        let total = result.total_count();

        // Then
        assert_eq!(total, 6);
    }

    // Scenario: Counting errors and warnings in added
    #[test]
    fn given_added_findings_when_counting_then_correct_counts() {
        // Given
        let mut result = FileResult::new("test.rs");
        result.add_added(Finding::error("e1"));
        result.add_added(Finding::error("e2"));
        result.add_added(Finding::fatal("f1"));
        result.add_added(Finding::warning("w1"));
        result.add_added(Finding::hint("h1"));

        // When
        let errors = result.added_errors();
        let warnings = result.added_warnings();

        // Then
        assert_eq!(errors, 3); // error + error + fatal
        assert_eq!(warnings, 1);
    }

    // Scenario: Detecting changes
    #[test]
    fn given_file_result_when_only_unchanged_then_no_changes() {
        // Given
        let mut result = FileResult::new("test.rs");
        result.add_unchanged(Finding::hint("h1"));
        result.add_unchanged(Finding::warning("w1"));

        // When/Then
        assert!(!result.has_changes());
        assert!(!result.has_added());
        assert!(!result.has_removed());
    }

    // Scenario: Adding findings via builder
    #[test]
    fn given_builder_when_add_finding_then_file_result_in_report() {
        // Given
        let builder = ReportBuilder::new();

        // When
        let report = builder
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("src/lib.rs", Finding::error("test error"))
            .build()
            .unwrap();

        // Then
        assert_eq!(report.files.len(), 1);
        assert_eq!(report.files[0].added.len(), 1);
    }

    // Scenario: Multiple findings to same file
    #[test]
    fn given_builder_when_multiple_findings_same_file_then_single_file_result() {
        // Given/When
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("src/lib.rs", Finding::error("e1"))
            .add_finding("src/lib.rs", Finding::warning("w1"))
            .add_finding("src/lib.rs", Finding::hint("h1"))
            .build()
            .unwrap();

        // Then
        assert_eq!(report.files.len(), 1);
        assert_eq!(report.files[0].added.len(), 3);
    }
}

// =============================================================================
// Feature: Finding Construction
// =============================================================================

mod finding_construction {
    use super::*;

    // Scenario: Creating basic finding
    #[test]
    fn given_message_and_severity_when_finding_new_then_finding_created() {
        // Given
        let message = "unused variable";
        let severity = Severity::Warning;

        // When
        let finding = Finding::new(message, severity);

        // Then
        assert_eq!(finding.message, message);
        assert_eq!(finding.severity, Severity::Warning);
        assert!(finding.code.is_none());
        assert!(finding.path.is_none());
        assert!(finding.line.is_none());
    }

    // Scenario: Using convenience constructors
    #[test]
    fn given_message_when_error_warning_hint_then_correct_severity() {
        // Given/When/Then
        assert_eq!(Finding::error("e").severity, Severity::Error);
        assert_eq!(Finding::warning("w").severity, Severity::Warning);
        assert_eq!(Finding::hint("h").severity, Severity::Hint);
        assert_eq!(Finding::note("n").severity, Severity::Note);
        assert_eq!(Finding::fatal("f").severity, Severity::Fatal);
    }

    // Scenario: Building finding with all attributes
    #[test]
    fn given_finding_when_with_all_attributes_then_all_set() {
        // Given
        let base = Finding::error("test error");

        // When
        let finding = base
            .with_code("E001")
            .with_path("src/lib.rs")
            .with_line(42)
            .with_column(10);

        // Then
        assert_eq!(finding.code, Some("E001".to_string()));
        assert_eq!(finding.path, Some("src/lib.rs".to_string()));
        assert_eq!(finding.line, Some(42));
        assert_eq!(finding.column, Some(10));
    }

    // Scenario: Setting location in one call
    #[test]
    fn given_finding_when_with_location_then_path_line_column_set() {
        // Given
        let base = Finding::warning("test");

        // When
        let finding = base.with_location("src/main.rs", 10, 5);

        // Then
        assert_eq!(finding.path, Some("src/main.rs".to_string()));
        assert_eq!(finding.line, Some(10));
        assert_eq!(finding.column, Some(5));
    }
}

// =============================================================================
// Feature: Severity Levels
// =============================================================================

mod severity_levels {
    use super::*;

    // Scenario: Severity ordering
    #[test]
    fn given_severities_when_compared_then_correct_order() {
        assert!(Severity::Fatal > Severity::Error);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Note);
        assert!(Severity::Note > Severity::Hint);
    }

    // Scenario: Problem detection
    #[test]
    fn given_severity_when_is_problem_then_correct_result() {
        assert!(!Severity::Hint.is_problem());
        assert!(!Severity::Note.is_problem());
        assert!(Severity::Warning.is_problem());
        assert!(Severity::Error.is_problem());
        assert!(Severity::Fatal.is_problem());
    }

    // Scenario: Blocking detection
    #[test]
    fn given_severity_when_is_blocking_then_correct_result() {
        assert!(!Severity::Hint.is_blocking());
        assert!(!Severity::Note.is_blocking());
        assert!(!Severity::Warning.is_blocking());
        assert!(Severity::Error.is_blocking());
        assert!(Severity::Fatal.is_blocking());
    }

    // Scenario: String representation
    #[test]
    fn given_severity_when_as_str_then_correct_string() {
        assert_eq!(Severity::Hint.as_str(), "hint");
        assert_eq!(Severity::Note.as_str(), "note");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Fatal.as_str(), "fatal");
    }

    // Scenario: Display trait
    #[test]
    fn given_severity_when_display_then_correct_output() {
        assert_eq!(format!("{}", Severity::Error), "error");
        assert_eq!(format!("{}", Severity::Warning), "warning");
    }
}

// =============================================================================
// Feature: Report Summary
// =============================================================================

mod report_summary {
    use super::*;

    // Scenario: Empty summary
    #[test]
    fn given_nothing_when_summary_new_then_all_zeros() {
        // Given/When
        let summary = ReportSummary::new();

        // Then
        assert_eq!(summary.total_added, 0);
        assert_eq!(summary.total_removed, 0);
        assert_eq!(summary.total_unchanged, 0);
        assert_eq!(summary.files_affected, 0);
    }

    // Scenario: Summary from counts
    #[test]
    fn given_counts_when_from_counts_then_summary_created() {
        // Given/When
        let summary = ReportSummary::from_counts(10, 5, 20, 3);

        // Then
        assert_eq!(summary.total_added, 10);
        assert_eq!(summary.total_removed, 5);
        assert_eq!(summary.total_unchanged, 20);
        assert_eq!(summary.files_affected, 3);
    }

    // Scenario: Total findings calculation
    #[test]
    fn given_summary_when_total_findings_then_sum_of_all() {
        // Given
        let summary = ReportSummary::from_counts(10, 5, 15, 2);

        // When
        let total = summary.total_findings();

        // Then
        assert_eq!(total, 30);
    }

    // Scenario: Has changes detection
    #[test]
    fn given_summary_when_has_changes_then_correct_result() {
        // No changes
        let no_changes = ReportSummary::from_counts(0, 0, 10, 0);
        assert!(!no_changes.has_changes());

        // With added
        let with_added = ReportSummary::from_counts(1, 0, 10, 1);
        assert!(with_added.has_changes());

        // With removed
        let with_removed = ReportSummary::from_counts(0, 1, 10, 1);
        assert!(with_removed.has_changes());
    }

    // Scenario: Net change calculation
    #[test]
    fn given_summary_when_net_change_then_added_minus_removed() {
        // Positive net
        let positive = ReportSummary::from_counts(10, 3, 0, 1);
        assert_eq!(positive.net_change(), 7);

        // Negative net
        let negative = ReportSummary::from_counts(2, 5, 0, 1);
        assert_eq!(negative.net_change(), -3);

        // Zero net
        let zero = ReportSummary::from_counts(5, 5, 0, 1);
        assert_eq!(zero.net_change(), 0);
    }

    // Scenario: Summary from file results
    #[test]
    fn given_file_results_when_from_file_results_then_correct_summary() {
        // Given
        let mut file1 = FileResult::new("a.rs");
        file1.add_added(Finding::error("e1"));
        file1.add_added(Finding::warning("w1"));
        file1.add_unchanged(Finding::hint("h1"));

        let mut file2 = FileResult::new("b.rs");
        file2.add_removed(Finding::error("e2"));

        let file3 = FileResult::new("c.rs"); // No changes

        // When
        let summary = ReportSummary::from_file_results(&[file1, file2, file3]);

        // Then
        assert_eq!(summary.total_added, 2);
        assert_eq!(summary.total_removed, 1);
        assert_eq!(summary.total_unchanged, 1);
        assert_eq!(summary.files_affected, 2); // file1 and file2 have changes
    }

    // Scenario: Empty file results
    #[test]
    fn given_no_file_results_when_from_file_results_then_empty_summary() {
        // Given/When
        let summary = ReportSummary::from_file_results(&[]);

        // Then
        assert_eq!(summary.total_added, 0);
        assert_eq!(summary.total_removed, 0);
        assert_eq!(summary.total_unchanged, 0);
        assert_eq!(summary.files_affected, 0);
    }
}

// =============================================================================
// Feature: Report Building
// =============================================================================

mod report_building {
    use super::*;

    // Scenario: Building minimal report
    #[test]
    fn given_minimal_builder_when_build_then_valid_report() {
        // Given
        let builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z");

        // When
        let report = builder.build().unwrap();

        // Then
        assert_eq!(report.tool.name, "lintdiff");
        assert_eq!(report.tool.version, "1.0.0");
        assert_eq!(report.timestamp, "2024-01-15T10:30:00Z");
        assert!(report.git.is_none());
        assert!(report.files.is_empty());
    }

    // Scenario: Building report with git info
    #[test]
    fn given_builder_with_git_when_build_then_report_has_git() {
        // Given
        let builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .with_git_info("abc123", Some("main"));

        // When
        let report = builder.build().unwrap();

        // Then
        assert!(report.git.is_some());
        let git = report.git.unwrap();
        assert_eq!(git.sha, "abc123");
        assert_eq!(git.ref_name, Some("main".to_string()));
    }

    // Scenario: Building report with findings
    #[test]
    fn given_builder_with_findings_when_build_then_report_has_files() {
        // Given
        let builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("src/lib.rs", Finding::error("e1"))
            .add_finding("src/main.rs", Finding::warning("w1"));

        // When
        let report = builder.build().unwrap();

        // Then
        assert_eq!(report.files.len(), 2);
        assert_eq!(report.summary.total_added, 2);
    }

    // Scenario: Files are sorted by path
    #[test]
    fn given_unordered_files_when_build_then_files_sorted() {
        // Given
        let builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("z.rs", Finding::error("z"))
            .add_finding("a.rs", Finding::error("a"))
            .add_finding("m.rs", Finding::error("m"));

        // When
        let report = builder.build().unwrap();

        // Then
        assert_eq!(report.files[0].path, "a.rs");
        assert_eq!(report.files[1].path, "m.rs");
        assert_eq!(report.files[2].path, "z.rs");
    }

    // Scenario: Custom summary overrides calculation
    #[test]
    fn given_custom_summary_when_build_then_uses_custom() {
        // Given
        let custom = ReportSummary::from_counts(100, 50, 200, 10);
        let builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("a.rs", Finding::error("a"))
            .add_summary(custom.clone());

        // When
        let report = builder.build().unwrap();

        // Then
        assert_eq!(report.summary, custom);
    }

    // Scenario: Quick report function
    #[test]
    fn given_quick_report_params_when_quick_report_then_valid_report() {
        // Given/When
        let report = quick_report("mytool", "2.0.0", "2024-01-15T10:30:00Z");

        // Then
        assert_eq!(report.tool.name, "mytool");
        assert_eq!(report.tool.version, "2.0.0");
        assert_eq!(report.timestamp, "2024-01-15T10:30:00Z");
        assert!(report.git.is_none());
        assert!(report.is_empty());
    }
}

// =============================================================================
// Feature: Validation
// =============================================================================

mod validation {
    use super::*;

    // Scenario: Missing tool name
    #[test]
    fn given_no_tool_name_when_validate_then_missing_tool_name_error() {
        // Given
        let builder = ReportBuilder::new().with_timestamp("2024-01-15T10:30:00Z");

        // When
        let err = builder.validate().unwrap_err();

        // Then
        assert!(matches!(err, ReportBuilderError::MissingToolName));
    }

    // Scenario: Empty tool name
    #[test]
    fn given_empty_tool_name_when_validate_then_missing_tool_name_error() {
        // Given
        let builder = ReportBuilder::new()
            .with_tool_info("", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z");

        // When
        let err = builder.validate().unwrap_err();

        // Then
        assert!(err.is_missing_tool_name());
    }

    // Scenario: Missing tool version
    #[test]
    fn given_no_tool_version_when_validate_then_missing_tool_version_error() {
        // Given
        let builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "")
            .with_timestamp("2024-01-15T10:30:00Z");

        // When
        let err = builder.validate().unwrap_err();

        // Then
        assert!(err.is_missing_tool_version());
    }

    // Scenario: Missing timestamp
    #[test]
    fn given_no_timestamp_when_validate_then_missing_timestamp_error() {
        // Given
        let builder = ReportBuilder::new().with_tool_info("lintdiff", "1.0.0");

        // When
        let err = builder.validate().unwrap_err();

        // Then
        assert!(err.is_missing_timestamp());
    }

    // Scenario: Invalid timestamp format (too short)
    #[test]
    fn given_invalid_timestamp_when_validate_then_invalid_timestamp_error() {
        // Given - timestamp without date separator
        let builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("notadate"); // No dash, fails validation

        // When
        let err = builder.validate().unwrap_err();

        // Then
        assert!(err.is_invalid_timestamp());
    }

    // Scenario: Timestamp too short
    #[test]
    fn given_short_timestamp_when_validate_then_invalid_timestamp_error() {
        // Given
        let builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024");

        // When
        let err = builder.validate().unwrap_err();

        // Then
        assert!(err.is_invalid_timestamp());
    }

    // Scenario: Valid builder passes validation
    #[test]
    fn given_valid_builder_when_validate_then_ok() {
        // Given
        let builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z");

        // When/Then
        assert!(builder.validate().is_ok());
    }

    // Scenario: Error type checking methods
    #[test]
    fn given_errors_when_checking_types_then_correct_results() {
        // MissingToolName
        let err = ReportBuilderError::MissingToolName;
        assert!(err.is_missing_tool_name());
        assert!(!err.is_missing_tool_version());
        assert!(!err.is_missing_timestamp());
        assert!(!err.is_invalid_timestamp());

        // MissingToolVersion
        let err = ReportBuilderError::MissingToolVersion;
        assert!(!err.is_missing_tool_name());
        assert!(err.is_missing_tool_version());
        assert!(!err.is_missing_timestamp());
        assert!(!err.is_invalid_timestamp());

        // MissingTimestamp
        let err = ReportBuilderError::MissingTimestamp;
        assert!(!err.is_missing_tool_name());
        assert!(!err.is_missing_tool_version());
        assert!(err.is_missing_timestamp());
        assert!(!err.is_invalid_timestamp());

        // InvalidTimestampFormat
        let err = ReportBuilderError::InvalidTimestampFormat("bad".to_string());
        assert!(!err.is_missing_tool_name());
        assert!(!err.is_missing_tool_version());
        assert!(!err.is_missing_timestamp());
        assert!(err.is_invalid_timestamp());
    }
}

// =============================================================================
// Feature: Report Accessors
// =============================================================================

mod report_accessors {
    use super::*;

    // Scenario: File count
    #[test]
    fn given_report_with_files_when_file_count_then_correct_number() {
        // Given
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("a.rs", Finding::error("a"))
            .add_finding("b.rs", Finding::error("b"))
            .add_finding("c.rs", Finding::error("c"))
            .build()
            .unwrap();

        // When/Then
        assert_eq!(report.file_count(), 3);
    }

    // Scenario: Is empty check
    #[test]
    fn given_report_when_is_empty_then_correct_result() {
        // Empty
        let empty = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .build()
            .unwrap();
        assert!(empty.is_empty());

        // Not empty
        let not_empty = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("a.rs", Finding::error("a"))
            .build()
            .unwrap();
        assert!(!not_empty.is_empty());
    }

    // Scenario: Get file by path
    #[test]
    fn given_report_when_get_file_then_correct_file_or_none() {
        // Given
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("src/lib.rs", Finding::error("test"))
            .build()
            .unwrap();

        // When/Then
        assert!(report.get_file("src/lib.rs").is_some());
        assert!(report.get_file("nonexistent.rs").is_none());
    }

    // Scenario: Has added/removed
    #[test]
    fn given_report_when_has_added_removed_then_correct_result() {
        // With added
        let with_added = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("a.rs", Finding::error("a"))
            .build()
            .unwrap();
        assert!(with_added.has_added());
        assert!(!with_added.has_removed());

        // With removed
        let with_removed = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_removed_finding("a.rs", Finding::error("a"))
            .build()
            .unwrap();
        assert!(!with_removed.has_added());
        assert!(with_removed.has_removed());

        // With both
        let with_both = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("a.rs", Finding::error("a"))
            .add_removed_finding("a.rs", Finding::warning("w"))
            .build()
            .unwrap();
        assert!(with_both.has_added());
        assert!(with_both.has_removed());
    }
}

// =============================================================================
// Feature: ToolInfo and GitInfo
// =============================================================================

mod info_types {
    use super::*;

    // Scenario: ToolInfo creation
    #[test]
    fn given_name_and_version_when_tool_info_new_then_created() {
        // Given/When
        let info = ToolInfo::new("mylint", "3.0.0");

        // Then
        assert_eq!(info.name, "mylint");
        assert_eq!(info.version, "3.0.0");
    }

    // Scenario: GitInfo from SHA only
    #[test]
    fn given_sha_when_git_info_from_sha_then_no_ref() {
        // Given/When
        let git = GitInfo::from_sha("abc123");

        // Then
        assert_eq!(git.sha, "abc123");
        assert_eq!(git.ref_name, None);
    }

    // Scenario: GitInfo with ref
    #[test]
    fn given_sha_and_ref_when_git_info_new_then_both_set() {
        // Given/When
        let git = GitInfo::new("def456", Some("develop"));

        // Then
        assert_eq!(git.sha, "def456");
        assert_eq!(git.ref_name, Some("develop".to_string()));
    }

    // Scenario: Equality
    #[test]
    fn test_info_equality() {
        let info1 = ToolInfo::new("tool", "1.0");
        let info2 = ToolInfo::new("tool", "1.0");
        let info3 = ToolInfo::new("tool", "2.0");
        assert_eq!(info1, info2);
        assert_ne!(info1, info3);

        let git1 = GitInfo::new("sha1", Some("main"));
        let git2 = GitInfo::new("sha1", Some("main"));
        let git3 = GitInfo::new("sha2", Some("main"));
        assert_eq!(git1, git2);
        assert_ne!(git1, git3);
    }
}

// =============================================================================
// Feature: Edge Cases
// =============================================================================

mod edge_cases {
    use super::*;

    // Scenario: Empty report
    #[test]
    fn given_no_findings_when_build_then_empty_valid_report() {
        // Given/When
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .build()
            .unwrap();

        // Then
        assert!(report.is_empty());
        assert_eq!(report.summary.total_added, 0);
        assert_eq!(report.summary.total_removed, 0);
        assert_eq!(report.summary.files_affected, 0);
    }

    // Scenario: Only unchanged findings
    #[test]
    fn given_only_unchanged_when_build_then_no_changes() {
        // Given/When
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_unchanged_finding("a.rs", Finding::warning("w1"))
            .add_unchanged_finding("a.rs", Finding::hint("h1"))
            .build()
            .unwrap();

        // Then
        assert!(!report.is_empty());
        assert!(!report.has_added());
        assert!(!report.has_removed());
        assert_eq!(report.summary.total_unchanged, 2);
        assert_eq!(report.summary.files_affected, 0);
    }

    // Scenario: Many files
    #[test]
    fn given_many_files_when_build_then_all_included() {
        // Given
        let mut builder = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z");

        for i in 0..100 {
            builder = builder.add_finding(&format!("file{}.rs", i), Finding::error("e"));
        }

        // When
        let report = builder.build().unwrap();

        // Then
        assert_eq!(report.files.len(), 100);
        assert_eq!(report.summary.total_added, 100);
        assert_eq!(report.summary.files_affected, 100);
    }

    // Scenario: Same file multiple finding types
    #[test]
    fn given_same_file_all_types_when_build_then_all_counted() {
        // Given/When
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("lib.rs", Finding::error("added"))
            .add_removed_finding("lib.rs", Finding::warning("removed"))
            .add_unchanged_finding("lib.rs", Finding::hint("unchanged"))
            .build()
            .unwrap();

        // Then
        assert_eq!(report.files.len(), 1);
        let file = &report.files[0];
        assert_eq!(file.added.len(), 1);
        assert_eq!(file.removed.len(), 1);
        assert_eq!(file.unchanged.len(), 1);
        assert_eq!(report.summary.total_added, 1);
        assert_eq!(report.summary.total_removed, 1);
        assert_eq!(report.summary.total_unchanged, 1);
    }

    // Scenario: Long paths
    #[test]
    fn given_long_path_when_build_then_handled_correctly() {
        // Given
        let long_path = "src/very/deeply/nested/directory/structure/with/many/components/file.rs";

        // When
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding(long_path, Finding::error("e"))
            .build()
            .unwrap();

        // Then
        assert_eq!(report.files[0].path, long_path);
    }

    // Scenario: Unicode in messages
    #[test]
    fn given_unicode_message_when_build_then_handled_correctly() {
        // Given/When
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .add_finding("lib.rs", Finding::error("Error: 中文测试 🎉"))
            .build()
            .unwrap();

        // Then
        assert_eq!(report.files[0].added[0].message, "Error: 中文测试 🎉");
    }
}

// =============================================================================
// Feature: Property-Based Tests
// =============================================================================

mod property_tests {
    use super::*;

    proptest! {
        // Finding message roundtrip
        #[test]
        fn finding_message_preserved(msg in ".*") {
            let finding = Finding::error(&msg);
            prop_assert_eq!(finding.message, msg);
        }

        // Tool name preserved in report
        #[test]
        fn tool_name_preserved_in_report(name in "[a-zA-Z0-9_-]+") {
            let report = ReportBuilder::new()
                .with_tool_info(&name, "1.0.0")
                .with_timestamp("2024-01-15T10:30:00Z")
                .build()
                .unwrap();
            prop_assert_eq!(report.tool.name, name);
        }

        // Tool version preserved in report
        #[test]
        fn tool_version_preserved_in_report(version in "[0-9]+(\\.[0-9]+)*") {
            let report = ReportBuilder::new()
                .with_tool_info("tool", &version)
                .with_timestamp("2024-01-15T10:30:00Z")
                .build()
                .unwrap();
            prop_assert_eq!(report.tool.version, version);
        }

        // File count matches added files
        #[test]
        fn file_count_matches_added(file_count in 0usize..20) {
            let mut builder = ReportBuilder::new()
                .with_tool_info("tool", "1.0")
                .with_timestamp("2024-01-15T10:30:00Z");

            for i in 0..file_count {
                builder = builder.add_finding(&format!("file{}.rs", i), Finding::error("e"));
            }

            let report = builder.build().unwrap();
            prop_assert_eq!(report.files.len(), file_count);
        }

        // Summary totals are non-negative
        #[test]
        fn summary_totals_nonnegative(
            added in 0usize..100,
            removed in 0usize..100,
            unchanged in 0usize..100
        ) {
            let summary = ReportSummary::from_counts(added, removed, unchanged, 0);
            prop_assert_eq!(summary.total_added, added);
            prop_assert_eq!(summary.total_removed, removed);
            prop_assert_eq!(summary.total_unchanged, unchanged);
            prop_assert!(summary.total_findings() >= 0);
        }

        // Net change calculation
        #[test]
        fn net_change_is_added_minus_removed(added in 0usize..1000, removed in 0usize..1000) {
            let summary = ReportSummary::from_counts(added, removed, 0, 1);
            let expected = added as isize - removed as isize;
            prop_assert_eq!(summary.net_change(), expected);
        }

        // Severity ordering is transitive
        #[test]
        fn severity_ordering_transitive(
            a in 0u8..5,
            b in 0u8..5,
            c in 0u8..5
        ) {
            let severities = [Severity::Hint, Severity::Note, Severity::Warning, Severity::Error, Severity::Fatal];
            let sa = severities[a as usize];
            let sb = severities[b as usize];
            let sc = severities[c as usize];

            if sa >= sb && sb >= sc {
                prop_assert!(sa >= sc);
            }
            if sa <= sb && sb <= sc {
                prop_assert!(sa <= sc);
            }
        }

        // Line and column preserved
        #[test]
        fn line_column_preserved(line in 1u32..10000, col in 1u32..500) {
            let finding = Finding::error("test")
                .with_line(line)
                .with_column(col);
            prop_assert_eq!(finding.line, Some(line));
            prop_assert_eq!(finding.column, Some(col));
        }

        // File result total count
        #[test]
        fn file_result_total_count(
            added in 0usize..10,
            removed in 0usize..10,
            unchanged in 0usize..10
        ) {
            let mut result = FileResult::new("test.rs");
            for _ in 0..added {
                result.add_added(Finding::error("e"));
            }
            for _ in 0..removed {
                result.add_removed(Finding::warning("w"));
            }
            for _ in 0..unchanged {
                result.add_unchanged(Finding::hint("h"));
            }
            prop_assert_eq!(result.total_count(), added + removed + unchanged);
        }
    }
}

// =============================================================================
// Feature: Serde Serialization (conditional)
// =============================================================================

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    #[test]
    fn test_severity_serde() {
        let severity = Severity::Warning;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, r#""warning""#);

        let deserialized: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Severity::Warning);
    }

    #[test]
    fn test_finding_serde() {
        let finding = Finding::error("test error").with_code("E001").with_line(42);

        let json = serde_json::to_string(&finding).unwrap();
        assert!(json.contains("test error"));
        assert!(json.contains("E001"));

        let deserialized: Finding = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.message, "test error");
        assert_eq!(deserialized.code, Some("E001".to_string()));
        assert_eq!(deserialized.line, Some(42));
    }

    #[test]
    fn test_file_result_serde() {
        let mut result = FileResult::new("src/lib.rs");
        result.add_added(Finding::error("e1"));

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("src/lib.rs"));

        let deserialized: FileResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.path, "src/lib.rs");
        assert_eq!(deserialized.added.len(), 1);
    }

    #[test]
    fn test_report_summary_serde() {
        let summary = ReportSummary::from_counts(10, 5, 20, 3);

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: ReportSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_added, 10);
        assert_eq!(deserialized.total_removed, 5);
        assert_eq!(deserialized.total_unchanged, 20);
        assert_eq!(deserialized.files_affected, 3);
    }

    #[test]
    fn test_tool_info_serde() {
        let info = ToolInfo::new("lintdiff", "1.0.0");

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ToolInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "lintdiff");
        assert_eq!(deserialized.version, "1.0.0");
    }

    #[test]
    fn test_git_info_serde() {
        let git = GitInfo::new("abc123", Some("main"));

        let json = serde_json::to_string(&git).unwrap();
        let deserialized: GitInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.sha, "abc123");
        assert_eq!(deserialized.ref_name, Some("main".to_string()));
    }

    #[test]
    fn test_report_serde() {
        let report = ReportBuilder::new()
            .with_tool_info("lintdiff", "1.0.0")
            .with_timestamp("2024-01-15T10:30:00Z")
            .with_git_info("abc123", Some("main"))
            .add_finding("src/lib.rs", Finding::error("test"))
            .build()
            .unwrap();

        let json = serde_json::to_string(&report).unwrap();
        let deserialized: lintdiff_report_builder::Report = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tool.name, "lintdiff");
        assert_eq!(deserialized.timestamp, "2024-01-15T10:30:00Z");
        assert!(deserialized.git.is_some());
        assert_eq!(deserialized.files.len(), 1);
    }
}

// =============================================================================
// Feature: Error Messages
// =============================================================================

mod error_messages {
    use super::*;

    #[test]
    fn test_missing_tool_name_message() {
        let err = ReportBuilderError::MissingToolName;
        assert!(err.to_string().contains("Tool name"));
    }

    #[test]
    fn test_missing_tool_version_message() {
        let err = ReportBuilderError::MissingToolVersion;
        assert!(err.to_string().contains("Tool version"));
    }

    #[test]
    fn test_missing_timestamp_message() {
        let err = ReportBuilderError::MissingTimestamp;
        assert!(err.to_string().contains("Timestamp"));
    }

    #[test]
    fn test_invalid_timestamp_message() {
        let err = ReportBuilderError::InvalidTimestampFormat("bad-date".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Invalid timestamp"));
        assert!(msg.contains("bad-date"));
    }

    #[test]
    fn test_empty_file_path_message() {
        let err = ReportBuilderError::EmptyFilePath;
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn test_duplicate_file_path_message() {
        let err = ReportBuilderError::DuplicateFilePath("test.rs".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Duplicate"));
        assert!(msg.contains("test.rs"));
    }
}

// =============================================================================
// Feature: Default Implementations
// =============================================================================

mod defaults {
    use super::*;

    #[test]
    fn test_severity_default() {
        let severity = Severity::default();
        assert_eq!(severity, Severity::Warning);
    }

    #[test]
    fn test_report_summary_default() {
        let summary = ReportSummary::default();
        assert_eq!(summary.total_added, 0);
        assert_eq!(summary.total_removed, 0);
        assert_eq!(summary.total_unchanged, 0);
        assert_eq!(summary.files_affected, 0);
    }

    #[test]
    fn test_report_builder_default() {
        let report = ReportBuilder::default()
            .with_tool_info("test", "1.0")
            .with_timestamp("2024-01-01T00:00:00Z")
            .build()
            .unwrap();
        assert!(report.is_empty());
    }
}
