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
