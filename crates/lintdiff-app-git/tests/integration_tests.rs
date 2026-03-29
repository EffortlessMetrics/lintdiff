//! Integration tests for lintdiff-app-git
//!
//! These tests create temporary git repositories to test git operations.
//! Tests are skipped if git is not available on the system.

use std::fs;
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

use lintdiff_app_git::{acquire_diff, determine_repo_root, gather_git_info, AppGitError};
use tempfile::TempDir;

/// Helper to check if git is available on the system
fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Helper to run a git command in a directory
fn git_run(dir: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(|e| format!("failed to execute git: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn cwd_lock() -> MutexGuard<'static, ()> {
    static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    CWD_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("cwd lock poisoned")
}

struct CurrentDirGuard {
    original_dir: std::path::PathBuf,
}

impl CurrentDirGuard {
    fn change_to(path: &std::path::Path) -> Self {
        let original_dir = std::env::current_dir().expect("failed to get current dir");
        std::env::set_current_dir(path).expect("failed to change dir");
        Self { original_dir }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.original_dir).expect("failed to restore dir");
    }
}

/// Helper to create a temporary git repository with initial commit
fn create_test_repo() -> Result<TempDir, String> {
    let temp_dir = TempDir::new().map_err(|e| format!("failed to create temp dir: {e}"))?;
    let repo_path = temp_dir.path();

    // Initialize git repo with main as default branch
    git_run(repo_path, &["init", "--initial-branch=main"])?;
    git_run(repo_path, &["config", "user.email", "test@example.com"])?;
    git_run(repo_path, &["config", "user.name", "Test User"])?;

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repository\n").map_err(|e| e.to_string())?;
    git_run(repo_path, &["add", "README.md"])?;
    git_run(repo_path, &["commit", "-m", "Initial commit"])?;

    // Ensure branch is named "main" (for compatibility with older git versions)
    git_run(repo_path, &["branch", "-M", "main"])?;

    Ok(temp_dir)
}

/// Helper to create a commit with a file
fn create_commit(
    repo_path: &std::path::Path,
    filename: &str,
    content: &str,
    message: &str,
) -> Result<String, String> {
    fs::write(repo_path.join(filename), content).map_err(|e| e.to_string())?;
    git_run(repo_path, &["add", filename])?;
    git_run(repo_path, &["commit", "-m", message])?;
    git_run(repo_path, &["rev-parse", "HEAD"])
}

// =============================================================================
// Repo Root Detection Tests
// =============================================================================

mod repo_root_detection {
    use super::*;

    #[test]
    fn test_determine_repo_root_explicit_path() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let explicit_path = std::env::current_dir().unwrap();
        let result = determine_repo_root(Some(&explicit_path));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), explicit_path);
    }

    #[test]
    fn test_determine_repo_root_from_subdirectory() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let _cwd_lock = cwd_lock();
        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create a subdirectory
        let subdir = repo_path.join("src").join("nested").join("deep");
        fs::create_dir_all(&subdir).expect("failed to create subdir");

        // Change to subdirectory and find root.
        let _cwd_guard = CurrentDirGuard::change_to(&subdir);

        let result = determine_repo_root(None);

        assert!(result.is_ok());
        let found_root = result.unwrap();
        // On Windows, git may return paths with different casing or format
        // So we check canonicalized paths
        let found_canonical = found_root.canonicalize().unwrap_or(found_root);
        let expected_canonical = repo_path.canonicalize().unwrap_or(repo_path.to_path_buf());
        assert_eq!(found_canonical, expected_canonical);
    }

    #[test]
    fn test_determine_repo_root_nested_repos() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let _cwd_lock = cwd_lock();
        let outer_repo = create_test_repo().expect("failed to create outer repo");
        let outer_path = outer_repo.path();

        // Create a nested git repository
        let inner_path = outer_path.join("nested");
        fs::create_dir_all(&inner_path).expect("failed to create nested dir");

        git_run(&inner_path, &["init"]).expect("failed to init inner repo");
        git_run(&inner_path, &["config", "user.email", "test@example.com"]).expect("failed config");
        git_run(&inner_path, &["config", "user.name", "Test User"]).expect("failed config");

        // Create a file in inner repo
        fs::write(inner_path.join("inner.txt"), "inner content").expect("failed to write");
        git_run(&inner_path, &["add", "inner.txt"]).expect("failed to add");
        git_run(&inner_path, &["commit", "-m", "Inner commit"]).expect("failed to commit");

        // From inner repo, should find inner repo root.
        let _cwd_guard = CurrentDirGuard::change_to(&inner_path);

        let result = determine_repo_root(None);

        assert!(result.is_ok());
        let found_root = result.unwrap();
        let found_canonical = found_root.canonicalize().unwrap_or(found_root);
        let expected_canonical = inner_path
            .canonicalize()
            .unwrap_or(inner_path.to_path_buf());
        assert_eq!(found_canonical, expected_canonical);
    }

    #[test]
    fn test_determine_repo_root_non_git_directory() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let _cwd_lock = cwd_lock();
        // Create a temporary directory without git
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let non_git_path = temp_dir.path();

        let _cwd_guard = CurrentDirGuard::change_to(non_git_path);

        // determine_repo_root should fall back to current_dir when git command fails
        let result = determine_repo_root(None);

        // Should return current directory as fallback
        assert!(result.is_ok());
    }
}

