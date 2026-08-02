# Microcrate Expansion Plan for lintdiff

> Historical proposal. The collapse campaign superseded this expansion direction;
> use ADR-006 and the collapse ledger for current topology.

## Executive Summary

This document analyzes the current lintdiff crate architecture and proposes new microcrates following the Single Responsibility Principle (SRP). The goal is to **expand** the number of microcrates to enable parallel development, stable APIs, and future buildout.

## Current State Analysis

### Crate Inventory (19 crates)

| Crate | Lines | Responsibility | SRP Status |
|-------|-------|----------------|------------|
| `lintdiff-cli` | ~50 | Binary entry point | ✅ Compliant |
| `lintdiff-app` | 377 | Application orchestration | ⚠️ Mixed concerns |
| `lintdiff-app-git` | 146 | Git adapter | ✅ Compliant |
| `lintdiff-app-io` | 131 | I/O adapter | ✅ Compliant |
| `lintdiff-bdd` | 4 | Deprecated shim | ⚠️ Deprecated |
| `lintdiff-bdd-grid` | 202 | Feature flag matrix | ✅ Compliant |
| `lintdiff-bdd-harness` | 175 | BDD test harness | ✅ Compliant |
| `lintdiff-bench` | N/A | Benchmarking | ✅ Compliant |
| `lintdiff-diagnostics` | 353 | Diagnostics parsing | ✅ Compliant |
| `lintdiff-diff` | 377 | Diff parsing | ✅ Compliant |
| `lintdiff-feature-flags` | 299 | Feature flag registry | ✅ Compliant |
| `lintdiff-fingerprint` | 257 | Fingerprint computation | ✅ Compliant |
| `lintdiff-i18n` | 582 | Internationalization | ⚠️ Could split |
| `lintdiff-ingest-core` | 712 | Core ingest pipeline | ⚠️ Multiple concerns |
| `lintdiff-match` | ~200 | Matching logic | ✅ Compliant |
| `lintdiff-policy` | N/A | Policy/verdict logic | ✅ Compliant |
| `lintdiff-render` | 549 | Rendering | ⚠️ Multiple formats |
| `lintdiff-types` | ~300 | Shared types | ⚠️ Config mixed in |

### Key Findings

#### 1. `lintdiff-ingest-core` (712 lines) - Largest Crate

**Current Responsibilities:**
- Ingest pipeline orchestration
- Diagnostic-to-finding transformation
- Span matching coordination
- Explain summary generation
- Statistics collection
- Message truncation
- Report finalization

**SRP Violations:**
- Mixes pipeline orchestration with business logic
- Contains utility functions like [`truncate_message()`](crates/lintdiff-ingest-core/src/lib.rs:401) that should be separate
- Explain/disposition tracking is a distinct concern

#### 2. `lintdiff-render` (549 lines) - Multiple Output Formats

**Current Responsibilities:**
- Markdown rendering ([`render_markdown()`](crates/lintdiff-render/src/lib.rs:164))
- GitHub annotations rendering ([`render_github_annotations()`](crates/lintdiff-render/src/lib.rs:365))
- Output budgeting/truncation
- Format-specific escaping

**SRP Violations:**
- Two distinct output formats in one crate
- Shared utilities mixed with format-specific logic

#### 3. `lintdiff-types` - Configuration Mixed with Types

**Current Responsibilities:**
- Report DTOs
- Configuration types ([`config.rs`](crates/lintdiff-types/src/config.rs))
- Path normalization
- Finding ordering

**SRP Violations:**
- Configuration is a separate domain from shared types
- Could benefit from config-specific crate

#### 4. `lintdiff-app` - Orchestration with Business Logic

**Current Responsibilities:**
- CLI option handling
- Ingest orchestration
- Exit code classification
- Feature flag application

**SRP Violations:**
- Exit code logic embedded in orchestration
- Feature flag application mixed with app logic

---

## Proposed New Microcrates

### HIGH PRIORITY

#### 1. `lintdiff-render-markdown`

**Responsibility:** Markdown rendering for reports

