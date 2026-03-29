//! Diff statistics aggregation and reporting for lintdiff.
//!
//! Provides types for tracking and reporting diff statistics including
//! files changed, lines added/removed, and file operations (add, delete, rename).
//!
//! # Example
//!
//! ```
//! use lintdiff_diff_stats::{DiffStatsBuilder, FileDiffStats, ChangeType, format_stats};
//!
//! let mut builder = DiffStatsBuilder::new();
//! builder.add_file(&FileDiffStats::new("src/main.rs", 10, 5, ChangeType::Modified));
//! builder.add_file(&FileDiffStats::new("src/new.rs", 20, 0, ChangeType::Added));
//!
//! let stats = builder.build();
//! println!("{}", format_stats(&stats));
//! ```

use std::fmt;
use std::ops::{Add, AddAssign};

/// Type of file change in a diff.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChangeType {
    /// New file created.
    Added,
    /// File deleted.
    Deleted,
    /// File modified.
    #[default]
    Modified,
    /// File renamed.
    Renamed,
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Added => write!(f, "added"),
            Self::Deleted => write!(f, "deleted"),
            Self::Modified => write!(f, "modified"),
            Self::Renamed => write!(f, "renamed"),
        }
    }
}

impl ChangeType {
    /// Get a short symbol for this change type.
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Modified => "M",
            Self::Renamed => "R",
        }
    }

    /// Check if this change type represents a file addition.
    #[must_use]
    pub const fn is_addition(&self) -> bool {
        matches!(self, Self::Added)
    }

    /// Check if this change type represents a file deletion.
    #[must_use]
    pub const fn is_deletion(&self) -> bool {
        matches!(self, Self::Deleted)
    }

    /// Check if this change type represents a modification.
    #[must_use]
    pub const fn is_modification(&self) -> bool {
        matches!(self, Self::Modified)
    }

    /// Check if this change type represents a rename.
    #[must_use]
    pub const fn is_rename(&self) -> bool {
        matches!(self, Self::Renamed)
    }
}

/// Per-file diff statistics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileDiffStats {
    /// File path.
    pub path: String,
    /// Lines added in this file.
    pub lines_added: usize,
    /// Lines removed in this file.
    pub lines_removed: usize,
    /// Type of change.
    pub change_type: ChangeType,
}

impl FileDiffStats {
    /// Create new file diff stats.
    #[must_use]
    pub fn new(path: impl Into<String>, lines_added: usize, lines_removed: usize, change_type: ChangeType) -> Self {
        Self {
            path: path.into(),
            lines_added,
            lines_removed,
            change_type,
        }
    }

    /// Create stats for an added file.
    #[must_use]
    pub fn added(path: impl Into<String>, lines_added: usize) -> Self {
        Self::new(path, lines_added, 0, ChangeType::Added)
    }

    /// Create stats for a deleted file.
    #[must_use]
    pub fn deleted(path: impl Into<String>, lines_removed: usize) -> Self {
        Self::new(path, 0, lines_removed, ChangeType::Deleted)
    }

    /// Create stats for a modified file.
    #[must_use]
    pub fn modified(path: impl Into<String>, lines_added: usize, lines_removed: usize) -> Self {
        Self::new(path, lines_added, lines_removed, ChangeType::Modified)
    }

    /// Create stats for a renamed file.
    #[must_use]
    pub fn renamed(path: impl Into<String>) -> Self {
        Self::new(path, 0, 0, ChangeType::Renamed)
    }

    /// Get the net line change for this file.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub const fn net_lines(&self) -> isize {
        self.lines_added as isize - self.lines_removed as isize
    }

    /// Check if this file has any line changes.
    #[must_use]
    pub const fn has_changes(&self) -> bool {
        self.lines_added > 0 || self.lines_removed > 0
    }

    /// Get the total lines changed (added + removed).
    #[must_use]
    pub const fn total_lines_changed(&self) -> usize {
        self.lines_added + self.lines_removed
    }
}

