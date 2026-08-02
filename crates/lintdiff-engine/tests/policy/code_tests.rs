//! Comprehensive tests for code normalization and handling.
//!
//! These tests cover:
//! - Code normalization (clippy::, rustc, etc.)
//! - URL generation for known lints
//! - Unknown code handling
//! - Edge cases (empty codes, special characters)

use lintdiff_engine::{
    format_level, is_code_allowed, map_level_to_severity, normalize_diagnostic_code,
    DiagnosticLevel,
};
use lintdiff_types::{LintdiffConfig, Severity};

// =============================================================================
// normalize_diagnostic_code tests
// =============================================================================

mod normalize_code {
    use super::*;

    #[test]
    fn none_code_returns_unknown() {
        let (code, url) = normalize_diagnostic_code(None);
        assert_eq!(code, "lintdiff.diagnostic.unknown");
        assert!(url.is_none());
    }

    #[test]
    fn clippy_code_normalizes_correctly() {
        let (code, url) = normalize_diagnostic_code(Some("clippy::needless_borrow"));
        assert_eq!(code, "lintdiff.diagnostic.clippy.needless_borrow");
        assert_eq!(
            url,
            Some(
                "https://rust-lang.github.io/rust-clippy/master/index.html#needless_borrow"
                    .to_string()
            )
        );
    }

    #[test]
    fn clippy_code_with_underscores() {
        let (code, url) = normalize_diagnostic_code(Some("clippy::too_many_arguments"));
        assert_eq!(code, "lintdiff.diagnostic.clippy.too_many_arguments");
        assert!(url.is_some());
    }

    #[test]
    fn clippy_code_with_numbers() {
        let (code, url) = normalize_diagnostic_code(Some("clippy::cast_lossless"));
        assert_eq!(code, "lintdiff.diagnostic.clippy.cast_lossless");
        assert!(url.is_some());
    }

    #[test]
    fn rustc_error_code_e0502() {
        let (code, url) = normalize_diagnostic_code(Some("E0502"));
        assert_eq!(code, "lintdiff.diagnostic.rustc.E0502");
        assert_eq!(
            url,
            Some("https://doc.rust-lang.org/error_codes/E0502.html".to_string())
        );
    }

    #[test]
    fn rustc_error_code_e0425() {
        let (code, url) = normalize_diagnostic_code(Some("E0425"));
        assert_eq!(code, "lintdiff.diagnostic.rustc.E0425");
        assert_eq!(
            url,
            Some("https://doc.rust-lang.org/error_codes/E0425.html".to_string())
        );
    }

    #[test]
    fn rustc_error_code_e0001() {
        let (code, url) = normalize_diagnostic_code(Some("E0001"));
        assert_eq!(code, "lintdiff.diagnostic.rustc.E0001");
        assert!(url.is_some());
    }

    #[test]
    fn rustc_error_code_e9999() {
        let (code, url) = normalize_diagnostic_code(Some("E9999"));
        assert_eq!(code, "lintdiff.diagnostic.rustc.E9999");
        assert!(url.is_some());
    }

    #[test]
    fn non_error_code_becomes_rustc_lint() {
        let (code, url) = normalize_diagnostic_code(Some("unused_variables"));
        assert_eq!(code, "lintdiff.diagnostic.rustc_lint.unused_variables");
        assert!(url.is_none());
    }

    #[test]
    fn code_with_colons_converts_to_dots() {
        let (code, _url) = normalize_diagnostic_code(Some("some::complex::code"));
        assert!(code.contains('.'));
        assert!(!code.contains("::"));
    }

    #[test]
    fn code_with_special_characters_slugifies() {
        let (code, _url) = normalize_diagnostic_code(Some("some-code with spaces"));
        // Special characters should be converted to underscores
        assert!(code.contains('_'));
    }

    #[test]
    fn code_with_uppercase_lowercased() {
        let (code, _url) = normalize_diagnostic_code(Some("Unused_Mut"));
        assert!(code.contains("unused_mut"));
    }

    #[test]
    fn empty_code_string() {
        let (code, _url) = normalize_diagnostic_code(Some(""));
        // Empty string is not a valid rustc error code
        assert_ne!(code, "lintdiff.diagnostic.unknown");
    }

    #[test]
    fn code_with_numbers_only_not_error_code() {
        // "12345" is 5 chars but doesn't start with E
        let (code, url) = normalize_diagnostic_code(Some("12345"));
        assert!(code.starts_with("lintdiff.diagnostic.rustc_lint"));
        assert!(url.is_none());
    }

    #[test]
    fn code_e_too_short_not_error_code() {
        // "E123" is only 4 chars
        let (code, _url) = normalize_diagnostic_code(Some("E123"));
        assert!(code.starts_with("lintdiff.diagnostic.rustc_lint"));
    }

