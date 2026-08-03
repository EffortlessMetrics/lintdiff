//! Complete internal observations used by current-mode report projection.

use crate::diagnostics::{
    CargoAnalysis, Diagnostic, DiagnosticObservation, ObservationSpan, ProducerUnit, Span,
};
use crate::identity::normalize_message;
use crate::policy::{format_level, normalize_diagnostic_code};
use lintdiff_types::inventory::{
    AnalysisProvenance, CargoTarget as WireCargoTarget, CompletionState, ContextualProvenance,
    DiagnosticChild as WireDiagnosticChild, DiagnosticRecord, DiagnosticSpan, DiagnosticSuggestion,
    HardProvenance, Inventory, InventorySummary, ProducerUnit as WireProducerUnit,
    UpstreamEvidence, INVENTORY_ID_ALGORITHM, INVENTORY_SCHEMA_ID,
};
use lintdiff_types::ToolInfo;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

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

/// Convert the canonical internal observations to the versioned inventory protocol.
pub fn inventory_from_analysis(
    analysis: &CargoAnalysis,
    tool: ToolInfo,
    contextual: ContextualProvenance,
) -> Result<Inventory, serde_json::Error> {
    let internal = InternalInventory::from_analysis(analysis);
    let mut diagnostics = Vec::with_capacity(internal.observations.len());
    let mut previous_seed = None;
    let mut duplicate_rank = 0_u64;

    for observation in &internal.observations {
        let seed = observation_seed(observation)?;
        let seed_bytes = serde_json::to_vec(&seed)?;
        if previous_seed.as_deref() == Some(seed_bytes.as_slice()) {
            duplicate_rank = duplicate_rank.saturating_add(1);
        } else {
            duplicate_rank = 0;
            previous_seed = Some(seed_bytes.clone());
        }

        let source = &observation.source;
        let normalized_message = normalize_message(&source.message);
        let spans = source.spans.iter().map(wire_span).collect::<Vec<_>>();
        let producer = wire_producer(&source.producer);
        let occurrence_seed = json!({
            "producer": producer,
            "code": observation.code,
            "normalized_message": normalized_message,
            "spans": spans,
        });
        let semantic_seed = json!({
            "producer": wire_producer(&source.producer),
            "code": observation.code,
            "normalized_message": normalized_message,
        });

        let observation_id = digest(
            "observation_id_v1",
            &json!({
                "seed": seed,
                "duplicate_rank": duplicate_rank,
            }),
        )?;
        let occurrence_id = digest("occurrence_id_v1", &occurrence_seed)?;
        let semantic_id = digest("semantic_id_v1", &semantic_seed)?;
        let primary_span = source.spans.iter().position(|span| span.is_primary);
        let children = source.children.iter().map(wire_child).collect();

        diagnostics.push(DiagnosticRecord {
            observation_id,
            occurrence_id,
            semantic_id,
            context_id: None,
            producer,
            level_raw: source.raw_level.clone(),
            level: format_level(&source.diagnostic.level),
            code_raw: source.raw_code.clone(),
            code: observation.code.clone(),
            message: source.message.clone(),
            normalized_message,
            rendered: source.rendered.clone(),
            spans,
            primary_span,
            children,
        });
    }

    let mut contextual = contextual;
    if contextual.lint_config_hash.is_none() {
        contextual.lint_config_hash = analysis.scope.lint_config_hash.clone();
    }

    let provenance = AnalysisProvenance {
        hard: HardProvenance {
            diagnostic_format: "cargo-json".to_string(),
            command: analysis.execution.command.clone(),
            repository: analysis.scope.repository.clone(),
            revision: analysis.scope.revision.clone(),
            toolchain: analysis.scope.toolchain.clone(),
            target: analysis.scope.target.clone(),
            features: analysis.scope.features.clone(),
            package_selection: analysis.scope.package_selection.clone(),
            target_selection: analysis.scope.target_selection.clone(),
        },
        contextual,
    };
    let upstream = UpstreamEvidence {
        completion: completion_state(analysis),
        build_finished_seen: analysis.execution.build_finished_seen,
        build_success: analysis.execution.build_success,
        exit_code: analysis.execution.exit_code,
        duration_ms: analysis.execution.duration_ms,
    };
    let inventory_id = digest(
        "inventory_id_v1",
        &json!({
            "algorithm": INVENTORY_ID_ALGORITHM,
            "analysis": provenance,
            "upstream": upstream,
            "diagnostics": diagnostics.iter().map(|diagnostic| json!({
                "observation_id": diagnostic.observation_id,
                "occurrence_id": diagnostic.occurrence_id,
                "semantic_id": diagnostic.semantic_id,
            })).collect::<Vec<_>>(),
        }),
    )?;

    Ok(Inventory {
        schema: INVENTORY_SCHEMA_ID.to_string(),
        tool,
        analysis: provenance,
        upstream,
        inventory_id,
        summary: inventory_summary(&diagnostics),
        diagnostics,
    })
}

