//! Complete internal observations used by current-mode report projection.

use crate::diagnostics::{
    CargoAnalysis, Diagnostic, DiagnosticObservation, ObservationSpan, ProducerUnit, Span,
};
use crate::policy::normalize_diagnostic_code;

/// One complete observation plus normalization needed by current reporting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InventoryObservation {
    pub source: DiagnosticObservation,
    pub code: String,
    pub url: Option<String>,
}

impl InventoryObservation {
    fn from_source(source: DiagnosticObservation) -> Self {
        let (code, url) = normalize_diagnostic_code(source.raw_code.as_deref());
        Self { source, code, url }
    }

    fn from_legacy(diagnostic: Diagnostic) -> Self {
        let raw_level = diagnostic_level_name(&diagnostic);
        let source = DiagnosticObservation {
            producer: ProducerUnit::default(),
            raw_level,
            raw_code: diagnostic.code_raw.clone(),
            message: diagnostic.message.clone(),
            rendered: diagnostic.rendered.clone(),
            spans: diagnostic.spans.iter().map(legacy_span).collect(),
            children: Vec::new(),
            diagnostic,
        };
        Self::from_source(source)
    }

    fn sort_key(&self) -> String {
        format!("{:?}", self.source)
    }
}

/// The complete internal inventory before diff scope, policy, or budgets.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct InternalInventory {
    pub observations: Vec<InventoryObservation>,
}

impl InternalInventory {
    pub(crate) fn from_analysis(analysis: &CargoAnalysis) -> Self {
        let mut inventory = Self {
            observations: analysis
                .observations
                .iter()
                .cloned()
                .map(InventoryObservation::from_source)
                .collect(),
        };
        inventory.sort_deterministically();
        inventory
    }

    pub(crate) fn from_diagnostics(diagnostics: &[Diagnostic]) -> Self {
        let mut inventory = Self {
            observations: diagnostics
                .iter()
                .cloned()
                .map(InventoryObservation::from_legacy)
                .collect(),
        };
        inventory.sort_deterministically();
        inventory
    }

    fn sort_deterministically(&mut self) {
        self.observations
            .sort_by_key(InventoryObservation::sort_key);
    }
}

fn diagnostic_level_name(diagnostic: &Diagnostic) -> String {
    match &diagnostic.level {
        crate::diagnostics::DiagnosticLevel::Error => "error".to_string(),
        crate::diagnostics::DiagnosticLevel::Warning => "warning".to_string(),
        crate::diagnostics::DiagnosticLevel::Note => "note".to_string(),
        crate::diagnostics::DiagnosticLevel::Help => "help".to_string(),
        crate::diagnostics::DiagnosticLevel::Other(level) => level.clone(),
    }
}

fn legacy_span(span: &Span) -> ObservationSpan {
    ObservationSpan {
        raw_file_name: Some(span.file.as_str().to_string()),
        raw_line_start: Some(span.line_start),
        raw_line_end: Some(span.line_end),
        raw_column_start: span.col_start,
        raw_column_end: span.col_end,
        normalized: Some(span.clone()),
        is_primary: span.is_primary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{parse_cargo_analysis, DiagnosticLevel};
    use std::io::Cursor;

    #[test]
    fn analysis_observations_survive_before_report_policy() -> Result<(), Box<dyn std::error::Error>>
    {
        let input = r#"{"reason":"compiler-message","package_id":"pkg-a","target":{"name":"a"},"message":{"level":"warning","message":"outside","spans":[{"file_name":"src/a.rs","line_start":9,"line_end":9,"is_primary":true}]}}
{"reason":"compiler-message","package_id":"pkg-b","target":{"name":"b"},"message":{"level":"warning","message":"no location","spans":[]}}
{"reason":"build-finished","success":true}"#;
        let analysis = parse_cargo_analysis(Cursor::new(input))?;
        let inventory = InternalInventory::from_analysis(&analysis);

        assert_eq!(inventory.observations.len(), 2);
        assert_eq!(
            inventory.observations[0].code,
            "lintdiff.diagnostic.unknown"
        );
        assert_eq!(
            inventory.observations[1].code,
            "lintdiff.diagnostic.unknown"
        );
        assert_eq!(
            inventory.observations[0].source.producer.package_id,
            Some("pkg-a".to_string())
        );
        assert_eq!(
            inventory.observations[1].source.producer.package_id,
            Some("pkg-b".to_string())
        );
        Ok(())
    }

    #[test]
    fn legacy_diagnostics_are_adapted_without_changing_their_shape() {
        let diagnostic = Diagnostic {
            level: DiagnosticLevel::Warning,
            code_raw: Some("clippy::needless_borrow".to_string()),
            message: "message".to_string(),
            spans: vec![Span {
                file: lintdiff_types::NormPath::from_repo_path("src/lib.rs"),
                line_start: 3,
                line_end: 3,
                col_start: None,
                col_end: None,
                is_primary: true,
            }],
            rendered: None,
        };
        let inventory = InternalInventory::from_diagnostics(std::slice::from_ref(&diagnostic));

        assert_eq!(inventory.observations.len(), 1);
        assert_eq!(inventory.observations[0].source.diagnostic, diagnostic);
        assert_eq!(
            inventory.observations[0].code,
            "lintdiff.diagnostic.clippy.needless_borrow"
        );
    }
}
