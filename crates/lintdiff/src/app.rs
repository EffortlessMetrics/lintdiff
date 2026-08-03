use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

pub use crate::git::AppGitError;
pub use crate::io::AppIoError;

use crate::git::{acquire_diff, determine_repo_root, gather_git_info};
use crate::io::{
    acquire_diagnostics_with_status, load_config, now_rfc3339, write_report_json, write_text,
};
use lintdiff_engine::{ingest_on_diff, IngestOnDiffParams};
use lintdiff_engine::{parse_cargo_messages_with_status, parse_unified_diff, Diagnostic};
use lintdiff_render::{
    render_github_annotations, render_markdown, MarkdownOptions, DEFAULT_REPORT_PATH,
};
use lintdiff_types::{FailOn, HostInfo, LintdiffConfig, NormPath, Report, RunInfo, ToolInfo};

use crate::config::feature_flags::set_feature_flags_from_assignments;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("failed to run command: {msg}")]
    RunCommand { msg: String },
    #[error("failed to parse diff: {msg}")]
    DiffParse { msg: String },
    #[error("invalid feature flag assignment: {msg}")]
    FeatureFlag { msg: String },
    #[error("CI environment detection failed: {msg}")]
    CiDetection { msg: String },
    #[error("config error: {msg}")]
    Config { msg: String },
    #[error("I/O failure: {0}")]
    Io(#[from] AppIoError),
    #[error("git failure: {0}")]
    Git(#[from] AppGitError),
}

#[derive(Clone, Debug)]
pub enum AnnotationFormat {
    Github,
    None,
}

#[derive(Clone, Debug)]
pub struct IngestOptions {
    pub diagnostics_path: Option<PathBuf>,
    pub diff_file: Option<PathBuf>,
    pub base: Option<String>,
    pub head: Option<String>,
    pub root: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub feature_flags: Vec<String>,

    pub out_path: PathBuf,
    pub md_path: Option<PathBuf>,
    pub annotations: AnnotationFormat,

    pub tool: ToolInfo,
    /// Optional "how to reproduce" command string.
    pub repro: Option<String>,
    /// Override fail_on policy (from CLI --fail-on).
    pub fail_on_override: Option<String>,
    /// Optional upstream status supplied by a file or CI workflow.
    pub upstream: Option<UpstreamInput>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpstreamInput {
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub build_finished: Option<bool>,
    pub build_success: Option<bool>,
}

pub struct IngestOutcome {
    pub report: Report,
    pub markdown: Option<String>,
    pub annotations: Option<String>,
    pub exit_code: i32,
}

pub fn run_ingest(opts: IngestOptions) -> Result<IngestOutcome, AppError> {
    let started = now_rfc3339();
    let timer = Instant::now();
    let mut opts = opts;
    let stream = acquire_diagnostics_with_status(opts.diagnostics_path.as_deref())?;
    let parsed_upstream = stream.as_ref().and_then(|stream| {
        (stream.build_finished || opts.upstream.is_some()).then(|| UpstreamInput {
            command: Vec::new(),
            exit_code: None,
            build_finished: Some(stream.build_finished),
            build_success: stream.build_success,
        })
    });
    let upstream = merge_upstream(parsed_upstream, opts.upstream.take());
    let diagnostics = stream.map(|stream| stream.diagnostics);

    ingest_with_diagnostics(opts, diagnostics, upstream, started, timer)
}

fn ingest_with_diagnostics(
    opts: IngestOptions,
    diagnostics: Option<Vec<Diagnostic>>,
    upstream: Option<UpstreamInput>,
    started: String,
    timer: Instant,
) -> Result<IngestOutcome, AppError> {
    let root = determine_repo_root(opts.root.as_deref())?;
    let repo_root = NormPath::from_repo_path(root.to_string_lossy());

    let mut cfg = load_config(&root, opts.config_path.as_deref())?;
    apply_feature_flag_overrides(&mut cfg, &opts.feature_flags)?;
    if let Some(ref fo) = opts.fail_on_override {
        apply_fail_on_override(&mut cfg, fo)?;
    }
    let eff = cfg.effective();

    let diff_text = acquire_diff(
        &root,
        opts.diff_file.as_deref(),
        opts.base.as_deref(),
        opts.head.as_deref(),
    )?;
    let diff_map =
        parse_unified_diff(&diff_text).map_err(|e| AppError::DiffParse { msg: e.to_string() })?;

    let ended = now_rfc3339();

    let run = RunInfo {
        started_at: started,
        ended_at: ended,
        duration_ms: Some(timer.elapsed().as_millis() as u64),
        host: None,
        git: None,
    };

    let host = Some(HostInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    });

    let git = gather_git_info(&root, opts.base.as_deref(), opts.head.as_deref()).ok();

    let mut report = ingest_on_diff(IngestOnDiffParams {
        tool: opts.tool.clone(),
        run,
        host,
        git,
        diff_map: Some(diff_map),
        diagnostics,
        repo_root: Some(repo_root),
        config: eff.clone(),
        repro: opts.repro.clone(),
    });

    add_upstream_evidence(&mut report, upstream);

    write_report_json(&report, &opts.out_path)?;

    let markdown = opts.md_path.as_ref().map(|p| {
        let md = render_markdown(
            &report,
            MarkdownOptions {
                max_items: 20,
                report_path: DEFAULT_REPORT_PATH.to_string(),
            },
        );
        let _ = write_text(p, &md);
        md
    });

    let annotations = match opts.annotations {
        AnnotationFormat::Github => Some(render_github_annotations(&report, eff.max_annotations)),
        AnnotationFormat::None => None,
    };

    if let Some(ann) = &annotations {
        print!("{ann}");
    }

    let exit_code = classify_exit_code(&report);
    Ok(IngestOutcome {
        report,
        markdown,
        annotations,
        exit_code,
    })
}

pub fn run_and_ingest(
    mut opts: IngestOptions,
    command: Vec<String>,
) -> Result<IngestOutcome, AppError> {
    if command.is_empty() {
        return Err(AppError::RunCommand {
            msg: "no command provided (use -- <command...>)".to_string(),
        });
    }

    let started = now_rfc3339();
    let timer = Instant::now();
    let mut cmd = Command::new(&command[0]);
    if command.len() > 1 {
        cmd.args(&command[1..]);
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());

    let mut child = cmd.spawn().map_err(|e| AppError::RunCommand {
        msg: format!("failed to spawn command: {e}"),
    })?;

    let mut stdout = child.stdout.take().ok_or_else(|| AppError::RunCommand {
        msg: "failed to capture stdout".to_string(),
    })?;

    let mut buf = String::new();
    stdout
        .read_to_string(&mut buf)
        .map_err(|e| AppError::RunCommand {
            msg: format!("failed reading command stdout: {e}"),
        })?;

    let status = child.wait().map_err(|e| AppError::RunCommand {
        msg: format!("failed waiting for command: {e}"),
    })?;

    let stream = parse_cargo_messages_with_status(std::io::BufReader::new(buf.as_bytes()))
        .map_err(|e| AppError::Io(AppIoError::DiagnosticsParse { msg: e.to_string() }))?;
    opts.upstream = Some(UpstreamInput {
        command: command.clone(),
        exit_code: status.code(),
        build_finished: Some(stream.build_finished),
        build_success: stream.build_success,
    });

    ingest_with_diagnostics(opts, Some(stream.diagnostics), None, started, timer)
}

/// Run lintdiff in GitHub Actions mode, auto-detecting base/head from environment.
///
/// Reads `GITHUB_BASE_REF`, `GITHUB_SHA`, `GITHUB_HEAD_REF`, `GITHUB_WORKSPACE`,
/// and `GITHUB_EVENT_NAME` to determine diff parameters automatically.
#[allow(clippy::too_many_arguments)]
pub fn run_ci_github(
    tool: ToolInfo,
    base_override: Option<String>,
    head_override: Option<String>,
    root_override: Option<PathBuf>,
    config_path: Option<PathBuf>,
    fail_on_override: Option<String>,
    diagnostics_path: Option<PathBuf>,
    feature_flags: Vec<String>,
    out_path: PathBuf,
    md_path: Option<PathBuf>,
    annotations: AnnotationFormat,
    upstream: Option<UpstreamInput>,
) -> Result<IngestOutcome, AppError> {
    let base = base_override.or_else(|| std::env::var("GITHUB_BASE_REF").ok());
    let head = head_override
        .or_else(|| std::env::var("GITHUB_SHA").ok())
        .or_else(|| std::env::var("GITHUB_HEAD_REF").ok());

    if base.is_none() || head.is_none() {
        return Err(AppError::CiDetection {
            msg: "Could not detect CI environment. Ensure GITHUB_BASE_REF and GITHUB_SHA \
                  are set (run inside GitHub Actions), or provide --base and --head explicitly."
                .to_string(),
        });
    }

    let root = root_override.or_else(|| std::env::var("GITHUB_WORKSPACE").ok().map(PathBuf::from));

    let repro = format!(
        "lintdiff ci github --base {} --head {}",
        base.as_deref().unwrap_or("?"),
        head.as_deref().unwrap_or("?"),
    );

    run_ingest(IngestOptions {
        diagnostics_path,
        diff_file: None,
        base,
        head,
        root,
        config_path,
        feature_flags,
        out_path,
        md_path,
        annotations,
        tool,
        repro: Some(repro),
        fail_on_override,
        upstream,
    })
}

fn apply_feature_flag_overrides(
    config: &mut LintdiffConfig,
    assignments: &[String],
) -> Result<(), AppError> {
    set_feature_flags_from_assignments(&mut config.feature_flags, assignments.iter())
        .map_err(|msg| AppError::FeatureFlag { msg })
}

fn merge_upstream(
    parsed: Option<UpstreamInput>,
    overrides: Option<UpstreamInput>,
) -> Option<UpstreamInput> {
    match (parsed, overrides) {
        (None, None) => None,
        (Some(parsed), None) => Some(parsed),
        (None, Some(overrides)) => Some(overrides),
        (Some(mut parsed), Some(overrides)) => {
            if !overrides.command.is_empty() {
                parsed.command = overrides.command;
            }
            if overrides.exit_code.is_some() {
                parsed.exit_code = overrides.exit_code;
            }
            if overrides.build_finished.is_some() {
                parsed.build_finished = overrides.build_finished;
            }
            if overrides.build_success.is_some() {
                parsed.build_success = overrides.build_success;
            }
            Some(parsed)
        }
    }
}

fn add_upstream_evidence(report: &mut Report, upstream: Option<UpstreamInput>) {
    let Some(upstream) = upstream else {
        return;
    };

    let build_finished = upstream.build_finished.unwrap_or(false);
    let complete = build_finished && upstream.build_success.is_some();
    let mut data = report.data.take().unwrap_or_else(|| json!({}));
    if let Some(obj) = data.as_object_mut() {
        obj.insert(
            "upstream".to_string(),
            json!({
                "command": upstream.command,
                "exit_code": upstream.exit_code,
                "build_finished": build_finished,
                "build_success": upstream.build_success,
                "complete": complete,
            }),
        );
        if let Some(exit_code) = upstream.exit_code {
            obj.insert("upstream_exit_ok".to_string(), json!(exit_code == 0));
        }
    }
    report.data = Some(data);
}

/// Apply fail_on override from CLI to config.
fn apply_fail_on_override(
    config: &mut LintdiffConfig,
    override_value: &str,
) -> Result<(), AppError> {
    config.fail_on = Some(
        override_value
            .parse::<FailOn>()
            .map_err(|e| AppError::Config { msg: e })?,
    );
    Ok(())
}

fn classify_exit_code(report: &Report) -> i32 {
    // 0 - ok (pass/warn/skip)
    // 2 - policy failure
    // 1 - tool/runtime error
    match report.verdict.status {
        lintdiff_types::VerdictStatus::Fail => {
            if report.findings.iter().any(|f| {
                f.code.starts_with("lintdiff.input.")
                    || f.check_id.as_deref() == Some("lintdiff.runtime")
            }) {
                1
            } else {
                2
            }
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn empty_report() -> Result<Report, serde_json::Error> {
        serde_json::from_value(json!({
            "schema": "lintdiff.report.v1",
            "tool": {"name": "lintdiff", "version": "test"},
            "run": {"started_at": "2026-01-01T00:00:00Z", "ended_at": "2026-01-01T00:00:01Z"},
            "verdict": {"status": "pass", "counts": {"info": 0, "warn": 0, "error": 0}}
        }))
    }

    #[test]
    fn upstream_evidence_preserves_exact_failure_and_completion_fields(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut report = empty_report()?;

        add_upstream_evidence(
            &mut report,
            Some(UpstreamInput {
                command: vec!["cargo".to_string(), "clippy".to_string()],
                exit_code: Some(101),
                build_finished: Some(true),
                build_success: Some(false),
            }),
        );

        let upstream = report
            .data
            .as_ref()
            .and_then(|data| data.get("upstream"))
            .ok_or_else(|| std::io::Error::other("upstream evidence should be present"))?;
        assert_eq!(upstream["command"], json!(["cargo", "clippy"]));
        assert_eq!(upstream["exit_code"], json!(101));
        assert_eq!(upstream["build_finished"], json!(true));
        assert_eq!(upstream["build_success"], json!(false));
        assert_eq!(upstream["complete"], json!(true));
        let data = report
            .data
            .as_ref()
            .ok_or_else(|| std::io::Error::other("report data should be present"))?;
        assert_eq!(data["upstream_exit_ok"], json!(false));
        Ok(())
    }

    #[test]
    fn incomplete_upstream_evidence_is_not_complete() -> Result<(), Box<dyn std::error::Error>> {
        let mut report = empty_report()?;

        add_upstream_evidence(
            &mut report,
            Some(UpstreamInput {
                command: vec!["cargo".to_string(), "clippy".to_string()],
                exit_code: Some(1),
                build_finished: Some(false),
                build_success: None,
            }),
        );

        let data = report
            .data
            .as_ref()
            .ok_or_else(|| std::io::Error::other("report data should be present"))?;
        assert_eq!(data["upstream"]["complete"], json!(false));
        assert_eq!(data["upstream"]["build_success"], json!(null));
        Ok(())
    }

    #[test]
    fn explicit_upstream_values_override_parsed_stream_values(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let merged = merge_upstream(
            Some(UpstreamInput {
                command: Vec::new(),
                exit_code: Some(0),
                build_finished: Some(true),
                build_success: Some(true),
            }),
            Some(UpstreamInput {
                command: vec!["cargo".to_string(), "clippy".to_string()],
                exit_code: Some(101),
                build_finished: Some(false),
                build_success: None,
            }),
        )
        .ok_or_else(|| std::io::Error::other("merged upstream evidence should be present"))?;

        assert_eq!(merged.command, vec!["cargo", "clippy"]);
        assert_eq!(merged.exit_code, Some(101));
        assert_eq!(merged.build_finished, Some(false));
        assert_eq!(merged.build_success, Some(true));
        Ok(())
    }
}
