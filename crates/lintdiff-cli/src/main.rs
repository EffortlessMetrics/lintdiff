use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use lintdiff_app::{run_and_ingest, run_ingest, AnnotationFormat, IngestOptions};
use lintdiff_render::{render_github_annotations, render_markdown, MarkdownOptions};
use lintdiff_types::{Report, ToolInfo};

#[derive(Parser, Debug)]
#[command(name = "lintdiff")]
#[command(version)]
#[command(about = "Diff-scoped filter for Rust diagnostics (rustc/Clippy), emitting a cockpit receipt.")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Ingest an existing diagnostics stream + diff and emit a receipt.
    Ingest {
        /// Path to diagnostics jsonl (cargo --message-format=json). If omitted, read stdin.
        #[arg(long)]
        diagnostics: Option<PathBuf>,

        /// Diff patch file to use instead of git diff.
        #[arg(long)]
        diff_file: Option<PathBuf>,

        /// Base ref/sha for git diff (requires --head).
        #[arg(long)]
        base: Option<String>,

        /// Head ref/sha for git diff (requires --base).
        #[arg(long)]
        head: Option<String>,

        /// Repo root (defaults to git toplevel if available, else cwd).
        #[arg(long)]
        root: Option<PathBuf>,

        /// lintdiff.toml path (defaults to <root>/lintdiff.toml if present).
        #[arg(long)]
        config: Option<PathBuf>,

        /// Where to write report.json.
        #[arg(long, default_value = "artifacts/lintdiff/report.json")]
        out: PathBuf,

        /// Where to write a markdown comment section.
        #[arg(long)]
        md: Option<PathBuf>,

        /// Emit CI annotations.
        #[arg(long, value_enum, default_value_t = AnnotationsArg::None)]
        annotations: AnnotationsArg,
    },

    /// Run a command (usually cargo clippy) and ingest its JSON output.
    Run {
        /// Diff patch file to use instead of git diff.
        #[arg(long)]
        diff_file: Option<PathBuf>,

        /// Base ref/sha for git diff (requires --head).
        #[arg(long)]
        base: Option<String>,

        /// Head ref/sha for git diff (requires --base).
        #[arg(long)]
        head: Option<String>,

        /// Repo root (defaults to git toplevel if available, else cwd).
        #[arg(long)]
        root: Option<PathBuf>,

        /// lintdiff.toml path (defaults to <root>/lintdiff.toml if present).
        #[arg(long)]
        config: Option<PathBuf>,

        /// Where to write report.json.
        #[arg(long, default_value = "artifacts/lintdiff/report.json")]
        out: PathBuf,

        /// Where to write a markdown comment section.
        #[arg(long)]
        md: Option<PathBuf>,

        /// Emit CI annotations.
        #[arg(long, value_enum, default_value_t = AnnotationsArg::None)]
        annotations: AnnotationsArg,

        /// Command to run (use `--` before the command).
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },

    /// Render markdown from an existing report.json.
    Md {
        #[arg(long, default_value = "artifacts/lintdiff/report.json")]
        report: PathBuf,
        #[arg(long, default_value_t = 20)]
        max_items: usize,
    },

    /// Render GitHub annotations from an existing report.json.
    Annotations {
        #[arg(long, default_value = "artifacts/lintdiff/report.json")]
        report: PathBuf,
        #[arg(long, default_value_t = 50)]
        max: usize,
    },

    /// Explain a lintdiff-owned code or check id.
    Explain {
        code_or_check: String,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum AnnotationsArg {
    Github,
    None,
}

impl From<AnnotationsArg> for AnnotationFormat {
    fn from(v: AnnotationsArg) -> Self {
        match v {
            AnnotationsArg::Github => AnnotationFormat::Github,
            AnnotationsArg::None => AnnotationFormat::None,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::Ingest {
            diagnostics,
            diff_file,
            base,
            head,
            root,
            config,
            out,
            md,
            annotations,
        } => {
            let tool = ToolInfo {
                name: "lintdiff".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                commit: option_env!("GIT_SHA").map(|s| s.to_string()),
            };

            let repro = repro_string_ingest(&diagnostics, &diff_file, &base, &head);

            let res = run_ingest(IngestOptions {
                diagnostics_path: diagnostics,
                diff_file,
                base,
                head,
                root,
                config_path: config,
                out_path: out,
                md_path: md,
                annotations: annotations.into(),
                tool,
                repro: Some(repro),
            });

            match res {
                Ok(outcome) => ExitCode::from(outcome.exit_code as u8),
                Err(e) => {
                    eprintln!("lintdiff error: {e}");
                    ExitCode::from(1)
                }
            }
        }

        Commands::Run {
            diff_file,
            base,
            head,
            root,
            config,
            out,
            md,
            annotations,
            command,
        } => {
            let tool = ToolInfo {
                name: "lintdiff".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                commit: option_env!("GIT_SHA").map(|s| s.to_string()),
            };

            let repro = Some(format!("lintdiff run -- {}", command.join(" ")));

            let res = run_and_ingest(
                IngestOptions {
                    diagnostics_path: None,
                    diff_file,
                    base,
                    head,
                    root,
                    config_path: config,
                    out_path: out,
                    md_path: md,
                    annotations: annotations.into(),
                    tool,
                    repro,
                },
                command,
            );

            match res {
                Ok(outcome) => ExitCode::from(outcome.exit_code as u8),
                Err(e) => {
                    eprintln!("lintdiff error: {e}");
                    ExitCode::from(1)
                }
            }
        }

        Commands::Md { report, max_items } => {
            let report = load_report(&report);
            match report {
                Ok(r) => {
                    let md = render_markdown(
                        &r,
                        MarkdownOptions {
                            max_items,
                            report_path: report_path_string(&report),
                        },
                    );
                    print!("{md}");
                    ExitCode::from(0)
                }
                Err(e) => {
                    eprintln!("lintdiff error: {e}");
                    ExitCode::from(1)
                }
            }
        }

        Commands::Annotations { report, max } => {
            let report = load_report(&report);
            match report {
                Ok(r) => {
                    let out = render_github_annotations(&r, max);
                    print!("{out}");
                    ExitCode::from(0)
                }
                Err(e) => {
                    eprintln!("lintdiff error: {e}");
                    ExitCode::from(1)
                }
            }
        }

        Commands::Explain { code_or_check } => {
            print!("{}", explain(&code_or_check));
            ExitCode::from(0)
        }
    }
}

fn load_report(path: &PathBuf) -> Result<Report, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("failed to read report: {e}"))?;
    serde_json::from_str::<Report>(&raw).map_err(|e| format!("invalid report json: {e}"))
}

