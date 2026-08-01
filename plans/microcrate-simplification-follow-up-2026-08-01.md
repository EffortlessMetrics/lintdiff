# Microcrate Simplification Follow-up Plan (2026-08-01)

## Scope

Historical boundary memo from recovery lane. The publication boundary is now implemented as a two-crate public API closure (`lintdiff-types` + `lintdiff-ingest-core`) with release-only CLI/action distribution, and this memo is retained for traceability only.

## Current evidence position

- Snapshot evidence from this historical lane is preserved for continuity.
- `lintdiff` runtime reachability and support-crate counts are preserved in archival command logs.

## Boundaries (working model)

1. **Candidate supported consumer surfaces**
  - `lintdiff`
  - `lintdiff-ingest-core`
2. **Registry support crates (pending)**
  - `lintdiff-types` (published)
  - `lintdiff-ingest-core` (published; supported API root)
  - `lintdiff-report-schema` (internal-only; no direct public contract)
3. **Workspace-internal and test/tooling**
  - all remaining workspace and tool-support crates until Gate D resolves publish obligations

## Follow-up slices

### Slice 1 — Publication closure proof (Gate D0/D1)

Use an ADR + evidence to decide final distribution model before additional `publish = false` expansion.

### Slice 2 — Facade consolidation (retired)

Façade consolidation was completed in a prior slice (`lintdiff-bdd` retirement).

### Slice 3 — Utility reduction

Fold or retain utility crates only after stable publication model is chosen and each PR can prove no external contract break.

## Open constraints

- Do not equate “no direct API support promise” with “must be `publish = false`.”
- Do not collapse all remaining internal crates in one PR.
- Do not assume counts are policy in isolation; keep all claims referenced to Gate D proof artifacts.
