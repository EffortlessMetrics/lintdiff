use std::cmp::Ordering;

use crate::{Finding, Severity};

/// Deterministic ordering contract for findings.
///
/// Normative ordering key:
/// 1) severity desc (error > warn > info)
/// 2) path asc
/// 3) line asc (missing last)
/// 4) code asc
/// 5) message asc
pub fn sort_findings(findings: &mut [Finding]) {
    findings.sort_by(sort_findings_cmp);
}

fn severity_rank(s: &Severity) -> u8 {
    match s {
        Severity::Error => 0,
        Severity::Warn => 1,
        Severity::Info => 2,
    }
}

/// Comparator for deterministic finding ordering. Public for paired-sort use.
pub fn sort_findings_cmp(a: &Finding, b: &Finding) -> Ordering {
    severity_rank(&a.severity)
        .cmp(&severity_rank(&b.severity))
        .then_with(|| path_of(a).cmp(path_of(b)))
        .then_with(|| line_of(a).cmp(&line_of(b)))
        .then_with(|| a.code.cmp(&b.code))
        .then_with(|| a.message.cmp(&b.message))
}

fn path_of(f: &Finding) -> &str {
    f.location.as_ref().map(|l| l.path.as_str()).unwrap_or("")
}

fn line_of(f: &Finding) -> u32 {
    // missing line goes last, so use a big sentinel
    f.location.as_ref().and_then(|l| l.line).unwrap_or(u32::MAX)
}
