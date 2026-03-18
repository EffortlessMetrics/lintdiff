# lintdiff architecture

> **⚠️ Deprecation Notice (v0.2.0)**
>
> The following façade crates have been **deprecated** and will be removed in a future version:
> - `lintdiff-domain` — use `lintdiff-ingest-core` instead
> - `lintdiff-core` — use `lintdiff-ingest-core` instead
> - `lintdiff-ingest` — use `lintdiff-ingest-core` instead
>
> These crates were intermediate façades that simply re-exported items from `lintdiff-ingest-core`.
> See the [Migration Guide](migration-guide.md) for instructions on updating your dependencies.

## Role in the cockpit ecosystem

lintdiff is a **build-truth consumer** that answers:

> “Did this PR introduce new compiler/Clippy diagnostics on changed lines?”

It sits in the “confidence dials” tier: valuable, but only cost-effective if your CI already produces diagnostics (`cargo clippy` / `cargo check` in JSON).

## Truth-layer boundary

lintdiff is intentionally narrow:

- **Consumes** a diagnostics stream (usually cargo JSON).
- **Consumes** a diff (base/head or patch file).
- **Maps** diagnostics onto the diff’s new-side changed lines.
- **Emits** a receipt (`artifacts/lintdiff/report.json`) + optional renderings.

lintdiff does **not**:

- run a linter engine of its own (beyond optional convenience “run” mode)
- scan the whole repo (diff-only by default)
- invent policy (blocking vs informational is a cockpit config decision)
- attempt to “explain the compiler” (it reports diagnostics; it doesn’t become rustc)

## Contracts (API surface)

lintdiff treats these as stable contracts:

- Testing contracts:
  - `lintdiff-bdd-grid` provides reusable scenario-grid primitives for feature-flag experiments.
  - `lintdiff-bdd` provides reusable fixtures + scenario helpers for adapters and tests.

- Canonical artifacts:
  - `artifacts/lintdiff/report.json` (**required**)
  - `artifacts/lintdiff/comment.md` (**optional**)
  - `artifacts/lintdiff/annotations.txt` (**optional**, or emitted to stdout)
- Receipt schema: `schemas/lintdiff.report.v1.json`
- Finding identity and determinism:
  - stable code mapping
  - stable ordering key for findings and rendered output
- Exit codes:
  - `0` ok (pass or warn, unless warn-as-fail is enabled by config/profile)
  - `2` policy failure (blocking findings)
  - `1` tool/runtime error (I/O, parse failure, invalid config, etc.)

## Director integration assumptions

lintdiff assumes a receipts-first director:

- Director ingests `artifacts/lintdiff/report.json`.
- Director applies composition policy (blocking/missing/warn-as-fail/budgets).
- Director renders the final cockpit surface, budgeted and deterministic.

lintdiff’s own `comment.md` is designed to be useful standalone, but is not required for the director.

## Failure-mode posture

lintdiff is strict about not producing false confidence:

- Missing diagnostics input → **skip** (explicit reason), not “pass”.
- Missing base/head and no patch file → tool error (exit `1`), with clear remediation.
- Unparseable diagnostics/diff → tool error (exit `1`).
- If matching is impossible due to path normalization mismatch → **warn** with an explicit reason and guidance.

## Design guardrails

- **One extension point**: tool-specific payload lives under `report.data` and `finding.data`.
- Top-level report is strict (`additionalProperties=false` in schema).
- Determinism is contractual:
  - stable ordering
  - stable truncation semantics
  - no dependence on filesystem iteration order

## Crate Architecture

lintdiff follows a modular crate architecture with clear separation of concerns:

### Recommended Crate (Public API)

For most use cases, you should depend on **`lintdiff-ingest-core`**:

```toml
[dependencies]
lintdiff-ingest-core = "0.2"
```

This crate provides the complete public API for:
- Ingest pipeline for processing diagnostics and diffs
- Policy evaluation and verdict computation
- Finding types and report generation
- Fingerprinting for stable finding identity

### Crate Overview

| Crate | Purpose | Status |
|-------|---------|--------|
| `lintdiff-ingest-core` | **Recommended**: Complete public API for ingestion and processing | ✅ Active |
| `lintdiff-domain` | ~~Façade for domain types~~ | ⚠️ Deprecated |
| `lintdiff-core` | ~~Façade for core logic~~ | ⚠️ Deprecated |
| `lintdiff-ingest` | ~~Façade for ingestion~~ | ⚠️ Deprecated |
| `lintdiff-types` | Configuration and report types | Internal use |
| `lintdiff-diagnostics` | Diagnostics parsing | Internal use |
| `lintdiff-diff` | Diff parsing | Internal use |
| `lintdiff-match` | Matching logic (spans, paths, filters) | Internal use |
| `lintdiff-fingerprint` | Finding fingerprint computation | Internal use |
| `lintdiff-render` | Output rendering (markdown, annotations) | Internal use |
| `lintdiff-app` | Application orchestration | Internal use |
| `lintdiff-cli` | Command-line interface | Binary only |

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     Public API Surface                           │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              lintdiff-ingest-core                        │    │
│  │   (IngestPipeline, Policy, Verdict, Finding, Report)    │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Internal Crates                             │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │lintdiff-types│ │lintdiff-diag │ │ lintdiff-diff│             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │lintdiff-match│ │lintdiff-fp   │ │lintdiff-render│            │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Application Layer                           │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │ lintdiff-app │ │lintdiff-app-io│ │lintdiff-app-git│          │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
└─────────────────────────────────────────────────────────────────┘
```

### Migration from Deprecated Crates

If you are currently using any of the deprecated façade crates, migrate to `lintdiff-ingest-core`:

```diff
- use lintdiff_domain::IngestPipeline;
+ use lintdiff_ingest_core::IngestPipeline;
```

See the [Migration Guide](migration-guide.md) for detailed instructions.