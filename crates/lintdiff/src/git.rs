use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use lintdiff_types::GitInfo;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppGitError {
    #[error("failed to run git command: {msg}")]
    Command { msg: String },
    #[error("failed to determine repository root: {msg}")]
    RepoRoot { msg: String },
}

pub fn acquire_diff(
    repo_root: &Path,
    diff_file: Option<&Path>,
    base: Option<&str>,
    head: Option<&str>,
) -> Result<String, AppGitError> {
    if let Some(p) = diff_file {
        return std::fs::read_to_string(p).map_err(|e| AppGitError::Command {
            msg: format!("failed to read diff file '{}': {e}", p.to_string_lossy()),
        });
    }

    let Some(base) = base else {
        return Err(AppGitError::Command {
            msg: "missing --base (or provide --diff-file)".to_string(),
        });
    };
    let Some(head) = head else {
        return Err(AppGitError::Command {
            msg: "missing --head (or provide --diff-file)".to_string(),
        });
    };

    let range = format!("{base}..{head}");
    let out = Command::new("git")
        .current_dir(repo_root)
        .args(["diff", "--unified=0", &range])
        .output()
        .map_err(|e| AppGitError::Command {
            msg: format!("failed to run git diff: {e}"),
        })?;

    if !out.status.success() {
        return Err(AppGitError::Command {
            msg: format!("git diff failed: {}", String::from_utf8_lossy(&out.stderr)),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn determine_repo_root(explicit: Option<&Path>) -> Result<PathBuf, AppGitError> {
    if let Some(p) = explicit {
        return Ok(p.to_path_buf());
    }

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

    std::env::current_dir().map_err(|e| AppGitError::RepoRoot {
        msg: format!("failed to determine repo root: {e}"),
    })
}

pub fn gather_git_info(
    repo_root: &Path,
    base: Option<&str>,
    head: Option<&str>,
) -> Result<GitInfo, AppGitError> {
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

pub fn current_revision(repo_root: &Path) -> Result<String, AppGitError> {
    let out = Command::new("git")
        .current_dir(repo_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| AppGitError::Command {
            msg: format!("failed to run git rev-parse: {e}"),
        })?;

    if !out.status.success() {
        return Err(AppGitError::Command {
            msg: format!(
                "git rev-parse failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_merge_base(repo_root: &Path, base: &str, head: &str) -> Result<String, AppGitError> {
    let out = Command::new("git")
        .current_dir(repo_root)
        .args(["merge-base", base, head])
        .output()
        .map_err(|e| AppGitError::Command {
            msg: format!("failed to run git merge-base: {e}"),
        })?;

    if !out.status.success() {
        return Err(AppGitError::Command {
            msg: format!(
                "git merge-base failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_config_get<I, S>(repo_root: &Path, args: I) -> Result<String, AppGitError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let out = Command::new("git")
        .current_dir(repo_root)
        .args(args)
        .output()
        .map_err(|e| AppGitError::Command {
            msg: format!("failed to run git config: {e}"),
        })?;

    if !out.status.success() {
        return Err(AppGitError::Command {
            msg: format!(
                "git config failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_revision_reads_repository_head() -> Result<(), AppGitError> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| AppGitError::RepoRoot {
                msg: "missing workspace root".to_string(),
            })?
            .to_path_buf();
        let revision = current_revision(&root)?;
        if revision.is_empty() {
            return Err(AppGitError::Command {
                msg: "git revision was empty".to_string(),
            });
        }
        Ok(())
    }

    #[test]
    fn current_revision_reports_invalid_repository() {
        let error = current_revision(Path::new("C:/this/path/does/not/exist"))
            .expect_err("invalid repository should fail");
        assert!(error.to_string().contains("git"));
    }
}