// =============================================================================
// Diff Acquisition Tests
// =============================================================================

mod diff_acquisition {
    use super::*;

    #[test]
    fn test_acquire_diff_between_two_commits() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create first commit
        let sha1 = create_commit(repo_path, "file1.txt", "Content 1\n", "Add file1")
            .expect("failed to create commit");

        // Create second commit
        let sha2 = create_commit(repo_path, "file2.txt", "Content 2\n", "Add file2")
            .expect("failed to create commit");

        // Get diff between commits
        let result = acquire_diff(repo_path, None, Some(&sha1), Some(&sha2));
        assert!(result.is_ok());

        let diff = result.unwrap();
        assert!(diff.contains("file2.txt"));
        assert!(diff.contains("+Content 2"));
    }

    #[test]
    fn test_acquire_diff_with_modified_file() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create initial file
        let sha1 = create_commit(repo_path, "config.txt", "setting=value\n", "Add config")
            .expect("failed to create commit");

        // Modify the file
        let sha2 = create_commit(
            repo_path,
            "config.txt",
            "setting=newvalue\nother=thing\n",
            "Update config",
        )
        .expect("failed to create commit");

        // Get diff
        let result = acquire_diff(repo_path, None, Some(&sha1), Some(&sha2));
        assert!(result.is_ok());

        let diff = result.unwrap();
        assert!(diff.contains("config.txt"));
        assert!(diff.contains("-setting=value"));
        assert!(diff.contains("+setting=newvalue"));
        assert!(diff.contains("+other=thing"));
    }

    #[test]
    fn test_acquire_diff_with_renamed_file() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create file
        let sha1 = create_commit(repo_path, "old_name.txt", "Same content\n", "Add file")
            .expect("failed to create commit");

        // Rename file using git mv
        git_run(repo_path, &["mv", "old_name.txt", "new_name.txt"]).expect("failed to rename");
        git_run(repo_path, &["commit", "-m", "Rename file"]).expect("failed to commit");
        let sha2 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        // Get diff with rename detection
        let result = acquire_diff(repo_path, None, Some(&sha1), Some(&sha2));
        assert!(result.is_ok());

        let diff = result.unwrap();
        // The diff should mention both files
        assert!(diff.contains("old_name.txt") || diff.contains("new_name.txt"));
    }

    #[test]
    fn test_acquire_diff_with_binary_file() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create initial commit
        let sha1 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        // Create a binary file (simple binary content with null bytes)
        let binary_content = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];
        fs::write(repo_path.join("binary.bin"), &binary_content).expect("failed to write binary");
        git_run(repo_path, &["add", "binary.bin"]).expect("failed to add");
        git_run(repo_path, &["commit", "-m", "Add binary file"]).expect("failed to commit");
        let sha2 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        // Get diff - should handle binary gracefully
        let result = acquire_diff(repo_path, None, Some(&sha1), Some(&sha2));
        assert!(result.is_ok());

        let diff = result.unwrap();
        // Binary files show as "Binary files a/... and b/... differ" or similar
        assert!(diff.contains("binary.bin") || diff.contains("Binary"));
    }

    #[test]
    fn test_acquire_diff_empty_diff() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create a commit
        let sha = create_commit(repo_path, "file.txt", "Content\n", "Add file")
            .expect("failed to create commit");

        // Diff same commit against itself should be empty
        let result = acquire_diff(repo_path, None, Some(&sha), Some(&sha));
        assert!(result.is_ok());

        let diff = result.unwrap();
        // Empty diff should be empty string or just whitespace
        assert!(diff.trim().is_empty());
    }

    #[test]
    fn test_acquire_diff_missing_base() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        let result = acquire_diff(repo_path, None, None, Some("HEAD"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppGitError::Command { .. }));
    }

    #[test]
    fn test_acquire_diff_missing_head() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        let result = acquire_diff(repo_path, None, Some("HEAD"), None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppGitError::Command { .. }));
    }

    #[test]
    fn test_acquire_diff_from_file() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let repo_path = temp_dir.path();

        // Create a diff file
        let diff_content = "diff --git a/test.txt b/test.txt\nnew file mode 100644\nindex 0000000..e69de29\n--- /dev/null\n+++ b/test.txt\n@@ -0,0 +1 @@\n+hello\n";
        let diff_file = repo_path.join("test.diff");
        fs::write(&diff_file, diff_content).expect("failed to write diff file");

        let result = acquire_diff(repo_path, Some(&diff_file), None, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), diff_content);
    }

    #[test]
    fn test_acquire_diff_file_not_found() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let repo_path = temp_dir.path();

        let non_existent = repo_path.join("nonexistent.diff");
        let result = acquire_diff(repo_path, Some(&non_existent), None, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppGitError::Command { .. }));
    }

    #[test]
    fn test_acquire_diff_with_deleted_file() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create file
        let sha1 = create_commit(repo_path, "to_delete.txt", "Delete me\n", "Add file")
            .expect("failed to create commit");

        // Delete the file
        fs::remove_file(repo_path.join("to_delete.txt")).expect("failed to delete");
        git_run(repo_path, &["add", "-A"]).expect("failed to stage deletion");
        git_run(repo_path, &["commit", "-m", "Delete file"]).expect("failed to commit");
        let sha2 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        // Get diff
        let result = acquire_diff(repo_path, None, Some(&sha1), Some(&sha2));
        assert!(result.is_ok());

        let diff = result.unwrap();
        assert!(diff.contains("to_delete.txt"));
        assert!(diff.contains("-Delete me"));
    }

    #[test]
    fn test_acquire_diff_multiple_files() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create initial commit
        let sha1 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        // Create multiple files
        fs::write(repo_path.join("file1.rs"), "fn main() {}\n").expect("failed to write");
        fs::write(repo_path.join("file2.rs"), "fn other() {}\n").expect("failed to write");
        fs::write(repo_path.join("file3.txt"), "text content\n").expect("failed to write");
        git_run(repo_path, &["add", "."]).expect("failed to add");
        git_run(repo_path, &["commit", "-m", "Add multiple files"]).expect("failed to commit");
        let sha2 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        // Get diff
        let result = acquire_diff(repo_path, None, Some(&sha1), Some(&sha2));
        assert!(result.is_ok());

        let diff = result.unwrap();
        assert!(diff.contains("file1.rs"));
        assert!(diff.contains("file2.rs"));
        assert!(diff.contains("file3.txt"));
    }
}

