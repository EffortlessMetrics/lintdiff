//! Comprehensive tests for lintdiff-config crate.

use std::io::Write;
use std::path::PathBuf;

use lintdiff_config::{ConfigError, FailOn, LintdiffConfig, OutputFormat};
use tempfile::NamedTempFile;

// =============================================================================
// Default Configuration Tests
// =============================================================================

#[test]
fn test_default_config_has_correct_defaults() {
    let config = LintdiffConfig::new();

    assert_eq!(config.fail_on, FailOn::Error);
    assert_eq!(config.output, OutputFormat::Json);
    assert!(config.suppress.is_empty());
    assert!(config.deny.is_empty());
    assert!(config.workspace_only);
}

#[test]
fn test_default_trait_matches_new() {
    let config_new = LintdiffConfig::new();
    let config_default = LintdiffConfig::default();

    assert_eq!(config_new.fail_on, config_default.fail_on);
    assert_eq!(config_new.output, config_default.output);
    assert_eq!(config_new.suppress, config_default.suppress);
    assert_eq!(config_new.deny, config_default.deny);
    assert_eq!(config_new.workspace_only, config_default.workspace_only);
}

// =============================================================================
// FailOn Tests
// =============================================================================

#[test]
fn test_fail_on_default_is_error() {
    assert_eq!(FailOn::default(), FailOn::Error);
    assert_eq!(FailOn::default(), FailOn::Error);
}

#[test]
fn test_fail_on_from_str_valid() {
    assert_eq!("error".parse::<FailOn>().unwrap(), FailOn::Error);
    assert_eq!("ERROR".parse::<FailOn>().unwrap(), FailOn::Error);
    assert_eq!("Error".parse::<FailOn>().unwrap(), FailOn::Error);

    assert_eq!("warning".parse::<FailOn>().unwrap(), FailOn::Warning);
    assert_eq!("warn".parse::<FailOn>().unwrap(), FailOn::Warning);
    assert_eq!("WARNING".parse::<FailOn>().unwrap(), FailOn::Warning);
    assert_eq!("WARN".parse::<FailOn>().unwrap(), FailOn::Warning);

    assert_eq!("note".parse::<FailOn>().unwrap(), FailOn::Note);
    assert_eq!("NOTE".parse::<FailOn>().unwrap(), FailOn::Note);
}

#[test]
fn test_fail_on_from_str_invalid() {
    let result = "invalid".parse::<FailOn>();
    assert!(result.is_err());

    let result = "critical".parse::<FailOn>();
    assert!(result.is_err());

    let result = "".parse::<FailOn>();
    assert!(result.is_err());
}

#[test]
fn test_fail_on_display() {
    assert_eq!(FailOn::Error.to_string(), "error");
    assert_eq!(FailOn::Warning.to_string(), "warning");
    assert_eq!(FailOn::Note.to_string(), "note");
}

