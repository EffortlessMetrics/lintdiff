//! Git information types for lintdiff.
//!
//! Provides types for representing Git commit hashes, references, and
//! repository information used in CI/CD pipelines and reports.
//!
//! # Example
//!
//! ```
//! use lintdiff_git_info::{GitSha, GitRef, GitInfo, parse_sha, parse_ref};
//!
//! // Parse a SHA
//! let sha = parse_sha("0123456789abcdef0123456789abcdef01234567").unwrap();
//! assert_eq!(sha.short(7), "0123456");
//! assert!(!sha.is_zero());
//!
//! // Parse a reference
//! let git_ref = parse_ref("refs/heads/main");
//! assert_eq!(git_ref.as_branch(), Some("main"));
//!
//! // Build GitInfo
//! let info = GitInfo::new(sha)
//!     .with_ref_name("main".to_string())
//!     .with_dirty(false);
//! assert_eq!(info.ref_name, Some("main".to_string()));
//! ```

use std::fmt;
use std::str::FromStr;

/// SHA-1 hash length in characters.
const SHA_LENGTH: usize = 40;

/// Zero SHA constant.
const ZERO_SHA: &str = "0000000000000000000000000000000000000000";

/// Error type for Git information parsing failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GitInfoError {
    /// The SHA string is not valid (wrong length or non-hex characters).
    #[error("Invalid SHA: {0}")]
    InvalidSha(String),

    /// The SHA string has an invalid length.
    #[error("Invalid SHA length: expected {expected}, got {actual}")]
    InvalidShaLength {
        /// Expected length.
        expected: usize,
        /// Actual length.
        actual: usize,
    },

    /// The SHA string contains invalid hex characters.
    #[error("Invalid hex character in SHA at position {position}: '{character}'")]
    InvalidHexCharacter {
        /// Position of the invalid character.
        position: usize,
        /// The invalid character.
        character: char,
    },
}

/// A Git SHA-1 hash wrapper with validation and utilities.
///
/// # Examples
///
/// ```
/// use lintdiff_git_info::GitSha;
///
/// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
/// assert_eq!(sha.as_str(), "0123456789abcdef0123456789abcdef01234567");
/// assert_eq!(sha.short(7), "0123456");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GitSha(String);

impl GitSha {
    /// Create a new `GitSha` from a string, validating the format.
    ///
    /// # Errors
    ///
    /// Returns `GitInfoError::InvalidSha` if the string is not a valid 40-character hex string.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitSha;
    ///
    /// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
    /// assert_eq!(sha.as_str(), "0123456789abcdef0123456789abcdef01234567");
    ///
    /// // Invalid SHA (too short)
    /// assert!(GitSha::new("abc").is_err());
    /// ```
    pub fn new(s: &str) -> Result<Self, GitInfoError> {
        validate_sha(s)?;
        Ok(Self(s.to_lowercase()))
    }

    /// Create a `GitSha` from raw 20 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitSha;
    ///
    /// let bytes = [0u8; 20];
    /// let sha = GitSha::from_bytes(bytes);
    /// assert!(sha.is_zero());
    /// ```
    #[must_use]
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(hex::encode(bytes))
    }

    /// Get the SHA as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitSha;
    ///
    /// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
    /// assert_eq!(sha.as_str(), "0123456789abcdef0123456789abcdef01234567");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get a short SHA of the specified length.
    ///
    /// If `len` is greater than 40, returns the full SHA.
    /// If `len` is 0, returns an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitSha;
    ///
    /// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
    /// assert_eq!(sha.short(7), "0123456");
    /// assert_eq!(sha.short(40), "0123456789abcdef0123456789abcdef01234567");
    /// assert_eq!(sha.short(50), "0123456789abcdef0123456789abcdef01234567");
    /// assert_eq!(sha.short(0), "");
    /// ```
    #[must_use]
    pub fn short(&self, len: usize) -> String {
        if len == 0 {
            return String::new();
        }
        if len >= SHA_LENGTH {
            return self.0.clone();
        }
        self.0[..len].to_string()
    }

    /// Check if this SHA is all zeros (null SHA).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitSha;
    ///
    /// let zero = GitSha::new("0000000000000000000000000000000000000000").unwrap();
    /// assert!(zero.is_zero());
    ///
    /// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
    /// assert!(!sha.is_zero());
    /// ```
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == ZERO_SHA
    }

    /// Get the default short SHA (7 characters).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitSha;
    ///
    /// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
    /// assert_eq!(sha.short_default(), "0123456");
    /// ```
    #[must_use]
    pub fn short_default(&self) -> String {
        self.short(7)
    }

    /// Create a zero SHA.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitSha;
    ///
    /// let zero = GitSha::zero();
    /// assert!(zero.is_zero());
    /// ```
    #[must_use]
    pub fn zero() -> Self {
        Self(ZERO_SHA.to_string())
    }
}