impl Default for FileDiffStats {
    fn default() -> Self {
        Self {
            path: String::new(),
            lines_added: 0,
            lines_removed: 0,
            change_type: ChangeType::Modified,
        }
    }
}

/// Overall diff statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiffStats {
    /// Number of files changed.
    pub files_changed: usize,
    /// Total lines added.
    pub lines_added: usize,
    /// Total lines removed.
    pub lines_removed: usize,
    /// New files created.
    pub files_added: usize,
    /// Files deleted.
    pub files_deleted: usize,
    /// Files renamed.
    pub files_renamed: usize,
}

impl DiffStats {
    /// Create new zeroed diff stats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            files_changed: 0,
            lines_added: 0,
            lines_removed: 0,
            files_added: 0,
            files_deleted: 0,
            files_renamed: 0,
        }
    }

    /// Create diff stats from individual values.
    #[must_use]
    pub const fn from_values(
        files_changed: usize,
        lines_added: usize,
        lines_removed: usize,
        files_added: usize,
        files_deleted: usize,
        files_renamed: usize,
    ) -> Self {
        Self {
            files_changed,
            lines_added,
            lines_removed,
            files_added,
            files_deleted,
            files_renamed,
        }
    }

    /// Get the total number of files involved in the diff.
    #[must_use]
    pub const fn total_files(&self) -> usize {
        self.files_changed
    }

    /// Get the net line change.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub const fn net_lines(&self) -> isize {
        self.lines_added as isize - self.lines_removed as isize
    }

    /// Get the total lines changed (added + removed).
    #[must_use]
    pub const fn total_lines_changed(&self) -> usize {
        self.lines_added + self.lines_removed
    }

    /// Check if there are any changes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.files_changed == 0 && self.lines_added == 0 && self.lines_removed == 0
    }

    /// Check if there are any file additions.
    #[must_use]
    pub const fn has_additions(&self) -> bool {
        self.files_added > 0
    }

    /// Check if there are any file deletions.
    #[must_use]
    pub const fn has_deletions(&self) -> bool {
        self.files_deleted > 0
    }

    /// Check if there are any file renames.
    #[must_use]
    pub const fn has_renames(&self) -> bool {
        self.files_renamed > 0
    }

    /// Check if there are any modifications (non-add/delete/rename).
    #[must_use]
    pub const fn has_modifications(&self) -> bool {
        let special_files = self.files_added + self.files_deleted + self.files_renamed;
        self.files_changed > special_files
    }
}

impl Default for DiffStats {
    fn default() -> Self {
        Self::new()
    }
}

impl Add for DiffStats {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            files_changed: self.files_changed + other.files_changed,
            lines_added: self.lines_added + other.lines_added,
            lines_removed: self.lines_removed + other.lines_removed,
            files_added: self.files_added + other.files_added,
            files_deleted: self.files_deleted + other.files_deleted,
            files_renamed: self.files_renamed + other.files_renamed,
        }
    }
}

impl AddAssign for DiffStats {
    fn add_assign(&mut self, other: Self) {
        self.files_changed += other.files_changed;
        self.lines_added += other.lines_added;
        self.lines_removed += other.lines_removed;
        self.files_added += other.files_added;
        self.files_deleted += other.files_deleted;
        self.files_renamed += other.files_renamed;
    }
}

/// Builder for aggregating diff statistics.
#[derive(Debug, Clone, Default)]
pub struct DiffStatsBuilder {
    files_changed: usize,
    lines_added: usize,
    lines_removed: usize,
    files_added: usize,
    files_deleted: usize,
    files_renamed: usize,
}

