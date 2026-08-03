//! Policy and matching helpers extracted from domain.
//!
//! This crate provides utilities for:
//! - **Code normalization**: Converting diagnostic codes to stable identifiers
//! - **Fingerprinting**: Creating deterministic hashes for findings
//! - **Verdict computation**: Determining pass/warn/fail status from findings
//!
//! # Example: Normalizing diagnostic codes
//!
//! ```
//! use lintdiff_engine::normalize_diagnostic_code;
//!
//! // Clippy lints get special handling
//! let (code, url) = normalize_diagnostic_code(Some("clippy::needless_borrow"));
//! assert_eq!(code, "lintdiff.diagnostic.clippy.needless_borrow");
//! assert!(url.is_some());
//!
//! // Rustc error codes also get URLs
//! let (code, url) = normalize_diagnostic_code(Some("E0502"));
//! assert_eq!(code, "lintdiff.diagnostic.rustc.E0502");
//! assert!(url.is_some());
//!
//! // Unknown codes are still normalized
//! let (code, url) = normalize_diagnostic_code(Some("custom_lint"));
//! assert!(code.starts_with("lintdiff.diagnostic."));
//! assert!(url.is_none());
//!
//! // None becomes unknown
//! let (code, url) = normalize_diagnostic_code(None);
//! assert_eq!(code, "lintdiff.diagnostic.unknown");
//! assert!(url.is_none());
//! ```
//!
//! # Example: Fingerprinting findings
//!
//! ```
//! use lintdiff_engine::fingerprint;
//! use lintdiff_types::{Location, NormPath};
//!
//! let loc = Location {
//!     path: NormPath::new("src/lib.rs"),
//!     line: Some(42),
//!     col: None,
//! };
//!
//! // Same inputs always produce the same fingerprint
//! let fp1 = fingerprint("test.code", Some(&loc), "message");
//! let fp2 = fingerprint("test.code", Some(&loc), "message");
//! assert_eq!(fp1, fp2);
//!
//! // Different inputs produce different fingerprints
//! let fp3 = fingerprint("other.code", Some(&loc), "message");
//! assert_ne!(fp1, fp3);
//! ```
//!
//! # Example: Computing verdicts
//!
//! ```
//! use lintdiff_engine::{compute_verdict, counts_from_findings};
//! use lintdiff_types::{Finding, Location, NormPath, Severity, LintdiffConfig, FailOn, VerdictStatus};
//!
//! // Create findings (normally you'd get these from the ingest pipeline)
//! let finding = Finding {
//!     severity: Severity::Error,
//!     check_id: Some("test.check".to_string()),
//!     code: "test.code".to_string(),
//!     message: "Something went wrong".to_string(),
//!     location: Some(Location {
//!         path: NormPath::new("src/lib.rs"),
//!         line: Some(1),
//!         col: None,
//!     }),
//!     help: None,
//!     url: None,
//!     fingerprint: None,
//!     data: None,
//! };
//!
//! // Configure fail-on behavior
//! let mut config = LintdiffConfig::default();
//! config.fail_on = Some(FailOn::Error);
//! let effective = config.effective();
//!
//! // Compute verdict
//! let verdict = compute_verdict(&effective, &[finding.clone()], 0, 0);
//!
//! // With an error and fail_on=Error, status should be Fail
//! assert_eq!(verdict.status, VerdictStatus::Fail);
//!
//! // Get counts from findings
//! let counts = counts_from_findings(&[finding]);
//! assert_eq!(counts.error, 1);
//! assert_eq!(counts.warn, 0);
//! assert_eq!(counts.info, 0);
//! ```

mod code;
mod verdict;

pub use code::{format_level, is_code_allowed, map_level_to_severity, normalize_diagnostic_code};
pub use verdict::{compute_verdict, counts_from_findings};

use lintdiff_types::delta::{
    DeltaKind, DeltaReceipt, DeltaVerdict, DeltaVerdictStatus, PairingEvidence,
};

/// Apply the experimental delta policy after evidence and classifications exist.
pub fn evaluate_delta_policy(receipt: &DeltaReceipt) -> DeltaVerdict {
    if receipt.provenance.comparability.status
        == lintdiff_types::delta::ComparabilityStatus::Incomparable
    {
        return DeltaVerdict {
            status: DeltaVerdictStatus::Incomparable,
            reasons: receipt
                .provenance
                .comparability
                .reasons
                .iter()
                .map(|reason| format!("incomparable_{}", delta_reason_token(*reason)))
                .collect(),
        };
    }

    let mut reasons = Vec::new();
    for item in &receipt.items {
        match (&item.pairing, item.change_kind) {
            (PairingEvidence::HeadOnly { head }, Some(DeltaKind::New))
                if receipt.policy.block_new_errors && head.level == "error" =>
            {
                reasons.push("new_error".to_string());
            }
            (PairingEvidence::HeadOnly { head }, Some(DeltaKind::New))
                if receipt.policy.block_new_warnings && head.level == "warning" =>
            {
                reasons.push("new_warning".to_string());
            }
            (PairingEvidence::Ambiguous { .. }, None) if receipt.policy.block_ambiguous => {
                reasons.push("ambiguous_pairing".to_string());
            }
            _ => {}
        }
    }
    reasons.sort();
    reasons.dedup();
    DeltaVerdict {
        status: if reasons.is_empty() {
            DeltaVerdictStatus::Accepted
        } else {
            DeltaVerdictStatus::Rejected
        },
        reasons,
    }
}

fn delta_reason_token(reason: lintdiff_types::delta::DeltaReason) -> &'static str {
    match reason {
        lintdiff_types::delta::DeltaReason::BaseAnalysisIncomplete => "base_analysis_incomplete",
        lintdiff_types::delta::DeltaReason::HeadAnalysisIncomplete => "head_analysis_incomplete",
        lintdiff_types::delta::DeltaReason::HardScopeMismatch => "hard_scope_mismatch",
        lintdiff_types::delta::DeltaReason::MultipleCandidates => "multiple_candidates",
        lintdiff_types::delta::DeltaReason::NoEarnedCorrespondence => "no_earned_correspondence",
    }
}
