//! Source code URL generation for lintdiff.
//!
//! Generates clickable URLs to source code locations in various
//! Git hosting providers (GitHub, GitLab, Bitbucket, Azure DevOps).
//!
//! # Example
//!
//! ```
//! use lintdiff_code_url::{CodeUrlConfig, CodeLocation, CodeUrlBuilder, UrlProvider};
//!
//! // Create a config for a GitHub repository
//! let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
//!
//! // Generate a URL to a specific file and line
//! let url = config.file_url("src/lib.rs", Some(42), None);
//! assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L42");
//!
//! // Use the builder pattern for more control
//! let builder = CodeUrlBuilder::github("user", "repo", "main");
//! let location = CodeLocation::file("src/lib.rs").with_line(42);
//! let url = builder.url(&location);
//! assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L42");
//! ```


/// Configuration for generating source code URLs.
#[derive(Debug, Clone)]
pub struct CodeUrlConfig {
    /// Base URL for the repository (e.g., `<https://github.com/user/repo>`)
    pub base_url: String,
    /// Git reference (branch, tag, or commit SHA)
    pub git_ref: String,
    /// URL provider type (auto-detected if None)
    pub provider: Option<UrlProvider>,
}

impl CodeUrlConfig {
    /// Create a new config with the given base URL and git ref.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeUrlConfig;
    ///
    /// let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
    /// ```
    #[must_use]
    pub fn new(base_url: impl Into<String>, git_ref: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            git_ref: git_ref.into(),
            provider: None,
        }
    }

    /// Set the URL provider explicitly.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::{CodeUrlConfig, UrlProvider};
    ///
    /// let config = CodeUrlConfig::new("https://github.com/user/repo", "main")
    ///     .with_provider(UrlProvider::GitHub);
    /// ```
    #[must_use]
    pub const fn with_provider(mut self, provider: UrlProvider) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Auto-detect the provider from the base URL.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::{CodeUrlConfig, UrlProvider};
    ///
    /// let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
    /// assert_eq!(config.detect_provider(), UrlProvider::GitHub);
    ///
    /// let config = CodeUrlConfig::new("https://gitlab.com/user/repo", "main");
    /// assert_eq!(config.detect_provider(), UrlProvider::GitLab);
    /// ```
    #[must_use]
    pub fn detect_provider(&self) -> UrlProvider {
        self.provider.unwrap_or_else(|| UrlProvider::from_url(&self.base_url))
    }

    /// Generate a URL to a specific file and line.
    ///
    /// # Arguments
    /// * `path` - File path relative to repo root
    /// * `line` - Optional starting line number (1-based)
    /// * `end_line` - Optional ending line number for ranges
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeUrlConfig;
    ///
    /// let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
    /// let url = config.file_url("src/lib.rs", Some(42), None);
    /// assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L42");
    ///
    /// // With line range
    /// let url = config.file_url("src/lib.rs", Some(10), Some(20));
    /// assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L10-L20");
    /// ```
    #[must_use]
    pub fn file_url(&self, path: &str, line: Option<u32>, end_line: Option<u32>) -> String {
        let provider = self.detect_provider();
        let normalized_path = normalize_path(path);

        match provider {
            UrlProvider::GitHub => self.github_file_url(&normalized_path, line, end_line),
            UrlProvider::GitLab => self.gitlab_file_url(&normalized_path, line, end_line),
            UrlProvider::Bitbucket => self.bitbucket_file_url(&normalized_path, line, end_line),
            UrlProvider::AzureDevOps => self.azure_file_url(&normalized_path, line, end_line),
            UrlProvider::Generic => self.generic_file_url(&normalized_path, line, end_line),
        }
    }

    /// Generate a GitHub-style file URL.
    fn github_file_url(&self, path: &str, line: Option<u32>, end_line: Option<u32>) -> String {
        let base = format!("{}/blob/{}/{}", self.base_url, self.git_ref, path);
        add_line_fragment(&base, line, end_line)
    }

    /// Generate a GitLab-style file URL.
    fn gitlab_file_url(&self, path: &str, line: Option<u32>, end_line: Option<u32>) -> String {
        let base = format!("{}/-/blob/{}/{}", self.base_url, self.git_ref, path);
        add_line_fragment(&base, line, end_line)
    }

    /// Generate a Bitbucket-style file URL.
    fn bitbucket_file_url(&self, path: &str, line: Option<u32>, end_line: Option<u32>) -> String {
        let base = format!("{}/src/{}/{}", self.base_url, self.git_ref, path);
        match (line, end_line) {
            (Some(start), Some(end)) => format!("{base}#lines-{start}:{end}"),
            (Some(l), None) => format!("{base}#lines-{l}"),
            (None, _) => base,
        }
    }

    /// Generate an Azure DevOps-style file URL.
    fn azure_file_url(&self, path: &str, line: Option<u32>, end_line: Option<u32>) -> String {
        let mut url = format!(
            "{}?path={}&version={}",
            self.base_url,
            urlencoding(path),
            self.git_ref
        );
        if let Some(l) = line {
            url.push_str("&line=");
            url.push_str(&l.to_string());
            if let Some(end) = end_line {
                url.push_str("&lineEnd=");
                url.push_str(&end.to_string());
            }
        }
        url
    }

    /// Generate a generic file URL.
    fn generic_file_url(&self, path: &str, line: Option<u32>, end_line: Option<u32>) -> String {
        let base = format!("{}/blob/{}/{}", self.base_url, self.git_ref, path);
        add_line_fragment(&base, line, end_line)
    }

    /// Generate a URL to a specific commit.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeUrlConfig;
    ///
    /// let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
    /// let url = config.commit_url("abc123");
    /// assert_eq!(url, "https://github.com/user/repo/commit/abc123");
    /// ```
    #[must_use]
    pub fn commit_url(&self, sha: &str) -> String {
        let provider = self.detect_provider();

        match provider {
            UrlProvider::GitLab => format!("{}/-/commit/{}", self.base_url, sha),
            UrlProvider::Bitbucket => format!("{}/commits/{}", self.base_url, sha),
            UrlProvider::GitHub | UrlProvider::AzureDevOps | UrlProvider::Generic => {
                format!("{}/commit/{}", self.base_url, sha)
            }
        }
    }
}

