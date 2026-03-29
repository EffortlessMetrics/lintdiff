//! Sorting utilities for lintdiff.
//!
//! Provides stable, deterministic sorting for diagnostics, findings,
//! and other collections with consistent ordering across runs.

use std::cmp::Ordering;

/// Sort key for determining sort order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SortKey {
    /// Sort by file path (default).
    #[default]
    Path,
    /// Sort by severity (most severe first).
    Severity,
    /// Sort by line number.
    Line,
    /// Sort by column number.
    Column,
    /// Sort by code/lint name.
    Code,
    /// Sort by message content.
    Message,
    /// Sort by fingerprint.
    Fingerprint,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SortDirection {
    /// Ascending order (A-Z, 0-9).
    #[default]
    Ascending,
    /// Descending order (Z-A, 9-0).
    Descending,
}

/// Configuration for sorting behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortConfig {
    /// Primary sort key.
    pub primary: SortKey,
    /// Secondary sort key (for ties).
    pub secondary: Option<SortKey>,
    /// Tertiary sort key (for ties).
    pub tertiary: Option<SortKey>,
    /// Sort direction.
    pub direction: SortDirection,
}

impl Default for SortConfig {
    fn default() -> Self {
        Self {
            primary: SortKey::Path,
            secondary: Some(SortKey::Line),
            tertiary: Some(SortKey::Column),
            direction: SortDirection::Ascending,
        }
    }
}

impl SortConfig {
    /// Create a new sort config with the given primary key.
    #[must_use]
    pub fn new(primary: SortKey) -> Self {
        Self {
            primary,
            ..Self::default()
        }
    }

    /// Set the secondary sort key.
    #[must_use]
    pub const fn with_secondary(mut self, key: SortKey) -> Self {
        self.secondary = Some(key);
        self
    }

    /// Set the tertiary sort key.
    #[must_use]
    pub const fn with_tertiary(mut self, key: SortKey) -> Self {
        self.tertiary = Some(key);
        self
    }

    /// Set sort direction to descending.
    #[must_use]
    pub const fn descending(mut self) -> Self {
        self.direction = SortDirection::Descending;
        self
    }

    /// Set sort direction to ascending.
    #[must_use]
    pub const fn ascending(mut self) -> Self {
        self.direction = SortDirection::Ascending;
        self
    }

    /// Create a config for sorting by severity (most severe first).
    #[must_use]
    pub fn by_severity() -> Self {
        Self::new(SortKey::Severity)
            .with_secondary(SortKey::Path)
            .ascending()
    }

    /// Create a config for sorting by file path.
    #[must_use]
    pub fn by_path() -> Self {
        Self::new(SortKey::Path)
            .with_secondary(SortKey::Line)
            .with_tertiary(SortKey::Column)
    }

    /// Create a config for sorting by code/lint name.
    #[must_use]
    pub fn by_code() -> Self {
        Self::new(SortKey::Code).with_secondary(SortKey::Path)
    }
}

/// A trait for items that can be sorted.
pub trait Sortable {
    /// Get the file path for sorting.
    fn sort_path(&self) -> &str;

    /// Get the severity level (0=hint, 1=note, 2=warning, 3=error, 4=fatal).
    fn sort_severity(&self) -> u8;

    /// Get the line number for sorting.
    fn sort_line(&self) -> u32;

    /// Get the column number for sorting.
    fn sort_column(&self) -> u32;

    /// Get the code/lint name for sorting.
    fn sort_code(&self) -> &str;

    /// Get the message for sorting.
    fn sort_message(&self) -> &str;

    /// Get the fingerprint for sorting.
    fn sort_fingerprint(&self) -> &str;
}

/// Compare two sortable items using the given key.
#[must_use]
pub fn compare_by_key<T: Sortable>(a: &T, b: &T, key: SortKey) -> Ordering {
    match key {
        SortKey::Path => a.sort_path().cmp(b.sort_path()),
        SortKey::Severity => b.sort_severity().cmp(&a.sort_severity()), // Higher severity first
        SortKey::Line => a.sort_line().cmp(&b.sort_line()),
        SortKey::Column => a.sort_column().cmp(&b.sort_column()),
        SortKey::Code => a.sort_code().cmp(b.sort_code()),
        SortKey::Message => a.sort_message().cmp(b.sort_message()),
        SortKey::Fingerprint => a.sort_fingerprint().cmp(b.sort_fingerprint()),
    }
}

/// Compare two sortable items using a configuration.
#[must_use]
pub fn compare<T: Sortable>(a: &T, b: &T, config: &SortConfig) -> Ordering {
    let primary = compare_by_key(a, b, config.primary);
    if primary != Ordering::Equal {
        return apply_direction(primary, config.direction);
    }

    if let Some(secondary) = config.secondary {
        let secondary = compare_by_key(a, b, secondary);
        if secondary != Ordering::Equal {
            return apply_direction(secondary, config.direction);
        }
    }

    if let Some(tertiary) = config.tertiary {
        let tertiary = compare_by_key(a, b, tertiary);
        if tertiary != Ordering::Equal {
            return apply_direction(tertiary, config.direction);
        }
    }

    Ordering::Equal
}

