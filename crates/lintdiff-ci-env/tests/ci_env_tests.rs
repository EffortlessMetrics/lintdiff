//! Comprehensive tests for lintdiff-ci-env crate.
//!
//! These tests cover CI platform detection, repository info extraction,
//! commit info extraction, PR detection, and edge cases.

use lintdiff_ci_env::{CiEnvironment, CiPlatform, CommitInfo, PullRequestInfo, RepositoryInfo};
use temp_env::{with_var, with_vars};

// =============================================================================
// CiPlatform::detect tests (10 tests)
// =============================================================================

#[test]
fn test_detect_github_actions() {
    with_var("GITHUB_ACTIONS", Some("true"), || {
        let platform = CiPlatform::detect();
        assert_eq!(platform, CiPlatform::GitHubActions);
    });
}

#[test]
fn test_detect_gitlab_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", Some("true")),
        ],
        || {
            let platform = CiPlatform::detect();
            assert_eq!(platform, CiPlatform::GitLabCi);
        },
    );
}

#[test]
fn test_detect_circleci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", Some("true")),
        ],
        || {
            let platform = CiPlatform::detect();
            assert_eq!(platform, CiPlatform::CircleCi);
        },
    );
}

#[test]
fn test_detect_travis_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", Some("true")),
        ],
        || {
            let platform = CiPlatform::detect();
            assert_eq!(platform, CiPlatform::TravisCi);
        },
    );
}

#[test]
fn test_detect_azure_pipelines() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", Some("True")),
        ],
        || {
            let platform = CiPlatform::detect();
            assert_eq!(platform, CiPlatform::AzurePipelines);
        },
    );
}

#[test]
fn test_detect_jenkins() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", Some("http://jenkins.example.com")),
        ],
        || {
            let platform = CiPlatform::detect();
            assert_eq!(platform, CiPlatform::Jenkins);
        },
    );
}

#[test]
fn test_detect_bitbucket_pipelines() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", Some("123")),
        ],
        || {
            let platform = CiPlatform::detect();
            assert_eq!(platform, CiPlatform::BitbucketPipelines);
        },
    );
}

#[test]
fn test_detect_appveyor() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", None::<&str>),
            ("APPVEYOR", Some("true")),
        ],
        || {
            let platform = CiPlatform::detect();
            assert_eq!(platform, CiPlatform::AppVeyor);
        },
    );
}

#[test]
fn test_detect_drone_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", None::<&str>),
            ("APPVEYOR", None::<&str>),
            ("DRONE", Some("true")),
        ],
        || {
            let platform = CiPlatform::detect();
            assert_eq!(platform, CiPlatform::DroneCi);
        },
    );
}

#[test]
fn test_detect_teamcity() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", None::<&str>),
            ("APPVEYOR", None::<&str>),
            ("DRONE", None::<&str>),
            ("TEAMCITY_VERSION", Some("2023.11")),
        ],
        || {
            let platform = CiPlatform::detect();
            assert_eq!(platform, CiPlatform::TeamCity);
        },
    );
}

// =============================================================================
// CiPlatform::is_ci tests (3 tests)
// =============================================================================

#[test]
fn test_is_ci_returns_true_in_github_actions() {
    with_var("GITHUB_ACTIONS", Some("true"), || {
        assert!(CiPlatform::is_ci());
    });
}

#[test]
fn test_is_ci_returns_true_in_gitlab_ci() {
    with_vars(
        vec![("GITHUB_ACTIONS", None::<&str>), ("GITLAB_CI", Some("true"))],
        || {
            assert!(CiPlatform::is_ci());
        },
    );
}

#[test]
fn test_is_ci_returns_false_outside_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", None::<&str>),
            ("APPVEYOR", None::<&str>),
            ("DRONE", None::<&str>),
            ("TEAMCITY_VERSION", None::<&str>),
        ],
        || {
            assert!(!CiPlatform::is_ci());
        },
    );
}

