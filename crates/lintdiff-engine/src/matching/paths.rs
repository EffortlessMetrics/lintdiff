use lintdiff_types::NormPath;

pub fn relativize_span_path(
    file: &NormPath,
    repo_root: Option<&NormPath>,
    workspace_only: bool,
) -> Option<NormPath> {
    let s = file.as_str();

    // Already repo-relative (best effort): doesn't look absolute.
    if !looks_absolute(s) {
        return Some(NormPath::new(normalize_separators(s)));
    }

    let Some(root) = repo_root else {
        return if workspace_only {
            None
        } else {
            Some(NormPath::new(normalize_separators(s)))
        };
    };

    let root_s = root.as_str().trim_end_matches('/');
    if let Some(stripped) = s.strip_prefix(root_s) {
        let stripped = stripped.trim_start_matches('/');
        if stripped.is_empty() {
            return if workspace_only {
                None
            } else {
                Some(NormPath::new(normalize_separators(s)))
            };
        }
        return Some(NormPath::new(normalize_separators(stripped)));
    }

    if workspace_only {
        None
    } else {
        Some(NormPath::new(normalize_separators(s)))
    }
}

/// Normalize path separators to forward slashes for consistent glob matching.
fn normalize_separators(s: &str) -> String {
    s.replace('\\', "/")
}

fn looks_absolute(s: &str) -> bool {
    s.starts_with('/') || (s.len() >= 3 && s.as_bytes()[1] == b':' && s.as_bytes()[2] == b'/')
}

#[cfg(test)]
mod tests {
    use super::*;

    // Already repo-relative path stays as-is
    #[test]
    fn relative_path_passes_through() {
        let result = relativize_span_path(&NormPath::new("src/lib.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    // Absolute path stripped to relative when matching repo_root
    #[test]
    fn absolute_path_stripped_with_repo_root() {
        let result = relativize_span_path(
            &NormPath::new("/repo/src/lib.rs"),
            Some(&NormPath::new("/repo")),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    // Absolute path outside repo_root returns None when workspace_only
    #[test]
    fn absolute_path_outside_root_none_when_workspace_only() {
        let result = relativize_span_path(
            &NormPath::new("/other/src/lib.rs"),
            Some(&NormPath::new("/repo")),
            true,
        );
        assert!(result.is_none());
    }

    // Absolute path outside repo_root returns path when not workspace_only
    #[test]
    fn absolute_path_outside_root_passes_when_not_workspace_only() {
        let result = relativize_span_path(
            &NormPath::new("/other/src/lib.rs"),
            Some(&NormPath::new("/repo")),
            false,
        );
        assert!(result.is_some());
    }

    // Absolute path with no repo_root + workspace_only returns None
    #[test]
    fn absolute_path_no_root_workspace_only_none() {
        let result = relativize_span_path(&NormPath::new("/repo/src/lib.rs"), None, true);
        assert!(result.is_none());
    }

    // Absolute path with no repo_root + not workspace_only returns Some
    #[test]
    fn absolute_path_no_root_not_workspace_only_some() {
        let result = relativize_span_path(&NormPath::new("/repo/src/lib.rs"), None, false);
        assert!(result.is_some());
    }

    // Windows-style path with backslashes gets normalized
    #[test]
    fn windows_path_normalized() {
        let result = relativize_span_path(&NormPath::new("src\\lib.rs"), None, true);
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }

    // Root with trailing slash works
    #[test]
    fn repo_root_trailing_slash() {
        let result = relativize_span_path(
            &NormPath::new("/repo/src/lib.rs"),
            Some(&NormPath::new("/repo/")),
            true,
        );
        assert_eq!(result.unwrap().as_str(), "src/lib.rs");
    }
}
