//! Integration tests for lintdiff-stats crate.

use std::collections::HashMap;

use lintdiff_stats::{Stats, StatsSource};
use lintdiff_types::{Finding, Location, NormPath, Severity};

fn create_finding(severity: Severity, code: &str, path: &str) -> Finding {
    Finding {
        severity,
        code: code.to_string(),
        message: "test message".to_string(),
        location: Some(Location {
            path: NormPath::new(path),
            line: Some(1),
            col: None,
        }),
        check_id: None,
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

fn create_finding_no_location(severity: Severity, code: &str) -> Finding {
    Finding {
        severity,
        code: code.to_string(),
        message: "test message".to_string(),
        location: None,
        check_id: None,
        help: None,
        url: None,
        fingerprint: None,
        data: None,
    }
}

mod new_and_default {
    use super::*;

    #[test]
    fn new_returns_empty_stats() {
        let stats = Stats::new();

        assert_eq!(stats.total_diagnostics, 0);
        assert_eq!(stats.matched_diagnostics, 0);
        assert_eq!(stats.filtered_diagnostics, 0);
        assert!(stats.by_severity.is_empty());
        assert!(stats.by_code.is_empty());
        assert_eq!(stats.files_affected, 0);
    }

    #[test]
    fn default_returns_empty_stats() {
        let stats = Stats::default();

        assert_eq!(stats.total_diagnostics, 0);
        assert_eq!(stats.matched_diagnostics, 0);
        assert_eq!(stats.filtered_diagnostics, 0);
    }

    #[test]
    fn new_and_default_are_equivalent() {
        let new_stats = Stats::new();
        let default_stats = Stats::default();

        assert_eq!(new_stats.total_diagnostics, default_stats.total_diagnostics);
        assert_eq!(
            new_stats.matched_diagnostics,
            default_stats.matched_diagnostics
        );
        assert_eq!(
            new_stats.filtered_diagnostics,
            default_stats.filtered_diagnostics
        );
        assert_eq!(new_stats.files_affected, default_stats.files_affected);
    }
}

mod from_findings {
    use super::*;

    #[test]
    fn empty_findings_produces_empty_stats() {
        let findings: Vec<Finding> = vec![];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.total_diagnostics, 0);
        assert_eq!(stats.matched_diagnostics, 0);
        assert_eq!(stats.filtered_diagnostics, 0);
        assert_eq!(stats.files_affected, 0);
    }

    #[test]
    fn single_finding_sets_total_and_matched_to_one() {
        let findings = vec![create_finding(Severity::Error, "E001", "src/lib.rs")];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.total_diagnostics, 1);
        assert_eq!(stats.matched_diagnostics, 1);
        assert_eq!(stats.filtered_diagnostics, 0);
    }

    #[test]
    fn severity_error_is_counted_correctly() {
        let findings = vec![create_finding(Severity::Error, "E001", "src/lib.rs")];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.by_severity.get("error"), Some(&1));
        assert_eq!(stats.by_severity.get("warning"), None);
        assert_eq!(stats.by_severity.get("info"), None);
    }

    #[test]
    fn severity_warning_is_counted_correctly() {
        let findings = vec![create_finding(Severity::Warn, "W001", "src/lib.rs")];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.by_severity.get("warning"), Some(&1));
        assert_eq!(stats.by_severity.get("error"), None);
        assert_eq!(stats.by_severity.get("info"), None);
    }

    #[test]
    fn severity_info_is_counted_correctly() {
        let findings = vec![create_finding(Severity::Info, "I001", "src/lib.rs")];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.by_severity.get("info"), Some(&1));
        assert_eq!(stats.by_severity.get("error"), None);
        assert_eq!(stats.by_severity.get("warning"), None);
    }

    #[test]
    fn multiple_severities_are_aggregated() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Error, "E002", "src/lib.rs"),
            create_finding(Severity::Warn, "W001", "src/lib.rs"),
            create_finding(Severity::Info, "I001", "src/lib.rs"),
        ];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.by_severity.get("error"), Some(&2));
        assert_eq!(stats.by_severity.get("warning"), Some(&1));
        assert_eq!(stats.by_severity.get("info"), Some(&1));
    }

    #[test]
    fn code_is_counted_correctly() {
        let findings = vec![create_finding(Severity::Error, "clippy::unwrap_used", "src/lib.rs")];
        let stats = Stats::from_findings(&findings);

        assert_eq!(
            stats.by_code.get("clippy::unwrap_used"),
            Some(&1)
        );
    }

    #[test]
    fn same_code_multiple_times_is_aggregated() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Error, "E001", "src/main.rs"),
            create_finding(Severity::Error, "E001", "src/utils.rs"),
        ];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.by_code.get("E001"), Some(&3));
    }

    #[test]
    fn different_codes_are_tracked_separately() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Error, "E002", "src/lib.rs"),
            create_finding(Severity::Warn, "W001", "src/lib.rs"),
        ];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.by_code.get("E001"), Some(&1));
        assert_eq!(stats.by_code.get("E002"), Some(&1));
        assert_eq!(stats.by_code.get("W001"), Some(&1));
    }

    #[test]
    fn single_file_is_counted_correctly() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Warn, "W001", "src/lib.rs"),
        ];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.files_affected, 1);
    }

    #[test]
    fn multiple_files_are_counted_correctly() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Warn, "W001", "src/main.rs"),
            create_finding(Severity::Info, "I001", "src/utils.rs"),
        ];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.files_affected, 3);
    }

    #[test]
    fn finding_without_location_does_not_contribute_to_file_count() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding_no_location(Severity::Warn, "W001"),
        ];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.files_affected, 1);
    }

    #[test]
    fn all_findings_without_location_result_in_zero_files() {
        let findings = vec![
            create_finding_no_location(Severity::Error, "E001"),
            create_finding_no_location(Severity::Warn, "W001"),
        ];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.files_affected, 0);
        assert_eq!(stats.total_diagnostics, 2); // Still counted in total
    }
}