// =============================================================================
// CiPlatform::supports_* methods tests (4 tests)
// =============================================================================

#[test]
fn test_supports_github_annotations_only_on_github() {
    assert!(CiPlatform::GitHubActions.supports_github_annotations());
    assert!(!CiPlatform::GitLabCi.supports_github_annotations());
    assert!(!CiPlatform::CircleCi.supports_github_annotations());
    assert!(!CiPlatform::TravisCi.supports_github_annotations());
    assert!(!CiPlatform::AzurePipelines.supports_github_annotations());
    assert!(!CiPlatform::Jenkins.supports_github_annotations());
    assert!(!CiPlatform::BitbucketPipelines.supports_github_annotations());
    assert!(!CiPlatform::AppVeyor.supports_github_annotations());
    assert!(!CiPlatform::DroneCi.supports_github_annotations());
    assert!(!CiPlatform::TeamCity.supports_github_annotations());
    assert!(!CiPlatform::Unknown.supports_github_annotations());
}

#[test]
fn test_supports_markdown_on_github_and_gitlab() {
    assert!(CiPlatform::GitHubActions.supports_markdown());
    assert!(CiPlatform::GitLabCi.supports_markdown());
    assert!(!CiPlatform::CircleCi.supports_markdown());
    assert!(!CiPlatform::TravisCi.supports_markdown());
    assert!(!CiPlatform::AzurePipelines.supports_markdown());
    assert!(!CiPlatform::Jenkins.supports_markdown());
    assert!(!CiPlatform::BitbucketPipelines.supports_markdown());
    assert!(!CiPlatform::AppVeyor.supports_markdown());
    assert!(!CiPlatform::DroneCi.supports_markdown());
    assert!(!CiPlatform::TeamCity.supports_markdown());
    assert!(!CiPlatform::Unknown.supports_markdown());
}

#[test]
fn test_platform_name_returns_correct_strings() {
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
fn test_platform_default_uses_detect() {
    with_var("GITHUB_ACTIONS", Some("true"), || {
        let platform = CiPlatform::default();
        assert_eq!(platform, CiPlatform::GitHubActions);
    });
}

// =============================================================================
// PullRequestInfo::detect tests (5 tests)
// =============================================================================

#[test]
fn test_pr_info_detect_gitlab_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", Some("true")),
            ("CI_MERGE_REQUEST_IID", Some("42")),
            ("CI_MERGE_REQUEST_SOURCE_BRANCH_NAME", Some("feature-branch")),
            ("CI_MERGE_REQUEST_TARGET_BRANCH_NAME", Some("main")),
        ],
        || {
            let pr = PullRequestInfo::detect();
            assert!(pr.is_some());
            let pr = pr.unwrap();
            assert_eq!(pr.number, 42);
            assert_eq!(pr.source_branch, "feature-branch");
            assert_eq!(pr.target_branch, "main");
        },
    );
}

#[test]
fn test_pr_info_detect_circleci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", Some("true")),
            ("CIRCLE_PULL_REQUEST", Some("https://github.com/owner/repo/pull/123")),
            ("CIRCLE_BRANCH", Some("feature")),
        ],
        || {
            let pr = PullRequestInfo::detect();
            assert!(pr.is_some());
            let pr = pr.unwrap();
            assert_eq!(pr.number, 123);
            assert_eq!(pr.source_branch, "feature");
        },
    );
}

#[test]
fn test_pr_info_detect_travis_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", Some("true")),
            ("TRAVIS_PULL_REQUEST", Some("456")),
            ("TRAVIS_PULL_REQUEST_BRANCH", Some("dev")),
            ("TRAVIS_BRANCH", Some("main")),
        ],
        || {
            let pr = PullRequestInfo::detect();
            assert!(pr.is_some());
            let pr = pr.unwrap();
            assert_eq!(pr.number, 456);
            assert_eq!(pr.source_branch, "dev");
            assert_eq!(pr.target_branch, "main");
        },
    );
}

