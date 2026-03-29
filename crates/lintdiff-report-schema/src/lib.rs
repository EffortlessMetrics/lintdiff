//! Report schema validation for lintdiff.
//!
//! Provides JSON schema validation for lintdiff output reports,
//! ensuring compatibility and version stability.

/// Report schema version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl SchemaVersion {
    /// Create a new schema version.
    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// Get the current (latest) schema version.
    #[must_use]
    pub const fn current() -> Self {
        Self::new(1, 0, 0)
    }

    /// Get the major version.
    #[must_use]
    pub const fn major(&self) -> u32 {
        self.major
    }

    /// Get the minor version.
    #[must_use]
    pub const fn minor(&self) -> u32 {
        self.minor
    }

    /// Get the patch version.
    #[must_use]
    pub const fn patch(&self) -> u32 {
        self.patch
    }

    /// Parse from a string (e.g., "1.0.0").
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not in the format "X.Y.Z" where X, Y, and Z are non-negative integers.
    pub fn parse(s: &str) -> Result<Self, SchemaVersionParseError> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(SchemaVersionParseError::InvalidFormat(s.to_string()));
        }
        let major = parts[0]
            .parse()
            .map_err(|_| SchemaVersionParseError::InvalidMajor(parts[0].to_string()))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| SchemaVersionParseError::InvalidMinor(parts[1].to_string()))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| SchemaVersionParseError::InvalidPatch(parts[2].to_string()))?;
        Ok(Self { major, minor, patch })
    }

    /// Check if this version is compatible with another.
    ///
    /// Versions are compatible if they have the same major version.
    #[must_use]
    pub const fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl std::str::FromStr for SchemaVersion {
    type Err = SchemaVersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::current()
    }
}

/// Error when parsing a schema version.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SchemaVersionParseError {
    #[error("Invalid version format: '{0}' (expected X.Y.Z)")]
    InvalidFormat(String),
    #[error("Invalid major version: '{0}'")]
    InvalidMajor(String),
    #[error("Invalid minor version: '{0}'")]
    InvalidMinor(String),
    #[error("Invalid patch version: '{0}'")]
    InvalidPatch(String),
}

/// A validation error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidationError {
    #[error("Missing required field: '{0}'")]
    MissingField(String),
    #[error("Invalid type for field '{0}': expected {1}, got {2}")]
    InvalidType(String, String, String),
    #[error("Invalid value for field '{0}': {1}")]
    InvalidValue(String, String),
    #[error("Schema version mismatch: expected {0}, got {1}")]
    VersionMismatch(String, String),
    #[error("Custom error: {0}")]
    Custom(String),
}

/// Result of schema validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed.
    pub is_valid: bool,
    /// Validation errors (if any).
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    /// Create a successful validation result.
    #[must_use]
    pub const fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    /// Create a failed validation result with errors.
    #[must_use]
    pub const fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }

    /// Create a failed validation result with a single error.
    #[must_use]
    pub fn with_error(error: ValidationError) -> Self {
        Self {
            is_valid: false,
            errors: vec![error],
        }
    }

    /// Check if validation passed.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.is_valid
    }

    /// Check if validation failed.
    #[must_use]
    pub const fn is_err(&self) -> bool {
        !self.is_valid
    }

    /// Get the errors.
    #[must_use]
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Merge another validation result into this one.
    pub fn merge(&mut self, other: Self) {
        if !other.is_valid {
            self.is_valid = false;
            self.errors.extend(other.errors);
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::valid()
    }
}

/// Schema validator for reports.
#[derive(Debug, Clone)]
pub struct ReportValidator {
    /// Expected schema version.
    version: SchemaVersion,
    /// Whether to require the version field.
    require_version: bool,
    /// Required fields.
    required_fields: Vec<String>,
}

impl ReportValidator {
    /// Create a new validator with the current schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: SchemaVersion::current(),
            require_version: true,
            required_fields: vec!["verdict".to_string(), "findings".to_string()],
        }
    }

    /// Create a validator for a specific version.
    #[must_use]
    pub fn for_version(version: SchemaVersion) -> Self {
        Self {
            version,
            ..Self::new()
        }
    }

    /// Set whether to require the version field.
    #[must_use]
    pub const fn require_version(mut self, require: bool) -> Self {
        self.require_version = require;
        self
    }

    /// Add a required field.
    #[must_use]
    pub fn with_required_field(mut self, field: impl Into<String>) -> Self {
        self.required_fields.push(field.into());
        self
    }

    /// Get the schema version.
    #[must_use]
    pub const fn version(&self) -> &SchemaVersion {
        &self.version
    }

    /// Validate a JSON value against the schema.
    ///
    /// This method validates that the JSON value is an object with the required fields
    /// and that the schema version (if present and required) is compatible.
    #[must_use]
    pub fn validate(&self, value: &serde_json::Value) -> ValidationResult {
        let mut result = ValidationResult::valid();

        // Check if it's an object
        let Some(obj) = value.as_object() else {
            result.merge(ValidationResult::with_error(ValidationError::InvalidType(
                "root".to_string(),
                "object".to_string(),
                value_type_name(value),
            )));
            return result;
        };

        // Check version if required
        if self.require_version {
            if let Some(version_val) = obj.get("schema_version") {
                if let Some(version_str) = version_val.as_str() {
                    match SchemaVersion::parse(version_str) {
                        Ok(parsed) => {
                            if !parsed.is_compatible_with(&self.version) {
                                result.merge(ValidationResult::with_error(
                                    ValidationError::VersionMismatch(
                                        self.version.to_string(),
                                        parsed.to_string(),
                                    ),
                                ));
                            }
                        }
                        Err(e) => {
                            result.merge(ValidationResult::with_error(
                                ValidationError::InvalidValue(
                                    "schema_version".to_string(),
                                    e.to_string(),
                                ),
                            ));
                        }
                    }
                } else {
                    result.merge(ValidationResult::with_error(ValidationError::InvalidType(
                        "schema_version".to_string(),
                        "string".to_string(),
                        value_type_name(version_val),
                    )));
                }
            } else {
                result.merge(ValidationResult::with_error(ValidationError::MissingField(
                    "schema_version".to_string(),
                )));
            }
        }

        // Check required fields
        for field in &self.required_fields {
            if !obj.contains_key(field) {
                result.merge(ValidationResult::with_error(ValidationError::MissingField(
                    field.clone(),
                )));
            }
        }

        result
    }

    /// Validate a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is not valid JSON.
    pub fn validate_str(&self, json: &str) -> Result<ValidationResult, serde_json::Error> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        Ok(self.validate(&value))
    }
}

