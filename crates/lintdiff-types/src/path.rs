use std::fmt;

use serde::{Deserialize, Serialize};

/// A repo-relative, forward-slash path.
///
/// Protocol discipline: this appears in receipts, and therefore should be treated as stable.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NormPath(String);

impl NormPath {
    pub fn new(raw: impl AsRef<str>) -> Self {
        normalize_path(raw.as_ref())
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

/// Normalize an incoming path-like string into repo-relative forward-slash form.
///
/// - Converts Windows `\` to `/`
/// - Strips leading `./`
/// - Strips leading `a/` or `b/` (diff prefixes)
/// - Collapses repeated slashes
pub fn normalize_path(raw: &str) -> NormPath {
    let mut s = raw.trim().replace('\\', "/");

    // strip diff prefixes
    if let Some(stripped) = s.strip_prefix("a/") {
        s = stripped.to_string();
    } else if let Some(stripped) = s.strip_prefix("b/") {
        s = stripped.to_string();
    }

    // strip leading ./ (repeat to be safe)
    while let Some(stripped) = s.strip_prefix("./") {
        s = stripped.to_string();
    }

    // collapse multiple slashes
    while s.contains("//") {
        s = s.replace("//", "/");
    }

    NormPath(s)
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