#[test]
fn test_pr_info_detect_bitbucket() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", Some("1")),
            ("BITBUCKET_PR_ID", Some("789")),
            ("BITBUCKET_BRANCH", Some("feature")),
        ],
        || {
            let pr = PullRequestInfo::detect();
            assert!(pr.is_some());
            let pr = pr.unwrap();
            assert_eq!(pr.number, 789);
            assert_eq!(pr.source_branch, "feature");
        },
    );
}

#[test]
fn test_pr_info_detect_returns_none_outside_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", None::<&str>),
            ("APPVEYOR", None::<&str>),
            ("DRONE", None::<&str>),
            ("TEAMCITY_VERSION", None::<&str>),
        ],
        || {
            let pr = PullRequestInfo::detect();
            assert!(pr.is_none());
        },
    );
}

// =============================================================================
// RepositoryInfo::detect tests (5 tests)
// =============================================================================

#[test]
fn test_repo_info_detect_github_actions() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", Some("true")),
            ("GITHUB_REPOSITORY", Some("owner/repo")),
        ],
        || {
            let repo = RepositoryInfo::detect();
            assert!(repo.is_some());
            let repo = repo.unwrap();
            assert_eq!(repo.owner, "owner");
            assert_eq!(repo.name, "repo");
            assert_eq!(repo.slug, "owner/repo");
        },
    );
}

#[test]
fn test_repo_info_detect_gitlab_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", Some("true")),
            ("CI_PROJECT_NAMESPACE", Some("my-org")),
            ("CI_PROJECT_NAME", Some("my-project")),
        ],
        || {
            let repo = RepositoryInfo::detect();
            assert!(repo.is_some());
            let repo = repo.unwrap();
            assert_eq!(repo.owner, "my-org");
            assert_eq!(repo.name, "my-project");
            assert_eq!(repo.slug, "my-org/my-project");
        },
    );
}

#[test]
fn test_repo_info_detect_travis_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", Some("true")),
            ("TRAVIS_REPO_SLUG", Some("travis-owner/travis-repo")),
        ],
        || {
            let repo = RepositoryInfo::detect();
            assert!(repo.is_some());
            let repo = repo.unwrap();
            assert_eq!(repo.owner, "travis-owner");
            assert_eq!(repo.name, "travis-repo");
            assert_eq!(repo.slug, "travis-owner/travis-repo");
        },
    );
}

#[test]
fn test_repo_info_detect_bitbucket() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", Some("1")),
            ("BITBUCKET_REPO_FULL_NAME", Some("bb-owner/bb-repo")),
        ],
        || {
            let repo = RepositoryInfo::detect();
            assert!(repo.is_some());
            let repo = repo.unwrap();
            assert_eq!(repo.owner, "bb-owner");
            assert_eq!(repo.name, "bb-repo");
            assert_eq!(repo.slug, "bb-owner/bb-repo");
        },
    );
}

#[test]
fn test_repo_info_detect_returns_none_outside_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", None::<&str>),
            ("APPVEYOR", None::<&str>),
            ("DRONE", None::<&str>),
            ("TEAMCITY_VERSION", None::<&str>),
        ],
        || {
            let repo = RepositoryInfo::detect();
            assert!(repo.is_none());
        },
    );
}

// =============================================================================
// CommitInfo::detect tests (5 tests)
// =============================================================================

#[test]
fn test_commit_info_detect_github_actions() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", Some("true")),
            ("GITHUB_SHA", Some("abcdef1234567890abcdef1234567890abcdef12")),
            ("GITHUB_REF", Some("refs/heads/main")),
        ],
        || {
            let commit = CommitInfo::detect();
            assert!(commit.is_some());
            let commit = commit.unwrap();
            assert_eq!(commit.sha, "abcdef1234567890abcdef1234567890abcdef12");
            assert_eq!(commit.short_sha, "abcdef1");
            assert_eq!(commit.branch, Some("main".to_string()));
        },
    );
}

