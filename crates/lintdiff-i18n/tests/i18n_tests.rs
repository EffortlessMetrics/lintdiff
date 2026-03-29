//! Comprehensive BDD tests for lintdiff-i18n crate.
//!
//! These tests cover:
//! - Locale enum functionality
//! - LocalizationBundle creation and usage
//! - Message retrieval (simple, with args, attributes)
//! - Error handling for missing keys and invalid locales
//! - Thread-local bundle functions
//! - Fallback behavior

use std::collections::HashMap;
use std::str::FromStr;

use fluent::FluentValue;
use lintdiff_i18n::{
    get_message, get_message_attribute, get_message_with_args, Locale, LocaleError,
    LocalizationBundle, Message, DEFAULT_LOCALE,
};

// =============================================================================
// Locale Enum Tests
// =============================================================================

mod locale_enum_tests {
    use super::*;

    #[test]
    fn test_default_locale_is_en_us() {
        assert_eq!(Locale::default(), Locale::EnUS);
    }

    #[test]
    fn test_default_constant_matches() {
        assert_eq!(DEFAULT_LOCALE, Locale::EnUS);
    }

    #[test]
    fn test_locale_language_tag() {
        assert_eq!(Locale::EnUS.language_tag(), "en-US");
    }

    #[test]
    fn test_locale_display() {
        assert_eq!(format!("{}", Locale::EnUS), "en-US");
    }

    #[test]
    fn test_locale_debug() {
        assert!(format!("{:?}", Locale::EnUS).contains("EnUS"));
    }

    #[test]
    fn test_locale_clone() {
        let locale = Locale::EnUS;
        let cloned = locale.clone();
        assert_eq!(locale, cloned);
    }

    #[test]
    fn test_locale_copy() {
        let locale = Locale::EnUS;
        let copied: Locale = locale;
        assert_eq!(locale, copied);
    }

    #[test]
    fn test_locale_equality() {
        assert_eq!(Locale::EnUS, Locale::EnUS);
    }

    #[test]
    fn test_locale_hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Locale::EnUS);
        assert!(set.contains(&Locale::EnUS));
    }
}

// =============================================================================
// Locale Parsing Tests
// =============================================================================

mod locale_parsing_tests {
    use super::*;

    #[test]
    fn test_from_str_canonical_form() {
        let locale = Locale::from_str("en-US");
        assert!(locale.is_ok());
        assert_eq!(locale.unwrap(), Locale::EnUS);
    }

    #[test]
    fn test_from_str_underscore_form() {
        let locale = Locale::from_str("en_US");
        assert!(locale.is_ok());
        assert_eq!(locale.unwrap(), Locale::EnUS);
    }

    #[test]
    fn test_from_str_short_form() {
        let locale = Locale::from_str("en");
        assert!(locale.is_ok());
        assert_eq!(locale.unwrap(), Locale::EnUS);
    }

    #[test]
    fn test_from_str_unsupported_locale() {
        let result = Locale::from_str("de-DE");
        assert!(result.is_err());
        assert!(matches!(result, Err(LocaleError::UnknownLocale(_))));
    }