fn report_path_string(p: &PathBuf) -> String {
    p.to_string_lossy().to_string()
}

fn repro_string_ingest(
    diagnostics: &Option<PathBuf>,
    diff_file: &Option<PathBuf>,
    base: &Option<String>,
    head: &Option<String>,
) -> String {
    let mut parts: Vec<String> = vec!["lintdiff ingest".to_string()];

    if let Some(p) = diagnostics {
        parts.push(format!("--diagnostics {}", p.to_string_lossy()));
    } else {
        parts.push("< diagnostics.jsonl".to_string());
    }

    if let Some(p) = diff_file {
        parts.push(format!("--diff-file {}", p.to_string_lossy()));
    } else if base.is_some() && head.is_some() {
        parts.push(format!("--base {} --head {}", base.as_ref().unwrap(), head.as_ref().unwrap()));
    } else {
        parts.push("--base <base> --head <head>".to_string());
    }

    parts.join(" ")
}

fn explain(key: &str) -> String {
    match key {
        "diagnostics.on_diff" => {
            "diagnostics.on_diff\n\nMatches rustc/Clippy diagnostics whose primary spans intersect changed lines in the PR diff.\n\n".to_string()
        }
        "lintdiff.input.missing_diff" => "lintdiff.input.missing_diff\n\nDiff input is required. Provide --base and --head (git diff) or --diff-file.\n".to_string(),
        "lintdiff.input.missing_diagnostics" => "lintdiff.input.missing_diagnostics\n\nDiagnostics input is required to evaluate. Provide --diagnostics or pipe cargo JSON to stdin.\n".to_string(),
        "lintdiff.matching.no_matches" => "lintdiff.matching.no_matches\n\nNo diagnostics matched changed lines. This usually means path normalization mismatch (absolute paths vs repo-relative) or an unexpected diff range.\n".to_string(),
        _ => format!("{key}\n\nNo local explanation available.\n"),
    }
}