#[test]
fn test_commit_info_detect_gitlab_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", Some("true")),
            ("CI_COMMIT_SHA", Some("1234567890abcdef1234567890abcdef12345678")),
            ("CI_COMMIT_REF_NAME", Some("develop")),
            ("CI_COMMIT_MESSAGE", Some("Fix bug")),
            ("CI_COMMIT_AUTHOR", Some("John Doe")),
        ],
        || {
            let commit = CommitInfo::detect();
            assert!(commit.is_some());
            let commit = commit.unwrap();
            assert_eq!(commit.sha, "1234567890abcdef1234567890abcdef12345678");
            assert_eq!(commit.short_sha, "1234567");
            assert_eq!(commit.branch, Some("develop".to_string()));
            assert_eq!(commit.message, Some("Fix bug".to_string()));
            assert_eq!(commit.author, Some("John Doe".to_string()));
        },
    );
}

#[test]
fn test_commit_info_detect_circleci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", Some("true")),
            ("CIRCLE_SHA1", Some("fedcba0987654321fedcba0987654321fedcba09")),
            ("CIRCLE_BRANCH", Some("feature-xyz")),
        ],
        || {
            let commit = CommitInfo::detect();
            assert!(commit.is_some());
            let commit = commit.unwrap();
            assert_eq!(commit.sha, "fedcba0987654321fedcba0987654321fedcba09");
            assert_eq!(commit.short_sha, "fedcba0");
            assert_eq!(commit.branch, Some("feature-xyz".to_string()));
        },
    );
}

#[test]
fn test_commit_info_detect_bitbucket() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", Some("1")),
            ("BITBUCKET_COMMIT", Some("1111111111111111111111111111111111111111")),
            ("BITBUCKET_BRANCH", Some("release")),
        ],
        || {
            let commit = CommitInfo::detect();
            assert!(commit.is_some());
            let commit = commit.unwrap();
            assert_eq!(commit.sha, "1111111111111111111111111111111111111111");
            assert_eq!(commit.short_sha, "1111111");
            assert_eq!(commit.branch, Some("release".to_string()));
        },
    );
}

#[test]
fn test_commit_info_detect_returns_none_outside_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", None::<&str>),
            ("APPVEYOR", None::<&str>),
            ("DRONE", None::<&str>),
            ("TEAMCITY_VERSION", None::<&str>),
        ],
        || {
            let commit = CommitInfo::detect();
            assert!(commit.is_none());
        },
    );
}

// =============================================================================
// CiEnvironment::detect tests (5 tests)
// =============================================================================

#[test]
fn test_ci_environment_detect_github_actions() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", Some("true")),
            ("GITHUB_REPOSITORY", Some("owner/repo")),
            ("GITHUB_SHA", Some("abcdef1234567890")),
        ],
        || {
            let env = CiEnvironment::detect();
            assert_eq!(env.platform, CiPlatform::GitHubActions);
            assert!(env.repository.is_some());
            assert!(env.commit.is_some());
        },
    );
}

#[test]
fn test_ci_environment_detect_gitlab_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", Some("true")),
            ("CI_PROJECT_NAMESPACE", Some("org")),
            ("CI_PROJECT_NAME", Some("project")),
            ("CI_COMMIT_SHA", Some("1234567890")),
        ],
        || {
            let env = CiEnvironment::detect();
            assert_eq!(env.platform, CiPlatform::GitLabCi);
            assert!(env.repository.is_some());
            assert!(env.commit.is_some());
        },
    );
}

#[test]
fn test_ci_environment_detect_returns_unknown_outside_ci() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", None::<&str>),
            ("APPVEYOR", None::<&str>),
            ("DRONE", None::<&str>),
            ("TEAMCITY_VERSION", None::<&str>),
        ],
        || {
            let env = CiEnvironment::detect();
            assert_eq!(env.platform, CiPlatform::Unknown);
            assert!(env.repository.is_none());
            assert!(env.commit.is_none());
            assert!(env.pull_request.is_none());
        },
    );
}

