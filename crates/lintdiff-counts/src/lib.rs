//! Diagnostic/finding count tracking for lintdiff.
//!
//! Provides types for counting and aggregating diagnostic findings
//! by severity, file, and category.

use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, AddAssign};

/// Counts of diagnostics by severity level.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeverityCounts {
    /// Number of hint-level diagnostics.
    pub hints: u64,
    /// Number of note-level diagnostics.
    pub notes: u64,
    /// Number of warnings.
    pub warnings: u64,
    /// Number of errors.
    pub errors: u64,
    /// Number of fatal errors.
    pub fatals: u64,
}

impl SeverityCounts {
    /// Create a new zeroed count.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hints: 0,
            notes: 0,
            warnings: 0,
            errors: 0,
            fatals: 0,
        }
    }

    /// Create counts from individual values.
    #[must_use]
    pub const fn from_values(hints: u64, notes: u64, warnings: u64, errors: u64, fatals: u64) -> Self {
        Self {
            hints,
            notes,
            warnings,
            errors,
            fatals,
        }
    }

    /// Get the total count across all severities.
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.hints + self.notes + self.warnings + self.errors + self.fatals
    }

    /// Check if all counts are zero.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.total() == 0
    }

    /// Get the count of problems (warnings + errors + fatals).
    #[must_use]
    pub const fn problems(&self) -> u64 {
        self.warnings + self.errors + self.fatals
    }

    /// Get the count of blocking issues (errors + fatals).
    #[must_use]
    pub const fn blocking(&self) -> u64 {
        self.errors + self.fatals
    }

    /// Check if there are any blocking issues.
    #[must_use]
    pub const fn has_blocking(&self) -> bool {
        self.blocking() > 0
    }

    /// Check if there are any problems.
    #[must_use]
    pub const fn has_problems(&self) -> bool {
        self.problems() > 0
    }

    /// Increment the count for a severity level.
    pub const fn increment(&mut self, severity: SeverityLevel) {
        match severity {
            SeverityLevel::Hint => self.hints += 1,
            SeverityLevel::Note => self.notes += 1,
            SeverityLevel::Warning => self.warnings += 1,
            SeverityLevel::Error => self.errors += 1,
            SeverityLevel::Fatal => self.fatals += 1,
        }
    }

    /// Get count for a specific severity.
    #[must_use]
    pub const fn get(&self, severity: SeverityLevel) -> u64 {
        match severity {
            SeverityLevel::Hint => self.hints,
            SeverityLevel::Note => self.notes,
            SeverityLevel::Warning => self.warnings,
            SeverityLevel::Error => self.errors,
            SeverityLevel::Fatal => self.fatals,
        }
    }

    /// Calculate the pass rate (non-problems / total).
    /// Returns None if total is 0.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn pass_rate(&self) -> Option<f64> {
        let total = self.total();
        if total == 0 {
            return None;
        }
        let non_problems = self.hints + self.notes;
        Some(non_problems as f64 / total as f64)
    }
}

impl Add for SeverityCounts {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            hints: self.hints + other.hints,
            notes: self.notes + other.notes,
            warnings: self.warnings + other.warnings,
            errors: self.errors + other.errors,
            fatals: self.fatals + other.fatals,
        }
    }
}

impl AddAssign for SeverityCounts {
    fn add_assign(&mut self, other: Self) {
        self.hints += other.hints;
        self.notes += other.notes;
        self.warnings += other.warnings;
        self.errors += other.errors;
        self.fatals += other.fatals;
    }
}

/// Severity levels for counting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeverityLevel {
    /// Hint-level diagnostic.
    Hint,
    /// Note-level diagnostic.
    Note,
    /// Warning-level diagnostic.
    Warning,
    /// Error-level diagnostic.
    Error,
    /// Fatal-level diagnostic.
    Fatal,
}

/// Counts by file path.
#[derive(Debug, Clone, Default)]
pub struct FileCounts {
    /// Per-file counts.
    by_file: HashMap<String, SeverityCounts>,
    /// Total counts across all files.
    total: SeverityCounts,
}

