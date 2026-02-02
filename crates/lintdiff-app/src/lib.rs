//! Adapters and orchestration glue.
//!
//! The job of this crate is to:
//! - load config
//! - acquire diff + diagnostics
//! - call the domain
//! - write artifacts
//! - map report verdict to exit codes

use std::ffi::OsStr;
use std::fs;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use lintdiff_diagnostics::parse_cargo_messages;
use lintdiff_diff::parse_unified_diff;
use lintdiff_domain::{ingest_on_diff, IngestOnDiffParams};
use lintdiff_render::{render_github_annotations, render_markdown, MarkdownOptions, DEFAULT_REPORT_PATH};
use lintdiff_types::{EffectiveConfig, GitInfo, HostInfo, LintdiffConfig, NormPath, Report, RunInfo, ToolInfo};
use serde_json::json;
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("failed to read file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write file {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse config toml: {source}")]
    ParseConfig {
        #[source]
        source: toml::de::Error,
    },
    #[error("failed to run git command: {msg}")]
    Git { msg: String },
    #[error("failed to parse diff: {msg}")]
    DiffParse { msg: String },
    #[error("failed to parse diagnostics: {msg}")]
    DiagnosticsParse { msg: String },
    #[error("failed to run command: {msg}")]
    RunCommand { msg: String },
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

    pub out_path: PathBuf,
    pub md_path: Option<PathBuf>,
    pub annotations: AnnotationFormat,

    pub tool: ToolInfo,
    /// Optional "how to reproduce" command string.
    pub repro: Option<String>,
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
    let repo_root = NormPath::new(root.to_string_lossy().to_string());

    let cfg = load_config(&root, opts.config_path.as_deref())?;
    let eff = cfg.effective();

    let diff_text = acquire_diff(&root, opts.diff_file.as_deref(), opts.base.as_deref(), opts.head.as_deref())?;
    let diff_map = parse_unified_diff(&diff_text).map_err(|e| AppError::DiffParse { msg: e.to_string() })?;

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
        // Emit to stdout for CI systems that pick it up.
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

    // Run the command and capture stdout (cargo json messages).
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
    stdout.read_to_string(&mut buf).map_err(|e| AppError::RunCommand {
        msg: format!("failed reading command stdout: {e}"),
    })?;

    let status = child.wait().map_err(|e| AppError::RunCommand {
        msg: format!("failed waiting for command: {e}"),
    })?;

    // Use stdout as the diagnostics source for ingest.
    let diags = {
        let reader = BufReader::new(buf.as_bytes());
        parse_cargo_messages(reader).map_err(|e| AppError::DiagnosticsParse { msg: e.to_string() })?
    };

    // Write to a temporary file? Not required. We pass directly to domain by overriding acquire_diagnostics path.
    // We'll inject the diagnostics into ingest params by calling run_ingest-like flow directly.

    let started = now_rfc3339();
    let root = determine_repo_root(opts.root.as_deref())?;
    let repo_root = NormPath::new(root.to_string_lossy().to_string());
    let cfg = load_config(&root, opts.config_path.as_deref())?;
    let eff = cfg.effective();

    let diff_text = acquire_diff(&root, opts.diff_file.as_deref(), opts.base.as_deref(), opts.head.as_deref())?;
    let diff_map = parse_unified_diff(&diff_text).map_err(|e| AppError::DiffParse { msg: e.to_string() })?;
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
        diagnostics: Some(diags),
        repo_root: Some(repo_root),
        config: eff.clone(),
        repro: opts.repro.clone().or_else(|| Some(command.join(" "))),
    });

    // include upstream command status in report.data
    let mut report = report;
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
    Ok(IngestOutcome { report, markdown, annotations, exit_code })
}

fn load_config(repo_root: &Path, explicit: Option<&Path>) -> Result<LintdiffConfig, AppError> {
    let path = if let Some(p) = explicit {
        Some(p.to_path_buf())
    } else {
        let candidate = repo_root.join("lintdiff.toml");
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    };

    let Some(path) = path else {
        return Ok(LintdiffConfig::default());
    };

    let raw = fs::read_to_string(&path).map_err(|e| AppError::ReadFile { path: path.clone(), source: e })?;
    let cfg: LintdiffConfig = toml::from_str(&raw).map_err(|e| AppError::ParseConfig { source: e })?;
    Ok(cfg)
}

