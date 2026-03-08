//! Scenario-grid helpers for applying feature flags in BDD-style matrix tests.

use lintdiff_feature_flags::{
    feature_flags, parse_feature_flag_assignment, set_feature_flags_from_assignments,
};
use lintdiff_types::{FeatureFlags, LintdiffConfig};

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
            assignments.push((spec.as_str().to_string(), enabled));
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
        Self::new(feature_flags().iter().map(|spec| spec.key))
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
}
