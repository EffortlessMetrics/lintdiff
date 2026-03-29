//! CI environment detection for lintdiff.
//!
//! Provides types and functions for detecting which CI system
//! is running and extracting relevant environment information.
//!
//! # Example
//!
//! ```
//! use lintdiff_ci_env::{CiEnvironment, CiPlatform};
//!
//! let env = CiEnvironment::detect();
//! println!("Running on: {:?}", env.platform);
//! if env.is_ci() {
//!     if let Some(repo) = &env.repository {
//!         println!("Repository: {}", repo.slug);
//!     }
//! }
//! ```

use std::env;
use std::fs;

/// Supported CI platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CiPlatform {
    /// GitHub Actions
    GitHubActions,
    /// GitLab CI
    GitLabCi,
    /// `CircleCI`
    CircleCi,
    /// Travis CI
    TravisCi,
    /// Azure Pipelines
    AzurePipelines,
    /// Jenkins
    Jenkins,
    /// Bitbucket Pipelines
    BitbucketPipelines,
    /// `AppVeyor`
    AppVeyor,
    /// Drone CI
    DroneCi,
    /// `TeamCity`
    TeamCity,
    /// Unknown/not in CI
    Unknown,
}

impl CiPlatform {
    /// Detect the current CI platform from environment variables.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_ci_env::CiPlatform;
    ///
    /// let platform = CiPlatform::detect();
    /// println!("Running on: {:?}", platform);
    /// ```
    #[must_use]
    pub fn detect() -> Self {
        // Check for GitHub Actions
        if env::var("GITHUB_ACTIONS").is_ok_and(|v| v == "true") {
            return Self::GitHubActions;
        }

        // Check for GitLab CI
        if env::var("GITLAB_CI").is_ok_and(|v| v == "true") {
            return Self::GitLabCi;
        }

        // Check for CircleCI
        if env::var("CIRCLECI").is_ok_and(|v| v == "true") {
            return Self::CircleCi;
        }

        // Check for Travis CI
        if env::var("TRAVIS").is_ok_and(|v| v == "true") {
            return Self::TravisCi;
        }

        // Check for Azure Pipelines
        if env::var("TF_BUILD").is_ok_and(|v| v == "True") {
            return Self::AzurePipelines;
        }

        // Check for Jenkins
        if env::var("JENKINS_URL").is_ok() {
            return Self::Jenkins;
        }

        // Check for Bitbucket Pipelines
        if env::var("BITBUCKET_BUILD_NUMBER").is_ok() {
            return Self::BitbucketPipelines;
        }

        // Check for AppVeyor
        if env::var("APPVEYOR").is_ok_and(|v| v == "true") {
            return Self::AppVeyor;
        }

        // Check for Drone CI
        if env::var("DRONE").is_ok_and(|v| v == "true") {
            return Self::DroneCi;
        }

        // Check for TeamCity
        if env::var("TEAMCITY_VERSION").is_ok() {
            return Self::TeamCity;
        }

        Self::Unknown
    }

    /// Check if currently running in any CI environment.
    #[must_use]
    pub fn is_ci() -> bool {
        Self::detect() != Self::Unknown
    }

    /// Get a human-readable name for this platform.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::GitHubActions => "GitHub Actions",
            Self::GitLabCi => "GitLab CI",
            Self::CircleCi => "CircleCI",
            Self::TravisCi => "Travis CI",
            Self::AzurePipelines => "Azure Pipelines",
            Self::Jenkins => "Jenkins",
            Self::BitbucketPipelines => "Bitbucket Pipelines",
            Self::AppVeyor => "AppVeyor",
            Self::DroneCi => "Drone CI",
            Self::TeamCity => "TeamCity",
            Self::Unknown => "Unknown",
        }
    }

    /// Check if this platform supports GitHub-style annotations.
    #[must_use]
    pub const fn supports_github_annotations(self) -> bool {
        matches!(self, Self::GitHubActions)
    }

    /// Check if this platform supports markdown in output.
    #[must_use]
    pub const fn supports_markdown(self) -> bool {
        matches!(self, Self::GitHubActions | Self::GitLabCi)
    }
}

impl Default for CiPlatform {
    fn default() -> Self {
        Self::detect()
    }
}

/// Information about a pull request.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PullRequestInfo {
    /// PR number.
    pub number: u64,
    /// Source branch name.
    pub source_branch: String,
    /// Target branch name.
    pub target_branch: String,
}