mod merge {
    use super::*;

    #[test]
    fn merging_empty_stats_does_not_change_original() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 10;
        stats.matched_diagnostics = 5;

        let empty = Stats::new();
        stats.merge(&empty);

        assert_eq!(stats.total_diagnostics, 10);
        assert_eq!(stats.matched_diagnostics, 5);
    }

    #[test]
    fn merging_into_empty_stats_copies_values() {
        let mut stats = Stats::new();
        let mut other = Stats::new();
        other.total_diagnostics = 10;
        other.matched_diagnostics = 5;
        other.filtered_diagnostics = 5;

        stats.merge(&other);

        assert_eq!(stats.total_diagnostics, 10);
        assert_eq!(stats.matched_diagnostics, 5);
        assert_eq!(stats.filtered_diagnostics, 5);
    }

    #[test]
    fn merge_adds_total_diagnostics() {
        let mut stats1 = Stats::new();
        stats1.total_diagnostics = 10;

        let mut stats2 = Stats::new();
        stats2.total_diagnostics = 20;

        stats1.merge(&stats2);

        assert_eq!(stats1.total_diagnostics, 30);
    }

    #[test]
    fn merge_adds_matched_diagnostics() {
        let mut stats1 = Stats::new();
        stats1.matched_diagnostics = 5;

        let mut stats2 = Stats::new();
        stats2.matched_diagnostics = 10;

        stats1.merge(&stats2);

        assert_eq!(stats1.matched_diagnostics, 15);
    }

    #[test]
    fn merge_adds_filtered_diagnostics() {
        let mut stats1 = Stats::new();
        stats1.filtered_diagnostics = 3;

        let mut stats2 = Stats::new();
        stats2.filtered_diagnostics = 7;

        stats1.merge(&stats2);

        assert_eq!(stats1.filtered_diagnostics, 10);
    }

    #[test]
    fn merge_combines_severity_counts() {
        let mut stats1 = Stats::new();
        stats1.by_severity = HashMap::from([("error".to_string(), 3), ("warning".to_string(), 2)]);

        let mut stats2 = Stats::new();
        stats2.by_severity = HashMap::from([("error".to_string(), 5), ("info".to_string(), 1)]);

        stats1.merge(&stats2);

        assert_eq!(stats1.by_severity.get("error"), Some(&8));
        assert_eq!(stats1.by_severity.get("warning"), Some(&2));
        assert_eq!(stats1.by_severity.get("info"), Some(&1));
    }

    #[test]
    fn merge_combines_code_counts() {
        let mut stats1 = Stats::new();
        stats1.by_code = HashMap::from([("E001".to_string(), 2)]);

        let mut stats2 = Stats::new();
        stats2.by_code = HashMap::from([("E001".to_string(), 3), ("W001".to_string(), 1)]);

        stats1.merge(&stats2);

        assert_eq!(stats1.by_code.get("E001"), Some(&5));
        assert_eq!(stats1.by_code.get("W001"), Some(&1));
    }

    #[test]
    fn merge_takes_max_files_affected() {
        let mut stats1 = Stats::new();
        stats1.files_affected = 5;

        let mut stats2 = Stats::new();
        stats2.files_affected = 10;

        stats1.merge(&stats2);

        assert_eq!(stats1.files_affected, 10);
    }

    #[test]
    fn merge_full_integration() {
        let mut stats1 = Stats {
            total_diagnostics: 100,
            matched_diagnostics: 60,
            filtered_diagnostics: 40,
            by_severity: HashMap::from([
                ("error".to_string(), 20),
                ("warning".to_string(), 40),
            ]),
            by_code: HashMap::from([
                ("E001".to_string(), 10),
                ("W001".to_string(), 30),
            ]),
            files_affected: 15,
        };

        let stats2 = Stats {
            total_diagnostics: 50,
            matched_diagnostics: 30,
            filtered_diagnostics: 20,
            by_severity: HashMap::from([
                ("error".to_string(), 10),
                ("info".to_string(), 20),
            ]),
            by_code: HashMap::from([
                ("E001".to_string(), 5),
                ("I001".to_string(), 15),
            ]),
            files_affected: 20,
        };

        stats1.merge(&stats2);

        assert_eq!(stats1.total_diagnostics, 150);
        assert_eq!(stats1.matched_diagnostics, 90);
        assert_eq!(stats1.filtered_diagnostics, 60);
        assert_eq!(stats1.by_severity.get("error"), Some(&30));
        assert_eq!(stats1.by_severity.get("warning"), Some(&40));
        assert_eq!(stats1.by_severity.get("info"), Some(&20));
        assert_eq!(stats1.by_code.get("E001"), Some(&15));
        assert_eq!(stats1.by_code.get("W001"), Some(&30));
        assert_eq!(stats1.by_code.get("I001"), Some(&15));
        assert_eq!(stats1.files_affected, 20);
    }
}