#[test]
fn test_fail_on_serde_roundtrip() {
    let original = FailOn::Warning;
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: FailOn = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

// =============================================================================
// OutputFormat Tests
// =============================================================================

#[test]
fn test_output_format_default_is_json() {
    assert_eq!(OutputFormat::default(), OutputFormat::Json);
}

#[test]
fn test_output_format_from_str_valid() {
    assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
    assert_eq!("JSON".parse::<OutputFormat>().unwrap(), OutputFormat::Json);

    assert_eq!(
        "markdown".parse::<OutputFormat>().unwrap(),
        OutputFormat::Markdown
    );
    assert_eq!(
        "md".parse::<OutputFormat>().unwrap(),
        OutputFormat::Markdown
    );
    assert_eq!(
        "MARKDOWN".parse::<OutputFormat>().unwrap(),
        OutputFormat::Markdown
    );

    assert_eq!(
        "annotations".parse::<OutputFormat>().unwrap(),
        OutputFormat::Annotations
    );
    assert_eq!(
        "github".parse::<OutputFormat>().unwrap(),
        OutputFormat::Annotations
    );
    assert_eq!(
        "GITHUB".parse::<OutputFormat>().unwrap(),
        OutputFormat::Annotations
    );

    assert_eq!("text".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
    assert_eq!("TEXT".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
}

#[test]
fn test_output_format_from_str_invalid() {
    let result = "invalid".parse::<OutputFormat>();
    assert!(result.is_err());

    let result = "html".parse::<OutputFormat>();
    assert!(result.is_err());

    let result = "".parse::<OutputFormat>();
    assert!(result.is_err());
}

#[test]
fn test_output_format_display() {
    assert_eq!(OutputFormat::Json.to_string(), "json");
    assert_eq!(OutputFormat::Markdown.to_string(), "markdown");
    assert_eq!(OutputFormat::Annotations.to_string(), "annotations");
    assert_eq!(OutputFormat::Text.to_string(), "text");
}

#[test]
fn test_output_format_serde_roundtrip() {
    let original = OutputFormat::Markdown;
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: OutputFormat = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

// =============================================================================
// TOML Parsing Tests
// =============================================================================

#[test]
fn test_from_toml_empty() {
    let config = LintdiffConfig::from_toml("").unwrap();
    assert_eq!(config.fail_on, FailOn::Error);
    assert_eq!(config.output, OutputFormat::Json);
}

#[test]
fn test_from_toml_full() {
    let toml = r#"
        fail_on = "warning"
        suppress = ["unused_variables", "dead_code", "unused_imports"]
        deny = ["unsafe_code", "deprecated"]
        workspace_only = false
        output = "annotations"
    "#;
    let config = LintdiffConfig::from_toml(toml).unwrap();

    assert_eq!(config.fail_on, FailOn::Warning);
    assert_eq!(
        config.suppress,
        vec!["unused_variables", "dead_code", "unused_imports"]
    );
    assert_eq!(config.deny, vec!["unsafe_code", "deprecated"]);
    assert!(!config.workspace_only);
    assert_eq!(config.output, OutputFormat::Annotations);
}

#[test]
fn test_from_toml_partial() {
    let toml = r#"
        fail_on = "note"
        output = "text"
    "#;
    let config = LintdiffConfig::from_toml(toml).unwrap();

    assert_eq!(config.fail_on, FailOn::Note);
    assert!(config.suppress.is_empty());
    assert!(config.deny.is_empty());
    assert!(config.workspace_only); // default
    assert_eq!(config.output, OutputFormat::Text);
}

#[test]
fn test_from_toml_only_suppress() {
    let toml = r#"
        suppress = ["clippy::all"]
    "#;
    let config = LintdiffConfig::from_toml(toml).unwrap();

    assert_eq!(config.suppress, vec!["clippy::all"]);
    assert_eq!(config.fail_on, FailOn::Error); // default
}

#[test]
fn test_from_toml_only_deny() {
    let toml = r#"
        deny = ["unsafe_code"]
    "#;
    let config = LintdiffConfig::from_toml(toml).unwrap();

    assert_eq!(config.deny, vec!["unsafe_code"]);
    assert_eq!(config.fail_on, FailOn::Error); // default
}

#[test]
fn test_from_toml_invalid_syntax() {
    let toml = r#"
        fail_on = "warning
        # missing closing quote
    "#;
    let result = LintdiffConfig::from_toml(toml);
    assert!(result.is_err());

    match result {
        Err(ConfigError::ParseError(_)) => {}
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_from_toml_invalid_fail_on() {
    let toml = r#"
        fail_on = "critical"
    "#;
    let result = LintdiffConfig::from_toml(toml);
    assert!(result.is_err());
}

#[test]
fn test_from_toml_invalid_output() {
    let toml = r#"
        output = "xml"
    "#;
    let result = LintdiffConfig::from_toml(toml);
    assert!(result.is_err());
}

// =============================================================================
// File Loading Tests
// =============================================================================

#[test]
fn test_from_file_valid() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let content = r#"
        fail_on = "warning"
        suppress = ["unused"]
        output = "markdown"
    "#;
    temp_file.write_all(content.as_bytes()).unwrap();

    let config = LintdiffConfig::from_file(temp_file.path()).unwrap();
    assert_eq!(config.fail_on, FailOn::Warning);
    assert_eq!(config.suppress, vec!["unused"]);
    assert_eq!(config.output, OutputFormat::Markdown);
}

#[test]
fn test_from_file_nonexistent() {
    let result = LintdiffConfig::from_file(PathBuf::from("nonexistent_file.toml").as_path());
    assert!(result.is_err());

    match result {
        Err(ConfigError::IoError(_)) => {}
        _ => panic!("Expected IoError"),
    }
}

// =============================================================================
// Validation Tests
// =============================================================================

#[test]
fn test_validate_valid_config() {
    let config = LintdiffConfig::new();
    assert!(config.validate().is_ok());
}

#[test]
fn test_validate_conflict_same_code() {
    let config = LintdiffConfig {
        suppress: vec!["unused".to_string()],
        deny: vec!["unused".to_string()],
        ..LintdiffConfig::new()
    };

    let result = config.validate();
    assert!(result.is_err());

    match result {
        Err(ConfigError::ValidationError(msg)) => {
            assert!(msg.contains("unused"));
            assert!(msg.contains("suppressed and denied"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_validate_conflict_multiple_codes() {
    let config = LintdiffConfig {
        suppress: vec!["unused".to_string(), "dead_code".to_string()],
        deny: vec!["dead_code".to_string(), "unsafe".to_string()],
        ..LintdiffConfig::new()
    };

    let result = config.validate();
    assert!(result.is_err());

    match result {
        Err(ConfigError::ValidationError(msg)) => {
            assert!(msg.contains("dead_code"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_validate_empty_suppress_code() {
    let config = LintdiffConfig {
        suppress: vec![String::new()],
        ..LintdiffConfig::new()
    };

    let result = config.validate();
    assert!(result.is_err());

    match result {
        Err(ConfigError::ValidationError(msg)) => {
            assert!(msg.contains("empty strings"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_validate_whitespace_only_code() {
    let config = LintdiffConfig {
        deny: vec!["   ".to_string()],
        ..LintdiffConfig::new()
    };

    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_no_conflict_different_codes() {
    let config = LintdiffConfig {
        suppress: vec!["unused".to_string()],
        deny: vec!["unsafe".to_string()],
        ..LintdiffConfig::new()
    };

    assert!(config.validate().is_ok());
}

// =============================================================================
// is_suppressed / is_denied Tests
// =============================================================================

#[test]
fn test_is_suppressed_empty_list() {
    let config = LintdiffConfig::new();
    assert!(!config.is_suppressed("any_code"));
}

#[test]
fn test_is_suppressed_found() {
    let config = LintdiffConfig {
        suppress: vec!["unused".to_string(), "dead_code".to_string()],
        ..LintdiffConfig::new()
    };

    assert!(config.is_suppressed("unused"));
    assert!(config.is_suppressed("dead_code"));
}

#[test]
fn test_is_suppressed_not_found() {
    let config = LintdiffConfig {
        suppress: vec!["unused".to_string()],
        ..LintdiffConfig::new()
    };

    assert!(!config.is_suppressed("dead_code"));
    assert!(!config.is_suppressed("unsafe"));
}

#[test]
fn test_is_suppressed_case_sensitive() {
    let config = LintdiffConfig {
        suppress: vec!["Unused".to_string()],
        ..LintdiffConfig::new()
    };

    assert!(config.is_suppressed("Unused"));
    assert!(!config.is_suppressed("unused"));
}

#[test]
fn test_is_denied_empty_list() {
    let config = LintdiffConfig::new();
    assert!(!config.is_denied("any_code"));
}

#[test]
fn test_is_denied_found() {
    let config = LintdiffConfig {
        deny: vec!["unsafe".to_string(), "deprecated".to_string()],
        ..LintdiffConfig::new()
    };

    assert!(config.is_denied("unsafe"));
    assert!(config.is_denied("deprecated"));
}

#[test]
fn test_is_denied_not_found() {
    let config = LintdiffConfig {
        deny: vec!["unsafe".to_string()],
        ..LintdiffConfig::new()
    };

    assert!(!config.is_denied("unused"));
    assert!(!config.is_denied("deprecated"));
}

#[test]
fn test_is_denied_case_sensitive() {
    let config = LintdiffConfig {
        deny: vec!["Unsafe".to_string()],
        ..LintdiffConfig::new()
    };

    assert!(config.is_denied("Unsafe"));
    assert!(!config.is_denied("unsafe"));
}

// =============================================================================
// Serialization Tests
// =============================================================================

#[test]
fn test_serialize_config() {
    let config = LintdiffConfig {
        fail_on: FailOn::Warning,
        suppress: vec!["unused".to_string()],
        deny: vec!["unsafe".to_string()],
        workspace_only: false,
        output: OutputFormat::Markdown,
    };

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("\"fail_on\":\"warning\""));
    assert!(json.contains("\"suppress\":[\"unused\"]"));
    assert!(json.contains("\"deny\":[\"unsafe\"]"));
    assert!(json.contains("\"workspace_only\":false"));
    assert!(json.contains("\"output\":\"markdown\""));
}

#[test]
fn test_deserialize_config() {
    let json = r#"{
        "fail_on": "note",
        "suppress": ["a", "b"],
        "deny": ["c"],
        "workspace_only": true,
        "output": "text"
    }"#;

    let config: LintdiffConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.fail_on, FailOn::Note);
    assert_eq!(config.suppress, vec!["a", "b"]);
    assert_eq!(config.deny, vec!["c"]);
    assert!(config.workspace_only);
    assert_eq!(config.output, OutputFormat::Text);
}

#[test]
fn test_roundtrip_serialization() {
    let original = LintdiffConfig {
        fail_on: FailOn::Warning,
        suppress: vec!["unused".to_string(), "dead_code".to_string()],
        deny: vec!["unsafe".to_string()],
        workspace_only: true,
        output: OutputFormat::Annotations,
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: LintdiffConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(original.fail_on, deserialized.fail_on);
    assert_eq!(original.suppress, deserialized.suppress);
    assert_eq!(original.deny, deserialized.deny);
    assert_eq!(original.workspace_only, deserialized.workspace_only);
    assert_eq!(original.output, deserialized.output);
}

// =============================================================================
// Clippy Code Tests
// =============================================================================

#[test]
fn test_clippy_codes() {
    let config = LintdiffConfig {
        suppress: vec!["clippy::all".to_string(), "clippy::pedantic".to_string()],
        deny: vec!["clippy::unwrap_used".to_string()],
        ..LintdiffConfig::new()
    };

    assert!(config.is_suppressed("clippy::all"));
    assert!(config.is_suppressed("clippy::pedantic"));
    assert!(config.is_denied("clippy::unwrap_used"));
}

#[test]
fn test_rustc_codes() {
    let config = LintdiffConfig {
        suppress: vec!["unused_variables".to_string()],
        deny: vec!["unsafe_code".to_string()],
        ..LintdiffConfig::new()
    };

    assert!(config.is_suppressed("unused_variables"));
    assert!(config.is_denied("unsafe_code"));
}
