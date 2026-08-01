# Microcrate Boundary Audit (2026-08-01)

## Status

- **Owner context:** architecture seam recovery lane
- **Evidence date:** 2026-08-01
- **Purpose:** distinguish public seam intent from registry packaging support

## Snapshot inputs

- This document records historical snapshot inputs from pre-final migration planning.
- Runtime and non-runtime counts in the working snapshot are preserved for archived comparison only.

## Working boundary classes (non-final)

### Supported consumer surfaces

These are the seams the project is currently documenting as consumer-facing:

- `lintdiff` (`crates/lintdiff-cli`)
- `lintdiff-ingest-core`

### Registry support crates (finalized)

Direct public registry roots after closure migration:

- `lintdiff-types` (publishable)
- `lintdiff-ingest-core` (publishable)
- `lintdiff-report-schema` (workspace-internal)

### Workspace-internal crates

All workspace crates not in “supported consumer surfaces” or “registry support” that remain necessary for implementation or packaging:

- Runtime adapters and internal utility/type modules (`lintdiff-app*`, parsing/matching/policy/render helper crates, and feature/utility crates used by current product flow).

### Test/tooling crates

- `lintdiff-bdd`
- `lintdiff-bdd-harness`
- `lintdiff-bdd-grid`
- `lintdiff-bench`

## Notes on evidence interpretation

- Zero indegree is a directional metric, not a standalone API-support proof.
- A crate can be required for package closure without being a direct consumer-facing promise.
- The 21-crate historical view is not representative of current workspace breadth.

## Open outcomes before PR D

1. Publish `lintdiff-types` and `lintdiff-ingest-core` in order.
2. Re-run Gate D in final mode once registry visibility is available.
3. Move the publication contract phase to `final` after the ordered publish receipts are complete.
