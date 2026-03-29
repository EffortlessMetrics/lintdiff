//! Comprehensive tests for lintdiff-report-schema.
//!
//! This test suite covers:
//! 1. SchemaVersion creation and parsing (10 tests)
//! 2. SchemaVersion compatibility (5 tests)
//! 3. ValidationError variants (6 tests)
//! 4. ValidationResult methods (8 tests)
//! 5. ReportValidator basic validation (10 tests)
//! 6. ReportValidator required fields (6 tests)
//! 7. current_schema function (5 tests)

use lintdiff_report_schema::{
    current_schema, ReportValidator, SchemaVersion, SchemaVersionParseError, ValidationError,
    ValidationResult,
};
use proptest::prelude::*;

// =============================================================================
// 1. SchemaVersion creation and parsing (10 tests)
// =============================================================================

mod schema_version_creation_and_parsing {
    use super::*;

    #[test]
    fn test_new_creates_version_with_correct_components() {
        let version = SchemaVersion::new(2, 3, 4);
        assert_eq!(version.major(), 2);
        assert_eq!(version.minor(), 3);
        assert_eq!(version.patch(), 4);
    }

    #[test]
    fn test_current_returns_version_1_0_0() {
        let current = SchemaVersion::current();
        assert_eq!(current.major(), 1);
        assert_eq!(current.minor(), 0);
        assert_eq!(current.patch(), 0);
    }

    #[test]
    fn test_default_returns_current_version() {
        let default = SchemaVersion::default();
        assert_eq!(default, SchemaVersion::current());
    }

    #[test]
    fn test_parse_valid_version_string() {
        let version = SchemaVersion::parse("1.2.3").unwrap();
        assert_eq!(version.major(), 1);
        assert_eq!(version.minor(), 2);
        assert_eq!(version.patch(), 3);
    }

    #[test]
    fn test_parse_version_with_zeros() {
        let version = SchemaVersion::parse("0.0.0").unwrap();
        assert_eq!(version.major(), 0);
        assert_eq!(version.minor(), 0);
        assert_eq!(version.patch(), 0);
    }

    #[test]
    fn test_parse_version_with_large_numbers() {
        let version = SchemaVersion::parse("999.888.777").unwrap();
        assert_eq!(version.major(), 999);
        assert_eq!(version.minor(), 888);
        assert_eq!(version.patch(), 777);
    }

    #[test]
    fn test_parse_invalid_format_missing_parts() {
        let result = SchemaVersion::parse("1.2");
        assert!(matches!(result, Err(SchemaVersionParseError::InvalidFormat(_))));
    }

    #[test]
    fn test_parse_invalid_format_extra_parts() {
        let result = SchemaVersion::parse("1.2.3.4");
        assert!(matches!(result, Err(SchemaVersionParseError::InvalidFormat(_))));
    }

    #[test]
    fn test_parse_invalid_major_not_a_number() {
        let result = SchemaVersion::parse("a.2.3");
        assert!(matches!(result, Err(SchemaVersionParseError::InvalidMajor(_))));
    }

    #[test]
    fn test_parse_invalid_minor_not_a_number() {
        let result = SchemaVersion::parse("1.b.3");
        assert!(matches!(result, Err(SchemaVersionParseError::InvalidMinor(_))));
    }

    #[test]
    fn test_parse_invalid_patch_not_a_number() {
        let result = SchemaVersion::parse("1.2.c");
        assert!(matches!(result, Err(SchemaVersionParseError::InvalidPatch(_))));
    }
}

// =============================================================================
// 2. SchemaVersion compatibility (5 tests)
// =============================================================================

mod schema_version_compatibility {
    use super::*;

    #[test]
    fn test_same_major_versions_are_compatible() {
        let v1_0_0 = SchemaVersion::new(1, 0, 0);
        let v1_5_3 = SchemaVersion::new(1, 5, 3);
        assert!(v1_0_0.is_compatible_with(&v1_5_3));
        assert!(v1_5_3.is_compatible_with(&v1_0_0));
    }

