use globset::{Glob, GlobSet, GlobSetBuilder};

use lintdiff_types::EffectiveConfig;

#[derive(Debug, Clone)]
pub struct Filters {
    pub include: Option<GlobSet>,
    pub exclude: Option<GlobSet>,
}

pub fn compile_filters(cfg: &EffectiveConfig) -> Filters {
    Filters {
        include: build_globset(&cfg.filter.include_paths),
        exclude: build_globset(&cfg.filter.exclude_paths),
    }
}

pub fn path_allowed(filters: &Filters, path: &str) -> bool {
    if let Some(ex) = &filters.exclude {
        if ex.is_match(path) {
            return false;
        }
    }
    if let Some(inc) = &filters.include {
        return inc.is_match(path);
    }
    true
}

fn build_globset(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }

    let mut b = GlobSetBuilder::new();
    for p in patterns {
        if let Ok(g) = Glob::new(p) {
            b.add(g);
        }
    }
    b.build().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintdiff_types::LintdiffConfig;

    fn filters_from_config(include: &[&str], exclude: &[&str]) -> Filters {
        let mut cfg = LintdiffConfig::default();
        cfg.filter.include_paths = include.iter().map(|s| s.to_string()).collect();
        cfg.filter.exclude_paths = exclude.iter().map(|s| s.to_string()).collect();
        compile_filters(&cfg.effective())
    }

    #[test]
    fn empty_filters_allow_all() {
        let f = filters_from_config(&[], &[]);
        assert!(path_allowed(&f, "src/lib.rs"));
        assert!(path_allowed(&f, "anything/at/all.txt"));
    }

    #[test]
    fn exclude_blocks_matching_path() {
        let f = filters_from_config(&[], &["src/lib.rs"]);
        assert!(!path_allowed(&f, "src/lib.rs"));
        assert!(path_allowed(&f, "src/main.rs"));
    }

    #[test]
    fn include_restricts_to_matching() {
        let f = filters_from_config(&["src/**/*.rs"], &[]);
        assert!(path_allowed(&f, "src/lib.rs"));
        assert!(path_allowed(&f, "src/nested/mod.rs"));
        assert!(!path_allowed(&f, "tests/integration.rs"));
    }

    #[test]
    fn exclude_takes_precedence_over_include() {
        let f = filters_from_config(&["src/**"], &["src/lib.rs"]);
        assert!(!path_allowed(&f, "src/lib.rs"));
        assert!(path_allowed(&f, "src/main.rs"));
    }

    #[test]
    fn glob_double_star_matches_nested() {
        let f = filters_from_config(&[], &["**/generated/**"]);
        assert!(!path_allowed(&f, "src/generated/api.rs"));
        assert!(!path_allowed(&f, "deep/nested/generated/file.rs"));
        assert!(path_allowed(&f, "src/lib.rs"));
    }

    #[test]
    fn multiple_exclude_patterns() {
        let f = filters_from_config(&[], &["*.generated.rs", "target/**"]);
        assert!(!path_allowed(&f, "src/api.generated.rs"));
        assert!(!path_allowed(&f, "target/debug/main"));
        assert!(path_allowed(&f, "src/lib.rs"));
    }
}
