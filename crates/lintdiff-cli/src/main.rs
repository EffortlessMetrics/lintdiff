use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use lintdiff_app::{run_and_ingest, run_ci_github, run_ingest, AnnotationFormat, IngestOptions};
use lintdiff_render::{render_github_annotations, render_markdown, MarkdownOptions};
use lintdiff_types::{Report, ToolInfo};

// ANSI color codes for terminal output
mod colors {
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const CYAN: &str = "\x1b[36m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RESET: &str = "\x1b[0m";
}

/// Check if we should use colorized output
fn use_colors() -> bool {
    io::stdout().is_terminal() && io::stderr().is_terminal()
}

/// Format an error message with context and remediation guidance
struct ErrorGuidance {
    /// The main error message
    error: String,
    /// Why this error matters
    context: Option<String>,
    /// How to fix the error
    suggestion: Option<String>,
    /// Example command or code
    example: Option<String>,
}

impl ErrorGuidance {
    fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            context: None,
            suggestion: None,
            example: None,
        }
    }

    fn context(mut self, ctx: impl Into<String>) -> Self {
        self.context = Some(ctx.into());
        self
    }

    fn suggestion(mut self, sug: impl Into<String>) -> Self {
        self.suggestion = Some(sug.into());
        self
    }

    fn example(mut self, ex: impl Into<String>) -> Self {
        self.example = Some(ex.into());
        self
    }

    /// Render the error with optional colorization
    fn render(&self, colorize: bool) -> String {
        let mut output = String::new();

        if colorize {
            output.push_str(colors::RED);
            output.push_str(colors::BOLD);
            output.push_str("Error: ");
            output.push_str(colors::RESET);
            output.push_str(&self.error);
            output.push('\n');
        } else {
            output.push_str("Error: ");
            output.push_str(&self.error);
            output.push('\n');
        }

        if let Some(ref ctx) = self.context {
            output.push('\n');
            if colorize {
                output.push_str(colors::YELLOW);
                output.push_str("Context: ");
                output.push_str(colors::RESET);
            } else {
                output.push_str("Context: ");
            }
            output.push_str(ctx);
            output.push('\n');
        }

        if let Some(ref sug) = self.suggestion {
            output.push('\n');
            if colorize {
                output.push_str(colors::CYAN);
                output.push_str("Suggestion: ");
                output.push_str(colors::RESET);
            } else {
                output.push_str("Suggestion: ");
            }
            output.push_str(sug);
            output.push('\n');
        }

        if let Some(ref ex) = self.example {
            output.push('\n');
            output.push_str("Example: ");
            output.push_str(ex);
            output.push('\n');
        }

        output
    }
}

/// Print an error with guidance to stderr
fn print_error(guidance: &ErrorGuidance) {
    eprintln!("{}", guidance.render(use_colors()));
}

