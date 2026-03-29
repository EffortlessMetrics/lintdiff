//! Comprehensive tests for lintdiff-code-url crate.
//!
//! Test categories:
//! 1. GitHub URL generation (12 tests)
//! 2. GitLab URL generation (10 tests)
//! 3. Bitbucket URL generation (10 tests)
//! 4. Azure DevOps URL generation (8 tests)
//! 5. Provider detection from URLs (10 tests)
//! 6. CodeLocation builder (6 tests)
//! 7. Edge cases and error handling (4 tests)

use lintdiff_code_url::{CodeLocation, CodeUrlBuilder, CodeUrlConfig, UrlProvider};

// =============================================================================
// 1. GitHub URL Generation Tests (12 tests)
// =============================================================================

mod github_tests {
    use super::*;

    #[test]
    fn github_file_url_no_line() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
        let url = config.file_url("src/lib.rs", None, None);
        assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs");
    }

    #[test]
    fn github_file_url_with_line() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
        let url = config.file_url("src/lib.rs", Some(42), None);
        assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L42");
    }

    #[test]
    fn github_file_url_with_line_range() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
        let url = config.file_url("src/lib.rs", Some(10), Some(20));
        assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L10-L20");
    }

    #[test]
    fn github_file_url_with_commit_sha() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "abc123def456");
        let url = config.file_url("src/lib.rs", Some(1), None);
        assert_eq!(url, "https://github.com/user/repo/blob/abc123def456/src/lib.rs#L1");
    }

    #[test]
    fn github_file_url_nested_path() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
        let url = config.file_url("src/deeply/nested/module.rs", Some(100), None);
        assert_eq!(
            url,
            "https://github.com/user/repo/blob/main/src/deeply/nested/module.rs#L100"
        );
    }

    #[test]
    fn github_commit_url() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
        let url = config.commit_url("abc123def456");
        assert_eq!(url, "https://github.com/user/repo/commit/abc123def456");
    }

    #[test]
    fn github_builder_file_url() {
        let builder = CodeUrlBuilder::github("owner", "repo", "develop");
        let url = builder.file_url("src/main.rs");
        assert_eq!(url, "https://github.com/owner/repo/blob/develop/src/main.rs");
    }

    #[test]
    fn github_builder_line_url() {
        let builder = CodeUrlBuilder::github("owner", "repo", "main");
        let url = builder.line_url("src/lib.rs", 42);
        assert_eq!(url, "https://github.com/owner/repo/blob/main/src/lib.rs#L42");
    }

    #[test]
    fn github_builder_range_url() {
        let builder = CodeUrlBuilder::github("owner", "repo", "main");
        let url = builder.range_url("src/lib.rs", 10, 20);
        assert_eq!(url, "https://github.com/owner/repo/blob/main/src/lib.rs#L10-L20");
    }

    #[test]
    fn github_builder_commit_url() {
        let builder = CodeUrlBuilder::github("owner", "repo", "main");
        let url = builder.commit_url("abc123");
        assert_eq!(url, "https://github.com/owner/repo/commit/abc123");
    }

    #[test]
    fn github_with_location() {
        let builder = CodeUrlBuilder::github("user", "repo", "main");
        let location = CodeLocation::file("src/lib.rs").with_line(42);
        let url = builder.url(&location);
        assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L42");
    }

    #[test]
    fn github_with_line_range_location() {
        let builder = CodeUrlBuilder::github("user", "repo", "main");
        let location = CodeLocation::file("src/lib.rs").with_line_range(5, 15);
        let url = builder.url(&location);
        assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L5-L15");
    }
}

// =============================================================================
// 2. GitLab URL Generation Tests (10 tests)
// =============================================================================

mod gitlab_tests {
    use super::*;

    #[test]
    fn gitlab_file_url_no_line() {
        let config = CodeUrlConfig::new("https://gitlab.com/user/repo", "main");
        let url = config.file_url("src/lib.rs", None, None);
        assert_eq!(url, "https://gitlab.com/user/repo/-/blob/main/src/lib.rs");
    }

    #[test]
    fn gitlab_file_url_with_line() {
        let config = CodeUrlConfig::new("https://gitlab.com/user/repo", "main");
        let url = config.file_url("src/lib.rs", Some(42), None);
        assert_eq!(url, "https://gitlab.com/user/repo/-/blob/main/src/lib.rs#L42");
    }

