use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use lintdiff_app_git::{acquire_diff, determine_repo_root, gather_git_info, AppGitError};
use lintdiff_app_io::{
    acquire_diagnostics, load_config, now_rfc3339, parse_diagnostics, write_report_json,
    write_text, AppIoError,
};
use lintdiff_diff::parse_unified_diff;
use lintdiff_domain::{ingest_on_diff, IngestOnDiffParams};
use lintdiff_feature_flags::set_feature_flags_from_assignments;
use lintdiff_render::{
    render_github_annotations, render_markdown, MarkdownOptions, DEFAULT_REPORT_PATH,
};
use lintdiff_types::{FailOn, HostInfo, LintdiffConfig, NormPath, Report, RunInfo, ToolInfo};
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
}

pub struct IngestOutcome {
    pub report: Report,
    pub markdown: Option<String>,
    pub annotations: Option<String>,
    pub exit_code: i32,
}

pub fn run_ingest(opts: IngestOptions) -> Result<IngestOutcome, AppError> {
    let started = now_rfc3339();
    let root = determine_repo_root(opts.root.as_deref())?;
    let repo_root = NormPath::new(root.to_string_lossy());

    let mut cfg = load_config(&root, opts.config_path.as_deref())?;
    apply_feature_flag_overrides(&mut cfg, &opts.feature_flags)?;
    if let Some(ref fo) = opts.fail_on_override {
        cfg.fail_on = Some(
            fo.parse::<FailOn>()
                .map_err(|e| AppError::Config { msg: e })?,
        );
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

    let diagnostics = acquire_diagnostics(opts.diagnostics_path.as_deref())?;
    let ended = now_rfc3339();

    let run = RunInfo {
        started_at: started,
        ended_at: ended,
        duration_ms: None,
        host: None,
        git: None,
    };

    let host = Some(HostInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    });

    let git = gather_git_info(&root, opts.base.as_deref(), opts.head.as_deref()).ok();

    let report = ingest_on_diff(IngestOnDiffParams {
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
    opts: IngestOptions,
    command: Vec<String>,
) -> Result<IngestOutcome, AppError> {
    if command.is_empty() {
        return Err(AppError::RunCommand {
            msg: "no command provided (use -- <command...>)".to_string(),
        });
    }

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

    let diags = {
        let reader = BufReader::new(buf.as_bytes());
        parse_diagnostics(reader)?
    };

    let started = now_rfc3339();
    let root = determine_repo_root(opts.root.as_deref())?;
    let repo_root = NormPath::new(root.to_string_lossy());

    let mut cfg = load_config(&root, opts.config_path.as_deref())?;
    apply_feature_flag_overrides(&mut cfg, &opts.feature_flags)?;
    if let Some(ref fo) = opts.fail_on_override {
        cfg.fail_on = Some(
            fo.parse::<FailOn>()
                .map_err(|e| AppError::Config { msg: e })?,
        );
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
        duration_ms: None,
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
        diagnostics: Some(diags),
        repo_root: Some(repo_root),
        config: eff.clone(),
        repro: opts.repro.clone().or_else(|| Some(command.join(" "))),
    });

    let mut data = report.data.take().unwrap_or_else(|| json!({}));
    if let Some(obj) = data.as_object_mut() {
        obj.insert("upstream_exit_ok".to_string(), json!(status.success()));
    }
    report.data = Some(data);

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
    })
}

fn apply_feature_flag_overrides(
    config: &mut LintdiffConfig,
    assignments: &[String],
) -> Result<(), AppError> {
    set_feature_flags_from_assignments(&mut config.feature_flags, assignments.iter())
        .map_err(|msg| AppError::FeatureFlag { msg })
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