    #[test]
    fn test_different_major_versions_are_not_compatible() {
        let v1_0_0 = SchemaVersion::new(1, 0, 0);
        let v2_0_0 = SchemaVersion::new(2, 0, 0);
        assert!(!v1_0_0.is_compatible_with(&v2_0_0));
        assert!(!v2_0_0.is_compatible_with(&v1_0_0));
    }

    #[test]
    fn test_identical_versions_are_compatible() {
        let version = SchemaVersion::new(1, 2, 3);
        assert!(version.is_compatible_with(&version));
    }

    #[test]
    fn test_compatibility_is_reflexive() {
        let v1 = SchemaVersion::new(3, 7, 2);
        let v2 = SchemaVersion::new(3, 1, 9);
        assert_eq!(
            v1.is_compatible_with(&v2),
            v2.is_compatible_with(&v1)
        );
    }

    #[test]
    fn test_zero_major_version_compatibility() {
        let v0_1_0 = SchemaVersion::new(0, 1, 0);
        let v0_9_9 = SchemaVersion::new(0, 9, 9);
        let v1_0_0 = SchemaVersion::new(1, 0, 0);
        
        assert!(v0_1_0.is_compatible_with(&v0_9_9));
        assert!(!v0_1_0.is_compatible_with(&v1_0_0));
    }
}

// =============================================================================
// 3. ValidationError variants (6 tests)
// =============================================================================

mod validation_error_variants {
    use super::*;

    #[test]
    fn test_missing_field_error_message() {
        let error = ValidationError::MissingField("test_field".to_string());
        assert!(error.to_string().contains("test_field"));
        assert!(error.to_string().contains("Missing required field"));
    }

    #[test]
    fn test_invalid_type_error_message() {
        let error = ValidationError::InvalidType(
            "my_field".to_string(),
            "string".to_string(),
            "number".to_string(),
        );
        let msg = error.to_string();
        assert!(msg.contains("my_field"));
        assert!(msg.contains("string"));
        assert!(msg.contains("number"));
    }

    #[test]
    fn test_invalid_value_error_message() {
        let error = ValidationError::InvalidValue("count".to_string(), "must be positive".to_string());
        let msg = error.to_string();
        assert!(msg.contains("count"));
        assert!(msg.contains("must be positive"));
    }

    #[test]
    fn test_version_mismatch_error_message() {
        let error = ValidationError::VersionMismatch("1.0.0".to_string(), "2.0.0".to_string());
        let msg = error.to_string();
        assert!(msg.contains("1.0.0"));
        assert!(msg.contains("2.0.0"));
        assert!(msg.contains("mismatch"));
    }

    #[test]
    fn test_custom_error_message() {
        let error = ValidationError::Custom("Something went wrong".to_string());
        let msg = error.to_string();
        assert!(msg.contains("Something went wrong"));
    }

    #[test]
    fn test_error_can_be_cloned() {
        let error = ValidationError::MissingField("field".to_string());
        let cloned = error.clone();
        assert_eq!(error.to_string(), cloned.to_string());
    }
}

// =============================================================================
// 4. ValidationResult methods (8 tests)
// =============================================================================

mod validation_result_methods {
    use super::*;

    #[test]
    fn test_valid_creates_successful_result() {
        let result = ValidationResult::valid();
        assert!(result.is_valid);
        assert!(result.is_ok());
        assert!(!result.is_err());
        assert!(result.errors().is_empty());
    }

    #[test]
    fn test_invalid_creates_failed_result_with_errors() {
        let errors = vec![
            ValidationError::MissingField("field1".to_string()),
            ValidationError::MissingField("field2".to_string()),
        ];
        let result = ValidationResult::invalid(errors);
        assert!(!result.is_valid);
        assert!(result.is_err());
        assert!(!result.is_ok());
        assert_eq!(result.errors().len(), 2);
    }

    #[test]
    fn test_with_error_creates_failed_result_with_single_error() {
        let result = ValidationResult::with_error(ValidationError::Custom("error".to_string()));
        assert!(!result.is_valid);
        assert!(result.is_err());
        assert_eq!(result.errors().len(), 1);
    }