impl PullRequestInfo {
    /// Try to detect PR info from the environment.
    #[must_use]
    pub fn detect() -> Option<Self> {
        let platform = CiPlatform::detect();
        match platform {
            CiPlatform::GitHubActions => Self::detect_github_actions(),
            CiPlatform::GitLabCi => Self::detect_gitlab_ci(),
            CiPlatform::CircleCi => Self::detect_circleci(),
            CiPlatform::TravisCi => Self::detect_travis_ci(),
            CiPlatform::AzurePipelines => Self::detect_azure_pipelines(),
            CiPlatform::Jenkins => Self::detect_jenkins(),
            CiPlatform::BitbucketPipelines => Self::detect_bitbucket(),
            CiPlatform::AppVeyor => Self::detect_appveyor(),
            CiPlatform::DroneCi => Self::detect_drone(),
            CiPlatform::TeamCity | CiPlatform::Unknown => None,
        }
    }

    fn detect_github_actions() -> Option<Self> {
        let event_path = env::var("GITHUB_EVENT_PATH").ok()?;
        let event_content = fs::read_to_string(&event_path).ok()?;
        let event: serde_json::Value = serde_json::from_str(&event_content).ok()?;

        let pr = event.get("pull_request")?;
        let number = pr.get("number")?.as_u64()?;

        let source_branch = pr
            .get("head")
            .and_then(|h| h.get("ref"))
            .and_then(|r| r.as_str())
            .unwrap_or("")
            .to_string();

        let target_branch = pr
            .get("base")
            .and_then(|b| b.get("ref"))
            .and_then(|r| r.as_str())
            .unwrap_or("")
            .to_string();

        Some(Self {
            number,
            source_branch,
            target_branch,
        })
    }

    fn detect_gitlab_ci() -> Option<Self> {
        let number = env::var("CI_MERGE_REQUEST_IID")
            .ok()
            .and_then(|s| s.parse().ok())?;

        let source_branch = env::var("CI_MERGE_REQUEST_SOURCE_BRANCH_NAME")
            .unwrap_or_else(|_| env::var("CI_COMMIT_REF_NAME").unwrap_or_default());

        let target_branch = env::var("CI_MERGE_REQUEST_TARGET_BRANCH_NAME").unwrap_or_else(|_| {
            env::var("CI_DEFAULT_BRANCH").unwrap_or_else(|_| "main".to_string())
        });

        Some(Self {
            number,
            source_branch,
            target_branch,
        })
    }

    fn detect_circleci() -> Option<Self> {
        let pr_url = env::var("CIRCLE_PULL_REQUEST").ok()?;
        let number = pr_url.rsplit('/').next()?.parse().ok()?;

        let source_branch = env::var("CIRCLE_BRANCH").unwrap_or_default();
        // CircleCI doesn't provide target branch directly
        let target_branch = "main".to_string();

        Some(Self {
            number,
            source_branch,
            target_branch,
        })
    }

    fn detect_travis_ci() -> Option<Self> {
        let number = env::var("TRAVIS_PULL_REQUEST").ok().and_then(|s| {
            if s == "false" {
                None
            } else {
                s.parse().ok()
            }
        })?;

        let source_branch = env::var("TRAVIS_PULL_REQUEST_BRANCH").unwrap_or_default();
        let target_branch = env::var("TRAVIS_BRANCH").unwrap_or_default();

        Some(Self {
            number,
            source_branch,
            target_branch,
        })
    }

    fn detect_azure_pipelines() -> Option<Self> {
        let number = env::var("SYSTEM_PULLREQUEST_PULLREQUESTNUMBER")
            .or_else(|_| env::var("SYSTEM_PULLREQUEST_PULLREQUESTID"))
            .ok()
            .and_then(|s| s.parse().ok())?;

        let source_branch = env::var("BUILD_SOURCEBRANCHNAME").unwrap_or_default();
        let target_branch =
            env::var("SYSTEM_PULLREQUEST_TARGETBRANCH").unwrap_or_else(|_| "main".to_string());

        Some(Self {
            number,
            source_branch,
            target_branch,
        })
    }

    fn detect_jenkins() -> Option<Self> {
        let number = env::var("CHANGE_ID").ok().and_then(|s| s.parse().ok())?;

        let source_branch = env::var("CHANGE_BRANCH").unwrap_or_default();
        let target_branch = env::var("CHANGE_TARGET").unwrap_or_default();

        Some(Self {
            number,
            source_branch,
            target_branch,
        })
    }