mod pass_rate {
    use super::*;

    #[test]
    fn empty_stats_returns_one() {
        let stats = Stats::new();
        assert_eq!(stats.pass_rate(), 1.0);
    }

    #[test]
    fn zero_total_diagnostics_returns_one() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 0;
        stats.matched_diagnostics = 0;

        assert_eq!(stats.pass_rate(), 1.0);
    }

    #[test]
    fn full_match_returns_one() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 100;
        stats.matched_diagnostics = 100;

        assert_eq!(stats.pass_rate(), 1.0);
    }

    #[test]
    fn half_match_returns_point_five() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 10;
        stats.matched_diagnostics = 5;

        let rate = stats.pass_rate();
        assert!((rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn quarter_match_returns_point_two_five() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 100;
        stats.matched_diagnostics = 25;

        let rate = stats.pass_rate();
        assert!((rate - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn no_match_returns_zero() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 100;
        stats.matched_diagnostics = 0;

        assert_eq!(stats.pass_rate(), 0.0);
    }

    #[test]
    fn three_quarters_match() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 100;
        stats.matched_diagnostics = 75;

        let rate = stats.pass_rate();
        assert!((rate - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn one_third_match() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 3;
        stats.matched_diagnostics = 1;

        let rate = stats.pass_rate();
        assert!((rate - 1.0 / 3.0).abs() < f64::EPSILON);
    }
}

mod stats_source_trait {
    use super::*;

    #[test]
    fn stats_source_for_stats_returns_clone() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 42;
        stats.matched_diagnostics = 21;

        let result = stats.stats();

        assert_eq!(result.total_diagnostics, 42);
        assert_eq!(result.matched_diagnostics, 21);
    }

    #[test]
    fn stats_source_for_slice_returns_correct_stats() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Warn, "W001", "src/main.rs"),
        ];

        let stats: Stats = findings.as_slice().stats();

        assert_eq!(stats.total_diagnostics, 2);
        assert_eq!(stats.matched_diagnostics, 2);
        assert_eq!(stats.files_affected, 2);
    }

    #[test]
    fn stats_source_for_vec_returns_correct_stats() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Warn, "W001", "src/main.rs"),
        ];

        let stats = findings.stats();

        assert_eq!(stats.total_diagnostics, 2);
        assert_eq!(stats.matched_diagnostics, 2);
    }

    #[test]
    fn stats_source_for_empty_slice_returns_empty_stats() {
        let findings: Vec<Finding> = vec![];

        let stats: Stats = findings.as_slice().stats();

        assert_eq!(stats.total_diagnostics, 0);
    }
}