impl DiffStatsBuilder {
    /// Create a new builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            files_changed: 0,
            lines_added: 0,
            lines_removed: 0,
            files_added: 0,
            files_deleted: 0,
            files_renamed: 0,
        }
    }

    /// Add a file's diff stats to the builder.
    pub const fn add_file(&mut self, stats: &FileDiffStats) -> &mut Self {
        self.files_changed += 1;
        self.lines_added += stats.lines_added;
        self.lines_removed += stats.lines_removed;

        match stats.change_type {
            ChangeType::Added => self.files_added += 1,
            ChangeType::Deleted => self.files_deleted += 1,
            ChangeType::Modified => {}
            ChangeType::Renamed => self.files_renamed += 1,
        }

        self
    }

    /// Add multiple file stats.
    pub fn add_files(&mut self, files: impl IntoIterator<Item = FileDiffStats>) -> &mut Self {
        for file in files {
            self.add_file(&file);
        }
        self
    }

    /// Build the final diff stats.
    #[must_use]
    pub const fn build(&self) -> DiffStats {
        DiffStats {
            files_changed: self.files_changed,
            lines_added: self.lines_added,
            lines_removed: self.lines_removed,
            files_added: self.files_added,
            files_deleted: self.files_deleted,
            files_renamed: self.files_renamed,
        }
    }

    /// Reset the builder to empty state.
    pub const fn reset(&mut self) -> &mut Self {
        *self = Self::new();
        self
    }

    /// Get the current number of files tracked.
    #[must_use]
    pub const fn file_count(&self) -> usize {
        self.files_changed
    }

    /// Check if the builder is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.files_changed == 0
    }
}

// =============================================================================
// Formatting Functions
// =============================================================================

/// Format diff stats in human-readable format.
#[must_use]
pub fn format_stats(stats: &DiffStats) -> String {
    if stats.is_empty() {
        return "No changes".to_string();
    }

    let mut parts = Vec::new();

    // Files summary
    parts.push(format!("{} file{} changed", stats.files_changed, if stats.files_changed == 1 { "" } else { "s" }));

    // Line changes
    if stats.lines_added > 0 || stats.lines_removed > 0 {
        parts.push(format!("{} insertion{}, {} deletion{}",
            stats.lines_added,
            if stats.lines_added == 1 { "" } else { "s" },
            stats.lines_removed,
            if stats.lines_removed == 1 { "" } else { "s" }
        ));
    }

    // File operations
    let mut ops = Vec::new();
    if stats.files_added > 0 {
        ops.push(format!("{} added", stats.files_added));
    }
    if stats.files_deleted > 0 {
        ops.push(format!("{} deleted", stats.files_deleted));
    }
    if stats.files_renamed > 0 {
        ops.push(format!("{} renamed", stats.files_renamed));
    }
    if !ops.is_empty() {
        parts.push(format!("({})", ops.join(", ")));
    }

    parts.join(", ")
}

/// Format diff stats in short format (e.g., `+10-5 files:3`).
#[must_use]
pub fn format_stats_short(stats: &DiffStats) -> String {
    if stats.is_empty() {
        return "0".to_string();
    }

    format!("+{}-{} files:{}", stats.lines_added, stats.lines_removed, stats.files_changed)
}