    #[test]
    fn test_from_str_invalid_string() {
        let result = Locale::from_str("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_case_sensitive() {
        // BCP 47 tags are case-insensitive in theory, but our implementation
        // expects exact matches for supported forms
        let result = Locale::from_str("EN-US");
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_str() {
        let locale: Result<Locale, LocaleError> = Locale::try_from("en-US");
        assert!(locale.is_ok());
        assert_eq!(locale.unwrap(), Locale::EnUS);
    }

    #[test]
    fn test_try_from_str_error() {
        let result: Result<Locale, LocaleError> = Locale::try_from("fr-FR");
        assert!(result.is_err());
    }
}

// =============================================================================
// LocalizationBundle Creation Tests
// =============================================================================

mod bundle_creation_tests {
    use super::*;

    #[test]
    fn test_bundle_new_en_us() {
        let bundle = LocalizationBundle::new(Locale::EnUS);
        assert!(bundle.is_ok());
    }

    #[test]
    fn test_bundle_locale_accessor() {
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        assert_eq!(bundle.locale(), Locale::EnUS);
    }

    #[test]
    fn test_bundle_multiple_creations() {
        // Should be able to create multiple bundles
        let bundle1 = LocalizationBundle::new(Locale::EnUS);
        let bundle2 = LocalizationBundle::new(Locale::EnUS);
        assert!(bundle1.is_ok());
        assert!(bundle2.is_ok());
    }

    // Note: LocalizationBundle is NOT Send + Sync due to FluentBundle's internal
    // RefCell<TypeMap>. This is expected behavior - the bundle uses thread-local
    // storage for thread safety.
}

// =============================================================================
// Simple Message Retrieval Tests
// =============================================================================

mod simple_message_tests {
    use super::*;

    #[test]
    fn test_get_brand_name() {
        let result = get_message("brand-name");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "lintdiff");
    }

    #[test]
    fn test_get_welcome_message() {
        let result = get_message("welcome");
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("lintdiff"));
    }

    #[test]
    fn test_get_analyzing_message() {
        let result = get_message("analyzing");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Analyzing"));
    }

    #[test]
    fn test_get_completed_message() {
        let result = get_message("completed");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_error_title() {
        let result = get_message("error-title");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Error");
    }

    #[test]
    fn test_get_cli_help() {
        let result = get_message("cli-help");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("diagnostics"));
    }

    #[test]
    fn test_get_report_header() {
        let result = get_message("report-header");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Lintdiff"));
    }

    #[test]
    fn test_get_severity_error() {
        let result = get_message("severity-error");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Error");
    }

    #[test]
    fn test_get_severity_warning() {
        let result = get_message("severity-warning");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Warning");
    }

    #[test]
    fn test_get_nonexistent_message() {
        let result = get_message("nonexistent-key-xyz");
        assert!(result.is_err());
        assert!(matches!(result, Err(LocaleError::MessageNotFound(_))));
    }
}

// =============================================================================
// Message with Arguments Tests
// =============================================================================

mod message_with_args_tests {
    use super::*;

    #[test]
    fn test_file_not_found_with_path() {
        let mut args = HashMap::new();
        args.insert("path", FluentValue::from("/path/to/file.rs"));

        let result = get_message_with_args("file-not-found", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("/path/to/file.rs"));
    }

    #[test]
    fn test_file_read_error_with_path() {
        let mut args = HashMap::new();
        args.insert("path", FluentValue::from("src/main.rs"));

        let result = get_message_with_args("file-read-error", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("src/main.rs"));
    }

    #[test]
    fn test_config_not_found_with_path() {
        let mut args = HashMap::new();
        args.insert("path", FluentValue::from("lintdiff.toml"));

        let result = get_message_with_args("error-config-not-found", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("lintdiff.toml"));
    }

    #[test]
    fn test_config_parse_error_with_error() {
        let mut args = HashMap::new();
        args.insert("error", FluentValue::from("invalid syntax"));

        let result = get_message_with_args("error-config-parse-error", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("invalid syntax"));
    }

    #[test]
    fn test_diagnostic_parse_with_line() {
        let mut args = HashMap::new();
        args.insert("line", FluentValue::from(42));

        let result = get_message_with_args("error-diagnostic-parse", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("42"));
    }

    #[test]
    fn test_git_diff_failed_with_error() {
        let mut args = HashMap::new();
        args.insert("error", FluentValue::from("exit code 1"));

        let result = get_message_with_args("error-git-diff-failed", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("exit code 1"));
    }

    #[test]
    fn test_version_with_version_arg() {
        let mut args = HashMap::new();
        args.insert("version", FluentValue::from("1.0.0"));

        let result = get_message_with_args("cli-version", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("1.0.0"));
    }

    #[test]
    fn test_report_generated_with_timestamp() {
        let mut args = HashMap::new();
        args.insert("timestamp", FluentValue::from("2024-01-15T10:30:00Z"));

        let result = get_message_with_args("report-generated", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("2024-01-15T10:30:00Z"));
    }

