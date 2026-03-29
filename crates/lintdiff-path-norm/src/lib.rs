//! Path normalization for lintdiff.
//!
//! Provides utilities for normalizing file paths across different
//! operating systems and source control systems.

use std::borrow::Cow;
use std::path::Path;

/// Normalizes a file path for consistent comparison.
///
/// This function:
/// 1. Converts backslashes to forward slashes (Windows compatibility)
/// 2. Removes leading `./` prefixes
/// 3. Removes diff prefixes (`a/` or `b/`)
/// 4. Collapses duplicate slashes
/// 5. Removes trailing slashes
///
/// # Examples
/// ```
/// use lintdiff_path_norm::normalize;
///
/// assert_eq!(normalize("src\\lib.rs"), "src/lib.rs");
/// assert_eq!(normalize("./src/lib.rs"), "src/lib.rs");
/// assert_eq!(normalize("a/src/lib.rs"), "src/lib.rs");
/// assert_eq!(normalize("b/src/lib.rs"), "src/lib.rs");
/// ```
#[must_use]
pub fn normalize(path: &str) -> Cow<'_, str> {
    NormalizeConfig::default().normalize(path)
}

/// Normalizes a path and returns an owned String.
#[must_use]
pub fn normalize_owned(path: &str) -> String {
    normalize(path).into_owned()
}

/// Configuration for path normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct NormalizeConfig {
    /// Convert backslashes to forward slashes.
    pub slash_normalize: bool,
    /// Remove leading `./` prefixes.
    pub strip_dot_slash: bool,
    /// Remove diff prefixes (`a/` or `b/`).
    pub strip_diff_prefix: bool,
    /// Collapse duplicate slashes.
    pub collapse_slashes: bool,
    /// Remove trailing slashes.
    pub strip_trailing_slash: bool,
}

impl Default for NormalizeConfig {
    fn default() -> Self {
        Self {
            slash_normalize: true,
            strip_dot_slash: true,
            strip_diff_prefix: true,
            collapse_slashes: true,
            strip_trailing_slash: true,
        }
    }
}

impl NormalizeConfig {
    /// Create a new config with all options enabled.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slash_normalize: true,
            strip_dot_slash: true,
            strip_diff_prefix: true,
            collapse_slashes: true,
            strip_trailing_slash: true,
        }
    }

    /// Disable slash normalization (keep backslashes).
    #[must_use]
    pub const fn keep_backslashes(mut self) -> Self {
        self.slash_normalize = false;
        self
    }

    /// Disable dot-slash stripping.
    #[must_use]
    pub const fn keep_dot_slash(mut self) -> Self {
        self.strip_dot_slash = false;
        self
    }

    /// Disable diff prefix stripping.
    #[must_use]
    pub const fn keep_diff_prefix(mut self) -> Self {
        self.strip_diff_prefix = false;
        self
    }

    /// Normalize a path with this configuration.
    #[must_use]
    pub fn normalize<'a>(&self, path: &'a str) -> Cow<'a, str> {
        let mut result = Cow::Borrowed(path);

        // Step 1: Convert backslashes to forward slashes
        if self.slash_normalize && result.contains('\\') {
            result = Cow::Owned(result.replace('\\', "/"));
        }

        // Step 2: Collapse duplicate slashes
        if self.collapse_slashes {
            let owned = if result.contains("//") {
                let s = result.as_ref();
                let mut owned_result = String::with_capacity(s.len());
                let mut chars = s.chars().peekable();
                while let Some(c) = chars.next() {
                    owned_result.push(c);
                    if c == '/' {
                        // Skip all following slashes
                        while chars.peek() == Some(&'/') {
                            chars.next();
                        }
                    }
                }
                Some(owned_result)
            } else {
                None
            };
            if let Some(owned) = owned {
                result = Cow::Owned(owned);
            }
        }

        // Step 3: Strip diff prefixes (a/ or b/)
        if self.strip_diff_prefix {
            let stripped = result.strip_prefix("a/").or_else(|| result.strip_prefix("b/"));
            if let Some(s) = stripped {
                result = Cow::Owned(s.to_owned());
            }
        }

        // Step 4: Strip leading ./
        if self.strip_dot_slash {
            let stripped = result.strip_prefix("./");
            if let Some(s) = stripped {
                result = Cow::Owned(s.to_owned());
            }
        }

        // Step 5: Strip trailing slash
        if self.strip_trailing_slash {
            let len = result.len();
            if len > 1 && result.ends_with('/') {
                result = Cow::Owned(result[..len - 1].to_owned());
            }
        }

        result
    }
}

/// Checks if two paths are equivalent after normalization.
///
/// # Examples
/// ```
/// use lintdiff_path_norm::paths_eq;
///
/// assert!(paths_eq("src/lib.rs", "src\\lib.rs"));
/// assert!(paths_eq("./src/lib.rs", "src/lib.rs"));
/// assert!(paths_eq("a/src/lib.rs", "b/src/lib.rs"));
/// ```
#[must_use]
pub fn paths_eq(a: &str, b: &str) -> bool {
    normalize(a) == normalize(b)
}