mod serialization {
    use super::*;

    #[test]
    fn stats_serializes_to_json() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 10;
        stats.matched_diagnostics = 8;
        stats.filtered_diagnostics = 2;
        stats.files_affected = 3;

        let json = serde_json::to_string(&stats).unwrap();

        assert!(json.contains("\"total_diagnostics\":10"));
        assert!(json.contains("\"matched_diagnostics\":8"));
        assert!(json.contains("\"filtered_diagnostics\":2"));
        assert!(json.contains("\"files_affected\":3"));
    }

    #[test]
    fn stats_serializes_severity_map() {
        let mut stats = Stats::new();
        stats.by_severity = HashMap::from([
            ("error".to_string(), 5),
            ("warning".to_string(), 3),
        ]);

        let json = serde_json::to_string(&stats).unwrap();

        assert!(json.contains("\"by_severity\""));
        assert!(json.contains("\"error\":5") || json.contains("\"error\": 5"));
        assert!(json.contains("\"warning\":3") || json.contains("\"warning\": 3"));
    }

    #[test]
    fn stats_serializes_code_map() {
        let mut stats = Stats::new();
        stats.by_code = HashMap::from([
            ("E001".to_string(), 10),
            ("W001".to_string(), 5),
        ]);

        let json = serde_json::to_string(&stats).unwrap();

        assert!(json.contains("\"by_code\""));
        assert!(json.contains("\"E001\":10") || json.contains("\"E001\": 10"));
        assert!(json.contains("\"W001\":5") || json.contains("\"W001\": 5"));
    }

    #[test]
    fn empty_stats_serializes_cleanly() {
        let stats = Stats::new();
        let json = serde_json::to_string(&stats).unwrap();

        assert!(json.contains("\"total_diagnostics\":0"));
        assert!(json.contains("\"matched_diagnostics\":0"));
        assert!(json.contains("\"filtered_diagnostics\":0"));
        assert!(json.contains("\"files_affected\":0"));
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn large_numbers_of_findings() {
        let findings: Vec<Finding> = (0..1000)
            .map(|_i| create_finding(Severity::Error, "E001", "src/lib.rs"))
            .collect();

        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.total_diagnostics, 1000);
        assert_eq!(stats.by_code.get("E001"), Some(&1000));
        assert_eq!(stats.files_affected, 1);
    }

    #[test]
    fn many_unique_files() {
        let findings: Vec<Finding> = (0..100)
            .map(|i| create_finding(Severity::Error, "E001", &format!("src/file_{i}.rs")))
            .collect();

        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.files_affected, 100);
    }

    #[test]
    fn many_unique_codes() {
        let findings: Vec<Finding> = (0..50)
            .map(|i| create_finding(Severity::Error, &format!("E{i:03}"), "src/lib.rs"))
            .collect();

        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.by_code.len(), 50);
    }

    #[test]
    fn normalized_paths_are_deduplicated() {
        // NormPath normalizes paths, so these should be treated as the same file
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Error, "E002", "src/lib.rs"),
        ];

        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.files_affected, 1);
    }

    #[test]
    fn merge_is_idempotent_for_empty() {
        let mut stats = Stats::new();
        let empty = Stats::new();

        stats.merge(&empty);
        stats.merge(&empty);
        stats.merge(&empty);

        assert_eq!(stats.total_diagnostics, 0);
    }
}
