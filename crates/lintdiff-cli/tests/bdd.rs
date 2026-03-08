use cucumber::{given, then, when, World as _};

use lintdiff_bdd::{
    apply_feature_flag_value, read_fixture as fixture, run_ingest_from_fixtures, verdict_status,
};
use lintdiff_match::{compile_filters, path_allowed};
use lintdiff_render::{render_github_annotations, render_markdown, MarkdownOptions};
use lintdiff_types::{LintdiffConfig, Report};

#[derive(Debug, Default, cucumber::World)]
struct LintdiffWorld {
    diff: Option<String>,
    diagnostics: Option<String>,
    config: LintdiffConfig,
    report: Option<Report>,
    /// Rendered markdown output
    markdown: Option<String>,
    /// Rendered GitHub annotations output
    annotations: Option<String>,
    /// Path being tested for filter behavior
    test_path: Option<String>,
    /// Result of path filter check
    path_allowed: Option<bool>,
}

#[given(expr = "a diff fixture {string}")]
async fn given_diff(world: &mut LintdiffWorld, name: String) {
    world.diff = Some(fixture(&name));
}

#[given(expr = "a diagnostics fixture {string}")]
async fn given_diagnostics(world: &mut LintdiffWorld, name: String) {
    world.diagnostics = Some(fixture(&name));
}

#[given(expr = "deny code {string}")]
async fn deny_code(world: &mut LintdiffWorld, code: String) {
    world.config.filter.deny_codes.push(code);
}

#[given(expr = "suppress code {string}")]
async fn suppress_code(world: &mut LintdiffWorld, code: String) {
    world.config.filter.suppress_codes.push(code);
}

#[given(expr = "filter exclude path {string}")]
async fn given_filter_exclude(world: &mut LintdiffWorld, pattern: String) {
    world.config.filter.exclude_paths.push(pattern);
}

#[given(expr = "filter include path {string}")]
async fn given_filter_include(world: &mut LintdiffWorld, pattern: String) {
    world.config.filter.include_paths.push(pattern);
}

#[given(expr = "feature flag {string} is {string}")]
async fn given_feature_flag(world: &mut LintdiffWorld, flag: String, value: String) {
    if let Err(err) = apply_feature_flag_value(&mut world.config, &flag, &value) {
        panic!("unknown feature flag '{flag}': {err}");
    }
}

#[when("lintdiff ingests the inputs")]
async fn when_ingest(world: &mut LintdiffWorld) {
    world.report = Some(run_ingest_from_fixtures(
        &world.diff,
        &world.diagnostics,
        &world.config,
    ));
}