**Stable Public API:**
```rust
pub struct MarkdownOptions {
    pub max_items: usize,
    pub report_path: String,
}

pub fn render_markdown(report: &Report, opts: MarkdownOptions) -> String;
```

**Dependencies:** `lintdiff-types`

**Dependents:** `lintdiff-render` (facade), `lintdiff-app`

**Priority:** HIGH

**Rationale:** 
- Markdown rendering is a stable, well-defined concern
- Enables parallel work on markdown improvements without touching other renderers
- Clear API boundary allows independent testing and fuzzing

---

#### 2. `lintdiff-render-annotations`

**Responsibility:** GitHub Actions annotation rendering

**Stable Public API:**
```rust
pub fn render_github_annotations(report: &Report, max: usize) -> String;

pub fn escape_github_command(s: &str) -> String;
```

**Dependencies:** `lintdiff-types`

**Dependents:** `lintdiff-render` (facade), `lintdiff-app`

**Priority:** HIGH

**Rationale:**
- GitHub annotations have specific format requirements
- May expand to support GitLab, Bitbucket in future
- Isolates CI-specific escaping logic

---

#### 3. `lintdiff-explain`

**Responsibility:** Diagnostic disposition tracking and explain summary generation

**Stable Public API:**
```rust
pub enum Disposition {
    Included,
    DroppedNoSpan,
    DroppedOutsideDiff,
    DroppedByPathFilter,
    SuppressedByCode,
    CutByBudget,
}

pub struct DiagnosticDisposition {
    pub code: String,
    pub message_preview: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub disposition: Disposition,
    pub fingerprint: Option<String>,
}

pub struct ExplainSummary {
    pub total: u32,
    pub included: u32,
    pub dropped_no_span: u32,
    pub dropped_outside_diff: u32,
    pub dropped_by_path_filter: u32,
    pub suppressed_by_code: u32,
    pub cut_by_budget: u32,
}

pub fn build_explain_summary(explain: &[DiagnosticDisposition]) -> ExplainSummary;
```

**Dependencies:** None (or `lintdiff-types` for shared types)

**Dependents:** `lintdiff-ingest-core`

**Priority:** HIGH

**Rationale:**
- Explain tracking is a distinct domain concern
- Used for debugging and transparency
- Stable API enables future enhancements like explain formatters

---

#### 4. `lintdiff-stats`

**Responsibility:** Statistics collection and aggregation for reports

**Stable Public API:**
```rust
pub struct IngestStats {
    pub diagnostics_total: usize,
    pub diagnostics_with_spans: u32,
    pub diagnostics_with_path_in_diff: u32,
    pub matched_findings_total: u32,
    pub matched_findings_emitted: usize,
    pub suppressed_by_code: u32,
    pub denied_by_code: u32,
    pub filtered_out_by_path: u32,
    pub diff_files: u32,
    pub diff_hunks: u32,
    pub diff_added_lines: u32,
}

pub fn compute_stats(
    diagnostics: &[Diagnostic],
    findings: &[Finding],
    diff_map: &DiffMap,
    // ... other inputs
) -> IngestStats;
```

**Dependencies:** `lintdiff-types`, `lintdiff-diagnostics`, `lintdiff-diff`

**Dependents:** `lintdiff-ingest-core`

**Priority:** HIGH

**Rationale:**
- Statistics collection is purely computational
- Enables future expansion like stats exporters (Prometheus, etc.)
- Clear separation makes testing easier

---

### MEDIUM PRIORITY

#### 5. `lintdiff-truncate`

**Responsibility:** Message and output truncation utilities

**Stable Public API:**
```rust
pub fn truncate_message(msg: &str, max_len: usize) -> String;
pub fn truncate_findings(findings: &mut Vec<Finding>, max: usize) -> TruncationResult;

pub struct TruncationResult {
    pub truncated: bool,
    pub original_count: usize,
    pub truncated_count: usize,
}
```

**Dependencies:** `lintdiff-types`

**Dependents:** `lintdiff-ingest-core`, `lintdiff-render-markdown`

**Priority:** MEDIUM

**Rationale:**
- Truncation logic is reused across multiple crates
- Deterministic truncation is a contract
- Centralizing ensures consistency

---

#### 6. `lintdiff-exit`

