use lintdiff_types::{Counts, EffectiveConfig, Finding, Severity, Verdict, VerdictStatus};

pub fn counts_from_findings(findings: &[Finding]) -> Counts {
    let mut c = Counts::default();
    for f in findings {
        match f.severity {
            Severity::Info => c.info += 1,
            Severity::Warn => c.warn += 1,
            Severity::Error => c.error += 1,
        }
    }
    c
}

pub fn compute_verdict(
    cfg: &EffectiveConfig,
    findings: &[Finding],
    suppressed: u32,
    denied: u32,
) -> Verdict {
    let counts = counts_from_findings(findings);

    let has_error = counts.error > 0;
    let has_warn = counts.warn > 0;

    let mut reasons: Vec<String> = Vec::new();
    if suppressed > 0 {
        reasons.push("suppressed".to_string());
    }
    if denied > 0 {
        reasons.push("deny_list".to_string());
    }

    let status = match cfg.fail_on {
        lintdiff_types::FailOn::Error => {
            if has_error {
                VerdictStatus::Fail
            } else if has_warn {
                VerdictStatus::Warn
            } else {
                VerdictStatus::Pass
            }
        }
        lintdiff_types::FailOn::Warn => {
            if has_error || has_warn {
                VerdictStatus::Fail
            } else {
                VerdictStatus::Pass
            }
        }
        lintdiff_types::FailOn::Never => {
            if has_error || has_warn {
                VerdictStatus::Warn
            } else {
                VerdictStatus::Pass
            }
        }
    };

    Verdict {
        status,
        counts,
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintdiff_types::{FailOn, Finding, LintdiffConfig, Location, NormPath};

    fn finding_with_severity(sev: Severity) -> Finding {
        Finding {
            severity: sev,
            check_id: Some("test".to_string()),
            code: "test.code".to_string(),
            message: "test message".to_string(),
            location: Some(Location {
                path: NormPath::new("src/lib.rs"),
                line: Some(1),
                col: None,
            }),
            help: None,
            url: None,
            fingerprint: None,
            data: None,
        }
    }

    fn cfg_with_fail_on(fail_on: FailOn) -> EffectiveConfig {
        let mut c = LintdiffConfig::default();
        c.fail_on = Some(fail_on);
        c.effective()
    }

    #[test]
    fn fail_on_error_with_errors_is_fail() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![finding_with_severity(Severity::Error)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn fail_on_error_with_warnings_only_is_warn() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![finding_with_severity(Severity::Warn)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Warn);
    }

    #[test]
    fn fail_on_error_with_info_only_is_pass() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let findings = vec![finding_with_severity(Severity::Info)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Pass);
    }

    #[test]
    fn fail_on_error_with_no_findings_is_pass() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 0, 0);
        assert_eq!(v.status, VerdictStatus::Pass);
    }

    #[test]
    fn fail_on_warn_with_warnings_is_fail() {
        let cfg = cfg_with_fail_on(FailOn::Warn);
        let findings = vec![finding_with_severity(Severity::Warn)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn fail_on_warn_with_errors_is_fail() {
        let cfg = cfg_with_fail_on(FailOn::Warn);
        let findings = vec![finding_with_severity(Severity::Error)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn fail_on_warn_with_info_only_is_pass() {
        let cfg = cfg_with_fail_on(FailOn::Warn);
        let findings = vec![finding_with_severity(Severity::Info)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Pass);
    }

    #[test]
    fn fail_on_never_with_errors_is_warn() {
        let cfg = cfg_with_fail_on(FailOn::Never);
        let findings = vec![finding_with_severity(Severity::Error)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Warn);
    }

    #[test]
    fn fail_on_never_with_warnings_is_warn() {
        let cfg = cfg_with_fail_on(FailOn::Never);
        let findings = vec![finding_with_severity(Severity::Warn)];
        let v = compute_verdict(&cfg, &findings, 0, 0);
        assert_eq!(v.status, VerdictStatus::Warn);
    }

    #[test]
    fn fail_on_never_with_no_findings_is_pass() {
        let cfg = cfg_with_fail_on(FailOn::Never);
        let v = compute_verdict(&cfg, &[], 0, 0);
        assert_eq!(v.status, VerdictStatus::Pass);
    }

    #[test]
    fn suppressed_count_adds_reason() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 3, 0);
        assert!(v.reasons.contains(&"suppressed".to_string()));
    }

    #[test]
    fn denied_count_adds_reason() {
        let cfg = cfg_with_fail_on(FailOn::Error);
        let v = compute_verdict(&cfg, &[], 0, 2);
        assert!(v.reasons.contains(&"deny_list".to_string()));
    }

    #[test]
    fn counts_from_findings_tallies_correctly() {
        let findings = vec![
            finding_with_severity(Severity::Error),
            finding_with_severity(Severity::Warn),
            finding_with_severity(Severity::Warn),
            finding_with_severity(Severity::Info),
        ];
        let c = counts_from_findings(&findings);
        assert_eq!(c.error, 1);
        assert_eq!(c.warn, 2);
        assert_eq!(c.info, 1);
    }
}
