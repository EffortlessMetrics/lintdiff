# lintdiff design

> Historical design note: the microcrate layout described below was superseded by
> [ADR-006](adr/ADR-006-canonical-package-topology.md). Current package ownership
> and dependency rules live in [architecture.md](architecture.md) and
> `contracts/package-topology.toml`.

lintdiff is implemented as a layered tool with a pure engine and a concrete
application shell. The goal is to keep analysis deterministic and testable, and
push Git, filesystem, process, environment, and terminal concerns to the edges.

## Architectural principles

- **Domain first**: matching and verdict logic are pure functions over inputs.
- **Application edges are thin**: git, filesystem, and clocks live outside the
  pure engine.
- **Schemas are contracts**: DTOs are versioned and validated in CI.
- **Determinism is enforced**: stable ordering and stable truncation.
- **Small extension points**: `report.data` and `finding.data` only.

## Current package layout

The workspace has four runtime packages and one repository-tooling package:

- `lintdiff-types`: public versioned protocol DTOs, wire primitives, and schema
  identifiers.
- `lintdiff-engine`: pure Cargo-diagnostic parsing, source mapping, matching,
  identity, policy, comparison, and receipt construction.
- `lintdiff-render`: pure Markdown, GitHub annotation, and RDJSONL projections.
- `lintdiff`: concrete application library and binary, including CLI dispatch,
  Git, filesystem, process, configuration, artifact, and exit handling.
- `xtask`: schema, fixture, topology, documentation, and release-contract
  checks for the repository.

The `fuzz/` workspace remains auxiliary and excluded from the product topology.
There is no shared test-support package unless a later consumer inventory
justifies one.

## Application boundary

The engine and render packages are pure over typed inputs. The `lintdiff`
application shell owns concrete adapters and orchestration; it passes acquired
data into the engine and sends typed receipts to the renderer. This preserves
the useful isolation without manufacturing adapter crates for concrete
functions that have no independent implementation or consumer.

The goal is: **you can run domain logic in tests with strings**, no git subprocess, no filesystem.

## Data flow

1. Acquire diff (git base/head or `--diff-file`).
2. Parse diff to `DiffMap`:
   - `path → merged line ranges` for new-side changed lines
3. Parse diagnostics stream to `Vec<Diagnostic>`.
4. Normalize diagnostic paths to repo-relative canonical form.
5. Match diagnostics:
   - select primary spans (or all)
   - check span line range intersects changed ranges
6. Apply policy:
   - allow/suppress/deny code lists
   - `fail_on`
   - profile severity mapping (optional)
7. Emit report:
   - stable finding ordering
   - stable truncation behavior
8. Render optional outputs (Markdown, annotations).

## Path normalization

This is the real footgun; treat it like protocol discipline.

Canonical path format everywhere:

- repo-relative
- forward slashes
- no leading `./`

Normalization handles:

- diff headers (`+++ b/<path>`)
- rustc spans (`file_name`, often absolute paths)
- Windows paths (`\` to `/`)
- optional stripping of repo root prefix

If `workspace_only=true` and a span cannot be mapped to repo-relative form, it is ignored.

## Diagnostic code mapping

lintdiff does not invent lints; it re-keys diagnostics into stable namespaced codes:

- rustc errors: `lintdiff.diagnostic.rustc.E0502`
- clippy lints: `lintdiff.diagnostic.clippy.needless_borrow`
- unknown: `lintdiff.diagnostic.other.<slug>`

The original raw code and level are preserved in `finding.data`.

## Deterministic ordering and fingerprinting

- Findings are sorted by the ordering key from `docs/requirements.md`.
- Fingerprint is SHA-256 over a stable tuple:
  - code + path + line + normalized(message)

Director can further dedupe across sensors using fingerprint.

## Rendering

Markdown output is compact:

- totals table (seen/matched/suppressed)
- status line (pass/warn/fail) and threshold policy
- top N findings (file:line, code, message)
- truncation marker if applicable
- repro line (when provided by config/app)

Annotations renderer emits GitHub workflow commands (`::warning` / `::error`) for top N findings with locations.

## Failure semantics

- Missing required inputs yields `skip` (not pass).
- Parse errors are tool/runtime errors (exit 1).
- When possible, lintdiff still writes a receipt on failures (verdict fail + `tool.runtime_error` finding).
