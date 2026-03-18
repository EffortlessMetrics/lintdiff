//! Matching and filter helpers extracted from domain.
//!
//! This crate provides path/span matching primitives including:
//! - **Filter compilation**: Compile include/exclude glob patterns for path filtering
//! - **Path relativization**: Convert absolute paths to repo-relative paths
//! - **Span selection**: Select primary spans from diagnostic span lists
//!
//! # Examples
//!
//! ## Filter Compilation and Matching
//!
//! ```
//! use lintdiff_match::{compile_filters, path_allowed, Filters};
//! use lintdiff_types::LintdiffConfig;
//!
//! // Create filters from configuration
//! let mut config = LintdiffConfig::default();
//! config.filter.include_paths = vec!["src/**/*.rs".to_string()];
//! config.filter.exclude_paths = vec!["**/generated/**".to_string()];
//!
//! let filters = compile_filters(&config.effective());
//!
//! // Check if paths are allowed
//! assert!(path_allowed(&filters, "src/lib.rs"));
//! assert!(!path_allowed(&filters, "src/generated/api.rs"));
//! assert!(!path_allowed(&filters, "tests/integration.rs"));
//! ```
//!
//! ## Path Relativization
//!
//! ```
//! use lintdiff_match::relativize_span_path;
//! use lintdiff_types::NormPath;
//!
//! // Convert absolute paths to repo-relative
//! let result = relativize_span_path(
//!     &NormPath::new("/home/user/project/src/lib.rs"),
//!     Some(&NormPath::new("/home/user/project")),
//!     true,  // workspace_only: filter out paths outside repo
//! );
//! assert_eq!(result.unwrap().as_str(), "src/lib.rs");
//!
//! // Relative paths pass through unchanged
//! let result = relativize_span_path(
//!     &NormPath::new("src/lib.rs"),
//!     None,
//!     true,
//! );
//! assert_eq!(result.unwrap().as_str(), "src/lib.rs");
//!
//! // Windows paths are normalized to forward slashes
//! let result = relativize_span_path(
//!     &NormPath::new("src\\lib.rs"),
//!     None,
//!     true,
//! );
//! assert_eq!(result.unwrap().as_str(), "src/lib.rs");
//! ```
//!
//! ## Span Selection
//!
//! ```
//! use lintdiff_match::select_spans;
//! use lintdiff_diagnostics::Span;
//! use lintdiff_types::NormPath;
//!
//! // Create spans with primary flag
//! let spans = vec![
//!     Span {
//!         file: NormPath::new("src/lib.rs"),
//!         line_start: 10,
//!         line_end: 15,
//!         col_start: None,
//!         col_end: None,
//!         is_primary: true,
//!     },
//!     Span {
//!         file: NormPath::new("src/lib.rs"),
//!         line_start: 5,
//!         line_end: 8,
//!         col_start: None,
//!         col_end: None,
//!         is_primary: false,  // Context span
//!     },
//! ];
//!
//! // Select only primary spans
//! let selected = select_spans(&spans);
//! assert_eq!(selected.len(), 1);
//! assert_eq!(selected[0].line_start, 10);
//! assert!(selected[0].is_primary);
//! ```
//!
//! ## Empty Filters Allow All Paths
//!
//! ```
//! use lintdiff_match::{compile_filters, path_allowed};
//! use lintdiff_types::LintdiffConfig;
//!
//! let config = LintdiffConfig::default();
//! let filters = compile_filters(&config.effective());
//!
//! // With no include/exclude patterns, all paths are allowed
//! assert!(path_allowed(&filters, "src/lib.rs"));
//! assert!(path_allowed(&filters, "any/path/file.txt"));
//! ```
//!
//! ## Exclude Takes Precedence Over Include
//!
//! ```
//! use lintdiff_match::{compile_filters, path_allowed};
//! use lintdiff_types::LintdiffConfig;
//!
//! let mut config = LintdiffConfig::default();
//! config.filter.include_paths = vec!["src/**".to_string()];
//! config.filter.exclude_paths = vec!["src/lib.rs".to_string()];
//!
//! let filters = compile_filters(&config.effective());
//!
//! // Included but then excluded
//! assert!(!path_allowed(&filters, "src/lib.rs"));
//!
//! // Included and not excluded
//! assert!(path_allowed(&filters, "src/main.rs"));
//! ```
//!
//! ## Workspace-only Mode for Path Relativization
//!
//! ```
//! use lintdiff_match::relativize_span_path;
//! use lintdiff_types::NormPath;
//!
//! // With workspace_only=true, paths outside repo return None
//! let result = relativize_span_path(
//!     &NormPath::new("/other/path/file.rs"),
//!     Some(&NormPath::new("/repo")),
//!     true,
//! );
//! assert!(result.is_none());
//!
//! // With workspace_only=false, paths outside repo are kept
//! let result = relativize_span_path(
//!     &NormPath::new("/other/path/file.rs"),
//!     Some(&NormPath::new("/repo")),
//!     false,
//! );
//! assert!(result.is_some());
//! ```

mod filters;
mod paths;
mod spans;

pub use filters::{compile_filters, path_allowed, Filters};
pub use paths::relativize_span_path;
pub use spans::select_spans;
