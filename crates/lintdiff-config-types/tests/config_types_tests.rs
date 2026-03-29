//! Comprehensive tests for lintdiff-config-types.

use lintdiff_config_types::{ConfigParseError, FailOn, FileSource, OutputFormat, SuppressRule};
use std::path::PathBuf;

// =============================================================================
// OutputFormat Tests (12 tests)
// =============================================================================

mod output_format_tests {
    use super::*;

    #[test]
    fn parse_text_format() {
        assert_eq!(OutputFormat::parse("text").unwrap(), OutputFormat::Text);
    }

    #[test]
    fn parse_txt_alias() {
        assert_eq!(OutputFormat::parse("txt").unwrap(), OutputFormat::Text);
    }

    #[test]
    fn parse_plain_alias() {
        assert_eq!(OutputFormat::parse("plain").unwrap(), OutputFormat::Text);
    }

    #[test]
    fn parse_json_format() {
        assert_eq!(OutputFormat::parse("json").unwrap(), OutputFormat::Json);
    }

    #[test]
    fn parse_github_format() {
        assert_eq!(OutputFormat::parse("github").unwrap(), OutputFormat::GitHub);
    }

    #[test]
    fn parse_gh_alias() {
        assert_eq!(OutputFormat::parse("gh").unwrap(), OutputFormat::GitHub);
    }

    #[test]
    fn parse_actions_alias() {
        assert_eq!(OutputFormat::parse("actions").unwrap(), OutputFormat::GitHub);
    }

    #[test]
    fn parse_markdown_format() {
        assert_eq!(OutputFormat::parse("markdown").unwrap(), OutputFormat::Markdown);
    }

    #[test]
    fn parse_md_alias() {
        assert_eq!(OutputFormat::parse("md").unwrap(), OutputFormat::Markdown);
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(OutputFormat::parse("JSON").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::parse("Text").unwrap(), OutputFormat::Text);
        assert_eq!(OutputFormat::parse("MARKDOWN").unwrap(), OutputFormat::Markdown);
    }

    #[test]
    fn extension_for_formats() {
        assert_eq!(OutputFormat::Text.extension(), "txt");
        assert_eq!(OutputFormat::Json.extension(), "json");
        assert_eq!(OutputFormat::GitHub.extension(), "txt");
        assert_eq!(OutputFormat::Markdown.extension(), "md");
    }

    #[test]
    fn is_machine_readable() {
        assert!(!OutputFormat::Text.is_machine_readable());
        assert!(OutputFormat::Json.is_machine_readable());
        assert!(!OutputFormat::GitHub.is_machine_readable());
        assert!(!OutputFormat::Markdown.is_machine_readable());
    }
}

// =============================================================================
// FailOn Tests (10 tests)
// =============================================================================

mod fail_on_tests {
    use super::*;

    #[test]
    fn parse_never() {
        assert_eq!(FailOn::parse("never").unwrap(), FailOn::Never);
    }

    #[test]
    fn parse_none_alias() {
        assert_eq!(FailOn::parse("none").unwrap(), FailOn::Never);
    }

    #[test]
    fn parse_off_alias() {
        assert_eq!(FailOn::parse("off").unwrap(), FailOn::Never);
    }

    #[test]
    fn parse_error() {
        assert_eq!(FailOn::parse("error").unwrap(), FailOn::Error);
    }

    #[test]
    fn parse_errors_alias() {
        assert_eq!(FailOn::parse("errors").unwrap(), FailOn::Error);
    }

    #[test]
    fn parse_warning() {
        assert_eq!(FailOn::parse("warning").unwrap(), FailOn::Warning);
    }

    #[test]
    fn parse_warnings_alias() {
        assert_eq!(FailOn::parse("warnings").unwrap(), FailOn::Warning);
    }

    #[test]
    fn parse_warn_alias() {
        assert_eq!(FailOn::parse("warn").unwrap(), FailOn::Warning);
    }

    #[test]
    fn parse_any() {
        assert_eq!(FailOn::parse("any").unwrap(), FailOn::Any);
    }

    #[test]
    fn min_severity_values() {
        assert_eq!(FailOn::Never.min_severity(), 255);
        assert_eq!(FailOn::Error.min_severity(), 3);
        assert_eq!(FailOn::Warning.min_severity(), 2);
        assert_eq!(FailOn::Any.min_severity(), 0);
    }
}

// =============================================================================
// FileSource Tests (10 tests)
// =============================================================================

mod file_source_tests {
    use super::*;

    #[test]
    fn path_constructor() {
        let source = FileSource::path("/some/path.txt");
        assert!(source.is_path());
        assert!(!source.is_stdin());
    }

    #[test]
    fn inline_constructor() {
        let source = FileSource::inline("content");
        assert!(matches!(source, FileSource::Inline(_)));
        assert!(!source.is_path());
        assert!(!source.is_stdin());
    }

    #[test]
    fn is_stdin_true() {
        assert!(FileSource::Stdin.is_stdin());
    }