const fn apply_direction(ordering: Ordering, direction: SortDirection) -> Ordering {
    match direction {
        SortDirection::Ascending => ordering,
        SortDirection::Descending => ordering.reverse(),
    }
}

/// Sort a slice of sortable items using the given configuration.
///
/// # Examples
/// ```
/// use lintdiff_sort::{sort_slice, SortConfig, Sortable};
///
/// #[derive(Debug, Clone)]
/// struct Item { path: String, line: u32 }
///
/// impl Sortable for Item {
///     fn sort_path(&self) -> &str { &self.path }
///     fn sort_severity(&self) -> u8 { 0 }
///     fn sort_line(&self) -> u32 { self.line }
///     fn sort_column(&self) -> u32 { 0 }
///     fn sort_code(&self) -> &str { "" }
///     fn sort_message(&self) -> &str { "" }
///     fn sort_fingerprint(&self) -> &str { "" }
/// }
///
/// let mut items = vec![
///     Item { path: "b.rs".into(), line: 10 },
///     Item { path: "a.rs".into(), line: 5 },
/// ];
///
/// sort_slice(&mut items, &SortConfig::by_path());
/// assert_eq!(items[0].path, "a.rs");
/// ```
pub fn sort_slice<T: Sortable>(items: &mut [T], config: &SortConfig) {
    items.sort_by(|a, b| compare(a, b, config));
}

/// Create a sorted iterator from an unsorted one.
pub fn sorted<'a, T: Sortable + 'a>(
    items: impl IntoIterator<Item = T> + 'a,
    config: &'a SortConfig,
) -> impl Iterator<Item = T> + 'a {
    let mut items: Vec<T> = items.into_iter().collect();
    sort_slice(&mut items, config);
    items.into_iter()
}

/// Compare two strings using natural sort order.
///
/// Natural sort handles numeric parts as numbers, so "file10" comes after "file2".
///
/// # Examples
/// ```
/// use lintdiff_sort::natural_compare;
/// use std::cmp::Ordering;
///
/// assert_eq!(natural_compare("file2", "file10"), Ordering::Less);
/// assert_eq!(natural_compare("file10", "file2"), Ordering::Greater);
/// assert_eq!(natural_compare("file2a", "file2b"), Ordering::Less);
/// ```
#[must_use]
pub fn natural_compare(a: &str, b: &str) -> Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        let a_chunk = next_chunk(&mut a_chars);
        let b_chunk = next_chunk(&mut b_chars);

        match (a_chunk, b_chunk) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(Chunk::Alpha(a_str)), Some(Chunk::Alpha(b_str))) => {
                let cmp = a_str.cmp(&b_str);
                if cmp != Ordering::Equal {
                    return cmp;
                }
            }
            (Some(Chunk::Num(a_num, a_len)), Some(Chunk::Num(b_num, b_len))) => {
                // Compare by numeric value first, then by length (for leading zeros)
                let cmp = a_num.cmp(&b_num);
                if cmp != Ordering::Equal {
                    return cmp;
                }
                // If numbers are equal, shorter string (fewer leading zeros) comes first
                let len_cmp = a_len.cmp(&b_len);
                if len_cmp != Ordering::Equal {
                    return len_cmp;
                }
            }
            (Some(Chunk::Alpha(_)), Some(Chunk::Num(_, _))) => {
                // Numbers come before letters in natural sort
                return Ordering::Greater;
            }
            (Some(Chunk::Num(_, _)), Some(Chunk::Alpha(_))) => {
                return Ordering::Less;
            }
        }
    }
}

/// Represents a chunk of either digits or non-digits
enum Chunk {
    Alpha(String),
    Num(u64, usize), // (numeric value, digit count for leading zero handling)
}

/// Extract the next chunk from a character iterator
fn next_chunk(chars: &mut std::iter::Peekable<impl Iterator<Item = char>>) -> Option<Chunk> {
    let first = *chars.peek()?;

    if first.is_ascii_digit() {
        // Collect all consecutive digits
        let mut digit_str = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                digit_str.push(c);
                chars.next();
            } else {
                break;
            }
        }
        let num: u64 = digit_str.parse().unwrap_or(0);
        Some(Chunk::Num(num, digit_str.len()))
    } else {
        // Collect all consecutive non-digits
        let mut alpha_str = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                break;
            }
            alpha_str.push(c);
            chars.next();
        }
        Some(Chunk::Alpha(alpha_str))
    }
}

/// Sort a slice of strings using natural sort order.
pub fn natural_sort(strings: &mut [&str]) {
    strings.sort_by(|a, b| natural_compare(a, b));
}

