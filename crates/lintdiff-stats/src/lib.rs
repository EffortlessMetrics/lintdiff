//! Statistics collection and aggregation for lintdiff.
//!
//! This microcrate provides a single responsibility: collecting and aggregating
//! statistics about diagnostics and findings from a lintdiff run.
//!
//! # Example: Creating Stats from Findings
//!
//! ```
//! use lintdiff_stats::Stats;
//! use lintdiff_types::{Finding, Severity, Location, NormPath};
//!
//! let findings = vec![
//!     Finding {
//!         severity: Severity::Error,
//!         code: "clippy::unwrap_used".to_string(),
//!         message: "used unwrap".to_string(),
//!         location: Some(Location {
//!             path: NormPath::new("src/lib.rs"),
//!             line: Some(10),
//!             col: None,
//!         }),
//!         check_id: None,
//!         help: None,
//!         url: None,
//!         fingerprint: None,
//!         data: None,
//!     },
//!     Finding {
//!         severity: Severity::Warn,
//!         code: "clippy::map_identity".to_string(),
//!         message: "identity map".to_string(),
//!         location: Some(Location {
//!             path: NormPath::new("src/main.rs"),
//!             line: Some(20),
//!             col: None,
//!         }),
//!         check_id: None,
//!         help: None,
//!         url: None,
//!         fingerprint: None,
//!         data: None,
//!     },
//! ];
//!
//! let stats = Stats::from_findings(&findings);
//! assert_eq!(stats.total_diagnostics, 2);
//! assert_eq!(stats.matched_diagnostics, 2);
//! assert_eq!(stats.files_affected, 2);
//! assert_eq!(stats.by_severity.get("error"), Some(&1));
//! assert_eq!(stats.by_severity.get("warning"), Some(&1));
//! ```
//!
//! # Example: Merging Stats
//!
//! ```
//! use lintdiff_stats::Stats;
//!
//! let mut stats1 = Stats::new();
//! stats1.total_diagnostics = 10;
//! stats1.matched_diagnostics = 5;
//!
//! let mut stats2 = Stats::new();
//! stats2.total_diagnostics = 20;
//! stats2.matched_diagnostics = 10;
//!
//! stats1.merge(&stats2);
//! assert_eq!(stats1.total_diagnostics, 30);
//! assert_eq!(stats1.matched_diagnostics, 15);
//! ```

#![warn(missing_docs)]

use std::collections::{HashMap, HashSet};

use lintdiff_types::{Finding, Severity};
use serde::Serialize;

/// Statistics about the lintdiff run.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Stats {
    /// Total diagnostics processed.
    pub total_diagnostics: usize,
    /// Diagnostics that matched changed lines.
    pub matched_diagnostics: usize,
    /// Diagnostics filtered out (outside diff).
    pub filtered_diagnostics: usize,
    /// Findings by severity (error, warning, note, etc.).
    pub by_severity: HashMap<String, usize>,
    /// Findings by code (e.g., "clippy::unwrap_used").
    pub by_code: HashMap<String, usize>,
    /// Files with findings.
    pub files_affected: usize,
}

impl Stats {
    /// Create empty stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Aggregate stats from a slice of findings.
    ///
    /// This computes all statistics from the provided findings:
    /// - Total and matched diagnostics are set to the length of the slice
    /// - Severity counts are aggregated from each finding's severity
    /// - Code counts are aggregated from each finding's code
    /// - Files affected is the count of unique file paths
    ///
    /// # Arguments
    ///
    /// * `findings` - Slice of findings to aggregate stats from
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_stats::Stats;
    /// use lintdiff_types::{Finding, Severity, Location, NormPath};
    ///
    /// let findings = vec![
    ///     Finding {
    ///         severity: Severity::Error,
    ///         code: "E001".to_string(),
    ///         message: "error".to_string(),
    ///         location: Some(Location {
    ///             path: NormPath::new("src/lib.rs"),
    ///             line: Some(1),
    ///             col: None,
    ///         }),
    ///         check_id: None,
    ///         help: None,
    ///         url: None,
    ///         fingerprint: None,
    ///         data: None,
    ///     },
    /// ];
    ///
    /// let stats = Stats::from_findings(&findings);
    /// assert_eq!(stats.total_diagnostics, 1);
    /// assert_eq!(stats.by_severity.get("error"), Some(&1));
    /// ```
    #[must_use]
    pub fn from_findings(findings: &[Finding]) -> Self {
        let mut stats = Self::new();
        stats.total_diagnostics = findings.len();
        stats.matched_diagnostics = findings.len();

        let mut files = HashSet::new();

        for finding in findings {
            // Count by severity
            let severity_key = match finding.severity {
                Severity::Error => "error",
                Severity::Warn => "warning",
                Severity::Info => "info",
            };
            *stats
                .by_severity
                .entry(severity_key.to_string())
                .or_insert(0) += 1;

            // Count by code
            *stats.by_code.entry(finding.code.clone()).or_insert(0) += 1;

            // Track unique files
            if let Some(ref location) = finding.location {
                files.insert(location.path.as_str().to_string());
            }
        }

        stats.files_affected = files.len();
        stats
    }

