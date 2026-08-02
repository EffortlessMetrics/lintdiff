//! Tests for config file loading functionality.

use std::fs;
use std::io::Write;

use lintdiff::io::{load_config, AppIoError};
use lintdiff_types::{FailOn, Profile};
use tempfile::TempDir;

fn create_config_file(dir: &TempDir, filename: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(filename);
    let mut file = fs::File::create(&path).expect("failed to create config file");
    file.write_all(content.as_bytes())
        .expect("failed to write config");
    path
}

#[test]
fn load_valid_config_file() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
profile = "strict"
fail_on = "warn"
max_findings = 100
max_annotations = 25
workspace_only = false

[filter]
include_paths = ["src/**"]
exclude_paths = ["tests/**"]
allow_codes = ["clippy::all"]
suppress_codes = ["dead_code"]
deny_codes = ["unsafe_code"]

[provenance]
record_rustc = true
record_clippy = true

[feature_flags]
prefer_primary_spans = true
path_filters = false
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.profile, Some(Profile::Strict));
    assert_eq!(config.fail_on, Some(FailOn::Warn));
    assert_eq!(config.max_findings, Some(100));
    assert_eq!(config.max_annotations, Some(25));
    assert_eq!(config.workspace_only, Some(false));

    assert_eq!(config.filter.include_paths, vec!["src/**"]);
    assert_eq!(config.filter.exclude_paths, vec!["tests/**"]);
    assert_eq!(config.filter.allow_codes, vec!["clippy::all"]);
    assert_eq!(config.filter.suppress_codes, vec!["dead_code"]);
    assert_eq!(config.filter.deny_codes, vec!["unsafe_code"]);

    assert!(config.provenance.record_rustc);
    assert!(config.provenance.record_clippy);

    assert!(config.feature_flags.prefer_primary_spans);
    assert!(!config.feature_flags.path_filters);
}

#[test]
fn load_minimal_config_file() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
profile = "advisory"
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.profile, Some(Profile::Advisory));
    // All other fields should be defaults
    assert_eq!(config.fail_on, None);
    assert_eq!(config.max_findings, None);
    assert_eq!(config.max_annotations, None);
    assert_eq!(config.workspace_only, None);
    assert!(config.filter.include_paths.is_empty());
    assert!(config.filter.exclude_paths.is_empty());
    assert!(!config.provenance.record_rustc);
    assert!(!config.provenance.record_clippy);
}

#[test]
fn load_empty_config_file() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = "";
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    // Empty config should give all defaults
    assert_eq!(config.profile, None);
    assert_eq!(config.fail_on, None);
}

#[test]
fn handle_missing_explicit_config_file() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let non_existent = dir.path().join("non_existent.toml");

    let result = load_config(dir.path(), Some(&non_existent));
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        AppIoError::ReadFile { path, source } => {
            assert_eq!(path, non_existent);
            assert!(source.kind() == std::io::ErrorKind::NotFound);
        }
        _ => panic!("expected ReadFile error"),
    }
}

#[test]
fn handle_missing_default_config_returns_defaults() {
    let dir = TempDir::new().expect("failed to create temp dir");
    // No lintdiff.toml created

    let result = load_config(dir.path(), None);
    assert!(result.is_ok());

    let config = result.unwrap();
    // Should return default config
    assert_eq!(config.profile, None);
    assert_eq!(config.fail_on, None);
    assert_eq!(config.max_findings, None);
}

#[test]
fn handle_invalid_toml_syntax() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
profile = "strict"
this is not valid toml [[[
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        AppIoError::ParseConfig { source: _ } => {
            // Expected parse error
        }
        _ => panic!("expected ParseConfig error"),
    }
}

#[test]
fn handle_invalid_config_value() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
profile = "invalid_profile_value"
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        AppIoError::ParseConfig { source: _ } => {
            // Expected parse error for invalid enum value
        }
        _ => panic!("expected ParseConfig error"),
    }
}

#[test]
fn handle_invalid_fail_on_value() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
fail_on = "invalid"
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_err());
}

#[test]
fn load_config_with_partial_filter_section() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
[filter]
include_paths = ["src/**"]
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.filter.include_paths, vec!["src/**"]);
    assert!(config.filter.exclude_paths.is_empty());
    assert!(config.filter.allow_codes.is_empty());
    assert!(config.filter.suppress_codes.is_empty());
    assert!(config.filter.deny_codes.is_empty());
}

#[test]
fn load_config_with_partial_provenance() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
[provenance]
record_rustc = true
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    assert!(config.provenance.record_rustc);
    assert!(!config.provenance.record_clippy);
}

#[test]
fn load_config_with_partial_feature_flags() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
[feature_flags]
prefer_primary_spans = false
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    assert!(!config.feature_flags.prefer_primary_spans);
    // path_filters should still be default true
    assert!(config.feature_flags.path_filters);
}

#[test]
fn load_default_config_from_repo_root() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
profile = "default"
fail_on = "error"
"#;
    // Create lintdiff.toml at repo root (temp dir)
    create_config_file(&dir, "lintdiff.toml", content);

    // Pass None for explicit path - should find lintdiff.toml in repo root
    let result = load_config(dir.path(), None);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.profile, Some(Profile::Default));
    assert_eq!(config.fail_on, Some(FailOn::Error));
}

