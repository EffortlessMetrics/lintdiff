use std::fmt;

use serde::{Deserialize, Serialize};

/// A repo-relative, forward-slash path.
///
/// Protocol discipline: this appears in receipts, and therefore should be treated as stable.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NormPath(String);

impl NormPath {
    /// Construct a path using the historical normalization rules.
    ///
    /// This constructor remains compatibility-preserving for existing callers. New
    /// product code should use [`Self::from_repo_path`] when the input is already a
    /// repository path rather than Git diff transport syntax.
    pub fn new(raw: impl AsRef<str>) -> Self {
        normalize_path(raw.as_ref())
    }

    /// Construct a path from repository identity without interpreting `a/` or `b/`
    /// as Git diff prefixes.
    pub fn from_repo_path(raw: impl AsRef<str>) -> Self {
        normalize_repo_path(raw.as_ref())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for NormPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for NormPath {
    fn from(value: String) -> Self {
        NormPath::new(value)
    }
}

impl From<&str> for NormPath {
    fn from(value: &str) -> Self {
        NormPath::new(value)
    }
}

/// Normalize an incoming path-like string using the historical `NormPath::new`
/// behavior.
///
/// - Converts Windows `\` to `/`
/// - Strips leading `./`
/// - Strips leading `a/` or `b/` (legacy diff-prefix behavior)
/// - Collapses repeated slashes
pub fn normalize_path(raw: &str) -> NormPath {
    let mut s = raw.trim().replace('\\', "/");
    let mut had_diff_prefix = false;

    // strip leading ./ (repeat to be safe)
    while let Some(stripped) = s.strip_prefix("./") {
        s = stripped.to_string();
    }

    // Retain the historical behavior of removing repeated diff prefixes.
    while let Some(stripped) = s.strip_prefix("a/") {
        s = stripped.to_string();
        had_diff_prefix = true;
    }
    while let Some(stripped) = s.strip_prefix("b/") {
        s = stripped.to_string();
        had_diff_prefix = true;
    }

    // Handle cases such as `a/./path` after removing a legacy prefix.
    while let Some(stripped) = s.strip_prefix("./") {
        s = stripped.to_string();
    }

    // collapse multiple slashes
    while s.contains("//") {
        s = s.replace("//", "/");
    }

    if had_diff_prefix {
        while s.ends_with('/') {
            s.pop();
        }
    }

    NormPath(s)
}

/// Normalize a repository path without interpreting its first directory as Git
/// diff transport syntax.
pub fn normalize_repo_path(raw: &str) -> NormPath {
    let mut s = raw.trim().replace('\\', "/");

    while let Some(stripped) = s.strip_prefix("./") {
        s = stripped.to_string();
    }

    while s.contains("//") {
        s = s.replace("//", "/");
    }

    NormPath(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_path_diff_prefix_is_idempotent_for_nested_prefix() {
        let first = normalize_path("a\\a\\0");
        let second = normalize_path(first.as_str());

        assert_eq!(first, second);
    }
}

/// Inclusive 1-based line range.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct LineRange {
    pub start: u32,
    pub end: u32,
}

impl LineRange {
    pub fn new(start: u32, end: u32) -> Self {
        debug_assert!(start >= 1);
        debug_assert!(end >= start);
        Self { start, end }
    }

    pub fn intersects(&self, other: &LineRange) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    pub fn contains_line(&self, line: u32) -> bool {
        self.start <= line && line <= self.end
    }
}