    #[test]
    fn is_stdin_false() {
        assert!(!FileSource::path("test").is_stdin());
    }

    #[test]
    fn is_path_true() {
        assert!(FileSource::path("test").is_path());
    }

    #[test]
    fn is_path_false() {
        assert!(!FileSource::Stdin.is_path());
    }

    #[test]
    fn as_path_some() {
        let source = FileSource::path("/test/path");
        let path = source.as_path();
        assert!(path.is_some());
        assert_eq!(path.unwrap(), &PathBuf::from("/test/path"));
    }

    #[test]
    fn as_path_none_for_stdin() {
        let source = FileSource::Stdin;
        assert!(source.as_path().is_none());
    }

    #[test]
    fn from_pathbuf() {
        let path = PathBuf::from("/test/path");
        let source: FileSource = path.clone().into();
        assert_eq!(source.as_path(), Some(&path));
    }

    #[test]
    fn from_str() {
        let source: FileSource = "/test/path".into();
        assert!(source.is_path());
    }
}

// =============================================================================
// SuppressRule Tests (10 tests)
// =============================================================================

mod suppress_rule_tests {
    use super::*;

    #[test]
    fn new_creates_rule_with_name() {
        let rule = SuppressRule::new("test-rule");
        assert_eq!(rule.name, "test-rule");
    }

    #[test]
    fn new_has_no_code() {
        let rule = SuppressRule::new("test");
        assert!(rule.code.is_none());
    }

    #[test]
    fn new_has_no_path() {
        let rule = SuppressRule::new("test");
        assert!(rule.path.is_none());
    }

    #[test]
    fn new_has_no_reason() {
        let rule = SuppressRule::new("test");
        assert!(rule.reason.is_none());
    }

    #[test]
    fn with_code_sets_code() {
        let rule = SuppressRule::new("test").with_code("E001");
        assert_eq!(rule.code, Some("E001".to_string()));
    }

    #[test]
    fn with_path_sets_path() {
        let rule = SuppressRule::new("test").with_path("src/**/*.rs");
        assert_eq!(rule.path, Some("src/**/*.rs".to_string()));
    }

    #[test]
    fn with_reason_sets_reason() {
        let rule = SuppressRule::new("test").with_reason("Known issue");
        assert_eq!(rule.reason, Some("Known issue".to_string()));
    }

    #[test]
    fn builder_chaining() {
        let rule = SuppressRule::new("test")
            .with_code("E001")
            .with_path("src/**/*.rs")
            .with_reason("Known issue");
        assert_eq!(rule.name, "test");
        assert_eq!(rule.code, Some("E001".to_string()));
        assert_eq!(rule.path, Some("src/**/*.rs".to_string()));
        assert_eq!(rule.reason, Some("Known issue".to_string()));
    }

    #[test]
    fn clone_creates_equal_instance() {
        let rule = SuppressRule::new("test").with_code("E001");
        let cloned = rule.clone();
        assert_eq!(rule, cloned);
    }

    #[test]
    fn debug_trait_implemented() {
        let rule = SuppressRule::new("test");
        let debug_str = format!("{rule:?}");
        assert!(debug_str.contains("SuppressRule"));
        assert!(debug_str.contains("test"));
    }
}

// =============================================================================
// Display Implementation Tests (5 tests)
// =============================================================================

mod display_tests {
    use super::*;

    #[test]
    fn display_output_format() {
        assert_eq!(OutputFormat::Text.to_string(), "text");
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::GitHub.to_string(), "github");
        assert_eq!(OutputFormat::Markdown.to_string(), "markdown");
    }

    #[test]
    fn display_fail_on() {
        assert_eq!(FailOn::Never.to_string(), "never");
        assert_eq!(FailOn::Error.to_string(), "error");
        assert_eq!(FailOn::Warning.to_string(), "warning");
        assert_eq!(FailOn::Any.to_string(), "any");
    }

    #[test]
    fn display_config_parse_error_invalid_format() {
        let err = ConfigParseError::InvalidFormat("xyz".to_string());
        assert_eq!(err.to_string(), "Invalid output format: 'xyz'");
    }

    #[test]
    fn display_config_parse_error_invalid_fail_on() {
        let err = ConfigParseError::InvalidFailOn("xyz".to_string());
        assert_eq!(err.to_string(), "Invalid fail-on value: 'xyz'");
    }

    #[test]
    fn display_config_parse_error_invalid_config() {
        let err = ConfigParseError::InvalidConfig("bad config".to_string());
        assert_eq!(err.to_string(), "Invalid configuration: bad config");
    }
}

// =============================================================================
// Error Cases Tests (3 tests)
// =============================================================================

mod error_tests {
    use super::*;

    #[test]
    fn invalid_format_error() {
        let result = OutputFormat::parse("invalid");
        assert!(matches!(result, Err(ConfigParseError::InvalidFormat(_))));
        if let Err(ConfigParseError::InvalidFormat(s)) = result {
            assert_eq!(s, "invalid");
        }
    }