#[test]
fn explicit_path_takes_precedence_over_repo_root() {
    let dir = TempDir::new().expect("failed to create temp dir");

    // Create default config at repo root
    let default_content = r#"
profile = "default"
"#;
    create_config_file(&dir, "lintdiff.toml", default_content);

    // Create explicit config with different settings
    let explicit_content = r#"
profile = "strict"
"#;
    let explicit_path = create_config_file(&dir, "explicit.toml", explicit_content);

    // Load with explicit path
    let result = load_config(dir.path(), Some(&explicit_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    // Should use explicit config, not repo root default
    assert_eq!(config.profile, Some(Profile::Strict));
}

#[test]
fn config_with_string_max_findings() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
max_findings = 500
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.max_findings, Some(500));
}

#[test]
fn config_with_boolean_workspace_only() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
workspace_only = false
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.workspace_only, Some(false));
}

#[test]
fn config_with_multiple_filter_lists() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
[filter]
include_paths = ["src/**", "lib/**"]
exclude_paths = ["tests/**", "examples/**"]
allow_codes = ["clippy::all", "rustdoc::all"]
suppress_codes = ["dead_code", "unused_variables"]
deny_codes = ["unsafe_code", "unreachable_pub"]
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.filter.include_paths, vec!["src/**", "lib/**"]);
    assert_eq!(config.filter.exclude_paths, vec!["tests/**", "examples/**"]);
    assert_eq!(
        config.filter.allow_codes,
        vec!["clippy::all", "rustdoc::all"]
    );
    assert_eq!(
        config.filter.suppress_codes,
        vec!["dead_code", "unused_variables"]
    );
    assert_eq!(
        config.filter.deny_codes,
        vec!["unsafe_code", "unreachable_pub"]
    );
}

#[test]
fn config_all_profiles() {
    for (profile_str, profile_expected) in [
        ("default", Profile::Default),
        ("strict", Profile::Strict),
        ("advisory", Profile::Advisory),
    ] {
        let dir = TempDir::new().expect("failed to create temp dir");
        let content = format!(r#"profile = "{}""#, profile_str);
        let config_path = create_config_file(&dir, "lintdiff.toml", &content);

        let result = load_config(dir.path(), Some(&config_path));
        assert!(result.is_ok(), "failed to parse profile: {}", profile_str);

        let config = result.unwrap();
        assert_eq!(
            config.profile,
            Some(profile_expected),
            "profile mismatch for: {}",
            profile_str
        );
    }
}

#[test]
fn config_all_fail_on_values() {
    for (fail_on_str, fail_on_expected) in [
        ("error", FailOn::Error),
        ("warn", FailOn::Warn),
        ("never", FailOn::Never),
    ] {
        let dir = TempDir::new().expect("failed to create temp dir");
        let content = format!(r#"fail_on = "{}""#, fail_on_str);
        let config_path = create_config_file(&dir, "lintdiff.toml", &content);

        let result = load_config(dir.path(), Some(&config_path));
        assert!(result.is_ok(), "failed to parse fail_on: {}", fail_on_str);

        let config = result.unwrap();
        assert_eq!(
            config.fail_on,
            Some(fail_on_expected),
            "fail_on mismatch for: {}",
            fail_on_str
        );
    }
}

#[test]
fn error_message_contains_path() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let non_existent = dir.path().join("missing_config.toml");

    let result = load_config(dir.path(), Some(&non_existent));
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing_config.toml"),
        "error message should contain path: {}",
        err_msg
    );
}

#[test]
fn parse_error_message_contains_details() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
[filter
"#; // Missing closing bracket
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("parse") || err_msg.contains("TOML"),
        "error message should mention parsing: {}",
        err_msg
    );
}

#[test]
fn effective_config_from_loaded_config() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
profile = "strict"
max_findings = 100
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    let effective = config.effective();

    assert_eq!(effective.profile, Profile::Strict);
    // Strict profile should default fail_on to Warn
    assert_eq!(effective.fail_on, FailOn::Warn);
    assert_eq!(effective.max_findings, 100);
    // max_annotations should use default
    assert_eq!(effective.max_annotations, 50);
}

#[test]
fn effective_config_advisory_profile() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = r#"
profile = "advisory"
"#;
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    let effective = config.effective();

    assert_eq!(effective.profile, Profile::Advisory);
    // Advisory profile should default fail_on to Never
    assert_eq!(effective.fail_on, FailOn::Never);
}

#[test]
fn effective_config_default_profile() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let content = "";
    let config_path = create_config_file(&dir, "lintdiff.toml", content);

    let result = load_config(dir.path(), Some(&config_path));
    assert!(result.is_ok());

    let config = result.unwrap();
    let effective = config.effective();

    assert_eq!(effective.profile, Profile::Default);
    // Default profile should default fail_on to Error
    assert_eq!(effective.fail_on, FailOn::Error);
    // Check all defaults
    assert_eq!(effective.max_findings, 200);
    assert_eq!(effective.max_annotations, 50);
    assert!(effective.workspace_only);
}