/// Convert an AppError to ErrorGuidance with detailed remediation info
fn app_error_to_guidance(error: &lintdiff_app::AppError) -> ErrorGuidance {
    use lintdiff_app::AppError;

    match error {
        AppError::RunCommand { msg } => {
            if msg.contains("no command provided") {
                ErrorGuidance::new(msg)
                    .context("The 'run' subcommand requires a command to execute after '--'.")
                    .suggestion("Add the command you want to run after '--', typically 'cargo clippy' or 'cargo check'.")
                    .example("lintdiff run --base main --head HEAD -- cargo clippy --message-format=json")
            } else if msg.contains("failed to spawn command") {
                ErrorGuidance::new(msg)
                    .context("The specified command could not be started. This usually means the command doesn't exist or is not in PATH.")
                    .suggestion("Verify the command exists and is accessible. Check spelling and PATH configuration.")
                    .example("lintdiff run -- cargo clippy --message-format=json")
            } else {
                ErrorGuidance::new(msg)
                    .context("An error occurred while running the command.")
                    .suggestion("Check that the command is valid and all arguments are correct.")
            }
        }

        AppError::DiffParse { msg } => {
            ErrorGuidance::new(format!("Failed to parse diff: {}", msg))
                .context("The diff input could not be parsed as a valid unified diff format.")
                .suggestion("Ensure the diff is in unified diff format (git diff output). Check for corrupted or truncated diff files.")
                .example("git diff main..HEAD > changes.diff && lintdiff ingest --diff-file changes.diff")
        }

        AppError::FeatureFlag { msg } => {
            ErrorGuidance::new(format!("Invalid feature flag: {}", msg))
                .context("Feature flags must be specified as 'name=value' pairs.")
                .suggestion("Use the format 'flag_name=true' or 'flag_name=false'. See documentation for available flags.")
                .example("lintdiff ingest --feature-flags strict_mode=true --base main --head HEAD")
        }

        AppError::CiDetection { msg } => {
            ErrorGuidance::new(msg)
                .context("The 'ci github' subcommand requires GitHub Actions environment variables to be set.")
                .suggestion("Run this command inside a GitHub Actions workflow, or use 'lintdiff ingest' with explicit --base and --head arguments.")
                .example("lintdiff ci github  # (inside GitHub Actions)\nlintdiff ingest --base $GITHUB_BASE_REF --head $GITHUB_SHA  # (manual)")
        }

        AppError::Config { msg } => {
            if msg.contains("fail_on") {
                ErrorGuidance::new(format!("Invalid configuration: {}", msg))
                    .context("The 'fail_on' policy value is not recognized.")
                    .suggestion("Valid values are: 'never', 'any', 'error', 'warning'.")
                    .example("fail_on = \"error\"  # in lintdiff.toml")
            } else {
                ErrorGuidance::new(format!("Configuration error: {}", msg))
                    .context("The configuration file contains invalid settings.")
                    .suggestion("Check lintdiff.toml for syntax errors and invalid field values.")
                    .example("See lintdiff.toml.example for a valid configuration template.")
            }
        }

        AppError::Io(io_error) => io_error_to_guidance(io_error),

        AppError::Git(git_error) => git_error_to_guidance(git_error),
    }
}

/// Convert an AppIoError to ErrorGuidance
fn io_error_to_guidance(error: &lintdiff_app_io::AppIoError) -> ErrorGuidance {
    use lintdiff_app_io::AppIoError;

    match error {
        AppIoError::ReadFile { path, source } => {
            let path_str = path.to_string_lossy();
            if path_str == "<stdin>" {
                ErrorGuidance::new(format!("Failed to read from stdin: {}", source))
                    .context("lintdiff tried to read diagnostics from standard input but failed.")
                    .suggestion("Ensure stdin is properly piped and contains valid JSON lines output from cargo.")
                    .example("cargo clippy --message-format=json | lintdiff ingest --base main --head HEAD")
            } else if source.kind() == io::ErrorKind::NotFound {
                ErrorGuidance::new(format!("File not found: {}", path_str))
                    .context("The specified file does not exist.")
                    .suggestion("Check the file path is correct and the file exists. Use forward slashes or escaped backslashes on Windows.")
                    .example("lintdiff ingest --diagnostics ./diagnostics.jsonl --diff-file ./changes.diff")
            } else if source.kind() == io::ErrorKind::PermissionDenied {
                ErrorGuidance::new(format!("Permission denied: {}", path_str))
                    .context("You don't have permission to read this file.")
                    .suggestion("Check file permissions and ensure you have read access.")
            } else {
                ErrorGuidance::new(format!("Failed to read file '{}': {}", path_str, source))
                    .context("An I/O error occurred while reading the file.")
                    .suggestion("Check the file exists and is accessible.")
            }
        }

        AppIoError::WriteFile { path, source } => {
            if source.kind() == io::ErrorKind::PermissionDenied {
                ErrorGuidance::new(format!("Permission denied writing to: {}", path.to_string_lossy()))
                    .context("You don't have permission to write to this location.")
                    .suggestion("Check directory permissions or choose a different output path with --out.")
                    .example("lintdiff ingest --out ./output/report.json --base main --head HEAD")
            } else if source.kind() == io::ErrorKind::NotFound {
                ErrorGuidance::new(format!("Cannot create file: {}", path.to_string_lossy()))
                    .context("The parent directory does not exist and could not be created.")
                    .suggestion("Create the parent directory manually or use a different output path.")
            } else {
                ErrorGuidance::new(format!("Failed to write file '{}': {}", path.to_string_lossy(), source))
                    .context("An I/O error occurred while writing the output file.")
                    .suggestion("Ensure the output directory exists and is writable.")
            }
        }

        AppIoError::ParseConfig { source } => {
            ErrorGuidance::new(format!("Invalid config file: {}", source))
                .context("The lintdiff.toml configuration file contains syntax errors.")
                .suggestion("Check for TOML syntax errors: missing quotes, invalid indentation, or typos in field names.")
                .example("See lintdiff.toml.example for a valid configuration template.")
        }

        AppIoError::DiagnosticsParse { msg } => {
            ErrorGuidance::new(format!("Failed to parse diagnostics: {}", msg))
                .context("The diagnostics input is not valid cargo JSON output.")
                .suggestion("Ensure diagnostics come from 'cargo clippy --message-format=json' or 'cargo check --message-format=json'. Each line should be a valid JSON object.")
                .example("cargo clippy --message-format=json > diagnostics.jsonl\nlintdiff ingest --diagnostics diagnostics.jsonl --base main --head HEAD")
        }

        AppIoError::Serialize { source } => {
            ErrorGuidance::new(format!("Failed to serialize report: {}", source))
                .context("Internal error: the report could not be converted to JSON.")
                .suggestion("This is likely a bug. Please report it with the input that caused this error.")
        }
    }
}