impl FileCounts {
    /// Create a new empty file counts.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add counts for a file.
    pub fn add_file(&mut self, path: impl Into<String>, counts: SeverityCounts) {
        self.total += counts.clone();
        self.by_file
            .entry(path.into())
            .or_default()
            .add_assign(counts);
    }

    /// Increment a severity for a file.
    pub fn increment(&mut self, path: impl Into<String>, severity: SeverityLevel) {
        self.total.increment(severity);
        self.by_file.entry(path.into()).or_default().increment(severity);
    }

    /// Get counts for a specific file.
    #[must_use]
    pub fn get(&self, path: &str) -> Option<&SeverityCounts> {
        self.by_file.get(path)
    }

    /// Get the total counts.
    #[must_use]
    pub const fn total(&self) -> &SeverityCounts {
        &self.total
    }

    /// Get the number of files with findings.
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.by_file.len()
    }

    /// Get all files with their counts.
    pub fn files(&self) -> impl Iterator<Item = (&String, &SeverityCounts)> {
        self.by_file.iter()
    }

    /// Check if there are any findings.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.total.is_empty()
    }
}

/// Counts by category/code.
#[derive(Debug, Clone, Default)]
pub struct CategoryCounts {
    /// Per-category counts.
    by_category: HashMap<String, SeverityCounts>,
    /// Total counts across all categories.
    total: SeverityCounts,
}

impl CategoryCounts {
    /// Create a new empty category counts.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add counts for a category.
    pub fn add_category(&mut self, category: impl Into<String>, counts: SeverityCounts) {
        self.total += counts.clone();
        self.by_category
            .entry(category.into())
            .or_default()
            .add_assign(counts);
    }

    /// Increment a severity for a category.
    pub fn increment(&mut self, category: impl Into<String>, severity: SeverityLevel) {
        self.total.increment(severity);
        self.by_category
            .entry(category.into())
            .or_default()
            .increment(severity);
    }

    /// Get counts for a specific category.
    #[must_use]
    pub fn get(&self, category: &str) -> Option<&SeverityCounts> {
        self.by_category.get(category)
    }

    /// Get the total counts.
    #[must_use]
    pub const fn total(&self) -> &SeverityCounts {
        &self.total
    }

    /// Get the number of unique categories.
    #[must_use]
    pub fn category_count(&self) -> usize {
        self.by_category.len()
    }

    /// Get all categories with their counts.
    pub fn categories(&self) -> impl Iterator<Item = (&String, &SeverityCounts)> {
        self.by_category.iter()
    }

    /// Check if there are any findings.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.total.is_empty()
    }
}

/// Combined counts for a complete summary.
#[derive(Debug, Clone, Default)]
pub struct CountSummary {
    /// Counts by severity.
    pub severity: SeverityCounts,
    /// Counts by file.
    pub by_file: FileCounts,
    /// Counts by category.
    pub by_category: CategoryCounts,
}

impl CountSummary {
    /// Create a new empty summary.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a finding.
    pub fn record(
        &mut self,
        file: impl Into<String>,
        category: impl Into<String>,
        severity: SeverityLevel,
    ) {
        self.severity.increment(severity);
        self.by_file.increment(file, severity);
        self.by_category.increment(category, severity);
    }

    /// Get the total count.
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.severity.total()
    }

    /// Check if there are any findings.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.severity.is_empty()
    }

    /// Check if there are blocking issues.
    #[must_use]
    pub const fn has_blocking(&self) -> bool {
        self.severity.has_blocking()
    }

    /// Merge another summary into this one.
    pub fn merge(&mut self, other: Self) {
        self.severity += other.severity;
        // Merge file counts
        for (path, counts) in other.by_file.by_file {
            self.by_file.add_file(path, counts);
        }
        // Merge category counts
        for (category, counts) in other.by_category.by_category {
            self.by_category.add_category(category, counts);
        }
    }
}

impl fmt::Display for SeverityCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} hints, {} notes, {} warnings, {} errors, {} fatals",
            self.hints, self.notes, self.warnings, self.errors, self.fatals
        )
    }
}

