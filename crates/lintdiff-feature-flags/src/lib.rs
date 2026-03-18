//! Feature flag registry and typed application surface.
//!
//! This crate is a small interoperability shim for both app/runtime and BDD
//! wiring. Flags are intentionally normalized and centrally documented here so
//! flag names are stable across CLI, tests, and adapters.
//!
//! # Overview
//!
//! The feature flags system provides:
//! - A central registry of all feature flags ([`feature_flags`])
//! - Type-safe flag identifiers ([`FeatureFlag`] enum)
//! - Parsing utilities for flag values and assignments
//! - Functions to apply flags to [`FeatureFlags`] instances
//!
//! # Available Flags
//!
//! | Flag | Key | Default | Description |
//! |------|-----|---------|-------------|
//! | `PrimarySpanMatching` | `primary_span_matching` | `true` | Prefer primary spans when choosing diagnostic spans |
//! | `PathFilters` | `path_filters` | `true` | Apply include/exclude path filters against normalized paths |
//!
//! # Quick Start
//!
//! ```
//! use lintdiff_feature_flags::{FeatureFlag, feature_flags, set_feature_flag};
//! use lintdiff_types::FeatureFlags;
//!
//! // Create default flags
//! let mut flags = FeatureFlags::default();
//!
//! // Disable a specific feature
//! set_feature_flag(&mut flags, FeatureFlag::PathFilters, false);
//! assert!(!flags.path_filters);
//! ```
//!
//! # Parsing Flag Values
//!
//! The crate accepts various boolean representations:
//!
//! ```
//! use lintdiff_feature_flags::parse_feature_flag_value;
//!
//! // All of these return Ok(true)
//! assert_eq!(parse_feature_flag_value("true"), Ok(true));
//! assert_eq!(parse_feature_flag_value("TRUE"), Ok(true));
//! assert_eq!(parse_feature_flag_value("1"), Ok(true));
//! assert_eq!(parse_feature_flag_value("on"), Ok(true));
//! assert_eq!(parse_feature_flag_value("enabled"), Ok(true));
//! assert_eq!(parse_feature_flag_value("yes"), Ok(true));
//!
//! // All of these return Ok(false)
//! assert_eq!(parse_feature_flag_value("false"), Ok(false));
//! assert_eq!(parse_feature_flag_value("0"), Ok(false));
//! assert_eq!(parse_feature_flag_value("off"), Ok(false));
//! assert_eq!(parse_feature_flag_value("disabled"), Ok(false));
//! assert_eq!(parse_feature_flag_value("no"), Ok(false));
//! ```
//!
//! # Parsing Assignments
//!
//! Parse `name=value` style assignments:
//!
//! ```
//! use lintdiff_feature_flags::{parse_feature_flag_assignment, FeatureFlag};
//!
//! let (flag, enabled) = parse_feature_flag_assignment("path_filters=false").unwrap();
//! assert_eq!(flag, FeatureFlag::PathFilters);
//! assert!(!enabled);
//! ```
//!
//! # Batch Operations
//!
//! Apply multiple flags at once:
//!
//! ```
//! use lintdiff_feature_flags::set_feature_flags_from_assignments;
//! use lintdiff_types::FeatureFlags;
//!
//! let mut flags = FeatureFlags::default();
//! set_feature_flags_from_assignments(
//!     &mut flags,
//!     vec!["primary_span_matching=off", "path_filters=false"]
//! ).unwrap();
//!
//! assert!(!flags.prefer_primary_spans);
//! assert!(!flags.path_filters);
//! ```
//!
//! # Case Insensitivity
//!
//! Flag names and values are case-insensitive:
//!
//! ```
//! use lintdiff_feature_flags::{parse_flag, parse_feature_flag_value, FeatureFlag};
//!
//! assert_eq!(parse_flag("PRIMARY_SPAN_MATCHING"), Some(FeatureFlag::PrimarySpanMatching));
//! assert_eq!(parse_flag("Path_Filters"), Some(FeatureFlag::PathFilters));
//! assert_eq!(parse_feature_flag_value("FALSE"), Ok(false));
//! assert_eq!(parse_feature_flag_value("ON"), Ok(true));
//! ```

use lintdiff_types::FeatureFlags;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeatureFlag {
    /// Match primary spans only when enabled (default: true).
    PrimarySpanMatching,
    /// Apply include/exclude path filters when enabled (default: true).
    PathFilters,
}

#[derive(Clone, Copy, Debug)]
pub struct FeatureFlagSpec {
    pub id: FeatureFlag,
    pub key: &'static str,
    pub description: &'static str,
    pub default_enabled: bool,
}

pub const FEATURE_FLAGS: &[FeatureFlagSpec] = &[
    FeatureFlagSpec {
        id: FeatureFlag::PrimarySpanMatching,
        key: "primary_span_matching",
        description: "Prefer primary spans when choosing diagnostic spans.",
        default_enabled: true,
    },
    FeatureFlagSpec {
        id: FeatureFlag::PathFilters,
        key: "path_filters",
        description: "Apply include/exclude path filters against normalized paths.",
        default_enabled: true,
    },
];

pub fn feature_flags() -> &'static [FeatureFlagSpec] {
    FEATURE_FLAGS
}