/// Convert an AppGitError to ErrorGuidance
fn git_error_to_guidance(error: &lintdiff_app_git::AppGitError) -> ErrorGuidance {
    use lintdiff_app_git::AppGitError;

    match error {
        AppGitError::Command { msg } => {
            if msg.contains("missing --base") || msg.contains("missing --head") {
                ErrorGuidance::new(msg)
                    .context("A diff source is required to determine which lines changed.")
                    .suggestion("Provide either --base and --head for git diff, or --diff-file for a pre-generated diff.")
                    .example("# Using git refs:\nlintdiff ingest --base main --head HEAD\n\n# Using a diff file:\nlintdiff ingest --diff-file changes.diff")
            } else if msg.contains("failed to read diff file") {
                ErrorGuidance::new(msg)
                    .context("The specified diff file could not be read.")
                    .suggestion("Check the file exists and contains valid unified diff output.")
                    .example("git diff main..HEAD > changes.diff\nlintdiff ingest --diff-file changes.diff")
            } else if msg.contains("git diff failed") {
                ErrorGuidance::new(msg)
                    .context("The git diff command failed. This may happen if refs don't exist or the repo is in an unexpected state.")
                    .suggestion("Ensure both --base and --head refer to valid commits/branches. Run 'git fetch' to update remote refs.")
                    .example("git fetch origin && lintdiff ingest --base origin/main --head HEAD")
            } else if msg.contains("git merge-base failed") {
                ErrorGuidance::new(msg)
                    .context("Could not compute merge base between the specified refs.")
                    .suggestion("Ensure both refs exist and share a common ancestor. Fetch remote refs if needed.")
            } else {
                ErrorGuidance::new(msg)
                    .context("A git command failed.")
                    .suggestion("Ensure you're running lintdiff inside a git repository.")
            }
        }

        AppGitError::RepoRoot { msg } => {
            ErrorGuidance::new(format!("Failed to determine repository root: {}", msg))
                .context("lintdiff could not find the root of the git repository.")
                .suggestion(
                    "Run lintdiff from within a git repository, or specify --root explicitly.",
                )
                .example("lintdiff ingest --root /path/to/repo --base main --head HEAD")
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "lintdiff")]
#[command(version)]
#[command(
    about = "Diff-scoped filter for Rust diagnostics (rustc/Clippy), emitting a cockpit receipt."
)]
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

        /// Override feature flags (name=value). Repeat for multiple flags.
        #[arg(long, value_name = "FLAG=VALUE")]
        feature_flags: Vec<String>,

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

        /// Override feature flags (name=value). Repeat for multiple flags.
        #[arg(long, value_name = "FLAG=VALUE")]
        feature_flags: Vec<String>,

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
    Explain { code_or_check: String },

    /// CI-aware subcommands that auto-detect environment variables.
    Ci {
        #[command(subcommand)]
        provider: CiProvider,
    },
}