    fn detect_bitbucket() -> Option<Self> {
        let number = env::var("BITBUCKET_PR_ID")
            .ok()
            .and_then(|s| s.parse().ok())?;

        let source_branch = env::var("BITBUCKET_BRANCH").unwrap_or_default();
        // Bitbucket doesn't provide target branch directly in PR builds
        let target_branch = "main".to_string();

        Some(Self {
            number,
            source_branch,
            target_branch,
        })
    }

    fn detect_appveyor() -> Option<Self> {
        let number = env::var("APPVEYOR_PULL_REQUEST_NUMBER")
            .ok()
            .and_then(|s| s.parse().ok())?;

        let source_branch = env::var("APPVEYOR_REPO_BRANCH").unwrap_or_default();
        // AppVeyor doesn't provide target branch directly
        let target_branch = "main".to_string();

        Some(Self {
            number,
            source_branch,
            target_branch,
        })
    }

    fn detect_drone() -> Option<Self> {
        let number = env::var("DRONE_PULL_REQUEST")
            .ok()
            .and_then(|s| s.parse().ok())?;

        let source_branch = env::var("DRONE_BRANCH").unwrap_or_default();
        // Drone doesn't provide target branch directly
        let target_branch = "main".to_string();

        Some(Self {
            number,
            source_branch,
            target_branch,
        })
    }
}

/// Information about the current repository.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RepositoryInfo {
    /// Repository owner/organization.
    pub owner: String,
    /// Repository name.
    pub name: String,
    /// Full repository slug (owner/name).
    pub slug: String,
}

impl RepositoryInfo {
    /// Try to detect repository info from the environment.
    #[must_use]
    pub fn detect() -> Option<Self> {
        let platform = CiPlatform::detect();
        match platform {
            CiPlatform::GitHubActions => Self::detect_github_actions(),
            CiPlatform::GitLabCi => Self::detect_gitlab_ci(),
            CiPlatform::CircleCi => Self::detect_circleci(),
            CiPlatform::TravisCi => Self::detect_travis_ci(),
            CiPlatform::AzurePipelines => Self::detect_azure_pipelines(),
            CiPlatform::Jenkins => Self::detect_jenkins(),
            CiPlatform::BitbucketPipelines => Self::detect_bitbucket(),
            CiPlatform::AppVeyor => Self::detect_appveyor(),
            CiPlatform::DroneCi => Self::detect_drone(),
            CiPlatform::TeamCity => Self::detect_teamcity(),
            CiPlatform::Unknown => None,
        }
    }

    /// Create from owner and name.
    #[must_use]
    pub fn new(owner: impl Into<String>, name: impl Into<String>) -> Self {
        let owner = owner.into();
        let name = name.into();
        let slug = format!("{owner}/{name}");
        Self { owner, name, slug }
    }

    /// Create from a slug (owner/name).
    #[must_use]
    pub fn from_slug(slug: impl Into<String>) -> Option<Self> {
        let slug = slug.into();
        let parts: Vec<&str> = slug.split('/').collect();
        if parts.len() == 2 {
            Some(Self {
                owner: parts[0].to_string(),
                name: parts[1].to_string(),
                slug,
            })
        } else {
            None
        }
    }

    fn detect_github_actions() -> Option<Self> {
        let slug = env::var("GITHUB_REPOSITORY").ok()?;
        Self::from_slug(slug)
    }

    fn detect_gitlab_ci() -> Option<Self> {
        let owner = env::var("CI_PROJECT_NAMESPACE").ok()?;
        let name = env::var("CI_PROJECT_NAME").ok()?;
        Some(Self::new(owner, name))
    }

    fn detect_circleci() -> Option<Self> {
        let owner = env::var("CIRCLE_PROJECT_USERNAME").ok()?;
        let name = env::var("CIRCLE_PROJECT_REPONAME").ok()?;
        Some(Self::new(owner, name))
    }

    fn detect_travis_ci() -> Option<Self> {
        let slug = env::var("TRAVIS_REPO_SLUG").ok()?;
        Self::from_slug(slug)
    }

    fn detect_azure_pipelines() -> Option<Self> {
        let slug = env::var("BUILD_REPOSITORY_NAME").ok()?;
        Self::from_slug(slug)
    }

    fn detect_jenkins() -> Option<Self> {
        // Jenkins JOB_NAME can be in various formats
        let job_name = env::var("JOB_NAME").ok()?;
        // Try to parse as org/repo format
        if job_name.contains('/') {
            let parts: Vec<&str> = job_name.split('/').collect();
            if parts.len() >= 2 {
                return Some(Self::new(parts[0], parts[1]));
            }
        }
        None
    }