impl Default for ReportValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Get a type name for a JSON value.
#[must_use]
fn value_type_name(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) => "boolean".to_string(),
        serde_json::Value::Number(_) => "number".to_string(),
        serde_json::Value::String(_) => "string".to_string(),
        serde_json::Value::Array(_) => "array".to_string(),
        serde_json::Value::Object(_) => "object".to_string(),
    }
}

/// Get the JSON schema for the current version.
#[must_use]
pub const fn current_schema() -> &'static str {
    r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Lintdiff Report",
  "type": "object",
  "required": ["verdict", "findings"],
  "properties": {
    "schema_version": {
      "type": "string",
      "pattern": "^[0-9]+\\.[0-9]+\\.[0-9]+$"
    },
    "verdict": {
      "type": "string",
      "enum": ["pass", "warn", "fail"]
    },
    "findings": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["path", "message"],
        "properties": {
          "path": { "type": "string" },
          "line": { "type": "integer", "minimum": 1 },
          "column": { "type": "integer", "minimum": 1 },
          "message": { "type": "string" },
          "severity": {
            "type": "string",
            "enum": ["hint", "note", "warning", "error", "fatal"]
          },
          "code": { "type": "string" }
        }
      }
    },
    "counts": {
      "type": "object",
      "properties": {
        "new": { "type": "integer", "minimum": 0 },
        "fixed": { "type": "integer", "minimum": 0 },
        "pre_existing": { "type": "integer", "minimum": 0 }
      }
    }
  }
}"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_new() {
        let version = SchemaVersion::new(1, 2, 3);
        assert_eq!(version.major(), 1);
        assert_eq!(version.minor(), 2);
        assert_eq!(version.patch(), 3);
    }

    #[test]
    fn test_schema_version_current() {
        let current = SchemaVersion::current();
        assert_eq!(current.major(), 1);
        assert_eq!(current.minor(), 0);
        assert_eq!(current.patch(), 0);
    }

    #[test]
    fn test_schema_version_default() {
        let default = SchemaVersion::default();
        assert_eq!(default, SchemaVersion::current());
    }

    #[test]
    fn test_schema_version_parse_valid() {
        let version = SchemaVersion::parse("1.2.3").unwrap();
        assert_eq!(version.major(), 1);
        assert_eq!(version.minor(), 2);
        assert_eq!(version.patch(), 3);
    }

    #[test]
    fn test_schema_version_parse_invalid_format() {
        assert!(matches!(
            SchemaVersion::parse("1.2"),
            Err(SchemaVersionParseError::InvalidFormat(_))
        ));
    }

    #[test]
    fn test_schema_version_compatibility() {
        let v1_0_0 = SchemaVersion::new(1, 0, 0);
        let v1_1_0 = SchemaVersion::new(1, 1, 0);
        let v2_0_0 = SchemaVersion::new(2, 0, 0);

        assert!(v1_0_0.is_compatible_with(&v1_1_0));
        assert!(!v1_0_0.is_compatible_with(&v2_0_0));
    }

    #[test]
    fn test_schema_version_display() {
        let version = SchemaVersion::new(1, 2, 3);
        assert_eq!(format!("{}", version), "1.2.3");
    }

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult::valid();
        assert!(result.is_ok());
        assert!(!result.is_err());
        assert!(result.errors().is_empty());
    }

    #[test]
    fn test_validation_result_invalid() {
        let result =
            ValidationResult::invalid(vec![ValidationError::MissingField("test".to_string())]);
        assert!(!result.is_ok());
        assert!(result.is_err());
        assert_eq!(result.errors().len(), 1);
    }

    #[test]
    fn test_report_validator_new() {
        let validator = ReportValidator::new();
        assert_eq!(validator.version(), &SchemaVersion::current());
    }

    #[test]
    fn test_report_validator_validate_valid_json() {
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
    fn test_report_validator_validate_missing_field() {
        let validator = ReportValidator::new();
        let json = serde_json::json!({
            "schema_version": "1.0.0"
        });
        let result = validator.validate(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_current_schema() {
        let schema = current_schema();
        assert!(schema.contains("Lintdiff Report"));
        assert!(schema.contains("verdict"));
        assert!(schema.contains("findings"));
    }
}
