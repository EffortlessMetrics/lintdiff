//! Deterministic diagnostic pairing with explicit uncertainty.

use std::collections::BTreeSet;

use lintdiff_types::delta::Movement;
use lintdiff_types::inventory::{DiagnosticRecord, Inventory};

use crate::source::{LocationMapping, SourceChangeSet};
use lintdiff_types::NormPath;

/// A stable reference to one complete inventory observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticRef {
    pub observation_id: String,
    pub occurrence_id: String,
    pub semantic_id: String,
    pub diagnostic: DiagnosticRecord,
}

impl DiagnosticRef {
    fn from_record(diagnostic: &DiagnosticRecord) -> Self {
        Self {
            observation_id: diagnostic.observation_id.clone(),
            occurrence_id: diagnostic.occurrence_id.clone(),
            semantic_id: diagnostic.semantic_id.clone(),
            diagnostic: diagnostic.clone(),
        }
    }
}

/// The evidence basis for a confident pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchBasis {
    ExactOccurrence,
    LineMapped,
    RenameMapped,
    SemanticUnique,
    ContextUnique,
    ModifiedContextUnique,
}

/// Reasons retained when a candidate set cannot support a confident pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum ReasonCode {
    BaseAnalysisIncomplete,
    HeadAnalysisIncomplete,
    HardScopeMismatch,
    MultipleCandidates,
    NoEarnedCorrespondence,
}

/// Pairing evidence. This type intentionally has no change classification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PairingEvidence {
    Matched {
        base: Box<DiagnosticRef>,
        head: Box<DiagnosticRef>,
        basis: MatchBasis,
    },
    BaseOnly {
        base: DiagnosticRef,
    },
    HeadOnly {
        head: DiagnosticRef,
    },
    Ambiguous {
        base_candidates: Vec<DiagnosticRef>,
        head_candidates: Vec<DiagnosticRef>,
        reasons: Vec<ReasonCode>,
    },
}

/// Whether the two analyses support the first full comparison contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComparabilityStatus {
    Comparable,
    Incomparable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comparability {
    pub status: ComparabilityStatus,
    pub reasons: Vec<ReasonCode>,
}

/// Contextual provenance changes retained separately from hard scope checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextualChange {
    pub field: String,
}

/// The pure result of pairing two complete inventories.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticComparison {
    pub comparability: Comparability,
    pub contextual_changes: Vec<ContextualChange>,
    pub evidence: Vec<PairingEvidence>,
}

/// Pair two inventories without applying repository policy or producing a verdict.
pub fn compare_inventories(
    base: &Inventory,
    head: &Inventory,
    source: &SourceChangeSet,
) -> DiagnosticComparison {
    let contextual_changes = contextual_changes(base, head);
    let reasons = comparability_reasons(base, head);
    if !reasons.is_empty() {
        return DiagnosticComparison {
            comparability: Comparability {
                status: ComparabilityStatus::Incomparable,
                reasons,
            },
            contextual_changes,
            evidence: Vec::new(),
        };
    }

    let base = sorted_refs(base);
    let head = sorted_refs(head);
    let mut base_used = vec![false; base.len()];
    let mut head_used = vec![false; head.len()];
    let mut evidence = Vec::new();

    evidence.extend(pair_unique(
        &base,
        &head,
        &mut base_used,
        &mut head_used,
        |base, head| same_producer(base, head) && base.occurrence_id == head.occurrence_id,
        |_, _| MatchBasis::ExactOccurrence,
    ));
    evidence.extend(pair_unique(
        &base,
        &head,
        &mut base_used,
        &mut head_used,
        |base, head| {
            same_producer(base, head)
                && base.diagnostic.code == head.diagnostic.code
                && base.diagnostic.normalized_message == head.diagnostic.normalized_message
                && location_relation(base, head, source).is_some()
        },
        mapped_basis,
    ));
    evidence.extend(pair_unique(
        &base,
        &head,
        &mut base_used,
        &mut head_used,
        |base, head| {
            same_producer(base, head)
                && base.semantic_id == head.semantic_id
                && location_relation(base, head, source).is_some()
        },
        |_, _| MatchBasis::SemanticUnique,
    ));
    evidence.extend(pair_unique(
        &base,
        &head,
        &mut base_used,
        &mut head_used,
        |base, head| {
            same_producer(base, head)
                && base.diagnostic.context_id.is_some()
                && base.diagnostic.context_id == head.diagnostic.context_id
                && location_relation(base, head, source).is_some()
        },
        |_, _| MatchBasis::ContextUnique,
    ));
    evidence.extend(pair_unique(
        &base,
        &head,
        &mut base_used,
        &mut head_used,
        |base, head| {
            same_producer(base, head)
                && base.diagnostic.code == head.diagnostic.code
                && base.semantic_id != head.semantic_id
                && location_relation(base, head, source).is_some()
        },
        |_, _| MatchBasis::ModifiedContextUnique,
    ));

    evidence.extend(ambiguous_groups(
        &base,
        &head,
        &mut base_used,
        &mut head_used,
        source,
    ));
    for (index, diagnostic) in base.iter().enumerate() {
        if !base_used[index] {
            evidence.push(PairingEvidence::BaseOnly {
                base: diagnostic.clone(),
            });
        }
    }
    for (index, diagnostic) in head.iter().enumerate() {
        if !head_used[index] {
            evidence.push(PairingEvidence::HeadOnly {
                head: diagnostic.clone(),
            });
        }
    }
    evidence.sort_by_key(evidence_key);

    DiagnosticComparison {
        comparability: Comparability {
            status: ComparabilityStatus::Comparable,
            reasons: Vec::new(),
        },
        contextual_changes,
        evidence,
    }
}