    fn detect_bitbucket() -> Option<Self> {
        let slug = env::var("BITBUCKET_REPO_FULL_NAME").ok()?;
        Self::from_slug(slug)
    }

    fn detect_appveyor() -> Option<Self> {
        let slug = env::var("APPVEYOR_REPO_NAME").ok()?;
        Self::from_slug(slug)
    }

    fn detect_drone() -> Option<Self> {
        // DRONE_REPO is the full repo name (owner/name)
        let slug = env::var("DRONE_REPO").ok()?;
        Self::from_slug(slug)
    }

    fn detect_teamcity() -> Option<Self> {
        // TeamCity doesn't have standard repo env vars
        // Try to parse from VCS_ROOT_URL if available
        let vcs_url = env::var("env.VCS_ROOT_URL").ok()?;
        // Try to parse GitHub/GitLab URL
        if let Some(slug) = vcs_url
            .strip_prefix("https://github.com/")
            .or_else(|| vcs_url.strip_prefix("git@github.com:"))
            .and_then(|s| s.strip_suffix(".git").or(Some(s)))
        {
            return Self::from_slug(slug);
        }
        None
    }
}

/// Information about the current commit.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommitInfo {
    /// Commit SHA.
    pub sha: String,
    /// Short SHA (first 7 characters).
    pub short_sha: String,
    /// Commit message.
    pub message: Option<String>,
    /// Author name.
    pub author: Option<String>,
    /// Branch name (if not a PR).
    pub branch: Option<String>,
}

impl CommitInfo {
    /// Try to detect commit info from the environment.
    #[must_use]
    pub fn detect() -> Option<Self> {
        let platform = CiPlatform::detect();
        match platform {
            CiPlatform::GitHubActions => Self::detect_github_actions(),
            CiPlatform::GitLabCi => Self::detect_gitlab_ci(),
            CiPlatform::CircleCi => Self::detect_circleci(),
            CiPlatform::TravisCi => Self::detect_travis_ci(),
            CiPlatform::AzurePipelines => Self::detect_azure_pipelines(),
            CiPlatform::Jenkins => Self::detect_jenkins(),
            CiPlatform::BitbucketPipelines => Self::detect_bitbucket(),
            CiPlatform::AppVeyor => Self::detect_appveyor(),
            CiPlatform::DroneCi => Self::detect_drone(),
            CiPlatform::TeamCity => Self::detect_teamcity(),
            CiPlatform::Unknown => None,
        }
    }

    /// Create from a SHA.
    #[must_use]
    pub fn from_sha(sha: impl Into<String>) -> Self {
        let sha = sha.into();
        let short_sha = if sha.len() >= 7 {
            sha[..7].to_string()
        } else {
            sha.clone()
        };
        Self {
            sha,
            short_sha,
            message: None,
            author: None,
            branch: None,
        }
    }

    fn detect_github_actions() -> Option<Self> {
        let sha = env::var("GITHUB_SHA").ok()?;
        let mut info = Self::from_sha(sha);

        // Try to get branch from GITHUB_REF
        if let Ok(github_ref) = env::var("GITHUB_REF") {
            if github_ref.starts_with("refs/heads/") {
                info.branch = Some(
                    github_ref
                        .strip_prefix("refs/heads/")
                        .unwrap_or(&github_ref)
                        .to_string(),
                );
            }
        }

        // Try to get commit message from event payload
        if let Ok(event_path) = env::var("GITHUB_EVENT_PATH") {
            if let Ok(event_content) = fs::read_to_string(&event_path) {
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(&event_content) {
                    if let Some(commits) = event.get("commits").and_then(|c| c.as_array()) {
                        if let Some(last_commit) = commits.last() {
                            info.message = last_commit
                                .get("message")
                                .and_then(|m| m.as_str())
                                .map(ToString::to_string);
                            info.author = last_commit
                                .get("author")
                                .and_then(|a| a.get("name"))
                                .and_then(|n| n.as_str())
                                .map(ToString::to_string);
                        }
                    }
                }
            }
        }

        Some(info)
    }

    fn detect_gitlab_ci() -> Option<Self> {
        let sha = env::var("CI_COMMIT_SHA").ok()?;
        let mut info = Self::from_sha(sha);

        info.branch = env::var("CI_COMMIT_REF_NAME").ok();
        info.message = env::var("CI_COMMIT_MESSAGE").ok();
        info.author = env::var("CI_COMMIT_AUTHOR").ok();

        Some(info)
    }

