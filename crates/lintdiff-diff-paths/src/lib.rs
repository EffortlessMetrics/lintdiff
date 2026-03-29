//! Diff path extraction for lintdiff.
//!
//! This crate provides utilities for extracting and normalizing file paths
//! from unified diff headers (`---` and `+++` lines).
//!
//! # Overview
//!
//! When parsing unified diffs, each file's changes are prefixed with header lines:
//! ```text
//! --- a/path/to/old_file.txt
//! +++ b/path/to/new_file.txt
//! ```
//!
//! This crate handles:
//! - Parsing these header lines into structured data
//! - Detecting file creation, deletion, and renames
//! - Normalizing paths by stripping diff prefixes (`a/`, `b/`, `i/`)
//! - Handling `/dev/null` as a special path
//!
//! # Examples
//!
//! ```
//! use lintdiff_diff_paths::{DiffPaths, extract_paths_from_header};
//!
//! let header = "--- a/src/lib.rs\n+++ b/src/lib.rs\n";
//! let paths = extract_paths_from_header(header)?.unwrap();
//!
//! assert_eq!(paths.canonical_path_normalized(), Some("src/lib.rs"));
//! assert!(!paths.is_creation());
//! assert!(!paths.is_deletion());
//! # Ok::<(), lintdiff_diff_paths::DiffPathsError>(())
//! ```

use std::fmt;

/// Error type for diff path parsing failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffPathsError {
    /// The error message.
    message: String,
    /// The line that caused the error (if available).
    line: Option<String>,
}

impl DiffPathsError {
    /// Creates a new error with a message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
        }
    }

    /// Creates a new error with a message and the problematic line.
    #[must_use]
    pub fn with_line(message: impl Into<String>, line: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: Some(line.into()),
        }
    }

    /// Returns the error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the line that caused the error (if available).
    #[must_use]
    pub fn line(&self) -> Option<&str> {
        self.line.as_deref()
    }
}

impl fmt::Display for DiffPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.line {
            Some(line) => write!(f, "{}: {:?}", self.message, line),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for DiffPathsError {}

/// Represents file paths extracted from a unified diff header.
///
/// This struct holds the paths from the `---` (old) and `+++` (new) lines
/// of a unified diff header, along with optional timestamps.
///
/// # Examples
///
/// ```
/// use lintdiff_diff_paths::DiffPaths;
///
/// let paths = DiffPaths {
///     old_path: Some("src/old.rs".to_string()),
///     new_path: Some("src/new.rs".to_string()),
///     old_timestamp: None,
///     new_timestamp: None,
/// };
///
/// assert!(paths.is_rename());
/// assert_eq!(paths.canonical_path(), Some("src/new.rs"));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiffPaths {
    /// Path from the `---` line (may be `/dev/null` for new files).
    pub old_path: Option<String>,
    /// Path from the `+++` line (may be `/dev/null` for deleted files).
    pub new_path: Option<String>,
    /// Optional timestamp from the `---` line.
    pub old_timestamp: Option<String>,
    /// Optional timestamp from the `+++` line.
    pub new_timestamp: Option<String>,
}