#[test]
fn test_ci_environment_default_uses_detect() {
    with_var("GITHUB_ACTIONS", Some("true"), || {
        let env = CiEnvironment::default();
        assert_eq!(env.platform, CiPlatform::GitHubActions);
    });
}

#[test]
fn test_ci_environment_is_ci() {
    with_var("GITHUB_ACTIONS", Some("true"), || {
        let env = CiEnvironment::detect();
        assert!(env.is_ci());
    });
}

// =============================================================================
// CiEnvironment helper methods tests (5 tests)
// =============================================================================

#[test]
fn test_ci_environment_is_pull_request_returns_true_when_pr_exists() {
    let env = CiEnvironment {
        platform: CiPlatform::GitHubActions,
        repository: None,
        commit: None,
        pull_request: Some(PullRequestInfo {
            number: 42,
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
        }),
    };
    assert!(env.is_pull_request());
}

#[test]
fn test_ci_environment_is_pull_request_returns_false_when_no_pr() {
    let env = CiEnvironment {
        platform: CiPlatform::GitHubActions,
        repository: None,
        commit: None,
        pull_request: None,
    };
    assert!(!env.is_pull_request());
}

#[test]
fn test_ci_environment_git_ref_returns_pr_source_branch_first() {
    let env = CiEnvironment {
        platform: CiPlatform::GitHubActions,
        repository: None,
        commit: Some(CommitInfo::from_sha("abc123")),
        pull_request: Some(PullRequestInfo {
            number: 42,
            source_branch: "feature-branch".to_string(),
            target_branch: "main".to_string(),
        }),
    };
    assert_eq!(env.git_ref(), Some("feature-branch"));
}

#[test]
fn test_ci_environment_git_ref_returns_branch_when_no_pr() {
    let mut commit = CommitInfo::from_sha("abc123");
    commit.branch = Some("main".to_string());
    let env = CiEnvironment {
        platform: CiPlatform::GitHubActions,
        repository: None,
        commit: Some(commit),
        pull_request: None,
    };
    assert_eq!(env.git_ref(), Some("main"));
}

#[test]
fn test_ci_environment_git_ref_returns_sha_when_no_branch() {
    let env = CiEnvironment {
        platform: CiPlatform::GitHubActions,
        repository: None,
        commit: Some(CommitInfo::from_sha("abcdef1234567890")),
        pull_request: None,
    };
    assert_eq!(env.git_ref(), Some("abcdef1234567890"));
}

// =============================================================================
// Edge cases and error handling tests (8 tests)
// =============================================================================

#[test]
fn test_commit_info_from_sha_with_short_sha() {
    let commit = CommitInfo::from_sha("abc");
    assert_eq!(commit.sha, "abc");
    assert_eq!(commit.short_sha, "abc");
}

#[test]
fn test_commit_info_from_sha_with_empty_sha() {
    let commit = CommitInfo::from_sha("");
    assert_eq!(commit.sha, "");
    assert_eq!(commit.short_sha, "");
}

#[test]
fn test_commit_info_from_sha_with_exact_seven_chars() {
    let commit = CommitInfo::from_sha("abcdefg");
    assert_eq!(commit.sha, "abcdefg");
    assert_eq!(commit.short_sha, "abcdefg");
}

#[test]
fn test_repository_info_from_slug_with_invalid_format() {
    let repo = RepositoryInfo::from_slug("no-slash");
    assert!(repo.is_none());
}

#[test]
fn test_repository_info_from_slug_with_multiple_slashes() {
    let repo = RepositoryInfo::from_slug("org/subgroup/repo");
    // Should only split on first occurrence
    assert!(repo.is_none()); // Our implementation expects exactly one slash
}

#[test]
fn test_repository_info_new_with_empty_strings() {
    let repo = RepositoryInfo::new("", "");
    assert_eq!(repo.owner, "");
    assert_eq!(repo.name, "");
    assert_eq!(repo.slug, "/");
}