fn mapped_basis(base: &DiagnosticRef, head: &DiagnosticRef) -> MatchBasis {
    match (base.diagnostic_path(), head.diagnostic_path()) {
        (Some((base_path, _)), Some((head_path, _))) if base_path != head_path => {
            MatchBasis::RenameMapped
        }
        _ => MatchBasis::LineMapped,
    }
}

fn pair_unique<F, B>(
    base: &[DiagnosticRef],
    head: &[DiagnosticRef],
    base_used: &mut [bool],
    head_used: &mut [bool],
    matches: F,
    basis: B,
) -> Vec<PairingEvidence>
where
    F: Fn(&DiagnosticRef, &DiagnosticRef) -> bool,
    B: Fn(&DiagnosticRef, &DiagnosticRef) -> MatchBasis,
{
    let mut evidence = Vec::new();
    for base_index in 0..base.len() {
        if base_used[base_index] {
            continue;
        }
        let candidates = (0..head.len())
            .filter(|&head_index| {
                !head_used[head_index] && matches(&base[base_index], &head[head_index])
            })
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            continue;
        }
        let head_index = candidates[0];
        let reverse_count = (0..base.len())
            .filter(|&other| !base_used[other] && matches(&base[other], &head[head_index]))
            .count();
        if reverse_count != 1 {
            continue;
        }
        base_used[base_index] = true;
        head_used[head_index] = true;
        evidence.push(PairingEvidence::Matched {
            base: Box::new(base[base_index].clone()),
            head: Box::new(head[head_index].clone()),
            basis: basis(&base[base_index], &head[head_index]),
        });
    }
    evidence
}

fn ambiguous_groups(
    base: &[DiagnosticRef],
    head: &[DiagnosticRef],
    base_used: &mut [bool],
    head_used: &mut [bool],
    source: &SourceChangeSet,
) -> Vec<PairingEvidence> {
    let mut evidence = Vec::new();
    for start in 0..base.len() {
        if base_used[start] {
            continue;
        }
        let initial_heads = candidate_heads(&base[start], head, head_used, source);
        if initial_heads.is_empty() {
            continue;
        }
        let mut base_group = BTreeSet::from([start]);
        let mut head_group = initial_heads.into_iter().collect::<BTreeSet<_>>();
        loop {
            let mut changed = false;
            for base_index in 0..base.len() {
                if !base_used[base_index]
                    && !base_group.contains(&base_index)
                    && head_group.iter().any(|&head_index| {
                        candidate_pair(&base[base_index], &head[head_index], source)
                    })
                {
                    changed |= base_group.insert(base_index);
                }
            }
            for head_index in 0..head.len() {
                if !head_used[head_index]
                    && !head_group.contains(&head_index)
                    && base_group.iter().any(|&base_index| {
                        candidate_pair(&base[base_index], &head[head_index], source)
                    })
                {
                    changed |= head_group.insert(head_index);
                }
            }
            if !changed {
                break;
            }
        }

        for &base_index in &base_group {
            base_used[base_index] = true;
        }
        for &head_index in &head_group {
            head_used[head_index] = true;
        }
        let mut reasons = vec![ReasonCode::NoEarnedCorrespondence];
        if base_group.len() > 1 || head_group.len() > 1 {
            reasons.push(ReasonCode::MultipleCandidates);
            reasons.sort_unstable();
        }
        evidence.push(PairingEvidence::Ambiguous {
            base_candidates: base_group
                .into_iter()
                .map(|index| base[index].clone())
                .collect(),
            head_candidates: head_group
                .into_iter()
                .map(|index| head[index].clone())
                .collect(),
            reasons,
        });
    }
    evidence
}