/// Compares two paths after normalization.
///
/// # Examples
/// ```
/// use lintdiff_path_norm::paths_cmp;
///
/// assert_eq!(paths_cmp("a/lib.rs", "b/lib.rs"), std::cmp::Ordering::Equal);
/// assert_eq!(paths_cmp("src/a.rs", "src/b.rs"), std::cmp::Ordering::Less);
/// ```
#[must_use]
pub fn paths_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    normalize(a).cmp(&normalize(b))
}

/// Extracts the file extension from a path.
///
/// # Examples
/// ```
/// use lintdiff_path_norm::extension;
///
/// assert_eq!(extension("src/lib.rs"), Some("rs"));
/// assert_eq!(extension("src/lib"), None);
/// ```
#[must_use]
pub fn extension(path: &str) -> Option<&str> {
    // Handle both forward and backslashes
    let last_sep = path.rfind(&['/', '\\'][..]).map_or(0, |i| i + 1);
    let filename = &path[last_sep..];
    
    // Find the extension (after last .)
    let dot_pos = filename.rfind('.')?;
    
    // Extension must not be the first character (hidden files like .gitignore)
    if dot_pos == 0 {
        return None;
    }
    
    Some(&filename[dot_pos + 1..])
}

/// Extracts the file name from a path.
///
/// # Examples
/// ```
/// use lintdiff_path_norm::file_name;
///
/// assert_eq!(file_name("src/lib.rs"), Some("lib.rs"));
/// assert_eq!(file_name("src/"), None);
/// ```
#[must_use]
pub fn file_name(path: &str) -> Option<&str> {
    if path.is_empty() {
        return None;
    }
    
    // If path ends with a slash, it's a directory - no file name
    if path.ends_with('/') || path.ends_with('\\') {
        return None;
    }
    
    // Find the last separator (forward or backslash)
    let last_sep = path.rfind(&['/', '\\'][..]);
    
    last_sep.map_or(
        if path.is_empty() { None } else { Some(path) },
        |pos| {
            let name = &path[pos + 1..];
            if name.is_empty() { None } else { Some(name) }
        },
    )
}

/// Extracts the parent directory from a path.
///
/// # Examples
/// ```
/// use lintdiff_path_norm::parent;
///
/// assert_eq!(parent("src/lib.rs"), Some("src"));
/// assert_eq!(parent("lib.rs"), None);
/// ```
#[must_use]
pub fn parent(path: &str) -> Option<&str> {
    if path.is_empty() {
        return None;
    }
    
    // Strip trailing slashes
    let path = path.trim_end_matches(&['/', '\\'][..]);
    
    if path.is_empty() {
        return None;
    }
    
    // Find the last separator (forward or backslash)
    let last_sep = path.rfind(&['/', '\\'][..])?;
    
    // Return everything before the last separator
    Some(&path[..last_sep])
}

/// Joins path components with forward slashes.
///
/// # Examples
/// ```
/// use lintdiff_path_norm::join;
///
/// assert_eq!(join(&["src", "lib.rs"]), "src/lib.rs");
/// assert_eq!(join(&["src", "foo", "bar.rs"]), "src/foo/bar.rs");
/// ```
#[must_use]
pub fn join(components: &[&str]) -> String {
    components.join("/")
}

/// A normalized path wrapper that ensures consistent comparison.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NormalizedPath(String);

impl NormalizedPath {
    /// Create a new normalized path.
    #[must_use]
    pub fn new(path: &str) -> Self {
        Self(normalize_owned(path))
    }

    /// Get the normalized path as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the file extension.
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        extension(&self.0)
    }

    /// Get the file name.
    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        file_name(&self.0)
    }

    /// Get the parent directory.
    #[must_use]
    pub fn parent(&self) -> Option<&str> {
        parent(&self.0)
    }

    /// Check if this path starts with the given prefix.
    #[must_use]
    pub fn starts_with(&self, prefix: &str) -> bool {
        let normalized_prefix = normalize(prefix);
        self.0.starts_with(normalized_prefix.as_ref())
    }

    /// Check if this path ends with the given suffix.
    #[must_use]
    pub fn ends_with(&self, suffix: &str) -> bool {
        self.0.ends_with(suffix)
    }
}

impl std::fmt::Display for NormalizedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for NormalizedPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<Path> for NormalizedPath {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl From<&str> for NormalizedPath {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl From<String> for NormalizedPath {
    fn from(path: String) -> Self {
        Self::new(&path)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for NormalizedPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for NormalizedPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(&s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_basic() {
        assert_eq!(normalize("src/lib.rs"), "src/lib.rs");
        assert_eq!(normalize("./src/lib.rs"), "src/lib.rs");
        assert_eq!(normalize("src/lib.rs/"), "src/lib.rs");
    }

    #[test]
    fn test_normalize_windows() {
        assert_eq!(normalize("src\\lib.rs"), "src/lib.rs");
        assert_eq!(normalize("src\\foo\\bar.rs"), "src/foo/bar.rs");
    }

    #[test]
    fn test_normalize_diff_prefix() {
        assert_eq!(normalize("a/src/lib.rs"), "src/lib.rs");
        assert_eq!(normalize("b/src/lib.rs"), "src/lib.rs");
    }

    #[test]
    fn test_normalize_duplicate_slashes() {
        assert_eq!(normalize("src//lib.rs"), "src/lib.rs");
        assert_eq!(normalize("src///lib.rs"), "src/lib.rs");
    }
}