impl fmt::Display for CountSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} files, {} categories, {}",
            self.by_file.file_count(),
            self.by_category.category_count(),
            self.severity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_counts_new() {
        let counts = SeverityCounts::new();
        assert_eq!(counts.hints, 0);
        assert_eq!(counts.notes, 0);
        assert_eq!(counts.warnings, 0);
        assert_eq!(counts.errors, 0);
        assert_eq!(counts.fatals, 0);
    }

    #[test]
    fn test_severity_counts_default() {
        let counts = SeverityCounts::default();
        assert_eq!(counts.hints, 0);
        assert_eq!(counts.total(), 0);
    }

    #[test]
    fn test_severity_counts_from_values() {
        let counts = SeverityCounts::from_values(1, 2, 3, 4, 5);
        assert_eq!(counts.hints, 1);
        assert_eq!(counts.notes, 2);
        assert_eq!(counts.warnings, 3);
        assert_eq!(counts.errors, 4);
        assert_eq!(counts.fatals, 5);
    }

    #[test]
    fn test_severity_counts_total() {
        let counts = SeverityCounts::from_values(1, 2, 3, 4, 5);
        assert_eq!(counts.total(), 15);
    }

    #[test]
    fn test_severity_counts_is_empty() {
        assert!(SeverityCounts::new().is_empty());
        assert!(!SeverityCounts::from_values(1, 0, 0, 0, 0).is_empty());
    }

    #[test]
    fn test_severity_counts_problems() {
        let counts = SeverityCounts::from_values(1, 2, 3, 4, 5);
        assert_eq!(counts.problems(), 12); // 3 + 4 + 5
    }

    #[test]
    fn test_severity_counts_blocking() {
        let counts = SeverityCounts::from_values(1, 2, 3, 4, 5);
        assert_eq!(counts.blocking(), 9); // 4 + 5
    }

    #[test]
    fn test_severity_counts_has_blocking() {
        assert!(!SeverityCounts::from_values(0, 0, 1, 0, 0).has_blocking());
        assert!(SeverityCounts::from_values(0, 0, 0, 1, 0).has_blocking());
        assert!(SeverityCounts::from_values(0, 0, 0, 0, 1).has_blocking());
    }

    #[test]
    fn test_severity_counts_has_problems() {
        assert!(!SeverityCounts::from_values(1, 1, 0, 0, 0).has_problems());
        assert!(SeverityCounts::from_values(0, 0, 1, 0, 0).has_problems());
    }

    #[test]
    fn test_severity_counts_increment() {
        let mut counts = SeverityCounts::new();
        counts.increment(SeverityLevel::Hint);
        assert_eq!(counts.hints, 1);
        counts.increment(SeverityLevel::Warning);
        assert_eq!(counts.warnings, 1);
    }

    #[test]
    fn test_severity_counts_get() {
        let counts = SeverityCounts::from_values(1, 2, 3, 4, 5);
        assert_eq!(counts.get(SeverityLevel::Hint), 1);
        assert_eq!(counts.get(SeverityLevel::Note), 2);
        assert_eq!(counts.get(SeverityLevel::Warning), 3);
        assert_eq!(counts.get(SeverityLevel::Error), 4);
        assert_eq!(counts.get(SeverityLevel::Fatal), 5);
    }

    #[test]
    fn test_severity_counts_add() {
        let a = SeverityCounts::from_values(1, 2, 3, 4, 5);
        let b = SeverityCounts::from_values(5, 4, 3, 2, 1);
        let c = a + b;
        assert_eq!(c.hints, 6);
        assert_eq!(c.notes, 6);
        assert_eq!(c.warnings, 6);
        assert_eq!(c.errors, 6);
        assert_eq!(c.fatals, 6);
    }

    #[test]
    fn test_severity_counts_add_assign() {
        let mut a = SeverityCounts::from_values(1, 2, 3, 4, 5);
        a += SeverityCounts::from_values(1, 1, 1, 1, 1);
        assert_eq!(a.hints, 2);
        assert_eq!(a.notes, 3);
        assert_eq!(a.warnings, 4);
        assert_eq!(a.errors, 5);
        assert_eq!(a.fatals, 6);
    }

    #[test]
    fn test_severity_counts_pass_rate_empty() {
        let counts = SeverityCounts::new();
        assert!(counts.pass_rate().is_none());
    }

    #[test]
    fn test_severity_counts_pass_rate_full() {
        let counts = SeverityCounts::from_values(5, 5, 0, 0, 0);
        assert_eq!(counts.pass_rate(), Some(1.0));
    }

    #[test]
    fn test_severity_counts_pass_rate_half() {
        let counts = SeverityCounts::from_values(5, 5, 5, 5, 0);
        let rate = counts.pass_rate().expect("pass rate should exist");
        assert!((rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_severity_counts_display() {
        let counts = SeverityCounts::from_values(1, 2, 3, 4, 5);
        let s = format!("{counts}");
        assert_eq!(s, "1 hints, 2 notes, 3 warnings, 4 errors, 5 fatals");
    }

    #[test]
    fn test_file_counts_new() {
        let fc = FileCounts::new();
        assert!(fc.is_empty());
        assert_eq!(fc.file_count(), 0);
    }

    #[test]
    fn test_file_counts_add_file() {
        let mut fc = FileCounts::new();
        fc.add_file("src/main.rs", SeverityCounts::from_values(1, 2, 3, 0, 0));
        assert_eq!(fc.file_count(), 1);
        assert_eq!(fc.total().total(), 6);
    }

    #[test]
    fn test_file_counts_increment() {
        let mut fc = FileCounts::new();
        fc.increment("src/lib.rs", SeverityLevel::Error);
        assert_eq!(fc.file_count(), 1);
        assert_eq!(fc.total().errors, 1);
    }

    #[test]
    fn test_file_counts_get() {
        let mut fc = FileCounts::new();
        fc.add_file("src/a.rs", SeverityCounts::from_values(1, 0, 0, 0, 0));
        let counts = fc.get("src/a.rs").expect("should exist");
        assert_eq!(counts.hints, 1);
        assert!(fc.get("src/b.rs").is_none());
    }

    #[test]
    fn test_file_counts_multiple_files() {
        let mut fc = FileCounts::new();
        fc.add_file("src/a.rs", SeverityCounts::from_values(1, 0, 0, 0, 0));
        fc.add_file("src/b.rs", SeverityCounts::from_values(0, 2, 0, 0, 0));
        assert_eq!(fc.file_count(), 2);
        assert_eq!(fc.total().total(), 3);
    }

    #[test]
    fn test_file_counts_same_file_accumulates() {
        let mut fc = FileCounts::new();
        fc.add_file("src/a.rs", SeverityCounts::from_values(1, 0, 0, 0, 0));
        fc.add_file("src/a.rs", SeverityCounts::from_values(0, 2, 0, 0, 0));
        assert_eq!(fc.file_count(), 1);
        let counts = fc.get("src/a.rs").expect("should exist");
        assert_eq!(counts.hints, 1);
        assert_eq!(counts.notes, 2);
    }

    #[test]
    fn test_category_counts_new() {
        let cc = CategoryCounts::new();
        assert!(cc.is_empty());
        assert_eq!(cc.category_count(), 0);
    }

    #[test]
    fn test_category_counts_add_category() {
        let mut cc = CategoryCounts::new();
        cc.add_category("clippy::style", SeverityCounts::from_values(0, 0, 5, 0, 0));
        assert_eq!(cc.category_count(), 1);
        assert_eq!(cc.total().warnings, 5);
    }

    #[test]
    fn test_category_counts_increment() {
        let mut cc = CategoryCounts::new();
        cc.increment("rustc::unused", SeverityLevel::Warning);
        assert_eq!(cc.category_count(), 1);
        assert_eq!(cc.total().warnings, 1);
    }

    #[test]
    fn test_category_counts_get() {
        let mut cc = CategoryCounts::new();
        cc.add_category("cat1", SeverityCounts::from_values(3, 0, 0, 0, 0));
        let counts = cc.get("cat1").expect("should exist");
        assert_eq!(counts.hints, 3);
        assert!(cc.get("cat2").is_none());
    }

    #[test]
    fn test_category_counts_multiple() {
        let mut cc = CategoryCounts::new();
        cc.add_category("cat1", SeverityCounts::from_values(1, 0, 0, 0, 0));
        cc.add_category("cat2", SeverityCounts::from_values(0, 2, 0, 0, 0));
        assert_eq!(cc.category_count(), 2);
        assert_eq!(cc.total().total(), 3);
    }

    #[test]
    fn test_count_summary_new() {
        let summary = CountSummary::new();
        assert!(summary.is_empty());
        assert_eq!(summary.total(), 0);
    }

    #[test]
    fn test_count_summary_record() {
        let mut summary = CountSummary::new();
        summary.record("src/main.rs", "clippy::style", SeverityLevel::Warning);
        assert!(!summary.is_empty());
        assert_eq!(summary.total(), 1);
        assert_eq!(summary.severity.warnings, 1);
    }

    #[test]
    fn test_count_summary_multiple_records() {
        let mut summary = CountSummary::new();
        summary.record("src/a.rs", "cat1", SeverityLevel::Hint);
        summary.record("src/b.rs", "cat2", SeverityLevel::Error);
        summary.record("src/a.rs", "cat1", SeverityLevel::Warning);
        assert_eq!(summary.total(), 3);
        assert_eq!(summary.by_file.file_count(), 2);
        assert_eq!(summary.by_category.category_count(), 2);
    }

    #[test]
    fn test_count_summary_has_blocking() {
        let mut summary = CountSummary::new();
        assert!(!summary.has_blocking());
        summary.record("a.rs", "cat", SeverityLevel::Warning);
        assert!(!summary.has_blocking());
        summary.record("b.rs", "cat", SeverityLevel::Error);
        assert!(summary.has_blocking());
    }

    #[test]
    fn test_count_summary_merge() {
        let mut a = CountSummary::new();
        a.record("a.rs", "cat1", SeverityLevel::Hint);

        let mut b = CountSummary::new();
        b.record("b.rs", "cat2", SeverityLevel::Error);

        a.merge(b);
        assert_eq!(a.total(), 2);
        assert_eq!(a.by_file.file_count(), 2);
        assert_eq!(a.by_category.category_count(), 2);
    }

    #[test]
    fn test_count_summary_display() {
        let mut summary = CountSummary::new();
        summary.record("a.rs", "cat1", SeverityLevel::Hint);
        summary.record("b.rs", "cat2", SeverityLevel::Error);
        let s = format!("{summary}");
        assert_eq!(s, "2 files, 2 categories, 1 hints, 0 notes, 0 warnings, 1 errors, 0 fatals");
    }

    #[test]
    fn test_severity_level_equality() {
        assert_eq!(SeverityLevel::Hint, SeverityLevel::Hint);
        assert_ne!(SeverityLevel::Hint, SeverityLevel::Warning);
    }

    #[test]
    fn test_severity_counts_clone() {
        let a = SeverityCounts::from_values(1, 2, 3, 4, 5);
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_severity_counts_partial_eq() {
        let a = SeverityCounts::from_values(1, 2, 3, 4, 5);
        let b = SeverityCounts::from_values(1, 2, 3, 4, 5);
        let c = SeverityCounts::from_values(0, 0, 0, 0, 0);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_file_counts_clone() {
        let mut fc = FileCounts::new();
        fc.add_file("a.rs", SeverityCounts::from_values(1, 0, 0, 0, 0));
        let fc2 = fc.clone();
        assert_eq!(fc.file_count(), fc2.file_count());
    }

    #[test]
    fn test_category_counts_clone() {
        let mut cc = CategoryCounts::new();
        cc.add_category("cat", SeverityCounts::from_values(1, 0, 0, 0, 0));
        let cc2 = cc.clone();
        assert_eq!(cc.category_count(), cc2.category_count());
    }

    #[test]
    fn test_count_summary_clone() {
        let mut summary = CountSummary::new();
        summary.record("a.rs", "cat", SeverityLevel::Hint);
        let summary2 = summary.clone();
        assert_eq!(summary.total(), summary2.total());
    }

    #[test]
    fn test_file_counts_files_iterator() {
        let mut fc = FileCounts::new();
        fc.add_file("a.rs", SeverityCounts::from_values(1, 0, 0, 0, 0));
        fc.add_file("b.rs", SeverityCounts::from_values(0, 2, 0, 0, 0));
        let files: Vec<_> = fc.files().collect();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_category_counts_categories_iterator() {
        let mut cc = CategoryCounts::new();
        cc.add_category("cat1", SeverityCounts::from_values(1, 0, 0, 0, 0));
        cc.add_category("cat2", SeverityCounts::from_values(0, 2, 0, 0, 0));
        let cats: Vec<_> = cc.categories().collect();
        assert_eq!(cats.len(), 2);
    }

    #[test]
    fn test_severity_counts_all_zeros() {
        let counts = SeverityCounts::from_values(0, 0, 0, 0, 0);
        assert!(counts.is_empty());
        assert_eq!(counts.total(), 0);
        assert_eq!(counts.problems(), 0);
        assert_eq!(counts.blocking(), 0);
        assert!(!counts.has_problems());
        assert!(!counts.has_blocking());
    }

    #[test]
    fn test_severity_counts_only_hints() {
        let counts = SeverityCounts::from_values(5, 0, 0, 0, 0);
        assert!(!counts.is_empty());
        assert_eq!(counts.total(), 5);
        assert_eq!(counts.problems(), 0);
        assert_eq!(counts.blocking(), 0);
        assert_eq!(counts.pass_rate(), Some(1.0));
    }

    #[test]
    fn test_severity_counts_only_errors() {
        let counts = SeverityCounts::from_values(0, 0, 0, 3, 0);
        assert!(!counts.is_empty());
        assert_eq!(counts.total(), 3);
        assert_eq!(counts.problems(), 3);
        assert_eq!(counts.blocking(), 3);
        assert_eq!(counts.pass_rate(), Some(0.0));
    }

    #[test]
    fn test_count_summary_merge_same_file() {
        let mut a = CountSummary::new();
        a.record("a.rs", "cat1", SeverityLevel::Hint);

        let mut b = CountSummary::new();
        b.record("a.rs", "cat2", SeverityLevel::Error);

        a.merge(b);
        assert_eq!(a.total(), 2);
        assert_eq!(a.by_file.file_count(), 1); // Same file merged
        let file_counts = a.by_file.get("a.rs").expect("should exist");
        assert_eq!(file_counts.hints, 1);
        assert_eq!(file_counts.errors, 1);
    }

    #[test]
    fn test_count_summary_merge_same_category() {
        let mut a = CountSummary::new();
        a.record("a.rs", "cat", SeverityLevel::Hint);

        let mut b = CountSummary::new();
        b.record("b.rs", "cat", SeverityLevel::Error);

        a.merge(b);
        assert_eq!(a.total(), 2);
        assert_eq!(a.by_category.category_count(), 1); // Same category merged
        let cat_counts = a.by_category.get("cat").expect("should exist");
        assert_eq!(cat_counts.hints, 1);
        assert_eq!(cat_counts.errors, 1);
    }

    #[test]
    fn test_file_counts_total_matches_sum() {
        let mut fc = FileCounts::new();
        fc.add_file("a.rs", SeverityCounts::from_values(1, 2, 3, 4, 5));
        fc.add_file("b.rs", SeverityCounts::from_values(2, 3, 4, 5, 6));
        
        let total = fc.total();
        assert_eq!(total.hints, 3);
        assert_eq!(total.notes, 5);
        assert_eq!(total.warnings, 7);
        assert_eq!(total.errors, 9);
        assert_eq!(total.fatals, 11);
    }

    #[test]
    fn test_category_counts_total_matches_sum() {
        let mut cc = CategoryCounts::new();
        cc.add_category("cat1", SeverityCounts::from_values(1, 2, 3, 4, 5));
        cc.add_category("cat2", SeverityCounts::from_values(2, 3, 4, 5, 6));
        
        let total = cc.total();
        assert_eq!(total.hints, 3);
        assert_eq!(total.notes, 5);
        assert_eq!(total.warnings, 7);
        assert_eq!(total.errors, 9);
        assert_eq!(total.fatals, 11);
    }

    #[test]
    fn test_severity_counts_increment_all_levels() {
        let mut counts = SeverityCounts::new();
        counts.increment(SeverityLevel::Hint);
        counts.increment(SeverityLevel::Note);
        counts.increment(SeverityLevel::Warning);
        counts.increment(SeverityLevel::Error);
        counts.increment(SeverityLevel::Fatal);
        
        assert_eq!(counts.hints, 1);
        assert_eq!(counts.notes, 1);
        assert_eq!(counts.warnings, 1);
        assert_eq!(counts.errors, 1);
        assert_eq!(counts.fatals, 1);
        assert_eq!(counts.total(), 5);
    }

    #[test]
    fn test_file_counts_increment_multiple_same_file() {
        let mut fc = FileCounts::new();
        fc.increment("a.rs", SeverityLevel::Hint);
        fc.increment("a.rs", SeverityLevel::Warning);
        fc.increment("a.rs", SeverityLevel::Error);
        
        assert_eq!(fc.file_count(), 1);
        let counts = fc.get("a.rs").expect("should exist");
        assert_eq!(counts.hints, 1);
        assert_eq!(counts.warnings, 1);
        assert_eq!(counts.errors, 1);
        assert_eq!(fc.total().total(), 3);
    }

    #[test]
    fn test_category_counts_increment_multiple_same_category() {
        let mut cc = CategoryCounts::new();
        cc.increment("cat", SeverityLevel::Hint);
        cc.increment("cat", SeverityLevel::Warning);
        cc.increment("cat", SeverityLevel::Error);
        
        assert_eq!(cc.category_count(), 1);
        let counts = cc.get("cat").expect("should exist");
        assert_eq!(counts.hints, 1);
        assert_eq!(counts.warnings, 1);
        assert_eq!(counts.errors, 1);
        assert_eq!(cc.total().total(), 3);
    }

    #[test]
    fn test_count_summary_record_same_file_different_categories() {
        let mut summary = CountSummary::new();
        summary.record("a.rs", "cat1", SeverityLevel::Hint);
        summary.record("a.rs", "cat2", SeverityLevel::Warning);
        
        assert_eq!(summary.total(), 2);
        assert_eq!(summary.by_file.file_count(), 1);
        assert_eq!(summary.by_category.category_count(), 2);
        
        let file_counts = summary.by_file.get("a.rs").expect("should exist");
        assert_eq!(file_counts.total(), 2);
    }

    #[test]
    fn test_severity_counts_pass_rate_various() {
        // 50% pass rate: 2 non-problems out of 4 total
        let counts = SeverityCounts::from_values(1, 1, 1, 1, 0);
        let rate = counts.pass_rate().expect("pass rate should exist");
        assert!((rate - 0.5).abs() < f64::EPSILON);
        
        // 25% pass rate: 1 non-problem out of 4 total
        let counts2 = SeverityCounts::from_values(1, 0, 1, 1, 1);
        let rate2 = counts2.pass_rate().expect("pass rate should exist");
        assert!((rate2 - 0.25).abs() < f64::EPSILON);
        
        // 75% pass rate: 3 non-problems out of 4 total
        let counts3 = SeverityCounts::from_values(2, 1, 1, 0, 0);
        let rate3 = counts3.pass_rate().expect("pass rate should exist");
        assert!((rate3 - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_severity_level_copy() {
        let level = SeverityLevel::Warning;
        let level2 = level; // Copy
        assert_eq!(level, level2);
    }

    #[test]
    fn test_severity_level_debug() {
        let level = SeverityLevel::Warning;
        let debug_str = format!("{level:?}");
        assert_eq!(debug_str, "Warning");
    }
}
