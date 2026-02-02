use std::fs;
use std::io::Cursor;

use cucumber::{given, then, when, World as _};

use lintdiff_diff::parse_unified_diff;
use lintdiff_diagnostics::parse_cargo_messages;
use lintdiff_domain::{ingest_on_diff, IngestOnDiffParams};
use lintdiff_types::{LintdiffConfig, NormPath, Report, RunInfo, ToolInfo, TOOL_NAME};

#[derive(Debug, Default, cucumber::World)]
struct LintdiffWorld {
    diff: Option<String>,
    diagnostics: Option<String>,
    config: LintdiffConfig,
    report: Option<Report>,
}

fn read_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{name}");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"))
}

#[given(expr = "a diff fixture {string}")]
async fn given_diff(world: &mut LintdiffWorld, name: String) {
    world.diff = Some(read_fixture(&name));
}

#[given(expr = "a diagnostics fixture {string}")]
async fn given_diagnostics(world: &mut LintdiffWorld, name: String) {
    world.diagnostics = Some(read_fixture(&name));
}

#[given(expr = "deny code {string}")]
async fn deny_code(world: &mut LintdiffWorld, code: String) {
    world.config.filter.deny_codes.push(code);
}

#[when("lintdiff ingests the inputs")]
async fn when_ingest(world: &mut LintdiffWorld) {
    let tool = ToolInfo {
        name: TOOL_NAME.to_string(),
        version: "test".to_string(),
        commit: None,
    };

    let run = RunInfo {
        started_at: "2026-01-01T00:00:00Z".to_string(),
        ended_at: "2026-01-01T00:00:01Z".to_string(),
        duration_ms: None,
        host: None,
        git: None,
    };

    let diff_map = world.diff.as_ref().map(|d| parse_unified_diff(d).unwrap());
    let diagnostics = world.diagnostics.as_ref().map(|d| {
        parse_cargo_messages(Cursor::new(d.as_bytes())).unwrap()
    });

    let report = ingest_on_diff(IngestOnDiffParams {
        tool,
        run,
        host: None,
        git: None,
        diff_map,
        diagnostics,
        repo_root: Some(NormPath::new("/repo")),
        config: world.config.effective(),
        repro: None,
    });

    world.report = Some(report);
}

#[then(expr = "verdict status is {string}")]
async fn then_status(world: &mut LintdiffWorld, expected: String) {
    let r = world.report.as_ref().expect("report produced");
    let actual = match r.verdict.status {
        lintdiff_types::VerdictStatus::Pass => "pass",
        lintdiff_types::VerdictStatus::Warn => "warn",
        lintdiff_types::VerdictStatus::Fail => "fail",
        lintdiff_types::VerdictStatus::Skip => "skip",
    };
    assert_eq!(actual, expected);
}

#[then(expr = "warn count is {int}")]
async fn then_warn_count(world: &mut LintdiffWorld, n: i32) {
    let r = world.report.as_ref().expect("report produced");
    assert_eq!(r.verdict.counts.warn as i32, n);
}

#[then(expr = "error count is {int}")]
async fn then_error_count(world: &mut LintdiffWorld, n: i32) {
    let r = world.report.as_ref().expect("report produced");
    assert_eq!(r.verdict.counts.error as i32, n);
}

#[tokio::main]
async fn main() {
    LintdiffWorld::run("tests/features").await;
}
