# Phase 4 Microcrate Expansion Plan

## Summary

This document analyzes the lintdiff codebase to identify additional SRP (Single Responsibility Principle) microcrate extraction opportunities for Phase 4, building on the 26 microcrates already extracted in Phases 1-3.

**Current State:**
- Phase 1: 9 microcrates (exit, stats, render-markdown, render-annotations, explain, truncate, config, escape, locale-detect)
- Phase 2: 8 microcrates (line-range, glob, severity, code-url, verdict, ci-env, path-norm, sort)
- Phase 3: 9 microcrates (span, location, counts, disposition, report-schema, message-norm, render-utils, config-types, feature-flags)
- **Total: 45 crates (26 extracted + 19 original)**

**Phase 4 Proposals:** 8 new microcrates identified across 3 priority tiers.

---

## High Priority

### 1. `lintdiff-jsonl`

- **Purpose**: JSON Lines parsing utilities for streaming JSON content
- **Source**: 
  - [`lintdiff-diagnostics/src/lib.rs:221-330`](crates/lintdiff-diagnostics/src/lib.rs:221) (line-by-line JSON parsing loop)
  - Similar patterns could be used for other JSONL formats
- **API**:
  ```rust
  /// Parse a JSON Lines stream into a vector of values.
  pub fn parse_jsonl<R: BufRead, T: DeserializeOwned>(
      reader: R,
  ) -> Result<Vec<T>, JsonlError> { ... }
  
  /// Stream JSON Lines with a callback.
  pub fn stream_jsonl<R: BufRead, T: DeserializeOwned, F>(
      reader: R,
      mut callback: F,
  ) -> Result<(), JsonlError>
  where
      F: FnMut(Result<T, JsonlError>) -> bool { ... }
  
  /// Filter JSON Lines by a predicate on raw lines.
  pub fn filter_jsonl<R: BufRead>(
      reader: R,
      predicate: impl Fn(&Value) -> bool,
  ) -> Result<Vec<Value>, JsonlError> { ... }
  
  /// Error type for JSONL parsing.
  #[derive(Debug, Error)]
  pub enum JsonlError {
      #[error("invalid JSON at line {line}: {source}")]
      InvalidJson { line: usize, source: serde_json::Error },
      #[error("IO error at line {line}: {source}")]
      IoError { line: usize, source: io::Error },
  }
  ```
- **Rationale**: JSON Lines parsing is a reusable pattern used in diagnostics parsing and could be used for:
  - Other tool output parsing (eslint, etc.)
  - Log file processing
  - Generic NDJSON handling
  - Enables independent testing and optimization of JSONL parsing

---

### 2. `lintdiff-hunk-header`

- **Purpose**: Unified diff hunk header parsing and manipulation
- **Source**: [`lintdiff-diff/src/lib.rs:256-291`](crates/lintdiff-diff/src/lib.rs:256) (`parse_hunk_header`)
- **API**:
  ```rust
  /// A parsed hunk header.
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct HunkHeader {
      /// Old file start line.
      pub old_start: u32,
      /// Old file line count (1 if omitted).
      pub old_count: Option<u32>,
      /// New file start line.
      pub new_start: u32,
      /// New file line count (1 if omitted).
      pub new_count: Option<u32>,
      /// Optional section header text.
      pub section: Option<String>,
  }
  
  impl HunkHeader {
      /// Parse from `@@` header line.
      pub fn parse(line: &str) -> Result<Self, HunkHeaderError> { ... }
      
      /// Render back to `@@` format.
      pub fn to_string(&self) -> String { ... }
      
      /// Get the old line range.
      pub fn old_range(&self) -> LineRange { ... }
      
      /// Get the new line range.
      pub fn new_range(&self) -> LineRange { ... }
  }
  
  /// Error type for hunk header parsing.
  #[derive(Debug, Error)]
  pub enum HunkHeaderError {
      #[error("not a hunk header")]
      NotAHunkHeader,
      #[error("missing minus segment")]
      MissingMinusSegment,
      #[error("missing plus segment")]
      MissingPlusSegment,
      #[error("invalid range: {0}")]
      InvalidRange(String),
  }
  ```