impl Default for CodeUrlConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            git_ref: "main".to_string(),
            provider: None,
        }
    }
}

/// Supported Git hosting providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlProvider {
    /// GitHub (github.com or GitHub Enterprise)
    GitHub,
    /// GitLab (gitlab.com or self-hosted)
    GitLab,
    /// Bitbucket (bitbucket.org or self-hosted)
    Bitbucket,
    /// Azure DevOps
    AzureDevOps,
    /// Unknown provider (use generic format)
    Generic,
}

impl UrlProvider {
    /// Detect provider from a URL string.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::UrlProvider;
    ///
    /// assert_eq!(UrlProvider::from_url("https://github.com/user/repo"), UrlProvider::GitHub);
    /// assert_eq!(UrlProvider::from_url("https://gitlab.com/user/repo"), UrlProvider::GitLab);
    /// assert_eq!(UrlProvider::from_url("https://bitbucket.org/user/repo"), UrlProvider::Bitbucket);
    /// assert_eq!(UrlProvider::from_url("https://dev.azure.com/org/project/_git/repo"), UrlProvider::AzureDevOps);
    /// ```
    #[must_use]
    pub fn from_url(url: &str) -> Self {
        let url_lower = url.to_lowercase();

        if url_lower.contains("github.com") || url_lower.contains("github") {
            Self::GitHub
        } else if url_lower.contains("gitlab.com") || url_lower.contains("gitlab") {
            Self::GitLab
        } else if url_lower.contains("bitbucket.org") || url_lower.contains("bitbucket") {
            Self::Bitbucket
        } else if url_lower.contains("dev.azure.com") || url_lower.contains("visualstudio.com") {
            Self::AzureDevOps
        } else {
            Self::Generic
        }
    }
}

/// A source code location for URL generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeLocation {
    /// File path relative to repository root.
    pub path: String,
    /// Starting line number (1-based).
    pub line: Option<u32>,
    /// Ending line number for ranges (1-based).
    pub end_line: Option<u32>,
    /// Optional column number (1-based).
    pub column: Option<u32>,
}

