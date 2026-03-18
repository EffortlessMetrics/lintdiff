//! Comprehensive tests for configuration types.
//!
//! Tests cover:
//! - FailOn enum parsing, display, and defaults
//! - Profile enum parsing, display, and defaults
//! - FeatureFlags defaults and serialization
//! - FilterConfig defaults and serialization
//! - ProvenanceConfig defaults
//! - LintdiffConfig effective config resolution
//! - Configuration serialization/deserialization

use lintdiff_types::*;

// =============================================================================
// FailOn Tests
// =============================================================================

mod fail_on_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn default_is_error() {
        let default = FailOn::default();
        assert_eq!(default, FailOn::Error);
    }

    #[test]
    fn from_str_valid_values() {
        assert_eq!(FailOn::from_str("error").unwrap(), FailOn::Error);
        assert_eq!(FailOn::from_str("warn").unwrap(), FailOn::Warn);
        assert_eq!(FailOn::from_str("never").unwrap(), FailOn::Never);
    }

    #[test]
    fn from_str_case_insensitive() {
        assert_eq!(FailOn::from_str("ERROR").unwrap(), FailOn::Error);
        assert_eq!(FailOn::from_str("WaRn").unwrap(), FailOn::Warn);
        assert_eq!(FailOn::from_str("NEVER").unwrap(), FailOn::Never);
    }

    #[test]
    fn from_str_invalid_value() {
        assert!(FailOn::from_str("invalid").is_err());
        assert!(FailOn::from_str("").is_err());
        assert!(FailOn::from_str("warning").is_err());
    }

    #[test]
    fn display_format() {
        assert_eq!(format!("{}", FailOn::Error), "error");
        assert_eq!(format!("{}", FailOn::Warn), "warn");
        assert_eq!(format!("{}", FailOn::Never), "never");
    }

    #[test]
    fn serialize_lowercase() {
        let json = serde_json::to_string(&FailOn::Error).unwrap();
        assert_eq!(json, r#""error""#);

        let json = serde_json::to_string(&FailOn::Warn).unwrap();
        assert_eq!(json, r#""warn""#);

        let json = serde_json::to_string(&FailOn::Never).unwrap();
        assert_eq!(json, r#""never""#);
    }

    #[test]
    fn deserialize_lowercase() {
        let value: FailOn = serde_json::from_str(r#""error""#).unwrap();
        assert_eq!(value, FailOn::Error);

        let value: FailOn = serde_json::from_str(r#""warn""#).unwrap();
        assert_eq!(value, FailOn::Warn);

        let value: FailOn = serde_json::from_str(r#""never""#).unwrap();
        assert_eq!(value, FailOn::Never);
    }

    #[test]
    fn clone_and_eq() {
        let a = FailOn::Error;
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_format() {
        let debug = format!("{:?}", FailOn::Error);
        assert!(debug.contains("Error"));
    }
}

// =============================================================================
// Profile Tests
// =============================================================================

mod profile_tests {
    use super::*;

    #[test]
    fn default_is_default() {
        let default = Profile::default();
        assert_eq!(default, Profile::Default);
    }

    #[test]
    fn serialize_lowercase() {
        let json = serde_json::to_string(&Profile::Default).unwrap();
        assert_eq!(json, r#""default""#);

        let json = serde_json::to_string(&Profile::Strict).unwrap();
        assert_eq!(json, r#""strict""#);

        let json = serde_json::to_string(&Profile::Advisory).unwrap();
        assert_eq!(json, r#""advisory""#);
    }

    #[test]
    fn deserialize_lowercase() {
        let value: Profile = serde_json::from_str(r#""default""#).unwrap();
        assert_eq!(value, Profile::Default);

        let value: Profile = serde_json::from_str(r#""strict""#).unwrap();
        assert_eq!(value, Profile::Strict);

        let value: Profile = serde_json::from_str(r#""advisory""#).unwrap();
        assert_eq!(value, Profile::Advisory);
    }

    #[test]
    fn clone_and_eq() {
        let a = Profile::Strict;
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_format() {
        let debug = format!("{:?}", Profile::Strict);
        assert!(debug.contains("Strict"));
    }
}

// =============================================================================
// FeatureFlags Tests
// =============================================================================

mod feature_flags_tests {
    use super::*;

    #[test]
    fn default_values_are_true() {
        let flags = FeatureFlags::default();
        assert!(flags.prefer_primary_spans);
        assert!(flags.path_filters);
    }

    #[test]
    fn serialize_with_defaults() {
        let flags = FeatureFlags::default();
        let json = serde_json::to_string(&flags).unwrap();
        assert!(json.contains("prefer_primary_spans"));
        assert!(json.contains("path_filters"));
    }

    #[test]
    fn deserialize_with_defaults() {
        // Empty object should use defaults
        let flags: FeatureFlags = serde_json::from_str("{}").unwrap();
        assert!(flags.prefer_primary_spans);
        assert!(flags.path_filters);
    }

    #[test]
    fn deserialize_explicit_false() {
        let json = r#"{"prefer_primary_spans":false,"path_filters":false}"#;
        let flags: FeatureFlags = serde_json::from_str(json).unwrap();
        assert!(!flags.prefer_primary_spans);
        assert!(!flags.path_filters);
    }

    #[test]
    fn deserialize_partial() {
        let json = r#"{"prefer_primary_spans":false}"#;
        let flags: FeatureFlags = serde_json::from_str(json).unwrap();
        assert!(!flags.prefer_primary_spans);
        assert!(flags.path_filters); // Should use default
    }

    #[test]
    fn clone() {
        let flags = FeatureFlags::default();
        let cloned = flags.clone();
        assert_eq!(flags.prefer_primary_spans, cloned.prefer_primary_spans);
        assert_eq!(flags.path_filters, cloned.path_filters);
    }

    #[test]
    fn debug_format() {
        let flags = FeatureFlags::default();
        let debug = format!("{:?}", flags);
        assert!(debug.contains("FeatureFlags"));
    }
}

// =============================================================================
// FilterConfig Tests
// =============================================================================

mod filter_config_tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        let config = FilterConfig::default();
        assert!(config.include_paths.is_empty());
        assert!(config.exclude_paths.is_empty());
        assert!(config.allow_codes.is_empty());
        assert!(config.suppress_codes.is_empty());
        assert!(config.deny_codes.is_empty());
    }

    #[test]
    fn serialize_empty() {
        let config = FilterConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("include_paths"));
        assert!(json.contains("exclude_paths"));
    }

    #[test]
    fn deserialize_with_values() {
        let json = r#"{
            "include_paths": ["src/**"],
            "exclude_paths": ["tests/**"],
            "allow_codes": ["E001"],
            "suppress_codes": ["W001"],
            "deny_codes": ["E002"]
        }"#;
        let config: FilterConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.include_paths, vec!["src/**"]);
        assert_eq!(config.exclude_paths, vec!["tests/**"]);
        assert_eq!(config.allow_codes, vec!["E001"]);
        assert_eq!(config.suppress_codes, vec!["W001"]);
        assert_eq!(config.deny_codes, vec!["E002"]);
    }

    #[test]
    fn clone() {
        let config = FilterConfig {
            include_paths: vec!["src/**".to_string()],
            exclude_paths: vec![],
            allow_codes: vec![],
            suppress_codes: vec![],
            deny_codes: vec![],
        };
        let cloned = config.clone();
        assert_eq!(config.include_paths, cloned.include_paths);
    }

    #[test]
    fn debug_format() {
        let config = FilterConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("FilterConfig"));
    }
}

// =============================================================================
// ProvenanceConfig Tests
// =============================================================================

mod provenance_config_tests {
    use super::*;

    #[test]
    fn default_is_false() {
        let config = ProvenanceConfig::default();
        assert!(!config.record_rustc);
        assert!(!config.record_clippy);
    }

    #[test]
    fn serialize() {
        let config = ProvenanceConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("record_rustc"));
        assert!(json.contains("record_clippy"));
    }

    #[test]
    fn deserialize_with_values() {
        let json = r#"{"record_rustc":true,"record_clippy":true}"#;
        let config: ProvenanceConfig = serde_json::from_str(json).unwrap();
        assert!(config.record_rustc);
        assert!(config.record_clippy);
    }

    #[test]
    fn clone() {
        let config = ProvenanceConfig {
            record_rustc: true,
            record_clippy: false,
        };
        let cloned = config.clone();
        assert_eq!(config.record_rustc, cloned.record_rustc);
        assert_eq!(config.record_clippy, cloned.record_clippy);
    }
}

// =============================================================================
// LintdiffConfig Tests
// =============================================================================

mod lintdiff_config_tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        let config = LintdiffConfig::default();
        assert!(config.profile.is_none());
        assert!(config.fail_on.is_none());
        assert!(config.max_findings.is_none());
        assert!(config.max_annotations.is_none());
        assert!(config.workspace_only.is_none());
    }

    #[test]
    fn effective_default_profile() {
        let config = LintdiffConfig::default();
        let effective = config.effective();
        assert_eq!(effective.profile, Profile::Default);
        assert_eq!(effective.fail_on, FailOn::Error);
    }

    #[test]
    fn effective_strict_profile() {
        let config = LintdiffConfig {
            profile: Some(Profile::Strict),
            ..Default::default()
        };
        let effective = config.effective();
        assert_eq!(effective.profile, Profile::Strict);
        assert_eq!(effective.fail_on, FailOn::Warn);
    }

    #[test]
    fn effective_advisory_profile() {
        let config = LintdiffConfig {
            profile: Some(Profile::Advisory),
            ..Default::default()
        };
        let effective = config.effective();
        assert_eq!(effective.profile, Profile::Advisory);
        assert_eq!(effective.fail_on, FailOn::Never);
    }

    #[test]
    fn effective_fail_on_override() {
        let config = LintdiffConfig {
            profile: Some(Profile::Default),
            fail_on: Some(FailOn::Never),
            ..Default::default()
        };
        let effective = config.effective();
        assert_eq!(effective.fail_on, FailOn::Never);
    }

    #[test]
    fn effective_max_findings_default() {
        let config = LintdiffConfig::default();
        let effective = config.effective();
        assert_eq!(effective.max_findings, 200);
    }

    #[test]
    fn effective_max_findings_override() {
        let config = LintdiffConfig {
            max_findings: Some(500),
            ..Default::default()
        };
        let effective = config.effective();
        assert_eq!(effective.max_findings, 500);
    }

    #[test]
    fn effective_max_annotations_default() {
        let config = LintdiffConfig::default();
        let effective = config.effective();
        assert_eq!(effective.max_annotations, 50);
    }

    #[test]
    fn effective_max_annotations_override() {
        let config = LintdiffConfig {
            max_annotations: Some(100),
            ..Default::default()
        };
        let effective = config.effective();
        assert_eq!(effective.max_annotations, 100);
    }

    #[test]
    fn effective_workspace_only_default() {
        let config = LintdiffConfig::default();
        let effective = config.effective();
        assert!(effective.workspace_only);
    }

    #[test]
    fn effective_workspace_only_override() {
        let config = LintdiffConfig {
            workspace_only: Some(false),
            ..Default::default()
        };
        let effective = config.effective();
        assert!(!effective.workspace_only);
    }

    #[test]
    fn effective_copies_filter() {
        let config = LintdiffConfig {
            filter: FilterConfig {
                include_paths: vec!["src/**".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let effective = config.effective();
        assert_eq!(effective.filter.include_paths, vec!["src/**"]);
    }

    #[test]
    fn effective_copies_provenance() {
        let config = LintdiffConfig {
            provenance: ProvenanceConfig {
                record_rustc: true,
                record_clippy: true,
            },
            ..Default::default()
        };
        let effective = config.effective();
        assert!(effective.provenance.record_rustc);
        assert!(effective.provenance.record_clippy);
    }

    #[test]
    fn effective_copies_feature_flags() {
        let config = LintdiffConfig {
            feature_flags: FeatureFlags {
                prefer_primary_spans: false,
                path_filters: false,
            },
            ..Default::default()
        };
        let effective = config.effective();
        assert!(!effective.feature_flags.prefer_primary_spans);
        assert!(!effective.feature_flags.path_filters);
    }

    #[test]
    fn serialize_full_config() {
        let config = LintdiffConfig {
            profile: Some(Profile::Strict),
            fail_on: Some(FailOn::Warn),
            max_findings: Some(100),
            max_annotations: Some(25),
            workspace_only: Some(true),
            filter: FilterConfig::default(),
            provenance: ProvenanceConfig::default(),
            feature_flags: FeatureFlags::default(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("profile"));
        assert!(json.contains("fail_on"));
        assert!(json.contains("max_findings"));
    }

    #[test]
    fn deserialize_full_config() {
        let json = r#"{
            "profile": "strict",
            "fail_on": "warn",
            "max_findings": 100,
            "max_annotations": 25,
            "workspace_only": false
        }"#;
        let config: LintdiffConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.profile, Some(Profile::Strict));
        assert_eq!(config.fail_on, Some(FailOn::Warn));
        assert_eq!(config.max_findings, Some(100));
        assert_eq!(config.max_annotations, Some(25));
        assert_eq!(config.workspace_only, Some(false));
    }

    #[test]
    fn clone() {
        let config = LintdiffConfig {
            profile: Some(Profile::Strict),
            ..Default::default()
        };
        let cloned = config.clone();
        assert_eq!(config.profile, cloned.profile);
    }

    #[test]
    fn debug_format() {
        let config = LintdiffConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("LintdiffConfig"));
    }
}

// =============================================================================
// EffectiveConfig Tests
// =============================================================================

mod effective_config_tests {
    use super::*;

    #[test]
    fn debug_format() {
        let config = LintdiffConfig::default();
        let effective = config.effective();
        let debug = format!("{:?}", effective);
        assert!(debug.contains("EffectiveConfig"));
    }

    #[test]
    fn clone() {
        let config = LintdiffConfig::default();
        let effective = config.effective();
        let cloned = effective.clone();
        assert_eq!(effective.profile, cloned.profile);
        assert_eq!(effective.fail_on, cloned.fail_on);
        assert_eq!(effective.max_findings, cloned.max_findings);
    }
}
