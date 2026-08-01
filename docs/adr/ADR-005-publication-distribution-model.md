# ADR-005: Select publication distribution model before manifest hardening

## Status

Accepted

## Context

Recovery of the post-#88 stack introduced a four-class boundary model but also showed that packaging evidence is required before applying `publish = false`.

Running package checks on `main` currently fails because `cargo package` on `lintdiff` and `lintdiff-ingest-core` requires versioned dependency declarations for workspace path dependencies:

- `crates/lintdiff-cli/Cargo.toml`: `lintdiff-app` missing `version`
- `crates/lintdiff-ingest-core/Cargo.toml`: `lintdiff-diagnostics` missing `version`

For Gate D0 and until Gate D1 completion, this is the accepted distribution model.

Without an adopted and evidence-verified distribution model, it is unsafe to claim that non-surface crates can be made private-only.

## Decision

For Gate D0, adopt **Action + release-binary distribution only** as the initial model:

- Registry packaging obligation is centered on embedded API distribution (`lintdiff-ingest-core`) rather than CLI-install distribution.
- The `lintdiff` CLI remains an intended supported consumer surface via:
  - GitHub Action distribution (`EffortlessMetrics/lintdiff@<tag>`)
  - release binaries
- `cargo install lintdiff` is explicitly **not** treated as a supported registry distribution channel under this model.
- `lintdiff-types` and `lintdiff-report-schema` remain unresolved until Gate D1 evidence shows whether they are required for the final public registry closure.

## Consequences

- `publish = false` edits for workspace-internal crates are only valid after Gate D1 shows packaging succeeds for the selected root(s).
- PR D (`chore(architecture): codify verified crate publication boundaries`) remains blocked until Gate D1 evidence is recorded for this model.
- Any future shift to CLI-registry support (`cargo install lintdiff`) requires a new architecture decision and re-run of Gate D1.
