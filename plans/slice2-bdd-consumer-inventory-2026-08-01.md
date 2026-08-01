# Slice-2: lintdiff-bdd façade consolidation inventory

Date: 2026-08-01  
Lane: `recovery/slice2-bdd-consumer-migration`

## Decision boundary

The façade migration should be a consumer-migration step only:
- prove current consumers.
- move test consumers to the canonical harness crate.
- keep a future PR for optional façade retirement.

## Evidence collected

Runtime + manifest scan:
- `crates/lintdiff-cli/Cargo.toml`:
  - `[dependencies]` no `lintdiff-bdd`/`lintdiff-bdd-harness`.
  - `[dev-dependencies]` now uses `lintdiff-bdd-harness`.
- `rg -n "lintdiff_bdd\\b|lintdiff-bdd\\b" crates` (repository subset) returns:
  - `crates/lintdiff-cli/tests/bdd.rs` uses `lintdiff_bdd_harness::*` (post-change).
  - `crates/lintdiff-bdd-harness/Cargo.toml` depends on core test/runtime crates.
  - `crates/lintdiff-bdd/Cargo.toml` depends on `lintdiff-bdd-harness` and remains `publish = false`.

## Consumer classification

### Runtime consumers
- None currently reference `lintdiff-bdd`/`lintdiff-bdd-harness`.
- `crates/lintdiff-cli` runtime path remains unchanged.

### Test/tooling consumers
- `crates/lintdiff-cli/tests/bdd.rs` (BDD test target `bdd`) is the direct user of BDD helper APIs via `lintdiff-bdd-harness`.

## Migration action (this PR slice)

1. Replace `lintdiff-bdd` test dependency in `crates/lintdiff-cli/Cargo.toml` with `lintdiff-bdd-harness`.
2. Switch `crates/lintdiff-cli/tests/bdd.rs` import from `lintdiff_bdd` to `lintdiff_bdd_harness`.
3. Retirement milestone reached: removed `crates/lintdiff-bdd` shim from workspace and dependency graph.

## Follow-up (separate slice)

- Retirement was completed on `recovery/slice2-bdd-facade-retirement` by removing shim crate and workspace membership (this branch state now represents that candidate).

## Verification snapshot (2026-08-01)

### 1) Consumer inventory from source

Command: `rg -n "lintdiff_bdd|lintdiff-bdd" crates -g "*.rs" -g "Cargo.toml"`

Observed matches:
- `crates/lintdiff-cli/Cargo.toml:27` -> `lintdiff-bdd-harness` dev-dependency.
- `crates/lintdiff-cli/tests/bdd.rs:3` -> `use lintdiff_bdd_harness`.
- `crates/lintdiff-bdd-harness/Cargo.toml` and `crates/lintdiff-bdd/Cargo.toml` internal dependency declarations.
- No runtime crate path/import usage remains outside test-target scope.

### 2) Runtime impact check for CLI product

Command: `cargo tree -p lintdiff --edges normal,build`

Observed: `lintdiff-bdd` and `lintdiff-bdd-harness` are absent from normal/build dependency edges.

### 3) Consumer migration compile check

Command: `cargo test --package lintdiff --test bdd --no-run`

Observed: `bdd` test target compiled successfully after import change, confirming direct test-path migration resolves with the harness crate.

### 4) Optional shim retirement verification

Command: `rg -n "name = \"lintdiff-bdd\"|path = \"../lintdiff-bdd\"|lintdiff_bdd::" crates -g "*.rs" -g "Cargo.toml"`

Observed: no matches for shim crate declaration/import were found; remaining `lintdiff-bdd*` occurrences are in `lintdiff-bdd-harness` and `lintdiff-bdd-grid`.

## Notes

- This slice is intentionally scoped to proofable consumer movement only and does not change publication policy.
- No assumptions are made about whether the façade crate can be deleted until a follow-up explicit ownership decision is made.