    #[test]
    fn code_e_too_long_not_error_code() {
        // "E123456" is 7 chars
        let (code, _url) = normalize_diagnostic_code(Some("E123456"));
        assert!(code.starts_with("lintdiff.diagnostic.rustc_lint"));
    }

    #[test]
    fn code_e_with_non_digits_not_error_code() {
        // "E12AB" has non-digit characters
        let (code, _url) = normalize_diagnostic_code(Some("E12AB"));
        assert!(code.starts_with("lintdiff.diagnostic.rustc_lint"));
    }

    #[test]
    fn clippy_complex_name() {
        let (code, url) = normalize_diagnostic_code(Some("clippy::doc_markdown"));
        assert_eq!(code, "lintdiff.diagnostic.clippy.doc_markdown");
        assert!(url.is_some());
    }
}

// =============================================================================
// map_level_to_severity tests
// =============================================================================

mod level_to_severity {
    use super::*;

    fn severity_debug_contains(sev: &Severity, expected: &str) -> bool {
        format!("{:?}", sev).to_lowercase().contains(expected)
    }

    #[test]
    fn error_maps_to_error() {
        let sev = map_level_to_severity(&DiagnosticLevel::Error);
        assert!(severity_debug_contains(&sev, "error"));
    }

    #[test]
    fn warning_maps_to_warn() {
        let sev = map_level_to_severity(&DiagnosticLevel::Warning);
        assert!(severity_debug_contains(&sev, "warn"));
    }

    #[test]
    fn note_maps_to_info() {
        let sev = map_level_to_severity(&DiagnosticLevel::Note);
        assert!(severity_debug_contains(&sev, "info"));
    }

    #[test]
    fn help_maps_to_info() {
        let sev = map_level_to_severity(&DiagnosticLevel::Help);
        assert!(severity_debug_contains(&sev, "info"));
    }

    #[test]
    fn other_maps_to_info() {
        let sev = map_level_to_severity(&DiagnosticLevel::Other("custom".to_string()));
        assert!(severity_debug_contains(&sev, "info"));
    }

    #[test]
    fn other_empty_maps_to_info() {
        let sev = map_level_to_severity(&DiagnosticLevel::Other("".to_string()));
        assert!(severity_debug_contains(&sev, "info"));
    }
}

// =============================================================================
// format_level tests
// =============================================================================

mod format_level_tests {
    use super::*;

    #[test]
    fn error_formats_as_error() {
        assert_eq!(format_level(&DiagnosticLevel::Error), "error");
    }

    #[test]
    fn warning_formats_as_warning() {
        assert_eq!(format_level(&DiagnosticLevel::Warning), "warning");
    }

    #[test]
    fn note_formats_as_note() {
        assert_eq!(format_level(&DiagnosticLevel::Note), "note");
    }

    #[test]
    fn help_formats_as_help() {
        assert_eq!(format_level(&DiagnosticLevel::Help), "help");
    }

    #[test]
    fn other_preserves_value() {
        assert_eq!(
            format_level(&DiagnosticLevel::Other("custom-level".to_string())),
            "custom-level"
        );
    }

    #[test]
    fn other_empty_string() {
        assert_eq!(format_level(&DiagnosticLevel::Other("".to_string())), "");
    }
}

// =============================================================================
// is_code_allowed tests
// =============================================================================

mod is_code_allowed_tests {
    use super::*;

    fn make_config_with_allow(allow_codes: Vec<&str>) -> LintdiffConfig {
        let mut cfg = LintdiffConfig::default();
        cfg.filter.allow_codes = allow_codes.iter().map(|s| s.to_string()).collect();
        cfg
    }

    fn make_config_with_suppress(suppress_codes: Vec<&str>) -> LintdiffConfig {
        let mut cfg = LintdiffConfig::default();
        cfg.filter.suppress_codes = suppress_codes.iter().map(|s| s.to_string()).collect();
        cfg
    }

    fn make_config_with_allow_and_suppress(
        allow_codes: Vec<&str>,
        suppress_codes: Vec<&str>,
        deny_codes: Vec<&str>,
    ) -> LintdiffConfig {
        let mut cfg = LintdiffConfig::default();
        cfg.filter.allow_codes = allow_codes.iter().map(|s| s.to_string()).collect();
        cfg.filter.suppress_codes = suppress_codes.iter().map(|s| s.to_string()).collect();
        cfg.filter.deny_codes = deny_codes.iter().map(|s| s.to_string()).collect();
        cfg
    }

    #[test]
    fn empty_config_allows_all() {
        let cfg = LintdiffConfig::default();
        let eff = cfg.effective();
        assert!(is_code_allowed(&eff, "any_code"));
        assert!(is_code_allowed(&eff, "another_code"));
    }