/// Format diff stats as a markdown table.
#[must_use]
pub fn format_stats_markdown(stats: &DiffStats) -> String {
    use std::fmt::Write;
    let mut table = String::new();
    let _ = writeln!(table, "| Metric | Count |");
    let _ = writeln!(table, "|--------|-------|");
    let _ = writeln!(table, "| Files changed | {} |", stats.files_changed);
    let _ = writeln!(table, "| Lines added | {} |", stats.lines_added);
    let _ = writeln!(table, "| Lines removed | {} |", stats.lines_removed);
    let _ = writeln!(table, "| Net lines | {} |", stats.net_lines());
    let _ = writeln!(table, "| Files added | {} |", stats.files_added);
    let _ = writeln!(table, "| Files deleted | {} |", stats.files_deleted);
    let _ = write!(table, "| Files renamed | {} |", stats.files_renamed);
    table
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if diff stats has no changes.
#[must_use]
pub const fn is_empty(stats: &DiffStats) -> bool {
    stats.is_empty()
}

/// Calculate net line change (added - removed).
#[must_use]
pub const fn net_lines(stats: &DiffStats) -> isize {
    stats.net_lines()
}

/// Merge two diff stats together.
#[must_use]
pub fn merge_stats(a: &DiffStats, b: &DiffStats) -> DiffStats {
    a.clone() + b.clone()
}

/// Create empty diff stats.
#[must_use]
pub const fn empty_stats() -> DiffStats {
    DiffStats::new()
}

/// Calculate the diff stats from a slice of file stats.
#[must_use]
pub fn from_file_stats(files: &[FileDiffStats]) -> DiffStats {
    let mut builder = DiffStatsBuilder::new();
    for file in files {
        builder.add_file(file);
    }
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_type_display_works() {
        assert_eq!(ChangeType::Added.to_string(), "added");
        assert_eq!(ChangeType::Deleted.to_string(), "deleted");
        assert_eq!(ChangeType::Modified.to_string(), "modified");
        assert_eq!(ChangeType::Renamed.to_string(), "renamed");
    }

    #[test]
    fn change_type_symbol_works() {
        assert_eq!(ChangeType::Added.symbol(), "A");
        assert_eq!(ChangeType::Deleted.symbol(), "D");
        assert_eq!(ChangeType::Modified.symbol(), "M");
        assert_eq!(ChangeType::Renamed.symbol(), "R");
    }

    #[test]
    fn file_diff_stats_constructors_work() {
        let added = FileDiffStats::added("test.rs", 10);
        assert_eq!(added.path, "test.rs");
        assert_eq!(added.lines_added, 10);
        assert_eq!(added.lines_removed, 0);
        assert_eq!(added.change_type, ChangeType::Added);

        let deleted = FileDiffStats::deleted("old.rs", 5);
        assert_eq!(deleted.lines_added, 0);
        assert_eq!(deleted.lines_removed, 5);
        assert_eq!(deleted.change_type, ChangeType::Deleted);

        let modified = FileDiffStats::modified("src.rs", 10, 5);
        assert_eq!(modified.lines_added, 10);
        assert_eq!(modified.lines_removed, 5);
        assert_eq!(modified.change_type, ChangeType::Modified);

        let renamed = FileDiffStats::renamed("renamed.rs");
        assert_eq!(renamed.change_type, ChangeType::Renamed);
    }

    #[test]
    fn diff_stats_add_works() {
        let a = DiffStats::from_values(2, 10, 5, 1, 0, 1);
        let b = DiffStats::from_values(3, 20, 10, 2, 1, 0);
        let merged = a + b;

        assert_eq!(merged.files_changed, 5);
        assert_eq!(merged.lines_added, 30);
        assert_eq!(merged.lines_removed, 15);
        assert_eq!(merged.files_added, 3);
        assert_eq!(merged.files_deleted, 1);
        assert_eq!(merged.files_renamed, 1);
    }

    #[test]
    fn builder_produces_correct_stats() {
        let mut builder = DiffStatsBuilder::new();
        builder
            .add_file(&FileDiffStats::added("a.rs", 10))
            .add_file(&FileDiffStats::deleted("b.rs", 5))
            .add_file(&FileDiffStats::modified("c.rs", 3, 2))
            .add_file(&FileDiffStats::renamed("d.rs"));

        let stats = builder.build();
        assert_eq!(stats.files_changed, 4);
        assert_eq!(stats.lines_added, 13);
        assert_eq!(stats.lines_removed, 7);
        assert_eq!(stats.files_added, 1);
        assert_eq!(stats.files_deleted, 1);
        assert_eq!(stats.files_renamed, 1);
    }

    #[test]
    fn format_stats_empty_shows_no_changes() {
        let stats = DiffStats::new();
        assert_eq!(format_stats(&stats), "No changes");
    }

    #[test]
    fn format_stats_short_works() {
        let stats = DiffStats::from_values(3, 10, 5, 0, 0, 0);
        assert_eq!(format_stats_short(&stats), "+10-5 files:3");
    }

    #[test]
    fn net_lines_calculates_correctly() {
        let stats = DiffStats::from_values(1, 10, 5, 0, 0, 0);
        assert_eq!(net_lines(&stats), 5);

        let negative_stats = DiffStats::from_values(1, 5, 10, 0, 0, 0);
        assert_eq!(net_lines(&negative_stats), -5);
    }
}