    #[test]
    fn test_default_creates_valid_result() {
        let result = ValidationResult::default();
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_combines_errors_from_invalid_result() {
        let mut result1 = ValidationResult::with_error(ValidationError::MissingField("a".to_string()));
        let result2 = ValidationResult::with_error(ValidationError::MissingField("b".to_string()));
        
        result1.merge(result2);
        
        assert!(result1.is_err());
        assert_eq!(result1.errors().len(), 2);
    }

    #[test]
    fn test_merge_with_valid_result_does_not_change_validity() {
        let mut result1 = ValidationResult::valid();
        let result2 = ValidationResult::valid();
        
        result1.merge(result2);
        
        assert!(result1.is_ok());
        assert!(result1.errors().is_empty());
    }

    #[test]
    fn test_merge_valid_into_invalid_preserves_errors() {
        let mut result1 = ValidationResult::with_error(ValidationError::Custom("error".to_string()));
        let result2 = ValidationResult::valid();
        
        result1.merge(result2);
        
        assert!(result1.is_err());
        assert_eq!(result1.errors().len(), 1);
    }

    #[test]
    fn test_errors_returns_slice() {
        let errors = vec![
            ValidationError::MissingField("a".to_string()),
            ValidationError::MissingField("b".to_string()),
        ];
        let result = ValidationResult::invalid(errors);
        
        let error_slice = result.errors();
        assert_eq!(error_slice.len(), 2);
    }
}

// =============================================================================
// 5. ReportValidator basic validation (10 tests)
// =============================================================================

mod report_validator_basic_validation {
    use super::*;

    #[test]
    fn test_new_creates_validator_with_current_version() {
        let validator = ReportValidator::new();
        assert_eq!(validator.version(), &SchemaVersion::current());
    }

    #[test]
    fn test_default_creates_validator_with_current_version() {
        let validator = ReportValidator::default();
        assert_eq!(validator.version(), &SchemaVersion::current());
    }

    #[test]
    fn test_for_version_creates_validator_with_specific_version() {
        let version = SchemaVersion::new(2, 1, 0);
        let validator = ReportValidator::for_version(version);
        assert_eq!(validator.version(), &version);
    }

    #[test]
    fn test_validate_accepts_valid_json_object() {
        let validator = ReportValidator::new();
        let json = serde_json::json!({
            "schema_version": "1.0.0",
            "verdict": "pass",
            "findings": []
        });
        let result = validator.validate(&json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_rejects_non_object_json() {
        let validator = ReportValidator::new();
        let json = serde_json::json!("not an object");
        let result = validator.validate(&json);
        assert!(result.is_err());
        assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidType(..))));
    }

    #[test]
    fn test_validate_rejects_null_json() {
        let validator = ReportValidator::new();
        let json = serde_json::Value::Null;
        let result = validator.validate(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_rejects_array_json() {
        let validator = ReportValidator::new();
        let json = serde_json::json!([1, 2, 3]);
        let result = validator.validate(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_rejects_incompatible_schema_version() {
        let validator = ReportValidator::new();
        let json = serde_json::json!({
            "schema_version": "2.0.0",
            "verdict": "pass",
            "findings": []
        });
        let result = validator.validate(&json);
        assert!(result.is_err());
        assert!(result.errors().iter().any(|e| matches!(e, ValidationError::VersionMismatch(..))));
    }

    #[test]
    fn test_validate_rejects_invalid_version_format() {
        let validator = ReportValidator::new();
        let json = serde_json::json!({
            "schema_version": "invalid",
            "verdict": "pass",
            "findings": []
        });
        let result = validator.validate(&json);
        assert!(result.is_err());
        assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidValue(..))));
    }

    #[test]
    fn test_validate_rejects_non_string_version() {
        let validator = ReportValidator::new();
        let json = serde_json::json!({
            "schema_version": 123,
            "verdict": "pass",
            "findings": []
        });
        let result = validator.validate(&json);
        assert!(result.is_err());
        assert!(result.errors().iter().any(|e| matches!(e, ValidationError::InvalidType(..))));
    }

    #[test]
    fn test_validate_str_parses_and_validates_json_string() {
        let validator = ReportValidator::new();
        let json_str = r#"{"schema_version": "1.0.0", "verdict": "pass", "findings": []}"#;
        let result = validator.validate_str(json_str).unwrap();
        assert!(result.is_ok());
    }
}

// =============================================================================
// 6. ReportValidator required fields (6 tests)
// =============================================================================

mod report_validator_required_fields {
    use super::*;

