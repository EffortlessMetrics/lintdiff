# lintdiff design

`lintdiff` is implemented as a hexagonal (ports/adapters) tool with a small domain core.

## Architectural principles

- **Domain first**: matching, ordering, and verdict logic are pure functions over inputs.
- **Adapters are thin**: git, filesystem, and environment access live outside the domain.
- **Schemas are contracts**: DTOs are explicit and validated in CI.
- **Determinism is enforced**: stable ordering and stable truncation.
- **Small extension points**: `report.data` and `finding.data` only.

## Ingest-core architecture

- `lintdiff-types`
  - Shared protocol, config DTOs, and receipt schema constants.
  - Path normalization helpers.
  - Stable finding ordering key.
- `lintdiff-ingest-core`
  - Unified diff parsing (`DiffMap`).
  - Diagnostics parsing (`Diagnostic`, `Span`).
  - Matching, policy mapping (`fail_on`, allow/suppress/deny), and verdict computation.
  - Receipt/report generation and fingerprinting.
  - Path filters and canonicalization.
  - `matching`, `policy`, and `fingerprint` are private modules unless an explicit consumer contract is established.

## Other crates in the workspace

- `lintdiff-app`, `lintdiff-app-git`, `lintdiff-app-io`
  - Orchestration and I/O adapters around the core.
- `lintdiff-render`, `lintdiff-cli`
  - Rendering and CLI surfaces.
- `lintdiff-bdd-harness`, `lintdiff-bdd-grid`, `lintdiff-bench`
  - Test/tooling suites.

## Ports and adapters

Adapter boundaries still use dedicated workspace crates (`lintdiff-app-git`, `lintdiff-app-io`).
The goal remains: run domain logic in tests with strings and avoid filesystem/git dependence.

## Data flow

1. Acquire diff (git base/head or `--diff-file`).
2. Parse diff to `DiffMap`:
   - `path → merged line ranges` for new-side changed lines.
3. Parse diagnostics stream to `Vec<Diagnostic>`.
4. Normalize diagnostic paths to repo-relative canonical form.
5. Match diagnostics by span/path against changed ranges.
6. Apply policy (`fail_on`, allow/suppress/deny).
7. Emit report with deterministic ordering and truncation.
8. Render optional outputs (Markdown, annotations).

## Legacy crate migration status

- `lintdiff-diagnostics`, `lintdiff-diff`, `lintdiff-match`, `lintdiff-policy`, `lintdiff-fingerprint`
  are retired from active workspace membership and have been internalized under `lintdiff-ingest-core`.
- Historical references to these crates should be treated as migration history only.

## Failure semantics

- Missing required inputs yields `skip` (not pass).
- Parse errors are tool/runtime errors (exit 1).
- When possible, lintdiff still writes a receipt on failure (`tool.runtime_error` findings).
