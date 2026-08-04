# ADR-005: Select publication distribution model before manifest hardening

## Status

Superseded by the completed collapse decision in [ADR-006](ADR-006-canonical-package-topology.md).

> The package names and Gate D wording below are historical recovery context. The
> current publication intent is enforced in `contracts/package-topology.toml`:
> the four runtime packages are the intended registry closure, while `xtask`
> remains private. The publication amendment is recorded in ADR-006; this ADR
> does not claim that the `0.1.2` closure has already been published.

## Context

Recovery of the post-#88 stack introduced a four-class boundary model but also showed that packaging evidence is required before applying `publish = false`.

Running package checks on `main` currently fails because `cargo package` on `lintdiff` and `lintdiff-ingest-core` requires versioned dependency declarations for workspace path dependencies:

- `crates/lintdiff-cli/Cargo.toml`: `lintdiff-app` missing `version`
- `crates/lintdiff-ingest-core/Cargo.toml`: `lintdiff-diagnostics` missing `version`

Without an adopted distribution model, it is unsafe to claim that non-surface crates can be made private-only.

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