    fn detect_circleci() -> Option<Self> {
        let sha = env::var("CIRCLE_SHA1").ok()?;
        let mut info = Self::from_sha(sha);

        info.branch = env::var("CIRCLE_BRANCH").ok();

        Some(info)
    }

    fn detect_travis_ci() -> Option<Self> {
        let sha = env::var("TRAVIS_COMMIT").ok()?;
        let mut info = Self::from_sha(sha);

        info.branch = env::var("TRAVIS_BRANCH").ok();
        info.message = env::var("TRAVIS_COMMIT_MESSAGE").ok();

        Some(info)
    }

    fn detect_azure_pipelines() -> Option<Self> {
        let sha = env::var("BUILD_SOURCEVERSION").ok()?;
        let mut info = Self::from_sha(sha);

        // BUILD_SOURCEBRANCH is like refs/heads/main
        if let Ok(source_branch) = env::var("BUILD_SOURCEBRANCH") {
            if source_branch.starts_with("refs/heads/") {
                info.branch = Some(
                    source_branch
                        .strip_prefix("refs/heads/")
                        .unwrap_or(&source_branch)
                        .to_string(),
                );
            } else {
                info.branch = Some(source_branch);
            }
        }

        info.message = env::var("BUILD_SOURCEVERSIONMESSAGE").ok();
        info.author = env::var("BUILD_REQUESTEDFOR").ok();

        Some(info)
    }

    fn detect_jenkins() -> Option<Self> {
        let sha = env::var("GIT_COMMIT").ok()?;
        let mut info = Self::from_sha(sha);

        if let Ok(branch) = env::var("GIT_BRANCH") {
            // GIT_BRANCH can be origin/main or just main
            info.branch = Some(branch.rsplit('/').next().unwrap_or(&branch).to_string());
        }

        Some(info)
    }

    fn detect_bitbucket() -> Option<Self> {
        let sha = env::var("BITBUCKET_COMMIT").ok()?;
        let mut info = Self::from_sha(sha);

        info.branch = env::var("BITBUCKET_BRANCH").ok();

        Some(info)
    }

    fn detect_appveyor() -> Option<Self> {
        let sha = env::var("APPVEYOR_REPO_COMMIT").ok()?;
        let mut info = Self::from_sha(sha);

        info.branch = env::var("APPVEYOR_REPO_BRANCH").ok();
        info.message = env::var("APPVEYOR_REPO_COMMIT_MESSAGE").ok();
        info.author = env::var("APPVEYOR_REPO_COMMIT_AUTHOR").ok();

        Some(info)
    }

    fn detect_drone() -> Option<Self> {
        let sha = env::var("DRONE_COMMIT_SHA").ok()?;
        let mut info = Self::from_sha(sha);

        info.branch = env::var("DRONE_BRANCH").ok();
        info.message = env::var("DRONE_COMMIT_MESSAGE").ok();
        info.author = env::var("DRONE_COMMIT_AUTHOR").ok();

        Some(info)
    }

    fn detect_teamcity() -> Option<Self> {
        let sha = env::var("env.BUILD_VCS_NUMBER").ok()?;
        let info = Self::from_sha(sha);

        Some(info)
    }
}

/// Combined CI environment information.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CiEnvironment {
    /// The detected CI platform.
    pub platform: CiPlatform,
    /// Repository information (if available).
    pub repository: Option<RepositoryInfo>,
    /// Commit information (if available).
    pub commit: Option<CommitInfo>,
    /// Pull request information (if this is a PR build).
    pub pull_request: Option<PullRequestInfo>,
}

impl CiEnvironment {
    /// Detect the full CI environment.
    ///
    /// # Examples
    /// ```
    /// use lintdiff_ci_env::CiEnvironment;
    ///
    /// let env = CiEnvironment::detect();
    /// println!("Platform: {:?}", env.platform);
    /// if let Some(repo) = &env.repository {
    ///     println!("Repository: {}", repo.slug);
    /// }
    /// ```
    #[must_use]
    pub fn detect() -> Self {
        Self {
            platform: CiPlatform::detect(),
            repository: RepositoryInfo::detect(),
            commit: CommitInfo::detect(),
            pull_request: PullRequestInfo::detect(),
        }
    }

    /// Check if this is a pull request build.
    #[must_use]
    pub const fn is_pull_request(&self) -> bool {
        self.pull_request.is_some()
    }