impl DiffPaths {
    /// Creates a new `DiffPaths` with no paths set.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::new();
    /// assert!(paths.old_path.is_none());
    /// assert!(paths.new_path.is_none());
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            old_path: None,
            new_path: None,
            old_timestamp: None,
            new_timestamp: None,
        }
    }

    /// Creates a `DiffPaths` representing a file creation.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::creation("src/new_file.rs");
    /// assert!(paths.is_creation());
    /// assert!(!paths.is_deletion());
    /// ```
    #[must_use]
    pub fn creation(new_path: impl Into<String>) -> Self {
        Self {
            old_path: None,
            new_path: Some(new_path.into()),
            old_timestamp: None,
            new_timestamp: None,
        }
    }

    /// Creates a `DiffPaths` representing a file deletion.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::deletion("src/old_file.rs");
    /// assert!(paths.is_deletion());
    /// assert!(!paths.is_creation());
    /// ```
    #[must_use]
    pub fn deletion(old_path: impl Into<String>) -> Self {
        Self {
            old_path: Some(old_path.into()),
            new_path: None,
            old_timestamp: None,
            new_timestamp: None,
        }
    }

    /// Creates a `DiffPaths` representing a file modification (same path).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::modification("src/lib.rs");
    /// assert!(!paths.is_creation());
    /// assert!(!paths.is_deletion());
    /// assert!(!paths.is_rename());
    /// ```
    #[must_use]
    pub fn modification(path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            old_path: Some(path.clone()),
            new_path: Some(path),
            old_timestamp: None,
            new_timestamp: None,
        }
    }

    /// Creates a `DiffPaths` representing a file rename.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::rename("src/old.rs", "src/new.rs");
    /// assert!(paths.is_rename());
    /// assert_eq!(paths.canonical_path(), Some("src/new.rs"));
    /// ```
    #[must_use]
    pub fn rename(old_path: impl Into<String>, new_path: impl Into<String>) -> Self {
        Self {
            old_path: Some(old_path.into()),
            new_path: Some(new_path.into()),
            old_timestamp: None,
            new_timestamp: None,
        }
    }

    /// Parses `---` and `+++` lines from a diff header.
    ///
    /// The input should contain one or both of the header lines.
    /// Returns `Ok(None)` if no recognizable header lines are found.
    ///
    /// # Supported Formats
    ///
    /// - `--- path` or `--- path\ttimestamp`
    /// - `+++ path` or `+++ path\ttimestamp`
    /// - Combined: `--- a/file\n+++ b/file`
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let header = "--- a/src/lib.rs\n+++ b/src/lib.rs\n";
    /// let paths = DiffPaths::parse(header)?.unwrap();
    ///
    /// assert_eq!(paths.old_path, Some("a/src/lib.rs".to_string()));
    /// assert_eq!(paths.new_path, Some("b/src/lib.rs".to_string()));
    /// # Ok::<(), lintdiff_diff_paths::DiffPathsError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// This function currently never returns an error, but returns a `Result`
    /// for API consistency and future extensibility.
    pub fn parse(diff_header: &str) -> Result<Option<Self>, DiffPathsError> {
        let mut result = Self::new();

        for line in diff_header.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("---") {
                let rest = rest.trim();
                if rest.is_empty() {
                    continue;
                }
                let (path, timestamp) = split_path_and_timestamp(rest);
                result.old_path = Some(path.to_string());
                result.old_timestamp = timestamp.map(std::string::ToString::to_string);
            } else if let Some(rest) = trimmed.strip_prefix("+++") {
                let rest = rest.trim();
                if rest.is_empty() {
                    continue;
                }
                let (path, timestamp) = split_path_and_timestamp(rest);
                result.new_path = Some(path.to_string());
                result.new_timestamp = timestamp.map(std::string::ToString::to_string);
            }
        }

        if result.old_path.is_none() && result.new_path.is_none() {
            return Ok(None);
        }

        Ok(Some(result))
    }

    /// Returns the canonical path for this diff.
    ///
    /// This prefers the new path (if available), falling back to the old path.
    /// Returns `None` if both paths are missing (shouldn't happen in practice).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::modification("src/lib.rs");
    /// assert_eq!(paths.canonical_path(), Some("src/lib.rs"));
    ///
    /// let deletion = DiffPaths::deletion("src/deleted.rs");
    /// assert_eq!(deletion.canonical_path(), Some("src/deleted.rs"));
    /// ```
    #[must_use]
    pub fn canonical_path(&self) -> Option<&str> {
        self.new_path
            .as_deref()
            .filter(|p| !is_dev_null(p))
            .or_else(|| self.old_path.as_deref().filter(|p| !is_dev_null(p)))
    }

    /// Returns the old path, stripping any diff prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::parse("--- a/src/lib.rs\n+++ b/src/lib.rs\n")?.unwrap();
    /// assert_eq!(paths.old_path_normalized(), Some("src/lib.rs"));
    /// # Ok::<(), lintdiff_diff_paths::DiffPathsError>(())
    /// ```
    #[must_use]
    pub fn old_path_normalized(&self) -> Option<&str> {
        self.old_path.as_deref().map(strip_diff_prefix)
    }

    /// Returns the new path, stripping any diff prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::parse("--- a/src/lib.rs\n+++ b/src/lib.rs\n")?.unwrap();
    /// assert_eq!(paths.new_path_normalized(), Some("src/lib.rs"));
    /// # Ok::<(), lintdiff_diff_paths::DiffPathsError>(())
    /// ```
    #[must_use]
    pub fn new_path_normalized(&self) -> Option<&str> {
        self.new_path.as_deref().map(strip_diff_prefix)
    }

    /// Returns the canonical path with diff prefixes stripped.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::parse("--- a/src/lib.rs\n+++ b/src/lib.rs\n")?.unwrap();
    /// assert_eq!(paths.canonical_path_normalized(), Some("src/lib.rs"));
    /// # Ok::<(), lintdiff_diff_paths::DiffPathsError>(())
    /// ```
    #[must_use]
    pub fn canonical_path_normalized(&self) -> Option<&str> {
        self.new_path_normalized()
            .or_else(|| self.old_path_normalized())
    }

    /// Checks if this represents a new file creation.
    ///
    /// Returns `true` if the old path is `/dev/null` or missing.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let creation = DiffPaths::parse("--- /dev/null\n+++ b/new_file.rs\n")?.unwrap();
    /// assert!(creation.is_creation());
    ///
    /// let modification = DiffPaths::modification("src/lib.rs");
    /// assert!(!modification.is_creation());
    /// # Ok::<(), lintdiff_diff_paths::DiffPathsError>(())
    /// ```
    #[must_use]
    pub fn is_creation(&self) -> bool {
        self.old_path.as_ref().is_none_or(|p| is_dev_null(p))
    }

    /// Checks if this represents a file deletion.
    ///
    /// Returns `true` if the new path is `/dev/null` or missing.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let deletion = DiffPaths::parse("--- a/old_file.rs\n+++ /dev/null\n")?.unwrap();
    /// assert!(deletion.is_deletion());
    ///
    /// let modification = DiffPaths::modification("src/lib.rs");
    /// assert!(!modification.is_deletion());
    /// # Ok::<(), lintdiff_diff_paths::DiffPathsError>(())
    /// ```
    #[must_use]
    pub fn is_deletion(&self) -> bool {
        self.new_path.as_ref().is_none_or(|p| is_dev_null(p))
    }

    /// Checks if this represents a file rename.
    ///
    /// Returns `true` if both paths exist and are different (not counting
    /// creation/deletion via `/dev/null`).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let rename = DiffPaths::rename("src/old.rs", "src/new.rs");
    /// assert!(rename.is_rename());
    ///
    /// let modification = DiffPaths::modification("src/lib.rs");
    /// assert!(!modification.is_rename());
    ///
    /// let creation = DiffPaths::creation("src/new.rs");
    /// assert!(!creation.is_rename());
    /// ```
    #[must_use]
    pub fn is_rename(&self) -> bool {
        match (self.old_path_normalized(), self.new_path_normalized()) {
            (Some(old), Some(new)) => {
                !is_dev_null(old) && !is_dev_null(new) && old != new
            }
            _ => false,
        }
    }

    /// Checks if this represents a modification (same file, not rename/create/delete).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let mod_paths = DiffPaths::modification("src/lib.rs");
    /// assert!(mod_paths.is_modification());
    ///
    /// let rename = DiffPaths::rename("src/old.rs", "src/new.rs");
    /// assert!(!rename.is_modification());
    /// ```
    #[must_use]
    pub fn is_modification(&self) -> bool {
        match (self.old_path_normalized(), self.new_path_normalized()) {
            (Some(old), Some(new)) => {
                !is_dev_null(old) && !is_dev_null(new) && old == new
            }
            _ => false,
        }
    }

    /// Checks if this represents a binary file change.
    ///
    /// Binary files may have special markers in some diff formats.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::modification("image.png");
    /// // Standard diff headers don't indicate binary directly
    /// assert!(!paths.is_binary());
    /// ```
    #[must_use]
    pub const fn is_binary(&self) -> bool {
        // Standard diff headers don't carry binary info
        // This is a placeholder for extended formats
        false
    }

    /// Strips a common prefix from both paths.
    ///
    /// This is useful for removing the `a/` and `b/` prefixes added by git.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::parse("--- a/src/lib.rs\n+++ b/src/lib.rs\n")?.unwrap();
    /// let stripped = paths.strip_prefix("a/");
    ///
    /// assert_eq!(stripped.old_path, Some("src/lib.rs".to_string()));
    /// # Ok::<(), lintdiff_diff_paths::DiffPathsError>(())
    /// ```
    #[must_use]
    pub fn strip_prefix(&self, prefix: &str) -> Self {
        Self {
            old_path: self.old_path.as_ref().map(|p| {
                if is_dev_null(p) {
                    p.clone()
                } else {
                    p.strip_prefix(prefix).unwrap_or(p).to_string()
                }
            }),
            new_path: self.new_path.as_ref().map(|p| {
                if is_dev_null(p) {
                    p.clone()
                } else {
                    p.strip_prefix(prefix).unwrap_or(p).to_string()
                }
            }),
            old_timestamp: self.old_timestamp.clone(),
            new_timestamp: self.new_timestamp.clone(),
        }
    }

    /// Strips diff prefixes (`a/`, `b/`, `i/`) from both paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::parse("--- a/src/lib.rs\n+++ b/src/lib.rs\n")?.unwrap();
    /// let normalized = paths.strip_diff_prefixes();
    ///
    /// assert_eq!(normalized.old_path, Some("src/lib.rs".to_string()));
    /// assert_eq!(normalized.new_path, Some("src/lib.rs".to_string()));
    /// # Ok::<(), lintdiff_diff_paths::DiffPathsError>(())
    /// ```
    #[must_use]
    pub fn strip_diff_prefixes(&self) -> Self {
        Self {
            old_path: self.old_path.as_ref().map(|p| normalize_path(p).into_owned()),
            new_path: self.new_path.as_ref().map(|p| normalize_path(p).into_owned()),
            old_timestamp: self.old_timestamp.clone(),
            new_timestamp: self.new_timestamp.clone(),
        }
    }

    /// Returns both paths as a tuple (old, new).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::rename("old.rs", "new.rs");
    /// assert_eq!(paths.as_tuple(), (Some("old.rs"), Some("new.rs")));
    /// ```
    #[must_use]
    pub fn as_tuple(&self) -> (Option<&str>, Option<&str>) {
        (self.old_path.as_deref(), self.new_path.as_deref())
    }

    /// Returns both normalized paths as a tuple (old, new).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::parse("--- a/old.rs\n+++ b/new.rs\n")?.unwrap();
    /// assert_eq!(paths.as_tuple_normalized(), (Some("old.rs"), Some("new.rs")));
    /// # Ok::<(), lintdiff_diff_paths::DiffPathsError>(())
    /// ```
    #[must_use]
    pub fn as_tuple_normalized(&self) -> (Option<&str>, Option<&str>) {
        (self.old_path_normalized(), self.new_path_normalized())
    }

    /// Checks if either path matches the given pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::modification("src/lib.rs");
    /// assert!(paths.matches_path("src/lib.rs"));
    /// assert!(!paths.matches_path("other.rs"));
    /// ```
    #[must_use]
    pub fn matches_path(&self, pattern: &str) -> bool {
        let normalized_pattern = strip_diff_prefix(pattern);
        
        self.old_path_normalized() == Some(normalized_pattern)
            || self.new_path_normalized() == Some(normalized_pattern)
    }

    /// Checks if either path ends with the given suffix.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_diff_paths::DiffPaths;
    ///
    /// let paths = DiffPaths::modification("src/lib.rs");
    /// assert!(paths.path_ends_with(".rs"));
    /// assert!(!paths.path_ends_with(".txt"));
    /// ```
    #[must_use]
    pub fn path_ends_with(&self, suffix: &str) -> bool {
        self.old_path_normalized()
            .is_some_and(|p| p.ends_with(suffix))
            || self.new_path_normalized()
                .is_some_and(|p| p.ends_with(suffix))
    }
}