    #[test]
    fn invalid_fail_on_error() {
        let result = FailOn::parse("invalid");
        assert!(matches!(result, Err(ConfigParseError::InvalidFailOn(_))));
        if let Err(ConfigParseError::InvalidFailOn(s)) = result {
            assert_eq!(s, "invalid");
        }
    }

    #[test]
    fn empty_string_is_invalid_format() {
        let result = OutputFormat::parse("");
        assert!(matches!(result, Err(ConfigParseError::InvalidFormat(_))));
    }
}

// =============================================================================
// Default Tests (3 tests)
// =============================================================================

mod default_tests {
    use super::*;

    #[test]
    fn output_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Text);
    }

    #[test]
    fn fail_on_default() {
        assert_eq!(FailOn::default(), FailOn::Error);
    }

    #[test]
    fn file_source_default() {
        assert_eq!(FileSource::default(), FileSource::Stdin);
    }
}

// =============================================================================
// Additional Coverage Tests (7 tests)
// =============================================================================

mod additional_tests {
    use super::*;

    #[test]
    fn output_format_equality() {
        assert_eq!(OutputFormat::Text, OutputFormat::Text);
        assert_ne!(OutputFormat::Text, OutputFormat::Json);
    }

    #[test]
    fn fail_on_equality() {
        assert_eq!(FailOn::Error, FailOn::Error);
        assert_ne!(FailOn::Error, FailOn::Warning);
    }

    #[test]
    fn file_source_equality() {
        let path1 = FileSource::path("/same/path");
        let path2 = FileSource::path("/same/path");
        assert_eq!(path1, path2);
    }

    #[test]
    fn file_source_inline_equality() {
        let inline1 = FileSource::inline("content");
        let inline2 = FileSource::inline("content");
        assert_eq!(inline1, inline2);
    }

    #[test]
    fn output_format_repr_values() {
        assert_eq!(OutputFormat::Text as u8, 0);
        assert_eq!(OutputFormat::Json as u8, 1);
        assert_eq!(OutputFormat::GitHub as u8, 2);
        assert_eq!(OutputFormat::Markdown as u8, 3);
    }

    #[test]
    fn fail_on_repr_values() {
        assert_eq!(FailOn::Never as u8, 0);
        assert_eq!(FailOn::Error as u8, 1);
        assert_eq!(FailOn::Warning as u8, 2);
        assert_eq!(FailOn::Any as u8, 3);
    }

    #[test]
    fn suppress_rule_equality() {
        let rule1 = SuppressRule::new("test").with_code("E001");
        let rule2 = SuppressRule::new("test").with_code("E001");
        assert_eq!(rule1, rule2);
    }
}

// =============================================================================
// Hash Tests (4 tests)
// =============================================================================

mod hash_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn output_format_hashable() {
        let mut set = HashSet::new();
        set.insert(OutputFormat::Text);
        set.insert(OutputFormat::Json);
        assert!(set.contains(&OutputFormat::Text));
        assert!(set.contains(&OutputFormat::Json));
    }

    #[test]
    fn fail_on_hashable() {
        let mut set = HashSet::new();
        set.insert(FailOn::Error);
        set.insert(FailOn::Warning);
        assert!(set.contains(&FailOn::Error));
        assert!(set.contains(&FailOn::Warning));
    }

    #[test]
    fn output_format_dedup_in_hashset() {
        let mut set = HashSet::new();
        set.insert(OutputFormat::Text);
        set.insert(OutputFormat::Text);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn fail_on_dedup_in_hashset() {
        let mut set = HashSet::new();
        set.insert(FailOn::Error);
        set.insert(FailOn::Error);
        assert_eq!(set.len(), 1);
    }
}

// =============================================================================
// Clone Tests (2 tests)
// =============================================================================

mod clone_tests {
    use super::*;

    #[test]
    fn output_format_clone() {
        let format = OutputFormat::Json;
        #[allow(clippy::clone_on_copy)]
        let cloned = format.clone();
        assert_eq!(format, cloned);
    }

    #[test]
    fn fail_on_clone() {
        let fail_on = FailOn::Warning;
        #[allow(clippy::clone_on_copy)]
        let cloned = fail_on.clone();
        assert_eq!(fail_on, cloned);
    }
}

// =============================================================================
// Debug Trait Tests (4 tests)
// =============================================================================

mod debug_tests {
    use super::*;

    #[test]
    fn output_format_debug() {
        let format = OutputFormat::Json;
        let debug = format!("{format:?}");
        assert!(debug.contains("Json"));
    }

    #[test]
    fn fail_on_debug() {
        let fail_on = FailOn::Error;
        let debug = format!("{fail_on:?}");
        assert!(debug.contains("Error"));
    }

    #[test]
    fn file_source_debug() {
        let source = FileSource::Stdin;
        let debug = format!("{source:?}");
        assert!(debug.contains("Stdin"));
    }

    #[test]
    fn config_parse_error_debug() {
        let err = ConfigParseError::InvalidFormat("test".to_string());
        let debug = format!("{err:?}");
        assert!(debug.contains("InvalidFormat"));
    }
}