#[derive(Subcommand, Debug)]
enum CiProvider {
    /// Run lintdiff in GitHub Actions, auto-detecting base/head from environment.
    Github {
        /// Override base ref (default: $GITHUB_BASE_REF).
        #[arg(long)]
        base: Option<String>,

        /// Override head ref (default: $GITHUB_SHA or $GITHUB_HEAD_REF).
        #[arg(long)]
        head: Option<String>,

        /// Repo root (defaults to $GITHUB_WORKSPACE or git toplevel).
        #[arg(long)]
        root: Option<PathBuf>,

        /// lintdiff.toml path.
        #[arg(long)]
        config: Option<PathBuf>,

        /// Override fail_on policy.
        #[arg(long)]
        fail_on: Option<String>,

        /// Path to diagnostics JSONL file (cargo clippy --message-format=json output).
        #[arg(long)]
        diagnostics: Option<PathBuf>,

        /// Override feature flags (name=value).
        #[arg(long, value_name = "FLAG=VALUE")]
        feature_flags: Vec<String>,

        /// Where to write report.json.
        #[arg(long, default_value = "artifacts/lintdiff/report.json")]
        out: PathBuf,

        /// Where to write a markdown comment section.
        #[arg(long)]
        md: Option<PathBuf>,

        /// Emit CI annotations.
        #[arg(long, value_enum, default_value_t = AnnotationsArg::Github)]
        annotations: AnnotationsArg,
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
            feature_flags,
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
                feature_flags,
                out_path: out,
                md_path: md,
                annotations: annotations.into(),
                tool,
                repro: Some(repro),
                fail_on_override: None,
            });

            match res {
                Ok(outcome) => ExitCode::from(outcome.exit_code as u8),
                Err(e) => {
                    let guidance = app_error_to_guidance(&e);
                    print_error(&guidance);
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
            feature_flags,
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
                    feature_flags,
                    out_path: out,
                    md_path: md,
                    annotations: annotations.into(),
                    tool,
                    repro,
                    fail_on_override: None,
                },
                command,
            );

            match res {
                Ok(outcome) => ExitCode::from(outcome.exit_code as u8),
                Err(e) => {
                    let guidance = app_error_to_guidance(&e);
                    print_error(&guidance);
                    ExitCode::from(1)
                }
            }
        }

        Commands::Md { report, max_items } => {
            let report_path = report_path_string(&report);
            let loaded = load_report(&report);
            match loaded {
                Ok(r) => {
                    let md = render_markdown(
                        &r,
                        MarkdownOptions {
                            max_items,
                            report_path,
                        },
                    );
                    print!("{md}");
                    ExitCode::from(0)
                }
                Err(e) => {
                    let guidance = report_load_error_guidance(&report, &e);
                    print_error(&guidance);
                    ExitCode::from(1)
                }
            }
        }

        Commands::Annotations { report, max } => {
            let report_path = report.clone();
            let loaded = load_report(&report);
            match loaded {
                Ok(r) => {
                    let out = render_github_annotations(&r, max);
                    print!("{out}");
                    ExitCode::from(0)
                }
                Err(e) => {
                    let guidance = report_load_error_guidance(&report_path, &e);
                    print_error(&guidance);
                    ExitCode::from(1)
                }
            }
        }

        Commands::Explain { code_or_check } => {
            print!("{}", explain(&code_or_check));
            ExitCode::from(0)
        }

        Commands::Ci { provider } => match provider {
            CiProvider::Github {
                base,
                head,
                root,
                config,
                fail_on,
                diagnostics,
                feature_flags,
                out,
                md,
                annotations,
            } => {
                let tool = ToolInfo {
                    name: "lintdiff".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    commit: option_env!("GIT_SHA").map(|s| s.to_string()),
                };

                let res = run_ci_github(
                    tool,
                    base,
                    head,
                    root,
                    config,
                    fail_on,
                    diagnostics,
                    feature_flags,
                    out,
                    md,
                    annotations.into(),
                );

                match res {
                    Ok(outcome) => ExitCode::from(outcome.exit_code as u8),
                    Err(e) => {
                        let guidance = app_error_to_guidance(&e);
                        print_error(&guidance);
                        ExitCode::from(1)
                    }
                }
            }
        },
    }
}