// ============================================================================
// Path normalization functions
// ============================================================================

/// Strips diff prefixes from a path.
///
/// Recognized prefixes: `a/`, `b/`, `i/`
///
/// # Examples
///
/// ```
/// use lintdiff_diff_paths::strip_diff_prefix;
///
/// assert_eq!(strip_diff_prefix("a/src/lib.rs"), "src/lib.rs");
/// assert_eq!(strip_diff_prefix("b/src/lib.rs"), "src/lib.rs");
/// assert_eq!(strip_diff_prefix("i/src/lib.rs"), "src/lib.rs");
/// assert_eq!(strip_diff_prefix("src/lib.rs"), "src/lib.rs");
/// assert_eq!(strip_diff_prefix("/dev/null"), "/dev/null");
/// ```
#[must_use]
pub fn strip_diff_prefix(path: &str) -> &str {
    if is_dev_null(path) {
        return path;
    }
    
    // Strip diff prefixes (a/, b/, i/) from the start of the path
    // Only strip once to avoid over-stripping paths like "a/a/b"
    path.strip_prefix("a/")
        .or_else(|| path.strip_prefix("b/"))
        .or_else(|| path.strip_prefix("i/"))
        .unwrap_or(path)
}

/// Checks if a path represents `/dev/null`.
///
/// # Examples
///
/// ```
/// use lintdiff_diff_paths::is_dev_null;
///
/// assert!(is_dev_null("/dev/null"));
/// assert!(is_dev_null("dev/null"));  // Also accepted
/// assert!(!is_dev_null("src/lib.rs"));
/// ```
#[must_use]
pub fn is_dev_null(path: &str) -> bool {
    path == "/dev/null" || path == "dev/null"
}