fn candidate_heads(
    base: &DiagnosticRef,
    head: &[DiagnosticRef],
    head_used: &[bool],
    source: &SourceChangeSet,
) -> Vec<usize> {
    (0..head.len())
        .filter(|&index| !head_used[index] && candidate_pair(base, &head[index], source))
        .collect()
}

fn candidate_pair(base: &DiagnosticRef, head: &DiagnosticRef, source: &SourceChangeSet) -> bool {
    same_producer(base, head)
        && (base.semantic_id == head.semantic_id || base.diagnostic.code == head.diagnostic.code)
        && (source.files.is_empty() || location_relation(base, head, source).is_some())
}

fn sorted_refs(inventory: &Inventory) -> Vec<DiagnosticRef> {
    let mut refs = inventory
        .diagnostics
        .iter()
        .map(DiagnosticRef::from_record)
        .collect::<Vec<_>>();
    refs.sort_by(|left, right| left.observation_id.cmp(&right.observation_id));
    refs
}

fn same_producer(base: &DiagnosticRef, head: &DiagnosticRef) -> bool {
    base.diagnostic.producer == head.diagnostic.producer
}

fn location_relation(
    base: &DiagnosticRef,
    head: &DiagnosticRef,
    source: &SourceChangeSet,
) -> Option<MatchMovement> {
    let base_location = base.diagnostic_path();
    let head_location = head.diagnostic_path();
    match (base_location, head_location) {
        (None, None) => Some(MatchMovement::Unknown),
        (Some((base_path, base_line)), Some((head_path, head_line)))
            if base_path == head_path && base_line == head_line =>
        {
            Some(MatchMovement::Same)
        }
        (Some((base_path, base_line)), Some((head_path, head_line))) => {
            match source.map_old_location(&base_path, base_line) {
                LocationMapping::Exact { new_path, new_line }
                    if new_path == head_path && new_line == head_line =>
                {
                    Some(MatchMovement::Same)
                }
                LocationMapping::Shifted { new_path, new_line }
                    if new_path == head_path && new_line == head_line =>
                {
                    Some(MatchMovement::Shifted)
                }
                LocationMapping::Renamed {
                    new_path, new_line, ..
                } if new_path == head_path && new_line == head_line => Some(MatchMovement::Renamed),
                LocationMapping::ShiftedAndRenamed {
                    new_path, new_line, ..
                } if new_path == head_path && new_line == head_line => {
                    Some(MatchMovement::ShiftedAndRenamed)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MatchMovement {
    Same,
    Shifted,
    Renamed,
    ShiftedAndRenamed,
    Unknown,
}

impl DiagnosticRef {
    pub(crate) fn diagnostic_path(&self) -> Option<(NormPath, u32)> {
        let span = self
            .diagnostic
            .primary_span
            .and_then(|index| self.diagnostic.spans.get(index))
            .or_else(|| {
                self.diagnostic
                    .spans
                    .iter()
                    .find(|span| span.path.is_some())
            })?;
        Some((
            NormPath::from_repo_path(span.path.as_deref()?),
            span.line_start?,
        ))
    }
}

pub(crate) fn movement_for_pair(
    base: &DiagnosticRef,
    head: &DiagnosticRef,
    source: &SourceChangeSet,
) -> Movement {
    match location_relation(base, head, source) {
        Some(MatchMovement::Same) => Movement::Same,
        Some(MatchMovement::Shifted) => Movement::Shifted,
        Some(MatchMovement::Renamed) => Movement::Renamed,
        Some(MatchMovement::ShiftedAndRenamed) => Movement::ShiftedAndRenamed,
        Some(MatchMovement::Unknown) | None => Movement::Unknown,
    }
}

fn comparability_reasons(base: &Inventory, head: &Inventory) -> Vec<ReasonCode> {
    let mut reasons = Vec::new();
    if base.upstream.completion != lintdiff_types::inventory::CompletionState::SuccessfulComplete {
        reasons.push(ReasonCode::BaseAnalysisIncomplete);
    }
    if head.upstream.completion != lintdiff_types::inventory::CompletionState::SuccessfulComplete {
        reasons.push(ReasonCode::HeadAnalysisIncomplete);
    }
    if base.analysis.hard.diagnostic_format != head.analysis.hard.diagnostic_format
        || base.analysis.hard.command != head.analysis.hard.command
        || base.analysis.hard.repository != head.analysis.hard.repository
        || base.analysis.hard.toolchain != head.analysis.hard.toolchain
        || base.analysis.hard.target != head.analysis.hard.target
        || base.analysis.hard.features != head.analysis.hard.features
        || base.analysis.hard.package_selection != head.analysis.hard.package_selection
        || base.analysis.hard.target_selection != head.analysis.hard.target_selection
    {
        reasons.push(ReasonCode::HardScopeMismatch);
    }
    reasons.sort_unstable();
    reasons.dedup();
    reasons
}

fn contextual_changes(base: &Inventory, head: &Inventory) -> Vec<ContextualChange> {
    let mut changes = Vec::new();
    if base.analysis.hard.revision != head.analysis.hard.revision {
        changes.push(ContextualChange {
            field: "revision".to_string(),
        });
    }
    let left = &base.analysis.contextual;
    let right = &head.analysis.contextual;
    if left.cargo_lock_hash != right.cargo_lock_hash {
        changes.push(ContextualChange {
            field: "cargo_lock_hash".to_string(),
        });
    }
    if left.lint_config_hash != right.lint_config_hash {
        changes.push(ContextualChange {
            field: "lint_config_hash".to_string(),
        });
    }
    if left.lintdiff_config_hash != right.lintdiff_config_hash {
        changes.push(ContextualChange {
            field: "lintdiff_config_hash".to_string(),
        });
    }
    if left.changed_manifests != right.changed_manifests {
        changes.push(ContextualChange {
            field: "changed_manifests".to_string(),
        });
    }
    if left.workflow != right.workflow {
        changes.push(ContextualChange {
            field: "workflow".to_string(),
        });
    }
    if left.operating_system != right.operating_system {
        changes.push(ContextualChange {
            field: "operating_system".to_string(),
        });
    }
    if left.architecture != right.architecture {
        changes.push(ContextualChange {
            field: "architecture".to_string(),
        });
    }
    changes
}

fn evidence_key(evidence: &PairingEvidence) -> String {
    match evidence {
        PairingEvidence::Matched { base, .. } => base.observation_id.clone(),
        PairingEvidence::BaseOnly { base } => base.observation_id.clone(),
        PairingEvidence::HeadOnly { head } => head.observation_id.clone(),
        PairingEvidence::Ambiguous {
            base_candidates,
            head_candidates,
            ..
        } => base_candidates
            .first()
            .map(|candidate| candidate.observation_id.clone())
            .or_else(|| {
                head_candidates
                    .first()
                    .map(|candidate| candidate.observation_id.clone())
            })
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintdiff_types::inventory::{
        AnalysisProvenance, CompletionState, HardProvenance, InventorySummary, ProducerUnit,
        UpstreamEvidence,
    };
    use proptest::prelude::*;

    fn inventory(records: Vec<DiagnosticRecord>) -> Inventory {
        Inventory {
            schema: "lintdiff.inventory.v1".to_string(),
            tool: lintdiff_types::ToolInfo {
                name: "lintdiff".to_string(),
                version: "test".to_string(),
                commit: None,
            },
            analysis: AnalysisProvenance {
                hard: HardProvenance {
                    diagnostic_format: "cargo-json".to_string(),
                    command: vec!["cargo".to_string(), "clippy".to_string()],
                    repository: Some("repo".to_string()),
                    toolchain: Some("rustc".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            upstream: UpstreamEvidence {
                completion: CompletionState::SuccessfulComplete,
                build_finished_seen: true,
                build_success: Some(true),
                exit_code: Some(0),
                duration_ms: None,
            },
            inventory_id: "inventory_id_v1:test".to_string(),
            summary: InventorySummary::default(),
            diagnostics: records,
        }
    }

    fn diagnostic(
        id: &str,
        occurrence: &str,
        semantic: &str,
        path: &str,
        line: u32,
    ) -> DiagnosticRecord {
        DiagnosticRecord {
            observation_id: format!("observation_id_v1:{id}"),
            occurrence_id: format!("occurrence_id_v1:{occurrence}"),
            semantic_id: format!("semantic_id_v1:{semantic}"),
            context_id: None,
            producer: ProducerUnit {
                package_id: Some("pkg".to_string()),
                ..Default::default()
            },
            level_raw: "warning".to_string(),
            level: "warning".to_string(),
            code_raw: Some("W".to_string()),
            code: "W".to_string(),
            message: "message".to_string(),
            normalized_message: "message".to_string(),
            rendered: None,
            spans: vec![lintdiff_types::inventory::DiagnosticSpan {
                raw_file_name: Some(path.to_string()),
                raw_line_start: Some(line),
                raw_line_end: Some(line),
                raw_column_start: None,
                raw_column_end: None,
                path: Some(path.to_string()),
                line_start: Some(line),
                line_end: Some(line),
                column_start: None,
                column_end: None,
                is_primary: true,
            }],
            primary_span: Some(0),
            children: Vec::new(),
        }
    }

    #[test]
    fn exact_occurrence_pairs_and_input_order_does_not_matter() {
        let base_record = diagnostic("a", "same", "semantic", "src/lib.rs", 2);
        let head_record = base_record.clone();
        let first = compare_inventories(
            &inventory(vec![base_record.clone()]),
            &inventory(vec![head_record.clone()]),
            &SourceChangeSet::default(),
        );
        let second = compare_inventories(
            &inventory(vec![base_record]),
            &inventory(vec![head_record]),
            &SourceChangeSet::default(),
        );
        assert_eq!(first, second);
        assert!(matches!(
            first.evidence.as_slice(),
            [PairingEvidence::Matched {
                basis: MatchBasis::ExactOccurrence,
                ..
            }]
        ));
    }

    #[test]
    fn source_mapping_pairs_shifted_diagnostics() {
        let base = diagnostic("base", "old", "semantic", "src/lib.rs", 3);
        let head = diagnostic("head", "new", "semantic", "src/lib.rs", 5);
        let source = crate::parse_source_change_set(
            "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -3,0 +3,2 @@\n+one\n+two\n",
        )
        .expect("valid source diff");
        let result = compare_inventories(&inventory(vec![base]), &inventory(vec![head]), &source);
        assert!(matches!(
            result.evidence.as_slice(),
            [PairingEvidence::Matched {
                basis: MatchBasis::LineMapped,
                ..
            }]
        ));
    }

    #[test]
    fn source_mapping_pairs_renamed_diagnostics() {
        let base = diagnostic("base", "old", "semantic", "src/old.rs", 2);
        let head = diagnostic("head", "new", "semantic", "src/new.rs", 2);
        let source = crate::parse_source_change_set(
            "diff --git a/src/old.rs b/src/new.rs\nsimilarity index 100%\nrename from src/old.rs\nrename to src/new.rs\n",
        )
        .expect("valid rename diff");
        let result = compare_inventories(&inventory(vec![base]), &inventory(vec![head]), &source);
        assert!(matches!(
            result.evidence.as_slice(),
            [PairingEvidence::Matched {
                basis: MatchBasis::RenameMapped,
                ..
            }]
        ));
    }

    #[test]
    fn unique_semantic_identity_pairs_after_message_change() {
        let base = diagnostic("base", "old", "semantic", "src/lib.rs", 2);
        let mut head = diagnostic("head", "new", "semantic", "src/lib.rs", 2);
        head.normalized_message = "changed".to_string();
        let result = compare_inventories(
            &inventory(vec![base]),
            &inventory(vec![head]),
            &SourceChangeSet::default(),
        );
        assert!(matches!(
            result.evidence.as_slice(),
            [PairingEvidence::Matched {
                basis: MatchBasis::SemanticUnique,
                ..
            }]
        ));
    }

    #[test]
    fn unique_context_identity_pairs_without_semantic_or_code_match() {
        let mut base = diagnostic("base", "old", "base-semantic", "src/lib.rs", 2);
        base.context_id = Some("context".to_string());
        let mut head = diagnostic("head", "new", "head-semantic", "src/lib.rs", 2);
        head.context_id = Some("context".to_string());
        head.code = "E".to_string();
        head.normalized_message = "changed".to_string();
        let result = compare_inventories(
            &inventory(vec![base]),
            &inventory(vec![head]),
            &SourceChangeSet::default(),
        );
        assert!(matches!(
            result.evidence.as_slice(),
            [PairingEvidence::Matched {
                basis: MatchBasis::ContextUnique,
                ..
            }]
        ));
    }

    #[test]
    fn modified_context_pairs_same_code_with_changed_semantics() {
        let base = diagnostic("base", "old", "base-semantic", "src/lib.rs", 2);
        let mut head = diagnostic("head", "new", "head-semantic", "src/lib.rs", 2);
        head.normalized_message = "changed".to_string();
        let result = compare_inventories(
            &inventory(vec![base]),
            &inventory(vec![head]),
            &SourceChangeSet::default(),
        );
        assert!(matches!(
            result.evidence.as_slice(),
            [PairingEvidence::Matched {
                basis: MatchBasis::ModifiedContextUnique,
                ..
            }]
        ));
    }

    #[test]
    fn contextual_provenance_changes_are_retained_without_blocking_pairing() {
        let record = diagnostic("same", "same", "semantic", "src/lib.rs", 2);
        let mut head = inventory(vec![record.clone()]);
        head.analysis.hard.revision = Some("head".to_string());
        head.analysis.contextual.workflow = Some("changed".to_string());
        let result =
            compare_inventories(&inventory(vec![record]), &head, &SourceChangeSet::default());
        assert_eq!(result.comparability.status, ComparabilityStatus::Comparable);
        assert_eq!(
            result
                .contextual_changes
                .iter()
                .map(|change| change.field.as_str())
                .collect::<Vec<_>>(),
            ["revision", "workflow"]
        );
    }

    #[test]
    fn duplicate_candidates_are_ambiguous_and_not_classified() {
        let base = vec![
            diagnostic("base-a", "same", "semantic", "src/lib.rs", 2),
            diagnostic("base-b", "same", "semantic", "src/lib.rs", 4),
        ];
        let head = vec![diagnostic("head", "same", "semantic", "src/lib.rs", 3)];
        let result = compare_inventories(
            &inventory(base),
            &inventory(head),
            &SourceChangeSet::default(),
        );
        assert!(matches!(
            result.evidence.as_slice(),
            [PairingEvidence::Ambiguous { .. }]
        ));
    }

    #[test]
    fn producer_units_do_not_cross_pair() {
        let base = diagnostic("base", "same", "semantic", "src/lib.rs", 2);
        let mut head = diagnostic("head", "same", "semantic", "src/lib.rs", 2);
        head.producer.package_id = Some("other".to_string());
        let result = compare_inventories(
            &inventory(vec![base.clone()]),
            &inventory(vec![head.clone()]),
            &SourceChangeSet::default(),
        );
        assert!(matches!(
            result.evidence[0],
            PairingEvidence::BaseOnly { .. }
        ));
        assert!(matches!(
            result.evidence[1],
            PairingEvidence::HeadOnly { .. }
        ));
    }

    proptest! {
        #[test]
        fn exact_pairing_consumes_each_observation_once(count in 0usize..16) {
            let base_records = (0..count)
                .map(|index| {
                    let id = index.to_string();
                    diagnostic(&id, &format!("occurrence-{id}"), &format!("semantic-{id}"), "src/lib.rs", index as u32 + 1)
                })
                .collect::<Vec<_>>();
            let mut head_records = base_records.clone();
            head_records.reverse();

            let result = compare_inventories(
                &inventory(base_records.clone()),
                &inventory(head_records),
                &SourceChangeSet::default(),
            );
            let mut matched_ids = BTreeSet::new();
            for evidence in &result.evidence {
                match evidence {
                    PairingEvidence::Matched { base, head, .. } => {
                        prop_assert_eq!(&base.observation_id, &head.observation_id);
                        prop_assert!(matched_ids.insert(base.observation_id.clone()));
                    }
                    other => prop_assert!(false, "unexpected evidence: {other:?}"),
                }
            }
            prop_assert_eq!(matched_ids.len(), count);
            prop_assert_eq!(matched_ids.len(), base_records.len());
        }
    }

    #[test]
    fn incomplete_and_hard_scope_mismatch_are_incomparable() {
        let record = diagnostic("a", "same", "semantic", "src/lib.rs", 2);
        let mut head = inventory(vec![record.clone()]);
        head.upstream.completion = CompletionState::IncompleteStream;
        let result = compare_inventories(
            &inventory(vec![record.clone()]),
            &head,
            &SourceChangeSet::default(),
        );
        assert_eq!(
            result.comparability.status,
            ComparabilityStatus::Incomparable
        );
        assert!(result.evidence.is_empty());
        let mut mismatched = inventory(vec![record]);
        mismatched
            .analysis
            .hard
            .command
            .push("--all-targets".to_string());
        let result = compare_inventories(
            &inventory(Vec::new()),
            &mismatched,
            &SourceChangeSet::default(),
        );
        assert!(result
            .comparability
            .reasons
            .contains(&ReasonCode::HardScopeMismatch));
    }
}