fn acquire_diagnostics(path: Option<&Path>) -> Result<Option<Vec<lintdiff_diagnostics::Diagnostic>>, AppError> {
    if let Some(p) = path {
        let f = fs::File::open(p).map_err(|e| AppError::ReadFile { path: p.to_path_buf(), source: e })?;
        let reader = BufReader::new(f);
        let diags = parse_cargo_messages(reader).map_err(|e| AppError::DiagnosticsParse { msg: e.to_string() })?;
        return Ok(Some(diags));
    }

    // stdin
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).map_err(|e| AppError::DiagnosticsParse {
        msg: format!("failed reading stdin: {e}"),
    })?;

    if buf.trim().is_empty() {
        return Ok(None);
    }

    let reader = BufReader::new(buf.as_bytes());
    let diags = parse_cargo_messages(reader).map_err(|e| AppError::DiagnosticsParse { msg: e.to_string() })?;
    Ok(Some(diags))
}

fn acquire_diff(repo_root: &Path, diff_file: Option<&Path>, base: Option<&str>, head: Option<&str>) -> Result<String, AppError> {
    if let Some(p) = diff_file {
        return fs::read_to_string(p).map_err(|e| AppError::ReadFile { path: p.to_path_buf(), source: e });
    }

    let Some(base) = base else {
        return Err(AppError::Git { msg: "missing --base (or provide --diff-file)".to_string() });
    };
    let Some(head) = head else {
        return Err(AppError::Git { msg: "missing --head (or provide --diff-file)".to_string() });
    };

    let range = format!("{base}..{head}");
    let out = Command::new("git")
        .current_dir(repo_root)
        .args(["diff", "--unified=0", &range])
        .output()
        .map_err(|e| AppError::Git { msg: format!("failed to run git diff: {e}") })?;

    if !out.status.success() {
        return Err(AppError::Git {
            msg: format!("git diff failed: {}", String::from_utf8_lossy(&out.stderr)),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn determine_repo_root(explicit: Option<&Path>) -> Result<PathBuf, AppError> {
    if let Some(p) = explicit {
        return Ok(p.to_path_buf());
    }

    // best effort: git toplevel
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();

    if let Ok(out) = out {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return Ok(PathBuf::from(s));
            }
        }
    }

    // fallback: cwd
    std::env::current_dir().map_err(|e| AppError::Git {
        msg: format!("failed to determine repo root: {e}"),
    })
}

fn gather_git_info(repo_root: &Path, base: Option<&str>, head: Option<&str>) -> Result<GitInfo, AppError> {
    let repo = git_config_get(repo_root, ["config", "--get", "remote.origin.url"]).ok();
    let merge_base = match (base, head) {
        (Some(b), Some(h)) => git_merge_base(repo_root, b, h).ok(),
        _ => None,
    };

    Ok(GitInfo {
        repo,
        base_ref: None,
        head_ref: None,
        base_sha: base.map(|s| s.to_string()),
        head_sha: head.map(|s| s.to_string()),
        merge_base,
    })
}

fn git_merge_base(repo_root: &Path, base: &str, head: &str) -> Result<String, AppError> {
    let out = Command::new("git")
        .current_dir(repo_root)
        .args(["merge-base", base, head])
        .output()
        .map_err(|e| AppError::Git { msg: format!("failed to run git merge-base: {e}") })?;

    if !out.status.success() {
        return Err(AppError::Git {
            msg: format!("git merge-base failed: {}", String::from_utf8_lossy(&out.stderr)),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_config_get<I, S>(repo_root: &Path, args: I) -> Result<String, AppError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let out = Command::new("git")
        .current_dir(repo_root)
        .args(args)
        .output()
        .map_err(|e| AppError::Git { msg: format!("failed to run git config: {e}") })?;

    if !out.status.success() {
        return Err(AppError::Git {
            msg: format!("git config failed: {}", String::from_utf8_lossy(&out.stderr)),
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn write_report_json(report: &Report, path: &Path) -> Result<(), AppError> {
    let bytes = serde_json::to_vec_pretty(report).expect("report must serialize");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::WriteFile { path: parent.to_path_buf(), source: e })?;
    }
    fs::write(path, bytes).map_err(|e| AppError::WriteFile { path: path.to_path_buf(), source: e })
}

fn write_text(path: &Path, contents: &str) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::WriteFile { path: parent.to_path_buf(), source: e })?;
    }
    fs::write(path, contents).map_err(|e| AppError::WriteFile { path: path.to_path_buf(), source: e })
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn classify_exit_code(report: &Report) -> i32 {
    // 0 - ok (pass/warn/skip)
    // 2 - policy failure
    // 1 - tool/runtime error
    match report.verdict.status {
        lintdiff_types::VerdictStatus::Fail => {
            if report
                .findings
                .iter()
                .any(|f| f.code.starts_with("lintdiff.input.") || f.check_id.as_deref() == Some("lintdiff.runtime"))
            {
                1
            } else {
                2
            }
        }
        _ => 0,
    }
}