impl fmt::Display for GitSha {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for GitSha {
    type Err = GitInfoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl AsRef<str> for GitSha {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Default for GitSha {
    fn default() -> Self {
        Self::zero()
    }
}

/// A Git reference (branch, tag, commit, or HEAD).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GitRef {
    /// A branch reference (e.g., "main", "develop").
    Branch(String),
    /// A tag reference (e.g., "v1.0.0").
    Tag(String),
    /// A direct commit reference.
    Commit(GitSha),
    /// The HEAD reference.
    Head,
    /// An unknown or unrecognized reference.
    #[default]
    Unknown,
}

impl GitRef {
    /// Try to get the branch name if this is a branch reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitRef;
    ///
    /// let git_ref = GitRef::Branch("main".to_string());
    /// assert_eq!(git_ref.as_branch(), Some("main"));
    ///
    /// let tag = GitRef::Tag("v1.0.0".to_string());
    /// assert_eq!(tag.as_branch(), None);
    /// ```
    #[must_use]
    pub fn as_branch(&self) -> Option<&str> {
        match self {
            Self::Branch(name) => Some(name),
            _ => None,
        }
    }

    /// Try to get the tag name if this is a tag reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitRef;
    ///
    /// let git_ref = GitRef::Tag("v1.0.0".to_string());
    /// assert_eq!(git_ref.as_tag(), Some("v1.0.0"));
    ///
    /// let branch = GitRef::Branch("main".to_string());
    /// assert_eq!(branch.as_tag(), None);
    /// ```
    #[must_use]
    pub fn as_tag(&self) -> Option<&str> {
        match self {
            Self::Tag(name) => Some(name),
            _ => None,
        }
    }

    /// Try to get the commit SHA if this is a commit reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::{GitSha, GitRef};
    ///
    /// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
    /// let git_ref = GitRef::Commit(sha.clone());
    /// assert_eq!(git_ref.as_commit(), Some(&sha));
    ///
    /// let branch = GitRef::Branch("main".to_string());
    /// assert_eq!(branch.as_commit(), None);
    /// ```
    #[must_use]
    pub const fn as_commit(&self) -> Option<&GitSha> {
        match self {
            Self::Commit(sha) => Some(sha),
            _ => None,
        }
    }

    /// Check if this is a branch reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitRef;
    ///
    /// assert!(GitRef::Branch("main".to_string()).is_branch());
    /// assert!(!GitRef::Tag("v1.0.0".to_string()).is_branch());
    /// ```
    #[must_use]
    pub const fn is_branch(&self) -> bool {
        matches!(self, Self::Branch(_))
    }

    /// Check if this is a tag reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitRef;
    ///
    /// assert!(GitRef::Tag("v1.0.0".to_string()).is_tag());
    /// assert!(!GitRef::Branch("main".to_string()).is_tag());
    /// ```
    #[must_use]
    pub const fn is_tag(&self) -> bool {
        matches!(self, Self::Tag(_))
    }

    /// Check if this is a HEAD reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitRef;
    ///
    /// assert!(GitRef::Head.is_head());
    /// assert!(!GitRef::Branch("main".to_string()).is_head());
    /// ```
    #[must_use]
    pub const fn is_head(&self) -> bool {
        matches!(self, Self::Head)
    }

    /// Get a human-readable name for this reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::{GitRef, GitSha};
    ///
    /// assert_eq!(GitRef::Branch("main".to_string()).name(), Some("main"));
    /// assert_eq!(GitRef::Tag("v1.0.0".to_string()).name(), Some("v1.0.0"));
    /// assert_eq!(GitRef::Head.name(), Some("HEAD"));
    /// assert_eq!(GitRef::Unknown.name(), None);
    /// ```
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Branch(name) | Self::Tag(name) => Some(name),
            Self::Commit(sha) => Some(sha.as_str()),
            Self::Head => Some("HEAD"),
            Self::Unknown => None,
        }
    }
}

