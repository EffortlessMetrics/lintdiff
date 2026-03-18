//! Shared DTOs and small utilities.
//!
//! This crate is intentionally "boring": mostly plain data structures with `serde`
//! derives and a few deterministic helpers (path normalization, finding ordering).
//!
//! # Modules
//!
//! - [`config`] - Configuration types for lintdiff
//! - [`ordering`] - Deterministic finding ordering
//! - [`path`] - Path normalization and line ranges
//! - [`report`] - Report structures and types
//!
//! # Example: Path Normalization
//!
//! ```
//! use lintdiff_types::NormPath;
//!
//! let path = NormPath::new("src\\lib.rs");
//! assert_eq!(path.as_str(), "src/lib.rs");
//!
//! let path = NormPath::new("a/src/lib.rs");
//! assert_eq!(path.as_str(), "src/lib.rs");
//!
//! let path = NormPath::new("./src//lib.rs");
//! assert_eq!(path.as_str(), "src/lib.rs");
//! ```
//!
//! # Example: Finding Ordering
//!
//! ```
//! use lintdiff_types::{Finding, Severity, Location, NormPath, sort_findings};
//!
//! let error = Finding {
//!     severity: Severity::Error,
//!     code: "E001".to_string(),
//!     message: "error".to_string(),
//!     location: Some(Location {
//!         path: NormPath::new("src/a.rs"),
//!         line: Some(10),
//!         col: None,
//!     }),
//!     check_id: None,
//!     help: None,
//!     url: None,
//!     fingerprint: None,
//!     data: None,
//! };
//!
//! let warn = Finding {
//!     severity: Severity::Warn,
//!     code: "W001".to_string(),
//!     message: "warning".to_string(),
//!     location: Some(Location {
//!         path: NormPath::new("src/a.rs"),
//!         line: Some(10),
//!         col: None,
//!     }),
//!     check_id: None,
//!     help: None,
//!     url: None,
//!     fingerprint: None,
//!     data: None,
//! };
//!
//! let mut findings = vec![warn, error];
//! sort_findings(&mut findings);
//!
//! // Errors come before warnings
//! assert_eq!(findings[0].severity, Severity::Error);
//! assert_eq!(findings[1].severity, Severity::Warn);
//! ```
//!
//! # Example: Configuration
//!
//! ```
//! use lintdiff_types::{LintdiffConfig, Profile, FailOn};
//!
//! let config = LintdiffConfig::default();
//! let effective = config.effective();
//!
//! // Default profile
//! assert_eq!(effective.profile, Profile::Default);
//!
//! // Default fail_on for Default profile
//! assert_eq!(effective.fail_on, FailOn::Error);
//!
//! // Strict profile changes fail_on
//! let strict_config = LintdiffConfig {
//!     profile: Some(Profile::Strict),
//!     ..Default::default()
//! };
//! assert_eq!(strict_config.effective().fail_on, FailOn::Warn);
//! ```
//!
//! # Example: Report Structure
//!
//! ```
//! use lintdiff_types::{Report, ToolInfo, RunInfo, Verdict, VerdictStatus, Counts, SCHEMA_ID, TOOL_NAME};
//!
//! let report = Report {
//!     schema: SCHEMA_ID.to_string(),
//!     tool: ToolInfo {
//!         name: TOOL_NAME.to_string(),
//!         version: "1.0.0".to_string(),
//!         commit: None,
//!     },
//!     run: RunInfo {
//!         started_at: "2024-01-01T00:00:00Z".to_string(),
//!         ended_at: "2024-01-01T00:00:01Z".to_string(),
//!         duration_ms: Some(1000),
//!         host: None,
//!         git: None,
//!     },
//!     verdict: Verdict {
//!         status: VerdictStatus::Pass,
//!         counts: Counts::default(),
//!         reasons: vec![],
//!     },
//!     findings: vec![],
//!     data: None,
//! };
//!
//! let json = serde_json::to_string(&report).unwrap();
//! assert!(json.contains("lintdiff.report.v1"));
//! ```
//!
//! # Example: Line Range
//!
//! ```
//! use lintdiff_types::LineRange;
//!
//! let range = LineRange::new(5, 10);
//!
//! assert!(range.contains_line(5));
//! assert!(range.contains_line(7));
//! assert!(range.contains_line(10));
//! assert!(!range.contains_line(4));
//! assert!(!range.contains_line(11));
//!
//! let other = LineRange::new(8, 15);
//! assert!(range.intersects(&other));
//!
//! let separate = LineRange::new(11, 20);
//! assert!(!range.intersects(&separate));
//! ```

mod config;
mod ordering;
mod path;
mod report;

pub use config::*;
pub use ordering::*;
pub use path::*;
pub use report::*;