// =============================================================================
// Git Info Retrieval Tests
// =============================================================================

mod git_info_retrieval {
    use super::*;

    #[test]
    fn test_gather_git_info_basic() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        let result = gather_git_info(repo_path, None, None);
        assert!(result.is_ok());

        let info = result.unwrap();
        // Without a remote, repo should be None
        assert!(info.repo.is_none());
        // Without base and head, merge_base should be None
        assert!(info.merge_base.is_none());
    }

    #[test]
    fn test_gather_git_info_with_remote() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Add a remote
        git_run(
            repo_path,
            &[
                "remote",
                "add",
                "origin",
                "https://github.com/test/repo.git",
            ],
        )
        .expect("failed to add remote");

        let result = gather_git_info(repo_path, None, None);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(info.repo.is_some());
        assert_eq!(info.repo.unwrap(), "https://github.com/test/repo.git");
    }

    #[test]
    fn test_gather_git_info_with_merge_base() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create a branch and make commits
        let base_sha = create_commit(repo_path, "base.txt", "base content\n", "Base commit")
            .expect("failed to create commit");

        git_run(repo_path, &["checkout", "-b", "feature"]).expect("failed to create branch");
        let head_sha = create_commit(
            repo_path,
            "feature.txt",
            "feature content\n",
            "Feature commit",
        )
        .expect("failed to create commit");

        let result = gather_git_info(repo_path, Some(&base_sha), Some(&head_sha));
        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(info.merge_base.is_some());
        // Merge base of base and head should be the base commit
        assert_eq!(info.merge_base.unwrap(), base_sha);

        // Cleanup - go back to main
        git_run(repo_path, &["checkout", "main"]).ok();
    }

    #[test]
    fn test_gather_git_info_with_refs() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        let base = "main";
        let head = "HEAD";

        let result = gather_git_info(repo_path, Some(base), Some(head));
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.base_sha, Some(base.to_string()));
        assert_eq!(info.head_sha, Some(head.to_string()));
    }

    #[test]
    fn test_gather_git_info_ssh_remote() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Add SSH remote
        git_run(
            repo_path,
            &["remote", "add", "origin", "git@github.com:test/repo.git"],
        )
        .expect("failed to add remote");

        let result = gather_git_info(repo_path, None, None);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.repo, Some("git@github.com:test/repo.git".to_string()));
    }

    #[test]
    fn test_gather_git_info_no_remote() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Don't add any remote
        let result = gather_git_info(repo_path, None, None);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(info.repo.is_none());
    }
}