    #[test]
    fn gitlab_file_url_with_line_range() {
        let config = CodeUrlConfig::new("https://gitlab.com/user/repo", "main");
        let url = config.file_url("src/lib.rs", Some(10), Some(20));
        assert_eq!(url, "https://gitlab.com/user/repo/-/blob/main/src/lib.rs#L10-L20");
    }

    #[test]
    fn gitlab_file_url_with_tag() {
        let config = CodeUrlConfig::new("https://gitlab.com/user/repo", "v1.0.0");
        let url = config.file_url("src/lib.rs", Some(1), None);
        assert_eq!(url, "https://gitlab.com/user/repo/-/blob/v1.0.0/src/lib.rs#L1");
    }

    #[test]
    fn gitlab_commit_url() {
        let config = CodeUrlConfig::new("https://gitlab.com/user/repo", "main");
        let url = config.commit_url("abc123def456");
        assert_eq!(url, "https://gitlab.com/user/repo/-/commit/abc123def456");
    }

    #[test]
    fn gitlab_builder_file_url() {
        let builder = CodeUrlBuilder::gitlab("owner", "repo", "develop");
        let url = builder.file_url("src/main.rs");
        assert_eq!(url, "https://gitlab.com/owner/repo/-/blob/develop/src/main.rs");
    }

    #[test]
    fn gitlab_builder_line_url() {
        let builder = CodeUrlBuilder::gitlab("owner", "repo", "main");
        let url = builder.line_url("src/lib.rs", 42);
        assert_eq!(url, "https://gitlab.com/owner/repo/-/blob/main/src/lib.rs#L42");
    }

    #[test]
    fn gitlab_builder_range_url() {
        let builder = CodeUrlBuilder::gitlab("owner", "repo", "main");
        let url = builder.range_url("src/lib.rs", 10, 20);
        assert_eq!(url, "https://gitlab.com/owner/repo/-/blob/main/src/lib.rs#L10-L20");
    }

    #[test]
    fn gitlab_builder_commit_url() {
        let builder = CodeUrlBuilder::gitlab("owner", "repo", "main");
        let url = builder.commit_url("abc123");
        assert_eq!(url, "https://gitlab.com/owner/repo/-/commit/abc123");
    }

    #[test]
    fn gitlab_with_location() {
        let builder = CodeUrlBuilder::gitlab("user", "repo", "main");
        let location = CodeLocation::file("src/lib.rs").with_line(42);
        let url = builder.url(&location);
        assert_eq!(url, "https://gitlab.com/user/repo/-/blob/main/src/lib.rs#L42");
    }
}

// =============================================================================
// 3. Bitbucket URL Generation Tests (10 tests)
// =============================================================================

mod bitbucket_tests {
    use super::*;

    #[test]
    fn bitbucket_file_url_no_line() {
        let config = CodeUrlConfig::new("https://bitbucket.org/user/repo", "main");
        let url = config.file_url("src/lib.rs", None, None);
        assert_eq!(url, "https://bitbucket.org/user/repo/src/main/src/lib.rs");
    }

    #[test]
    fn bitbucket_file_url_with_line() {
        let config = CodeUrlConfig::new("https://bitbucket.org/user/repo", "main");
        let url = config.file_url("src/lib.rs", Some(42), None);
        assert_eq!(url, "https://bitbucket.org/user/repo/src/main/src/lib.rs#lines-42");
    }

    #[test]
    fn bitbucket_file_url_with_line_range() {
        let config = CodeUrlConfig::new("https://bitbucket.org/user/repo", "main");
        let url = config.file_url("src/lib.rs", Some(10), Some(20));
        assert_eq!(
            url,
            "https://bitbucket.org/user/repo/src/main/src/lib.rs#lines-10:20"
        );
    }

    #[test]
    fn bitbucket_file_url_with_commit_sha() {
        let config = CodeUrlConfig::new("https://bitbucket.org/user/repo", "abc123def456");
        let url = config.file_url("src/lib.rs", Some(1), None);
        assert_eq!(
            url,
            "https://bitbucket.org/user/repo/src/abc123def456/src/lib.rs#lines-1"
        );
    }

    #[test]
    fn bitbucket_commit_url() {
        let config = CodeUrlConfig::new("https://bitbucket.org/user/repo", "main");
        let url = config.commit_url("abc123def456");
        assert_eq!(url, "https://bitbucket.org/user/repo/commits/abc123def456");
    }