    /// Merge another stats into this one.
    ///
    /// All counters are summed, and the severity/code maps are merged
    /// by adding counts for matching keys.
    ///
    /// # Arguments
    ///
    /// * `other` - The stats to merge into this one
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_stats::Stats;
    /// use std::collections::HashMap;
    ///
    /// let mut stats1 = Stats {
    ///     total_diagnostics: 10,
    ///     matched_diagnostics: 5,
    ///     filtered_diagnostics: 5,
    ///     by_severity: HashMap::from([("error".to_string(), 3)]),
    ///     by_code: HashMap::from([("E001".to_string(), 2)]),
    ///     files_affected: 2,
    /// };
    ///
    /// let stats2 = Stats {
    ///     total_diagnostics: 20,
    ///     matched_diagnostics: 15,
    ///     filtered_diagnostics: 5,
    ///     by_severity: HashMap::from([("error".to_string(), 5), ("warning".to_string(), 10)]),
    ///     by_code: HashMap::from([("E001".to_string(), 3), ("W001".to_string(), 12)]),
    ///     files_affected: 3,
    /// };
    ///
    /// stats1.merge(&stats2);
    /// assert_eq!(stats1.total_diagnostics, 30);
    /// assert_eq!(stats1.matched_diagnostics, 20);
    /// assert_eq!(stats1.by_severity.get("error"), Some(&8));
    /// assert_eq!(stats1.by_severity.get("warning"), Some(&10));
    /// ```
    pub fn merge(&mut self, other: &Stats) {
        self.total_diagnostics += other.total_diagnostics;
        self.matched_diagnostics += other.matched_diagnostics;
        self.filtered_diagnostics += other.filtered_diagnostics;

        for (severity, count) in &other.by_severity {
            *self.by_severity.entry(severity.clone()).or_insert(0) += count;
        }

        for (code, count) in &other.by_code {
            *self.by_code.entry(code.clone()).or_insert(0) += count;
        }

        // Note: files_affected cannot be accurately merged without tracking actual file names
        // We take the maximum as a conservative estimate
        self.files_affected = self.files_affected.max(other.files_affected);
    }

    /// Get the pass rate (matched / total).
    ///
    /// Returns the ratio of matched diagnostics to total diagnostics.
    /// A higher pass rate means more diagnostics matched the diff.
    /// Returns 1.0 if there are no diagnostics (avoiding division by zero).
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_stats::Stats;
    ///
    /// let mut stats = Stats::new();
    /// stats.total_diagnostics = 10;
    /// stats.matched_diagnostics = 8;
    ///
    /// let rate = stats.pass_rate();
    /// assert!((rate - 0.8).abs() < 0.001);
    ///
    /// // Empty stats returns 1.0
    /// let empty = Stats::new();
    /// assert_eq!(empty.pass_rate(), 1.0);
    /// ```
    #[must_use]
    pub fn pass_rate(&self) -> f64 {
        if self.total_diagnostics == 0 {
            return 1.0;
        }
        f64::from(self.matched_diagnostics as u32) / f64::from(self.total_diagnostics as u32)
    }
}

/// Trait for types that can provide stats.
///
/// This trait allows different types to be converted into statistics,
/// enabling consistent stats collection across the codebase.
pub trait StatsSource {
    /// Get statistics from this source.
    fn stats(&self) -> Stats;
}

impl StatsSource for Stats {
    fn stats(&self) -> Stats {
        self.clone()
    }
}

impl StatsSource for &[Finding] {
    fn stats(&self) -> Stats {
        Stats::from_findings(self)
    }
}