- **Rationale**: Hunk header parsing is a self-contained concern that:
  - Has clear boundaries and testable API
  - Could be reused by other diff-processing tools
  - Has property-testable invariants (parse/render roundtrip)
  - Currently embedded in larger diff parsing logic

---

### 3. `lintdiff-code-norm`

- **Purpose**: Diagnostic code normalization and URL generation
- **Source**: [`lintdiff-policy/src/code.rs:23-70`](crates/lintdiff-policy/src/code.rs:23)
- **API**:
  ```rust
  /// Normalize a diagnostic code to a stable identifier.
  pub fn normalize_diagnostic_code(raw: Option<&str>) -> (String, Option<String>) { ... }
  
  /// Check if a code is a rustc error code (e.g., E0425).
  pub fn is_rustc_error_code(code: &str) -> bool { ... }
  
  /// Convert a string to a URL-safe slug.
  pub fn slugify(s: &str) -> String { ... }
  
  /// Generate a documentation URL for a diagnostic code.
  pub fn doc_url(code: &str) -> Option<String> { ... }
  
  /// Code namespace for normalization.
  #[derive(Clone, Copy, Debug)]
  pub enum CodeNamespace {
      Rustc,
      Clippy,
      RustcLint,
      Unknown,
  }
  
  /// Determine the namespace for a diagnostic code.
  pub fn classify_code(code: &str) -> CodeNamespace { ... }
  ```
- **Rationale**: Code normalization is a focused concern that:
  - Is used across policy, fingerprint, and display code
  - Has clear extension points for new linters/tools
  - Can be tested independently with property tests
  - Could be extended to support other ecosystems (JS, Python, etc.)

---

## Medium Priority

### 4. `lintdiff-diff-paths`

- **Purpose**: Diff path extraction and manipulation utilities
- **Source**: [`lintdiff-diff/src/lib.rs:236-254`](crates/lintdiff-diff/src/lib.rs:236) (`parse_diff_git_paths`, `extract_diff_path`)
- **API**:
  ```rust
  /// Extract old and new paths from a `diff --git` line.
  pub fn parse_diff_git_paths(line: &str) -> Option<(String, String)> { ... }
  
  /// Strip a/ or b/ prefix from diff paths.
  pub fn extract_diff_path(prefixed: &str) -> &str { ... }
  
  /// Parse `--- ` or `+++ ` header lines.
  pub fn parse_diff_header_line(line: &str) -> Option<DiffPath> { ... }
  
  /// A path extracted from diff headers.
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub enum DiffPath {
      /// Regular file path.
      Path(String),
      /// /dev/null (file added or deleted).
      DevNull,
  }
  
  /// Detect rename from `rename from/to` lines.
  pub fn parse_rename_line(line: &str) -> Option<(RenameDirection, String)> { ... }
  
  #[derive(Clone, Copy, Debug)]
  pub enum RenameDirection { From, To }
  ```
- **Rationale**: Path extraction from diffs is:
  - Self-contained with clear API
  - Reusable for other diff-processing tools
  - Has edge cases (renames, /dev/null) worth centralizing
  - Currently scattered across diff parsing logic

---

### 5. `lintdiff-line-merge`

- **Purpose**: Line number to range merging utilities
- **Source**: [`lintdiff-diff/src/lib.rs:293-325`](crates/lintdiff-diff/src/lib.rs:293) (`merge_lines_to_ranges`)
- **API**:
  ```rust
  /// Merge a sorted list of line numbers into contiguous ranges.
  pub fn merge_lines_to_ranges(lines: Vec<u32>) -> Vec<LineRange> { ... }
  
  /// Merge overlapping or adjacent ranges.
  pub fn coalesce_ranges(ranges: Vec<LineRange>) -> Vec<LineRange> { ... }
  
  /// Expand ranges back to individual line numbers.
  pub fn expand_ranges(ranges: &[LineRange]) -> Vec<u32> { ... }
  
  /// Check if ranges cover a specific line.
  pub fn ranges_contain_line(ranges: &[LineRange], line: u32) -> bool { ... }
  
  /// Compute the union of two range sets.
  pub fn union_ranges(a: &[LineRange], b: &[LineRange]) -> Vec<LineRange> { ... }
  
  /// Compute the intersection of two range sets.
  pub fn intersect_ranges(a: &[LineRange], b: &[LineRange]) -> Vec<LineRange> { ... }
  ```