impl FeatureFlag {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PrimarySpanMatching => "primary_span_matching",
            Self::PathFilters => "path_filters",
        }
    }

    pub const fn default_enabled(self) -> bool {
        match self {
            Self::PrimarySpanMatching => true,
            Self::PathFilters => true,
        }
    }
}

pub fn parse_flag(name: &str) -> Option<FeatureFlag> {
    FEATURE_FLAGS
        .iter()
        .find(|spec| spec.key.eq_ignore_ascii_case(name))
        .map(|spec| spec.id)
}

pub const TRUE_VALUES: [&str; 5] = ["true", "1", "on", "enabled", "yes"];
pub const FALSE_VALUES: [&str; 5] = ["false", "0", "off", "disabled", "no"];

/// Parse a user-facing boolean value used by runtime and BDD wiring.
/// Accepts common truthy/falsey synonyms and normalizes case.
pub fn parse_feature_flag_value(raw: &str) -> Result<bool, String> {
    let normalized = raw.trim().to_ascii_lowercase();
    if TRUE_VALUES.contains(&normalized.as_str()) {
        Ok(true)
    } else if FALSE_VALUES.contains(&normalized.as_str()) {
        Ok(false)
    } else {
        Err(format!(
            "unknown feature flag value '{raw}'. expected one of true/false/on/off/1/0/enabled/disabled/yes/no"
        ))
    }
}

/// Parse an assignment like `feature=value` and map it to a flag and bool value.
pub fn parse_feature_flag_assignment(raw: &str) -> Result<(FeatureFlag, bool), String> {
    let (name, value) = raw
        .split_once('=')
        .ok_or_else(|| format!("invalid feature flag assignment '{raw}'. expected name=value"))?;

    let flag = parse_flag(name.trim()).ok_or_else(|| format!("unknown feature flag: {name}"))?;
    let enabled = parse_feature_flag_value(value)?;
    Ok((flag, enabled))
}

/// Apply a batch of `name=value` assignments.
///
/// This is useful for CLI matrix inputs and programmatic BDD setup.
pub fn set_feature_flags_from_assignments<I, S>(
    flags: &mut FeatureFlags,
    assignments: I,
) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    for raw in assignments {
        set_feature_flag_from_assignment(flags, raw.as_ref())?;
    }
    Ok(())
}

/// Parse and apply a `name=value` assignment.
pub fn set_feature_flag_from_assignment(flags: &mut FeatureFlags, raw: &str) -> Result<(), String> {
    let (flag, enabled) = parse_feature_flag_assignment(raw)?;
    set_feature_flag(flags, flag, enabled);
    Ok(())
}

/// Parse and apply a named flag plus value pair.
pub fn set_feature_flag_by_name_and_value(
    flags: &mut FeatureFlags,
    name: &str,
    value: &str,
) -> Result<(), String> {
    let enabled = parse_feature_flag_value(value)?;
    set_feature_flag_by_name(flags, name, enabled)
}

pub fn set_feature_flag(flags: &mut FeatureFlags, flag: FeatureFlag, enabled: bool) {
    match flag {
        FeatureFlag::PrimarySpanMatching => flags.prefer_primary_spans = enabled,
        FeatureFlag::PathFilters => flags.path_filters = enabled,
    }
}

pub fn set_feature_flag_by_name(
    flags: &mut FeatureFlags,
    name: &str,
    enabled: bool,
) -> Result<(), String> {
    let Some(flag) = parse_flag(name) else {
        return Err(format!("unknown feature flag: {name}"));
    };
    set_feature_flag(flags, flag, enabled);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintdiff_types::FeatureFlags;

    #[test]
    fn parse_and_apply_known_flag() {
        let mut flags = FeatureFlags::default();
        assert!(set_feature_flag_by_name(&mut flags, "path_filters", false).is_ok());
        assert!(!flags.path_filters);
        assert!(parse_flag("primary_span_matching").is_some());
    }

    #[test]
    fn parse_unknown_flag() {
        assert!(
            set_feature_flag_by_name(&mut FeatureFlags::default(), "does_not_exist", true).is_err()
        );
    }

    #[test]
    fn parse_feature_flag_value_synonyms() {
        assert!(parse_feature_flag_value("TRUE").is_ok_and(|v| v));
        assert!(parse_feature_flag_value("off").is_ok_and(|v| !v));
        assert!(parse_feature_flag_value(" maybe ").is_err());
    }

    #[test]
    fn set_feature_flags_from_assignments_is_supported() {
        let mut flags = FeatureFlags::default();
        assert!(set_feature_flags_from_assignments(
            &mut flags,
            vec!["primary_span_matching=off", "path_filters=false"]
        )
        .is_ok());
        assert!(!flags.prefer_primary_spans);
        assert!(!flags.path_filters);
    }

    #[test]
    fn parse_feature_flag_assignment_is_accepted() {
        let (flag, enabled) =
            parse_feature_flag_assignment("primary_span_matching=off").expect("valid assignment");
        assert_eq!(flag, FeatureFlag::PrimarySpanMatching);
        assert!(!enabled);
    }

    #[test]
    fn set_feature_flag_by_name_and_value_works() {
        let mut flags = FeatureFlags::default();
        assert!(set_feature_flag_by_name_and_value(&mut flags, "path_filters", "false").is_ok());
        assert!(!flags.path_filters);
        assert!(set_feature_flag_by_name_and_value(&mut flags, "does_not_exist", "true").is_err());
    }
}