impl CodeLocation {
    /// Create a new code location for a file.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeLocation;
    ///
    /// let location = CodeLocation::file("src/lib.rs");
    /// assert_eq!(location.path, "src/lib.rs");
    /// ```
    #[must_use]
    pub fn file(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            line: None,
            end_line: None,
            column: None,
        }
    }

    /// Add a line number.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeLocation;
    ///
    /// let location = CodeLocation::file("src/lib.rs").with_line(42);
    /// assert_eq!(location.line, Some(42));
    /// ```
    #[must_use]
    pub const fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    /// Add a line range.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeLocation;
    ///
    /// let location = CodeLocation::file("src/lib.rs").with_line_range(10, 20);
    /// assert_eq!(location.line, Some(10));
    /// assert_eq!(location.end_line, Some(20));
    /// ```
    #[must_use]
    pub const fn with_line_range(mut self, start: u32, end: u32) -> Self {
        self.line = Some(start);
        self.end_line = Some(end);
        self
    }

    /// Add a column number.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeLocation;
    ///
    /// let location = CodeLocation::file("src/lib.rs").with_line(42).with_column(5);
    /// assert_eq!(location.column, Some(5));
    /// ```
    #[must_use]
    pub const fn with_column(mut self, column: u32) -> Self {
        self.column = Some(column);
        self
    }

    /// Generate a URL using the given config.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::{CodeUrlConfig, CodeLocation};
    ///
    /// let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
    /// let location = CodeLocation::file("src/lib.rs").with_line(42);
    /// let url = location.to_url(&config);
    /// assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L42");
    /// ```
    #[must_use]
    pub fn to_url(&self, config: &CodeUrlConfig) -> String {
        config.file_url(&self.path, self.line, self.end_line)
    }
}

/// Builder for constructing code URLs.
#[derive(Debug, Clone)]
pub struct CodeUrlBuilder {
    config: CodeUrlConfig,
}

impl CodeUrlBuilder {
    /// Create a new builder with the given config.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::{CodeUrlBuilder, CodeUrlConfig};
    ///
    /// let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
    /// let builder = CodeUrlBuilder::new(config);
    /// ```
    #[must_use]
    pub const fn new(config: CodeUrlConfig) -> Self {
        Self { config }
    }

    /// Create a builder for GitHub.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeUrlBuilder;
    ///
    /// let builder = CodeUrlBuilder::github("user", "repo", "main");
    /// let url = builder.file_url("src/lib.rs");
    /// assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs");
    /// ```
    #[must_use]
    pub fn github(owner: &str, repo: &str, git_ref: &str) -> Self {
        Self {
            config: CodeUrlConfig::new(
                format!("https://github.com/{owner}/{repo}"),
                git_ref,
            )
            .with_provider(UrlProvider::GitHub),
        }
    }

    /// Create a builder for GitLab.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeUrlBuilder;
    ///
    /// let builder = CodeUrlBuilder::gitlab("user", "repo", "main");
    /// let url = builder.file_url("src/lib.rs");
    /// assert_eq!(url, "https://gitlab.com/user/repo/-/blob/main/src/lib.rs");
    /// ```
    #[must_use]
    pub fn gitlab(owner: &str, repo: &str, git_ref: &str) -> Self {
        Self {
            config: CodeUrlConfig::new(
                format!("https://gitlab.com/{owner}/{repo}"),
                git_ref,
            )
            .with_provider(UrlProvider::GitLab),
        }
    }

    /// Create a builder for Bitbucket.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeUrlBuilder;
    ///
    /// let builder = CodeUrlBuilder::bitbucket("user", "repo", "main");
    /// let url = builder.file_url("src/lib.rs");
    /// assert_eq!(url, "https://bitbucket.org/user/repo/src/main/src/lib.rs");
    /// ```
    #[must_use]
    pub fn bitbucket(owner: &str, repo: &str, git_ref: &str) -> Self {
        Self {
            config: CodeUrlConfig::new(
                format!("https://bitbucket.org/{owner}/{repo}"),
                git_ref,
            )
            .with_provider(UrlProvider::Bitbucket),
        }
    }

    /// Create a builder for Azure DevOps.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeUrlBuilder;
    ///
    /// let builder = CodeUrlBuilder::azure_devops("org", "project", "repo", "main");
    /// let url = builder.file_url("src/lib.rs");
    /// assert_eq!(url, "https://dev.azure.com/org/project/_git/repo?path=src/lib.rs&version=main");
    /// ```
    #[must_use]
    pub fn azure_devops(org: &str, project: &str, repo: &str, git_ref: &str) -> Self {
        Self {
            config: CodeUrlConfig::new(
                format!("https://dev.azure.com/{org}/{project}/_git/{repo}"),
                git_ref,
            )
            .with_provider(UrlProvider::AzureDevOps),
        }
    }