- **Rationale**: Line range merging is a general utility that:
  - Is used in diff parsing and span matching
  - Has property-testable invariants (idempotency, symmetry)
  - Could be useful for other line-based tools
  - Currently embedded in diff parsing

---

### 6. `lintdiff-timestamp`

- **Purpose**: RFC 3339 timestamp formatting utilities
- **Source**: [`lintdiff-app-io/src/lib.rs:126-130`](crates/lintdiff-app-io/src/lib.rs:126) (`now_rfc3339`)
- **API**:
  ```rust
  /// Get the current UTC time as an RFC 3339 string.
  pub fn now_rfc3339() -> String { ... }
  
  /// Format a timestamp as RFC 3339.
  pub fn format_rfc3339(dt: OffsetDateTime) -> String { ... }
  
  /// Parse an RFC 3339 timestamp.
  pub fn parse_rfc3339(s: &str) -> Result<OffsetDateTime, TimestampError> { ... }
  
  /// Get current timestamp with millisecond precision.
  pub fn now_millis() -> u64 { ... }
  
  /// Error type for timestamp parsing.
  #[derive(Debug, Error)]
  pub enum TimestampError {
      #[error("invalid RFC 3339 format: {0}")]
      InvalidFormat(String),
  }
  ```
- **Rationale**: Timestamp handling is:
  - Used across app-io, run info, and report generation
  - Currently duplicated in multiple places
  - Has consistent format requirements (RFC 3339)
  - Small but focused API

---

## Low Priority

### 7. `lintdiff-severity-map`

- **Purpose**: Severity level mapping between different tools
- **Source**: [`lintdiff-policy/src/code.rs:4-21`](crates/lintdiff-policy/src/code.rs:4) (`map_level_to_severity`, `format_level`)
- **API**:
  ```rust
  /// Map a diagnostic level to a severity.
  pub fn map_level_to_severity(level: &DiagnosticLevel) -> Severity { ... }
  
  /// Format a diagnostic level as a string.
  pub fn format_level(level: &DiagnosticLevel) -> String { ... }
  
  /// Parse a severity from a string.
  pub fn parse_severity(s: &str) -> Option<Severity> { ... }
  
  /// Severity mapping configuration.
  #[derive(Clone, Debug)]
  pub struct SeverityMapping {
      /// Map warning to error.
      pub warn_as_error: bool,
      /// Map info to warning.
      pub info_as_warn: bool,
  }
  
  impl SeverityMapping {
      /// Apply mapping to a severity.
      pub fn apply(&self, severity: Severity) -> Severity { ... }
  }
  ```
- **Rationale**: Severity mapping is:
  - A small but focused concern
  - Could be extended for other tool ecosystems
  - Currently embedded in policy code
  - May grow as more tools are supported

---

### 8. `lintdiff-explain-builder`

- **Purpose**: Explain summary and disposition tracking utilities
- **Source**: [`lintdiff-ingest-core/src/lib.rs:383-399`](crates/lintdiff-ingest-core/src/lib.rs:383) (`build_explain_summary`)
- **API**:
  ```rust
  /// Build an explain summary from dispositions.
  pub fn build_explain_summary(explain: &[DiagnosticDisposition]) -> ExplainSummary { ... }
  
  /// Track dispositions during ingest.
  pub struct ExplainTracker {
      entries: Vec<DiagnosticDisposition>,
  }
  
  impl ExplainTracker {
      /// Create a new tracker.
      pub fn new() -> Self { ... }
      
      /// Record a diagnostic disposition.
      pub fn record(&mut self, entry: DiagnosticDisposition) { ... }
      
      /// Build the final summary.
      pub fn summarize(&self) -> ExplainSummary { ... }
      
      /// Get all entries.
      pub fn entries(&self) -> &[DiagnosticDisposition] { ... }
  }
  ```