impl StatsSource for Vec<Finding> {
    fn stats(&self) -> Stats {
        Stats::from_findings(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintdiff_types::{Finding, Location, NormPath};

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

    #[test]
    fn test_new_creates_empty_stats() {
        let stats = Stats::new();
        assert_eq!(stats.total_diagnostics, 0);
        assert_eq!(stats.matched_diagnostics, 0);
        assert_eq!(stats.filtered_diagnostics, 0);
        assert!(stats.by_severity.is_empty());
        assert!(stats.by_code.is_empty());
        assert_eq!(stats.files_affected, 0);
    }

    #[test]
    fn test_default_creates_empty_stats() {
        let stats = Stats::default();
        assert_eq!(stats.total_diagnostics, 0);
        assert_eq!(stats.matched_diagnostics, 0);
    }

    #[test]
    fn test_from_empty_findings() {
        let findings: Vec<Finding> = vec![];
        let stats = Stats::from_findings(&findings);
        assert_eq!(stats.total_diagnostics, 0);
        assert_eq!(stats.matched_diagnostics, 0);
        assert_eq!(stats.files_affected, 0);
    }

    #[test]
    fn test_from_single_finding() {
        let findings = vec![create_finding(Severity::Error, "E001", "src/lib.rs")];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.total_diagnostics, 1);
        assert_eq!(stats.matched_diagnostics, 1);
        assert_eq!(stats.files_affected, 1);
        assert_eq!(stats.by_severity.get("error"), Some(&1));
        assert_eq!(stats.by_code.get("E001"), Some(&1));
    }

    #[test]
    fn test_from_multiple_findings_same_file() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Warn, "W001", "src/lib.rs"),
        ];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.total_diagnostics, 2);
        assert_eq!(stats.files_affected, 1);
        assert_eq!(stats.by_severity.get("error"), Some(&1));
        assert_eq!(stats.by_severity.get("warning"), Some(&1));
    }

    #[test]
    fn test_from_multiple_findings_different_files() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Warn, "W001", "src/main.rs"),
        ];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.total_diagnostics, 2);
        assert_eq!(stats.files_affected, 2);
    }

    #[test]
    fn test_from_findings_with_info_severity() {
        let findings = vec![create_finding(Severity::Info, "I001", "src/lib.rs")];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.by_severity.get("info"), Some(&1));
    }

    #[test]
    fn test_from_findings_same_code_multiple_times() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Error, "E001", "src/main.rs"),
            create_finding(Severity::Error, "E002", "src/lib.rs"),
        ];
        let stats = Stats::from_findings(&findings);

        assert_eq!(stats.by_code.get("E001"), Some(&2));
        assert_eq!(stats.by_code.get("E002"), Some(&1));
    }

    #[test]
    fn test_merge_empty_stats() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 10;
        stats.matched_diagnostics = 5;

        let empty = Stats::new();
        stats.merge(&empty);

        assert_eq!(stats.total_diagnostics, 10);
        assert_eq!(stats.matched_diagnostics, 5);
    }

    #[test]
    fn test_merge_into_empty_stats() {
        let mut stats = Stats::new();
        let mut other = Stats::new();
        other.total_diagnostics = 10;
        other.matched_diagnostics = 5;

        stats.merge(&other);

        assert_eq!(stats.total_diagnostics, 10);
        assert_eq!(stats.matched_diagnostics, 5);
    }

    #[test]
    fn test_merge_combines_counts() {
        let mut stats1 = Stats {
            total_diagnostics: 10,
            matched_diagnostics: 5,
            filtered_diagnostics: 5,
            by_severity: HashMap::from([("error".to_string(), 3)]),
            by_code: HashMap::from([("E001".to_string(), 2)]),
            files_affected: 2,
        };

        let stats2 = Stats {
            total_diagnostics: 20,
            matched_diagnostics: 15,
            filtered_diagnostics: 5,
            by_severity: HashMap::from([("error".to_string(), 5), ("warning".to_string(), 10)]),
            by_code: HashMap::from([("E001".to_string(), 3), ("W001".to_string(), 12)]),
            files_affected: 3,
        };

        stats1.merge(&stats2);

        assert_eq!(stats1.total_diagnostics, 30);
        assert_eq!(stats1.matched_diagnostics, 20);
        assert_eq!(stats1.filtered_diagnostics, 10);
        assert_eq!(stats1.by_severity.get("error"), Some(&8));
        assert_eq!(stats1.by_severity.get("warning"), Some(&10));
        assert_eq!(stats1.by_code.get("E001"), Some(&5));
        assert_eq!(stats1.by_code.get("W001"), Some(&12));
    }

    #[test]
    fn test_pass_rate_zero_diagnostics() {
        let stats = Stats::new();
        assert_eq!(stats.pass_rate(), 1.0);
    }

    #[test]
    fn test_pass_rate_full_match() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 10;
        stats.matched_diagnostics = 10;
        assert_eq!(stats.pass_rate(), 1.0);
    }

    #[test]
    fn test_pass_rate_half_match() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 10;
        stats.matched_diagnostics = 5;
        let rate = stats.pass_rate();
        assert!((rate - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_pass_rate_no_match() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 10;
        stats.matched_diagnostics = 0;
        assert_eq!(stats.pass_rate(), 0.0);
    }

    #[test]
    fn test_stats_source_for_stats() {
        let mut stats = Stats::new();
        stats.total_diagnostics = 10;

        let result = stats.stats();
        assert_eq!(result.total_diagnostics, 10);
    }

    #[test]
    fn test_stats_source_for_slice() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Warn, "W001", "src/main.rs"),
        ];

        let stats: Stats = findings.as_slice().stats();
        assert_eq!(stats.total_diagnostics, 2);
    }

    #[test]
    fn test_stats_source_for_vec() {
        let findings = vec![
            create_finding(Severity::Error, "E001", "src/lib.rs"),
            create_finding(Severity::Warn, "W001", "src/main.rs"),
        ];

        let stats = findings.stats();
        assert_eq!(stats.total_diagnostics, 2);
    }

    #[test]
    fn test_finding_without_location() {
        let finding = Finding {
            severity: Severity::Error,
            code: "E001".to_string(),
            message: "test".to_string(),
            location: None,
            check_id: None,
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        };

        let stats = Stats::from_findings(&[finding]);
        assert_eq!(stats.total_diagnostics, 1);
        assert_eq!(stats.files_affected, 0); // No location means no file
    }
}