    /// Check if running in any CI environment.
    #[must_use]
    pub fn is_ci(&self) -> bool {
        self.platform != CiPlatform::Unknown
    }

    /// Get the git reference for this build.
    ///
    /// Returns the PR branch, commit SHA, or branch name depending on context.
    #[must_use]
    pub fn git_ref(&self) -> Option<&str> {
        if let Some(pr) = &self.pull_request {
            return Some(&pr.source_branch);
        }
        if let Some(commit) = &self.commit {
            if let Some(branch) = &commit.branch {
                return Some(branch);
            }
            return Some(&commit.sha);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_platform_name() {
        assert_eq!(CiPlatform::GitHubActions.name(), "GitHub Actions");
        assert_eq!(CiPlatform::GitLabCi.name(), "GitLab CI");
        assert_eq!(CiPlatform::CircleCi.name(), "CircleCI");
        assert_eq!(CiPlatform::TravisCi.name(), "Travis CI");
        assert_eq!(CiPlatform::AzurePipelines.name(), "Azure Pipelines");
        assert_eq!(CiPlatform::Jenkins.name(), "Jenkins");
        assert_eq!(CiPlatform::BitbucketPipelines.name(), "Bitbucket Pipelines");
        assert_eq!(CiPlatform::AppVeyor.name(), "AppVeyor");
        assert_eq!(CiPlatform::DroneCi.name(), "Drone CI");
        assert_eq!(CiPlatform::TeamCity.name(), "TeamCity");
        assert_eq!(CiPlatform::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_ci_platform_supports_github_annotations() {
        assert!(CiPlatform::GitHubActions.supports_github_annotations());
        assert!(!CiPlatform::GitLabCi.supports_github_annotations());
        assert!(!CiPlatform::Unknown.supports_github_annotations());
    }

    #[test]
    fn test_ci_platform_supports_markdown() {
        assert!(CiPlatform::GitHubActions.supports_markdown());
        assert!(CiPlatform::GitLabCi.supports_markdown());
        assert!(!CiPlatform::CircleCi.supports_markdown());
        assert!(!CiPlatform::Unknown.supports_markdown());
    }

    #[test]
    fn test_repository_info_new() {
        let repo = RepositoryInfo::new("owner", "repo");
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
        assert_eq!(repo.slug, "owner/repo");
    }

    #[test]
    fn test_repository_info_from_slug() {
        let repo = RepositoryInfo::from_slug("owner/repo");
        assert!(repo.is_some());
        let repo = repo.unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
        assert_eq!(repo.slug, "owner/repo");
    }

    #[test]
    fn test_repository_info_from_invalid_slug() {
        let repo = RepositoryInfo::from_slug("invalid");
        assert!(repo.is_none());
    }

    #[test]
    fn test_commit_info_from_sha() {
        let commit = CommitInfo::from_sha("abcdef1234567890");
        assert_eq!(commit.sha, "abcdef1234567890");
        assert_eq!(commit.short_sha, "abcdef1");
        assert!(commit.message.is_none());
        assert!(commit.author.is_none());
        assert!(commit.branch.is_none());
    }

    #[test]
    fn test_commit_info_from_short_sha() {
        let commit = CommitInfo::from_sha("abc");
        assert_eq!(commit.sha, "abc");
        assert_eq!(commit.short_sha, "abc");
    }

    #[test]
    fn test_ci_environment_git_ref() {
        let env = CiEnvironment {
            platform: CiPlatform::Unknown,
            repository: None,
            commit: Some(CommitInfo::from_sha("abcdef1234567890")),
            pull_request: None,
        };
        assert_eq!(env.git_ref(), Some("abcdef1234567890"));
    }

    #[test]
    fn test_ci_environment_git_ref_with_branch() {
        let mut commit = CommitInfo::from_sha("abcdef1234567890");
        commit.branch = Some("main".to_string());
        let env = CiEnvironment {
            platform: CiPlatform::Unknown,
            repository: None,
            commit: Some(commit),
            pull_request: None,
        };
        assert_eq!(env.git_ref(), Some("main"));
    }

    #[test]
    fn test_ci_environment_git_ref_with_pr() {
        let env = CiEnvironment {
            platform: CiPlatform::Unknown,
            repository: None,
            commit: Some(CommitInfo::from_sha("abcdef1234567890")),
            pull_request: Some(PullRequestInfo {
                number: 42,
                source_branch: "feature".to_string(),
                target_branch: "main".to_string(),
            }),
        };
        assert_eq!(env.git_ref(), Some("feature"));
    }
}
