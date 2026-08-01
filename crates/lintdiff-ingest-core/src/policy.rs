//! Policy helpers for ingest.
//!
//! Internal code-normalization, verdict, and fingerprint helpers used by ingest
//! evaluation. These modules intentionally remain private unless an explicit
//! external contract is defined.

mod code;
mod fingerprint;
mod verdict;

pub use code::{format_level, is_code_allowed, map_level_to_severity, normalize_diagnostic_code};
pub use fingerprint::fingerprint;
pub use verdict::{compute_verdict, counts_from_findings};
