//! Scenario-grid helpers for applying feature flags in BDD-style matrix tests.

use lintdiff_types::{FeatureFlags, LintdiffConfig};

const FEATURE_FLAG_KEYS: [&str; 2] = ["primary_span_matching", "path_filters"];

fn parse_feature_flag_value(raw: &str) -> Result<bool, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "on" | "enabled" | "yes" => Ok(true),
        "false" | "0" | "off" | "disabled" | "no" => Ok(false),
        _ => Err(format!(
            "unknown feature flag value '{raw}'. expected one of true/false/on/off/1/0/enabled/disabled/yes/no"
        )),
    }
}

fn parse_feature_flag_assignment(raw: &str) -> Result<(String, bool), String> {
    let (name, value) = raw
        .split_once('=')
        .ok_or_else(|| format!("invalid feature flag assignment '{raw}'. expected name=value"))?;
    let name = name.trim();
    if !FEATURE_FLAG_KEYS
        .iter()
        .any(|key| key.eq_ignore_ascii_case(name))
    {
        return Err(format!("unknown feature flag: {name}"));
    }
    Ok((name.to_ascii_lowercase(), parse_feature_flag_value(value)?))
}

pub fn set_feature_flag_by_name_and_value(
    flags: &mut FeatureFlags,
    name: &str,
    value: &str,
) -> Result<(), String> {
    let enabled = parse_feature_flag_value(value)?;
    match name.trim().to_ascii_lowercase().as_str() {
        "primary_span_matching" => flags.prefer_primary_spans = enabled,
        "path_filters" => flags.path_filters = enabled,
        _ => return Err(format!("unknown feature flag: {name}")),
    }
    Ok(())
}