fn completion_state(analysis: &CargoAnalysis) -> CompletionState {
    match analysis.execution.completion {
        Some(crate::diagnostics::AnalysisCompletion::SuccessfulComplete) => {
            CompletionState::SuccessfulComplete
        }
        Some(crate::diagnostics::AnalysisCompletion::FailedComplete) => {
            CompletionState::FailedComplete
        }
        Some(crate::diagnostics::AnalysisCompletion::RuntimeFailure) => {
            CompletionState::RuntimeFailure
        }
        Some(crate::diagnostics::AnalysisCompletion::IncompleteStream) | None => {
            CompletionState::IncompleteStream
        }
    }
}

fn digest(label: &str, value: &Value) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(label.as_bytes());
    hasher.update([0]);
    hasher.update(bytes);
    Ok(format!("{label}:{}", hex::encode(hasher.finalize())))
}

fn observation_seed(observation: &InventoryObservation) -> Result<Value, serde_json::Error> {
    let source = &observation.source;
    Ok(json!({
        "producer": wire_producer(&source.producer),
        "raw_level": source.raw_level,
        "raw_code": source.raw_code,
        "message": source.message,
        "rendered": source.rendered,
        "code": observation.code,
        "spans": source.spans.iter().map(wire_span).collect::<Vec<_>>(),
        "children": source.children.iter().map(wire_child).collect::<Vec<_>>(),
    }))
}

fn wire_producer(producer: &ProducerUnit) -> WireProducerUnit {
    WireProducerUnit {
        package_id: producer.package_id.clone(),
        manifest_path: producer.manifest_path.clone(),
        profile: producer.profile.clone(),
        target: producer.target.as_ref().map(|target| WireCargoTarget {
            name: target.name.clone(),
            kind: target.kind.clone(),
            crate_types: target.crate_types.clone(),
            src_path: target.src_path.clone(),
            edition: target.edition.clone(),
        }),
    }
}

fn wire_span(span: &ObservationSpan) -> DiagnosticSpan {
    let normalized = span.normalized.as_ref();
    DiagnosticSpan {
        raw_file_name: span.raw_file_name.clone(),
        raw_line_start: span.raw_line_start,
        raw_line_end: span.raw_line_end,
        raw_column_start: span.raw_column_start,
        raw_column_end: span.raw_column_end,
        path: normalized.map(|value| value.file.as_str().to_string()),
        line_start: normalized.map(|value| value.line_start),
        line_end: normalized.map(|value| value.line_end),
        column_start: normalized.and_then(|value| value.col_start),
        column_end: normalized.and_then(|value| value.col_end),
        is_primary: span.is_primary,
    }
}

fn wire_child(child: &crate::diagnostics::DiagnosticChild) -> WireDiagnosticChild {
    WireDiagnosticChild {
        raw_level: child.raw_level.clone(),
        level: format_level(&child.level),
        message: child.message.clone(),
        rendered: child.rendered.clone(),
        spans: child.spans.iter().map(wire_span).collect(),
        suggestions: child
            .suggestions
            .iter()
            .map(|suggestion| DiagnosticSuggestion {
                file_name: suggestion.file_name.clone(),
                line_start: suggestion.line_start,
                line_end: suggestion.line_end,
                replacement: suggestion.replacement.clone(),
                applicability: suggestion.applicability.clone(),
            })
            .collect(),
    }
}

