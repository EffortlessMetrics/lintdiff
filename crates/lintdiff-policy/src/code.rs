use lintdiff_diagnostics::DiagnosticLevel;
use lintdiff_types::Severity;

pub fn map_level_to_severity(level: &DiagnosticLevel) -> Severity {
    match level {
        DiagnosticLevel::Error => Severity::Error,
        DiagnosticLevel::Warning => Severity::Warn,
        DiagnosticLevel::Note | DiagnosticLevel::Help => Severity::Info,
        DiagnosticLevel::Other(_) => Severity::Info,
    }
}

pub fn format_level(level: &DiagnosticLevel) -> String {
    match level {
        DiagnosticLevel::Error => "error".to_string(),
        DiagnosticLevel::Warning => "warning".to_string(),
        DiagnosticLevel::Note => "note".to_string(),
        DiagnosticLevel::Help => "help".to_string(),
        DiagnosticLevel::Other(s) => s.clone(),
    }
}

pub fn normalize_diagnostic_code(raw: Option<&str>) -> (String, Option<String>) {
    let Some(raw) = raw else {
        return ("lintdiff.diagnostic.unknown".to_string(), None);
    };

    if raw.starts_with("clippy::") {
        let name = raw.trim_start_matches("clippy::");
        let slug = slugify(name);
        let url = Some(format!(
            "https://rust-lang.github.io/rust-clippy/master/index.html#{}",
            slug
        ));
        return (format!("lintdiff.diagnostic.clippy.{slug}"), url);
    }

    if is_rustc_error_code(raw) {
        let url = Some(format!("https://doc.rust-lang.org/error_codes/{raw}.html"));
        return (format!("lintdiff.diagnostic.rustc.{raw}"), url);
    }

    let slug = slugify(raw);
    (format!("lintdiff.diagnostic.rustc_lint.{slug}"), None)
}

fn is_rustc_error_code(raw: &str) -> bool {
    let b = raw.as_bytes();
    if b.len() != 5 || b[0] != b'E' {
        return false;
    }
    b[1..].iter().all(|c| c.is_ascii_digit())
}

fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else if ch == ':' {
            out.push('.');
        } else {
            out.push('_');
        }
    }
    while out.contains("..") {
        out = out.replace("..", ".");
    }
    out.trim_matches('.').to_string()
}

pub fn is_code_allowed(cfg: &lintdiff_types::EffectiveConfig, code: &str) -> bool {
    if !cfg.filter.allow_codes.is_empty() {
        return cfg.filter.allow_codes.iter().any(|c| c == code);
    }
    if cfg.filter.suppress_codes.iter().any(|c| c == code) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintdiff_types::LintdiffConfig;

    #[test]
    fn maps_unknown_code() {
        assert_eq!(
            normalize_diagnostic_code(Some("E0502")),
            (
                "lintdiff.diagnostic.rustc.E0502".to_string(),
                Some("https://doc.rust-lang.org/error_codes/E0502.html".to_string())
            )
        );
    }

    #[test]
    fn allow_list_is_hard_filter() {
        let mut cfg = LintdiffConfig::default();
        cfg.filter.allow_codes = vec!["keep".into()];
        let eff = cfg.effective();
        assert!(is_code_allowed(&eff, "keep"));
        assert!(!is_code_allowed(&eff, "drop"));
    }
}