/// Sort a slice of strings using natural sort order (owned version).
pub fn natural_sort_owned(strings: &mut [String]) {
    strings.sort_by(|a, b| natural_compare(a, b));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestItem {
        path: String,
        severity: u8,
        line: u32,
        column: u32,
        code: String,
        message: String,
        fingerprint: String,
    }

    impl Sortable for TestItem {
        fn sort_path(&self) -> &str {
            &self.path
        }
        fn sort_severity(&self) -> u8 {
            self.severity
        }
        fn sort_line(&self) -> u32 {
            self.line
        }
        fn sort_column(&self) -> u32 {
            self.column
        }
        fn sort_code(&self) -> &str {
            &self.code
        }
        fn sort_message(&self) -> &str {
            &self.message
        }
        fn sort_fingerprint(&self) -> &str {
            &self.fingerprint
        }
    }

    impl Default for TestItem {
        fn default() -> Self {
            Self {
                path: String::new(),
                severity: 0,
                line: 0,
                column: 0,
                code: String::new(),
                message: String::new(),
                fingerprint: String::new(),
            }
        }
    }

    #[test]
    fn test_sort_key_default() {
        assert_eq!(SortKey::default(), SortKey::Path);
    }

    #[test]
    fn test_sort_direction_default() {
        assert_eq!(SortDirection::default(), SortDirection::Ascending);
    }

    #[test]
    fn test_sort_config_default() {
        let config = SortConfig::default();
        assert_eq!(config.primary, SortKey::Path);
        assert_eq!(config.secondary, Some(SortKey::Line));
        assert_eq!(config.tertiary, Some(SortKey::Column));
        assert_eq!(config.direction, SortDirection::Ascending);
    }

    #[test]
    fn test_sort_config_builder() {
        let config = SortConfig::new(SortKey::Severity)
            .with_secondary(SortKey::Code)
            .with_tertiary(SortKey::Message)
            .descending();

        assert_eq!(config.primary, SortKey::Severity);
        assert_eq!(config.secondary, Some(SortKey::Code));
        assert_eq!(config.tertiary, Some(SortKey::Message));
        assert_eq!(config.direction, SortDirection::Descending);
    }

    #[test]
    fn test_sort_config_presets() {
        let severity_config = SortConfig::by_severity();
        assert_eq!(severity_config.primary, SortKey::Severity);
        assert_eq!(severity_config.secondary, Some(SortKey::Path));
        // Note: by_severity uses ascending because compare_by_key for Severity already reverses
        assert_eq!(severity_config.direction, SortDirection::Ascending);

        let path_config = SortConfig::by_path();
        assert_eq!(path_config.primary, SortKey::Path);
        assert_eq!(path_config.secondary, Some(SortKey::Line));
        assert_eq!(path_config.tertiary, Some(SortKey::Column));
        assert_eq!(path_config.direction, SortDirection::Ascending);

        let code_config = SortConfig::by_code();
        assert_eq!(code_config.primary, SortKey::Code);
        assert_eq!(code_config.secondary, Some(SortKey::Path));
    }

    #[test]
    fn test_compare_by_key_path() {
        let a = TestItem {
            path: "a.rs".into(),
            ..Default::default()
        };
        let b = TestItem {
            path: "b.rs".into(),
            ..Default::default()
        };
        assert_eq!(compare_by_key(&a, &b, SortKey::Path), Ordering::Less);
        assert_eq!(compare_by_key(&b, &a, SortKey::Path), Ordering::Greater);
        assert_eq!(
            compare_by_key(&a, &a, SortKey::Path),
            Ordering::Equal
        );
    }

    #[test]
    fn test_compare_by_key_severity() {
        let low = TestItem {
            severity: 1,
            ..Default::default()
        };
        let high = TestItem {
            severity: 3,
            ..Default::default()
        };
        // Higher severity comes first
        assert_eq!(
            compare_by_key(&low, &high, SortKey::Severity),
            Ordering::Greater
        );
        assert_eq!(
            compare_by_key(&high, &low, SortKey::Severity),
            Ordering::Less
        );
    }

    #[test]
    fn test_sort_slice() {
        let mut items = vec![
            TestItem {
                path: "c.rs".into(),
                ..Default::default()
            },
            TestItem {
                path: "a.rs".into(),
                ..Default::default()
            },
            TestItem {
                path: "b.rs".into(),
                ..Default::default()
            },
        ];

        sort_slice(&mut items, &SortConfig::by_path());
        assert_eq!(items[0].path, "a.rs");
        assert_eq!(items[1].path, "b.rs");
        assert_eq!(items[2].path, "c.rs");
    }

    #[test]
    fn test_natural_compare_simple() {
        assert_eq!(natural_compare("file2", "file10"), Ordering::Less);
        assert_eq!(natural_compare("file10", "file2"), Ordering::Greater);
        assert_eq!(natural_compare("file2", "file2"), Ordering::Equal);
    }

    #[test]
    fn test_natural_compare_with_letters() {
        assert_eq!(natural_compare("file2a", "file2b"), Ordering::Less);
        assert_eq!(natural_compare("file2b", "file2a"), Ordering::Greater);
    }

    #[test]
    fn test_natural_sort() {
        let mut strings = ["file10", "file2", "file1"];
        natural_sort(&mut strings);
        assert_eq!(strings, ["file1", "file2", "file10"]);
    }
}