fn inventory_summary(diagnostics: &[DiagnosticRecord]) -> InventorySummary {
    let mut summary = InventorySummary {
        total: diagnostics.len().try_into().unwrap_or(u32::MAX),
        ..InventorySummary::default()
    };
    for diagnostic in diagnostics {
        match diagnostic.level.as_str() {
            "error" => summary.errors = summary.errors.saturating_add(1),
            "warning" => summary.warnings = summary.warnings.saturating_add(1),
            "note" => summary.notes = summary.notes.saturating_add(1),
            "help" => summary.helps = summary.helps.saturating_add(1),
            _ => summary.other = summary.other.saturating_add(1),
        }
    }
    summary
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
    use lintdiff_types::inventory::ContextualProvenance;
    use lintdiff_types::ToolInfo;
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

    #[test]
    fn public_inventory_is_stable_under_input_permutation() -> Result<(), Box<dyn std::error::Error>>
    {
        let first = r#"{"reason":"compiler-message","package_id":"pkg-b","target":{"name":"b"},"message":{"level":"error","message":"second","spans":[{"file_name":"src/b.rs","line_start":2,"line_end":2,"is_primary":true}]}}
{"reason":"compiler-message","package_id":"pkg-a","target":{"name":"a"},"message":{"level":"warning","message":"first","spans":[{"file_name":"src/a.rs","line_start":1,"line_end":1,"is_primary":true}]}}
{"reason":"build-finished","success":true}"#;
        let second = r#"{"reason":"compiler-message","package_id":"pkg-a","target":{"name":"a"},"message":{"level":"warning","message":"first","spans":[{"file_name":"src/a.rs","line_start":1,"line_end":1,"is_primary":true}]}}
{"reason":"compiler-message","package_id":"pkg-b","target":{"name":"b"},"message":{"level":"error","message":"second","spans":[{"file_name":"src/b.rs","line_start":2,"line_end":2,"is_primary":true}]}}
{"reason":"build-finished","success":true}"#;
        let tool = ToolInfo {
            name: "lintdiff".to_string(),
            version: "test".to_string(),
            commit: None,
        };
        let first = inventory_from_analysis(
            &parse_cargo_analysis(Cursor::new(first))?,
            tool.clone(),
            ContextualProvenance::default(),
        )?;
        let second = inventory_from_analysis(
            &parse_cargo_analysis(Cursor::new(second))?,
            tool,
            ContextualProvenance::default(),
        )?;

        assert_eq!(serde_json::to_vec(&first)?, serde_json::to_vec(&second)?);
        assert_eq!(first.inventory_id, second.inventory_id);
        assert_eq!(
            first.diagnostics[0].producer.package_id,
            Some("pkg-a".to_string())
        );
        Ok(())
    }

    #[test]
    fn public_inventory_preserves_unknown_locations_and_completion() {
        let input = r#"{"reason":"compiler-message","package_id":"pkg-a","manifest_path":"/repo/Cargo.toml","target":{"name":"demo","kind":["lib"],"crate_types":["lib"],"src_path":"/repo/src/lib.rs","edition":"2024"},"message":{"level":"warning","message":"unknown","spans":[{"file_name":"src/lib.rs","line_start":0,"is_primary":true}],"children":[{"level":"help","message":"replace it","spans":[{"file_name":"src/lib.rs","line_start":3,"line_end":3,"suggested_replacement":"new","suggestion_applicability":"MachineApplicable"}]}]}}
{"reason":"build-finished","success":false}"#;
        let analysis = parse_cargo_analysis(Cursor::new(input)).expect("valid analysis");
        let inventory = inventory_from_analysis(
            &analysis,
            ToolInfo {
                name: "lintdiff".to_string(),
                version: "test".to_string(),
                commit: None,
            },
            ContextualProvenance::default(),
        )
        .expect("inventory conversion");

        assert_eq!(
            inventory.upstream.completion,
            CompletionState::FailedComplete
        );
        assert_eq!(inventory.diagnostics[0].spans[0].raw_line_start, Some(0));
        assert_eq!(inventory.diagnostics[0].spans[0].line_start, None);
        assert_eq!(
            inventory.diagnostics[0]
                .producer
                .target
                .as_ref()
                .and_then(|target| target.name.as_deref()),
            Some("demo")
        );
        assert_eq!(
            inventory.diagnostics[0].children[0].suggestions[0]
                .replacement
                .as_deref(),
            Some("new")
        );
    }

    #[test]
    fn public_inventory_maps_runtime_failure_and_summary_levels() {
        let input = r#"{"reason":"compiler-message","message":{"level":"error","message":"error"}}
{"reason":"compiler-message","message":{"level":"note","message":"note"}}
{"reason":"compiler-message","message":{"level":"help","message":"help"}}
{"reason":"compiler-message","message":{"level":"custom","message":"other"}}"#;
        let analysis = parse_cargo_analysis(Cursor::new(input)).expect("valid analysis");
        let inventory = inventory_from_analysis(
            &analysis,
            ToolInfo {
                name: "lintdiff".to_string(),
                version: "test".to_string(),
                commit: None,
            },
            ContextualProvenance::default(),
        )
        .expect("inventory conversion");

        assert_eq!(
            inventory.upstream.completion,
            CompletionState::IncompleteStream
        );
        assert_eq!(inventory.summary.total, 4);
        assert_eq!(inventory.summary.errors, 1);
        assert_eq!(inventory.summary.notes, 1);
        assert_eq!(inventory.summary.helps, 1);
        assert_eq!(inventory.summary.other, 1);

        let runtime_failure = CargoAnalysis::runtime_failure(vec!["cargo".to_string()], Some(12));
        let inventory = inventory_from_analysis(
            &runtime_failure,
            ToolInfo {
                name: "lintdiff".to_string(),
                version: "test".to_string(),
                commit: None,
            },
            ContextualProvenance::default(),
        )
        .expect("runtime failure conversion");
        assert_eq!(
            inventory.upstream.completion,
            CompletionState::RuntimeFailure
        );
    }
}
