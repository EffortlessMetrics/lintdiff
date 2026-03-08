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