#[test]
fn test_ci_platform_detect_with_false_values() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", Some("false")),
            ("GITLAB_CI", Some("false")),
            ("CIRCLECI", Some("false")),
            ("TRAVIS", Some("false")),
            ("APPVEYOR", Some("false")),
            ("DRONE", Some("false")),
        ],
        || {
            let platform = CiPlatform::detect();
            assert_eq!(platform, CiPlatform::Unknown);
        },
    );
}

#[test]
fn test_pr_info_detect_travis_with_false_value() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", Some("true")),
            ("TRAVIS_PULL_REQUEST", Some("false")),
        ],
        || {
            let pr = PullRequestInfo::detect();
            assert!(pr.is_none());
        },
    );
}

// =============================================================================
// Mock environment tests using temp-env (5 tests)
// =============================================================================

#[test]
fn test_mock_github_actions_environment() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", Some("true")),
            ("GITHUB_REPOSITORY", Some("test-owner/test-repo")),
            ("GITHUB_SHA", Some("1234567890abcdef1234567890abcdef12345678")),
            ("GITHUB_REF", Some("refs/heads/test-branch")),
        ],
        || {
            let env = CiEnvironment::detect();
            assert_eq!(env.platform, CiPlatform::GitHubActions);
            assert!(env.is_ci());
            assert!(env.repository.is_some());
            let repo = env.repository.unwrap();
            assert_eq!(repo.slug, "test-owner/test-repo");
            assert!(env.commit.is_some());
            let commit = env.commit.unwrap();
            assert_eq!(commit.branch, Some("test-branch".to_string()));
        },
    );
}

#[test]
fn test_mock_gitlab_ci_environment() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", Some("true")),
            ("CI_PROJECT_NAMESPACE", Some("gitlab-org")),
            ("CI_PROJECT_NAME", Some("gitlab-project")),
            ("CI_COMMIT_SHA", Some("abcdef123456")),
            ("CI_COMMIT_REF_NAME", Some("develop")),
            ("CI_MERGE_REQUEST_IID", Some("100")),
            ("CI_MERGE_REQUEST_SOURCE_BRANCH_NAME", Some("feature")),
            ("CI_MERGE_REQUEST_TARGET_BRANCH_NAME", Some("develop")),
        ],
        || {
            let env = CiEnvironment::detect();
            assert_eq!(env.platform, CiPlatform::GitLabCi);
            assert!(env.is_ci());
            assert!(env.repository.is_some());
            assert!(env.commit.is_some());
            assert!(env.pull_request.is_some());
            let pr = env.pull_request.unwrap();
            assert_eq!(pr.number, 100);
            assert_eq!(pr.source_branch, "feature");
            assert_eq!(pr.target_branch, "develop");
        },
    );
}

#[test]
fn test_mock_circleci_environment() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", Some("true")),
            ("CIRCLE_PROJECT_USERNAME", Some("circle-org")),
            ("CIRCLE_PROJECT_REPONAME", Some("circle-repo")),
            ("CIRCLE_SHA1", Some("1111111111111111111111111111111111111111")),
            ("CIRCLE_BRANCH", Some("circle-branch")),
            ("CIRCLE_PULL_REQUEST", Some("https://github.com/circle-org/circle-repo/pull/50")),
        ],
        || {
            let env = CiEnvironment::detect();
            assert_eq!(env.platform, CiPlatform::CircleCi);
            assert!(env.is_ci());
            assert!(env.repository.is_some());
            assert!(env.commit.is_some());
            assert!(env.pull_request.is_some());
            let pr = env.pull_request.unwrap();
            assert_eq!(pr.number, 50);
        },
    );
}