/// Normalizes a path for consistent comparison.
///
/// This function:
/// 1. Preserves `/dev/null` as-is
/// 2. Strips diff prefixes (`a/`, `b/`, `i/`)
/// 3. Converts backslashes to forward slashes
///
/// # Examples
///
/// ```
/// use lintdiff_diff_paths::normalize_path;
///
/// assert_eq!(normalize_path("a/src/lib.rs"), "src/lib.rs");
/// assert_eq!(normalize_path("b\\src\\lib.rs"), "src/lib.rs");
/// assert_eq!(normalize_path("/dev/null"), "/dev/null");
/// ```
#[must_use]
pub fn normalize_path(path: &str) -> std::borrow::Cow<'_, str> {
    if is_dev_null(path) {
        return std::borrow::Cow::Borrowed(path);
    }
    
    // First convert backslashes to forward slashes
    let converted = if path.contains('\\') {
        std::borrow::Cow::Owned(path.replace('\\', "/"))
    } else {
        std::borrow::Cow::Borrowed(path)
    };
    
    // Then strip diff prefixes recursively until none remain
    let mut stripped = strip_diff_prefix(&converted);
    let mut prev_len = converted.len();
    
    loop {
        let next = strip_diff_prefix(stripped);
        if next.len() >= prev_len {
            // No more stripping possible
            break;
        }
        prev_len = stripped.len();
        stripped = next;
    }
    
    if stripped.len() == converted.len() {
        // No prefix was stripped, return the converted version
        converted
    } else {
        // Prefix was stripped, need to return the stripped version
        std::borrow::Cow::Owned(stripped.to_string())
    }
}