impl fmt::Display for GitRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Branch(name) => write!(f, "refs/heads/{name}"),
            Self::Tag(name) => write!(f, "refs/tags/{name}"),
            Self::Commit(sha) => write!(f, "{sha}"),
            Self::Head => write!(f, "HEAD"),
            Self::Unknown => write!(f, "(unknown)"),
        }
    }
}

/// Combined Git information for a repository state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GitInfo {
    /// Current commit SHA.
    pub sha: GitSha,
    /// Branch or tag name (if available).
    pub ref_name: Option<String>,
    /// Whether there are uncommitted changes.
    pub is_dirty: bool,
    /// First line of the commit message.
    pub message: Option<String>,
}

impl GitInfo {
    /// Create a new `GitInfo` with the given SHA and default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::{GitSha, GitInfo};
    ///
    /// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
    /// let info = GitInfo::new(sha.clone());
    /// assert_eq!(info.sha, sha);
    /// assert_eq!(info.ref_name, None);
    /// assert!(!info.is_dirty);
    /// ```
    #[must_use]
    pub const fn new(sha: GitSha) -> Self {
        Self {
            sha,
            ref_name: None,
            is_dirty: false,
            message: None,
        }
    }

    /// Set the reference name.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::{GitSha, GitInfo};
    ///
    /// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
    /// let info = GitInfo::new(sha).with_ref_name("main".to_string());
    /// assert_eq!(info.ref_name, Some("main".to_string()));
    /// ```
    #[must_use]
    pub fn with_ref_name(mut self, name: String) -> Self {
        self.ref_name = Some(name);
        self
    }

    /// Set the dirty flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::{GitSha, GitInfo};
    ///
    /// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
    /// let info = GitInfo::new(sha).with_dirty(true);
    /// assert!(info.is_dirty);
    /// ```
    #[must_use]
    pub const fn with_dirty(mut self, dirty: bool) -> Self {
        self.is_dirty = dirty;
        self
    }

    /// Set the commit message.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::{GitSha, GitInfo};
    ///
    /// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
    /// let info = GitInfo::new(sha).with_message("Initial commit".to_string());
    /// assert_eq!(info.message, Some("Initial commit".to_string()));
    /// ```
    #[must_use]
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Check if this represents a clean working directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::{GitSha, GitInfo};
    ///
    /// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
    /// let clean_info = GitInfo::new(sha.clone()).with_dirty(false);
    /// assert!(clean_info.is_clean());
    ///
    /// let dirty_info = GitInfo::new(sha).with_dirty(true);
    /// assert!(!dirty_info.is_clean());
    /// ```
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        !self.is_dirty
    }

    /// Get the short SHA (7 characters).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::{GitSha, GitInfo};
    ///
    /// let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
    /// let info = GitInfo::new(sha);
    /// assert_eq!(info.short_sha(), "0123456");
    /// ```
    #[must_use]
    pub fn short_sha(&self) -> String {
        self.sha.short_default()
    }

    /// Create a `GitInfo` with a zero SHA (for uninitialized state).
    ///
    /// # Examples
    ///
    /// ```
    /// use lintdiff_git_info::GitInfo;
    ///
    /// let info = GitInfo::empty();
    /// assert!(info.sha.is_zero());
    /// ```
    #[must_use]
    pub fn empty() -> Self {
        Self {
            sha: GitSha::zero(),
            ref_name: None,
            is_dirty: false,
            message: None,
        }
    }
}

impl Default for GitInfo {
    fn default() -> Self {
        Self::empty()
    }
}

/// Parse a SHA string into a `GitSha`.
///
/// # Errors
///
/// Returns `GitInfoError` if the string is not a valid 40-character hex string.
///
/// # Examples
///
/// ```
/// use lintdiff_git_info::parse_sha;
///
/// let sha = parse_sha("0123456789abcdef0123456789abcdef01234567").unwrap();
/// assert_eq!(sha.as_str(), "0123456789abcdef0123456789abcdef01234567");
///
/// assert!(parse_sha("invalid").is_err());
/// ```
pub fn parse_sha(s: &str) -> Result<GitSha, GitInfoError> {
    GitSha::new(s)
}