#[test]
fn test_mock_azure_pipelines_environment() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", Some("True")),
            ("BUILD_REPOSITORY_NAME", Some("azure-org/azure-repo")),
            ("BUILD_SOURCEVERSION", Some("2222222222222222222222222222222222222222")),
            ("BUILD_SOURCEBRANCH", Some("refs/heads/azure-branch")),
            ("SYSTEM_PULLREQUEST_PULLREQUESTNUMBER", Some("75")),
        ],
        || {
            let env = CiEnvironment::detect();
            assert_eq!(env.platform, CiPlatform::AzurePipelines);
            assert!(env.is_ci());
            assert!(env.repository.is_some());
            assert!(env.commit.is_some());
            let commit = env.commit.unwrap();
            assert_eq!(commit.branch, Some("azure-branch".to_string()));
        },
    );
}

#[test]
fn test_mock_drone_ci_environment() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", None::<&str>),
            ("APPVEYOR", None::<&str>),
            ("DRONE", Some("true")),
            ("DRONE_REPO", Some("drone-org/drone-repo")),
            ("DRONE_COMMIT_SHA", Some("3333333333333333333333333333333333333333")),
            ("DRONE_BRANCH", Some("drone-branch")),
            ("DRONE_COMMIT_MESSAGE", Some("Test commit")),
            ("DRONE_COMMIT_AUTHOR", Some("Test Author")),
            ("DRONE_PULL_REQUEST", Some("25")),
        ],
        || {
            let env = CiEnvironment::detect();
            assert_eq!(env.platform, CiPlatform::DroneCi);
            assert!(env.is_ci());
            assert!(env.repository.is_some());
            let repo = env.repository.unwrap();
            assert_eq!(repo.slug, "drone-org/drone-repo");
            assert!(env.commit.is_some());
            let commit = env.commit.unwrap();
            assert_eq!(commit.message, Some("Test commit".to_string()));
            assert_eq!(commit.author, Some("Test Author".to_string()));
            assert!(env.pull_request.is_some());
            let pr = env.pull_request.unwrap();
            assert_eq!(pr.number, 25);
        },
    );
}

// =============================================================================
// Additional coverage tests
// =============================================================================

#[test]
fn test_ci_platform_equality() {
    assert_eq!(CiPlatform::GitHubActions, CiPlatform::GitHubActions);
    assert_ne!(CiPlatform::GitHubActions, CiPlatform::GitLabCi);
}

#[test]
fn test_ci_platform_clone() {
    let platform = CiPlatform::GitHubActions;
    let cloned = platform.clone();
    assert_eq!(platform, cloned);
}

#[test]
fn test_ci_platform_copy() {
    let platform = CiPlatform::GitHubActions;
    let copied = platform;
    assert_eq!(platform, copied);
}

#[test]
fn test_pull_request_info_clone() {
    let pr = PullRequestInfo {
        number: 42,
        source_branch: "feature".to_string(),
        target_branch: "main".to_string(),
    };
    let cloned = pr.clone();
    assert_eq!(pr, cloned);
}

#[test]
fn test_repository_info_clone() {
    let repo = RepositoryInfo::new("owner", "repo");
    let cloned = repo.clone();
    assert_eq!(repo, cloned);
}

#[test]
fn test_commit_info_clone() {
    let commit = CommitInfo::from_sha("abc123");
    let cloned = commit.clone();
    assert_eq!(commit, cloned);
}

#[test]
fn test_ci_environment_clone() {
    let env = CiEnvironment::detect();
    let cloned = env.clone();
    assert_eq!(env.platform, cloned.platform);
}

#[test]
fn test_ci_platform_debug_format() {
    assert!(format!("{:?}", CiPlatform::GitHubActions).contains("GitHubActions"));
    assert!(format!("{:?}", CiPlatform::Unknown).contains("Unknown"));
}

#[test]
fn test_pull_request_info_debug_format() {
    let pr = PullRequestInfo {
        number: 1,
        source_branch: "a".to_string(),
        target_branch: "b".to_string(),
    };
    let debug_str = format!("{:?}", pr);
    assert!(debug_str.contains("number"));
    assert!(debug_str.contains("source_branch"));
}