pub fn set_feature_flags_from_assignments<'a, I>(
    flags: &mut FeatureFlags,
    assignments: I,
) -> Result<(), String>
where
    I: IntoIterator<Item = &'a String>,
{
    for raw in assignments {
        let (name, enabled) = parse_feature_flag_assignment(raw)?;
        set_feature_flag_by_name_and_value(flags, &name, if enabled { "true" } else { "false" })?;
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeatureFlagGridRow {
    assignments: Vec<(String, bool)>,
}

impl FeatureFlagGridRow {
    pub fn from_pairs<I, K, V>(pairs: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut assignments = Vec::new();
        for (flag, value) in pairs {
            let key = flag.as_ref();
            let raw = value.as_ref();
            let (spec, enabled) = parse_feature_flag_assignment(&format!("{key}={raw}"))
                .map_err(|err| format!("invalid feature flag assignment '{key}={raw}': {err}"))?;
            assignments.push((spec, enabled));
        }

        Ok(Self { assignments })
    }

    pub fn assignments(&self) -> Vec<String> {
        self.assignments
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect()
    }

    pub fn into_assignments(self) -> Vec<String> {
        self.assignments()
    }

    pub fn apply_to_flags(&self, flags: &mut FeatureFlags) -> Result<(), String> {
        set_feature_flags_from_assignments(flags, self.assignments().iter())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FeatureFlagGrid {
    columns: Vec<String>,
    rows: Vec<FeatureFlagGridRow>,
}

impl FeatureFlagGrid {
    pub fn with_feature_flags() -> Self {
        Self::new(FEATURE_FLAG_KEYS)
    }

    pub fn new<I, S>(columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            columns: columns.into_iter().map(Into::into).collect(),
            rows: Vec::new(),
        }
    }

    pub fn with_headers<I, S>(headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(headers)
    }

    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    pub fn rows(&self) -> &[FeatureFlagGridRow] {
        &self.rows
    }

    pub fn add_row<I, S>(&mut self, values: I) -> Result<&mut Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut values = values
            .into_iter()
            .map(|v| v.as_ref().to_string())
            .collect::<Vec<_>>();
        if values.len() != self.columns.len() {
            return Err(format!(
                "grid row size mismatch: expected {} columns, got {}",
                self.columns.len(),
                values.len()
            ));
        }

        let mut pairs = Vec::with_capacity(values.len());
        for (c, v) in self.columns.iter().zip(values.drain(..)) {
            let (_spec, enabled) = parse_feature_flag_assignment(&format!("{c}={v}"))
                .map_err(|err| format!("invalid feature flag assignment '{c}={v}': {err}"))?;
            pairs.push((c.to_string(), enabled.to_string()));
        }

        self.rows.push(FeatureFlagGridRow::from_pairs(pairs)?);
        Ok(self)
    }

    pub fn add_row_pairs<I, K, V>(&mut self, pairs: I) -> Result<&mut Self, String>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let row = FeatureFlagGridRow::from_pairs(pairs)?;
        if row.assignments.len() != self.columns.len() {
            return Err(format!(
                "grid row size mismatch: expected {} columns, got {}",
                self.columns.len(),
                row.assignments.len()
            ));
        }

        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for (key, _) in &row.assignments {
            if !seen.insert(key) {
                return Err(format!(
                    "duplicate feature-flag column in grid row: '{key}'"
                ));
            }
            if !self.columns.iter().any(|header| header == key) {
                return Err(format!("unknown feature flag in row: '{key}'"));
            }
        }
        self.rows.push(row);
        Ok(self)
    }

    pub fn to_reports_input(&self, config: &LintdiffConfig) -> Vec<Result<LintdiffConfig, String>> {
        self.rows
            .iter()
            .map(|row| {
                let mut cfg = config.clone();
                row.apply_to_flags(&mut cfg.feature_flags)?;
                Ok(cfg)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_a_deterministic_grid() {
        let mut grid = FeatureFlagGrid::new(["primary_span_matching", "path_filters"]);
        grid.add_row(["false", "true"]).unwrap();
        assert_eq!(grid.rows().len(), 1);
        assert_eq!(
            grid.rows()[0].assignments(),
            vec![
                "primary_span_matching=false".to_string(),
                "path_filters=true".to_string()
            ]
        );
    }

    #[test]
    fn builds_from_registered_feature_flags() {
        let grid = FeatureFlagGrid::with_feature_flags();
        assert_eq!(
            grid.columns(),
            &[
                "primary_span_matching".to_string(),
                "path_filters".to_string()
            ][..]
        );
    }

    #[test]
    fn rejects_unknown_flag_in_pair_row() {
        let mut grid = FeatureFlagGrid::new(["primary_span_matching"]);
        assert!(grid.add_row(["maybe"]).is_err());
    }

    #[test]
    fn rejects_unknown_column() {
        let mut grid = FeatureFlagGrid::new(["primary_span_matching"]);
        let err = grid
            .add_row_pairs([("does_not_exist", "true")])
            .unwrap_err();
        assert!(err.contains("unknown feature flag"));
    }

    #[test]
    fn applies_alias_values_and_rejects_invalid_assignments() {
        let mut flags = FeatureFlags::default();
        set_feature_flag_by_name_and_value(&mut flags, "PATH_FILTERS", "off").unwrap();
        assert!(!flags.path_filters);
        assert!(set_feature_flag_by_name_and_value(&mut flags, "unknown", "true").is_err());

        let assignments = ["primary_span_matching=enabled".to_string()];
        set_feature_flags_from_assignments(&mut flags, assignments.iter()).unwrap();
        assert!(flags.prefer_primary_spans);
        assert!(set_feature_flags_from_assignments(
            &mut flags,
            ["path_filters=maybe".to_string()].iter()
        )
        .is_err());
    }

    #[test]
    fn rejects_malformed_and_unknown_grid_values() {
        assert!(parse_feature_flag_assignment("path_filters").is_err());
        assert!(parse_feature_flag_assignment("unknown=true").is_err());
        let mut grid = FeatureFlagGrid::new(["path_filters"]);
        assert!(grid.add_row(["maybe"]).is_err());
    }
}