    #[test]
    fn bitbucket_builder_file_url() {
        let builder = CodeUrlBuilder::bitbucket("owner", "repo", "develop");
        let url = builder.file_url("src/main.rs");
        assert_eq!(url, "https://bitbucket.org/owner/repo/src/develop/src/main.rs");
    }

    #[test]
    fn bitbucket_builder_line_url() {
        let builder = CodeUrlBuilder::bitbucket("owner", "repo", "main");
        let url = builder.line_url("src/lib.rs", 42);
        assert_eq!(
            url,
            "https://bitbucket.org/owner/repo/src/main/src/lib.rs#lines-42"
        );
    }

    #[test]
    fn bitbucket_builder_range_url() {
        let builder = CodeUrlBuilder::bitbucket("owner", "repo", "main");
        let url = builder.range_url("src/lib.rs", 10, 20);
        assert_eq!(
            url,
            "https://bitbucket.org/owner/repo/src/main/src/lib.rs#lines-10:20"
        );
    }

    #[test]
    fn bitbucket_builder_commit_url() {
        let builder = CodeUrlBuilder::bitbucket("owner", "repo", "main");
        let url = builder.commit_url("abc123");
        assert_eq!(url, "https://bitbucket.org/owner/repo/commits/abc123");
    }

    #[test]
    fn bitbucket_with_location() {
        let builder = CodeUrlBuilder::bitbucket("user", "repo", "main");
        let location = CodeLocation::file("src/lib.rs").with_line(42);
        let url = builder.url(&location);
        assert_eq!(
            url,
            "https://bitbucket.org/user/repo/src/main/src/lib.rs#lines-42"
        );
    }
}

// =============================================================================
// 4. Azure DevOps URL Generation Tests (8 tests)
// =============================================================================

mod azure_devops_tests {
    use super::*;

    #[test]
    fn azure_file_url_no_line() {
        let config =
            CodeUrlConfig::new("https://dev.azure.com/org/project/_git/repo", "main");
        let url = config.file_url("src/lib.rs", None, None);
        assert_eq!(
            url,
            "https://dev.azure.com/org/project/_git/repo?path=src/lib.rs&version=main"
        );
    }

    #[test]
    fn azure_file_url_with_line() {
        let config =
            CodeUrlConfig::new("https://dev.azure.com/org/project/_git/repo", "main");
        let url = config.file_url("src/lib.rs", Some(42), None);
        assert_eq!(
            url,
            "https://dev.azure.com/org/project/_git/repo?path=src/lib.rs&version=main&line=42"
        );
    }

    #[test]
    fn azure_file_url_with_line_range() {
        let config =
            CodeUrlConfig::new("https://dev.azure.com/org/project/_git/repo", "main");
        let url = config.file_url("src/lib.rs", Some(10), Some(20));
        assert_eq!(
            url,
            "https://dev.azure.com/org/project/_git/repo?path=src/lib.rs&version=main&line=10&lineEnd=20"
        );
    }

    #[test]
    fn azure_commit_url() {
        let config =
            CodeUrlConfig::new("https://dev.azure.com/org/project/_git/repo", "main");
        let url = config.commit_url("abc123def456");
        assert_eq!(
            url,
            "https://dev.azure.com/org/project/_git/repo/commit/abc123def456"
        );
    }

    #[test]
    fn azure_builder_file_url() {
        let builder = CodeUrlBuilder::azure_devops("org", "project", "repo", "develop");
        let url = builder.file_url("src/main.rs");
        assert_eq!(
            url,
            "https://dev.azure.com/org/project/_git/repo?path=src/main.rs&version=develop"
        );
    }

    #[test]
    fn azure_builder_line_url() {
        let builder = CodeUrlBuilder::azure_devops("org", "project", "repo", "main");
        let url = builder.line_url("src/lib.rs", 42);
        assert_eq!(
            url,
            "https://dev.azure.com/org/project/_git/repo?path=src/lib.rs&version=main&line=42"
        );
    }

    #[test]
    fn azure_builder_range_url() {
        let builder = CodeUrlBuilder::azure_devops("org", "project", "repo", "main");
        let url = builder.range_url("src/lib.rs", 10, 20);
        assert_eq!(
            url,
            "https://dev.azure.com/org/project/_git/repo?path=src/lib.rs&version=main&line=10&lineEnd=20"
        );
    }

