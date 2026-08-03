//! Versioned evidence for a base/head diagnostic comparison.

use serde::{Deserialize, Serialize};

use crate::inventory::DiagnosticRecord;

pub const DELTA_SCHEMA_ID: &str = "lintdiff.delta.v1";
pub const SOURCE_DIFF_ID_ALGORITHM: &str = "sha256-v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaReceipt {
    pub schema: String,
    pub base_inventory_id: String,
    pub head_inventory_id: String,
    pub source_diff_id: String,
    pub source_diff_algorithm: String,
    pub provenance: DeltaProvenance,
    pub items: Vec<DeltaItem>,
    pub summary: DeltaSummary,
    pub policy: DeltaPolicy,
    pub verdict: DeltaVerdict,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaProvenance {
    pub comparability: Comparability,
    pub contextual_changes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comparability {
    pub status: ComparabilityStatus,
    pub reasons: Vec<DeltaReason>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparabilityStatus {
    Comparable,
    Incomparable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeltaReason {
    BaseAnalysisIncomplete,
    HeadAnalysisIncomplete,
    HardScopeMismatch,
    MultipleCandidates,
    NoEarnedCorrespondence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaItem {
    #[serde(flatten)]
    pub pairing: PairingEvidence,
    pub change_kind: Option<DeltaKind>,
    pub diff_scope: DiffScope,
    pub match_basis: MatchBasis,
    pub movement: Movement,
    pub label: Option<DeltaLabel>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum PairingEvidence {
    Matched {
        base: Box<DiagnosticRecord>,
        head: Box<DiagnosticRecord>,
        basis: MatchBasis,
    },
    BaseOnly {
        base: Box<DiagnosticRecord>,
    },
    HeadOnly {
        head: Box<DiagnosticRecord>,
    },
    Ambiguous {
        base_candidates: Vec<DiagnosticRecord>,
        head_candidates: Vec<DiagnosticRecord>,
        reasons: Vec<DeltaReason>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeltaKind {
    Unchanged,
    New,
    Resolved,
    Modified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffScope {
    Touched,
    Untouched,
    NoLocation,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchBasis {
    Exact,
    LineMapped,
    RenameMapped,
    Semantic,
    Context,
    ModifiedContext,
    None,
    Ambiguous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Movement {
    Same,
    Shifted,
    Renamed,
    ShiftedAndRenamed,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeltaLabel {
    NewOnDiff,
    NewOffDiff,
    ExistingTouched,
    ExistingUntouched,
    Resolved,
    Modified,
    Ambiguous,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaSummary {
    pub total: u32,
    pub unchanged: u32,
    pub new: u32,
    pub resolved: u32,
    pub modified: u32,
    pub ambiguous: u32,
    pub touched: u32,
    pub untouched: u32,
    pub no_location: u32,
    pub unknown_scope: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaPolicy {
    pub profile: DeltaProfile,
    pub block_new_errors: bool,
    pub block_new_warnings: bool,
    pub block_ambiguous: bool,
}

impl Default for DeltaPolicy {
    fn default() -> Self {
        Self {
            profile: DeltaProfile::Advisory,
            block_new_errors: false,
            block_new_warnings: false,
            block_ambiguous: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeltaProfile {
    Advisory,
    Strict,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaVerdict {
    pub status: DeltaVerdictStatus,
    pub reasons: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeltaVerdictStatus {
    Accepted,
    Rejected,
    Incomparable,
}
