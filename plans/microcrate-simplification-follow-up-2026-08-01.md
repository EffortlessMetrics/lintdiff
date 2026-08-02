# Microcrate Simplification Follow-up Plan (2026-08-01)

> Superseded by the completed collapse campaign. Retained for provenance; current
> topology and follow-up work are tracked by ADR-006 and the collapse ledger.

## Scope

This plan is post-PR-C cleanup. It is intentionally deferred until publication-closure evidence from Gate D0/D1 determines final package obligations.

## Current evidence position

- Workspace members: 69 (snapshot)
- Runtime reachability from `lintdiff`: 13
- Non-runtime support crates: 55

## Boundaries (working model)

1. **Candidate supported consumer surfaces**
   - `lintdiff`
   - `lintdiff-ingest-core`
2. **Registry support crates (pending)**
   - `lintdiff-types`
   - `lintdiff-report-schema`
3. **Workspace-internal and test/tooling**
   - all remaining workspace and tool-support crates until Gate D resolves publish obligations

## Follow-up slices

### Slice 1 — Publication closure proof (Gate D0/D1)

Use an ADR + evidence to decide the distribution model before any additional `publish = false` expansion.

### Slice 2 — Façade consolidation (`lintdiff-bdd` lane)

Concentrate on façade migration/deprecation once manifest boundaries are stable.

### Slice 3 — Utility reduction

Fold or retain utility crates only after stable publication model is chosen and each PR can prove no external contract break.

## Open constraints

- Do not equate “no direct API support promise” with “must be `publish = false`.”
- Do not collapse all remaining internal crates in one PR.
- Do not assume counts are policy in isolation; keep all claims referenced to Gate D proof artifacts.
