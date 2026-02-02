# lintdiff implementation plan

This plan is structured to de-risk correctness early: fixtures first, contracts first, and then performance/UX.

## Phase 0 — Contracts, schemas, and scaffolding (P0)

Deliverables:

- Workspace structure and crate boundaries (microcrates)
- JSON schemas:
  - `schemas/receipt.envelope.v1.json`
  - `schemas/lintdiff.report.v1.json`
- Canonical artifacts contract enforced by tests:
  - `artifacts/lintdiff/report.json` always written on success
- Baseline CLI:
  - `lintdiff ingest` (wired, even if logic is stubbed)
  - `lintdiff md`, `lintdiff annotations` render-only
  - `lintdiff explain` (minimal registry)

Tests:

- schema validation test (fixture report validates against schema)
- deterministic ordering test for `Finding` sorting utility

Exit criteria:

- `cargo test` passes on a fresh clone
- fixtures prove schema + determinism

## Phase 1 — Core diff parsing (P0)

Deliverables:

- `lintdiff-diff` parses unified diff into `DiffMap`:
  - per-file new-side changed line ranges
  - best-effort rename awareness
  - canonical path normalization
- Stable range merge and intersection implementation

Tests:

- golden fixtures:
  - simple addition
  - replacement
  - multi-hunk file
  - rename
  - deletion-only hunk (no matchable lines)
- proptest:
  - range merge is idempotent
  - intersection correctness (random spans vs merged ranges)

Exit criteria:

- no panics on malformed diff (returns structured error)
- stable `DiffMap` output and ordering

## Phase 2 — Diagnostics parsing (P0)

Deliverables:

- `lintdiff-diagnostics` consumes cargo JSON stream:
  - extracts compiler messages
  - captures code, level, message, spans
  - ignores non-diagnostic cargo messages
- robust to large streams (streaming reader)

Tests:

- fixture JSONL from cargo/clippy (trimmed) parses successfully
- failure fixture (invalid JSON) produces tool error

Exit criteria:

- parser produces predictable normalized diagnostics
- errors are clear and actionable

## Phase 3 — Matching + policy + report generation (P0)

Deliverables:

- `lintdiff-domain` matches diagnostics to diff ranges
- `fail_on` implemented (error|warn|never)
- allow/suppress/deny code lists
- receipts include:
  - counts
  - provenance summary
  - truncation markers

Tests:

- golden fixtures: (diff, diagnostics) → exact `report.json`
- BDD scenarios (Given/When/Then):
  - warning on changed line becomes finding
  - warning outside diff is ignored
  - primary-span selection behavior
  - path normalization edge cases
- property tests:
  - stable fingerprinting for stable inputs

Exit criteria:

- minimal viable lintdiff: ingest → receipt → md → annotations
- deterministic outputs confirmed by repeated runs

## Phase 4 — Renderers + UX polish (P1)

Deliverables:

- `lintdiff-render`:
  - Markdown renderer (budgeted)
  - GitHub annotations renderer (budgeted)
- CLI quality:
  - consistent flags (`--base/--head/--diff-file`, `--diagnostics`, `--out`, `--md`, `--annotations`)
  - helpful error messages and remediation
- Optional `run` mode:
  - executes a cargo command, captures JSON diagnostics, then ingests

Tests:

- golden markdown fixtures
- annotations output fixture (top N, stable formatting)

Exit criteria:

- usable locally and in CI
- outputs are short, stable, and link to artifacts

## Phase 5 — Hardening (P1)

Deliverables:

- fuzz targets (cargo-fuzz):
  - diff parser fuzz (never panic)
  - diagnostics parser fuzz (never panic)
- mutation testing (cargo-mutants) configuration and docs
- more fixtures from real-world cargo output (macro spans, generated files)

Exit criteria:

- fuzzing runs in CI on schedule (timeboxed)
- mutation tests are practical (e.g., weekly or on demand)

## Phase 6 — Release + adoption surface (P2)

Deliverables:

- prebuilt binaries (Linux/macOS/Windows)
- CI workflow snippet in README
- version pinning strategy documented (bundle-friendly)

Exit criteria:

- people can adopt with one paste
- deterministic outputs and stable codes are treated as API

