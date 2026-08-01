# lintdiff architecture

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

As of 2026-08-01 evidence snapshot:

- Workspace members: **69**
- Runtime non-dev/build reachability from `lintdiff`: **13 crates**
- Non-runtime workspace crates: **55**

The workspace is intentionally modeled with four classes:

### 1) Candidate supported consumer surfaces

Public-facing surfaces that the project currently documents as intended consumer seams:

- `lintdiff` (`crates/lintdiff-cli`)
  - CLI surface and GitHub Action behavior.
- `lintdiff-ingest-core`
  - Embedded ingestion core API.

### 2) Registry support crates

Crates that may be required to package/seal published products and must have a versioned package presence, but are not direct support promises:

- `lintdiff-types` — pending explicit contract decision.
- `lintdiff-report-schema` — pending explicit contract decision.

These remain unresolved until Gate D0/D1 publication-closure work confirms concrete required packages.

### 3) Workspace-internal crates

Runtime and utility crates used to satisfy internal implementation needs that are not currently documented as direct distribution contracts:

- `lintdiff-app`, `lintdiff-app-git`, `lintdiff-app-io`
- parsing, matching, policy, rendering, and feature/utility helper crates not listed in class 1 or 2
- other non-test workspace crates not in public surface classes

### 4) Test/tooling crates

- `lintdiff-bdd`, `lintdiff-bdd-harness`, `lintdiff-bdd-grid`
- `lintdiff-bench`

Important: zero workspace indegree is not treated as proof of external safety; this is a dependency-shape signal, not a direct API guarantee.

## Architecture evidence and change tracking

- The 21-crate historical review is retained in [`plans/microcrate-review.md`](plans/microcrate-review.md) for archival reference only.
- Current working evidence and follow-up decisions are tracked in:
  - [`plans/microcrate-boundary-audit-2026-08-01.md`](plans/microcrate-boundary-audit-2026-08-01.md)
  - [`plans/microcrate-simplification-follow-up-2026-08-01.md`](plans/microcrate-simplification-follow-up-2026-08-01.md)
- Counts are dated evidence snapshots and are not permanent, standalone policy; Gate D publication testing determines final class finalization.