/// Parse a Git reference string into a `GitRef`.
///
/// Recognizes:
/// - `refs/heads/<name>` -> Branch
/// - `refs/tags/<name>` -> Tag
/// - `HEAD` -> Head
/// - 40-character hex string -> Commit
/// - Anything else -> Unknown
///
/// # Examples
///
/// ```
/// use lintdiff_git_info::{parse_ref, GitRef};
///
/// assert!(matches!(parse_ref("refs/heads/main"), GitRef::Branch(_)));
/// assert!(matches!(parse_ref("refs/tags/v1.0.0"), GitRef::Tag(_)));
/// assert_eq!(parse_ref("HEAD"), GitRef::Head);
/// assert_eq!(parse_ref("unknown"), GitRef::Unknown);
/// ```
#[must_use]
pub fn parse_ref(s: &str) -> GitRef {
    let trimmed = s.trim();

    if trimmed == "HEAD" {
        return GitRef::Head;
    }

    if let Some(name) = trimmed.strip_prefix("refs/heads/") {
        return GitRef::Branch(name.to_string());
    }

    if let Some(name) = trimmed.strip_prefix("refs/tags/") {
        return GitRef::Tag(name.to_string());
    }

    // Try to parse as a commit SHA
    if is_valid_sha(trimmed) {
        if let Ok(sha) = GitSha::new(trimmed) {
            return GitRef::Commit(sha);
        }
    }

    GitRef::Unknown
}

/// Check if a string is a valid SHA format (40 hex characters).
///
/// # Examples
///
/// ```
/// use lintdiff_git_info::is_valid_sha;
///
/// assert!(is_valid_sha("0123456789abcdef0123456789abcdef01234567"));
/// assert!(is_valid_sha("0000000000000000000000000000000000000000"));
/// assert!(!is_valid_sha("invalid"));
/// assert!(!is_valid_sha("0123456789abcdef"));  // Too short
/// assert!(!is_valid_sha(""));
/// ```
#[must_use]
pub fn is_valid_sha(s: &str) -> bool {
    validate_sha(s).is_ok()
}

/// Validate a SHA string.
fn validate_sha(s: &str) -> Result<(), GitInfoError> {
    if s.len() != SHA_LENGTH {
        return Err(GitInfoError::InvalidShaLength {
            expected: SHA_LENGTH,
            actual: s.len(),
        });
    }

    for (i, c) in s.chars().enumerate() {
        if !c.is_ascii_hexdigit() {
            return Err(GitInfoError::InvalidHexCharacter {
                position: i,
                character: c,
            });
        }
    }

    Ok(())
}

// Internal hex encoding to avoid dependency
mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    #[allow(unreachable_pub)]
    pub fn encode(bytes: [u8; 20]) -> String {
        let mut result = String::with_capacity(40);
        for byte in bytes {
            result.push(HEX_CHARS[(byte >> 4) as usize] as char);
            result.push(HEX_CHARS[(byte & 0x0f) as usize] as char);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_sha_new_valid() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert_eq!(sha.as_str(), "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn test_git_sha_new_uppercase() {
        let sha = GitSha::new("0123456789ABCDEF0123456789ABCDEF01234567").unwrap();
        assert_eq!(sha.as_str(), "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn test_git_sha_new_invalid_length() {
        let result = GitSha::new("abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_sha_new_invalid_hex() {
        let result = GitSha::new("0123456789ghijef0123456789abcdef01234567");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_sha_is_zero() {
        let zero = GitSha::new("0000000000000000000000000000000000000000").unwrap();
        assert!(zero.is_zero());

        let non_zero = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert!(!non_zero.is_zero());
    }

    #[test]
    fn test_git_sha_short() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert_eq!(sha.short(7), "0123456");
        assert_eq!(sha.short(0), "");
        assert_eq!(sha.short(40), "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn test_parse_ref_branch() {
        let git_ref = parse_ref("refs/heads/main");
        assert_eq!(git_ref.as_branch(), Some("main"));
    }

    #[test]
    fn test_parse_ref_tag() {
        let git_ref = parse_ref("refs/tags/v1.0.0");
        assert_eq!(git_ref.as_tag(), Some("v1.0.0"));
    }

    #[test]
    fn test_parse_ref_head() {
        assert_eq!(parse_ref("HEAD"), GitRef::Head);
    }

    #[test]
    fn test_is_valid_sha() {
        assert!(is_valid_sha("0123456789abcdef0123456789abcdef01234567"));
        assert!(!is_valid_sha("invalid"));
        assert!(!is_valid_sha(""));
    }
}