    #[test]
    fn allow_list_only_allows_listed() {
        let cfg = make_config_with_allow(vec!["keep_me", "also_keep"]);
        let eff = cfg.effective();
        assert!(is_code_allowed(&eff, "keep_me"));
        assert!(is_code_allowed(&eff, "also_keep"));
        assert!(!is_code_allowed(&eff, "not_in_list"));
    }

    #[test]
    fn allow_list_is_exact_match() {
        let cfg = make_config_with_allow(vec!["keep"]);
        let eff = cfg.effective();
        assert!(is_code_allowed(&eff, "keep"));
        assert!(!is_code_allowed(&eff, "keep_extra"));
        assert!(!is_code_allowed(&eff, "prefix_keep"));
    }

    #[test]
    fn suppress_list_blocks_code() {
        let cfg = make_config_with_suppress(vec!["block_me"]);
        let eff = cfg.effective();
        assert!(!is_code_allowed(&eff, "block_me"));
        assert!(is_code_allowed(&eff, "allow_me"));
    }

    #[test]
    fn suppress_list_is_exact_match() {
        let cfg = make_config_with_suppress(vec!["block"]);
        let eff = cfg.effective();
        assert!(!is_code_allowed(&eff, "block"));
        assert!(is_code_allowed(&eff, "block_extra"));
        assert!(is_code_allowed(&eff, "prefix_block"));
    }

    #[test]
    fn allow_list_overrides_suppress_list() {
        // When allow list is non-empty, only allow-listed codes pass
        // and suppress list is ignored
        let cfg = make_config_with_allow_and_suppress(
            vec!["keep"],
            vec!["keep"], // This is in both lists
            vec![],
        );
        let eff = cfg.effective();
        // When allow list is non-empty, only codes in allow list pass
        assert!(is_code_allowed(&eff, "keep"));
    }

    #[test]
    fn deny_codes_dont_affect_is_code_allowed() {
        // deny_codes are tracked separately and don't affect is_code_allowed
        let cfg = make_config_with_allow_and_suppress(vec![], vec![], vec!["deny_me"]);
        let eff = cfg.effective();
        // deny_codes don't affect is_code_allowed directly
        assert!(is_code_allowed(&eff, "deny_me"));
    }

    #[test]
    fn multiple_suppress_codes() {
        let cfg = make_config_with_suppress(vec!["block1", "block2", "block3"]);
        let eff = cfg.effective();
        assert!(!is_code_allowed(&eff, "block1"));
        assert!(!is_code_allowed(&eff, "block2"));
        assert!(!is_code_allowed(&eff, "block3"));
        assert!(is_code_allowed(&eff, "allow_me"));
    }

    #[test]
    fn empty_string_code() {
        let cfg = LintdiffConfig::default();
        let eff = cfg.effective();
        assert!(is_code_allowed(&eff, ""));
    }

    #[test]
    fn code_with_special_characters() {
        let cfg = make_config_with_suppress(vec!["lintdiff.diagnostic.clippy.needless_borrow"]);
        let eff = cfg.effective();
        assert!(!is_code_allowed(
            &eff,
            "lintdiff.diagnostic.clippy.needless_borrow"
        ));
        assert!(is_code_allowed(&eff, "lintdiff.diagnostic.clippy.other"));
    }
}

// =============================================================================
// Edge cases and boundary conditions
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn very_long_code_name() {
        let long_code = "a".repeat(1000);
        let (code, _url) = normalize_diagnostic_code(Some(&long_code));
        assert!(code.starts_with("lintdiff.diagnostic.rustc_lint."));
    }

    #[test]
    fn unicode_in_code() {
        let (code, _url) = normalize_diagnostic_code(Some("code_with_unicode_αβγ"));
        // Unicode characters get slugified
        assert!(!code.is_empty());
    }

    #[test]
    fn code_with_newlines() {
        let (code, _url) = normalize_diagnostic_code(Some("code\nwith\nnewlines"));
        // Newlines should be handled
        assert!(!code.is_empty());
    }

    #[test]
    fn code_with_tabs() {
        let (code, _url) = normalize_diagnostic_code(Some("code\twith\ttabs"));
        assert!(!code.is_empty());
    }

    #[test]
    fn clippy_empty_after_prefix() {
        let (code, _url) = normalize_diagnostic_code(Some("clippy::"));
        // Should handle gracefully
        assert!(code.starts_with("lintdiff.diagnostic.clippy"));
    }

    #[test]
    fn multiple_double_colons() {
        let (code, _url) = normalize_diagnostic_code(Some("clippy::complex::nested::code"));
        // Should handle multiple colons
        assert!(!code.is_empty());
    }

    #[test]
    fn code_with_dots() {
        let (code, _url) = normalize_diagnostic_code(Some("some.code.with.dots"));
        assert!(!code.is_empty());
    }
}