// =============================================================================
// Edge Cases and Error Handling Tests
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_acquire_diff_invalid_ref() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        let result = acquire_diff(repo_path, None, Some("nonexistent-ref"), Some("HEAD"));
        assert!(result.is_err());
    }

    #[test]
    fn test_acquire_diff_with_subdirectory() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create subdirectory and file
        let sha1 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");
        fs::create_dir_all(repo_path.join("src/deep/nested")).expect("failed to create dir");
        fs::write(repo_path.join("src/deep/nested/file.rs"), "fn test() {}\n")
            .expect("failed to write");
        git_run(repo_path, &["add", "."]).expect("failed to add");
        git_run(repo_path, &["commit", "-m", "Add nested file"]).expect("failed to commit");
        let sha2 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        let result = acquire_diff(repo_path, None, Some(&sha1), Some(&sha2));
        assert!(result.is_ok());

        let diff = result.unwrap();
        // Path should use forward slashes in diff output
        assert!(
            diff.contains("src/deep/nested/file.rs") || diff.contains("src\\deep\\nested\\file.rs")
        );
    }

    #[test]
    fn test_acquire_diff_with_special_characters_in_filename() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create file with spaces
        let sha1 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");
        fs::write(repo_path.join("file with spaces.txt"), "content\n").expect("failed to write");
        git_run(repo_path, &["add", "."]).expect("failed to add");
        git_run(repo_path, &["commit", "-m", "Add file with spaces"]).expect("failed to commit");
        let sha2 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        let result = acquire_diff(repo_path, None, Some(&sha1), Some(&sha2));
        assert!(result.is_ok());

        let diff = result.unwrap();
        assert!(diff.contains("file with spaces.txt"));
    }

    #[test]
    fn test_acquire_diff_large_file() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create a large file (1MB of repeated content)
        let sha1 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");
        let large_content = "x".repeat(1024 * 1024);
        fs::write(repo_path.join("large.txt"), &large_content).expect("failed to write");
        git_run(repo_path, &["add", "."]).expect("failed to add");
        git_run(repo_path, &["commit", "-m", "Add large file"]).expect("failed to commit");
        let sha2 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        let result = acquire_diff(repo_path, None, Some(&sha1), Some(&sha2));
        assert!(result.is_ok());

        let diff = result.unwrap();
        assert!(diff.contains("large.txt"));
    }

    #[test]
    fn test_acquire_diff_unicode_content() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create file with unicode content
        let sha1 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");
        let unicode_content = "Hello 世界 🌍\nПривет мир\n";
        fs::write(repo_path.join("unicode.txt"), unicode_content).expect("failed to write");
        git_run(repo_path, &["add", "."]).expect("failed to add");
        git_run(repo_path, &["commit", "-m", "Add unicode file"]).expect("failed to commit");
        let sha2 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        let result = acquire_diff(repo_path, None, Some(&sha1), Some(&sha2));
        assert!(result.is_ok());

        let diff = result.unwrap();
        assert!(diff.contains("unicode.txt"));
        // The unicode content should be preserved
        assert!(diff.contains("世界") || diff.contains("🌍"));
    }

    #[test]
    #[cfg_attr(windows, ignore = "symlinks require admin privileges on Windows")]
    fn test_acquire_diff_symlink() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create file and symlink
        let sha1 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");
        fs::write(repo_path.join("original.txt"), "original content\n").expect("failed to write");

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(repo_path.join("original.txt"), repo_path.join("link.txt"))
                .expect("failed to create symlink");
        }

        git_run(repo_path, &["add", "."]).expect("failed to add");
        git_run(repo_path, &["commit", "-m", "Add file and symlink"]).expect("failed to commit");
        let sha2 = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        let result = acquire_diff(repo_path, None, Some(&sha1), Some(&sha2));
        assert!(result.is_ok());
    }
}