    /// Generate a URL for the given location.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::{CodeUrlBuilder, CodeLocation};
    ///
    /// let builder = CodeUrlBuilder::github("user", "repo", "main");
    /// let location = CodeLocation::file("src/lib.rs").with_line(42);
    /// let url = builder.url(&location);
    /// assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L42");
    /// ```
    #[must_use]
    pub fn url(&self, location: &CodeLocation) -> String {
        location.to_url(&self.config)
    }

    /// Generate a URL for a file.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeUrlBuilder;
    ///
    /// let builder = CodeUrlBuilder::github("user", "repo", "main");
    /// let url = builder.file_url("src/lib.rs");
    /// assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs");
    /// ```
    #[must_use]
    pub fn file_url(&self, path: &str) -> String {
        self.config.file_url(path, None, None)
    }

    /// Generate a URL for a file at a specific line.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeUrlBuilder;
    ///
    /// let builder = CodeUrlBuilder::github("user", "repo", "main");
    /// let url = builder.line_url("src/lib.rs", 42);
    /// assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L42");
    /// ```
    #[must_use]
    pub fn line_url(&self, path: &str, line: u32) -> String {
        self.config.file_url(path, Some(line), None)
    }

    /// Generate a URL for a file at a line range.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeUrlBuilder;
    ///
    /// let builder = CodeUrlBuilder::github("user", "repo", "main");
    /// let url = builder.range_url("src/lib.rs", 10, 20);
    /// assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L10-L20");
    /// ```
    #[must_use]
    pub fn range_url(&self, path: &str, start: u32, end: u32) -> String {
        self.config.file_url(path, Some(start), Some(end))
    }

    /// Generate a commit URL.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_code_url::CodeUrlBuilder;
    ///
    /// let builder = CodeUrlBuilder::github("user", "repo", "main");
    /// let url = builder.commit_url("abc123def456");
    /// assert_eq!(url, "https://github.com/user/repo/commit/abc123def456");
    /// ```
    #[must_use]
    pub fn commit_url(&self, sha: &str) -> String {
        self.config.commit_url(sha)
    }
}

/// Normalize a file path for URL usage.
/// Converts backslashes to forward slashes and removes leading slashes.
fn normalize_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    normalized.trim_start_matches('/').to_string()
}

/// Add line number fragment to a URL.
fn add_line_fragment(base: &str, line: Option<u32>, end_line: Option<u32>) -> String {
    match (line, end_line) {
        (Some(start), Some(end)) => format!("{base}#L{start}-L{end}"),
        (Some(l), None) => format!("{base}#L{l}"),
        (None, _) => base.to_string(),
    }
}

/// URL-encode a string for use in query parameters.
/// Note: Forward slashes are not encoded as they are valid in Azure DevOps paths.
fn urlencoding(s: &str) -> String {
    // Simple URL encoding for common characters
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ' ' => result.push_str("%20"),
            '#' => result.push_str("%23"),
            '%' => result.push_str("%25"),
            '&' => result.push_str("%26"),
            '+' => result.push_str("%2B"),
            '=' => result.push_str("%3D"),
            '?' => result.push_str("%3F"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("src/lib.rs"), "src/lib.rs");
        assert_eq!(normalize_path("src\\lib.rs"), "src/lib.rs");
        assert_eq!(normalize_path("/src/lib.rs"), "src/lib.rs");
        assert_eq!(normalize_path("\\src\\lib.rs"), "src/lib.rs");
    }

    #[test]
    fn test_urlencoding() {
        // Forward slashes are not encoded (valid in Azure DevOps paths)
        assert_eq!(urlencoding("src/lib.rs"), "src/lib.rs");
        assert_eq!(urlencoding("file name.rs"), "file%20name.rs");
        assert_eq!(urlencoding("test#file.rs"), "test%23file.rs");
    }

    #[test]
    fn test_add_line_fragment() {
        assert_eq!(add_line_fragment("http://example.com", None, None), "http://example.com");
        assert_eq!(add_line_fragment("http://example.com", Some(42), None), "http://example.com#L42");
        assert_eq!(add_line_fragment("http://example.com", Some(10), Some(20)), "http://example.com#L10-L20");
    }
}