    #[test]
    fn test_validate_requires_schema_version_by_default() {
        let validator = ReportValidator::new();
        let json = serde_json::json!({
            "verdict": "pass",
            "findings": []
        });
        let result = validator.validate(&json);
        assert!(result.is_err());
        assert!(result.errors().iter().any(|e| matches!(e, ValidationError::MissingField(f) if f == "schema_version")));
    }

    #[test]
    fn test_validate_requires_verdict() {
        let validator = ReportValidator::new();
        let json = serde_json::json!({
            "schema_version": "1.0.0",
            "findings": []
        });
        let result = validator.validate(&json);
        assert!(result.is_err());
        assert!(result.errors().iter().any(|e| matches!(e, ValidationError::MissingField(f) if f == "verdict")));
    }

    #[test]
    fn test_validate_requires_findings() {
        let validator = ReportValidator::new();
        let json = serde_json::json!({
            "schema_version": "1.0.0",
            "verdict": "pass"
        });
        let result = validator.validate(&json);
        assert!(result.is_err());
        assert!(result.errors().iter().any(|e| matches!(e, ValidationError::MissingField(f) if f == "findings")));
    }

    #[test]
    fn test_require_version_can_be_disabled() {
        let validator = ReportValidator::new().require_version(false);
        let json = serde_json::json!({
            "verdict": "pass",
            "findings": []
        });
        let result = validator.validate(&json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_with_required_field_adds_custom_required_field() {
        let validator = ReportValidator::new()
            .require_version(false)
            .with_required_field("custom_field");
        let json = serde_json::json!({
            "verdict": "pass",
            "findings": []
        });
        let result = validator.validate(&json);
        assert!(result.is_err());
        assert!(result.errors().iter().any(|e| matches!(e, ValidationError::MissingField(f) if f == "custom_field")));
    }

    #[test]
    fn test_multiple_required_fields_all_checked() {
        let validator = ReportValidator::new()
            .require_version(false)
            .with_required_field("field1")
            .with_required_field("field2");
        let json = serde_json::json!({
            "verdict": "pass",
            "findings": [],
            "field1": "value"
        });
        let result = validator.validate(&json);
        assert!(result.is_err());
        assert!(result.errors().iter().any(|e| matches!(e, ValidationError::MissingField(f) if f == "field2")));
    }
}

// =============================================================================
// 7. current_schema function (5 tests)
// =============================================================================

mod current_schema_function {
    use super::*;

    #[test]
    fn test_current_schema_returns_non_empty_string() {
        let schema = current_schema();
        assert!(!schema.is_empty());
    }

    #[test]
    fn test_current_schema_contains_json_schema_identifier() {
        let schema = current_schema();
        assert!(schema.contains("$schema"));
        assert!(schema.contains("json-schema.org"));
    }

    #[test]
    fn test_current_schema_contains_required_fields() {
        let schema = current_schema();
        assert!(schema.contains("verdict"));
        assert!(schema.contains("findings"));
    }

    #[test]
    fn test_current_schema_is_valid_json() {
        let schema = current_schema();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(schema);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_current_schema_defines_verdict_enum() {
        let schema = current_schema();
        assert!(schema.contains("pass"));
        assert!(schema.contains("warn"));
        assert!(schema.contains("fail"));
    }
}

// =============================================================================
// Additional tests: FromStr implementation, Display, Ordering
// =============================================================================

mod additional_tests {
    use super::*;

    #[test]
    fn test_from_str_trait_implementation() {
        let version: SchemaVersion = "1.2.3".parse().unwrap();
        assert_eq!(version.major(), 1);
        assert_eq!(version.minor(), 2);
        assert_eq!(version.patch(), 3);
    }

    #[test]
    fn test_from_str_returns_error_for_invalid_input() {
        let result: Result<SchemaVersion, _> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_display_trait_implementation() {
        let version = SchemaVersion::new(2, 5, 1);
        assert_eq!(format!("{}", version), "2.5.1");
    }

    #[test]
    fn test_to_string_method() {
        let version = SchemaVersion::new(3, 4, 5);
        assert_eq!(version.to_string(), "3.4.5");
    }

    #[test]
    fn test_version_ordering() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(1, 1, 0);
        let v3 = SchemaVersion::new(2, 0, 0);
        
        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_version_equality() {
        let v1 = SchemaVersion::new(1, 2, 3);
        let v2 = SchemaVersion::new(1, 2, 3);
        let v3 = SchemaVersion::new(1, 2, 4);
        
        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_version_hash() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        let v1 = SchemaVersion::new(1, 0, 0);
        let v2 = SchemaVersion::new(1, 0, 0);
        let v3 = SchemaVersion::new(2, 0, 0);
        
        set.insert(v1);
        assert!(set.contains(&v2));
        assert!(!set.contains(&v3));
    }

    #[test]
    fn test_validate_str_returns_error_for_invalid_json() {
        let validator = ReportValidator::new();
        let result = validator.validate_str("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_validator_can_be_cloned() {
        let validator = ReportValidator::new().require_version(false);
        let cloned = validator.clone();
        assert_eq!(validator.version(), cloned.version());
    }

    #[test]
    fn test_compatible_version_passes_validation() {
        let validator = ReportValidator::new();
        let json = serde_json::json!({
            "schema_version": "1.5.3",
            "verdict": "warn",
            "findings": [{"path": "src/main.rs", "message": "test"}]
        });
        let result = validator.validate(&json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_result_can_be_cloned() {
        let result = ValidationResult::with_error(ValidationError::Custom("test".to_string()));
        let cloned = result.clone();
        assert_eq!(result.is_valid, cloned.is_valid);
        assert_eq!(result.errors().len(), cloned.errors().len());
    }
}

// =============================================================================
// Property-based tests using proptest
// =============================================================================

mod property_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_parse_roundtrip(major in 0u32..100, minor in 0u32..100, patch in 0u32..100) {
            let version = SchemaVersion::new(major, minor, patch);
            let s = version.to_string();
            let parsed = SchemaVersion::parse(&s).unwrap();
            prop_assert_eq!(version, parsed);
        }

        #[test]
        fn test_compatibility_same_major(major in 0u32..10, minor1 in 0u32..100, patch1 in 0u32..100, minor2 in 0u32..100, patch2 in 0u32..100) {
            let v1 = SchemaVersion::new(major, minor1, patch1);
            let v2 = SchemaVersion::new(major, minor2, patch2);
            prop_assert!(v1.is_compatible_with(&v2));
        }

        #[test]
        fn test_compatibility_different_major(major1 in 0u32..10, major2 in 11u32..20, minor1 in 0u32..100, patch1 in 0u32..100, minor2 in 0u32..100, patch2 in 0u32..100) {
            let v1 = SchemaVersion::new(major1, minor1, patch1);
            let v2 = SchemaVersion::new(major2, minor2, patch2);
            prop_assert!(!v1.is_compatible_with(&v2));
        }

        #[test]
        fn test_version_ordering_transitive(a in 0u32..50, b in 51u32..100, c in 101u32..150) {
            let v1 = SchemaVersion::new(a, 0, 0);
            let v2 = SchemaVersion::new(b, 0, 0);
            let v3 = SchemaVersion::new(c, 0, 0);
            prop_assert!(v1 < v2 && v2 < v3 && v1 < v3);
        }
    }
}