**Responsibility:** Exit code classification and reporting

**Stable Public API:**
```rust
pub enum ExitCode {
    Success = 0,
    ToolError = 1,
    PolicyFailure = 2,
}

pub fn classify_exit_code(report: &Report) -> i32;
pub fn exit_code_description(code: i32) -> &'static str;
```

**Dependencies:** `lintdiff-types`

**Dependents:** `lintdiff-app`, `lintdiff-cli`

**Priority:** MEDIUM

**Rationale:**
- Exit codes are part of the public contract
- Centralizes documentation and testing
- Enables future expansion for more granular codes

---

#### 7. `lintdiff-config`

**Responsibility:** Configuration types, loading, and validation

**Stable Public API:**
```rust
pub struct LintdiffConfig { /* ... */ }
pub struct EffectiveConfig { /* ... */ }
pub struct FilterConfig { /* ... */ }
pub struct FeatureFlags { /* ... */ }
pub enum FailOn { /* ... */ }
pub enum Profile { /* ... */ }

pub fn validate_config(cfg: &LintdiffConfig) -> Result<(), ConfigError>;
pub fn merge_configs(base: &LintdiffConfig, overlay: &LintdiffConfig) -> LintdiffConfig;
```

**Dependencies:** `serde`, `toml`

**Dependents:** `lintdiff-types`, `lintdiff-app-io`, `lintdiff-ingest-core`

**Priority:** MEDIUM

**Rationale:**
- Configuration is a distinct domain
- Enables config-specific validation and testing
- Future: config migration, schema evolution

---

### LOW PRIORITY (Future Consideration)

#### 8. `lintdiff-escape`

**Responsibility:** Output format escaping utilities

**Stable Public API:**
```rust
pub fn escape_markdown_table(s: &str) -> String;
pub fn escape_github_command(s: &str) -> String;
pub fn escape_html(s: &str) -> String;
```

**Dependencies:** None

**Dependents:** `lintdiff-render-markdown`, `lintdiff-render-annotations`

**Priority:** LOW

**Rationale:**
- Small concern, may not warrant separate crate initially
- Could be extracted if escaping becomes complex

---

#### 9. `lintdiff-locale-detect`

**Responsibility:** System locale detection

**Stable Public API:**
```rust
pub fn detect_system_locale() -> Option<String>;
pub fn locale_from_env() -> Option<String>;
```

**Dependencies:** None

**Dependents:** `lintdiff-i18n`

**Priority:** LOW

**Rationale:**
- Currently part of `lintdiff-i18n`
- Extract if locale detection becomes platform-complex

---

## Architecture After Expansion

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          Public API Surface                              │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    lintdiff-ingest-core                          │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
        ┌───────────────────────────┼───────────────────────────┐
        ▼                           ▼                           ▼
┌───────────────┐         ┌───────────────┐         ┌───────────────┐
│lintdiff-explain│         │lintdiff-stats │         │lintdiff-truncate│
└───────────────┘         └───────────────┘         └───────────────┘
        │                           │                           │
        └───────────────────────────┼───────────────────────────┘
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          Internal Crates                                 │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐    │
│  │lintdiff-types│ │lintdiff-diag │ │ lintdiff-diff│ │lintdiff-match│    │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘    │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐    │
│  │lintdiff-fp   │ │lintdiff-config││lintdiff-policy││lintdiff-exit │    │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          Rendering Crates                                │
│  ┌─────────────────────┐ ┌─────────────────────┐ ┌───────────────────┐  │
│  │lintdiff-render-md   │ │lintdiff-render-anno │ │ lintdiff-escape   │  │
│  └─────────────────────┘ └─────────────────────┘ └───────────────────┘  │
│                      └─────────────┬─────────────┘                       │
│                                    ▼                                     │
│                         ┌─────────────────────┐                          │
│                         │  lintdiff-render    │ (facade)                 │
│                         └─────────────────────┘                          │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          Application Layer                               │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                     │
│  │ lintdiff-app │ │lintdiff-app-io│ │lintdiff-app-git│                   │
│  └──────────────┘ └──────────────┘ └──────────────┘                     │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Priority Order