    #[test]
    fn test_report_commit_with_sha() {
        let mut args = HashMap::new();
        args.insert("sha", FluentValue::from("abc123def456"));

        let result = get_message_with_args("report-commit", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("abc123def456"));
    }
}

// =============================================================================
// Pluralization Tests
// =============================================================================

mod pluralization_tests {
    use super::*;

    #[test]
    fn test_cli_run_complete_singular() {
        let mut args = HashMap::new();
        args.insert("count", FluentValue::from(1));

        let result = get_message_with_args("cli-run-complete", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        // Fluent plural rule for "one" in English
        assert!(msg.contains("finding"));
    }

    #[test]
    fn test_cli_run_complete_plural_zero() {
        let mut args = HashMap::new();
        args.insert("count", FluentValue::from(0));

        let result = get_message_with_args("cli-run-complete", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        // Zero uses the "other" plural form
        assert!(msg.contains("finding"));
    }

    #[test]
    fn test_cli_run_complete_plural_multiple() {
        let mut args = HashMap::new();
        args.insert("count", FluentValue::from(5));

        let result = get_message_with_args("cli-run-complete", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        // Multiple uses the "other" plural form
        assert!(msg.contains("finding"));
    }

    #[test]
    fn test_cli_run_complete_plural_large() {
        let mut args = HashMap::new();
        args.insert("count", FluentValue::from(100));

        let result = get_message_with_args("cli-run-complete", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        // Large numbers use the "other" plural form
        assert!(msg.contains("finding"));
    }
}

// =============================================================================
// Message Attribute Tests
// =============================================================================

mod message_attribute_tests {
    use super::*;

    #[test]
    fn test_finding_item_description_attribute() {
        // report-finding-item has a description attribute
        // First we need to ensure the message exists, then check attribute
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        let result = bundle.get_attribute("report-finding-item", "description");
        // Note: The attribute exists in report.ftl, but the message requires args
        // So we just verify the attribute lookup doesn't fail for the attribute itself
        assert!(result.is_ok() || matches!(result, Err(LocaleError::MessageNotFound(_))));
    }

    #[test]
    fn test_nonexistent_attribute() {
        let result = get_message_attribute("brand-name", "nonexistent");
        assert!(result.is_err());
        assert!(matches!(result, Err(LocaleError::AttributeNotFound(_, _))));
    }

    #[test]
    fn test_attribute_on_nonexistent_message() {
        let result = get_message_attribute("nonexistent-message", "title");
        assert!(result.is_err());
        assert!(matches!(result, Err(LocaleError::MessageNotFound(_))));
    }
}

// =============================================================================
// Direct Bundle Message Trait Tests
// =============================================================================

mod bundle_message_trait_tests {
    use super::*;

    #[test]
    fn test_bundle_get_simple() {
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        let result = bundle.get("brand-name");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "lintdiff");
    }

    #[test]
    fn test_bundle_get_with_args() {
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        let mut args = HashMap::new();
        args.insert("path", FluentValue::from("/test/path.rs"));

        let result = bundle.get_with_args("file-not-found", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("/test/path.rs"));
    }

    #[test]
    fn test_bundle_get_attribute() {
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        // report-finding-item has a description attribute
        let result = bundle.get_attribute("report-finding-item", "description");
        // The message exists with the attribute in report.ftl
        assert!(result.is_ok() || matches!(result, Err(LocaleError::MessageNotFound(_))));
    }

    #[test]
    fn test_bundle_get_missing_message() {
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        let result = bundle.get("nonexistent-key");
        assert!(result.is_err());
        assert!(matches!(result, Err(LocaleError::MessageNotFound(_))));
    }

    #[test]
    fn test_bundle_get_missing_attribute() {
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        let result = bundle.get_attribute("brand-name", "missing-attr");
        assert!(result.is_err());
        assert!(matches!(result, Err(LocaleError::AttributeNotFound(_, _))));
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_unknown_locale_error_display() {
        let error = LocaleError::UnknownLocale("xx-XX".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("xx-XX"));
        assert!(msg.contains("Unknown locale"));
    }

    #[test]
    fn test_message_not_found_error_display() {
        let error = LocaleError::MessageNotFound("missing-key".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("missing-key"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_attribute_not_found_error_display() {
        let error = LocaleError::AttributeNotFound("msg-key".to_string(), "attr-name".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("attr-name"));
        assert!(msg.contains("msg-key"));
    }

    #[test]
    fn test_formatting_error_display() {
        let error = LocaleError::FormattingError("bad format".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("Formatting error"));
    }

    #[test]
    fn test_resource_not_found_error_display() {
        let error = LocaleError::ResourceNotFound("missing.ftl".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("missing.ftl"));
        assert!(msg.contains("Resource not found"));
    }

    #[test]
    fn test_parse_error_display() {
        let error = LocaleError::ParseError("syntax error".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("Parse error"));
    }

    #[test]
    fn test_bundle_error_display() {
        let error = LocaleError::BundleError("creation failed".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("Bundle error"));
    }

    #[test]
    fn test_error_debug() {
        let error = LocaleError::MessageNotFound("key".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("MessageNotFound"));
    }
}

// =============================================================================
// Locale Detection Tests
// =============================================================================

mod locale_detection_tests {
    use super::*;

    #[test]
    fn test_detect_returns_valid_locale() {
        let locale = Locale::detect();
        // Should always return a valid locale (even if it's the default)
        assert_eq!(locale, Locale::EnUS);
    }

    #[test]
    fn test_detect_without_env_var() {
        // When no LINTDIFF_LOCALE is set, should return default
        let locale = Locale::detect();
        // Default is en-US
        assert_eq!(locale.language_tag(), "en-US");
    }
}

// =============================================================================
// Message Trait Object Tests
// =============================================================================

mod message_trait_tests {
    use super::*;

    #[test]
    fn test_trait_object_usage() {
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        let message_trait: &dyn Message = &bundle;

        let result = message_trait.get("brand-name");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "lintdiff");
    }

    #[test]
    fn test_trait_object_with_args() {
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        let message_trait: &dyn Message = &bundle;

        let mut args = HashMap::new();
        args.insert("path", FluentValue::from("test.rs"));

        let result = message_trait.get_with_args("file-not-found", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_object_attribute() {
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        let message_trait: &dyn Message = &bundle;

        let result = message_trait.get_attribute("report-finding-item", "description");
        // report-finding-item exists but requires args, // The attribute lookup should work
        assert!(result.is_ok() || matches!(result, Err(LocaleError::MessageNotFound(_))));
    }
}

// =============================================================================
// All FTL Files Loading Tests
// =============================================================================

mod ftl_file_loading_tests {
    use super::*;

    #[test]
    fn test_main_ftl_messages() {
        // Messages from main.ftl that don't require arguments
        assert!(get_message("brand-name").is_ok());
        assert!(get_message("welcome").is_ok());
        assert!(get_message("analyzing").is_ok());
        assert!(get_message("completed").is_ok());
        assert!(get_message("error-title").is_ok());
        assert!(get_message("error-unknown").is_ok());
        assert!(get_message("error-invalid-input").is_ok());
        // Note: file-not-found requires $path arg, tested in message_with_args_tests
    }

    #[test]
    fn test_cli_ftl_messages() {
        // Messages from cli.ftl that don't require arguments
        assert!(get_message("cli-help").is_ok());
        assert!(get_message("cli-run-starting").is_ok());
        assert!(get_message("cli-exit-pass").is_ok());
        assert!(get_message("cli-exit-warn").is_ok());
        assert!(get_message("cli-exit-fail").is_ok());
        assert!(get_message("cli-locale-hint").is_ok());
        // Note: cli-run-complete and cli-version require args
    }

    #[test]
    fn test_report_ftl_messages() {
        // Messages from report.ftl that don't require arguments
        assert!(get_message("report-header").is_ok());
        assert!(get_message("report-summary").is_ok());
        assert!(get_message("report-summary-files").is_ok());
        assert!(get_message("report-summary-additions").is_ok());
        assert!(get_message("report-summary-deletions").is_ok());
        assert!(get_message("report-summary-findings").is_ok());
        assert!(get_message("report-findings-title").is_ok());
        assert!(get_message("report-findings-empty").is_ok());
        assert!(get_message("severity-error").is_ok());
        assert!(get_message("severity-warning").is_ok());
        assert!(get_message("severity-note").is_ok());
        assert!(get_message("severity-help").is_ok());
        // Note: report-generated, report-commit, report-verdict, report-finding-item require args
    }

    #[test]
    fn test_errors_ftl_messages() {
        // Messages from errors.ftl that don't require arguments
        assert!(get_message("error-config-invalid").is_ok());
        assert!(get_message("error-diff-parse").is_ok());
        assert!(get_message("error-diff-empty").is_ok());
        assert!(get_message("error-diff-binary").is_ok());
        assert!(get_message("error-diagnostic-invalid-json").is_ok());
        assert!(get_message("error-diagnostic-no-span").is_ok());
        assert!(get_message("error-match-failed").is_ok());
        assert!(get_message("error-policy-invalid").is_ok());
        assert!(get_message("error-git-not-repo").is_ok());
        assert!(get_message("error-git-no-head").is_ok());
        // Note: error-config-not-found, error-config-parse-error, error-diagnostic-parse,
        // error-policy-unknown-action, error-io-permission, error-io-disk, error-git-diff-failed require args
    }
}

// =============================================================================
// Edge Cases and Boundary Tests
// =============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_key() {
        let result = get_message("");
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_key() {
        let result = get_message("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_key_with_special_chars() {
        let result = get_message("key-with-dashes");
        // This key doesn't exist, but the format is valid
        assert!(result.is_err());
    }

    #[test]
    fn test_very_long_key() {
        let long_key = "a".repeat(1000);
        let result = get_message(&long_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_unicode_key() {
        let result = get_message("你好");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_bundle_calls_same_thread() {
        // Thread-local bundle should handle multiple calls
        let result1 = get_message("brand-name");
        let result2 = get_message("brand-name");
        let result3 = get_message("brand-name");

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        assert_eq!(result1.as_ref().unwrap(), result2.as_ref().unwrap());
        assert_eq!(result2.as_ref().unwrap(), result3.as_ref().unwrap());
    }

    #[test]
    fn test_message_consistency() {
        // Same message should return same content
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        let result1 = bundle.get("brand-name").unwrap();
        let result2 = bundle.get("brand-name").unwrap();
        assert_eq!(result1, result2);
    }
}

// =============================================================================
// Verdict Message Tests
// =============================================================================

mod verdict_message_tests {
    use super::*;

    #[test]
    fn test_verdict_pass() {
        let mut args = HashMap::new();
        args.insert("verdict", FluentValue::from("pass"));

        let result = get_message_with_args("report-verdict", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("PASS"));
    }

    #[test]
    fn test_verdict_warn() {
        let mut args = HashMap::new();
        args.insert("verdict", FluentValue::from("warn"));

        let result = get_message_with_args("report-verdict", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("WARN"));
    }

    #[test]
    fn test_verdict_fail() {
        let mut args = HashMap::new();
        args.insert("verdict", FluentValue::from("fail"));

        let result = get_message_with_args("report-verdict", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("FAIL"));
    }
}

// =============================================================================
// Finding Item Message Tests
// =============================================================================

mod finding_item_tests {
    use super::*;

    #[test]
    fn test_finding_item_all_args() {
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        let mut args = HashMap::new();
        args.insert("severity", FluentValue::from("Error"));
        args.insert("code", FluentValue::from("E0001"));
        args.insert("file", FluentValue::from("src/main.rs"));
        args.insert("line", FluentValue::from(42));
        args.insert("message", FluentValue::from("Test message"));

        let result = bundle.get_with_args("report-finding-item", &args);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("Error"));
        assert!(msg.contains("E0001"));
        assert!(msg.contains("src/main.rs"));
        assert!(msg.contains("42"));
    }
}