    #[test]
    fn azure_builder_commit_url() {
        let builder = CodeUrlBuilder::azure_devops("org", "project", "repo", "main");
        let url = builder.commit_url("abc123");
        assert_eq!(
            url,
            "https://dev.azure.com/org/project/_git/repo/commit/abc123"
        );
    }
}

// =============================================================================
// 5. Provider Detection Tests (10 tests)
// =============================================================================

mod provider_detection_tests {
    use super::*;

    #[test]
    fn detect_github_from_url() {
        assert_eq!(
            UrlProvider::from_url("https://github.com/user/repo"),
            UrlProvider::GitHub
        );
    }

    #[test]
    fn detect_gitlab_from_url() {
        assert_eq!(
            UrlProvider::from_url("https://gitlab.com/user/repo"),
            UrlProvider::GitLab
        );
    }

    #[test]
    fn detect_bitbucket_from_url() {
        assert_eq!(
            UrlProvider::from_url("https://bitbucket.org/user/repo"),
            UrlProvider::Bitbucket
        );
    }

    #[test]
    fn detect_azure_devops_from_url() {
        assert_eq!(
            UrlProvider::from_url("https://dev.azure.com/org/project/_git/repo"),
            UrlProvider::AzureDevOps
        );
    }

    #[test]
    fn detect_github_enterprise() {
        assert_eq!(
            UrlProvider::from_url("https://github.enterprise.com/user/repo"),
            UrlProvider::GitHub
        );
    }

    #[test]
    fn detect_self_hosted_gitlab() {
        assert_eq!(
            UrlProvider::from_url("https://gitlab.company.com/user/repo"),
            UrlProvider::GitLab
        );
    }

    #[test]
    fn detect_generic_provider() {
        assert_eq!(
            UrlProvider::from_url("https://custom-git.example.com/user/repo"),
            UrlProvider::Generic
        );
    }

    #[test]
    fn detect_provider_case_insensitive() {
        assert_eq!(
            UrlProvider::from_url("https://GITHUB.COM/user/repo"),
            UrlProvider::GitHub
        );
        assert_eq!(
            UrlProvider::from_url("https://GITLAB.COM/user/repo"),
            UrlProvider::GitLab
        );
    }

    #[test]
    fn config_detect_provider_github() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
        assert_eq!(config.detect_provider(), UrlProvider::GitHub);
    }

    #[test]
    fn config_detect_provider_gitlab() {
        let config = CodeUrlConfig::new("https://gitlab.com/user/repo", "main");
        assert_eq!(config.detect_provider(), UrlProvider::GitLab);
    }
}

// =============================================================================
// 6. CodeLocation Builder Tests (6 tests)
// =============================================================================

mod code_location_tests {
    use super::*;

    #[test]
    fn code_location_file_only() {
        let location = CodeLocation::file("src/lib.rs");
        assert_eq!(location.path, "src/lib.rs");
        assert_eq!(location.line, None);
        assert_eq!(location.end_line, None);
        assert_eq!(location.column, None);
    }

    #[test]
    fn code_location_with_line() {
        let location = CodeLocation::file("src/lib.rs").with_line(42);
        assert_eq!(location.path, "src/lib.rs");
        assert_eq!(location.line, Some(42));
        assert_eq!(location.end_line, None);
    }

    #[test]
    fn code_location_with_line_range() {
        let location = CodeLocation::file("src/lib.rs").with_line_range(10, 20);
        assert_eq!(location.path, "src/lib.rs");
        assert_eq!(location.line, Some(10));
        assert_eq!(location.end_line, Some(20));
    }

    #[test]
    fn code_location_with_column() {
        let location = CodeLocation::file("src/lib.rs").with_line(42).with_column(5);
        assert_eq!(location.path, "src/lib.rs");
        assert_eq!(location.line, Some(42));
        assert_eq!(location.column, Some(5));
    }

    #[test]
    fn code_location_to_url() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
        let location = CodeLocation::file("src/lib.rs").with_line(42);
        let url = location.to_url(&config);
        assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L42");
    }

    #[test]
    fn code_location_equality() {
        let loc1 = CodeLocation::file("src/lib.rs").with_line(42);
        let loc2 = CodeLocation::file("src/lib.rs").with_line(42);
        let loc3 = CodeLocation::file("src/lib.rs").with_line(43);

        assert_eq!(loc1, loc2);
        assert_ne!(loc1, loc3);
    }
}

