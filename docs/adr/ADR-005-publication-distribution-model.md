# ADR-005: Select publication distribution model before manifest hardening

## Status

Accepted

## Context

Recovery of the post-#88 stack introduced a four-class boundary model and confirmed that packaging evidence is required before applying `publish = false`.

`cargo package` on `main` initially failed because package checks on `lintdiff` and `lintdiff-ingest-core` require versioned dependency declarations for workspace path dependencies:

- `crates/lintdiff-cli/Cargo.toml`: `lintdiff-app` missing `version`
- `crates/lintdiff-ingest-core/Cargo.toml`: `lintdiff-diagnostics` missing `version`

The current model therefore separates release channels from registry distribution and constrains the embedded API closure.

## Decision

Adopt a two-product distribution model with an explicit two-crate registry closure:

- The CLI and GitHub Action are release-only products:
  - GitHub Action distribution (`EffortlessMetrics/lintdiff@<tag>`)
  - GitHub release archives (`Linux x86_64`, `macOS Intel`, `macOS ARM`, `Windows x86_64`)
- `cargo install lintdiff` is explicitly **not** a supported channel under this model.
- `lintdiff` is not publishable.
- The embedded Rust API closure is constrained to:
  - `lintdiff-types` (`publish = true`, registry support)
  - `lintdiff-ingest-core` (`publish = true`, supported library)
- `lintdiff-report-schema` remains workspace-internal until a direct public contract requires publication.

## Consequences

1. `publish = false` is now the default intent for non-API workspace crates, once Gate D1 is proven.
2. Gate D was split into publish-order-aware sub-gates:
   - D1a: package graph contains only approved registry roots
   - D1b: `.crate` archives are structurally complete
   - D1c: temporary consumer compile proof in extracted archives
   - D1d: leaf closure visibility in crates.io index
   - D1e: root passes `cargo publish --dry-run` against the real registry
3. Any change to support `cargo install lintdiff` requires a new ADR and re-running Gate D in this expanded shape.
