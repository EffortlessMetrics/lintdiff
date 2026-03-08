use std::io::Cursor;

pub use lintdiff_bdd_grid::{FeatureFlagGrid, FeatureFlagGridRow};
use lintdiff_core::{ingest_on_diff, IngestOnDiffParams};
use lintdiff_diagnostics::parse_cargo_messages;
use lintdiff_diff::parse_unified_diff;
use lintdiff_feature_flags::set_feature_flag_by_name_and_value;
use lintdiff_types::{LintdiffConfig, NormPath, Report, RunInfo, ToolInfo, TOOL_NAME};

const TEST_TOOL_VERSION: &str = "test";
const TEST_RUN_STARTED_AT: &str = "2026-01-01T00:00:00Z";
const TEST_RUN_ENDED_AT: &str = "2026-01-01T00:00:01Z";
const TEST_REPO_ROOT: &str = "/repo";

/// Single deterministic row execution result for grid-driven BDD scenarios.
#[derive(Clone, Debug)]
pub struct GridRunResult {
    pub row: FeatureFlagGridRow,
    pub report: Report,
}

/// Read fixture files from the default test fixture directory.
pub fn read_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{name}");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"))
}

/// Apply a feature flag value to a config for BDD scenarios.
pub fn apply_feature_flag_value(
    config: &mut LintdiffConfig,
    flag: &str,
    value: &str,
) -> Result<(), String> {
    set_feature_flag_by_name_and_value(&mut config.feature_flags, flag, value)
}

/// Apply multiple feature-flag assignments to a scenario config.
pub fn apply_feature_flag_assignments(
    config: &mut LintdiffConfig,
    assignments: &[String],
) -> Result<(), String> {
    lintdiff_feature_flags::set_feature_flags_from_assignments(
        &mut config.feature_flags,
        assignments.iter(),
    )
}

/// Apply one grid row to `config` and run a deterministic fixture ingestion.
pub fn apply_feature_flag_grid_row(
    config: &mut LintdiffConfig,
    row: &FeatureFlagGridRow,
) -> Result<(), String> {
    row.apply_to_flags(&mut config.feature_flags)
}

/// Run the ingest pipeline from in-memory fixtures.
///
/// This keeps BDD scenario files focused on behavior instead of orchestration.
pub fn run_ingest_from_fixtures(
    diff: &Option<String>,
    diagnostics: &Option<String>,
    config: &LintdiffConfig,
) -> Report {
    let tool = ToolInfo {
        name: TOOL_NAME.to_string(),
        version: TEST_TOOL_VERSION.to_string(),
        commit: None,
    };

    let run = RunInfo {
        started_at: TEST_RUN_STARTED_AT.to_string(),
        ended_at: TEST_RUN_ENDED_AT.to_string(),
        duration_ms: None,
        host: None,
        git: None,
    };

    let diff_map = diff.as_ref().map(|d| parse_unified_diff(d).unwrap());
    let diagnostics = diagnostics
        .as_ref()
        .map(|d| parse_cargo_messages(Cursor::new(d.as_bytes())).unwrap());

    ingest_on_diff(IngestOnDiffParams {
        tool,
        run,
        host: None,
        git: None,
        diff_map,
        diagnostics,
        repo_root: Some(NormPath::new(TEST_REPO_ROOT)),
        config: config.effective(),
        repro: None,
    })
}

/// Run fixture ingestion after applying feature-flag assignments.
pub fn run_ingest_from_fixtures_with_flags(
    diff: &Option<String>,
    diagnostics: &Option<String>,
    config: &LintdiffConfig,
    assignments: &[String],
) -> Result<Report, String> {
    let mut cfg = config.clone();
    apply_feature_flag_assignments(&mut cfg, assignments)?;
    Ok(run_ingest_from_fixtures(diff, diagnostics, &cfg))
}

/// Apply a scenario-grid row and run a fixture ingestion.
pub fn run_ingest_from_fixtures_with_grid_row(
    diff: &Option<String>,
    diagnostics: &Option<String>,
    config: &LintdiffConfig,
    row: &FeatureFlagGridRow,
) -> Result<Report, String> {
    let mut cfg = config.clone();
    apply_feature_flag_grid_row(&mut cfg, row)?;
    Ok(run_ingest_from_fixtures(diff, diagnostics, &cfg))
}

/// Run fixture ingestion for every row in a feature-flag grid.
pub fn run_ingest_from_fixtures_with_grid(
    diff: &Option<String>,
    diagnostics: &Option<String>,
    config: &LintdiffConfig,
    grid: &FeatureFlagGrid,
) -> Result<Vec<Report>, String> {
    grid.rows()
        .iter()
        .map(|row| run_ingest_from_fixtures_with_grid_row(diff, diagnostics, config, row))
        .collect()
}

/// Run fixture ingestion for every row in a feature-flag grid with row-level context.
pub fn run_ingest_from_fixtures_with_grid_rows(
    diff: &Option<String>,
    diagnostics: &Option<String>,
    config: &LintdiffConfig,
    grid: &FeatureFlagGrid,
) -> Result<Vec<GridRunResult>, String> {
    let mut outputs = Vec::with_capacity(grid.rows().len());
    for row in grid.rows() {
        outputs.push(GridRunResult {
            row: row.clone(),
            report: run_ingest_from_fixtures_with_grid_row(diff, diagnostics, config, row)?,
        });
    }
    Ok(outputs)
}

/// Convert a report verdict into the BDD assertion string form used by `.feature` files.
pub fn verdict_status(report: &Report) -> &'static str {
    match report.verdict.status {
        lintdiff_types::VerdictStatus::Pass => "pass",
        lintdiff_types::VerdictStatus::Warn => "warn",
        lintdiff_types::VerdictStatus::Fail => "fail",
        lintdiff_types::VerdictStatus::Skip => "skip",
    }
}