### Phase 1: High Impact, Clear Boundaries

1. **`lintdiff-render-markdown`** - Clear extraction from render
2. **`lintdiff-render-annotations`** - Clear extraction from render
3. **`lintdiff-explain`** - Clear domain boundary
4. **`lintdiff-stats`** - Pure computation, easy to test

### Phase 2: Medium Complexity

5. **`lintdiff-truncate`** - Utility extraction
6. **`lintdiff-exit`** - Small but important contract
7. **`lintdiff-config`** - Requires type migration

### Phase 3: Future Consideration

8. **`lintdiff-escape`** - If needed
9. **`lintdiff-locale-detect`** - If i18n expands

---

## API Stability Guarantees

### Stable API Principles

1. **All public traits are sealed** - Prevent external implementation
2. **All public types are `#[non_exhaustive]`** - Allow future fields
3. **Breaking changes require major version bump** - Semver strict
4. **Deprecation path** - Minimum one minor version warning

### API Stability Tiers

| Tier | Stability | Examples |
|------|-----------|----------|
| Tier 1 | Never breaking | `Report`, `Finding`, `Verdict` |
| Tier 2 | Additive only | `MarkdownOptions`, `ExplainSummary` |
| Tier 3 | May change | Internal helpers, experimental |

---

## Parallelization Enablement

### How New Crates Enable Parallel Development

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Developer Team Parallelization                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Developer A          Developer B          Developer C               │
│  ───────────          ───────────          ───────────               │
│  render-markdown      render-annotations    explain                  │
│       │                    │                   │                      │
│       ▼                    ▼                   ▼                      │
│  ┌─────────┐          ┌─────────┐         ┌─────────┐                │
│  │ Tests   │          │ Tests   │         │ Tests   │                │
│  │ Docs    │          │ Docs    │         │ Docs    │                │
│  │ Fuzzing │          │ Fuzzing │         │ Props   │                │
│  └─────────┘          └─────────┘         └─────────┘                │
│                                                                      │
│  No merge conflicts - each crate is independent!                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Parallel Work Streams After Extraction

| Stream | Crates | Can Work Independently |
|--------|--------|------------------------|
| Rendering | `render-markdown`, `render-annotations`, `escape` | ✅ Yes |
| Core Logic | `explain`, `stats`, `truncate` | ✅ Yes |
| Config | `config` | ✅ Yes |
| Exit Codes | `exit` | ✅ Yes |

---

## Migration Strategy

### Extraction Pattern

For each new crate:

1. **Create crate** with empty lib.rs
2. **Move types/functions** preserving visibility
3. **Add re-export** in original crate (deprecation warning)
4. **Update dependents** to use new crate
5. **Remove re-export** after transition period

### Example: `lintdiff-render-markdown`

```rust
// crates/lintdiff-render-markdown/src/lib.rs
pub struct MarkdownOptions { /* ... */ }
pub fn render_markdown(report: &Report, opts: MarkdownOptions) -> String { /* ... */ }

// crates/lintdiff-render/src/lib.rs (after extraction)
#[deprecated(since = "0.3.0", note = "Use lintdiff_render_markdown directly")]
pub use lintdiff_render_markdown::{render_markdown, MarkdownOptions};
```

---

## Success Metrics

### Before Extraction (Current)

- 19 crates
- Largest crate: 712 lines
- Rendering: 2 formats in 1 crate
- Config: Mixed with types

### After Phase 1 (Target)

- 23 crates (+4)
- Largest crate: ~500 lines
- Rendering: 1 format per crate
- Clear domain boundaries

### Long-term (Vision)

- 25+ crates
- All crates <400 lines
- All crates follow SRP
- Stable public APIs

---

## Conclusion

This expansion plan identifies **7 high/medium priority microcrates** and **2 low priority** candidates. The focus is on:

1. **Stable APIs** - Each crate has a well-defined public interface
2. **Parallel development** - Developers can work on separate crates without conflicts
3. **SRP compliance** - Each crate does one thing well
4. **Future-proofing** - Architecture supports expansion

The recommended implementation order prioritizes high-impact extractions with clear boundaries, enabling immediate parallelization benefits.
