pub mod diagnostics;
pub mod diff;
mod fingerprint;
mod ingest;
mod matching;
mod policy;

pub use diagnostics::{Diagnostic, Span};
pub use diff::DiffMap;

pub use ingest::{ingest_on_diff, IngestOnDiffParams};