// =============================================================================
// Current Branch and Commit Tests
// =============================================================================

mod branch_and_commit {
    use super::*;

    #[test]
    fn test_current_branch_main() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Get current branch
        let branch =
            git_run(repo_path, &["branch", "--show-current"]).expect("failed to get branch");
        assert!(!branch.is_empty());
    }

    #[test]
    fn test_current_commit_sha() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Get current SHA
        let sha = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");
        assert_eq!(sha.len(), 40); // SHA-1 is 40 hex characters
        assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_short_sha() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Get short SHA
        let short_sha =
            git_run(repo_path, &["rev-parse", "--short", "HEAD"]).expect("failed to get short sha");
        assert!(!short_sha.is_empty());
        assert!(short_sha.len() < 40);
        assert!(short_sha.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_create_and_switch_branch() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create and switch to new branch
        git_run(repo_path, &["checkout", "-b", "feature-branch"]).expect("failed to create branch");

        let branch =
            git_run(repo_path, &["branch", "--show-current"]).expect("failed to get branch");
        assert_eq!(branch, "feature-branch");

        // Cleanup
        git_run(repo_path, &["checkout", "main"]).ok();
    }

    #[test]
    fn test_detached_head() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Get a commit SHA
        let sha = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        // Checkout detached HEAD
        git_run(repo_path, &["checkout", &sha]).expect("failed to checkout");

        // In detached HEAD, --show-current returns empty
        let branch = git_run(repo_path, &["branch", "--show-current"]).unwrap_or_default();

        // Should be empty in detached HEAD state
        assert!(branch.is_empty());

        // Cleanup - go back to main
        git_run(repo_path, &["checkout", "main"]).ok();
    }
}

// =============================================================================
// Merge Base Tests
// =============================================================================

mod merge_base {
    use super::*;

    #[test]
    fn test_merge_base_same_commit() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        let sha = git_run(repo_path, &["rev-parse", "HEAD"]).expect("failed to get sha");

        // Merge base of same commit should be that commit
        let merge_base =
            git_run(repo_path, &["merge-base", &sha, &sha]).expect("failed to get merge base");
        assert_eq!(merge_base, sha);
    }

    #[test]
    fn test_merge_base_diverged_branches() {
        if !git_available() {
            eprintln!("Skipping test: git not available");
            return;
        }

        let temp_dir = create_test_repo().expect("failed to create test repo");
        let repo_path = temp_dir.path();

        // Create base commit
        let base = create_commit(repo_path, "base.txt", "base\n", "Base")
            .expect("failed to create commit");

        // Create feature branch from here
        git_run(repo_path, &["checkout", "-b", "feature"]).expect("failed to create branch");
        let feature_sha = create_commit(repo_path, "feature.txt", "feature\n", "Feature")
            .expect("failed to create commit");

        // Go back to main and make another commit
        git_run(repo_path, &["checkout", "main"]).expect("failed to checkout main");
        let main_sha = create_commit(repo_path, "main.txt", "main\n", "Main commit")
            .expect("failed to create commit");

        // Merge base should be the common ancestor
        let merge_base = git_run(repo_path, &["merge-base", &feature_sha, &main_sha])
            .expect("failed to get merge base");
        assert_eq!(merge_base, base);
    }
}
