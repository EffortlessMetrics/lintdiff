use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::NormPath;

pub const SCHEMA_ID: &str = "lintdiff.report.v1";
pub const TOOL_NAME: &str = "lintdiff";
pub const CHECK_DIAGNOSTICS_ON_DIFF: &str = "diagnostics.on_diff";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub schema: String,
    pub tool: ToolInfo,
    pub run: RunInfo,
    pub verdict: Verdict,
    #[serde(default)]
    pub findings: Vec<Finding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunInfo {
    pub started_at: String,
    pub ended_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<HostInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostInfo {
    pub os: String,
    pub arch: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_base: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Verdict {
    pub status: VerdictStatus,
    pub counts: Counts,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerdictStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Counts {
    pub info: u32,
    pub warn: u32,
    pub error: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_id: Option<String>,
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Location {
    pub path: NormPath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,
}

/// Disposition of a single diagnostic through the ingest pipeline.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiagnosticDisposition {
    pub code: String,
    pub message_preview: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    pub disposition: Disposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
}

/// Why a diagnostic was included or excluded from the report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    Included,
    DroppedNoSpan,
    DroppedOutsideDiff,
    DroppedByPathFilter,
    SuppressedByCode,
    CutByBudget,
}

/// Summary counters for the explain artifact.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExplainSummary {
    pub total: u32,
    pub included: u32,
    pub dropped_no_span: u32,
    pub dropped_outside_diff: u32,
    pub dropped_by_path_filter: u32,
    pub suppressed_by_code: u32,
    pub cut_by_budget: u32,
}