#[test]
fn test_repository_info_debug_format() {
    let repo = RepositoryInfo::new("owner", "repo");
    let debug_str = format!("{:?}", repo);
    assert!(debug_str.contains("owner"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("slug"));
}

#[test]
fn test_commit_info_debug_format() {
    let commit = CommitInfo::from_sha("abc123");
    let debug_str = format!("{:?}", commit);
    assert!(debug_str.contains("sha"));
    assert!(debug_str.contains("short_sha"));
}

#[test]
fn test_ci_environment_debug_format() {
    let env = CiEnvironment::detect();
    let debug_str = format!("{:?}", env);
    assert!(debug_str.contains("platform"));
}

#[test]
fn test_appveyor_commit_detection() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", None::<&str>),
            ("BITBUCKET_BUILD_NUMBER", None::<&str>),
            ("APPVEYOR", Some("true")),
            ("APPVEYOR_REPO_COMMIT", Some("aaaa1111bbbb2222")),
            ("APPVEYOR_REPO_BRANCH", Some("appveyor-branch")),
            ("APPVEYOR_REPO_COMMIT_MESSAGE", Some("AppVeyor commit")),
            ("APPVEYOR_REPO_COMMIT_AUTHOR", Some("AppVeyor Author")),
        ],
        || {
            let commit = CommitInfo::detect();
            assert!(commit.is_some());
            let commit = commit.unwrap();
            assert_eq!(commit.sha, "aaaa1111bbbb2222");
            assert_eq!(commit.branch, Some("appveyor-branch".to_string()));
            assert_eq!(commit.message, Some("AppVeyor commit".to_string()));
            assert_eq!(commit.author, Some("AppVeyor Author".to_string()));
        },
    );
}

#[test]
fn test_jenkins_commit_detection() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", None::<&str>),
            ("JENKINS_URL", Some("http://jenkins.example.com")),
            ("GIT_COMMIT", Some("jjjjkkkkllll")),
            ("GIT_BRANCH", Some("origin/jenkins-branch")),
        ],
        || {
            let commit = CommitInfo::detect();
            assert!(commit.is_some());
            let commit = commit.unwrap();
            assert_eq!(commit.sha, "jjjjkkkkllll");
            // Branch should be extracted from origin/branch format
            assert_eq!(commit.branch, Some("jenkins-branch".to_string()));
        },
    );
}

#[test]
fn test_azure_pipelines_with_source_branch_id() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", None::<&str>),
            ("TF_BUILD", Some("True")),
            ("BUILD_SOURCEVERSION", Some("azure123")),
            ("SYSTEM_PULLREQUEST_PULLREQUESTID", Some("999")), // Alternative PR ID var
        ],
        || {
            let pr = PullRequestInfo::detect();
            assert!(pr.is_some());
            let pr = pr.unwrap();
            assert_eq!(pr.number, 999);
        },
    );
}

#[test]
fn test_git_ref_returns_none_when_no_info() {
    let env = CiEnvironment {
        platform: CiPlatform::Unknown,
        repository: None,
        commit: None,
        pull_request: None,
    };
    assert!(env.git_ref().is_none());
}

#[test]
fn test_travis_commit_detection() {
    with_vars(
        vec![
            ("GITHUB_ACTIONS", None::<&str>),
            ("GITLAB_CI", None::<&str>),
            ("CIRCLECI", None::<&str>),
            ("TRAVIS", Some("true")),
            ("TRAVIS_COMMIT", Some("travis123456")),
            ("TRAVIS_BRANCH", Some("travis-branch")),
            ("TRAVIS_COMMIT_MESSAGE", Some("Travis commit message")),
        ],
        || {
            let commit = CommitInfo::detect();
            assert!(commit.is_some());
            let commit = commit.unwrap();
            assert_eq!(commit.sha, "travis123456");
            assert_eq!(commit.branch, Some("travis-branch".to_string()));
            assert_eq!(commit.message, Some("Travis commit message".to_string()));
        },
    );
}