fn load_report(path: &PathBuf) -> Result<Report, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("failed to read report: {e}"))?;
    serde_json::from_str::<Report>(&raw).map_err(|e| format!("invalid report json: {e}"))
}

/// Create error guidance for report loading failures
fn report_load_error_guidance(path: &Path, error: &str) -> ErrorGuidance {
    let path_str = path.to_string_lossy();

    if error.contains("failed to read report") {
        if error.contains("No such file") || error.contains("cannot find") {
            ErrorGuidance::new(format!("Report file not found: {}", path_str))
                .context("The specified report.json file does not exist.")
                .suggestion("Run 'lintdiff ingest' first to generate a report, or check the path is correct.")
                .example("lintdiff ingest --base main --head HEAD --out artifacts/lintdiff/report.json\nlintdiff md --report artifacts/lintdiff/report.json")
        } else if error.contains("Permission denied") {
            ErrorGuidance::new(format!("Permission denied: {}", path_str))
                .context("You don't have permission to read the report file.")
                .suggestion("Check file permissions or specify a different report path.")
        } else {
            ErrorGuidance::new(error)
                .context("Failed to read the report file.")
                .suggestion("Ensure the file exists and is readable.")
        }
    } else if error.contains("invalid report json") {
        ErrorGuidance::new(format!("Invalid report format: {}", path_str))
            .context("The report file is not valid JSON or doesn't match the expected schema.")
            .suggestion("The file may be corrupted or from an incompatible lintdiff version. Regenerate the report.")
            .example("rm artifacts/lintdiff/report.json && lintdiff ingest --base main --head HEAD")
    } else {
        ErrorGuidance::new(error)
            .context("An error occurred while loading the report.")
            .suggestion("Check that the report file is valid and was generated by a compatible version of lintdiff.")
    }
}

fn report_path_string(p: &Path) -> String {
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
        parts.push(format!(
            "--base {} --head {}",
            base.as_ref().unwrap(),
            head.as_ref().unwrap()
        ));
    } else {
        parts.push("--base <base> --head <head>".to_string());
    }

    parts.join(" ")
}

/// Represents a lint explanation with structured metadata.
#[derive(Copy, Clone)]
struct LintExplanation {
    /// The lint code name (e.g., "clippy::unwrap_used")
    name: &'static str,
    /// Category: Rustc, Clippy, or Internal
    category: &'static str,
    /// Sub-category for Clippy lints (e.g., "Correctness", "Style")
    sub_category: Option<&'static str>,
    /// Brief description of the lint
    description: &'static str,
    /// Why this lint matters (optional)
    why_it_matters: Option<&'static str>,
    /// Suggested fix (optional)
    suggestion: Option<&'static str>,
    /// URL for more information (optional)
    url: Option<&'static str>,
}

impl LintExplanation {
    /// Format the explanation for display
    fn format(&self) -> String {
        let mut output = String::new();

        // Header: name [Category - SubCategory]
        if let Some(sub) = self.sub_category {
            output.push_str(&format!("{} [{} - {}]\n\n", self.name, self.category, sub));
        } else {
            output.push_str(&format!("{} [{}]\n\n", self.name, self.category));
        }

        // Description
        output.push_str(self.description);
        output.push_str("\n\n");

        // Why it matters (optional)
        if let Some(why) = self.why_it_matters {
            output.push_str(&format!("Why it matters: {}\n\n", why));
        }

        // Suggestion (optional)
        if let Some(suggestion) = self.suggestion {
            output.push_str(&format!("Suggestion: {}\n\n", suggestion));
        }

        // URL (optional)
        if let Some(url) = self.url {
            output.push_str(&format!("More info: {}\n", url));
        }

        output
    }
}

/// Normalize a lint code by stripping common prefixes for matching
fn normalize_lint_code(code: &str) -> String {
    code.trim().to_lowercase()
}