// =============================================================================
// 7. Edge Cases and Error Handling Tests (4 tests)
// =============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn empty_base_url() {
        let config = CodeUrlConfig::default();
        assert_eq!(config.base_url, "");
        assert_eq!(config.git_ref, "main");
    }

    #[test]
    fn path_normalization_backslashes() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
        // Backslashes should be converted to forward slashes
        let url = config.file_url("src\\lib.rs", Some(42), None);
        assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L42");
    }

    #[test]
    fn path_normalization_leading_slash() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
        // Leading slashes should be removed
        let url = config.file_url("/src/lib.rs", Some(42), None);
        assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L42");
    }

    #[test]
    fn explicit_provider_overrides_detection() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main")
            .with_provider(UrlProvider::GitLab);
        // Even though URL looks like GitHub, explicit provider should be used
        assert_eq!(config.detect_provider(), UrlProvider::GitLab);
        let url = config.file_url("src/lib.rs", Some(42), None);
        assert_eq!(url, "https://github.com/user/repo/-/blob/main/src/lib.rs#L42");
    }
}

// =============================================================================
// Additional Tests to Meet 60+ Test Requirement
// =============================================================================

mod additional_tests {
    use super::*;

    #[test]
    fn github_with_tag_ref() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "v2.0.0");
        let url = config.file_url("src/lib.rs", Some(1), None);
        assert_eq!(url, "https://github.com/user/repo/blob/v2.0.0/src/lib.rs#L1");
    }

    #[test]
    fn gitlab_with_tag_ref() {
        let config = CodeUrlConfig::new("https://gitlab.com/user/repo", "v2.0.0");
        let url = config.file_url("src/lib.rs", Some(1), None);
        assert_eq!(url, "https://gitlab.com/user/repo/-/blob/v2.0.0/src/lib.rs#L1");
    }

    #[test]
    fn bitbucket_with_tag_ref() {
        let config = CodeUrlConfig::new("https://bitbucket.org/user/repo", "v2.0.0");
        let url = config.file_url("src/lib.rs", Some(1), None);
        assert_eq!(
            url,
            "https://bitbucket.org/user/repo/src/v2.0.0/src/lib.rs#lines-1"
        );
    }

    #[test]
    fn generic_provider_file_url() {
        let config = CodeUrlConfig::new("https://custom.git.host/repo", "main");
        let url = config.file_url("src/lib.rs", Some(42), None);
        // Generic provider should use GitHub-like format
        assert_eq!(url, "https://custom.git.host/repo/blob/main/src/lib.rs#L42");
    }

    #[test]
    fn generic_provider_commit_url() {
        let config = CodeUrlConfig::new("https://custom.git.host/repo", "main");
        let url = config.commit_url("abc123");
        assert_eq!(url, "https://custom.git.host/repo/commit/abc123");
    }

    #[test]
    fn code_url_builder_new() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
        let builder = CodeUrlBuilder::new(config);
        let url = builder.file_url("src/lib.rs");
        assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs");
    }

    #[test]
    fn line_number_one() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
        let url = config.file_url("src/lib.rs", Some(1), None);
        assert_eq!(url, "https://github.com/user/repo/blob/main/src/lib.rs#L1");
    }

    #[test]
    fn large_line_number() {
        let config = CodeUrlConfig::new("https://github.com/user/repo", "main");
        let url = config.file_url("src/lib.rs", Some(999999), None);
        assert_eq!(
            url,
            "https://github.com/user/repo/blob/main/src/lib.rs#L999999"
        );
    }

    #[test]
    fn special_chars_in_path() {
        let config =
            CodeUrlConfig::new("https://dev.azure.com/org/project/_git/repo", "main");
        // Azure DevOps uses query params, so path should be URL-encoded (but not slashes)
        let url = config.file_url("src/my file.rs", None, None);
        assert_eq!(
            url,
            "https://dev.azure.com/org/project/_git/repo?path=src/my%20file.rs&version=main"
        );
    }

    #[test]
    fn visualstudio_url_detection() {
        // Visual Studio Team Services URLs should be detected as Azure DevOps
        assert_eq!(
            UrlProvider::from_url("https://company.visualstudio.com/project/_git/repo"),
            UrlProvider::AzureDevOps
        );
    }
}