#[then(expr = "verdict status is {string}")]
async fn then_status(world: &mut LintdiffWorld, expected: String) {
    let r = world.report.as_ref().expect("report produced");
    let actual = verdict_status(r);
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

// =============================================================================
// Rendering step definitions (lintdiff-render)
// =============================================================================

#[when("lintdiff renders markdown output")]
async fn when_render_markdown(world: &mut LintdiffWorld) {
    let r = world.report.as_ref().expect("report produced");
    let opts = MarkdownOptions::default();
    world.markdown = Some(render_markdown(r, opts));
}

#[when(expr = "lintdiff renders markdown output with max items {int}")]
async fn when_render_markdown_with_max(world: &mut LintdiffWorld, max: i32) {
    let r = world.report.as_ref().expect("report produced");
    let opts = MarkdownOptions {
        max_items: max as usize,
        ..Default::default()
    };
    world.markdown = Some(render_markdown(r, opts));
}

#[when("lintdiff renders GitHub annotations")]
async fn when_render_github_annotations(world: &mut LintdiffWorld) {
    let r = world.report.as_ref().expect("report produced");
    world.annotations = Some(render_github_annotations(r, 100));
}

#[then(expr = "markdown output contains {string}")]
async fn then_markdown_contains(world: &mut LintdiffWorld, expected: String) {
    let md = world.markdown.as_ref().expect("markdown rendered");
    assert!(
        md.contains(&expected),
        "Expected markdown to contain {:?}, but got:\n{}",
        expected,
        md
    );
}

#[then(expr = "markdown output does not contain {string}")]
async fn then_markdown_not_contains(world: &mut LintdiffWorld, expected: String) {
    let md = world.markdown.as_ref().expect("markdown rendered");
    assert!(
        !md.contains(&expected),
        "Expected markdown NOT to contain {:?}, but it did:\n{}",
        expected,
        md
    );
}

#[then("markdown output contains status badge")]
async fn then_markdown_has_status(world: &mut LintdiffWorld) {
    let md = world.markdown.as_ref().expect("markdown rendered");
    assert!(
        md.contains("**Status:**"),
        "Expected markdown to contain status badge, but got:\n{}",
        md
    );
}

#[then("markdown output contains counts summary")]
async fn then_markdown_has_counts(world: &mut LintdiffWorld) {
    let md = world.markdown.as_ref().expect("markdown rendered");
    assert!(
        md.contains("**Counts:**"),
        "Expected markdown to contain counts summary, but got:\n{}",
        md
    );
}

#[then("markdown output contains findings table")]
async fn then_markdown_has_table(world: &mut LintdiffWorld) {
    let md = world.markdown.as_ref().expect("markdown rendered");
    assert!(
        md.contains("| Sev | Location | Code | Message |"),
        "Expected markdown to contain findings table header, but got:\n{}",
        md
    );
}

#[then(expr = "GitHub annotations output contains {string}")]
async fn then_annotations_contains(world: &mut LintdiffWorld, expected: String) {
    let ann = world.annotations.as_ref().expect("annotations rendered");
    assert!(
        ann.contains(&expected),
        "Expected GitHub annotations to contain {:?}, but got:\n{}",
        expected,
        ann
    );
}

#[then("GitHub annotations output is empty")]
async fn then_annotations_empty(world: &mut LintdiffWorld) {
    let ann = world.annotations.as_ref().expect("annotations rendered");
    assert!(
        ann.trim().is_empty(),
        "Expected empty GitHub annotations, but got:\n{}",
        ann
    );
}

#[then(expr = "GitHub annotations count is {int}")]
async fn then_annotations_count(world: &mut LintdiffWorld, expected: i32) {
    let ann = world.annotations.as_ref().expect("annotations rendered");
    let count = ann.lines().filter(|l| !l.trim().is_empty()).count();
    assert_eq!(
        count as i32, expected,
        "Expected {} GitHub annotation lines, but got {}:\n{}",
        expected, count, ann
    );
}

// =============================================================================
// Path matching step definitions (lintdiff-match)
// =============================================================================

#[given(expr = "a test path {string}")]
async fn given_test_path(world: &mut LintdiffWorld, path: String) {
    world.test_path = Some(path);
}

#[when("lintdiff checks path against filters")]
async fn when_check_path_filters(world: &mut LintdiffWorld) {
    let path = world.test_path.as_ref().expect("test path set");
    let effective = world.config.effective();
    let filters = compile_filters(&effective);
    world.path_allowed = Some(path_allowed(&filters, path));
}

#[then("path is allowed")]
async fn then_path_allowed(world: &mut LintdiffWorld) {
    let allowed = world.path_allowed.expect("path check performed");
    assert!(
        allowed,
        "Expected path to be allowed, but it was filtered out"
    );
}

#[then("path is filtered out")]
async fn then_path_filtered(world: &mut LintdiffWorld) {
    let allowed = world.path_allowed.expect("path check performed");
    assert!(
        !allowed,
        "Expected path to be filtered out, but it was allowed"
    );
}

// =============================================================================
// End-to-end workflow step definitions
// =============================================================================

#[when("lintdiff runs full pipeline")]
async fn when_full_pipeline(world: &mut LintdiffWorld) {
    // Run ingest
    world.report = Some(run_ingest_from_fixtures(
        &world.diff,
        &world.diagnostics,
        &world.config,
    ));
    // Render outputs
    let r = world.report.as_ref().expect("report produced");
    world.markdown = Some(render_markdown(r, MarkdownOptions::default()));
    world.annotations = Some(render_github_annotations(r, 100));
}

#[then(expr = "findings count is {int}")]
async fn then_findings_count(world: &mut LintdiffWorld, expected: i32) {
    let r = world.report.as_ref().expect("report produced");
    assert_eq!(
        r.findings.len() as i32,
        expected,
        "Expected {} findings, but got {}",
        expected,
        r.findings.len()
    );
}

#[then(expr = "finding {int} has code {string}")]
async fn then_finding_code(world: &mut LintdiffWorld, index: i32, code: String) {
    let r = world.report.as_ref().expect("report produced");
    let idx = index as usize;
    assert!(
        idx < r.findings.len(),
        "Finding index {} out of bounds ({} findings)",
        idx,
        r.findings.len()
    );
    assert_eq!(
        r.findings[idx].code, code,
        "Expected finding {} to have code {:?}, but got {:?}",
        idx, code, r.findings[idx].code
    );
}

#[then(expr = "finding {int} has severity {string}")]
async fn then_finding_severity(world: &mut LintdiffWorld, index: i32, severity: String) {
    let r = world.report.as_ref().expect("report produced");
    let idx = index as usize;
    assert!(
        idx < r.findings.len(),
        "Finding index {} out of bounds ({} findings)",
        idx,
        r.findings.len()
    );
    let actual = format!("{:?}", r.findings[idx].severity).to_lowercase();
    assert_eq!(
        actual, severity,
        "Expected finding {} to have severity {:?}, but got {:?}",
        idx, severity, actual
    );
}

#[then("explain total equals diagnostics total")]
async fn then_explain_total_equals_diagnostics(world: &mut LintdiffWorld) {
    let r = world.report.as_ref().expect("report produced");
    let data = r.data.as_ref().expect("report has data");

    let explain = data
        .get("explain")
        .expect("data has explain")
        .as_array()
        .expect("explain is array");
    let stats_total = data
        .get("stats")
        .and_then(|s| s.get("diagnostics_total"))
        .and_then(|v| v.as_u64())
        .expect("stats has diagnostics_total");

    assert_eq!(
        explain.len() as u64,
        stats_total,
        "explain entries ({}) should equal diagnostics_total ({})",
        explain.len(),
        stats_total
    );
}

#[then(expr = "explain has {int} entries with disposition {string}")]
async fn then_explain_disposition_count(
    world: &mut LintdiffWorld,
    expected: i32,
    disposition: String,
) {
    let r = world.report.as_ref().expect("report produced");
    let data = r.data.as_ref().expect("report has data");

    let explain = data
        .get("explain")
        .expect("data has explain")
        .as_array()
        .expect("explain is array");

    let count = explain
        .iter()
        .filter(|e| {
            e.get("disposition")
                .and_then(|d| d.as_str())
                .is_some_and(|d| d == disposition)
        })
        .count();

    assert_eq!(
        count as i32, expected,
        "Expected {} explain entries with disposition {:?}, but got {}",
        expected, disposition, count
    );
}

#[then(expr = "finding {int} and {int} share fingerprint")]
async fn then_findings_share_fingerprint(world: &mut LintdiffWorld, left: i32, right: i32) {
    let r = world.report.as_ref().expect("report produced");
    let left = left as usize;
    let right = right as usize;

    assert!(
        left < r.findings.len(),
        "Finding index {} out of bounds ({} findings)",
        left,
        r.findings.len()
    );
    assert!(
        right < r.findings.len(),
        "Finding index {} out of bounds ({} findings)",
        right,
        r.findings.len()
    );

    let lf = r.findings[left]
        .fingerprint
        .as_ref()
        .unwrap_or_else(|| panic!("missing fingerprint for finding {}", left));
    let rf = r.findings[right]
        .fingerprint
        .as_ref()
        .unwrap_or_else(|| panic!("missing fingerprint for finding {}", right));

    assert!(
        !lf.is_empty(),
        "fingerprint for finding {} should not be empty",
        left
    );
    assert_eq!(lf, rf, "expected findings to share the same fingerprint");
}

#[tokio::main]
async fn main() {
    LintdiffWorld::run("tests/features").await;
}