/// Get explanation for a lint code
fn explain(key: &str) -> String {
    let normalized = normalize_lint_code(key);

    // Try to find a matching lint explanation
    if let Some(explanation) = get_lint_explanation(&normalized) {
        return explanation.format();
    }

    // Fallback for unknown codes
    format!("{}\n\nNo local explanation available.\n", key)
}

/// Get the lint explanation for a normalized code
fn get_lint_explanation(normalized: &str) -> Option<LintExplanation> {
    // Check all lints
    ALL_LINTS
        .iter()
        .find(|lint| {
            let normalized_name = normalize_lint_code(lint.name);
            normalized == normalized_name ||
        // Also match without clippy:: prefix
        normalized == normalized_name.strip_prefix("clippy::").unwrap_or(&normalized_name)
        })
        .copied()
}

/// All supported lint explanations
static ALL_LINTS: &[LintExplanation] = &[
    // ==================== INTERNAL LINTDIFF CODES ====================
    LintExplanation {
        name: "diagnostics.on_diff",
        category: "Internal",
        sub_category: None,
        description: "Matches rustc/Clippy diagnostics whose primary spans intersect changed lines in the PR diff.",
        why_it_matters: Some("This is the core filter that ensures lintdiff only reports lints introduced by the current changes."),
        suggestion: Some("Ensure your changes are properly tracked in git and the diff is correctly computed."),
        url: None,
    },
    LintExplanation {
        name: "lintdiff.input.missing_diff",
        category: "Internal",
        sub_category: None,
        description: "Diff input is required but was not provided.",
        why_it_matters: Some("Without a diff, lintdiff cannot determine which lines changed and cannot filter diagnostics."),
        suggestion: Some("Provide --base and --head (for git diff) or --diff-file to specify the diff."),
        url: None,
    },
    LintExplanation {
        name: "lintdiff.input.missing_diagnostics",
        category: "Internal",
        sub_category: None,
        description: "Diagnostics input is required but was not provided.",
        why_it_matters: Some("Without diagnostics, lintdiff has nothing to filter against the diff."),
        suggestion: Some("Provide --diagnostics path or pipe cargo clippy --message-format=json output to stdin."),
        url: None,
    },
    LintExplanation {
        name: "lintdiff.matching.no_matches",
        category: "Internal",
        sub_category: None,
        description: "No diagnostics matched changed lines.",
        why_it_matters: Some("This could indicate a configuration issue or that no new lints were introduced."),
        suggestion: Some("Check path normalization (absolute vs repo-relative paths) and verify the diff range is correct."),
        url: None,
    },

    // ==================== CLIPPY LINTS ====================
    LintExplanation {
        name: "clippy::unwrap_used",
        category: "Clippy",
        sub_category: Some("Correctness"),
        description: "Using `.unwrap()` on an Option or Result will panic if the value is None or Err.",
        why_it_matters: Some("Panics in production code cause the entire program to abort, leading to poor user experience and potential data loss."),
        suggestion: Some("Use expect() with a descriptive message, unwrap_or(), unwrap_or_default(), unwrap_or_else(), or proper error handling with ? operator."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#unwrap_used"),
    },
    LintExplanation {
        name: "clippy::expect_used",
        category: "Clippy",
        sub_category: Some("Correctness"),
        description: "Using `.expect()` on an Option or Result can panic if the value is None or Err.",
        why_it_matters: Some("While expect() provides better error messages than unwrap(), it still causes panics in production."),
        suggestion: Some("Consider using unwrap_or(), unwrap_or_default(), unwrap_or_else(), or proper error handling with the ? operator."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#expect_used"),
    },
    LintExplanation {
        name: "clippy::panic",
        category: "Clippy",
        sub_category: Some("Correctness"),
        description: "Explicit panic calls will abort the program.",
        why_it_matters: Some("Panics should be avoided in library code and production applications as they cause abrupt termination."),
        suggestion: Some("Return a Result type and propagate errors, or use proper error handling patterns."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#panic"),
    },
    LintExplanation {
        name: "clippy::indexing_slicing",
        category: "Clippy",
        sub_category: Some("Correctness"),
        description: "Indexing into a slice or array may panic if the index is out of bounds.",
        why_it_matters: Some("Out-of-bounds access causes a panic, which can crash your program."),
        suggestion: Some("Use .get() which returns Option, or check bounds with .len() before indexing."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#indexing_slicing"),
    },
    LintExplanation {
        name: "clippy::too_many_arguments",
        category: "Clippy",
        sub_category: Some("Complexity"),
        description: "A function has too many arguments (more than 7 by default).",
        why_it_matters: Some("Functions with many arguments are hard to read, maintain, and call correctly."),
        suggestion: Some("Group related arguments into a struct, use the builder pattern, or refactor into multiple functions."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#too_many_arguments"),
    },
    LintExplanation {
        name: "clippy::needless_borrow",
        category: "Clippy",
        sub_category: Some("Style"),
        description: "A borrow is unnecessary because the value can be used directly.",
        why_it_matters: Some("Unnecessary borths add visual noise without providing any benefit."),
        suggestion: Some("Remove the & and use the value directly."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#needless_borrow"),
    },
    LintExplanation {
        name: "clippy::redundant_clone",
        category: "Clippy",
        sub_category: Some("Perf"),
        description: "A clone is redundant because the original value is no longer used.",
        why_it_matters: Some("Unnecessary clones hurt performance by allocating memory and copying data."),
        suggestion: Some("Remove the .clone() call and use the original value directly."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#redundant_clone"),
    },
    LintExplanation {
        name: "clippy::single_match",
        category: "Clippy",
        sub_category: Some("Style"),
        description: "A match statement with a single arm can be simplified.",
        why_it_matters: Some("Using if let is more concise and idiomatic for single-arm matches."),
        suggestion: Some("Replace with `if let pattern = expr { ... }` or add an else clause if needed."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#single_match"),
    },
    LintExplanation {
        name: "clippy::vec_init_then_push",
        category: "Clippy",
        sub_category: Some("Style"),
        description: "A Vec is initialized with Vec::new() followed by multiple push() calls.",
        why_it_matters: Some("Using the vec![] macro is more concise and can be more efficient."),
        suggestion: Some("Replace with `vec![item1, item2, ...]` or use Vec::with_capacity() if the size is known."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#vec_init_then_push"),
    },
    LintExplanation {
        name: "clippy::linkedlist",
        category: "Clippy",
        sub_category: Some("Perf"),
        description: "A LinkedList is used where a Vec would be more appropriate.",
        why_it_matters: Some("LinkedList has poor cache locality and is usually slower than Vec for most operations."),
        suggestion: Some("Use Vec or VecDeque unless you specifically need O(1) insertion/removal at both ends."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#linkedlist"),
    },
    LintExplanation {
        name: "clippy::wildcard_imports",
        category: "Clippy",
        sub_category: Some("Pedantic"),
        description: "A wildcard import (use module::*) is used.",
        why_it_matters: Some("Wildcard imports make it unclear what items are used and can cause naming conflicts."),
        suggestion: Some("Import specific items explicitly: `use module::{item1, item2};`"),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#wildcard_imports"),
    },
    LintExplanation {
        name: "clippy::unused_unit",
        category: "Clippy",
        sub_category: Some("Style"),
        description: "An explicit unit type () is used where it is unnecessary.",
        why_it_matters: Some("Unnecessary unit types add visual noise without changing semantics."),
        suggestion: Some("Remove the explicit () - functions return unit by default."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#unused_unit"),
    },
    LintExplanation {
        name: "clippy::map_identity",
        category: "Clippy",
        sub_category: Some("Complexity"),
        description: "A map closure that returns its input unchanged.",
        why_it_matters: Some("Mapping to identity is a no-op that adds unnecessary computation."),
        suggestion: Some("Remove the .map() call entirely, or replace with a meaningful transformation."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#map_identity"),
    },
    LintExplanation {
        name: "clippy::redundant_pattern",
        category: "Clippy",
        sub_category: Some("Style"),
        description: "A pattern binding is redundant (e.g., `x @ _`).",
        why_it_matters: Some("Redundant patterns add visual noise without providing any benefit."),
        suggestion: Some("Simplify the pattern by removing the redundant binding."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#redundant_pattern"),
    },
    LintExplanation {
        name: "clippy::clone_on_copy",
        category: "Clippy",
        sub_category: Some("Complexity"),
        description: "Calling .clone() on a type that implements Copy.",
        why_it_matters: Some("Copy types are copied implicitly; .clone() is redundant and misleading."),
        suggestion: Some("Remove the .clone() call - the value will be copied automatically."),
        url: Some("https://rust-lang.github.io/rust-clippy/master/index.html#clone_on_copy"),
    },

    // ==================== RUSTC LINTS ====================
    LintExplanation {
        name: "unused_variables",
        category: "Rustc",
        sub_category: None,
        description: "A variable is declared but never used.",
        why_it_matters: Some("Unused variables may indicate a bug or incomplete implementation."),
        suggestion: Some("Prefix the variable with _ (e.g., `_x`) to suppress the warning, or remove it if truly unused."),
        url: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-variables"),
    },
    LintExplanation {
        name: "dead_code",
        category: "Rustc",
        sub_category: None,
        description: "Code is defined but never used.",
        why_it_matters: Some("Dead code increases maintenance burden and may indicate incomplete features."),
        suggestion: Some("Remove the unused code, or mark it with #[allow(dead_code)] if intentionally unused (e.g., public API)."),
        url: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#dead-code"),
    },
    LintExplanation {
        name: "unused_mut",
        category: "Rustc",
        sub_category: None,
        description: "A variable is declared as mutable but never mutated.",
        why_it_matters: Some("Unnecessary mut adds confusion about the variable's intended use."),
        suggestion: Some("Remove the `mut` keyword from the variable declaration."),
        url: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-mut"),
    },
    LintExplanation {
        name: "unused_imports",
        category: "Rustc",
        sub_category: None,
        description: "An import is declared but nothing from it is used.",
        why_it_matters: Some("Unused imports add noise and may cause compilation errors if the imported module changes."),
        suggestion: Some("Remove the unused import statement."),
        url: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-imports"),
    },
    LintExplanation {
        name: "non_snake_case",
        category: "Rustc",
        sub_category: None,
        description: "A name does not follow snake_case naming convention.",
        why_it_matters: Some("Consistent naming improves code readability and follows Rust idioms."),
        suggestion: Some("Rename to snake_case (e.g., `my_variable` instead of `myVariable`)."),
        url: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#non-snake-case"),
    },
    LintExplanation {
        name: "non_camel_case_types",
        category: "Rustc",
        sub_category: None,
        description: "A type name does not follow CamelCase naming convention.",
        why_it_matters: Some("Consistent naming improves code readability and follows Rust idioms."),
        suggestion: Some("Rename to CamelCase (e.g., `MyStruct` instead of `my_struct`)."),
        url: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#non-camel-case-types"),
    },
    LintExplanation {
        name: "unused_must_use",
        category: "Rustc",
        sub_category: None,
        description: "A value marked with #[must_use] is unused.",
        why_it_matters: Some("Must-use types (like Result) should be handled to avoid silently ignoring errors or important values."),
        suggestion: Some("Handle the value explicitly, or use `let _ = ...` to intentionally discard it."),
        url: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-must-use"),
    },
    LintExplanation {
        name: "unreachable_code",
        category: "Rustc",
        sub_category: None,
        description: "Code that will never be executed.",
        why_it_matters: Some("Unreachable code may indicate a logic error or misunderstanding of control flow."),
        suggestion: Some("Remove the unreachable code, or fix the control flow that makes it unreachable."),
        url: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unreachable-code"),
    },
    LintExplanation {
        name: "deprecated",
        category: "Rustc",
        sub_category: None,
        description: "Use of a deprecated item (marked with #[deprecated]).",
        why_it_matters: Some("Deprecated items may be removed in future versions and often have better alternatives."),
        suggestion: Some("Check the deprecation message for the recommended alternative and migrate to it."),
        url: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#deprecated"),
    },
    LintExplanation {
        name: "unused_assignments",
        category: "Rustc",
        sub_category: None,
        description: "A value is assigned to a variable but never read.",
        why_it_matters: Some("Unused assignments may indicate a bug or unnecessary computation."),
        suggestion: Some("Remove the assignment if unnecessary, or use the value if it was meant to be read."),
        url: Some("https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-assignments"),
    },
];
