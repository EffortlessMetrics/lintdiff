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
