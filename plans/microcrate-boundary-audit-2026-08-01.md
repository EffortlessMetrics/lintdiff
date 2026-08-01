# Microcrate Boundary Audit (2026-08-01)

## Status

- **Owner context:** architecture seam recovery lane
- **Evidence date:** 2026-08-01
- **Purpose:** distinguish public seam intent from registry packaging support

## Snapshot inputs

- `Cargo.toml` workspace members: **69**
- `cargo metadata --format-version 1 --no-deps`: **69 packages**
- Non-dev runtime-reachable from `lintdiff`: **13 crates**
- Non-runtime workspace crates: **55 crates**

## Working boundary classes (non-final)

### Supported consumer surfaces

These are the seams the project is currently documenting as consumer-facing:

- `lintdiff` (`crates/lintdiff-cli`)
- `lintdiff-ingest-core`

### Registry support crates

These crates are candidates for publication to satisfy Rust package closure, but are not yet committed as direct public API contracts:

- `lintdiff-types` (pending explicit contract decision)
- `lintdiff-report-schema` (pending explicit contract decision)

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

1. Record final Gate D decision (`distribution model`) and whether it requires `lintdiff` or only binary/action distribution.
2. For candidates in **registry support crates**, prove whether they must remain publishable.
3. Preserve `lintdiff-types` and `lintdiff-report-schema` as unresolved until compatibility and contract outcomes are signed off.
