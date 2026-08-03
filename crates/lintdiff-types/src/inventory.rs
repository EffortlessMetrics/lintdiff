use serde::{Deserialize, Serialize};

use crate::ToolInfo;

pub const INVENTORY_SCHEMA_ID: &str = "lintdiff.inventory.v1";
pub const INVENTORY_ID_ALGORITHM: &str = "sha256-v1";

/// A complete, normalized diagnostic analysis before source scope or policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub schema: String,
    pub tool: ToolInfo,
    pub analysis: AnalysisProvenance,
    pub upstream: UpstreamEvidence,
    pub inventory_id: String,
    pub diagnostics: Vec<DiagnosticRecord>,
    pub summary: InventorySummary,
}

/// Provenance split between hard comparison inputs and contextual diagnostics.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisProvenance {
    pub hard: HardProvenance,
    pub contextual: ContextualProvenance,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardProvenance {
    pub diagnostic_format: String,
    pub command: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub features: Vec<String>,
    pub package_selection: Vec<String>,
    pub target_selection: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextualProvenance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cargo_lock_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lint_config_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lintdiff_config_hash: Option<String>,
    pub changed_manifests: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operating_system: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamEvidence {
    pub completion: CompletionState,
    pub build_finished_seen: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_success: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionState {
    SuccessfulComplete,
    FailedComplete,
    IncompleteStream,
    RuntimeFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticRecord {
    pub observation_id: String,
    pub occurrence_id: String,
    pub semantic_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    pub producer: ProducerUnit,
    pub level_raw: String,
    pub level: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_raw: Option<String>,
    pub code: String,
    pub message: String,
    pub normalized_message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rendered: Option<String>,
    pub spans: Vec<DiagnosticSpan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_span: Option<usize>,
    pub children: Vec<DiagnosticChild>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProducerUnit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<CargoTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoTarget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub kind: Vec<String>,
    pub crate_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticSpan {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_line_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_line_end: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_column_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_column_end: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column_end: Option<u32>,
    pub is_primary: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticChild {
    pub raw_level: String,
    pub level: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rendered: Option<String>,
    pub spans: Vec<DiagnosticSpan>,
    pub suggestions: Vec<DiagnosticSuggestion>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticSuggestion {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applicability: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventorySummary {
    pub total: u32,
    pub errors: u32,
    pub warnings: u32,
    pub notes: u32,
    pub helps: u32,
    pub other: u32,
}