/// Normalizes a path and returns an owned String.
///
/// # Examples
///
/// ```
/// use lintdiff_diff_paths::normalize_path_owned;
///
/// assert_eq!(normalize_path_owned("a/src/lib.rs"), "src/lib.rs");
/// ```
#[must_use]
pub fn normalize_path_owned(path: &str) -> String {
    normalize_path(path).into_owned()
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Splits a path/timestamp string into (path, `optional_timestamp`).
///
/// Timestamps are typically separated by tabs or multiple spaces.
fn split_path_and_timestamp(s: &str) -> (&str, Option<&str>) {
    // First try tab-separated
    if let Some(idx) = s.find('\t') {
        return (&s[..idx], Some(&s[idx + 1..]));
    }
    
    // Then try double-space separated (some formats use this)
    if let Some(idx) = s.find("  ") {
        return (&s[..idx], Some(s[idx + 2..].trim()));
    }
    
    (s, None)
}

// ============================================================================
// Convenience functions
// ============================================================================

/// Extracts paths from a diff header string.
///
/// This is a convenience wrapper around [`DiffPaths::parse`].
///
/// # Examples
///
/// ```
/// use lintdiff_diff_paths::extract_paths_from_header;
///
/// let header = "--- a/src/lib.rs\n+++ b/src/lib.rs\n";
/// let paths = extract_paths_from_header(header)?.unwrap();
///
/// assert_eq!(paths.canonical_path_normalized(), Some("src/lib.rs"));
/// # Ok::<(), lintdiff_diff_paths::DiffPathsError>(())
/// ```
///
/// # Errors
///
/// This function currently never returns an error, but returns a `Result`
/// for API consistency and future extensibility.
pub fn extract_paths_from_header(header: &str) -> Result<Option<DiffPaths>, DiffPathsError> {
    DiffPaths::parse(header)
}

/// Extracts and normalizes paths from a diff header.
///
/// Returns paths with diff prefixes already stripped.
///
/// # Examples
///
/// ```
/// use lintdiff_diff_paths::extract_normalized_paths;
///
/// let header = "--- a/src/lib.rs\n+++ b/src/lib.rs\n";
/// let paths = extract_normalized_paths(header)?.unwrap();
///
/// assert_eq!(paths.old_path, Some("src/lib.rs".to_string()));
/// assert_eq!(paths.new_path, Some("src/lib.rs".to_string()));
/// # Ok::<(), lintdiff_diff_paths::DiffPathsError>(())
/// ```
///
/// # Errors
///
/// This function currently never returns an error, but returns a `Result`
/// for API consistency and future extensibility.
pub fn extract_normalized_paths(header: &str) -> Result<Option<DiffPaths>, DiffPathsError> {
    Ok(DiffPaths::parse(header)?.map(|p| p.strip_diff_prefixes()))
}

/// Parses a single `---` line.
///
/// # Examples
///
/// ```
/// use lintdiff_diff_paths::parse_old_line;
///
/// let (path, ts) = parse_old_line("--- a/src/lib.rs\t2024-01-01 12:00:00");
/// assert_eq!(path, Some("a/src/lib.rs"));
/// assert_eq!(ts, Some("2024-01-01 12:00:00"));
/// ```
#[must_use]
pub fn parse_old_line(line: &str) -> (Option<&str>, Option<&str>) {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("---") {
        let rest = rest.trim();
        if rest.is_empty() {
            return (None, None);
        }
        let (path, ts) = split_path_and_timestamp(rest);
        (Some(path), ts)
    } else {
        (None, None)
    }
}

/// Parses a single `+++` line.
///
/// # Examples
///
/// ```
/// use lintdiff_diff_paths::parse_new_line;
///
/// let (path, ts) = parse_new_line("+++ b/src/lib.rs\t2024-01-01 12:00:00");
/// assert_eq!(path, Some("b/src/lib.rs"));
/// assert_eq!(ts, Some("2024-01-01 12:00:00"));
/// ```
#[must_use]
pub fn parse_new_line(line: &str) -> (Option<&str>, Option<&str>) {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("+++") {
        let rest = rest.trim();
        if rest.is_empty() {
            return (None, None);
        }
        let (path, ts) = split_path_and_timestamp(rest);
        (Some(path), ts)
    } else {
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dev_null() {
        assert!(is_dev_null("/dev/null"));
        assert!(is_dev_null("dev/null"));
        assert!(!is_dev_null("/dev/nulls"));
        assert!(!is_dev_null("src/lib.rs"));
        assert!(!is_dev_null(""));
    }

    #[test]
    fn test_strip_diff_prefix() {
        assert_eq!(strip_diff_prefix("a/src/lib.rs"), "src/lib.rs");
        assert_eq!(strip_diff_prefix("b/src/lib.rs"), "src/lib.rs");
        assert_eq!(strip_diff_prefix("i/src/lib.rs"), "src/lib.rs");
        assert_eq!(strip_diff_prefix("src/lib.rs"), "src/lib.rs");
        assert_eq!(strip_diff_prefix("/dev/null"), "/dev/null");
        assert_eq!(strip_diff_prefix("a/"), "");
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("a/src/lib.rs"), "src/lib.rs");
        assert_eq!(normalize_path("b\\src\\lib.rs"), "src/lib.rs");
        assert_eq!(normalize_path("/dev/null"), "/dev/null");
    }

    #[test]
    fn test_split_path_and_timestamp() {
        assert_eq!(split_path_and_timestamp("path"), ("path", None));
        assert_eq!(
            split_path_and_timestamp("path\t2024-01-01"),
            ("path", Some("2024-01-01"))
        );
        assert_eq!(
            split_path_and_timestamp("path  2024-01-01"),
            ("path", Some("2024-01-01"))
        );
    }

    #[test]
    fn test_diff_paths_new() {
        let paths = DiffPaths::new();
        assert!(paths.old_path.is_none());
        assert!(paths.new_path.is_none());
        assert!(paths.old_timestamp.is_none());
        assert!(paths.new_timestamp.is_none());
    }

    #[test]
    fn test_diff_paths_creation() {
        let paths = DiffPaths::creation("src/new.rs");
        assert!(paths.is_creation());
        assert!(!paths.is_deletion());
        assert!(!paths.is_rename());
        assert_eq!(paths.canonical_path(), Some("src/new.rs"));
    }

    #[test]
    fn test_diff_paths_deletion() {
        let paths = DiffPaths::deletion("src/old.rs");
        assert!(paths.is_deletion());
        assert!(!paths.is_creation());
        assert!(!paths.is_rename());
        assert_eq!(paths.canonical_path(), Some("src/old.rs"));
    }

    #[test]
    fn test_diff_paths_modification() {
        let paths = DiffPaths::modification("src/lib.rs");
        assert!(paths.is_modification());
        assert!(!paths.is_creation());
        assert!(!paths.is_deletion());
        assert!(!paths.is_rename());
    }

    #[test]
    fn test_diff_paths_rename() {
        let paths = DiffPaths::rename("src/old.rs", "src/new.rs");
        assert!(paths.is_rename());
        assert!(!paths.is_creation());
        assert!(!paths.is_deletion());
        assert!(!paths.is_modification());
    }

    #[test]
    fn test_diff_paths_parse_standard() {
        let header = "--- a/src/lib.rs\n+++ b/src/lib.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();
        
        assert_eq!(paths.old_path, Some("a/src/lib.rs".to_string()));
        assert_eq!(paths.new_path, Some("b/src/lib.rs".to_string()));
        assert!(paths.is_modification());
    }

    #[test]
    fn test_diff_paths_parse_creation() {
        let header = "--- /dev/null\n+++ b/new_file.rs\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();
        
        assert!(paths.is_creation());
        assert_eq!(paths.canonical_path(), Some("b/new_file.rs"));
    }

    #[test]
    fn test_diff_paths_parse_deletion() {
        let header = "--- a/old_file.rs\n+++ /dev/null\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();
        
        assert!(paths.is_deletion());
        assert_eq!(paths.canonical_path(), Some("a/old_file.rs"));
    }

    #[test]
    fn test_diff_paths_parse_empty() {
        let result = DiffPaths::parse("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_diff_paths_parse_no_header_lines() {
        let result = DiffPaths::parse("some random text\nno headers here").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_strip_prefix() {
        let paths = DiffPaths::parse("--- a/src/lib.rs\n+++ b/src/lib.rs\n")
            .unwrap()
            .unwrap();
        let stripped = paths.strip_prefix("a/");
        
        assert_eq!(stripped.old_path, Some("src/lib.rs".to_string()));
        // b/ prefix not stripped since we only asked for a/
        assert_eq!(stripped.new_path, Some("b/src/lib.rs".to_string()));
    }

    #[test]
    fn test_strip_diff_prefixes() {
        let paths = DiffPaths::parse("--- a/src/lib.rs\n+++ b/src/lib.rs\n")
            .unwrap()
            .unwrap();
        let normalized = paths.strip_diff_prefixes();
        
        assert_eq!(normalized.old_path, Some("src/lib.rs".to_string()));
        assert_eq!(normalized.new_path, Some("src/lib.rs".to_string()));
    }

    #[test]
    fn test_parse_with_timestamp() {
        let header = "--- a/file.rs\t2024-01-01 12:00:00\n+++ b/file.rs\t2024-01-02 13:00:00\n";
        let paths = DiffPaths::parse(header).unwrap().unwrap();
        
        assert_eq!(paths.old_timestamp, Some("2024-01-01 12:00:00".to_string()));
        assert_eq!(paths.new_timestamp, Some("2024-01-02 13:00:00".to_string()));
    }

    #[test]
    fn test_error_display() {
        let err = DiffPathsError::new("test error");
        assert_eq!(format!("{}", err), "test error");
        
        let err_with_line = DiffPathsError::with_line("test error", "bad line");
        assert_eq!(format!("{}", err_with_line), "test error: \"bad line\"");
    }
}