- **Rationale**: Explain tracking is:
  - A focused concern for debugging/diagnostics
  - Currently embedded in ingest-core
  - Could be extended for richer explain output
  - Low priority but clean extraction opportunity

---

## Implementation Order

1. **lintdiff-jsonl** - High value, enables cleaner diagnostics parsing
2. **lintdiff-hunk-header** - Self-contained, well-tested, reusable
3. **lintdiff-code-norm** - High value, centralizes code handling
4. **lintdiff-diff-paths** - Medium value, cleans up diff parsing
5. **lintdiff-line-merge** - Medium value, general utility
6. **lintdiff-timestamp** - Low complexity, quick win
7. **lintdiff-severity-map** - Low complexity, small API
8. **lintdiff-explain-builder** - Low priority, clean extraction

---

## Migration Strategy

### Phase 4.1: High Priority Crates
1. Create `lintdiff-jsonl` with streaming JSONL API
2. Migrate `lintdiff-diagnostics` to use `lintdiff-jsonl`
3. Create `lintdiff-hunk-header` with property tests
4. Migrate `lintdiff-diff` to use `lintdiff-hunk-header`
5. Create `lintdiff-code-norm` with extension points
6. Migrate `lintdiff-policy` to use `lintdiff-code-norm`

### Phase 4.2: Medium Priority Crates
1. Create `lintdiff-diff-paths` with path extraction API
2. Create `lintdiff-line-merge` with range utilities
3. Create `lintdiff-timestamp` with RFC 3339 utilities
4. Update consumers to use new crates

### Phase 4.3: Low Priority Crates
1. Create `lintdiff-severity-map` with mapping utilities
2. Create `lintdiff-explain-builder` with tracking utilities
3. Update consumers to use new crates

---

## Analysis Notes

### Crates Analyzed But Not Recommended for Extraction

| Crate | Reason |
|-------|--------|
| `lintdiff-diagnostics` | Well-focused after Span extraction; JSONL extraction handles main concern |
| `lintdiff-diff` | Well-focused after hunk/path extractions |
| `lintdiff-app-io` | Small and focused; timestamp extraction handles main concern |
| `lintdiff-match` | Well-decomposed with modules; no clear extraction candidates |
| `lintdiff-ingest-core` | Core orchestration; should delegate to extracted crates |
| `lintdiff-types` | Already reduced by Phase 3 extractions |
| `lintdiff-finding` | Standalone crate with focused API |
| `lintdiff-bdd-harness` | Test infrastructure; not a library concern |

### Cross-Cutting Patterns Identified

1. **Streaming/Line-based Parsing**: JSONL pattern reusable across tools
2. **Range Operations**: Line merging, intersection, union utilities
3. **Code Normalization**: Extensible to other ecosystems
4. **Path Handling**: Diff paths, normalization, relativization

### Future Considerations

- **Multi-tool support**: `lintdiff-code-norm` could be extended for eslint, pylint, etc.
- **Diff format variants**: `lintdiff-hunk-header` could support context diffs, Git format
- **Performance**: `lintdiff-jsonl` could add zero-copy parsing options
- **Schema validation**: `lintdiff-jsonl` could integrate with JSON Schema

---

## Summary

| Priority | Count | Crates |
|----------|-------|--------|
| High | 3 | jsonl, hunk-header, code-norm |
| Medium | 3 | diff-paths, line-merge, timestamp |
| Low | 2 | severity-map, explain-builder |
| **Total** | **8** | |

After Phase 4 completion, the crate count would be **53 crates** (45 + 8).
