# lintdiff design

lintdiff is implemented as a hexagonal (ports/adapters) tool with a microcrate workspace. The goal is to keep the core deterministic and testable, and push I/O variance to the edges.

## Architectural principles

- **Domain first**: matching and verdict logic are pure functions over inputs.
- **Adapters are thin**: git, filesystem, and clocks live outside the domain.
- **Schemas are contracts**: DTOs are versioned and validated in CI.
- **Determinism is enforced**: stable ordering and stable truncation.
- **Small extension points**: `report.data` and `finding.data` only.

## Microcrate layout

- `lintdiff-types`
  - Receipt DTOs and config model (serde)
  - Schema id constants
  - Path normalization helpers
  - Stable finding ordering key
- `lintdiff-diff`
  - Unified diff parsing into `DiffMap` (new-side changed lines)
  - Rename handling (best effort)
  - Property tests for range merging and intersection correctness
- `lintdiff-diagnostics`
  - Cargo JSON parsing into `Diagnostic` + spans
  - Tolerant parsing (ignores non-diagnostic messages)
  - Clear failure on invalid JSON (tool error)
- `lintdiff-match`
  - Path/span matching primitives (filter compilation, span selection, path relativization)
- `lintdiff-policy`
  - Code normalization, allow/suppress/deny, verdict computation, fingerprinting
- `lintdiff-ingest-core`
  - Core ingest pipeline (diagnostics + diff → report)
- `lintdiff-ingest`
  - Compatibility facade over `lintdiff-ingest-core`
- `lintdiff-bdd-grid`
  - BDD matrix representation and feature-flag cell parsing
  - Deterministic assignment serialization for fixture-driven scenario combinatorics
- `lintdiff-core`
  - Domain engine for matching diagnostics to changed lines
  - Policy mapping (`fail_on`, allow/suppress/deny)
  - Receipt generation (verdict + findings + tool-specific data)
- `lintdiff-domain`
  - Compatibility façade over `lintdiff-core` (legacy crate name)
- `lintdiff-render`
  - Markdown renderer (budgeted)
  - GitHub annotations renderer (budgeted)
- `lintdiff-bdd`
  - Fixture loading and deterministic BDD ingest harness
  - Stable feature flag assignment helpers used by scenario grids
- `lintdiff-bdd-harness`
  - Fixture loading, ingest helpers, feature-flag matrix runners
- `lintdiff-app`
  - Orchestration layer, delegates to `lintdiff-app-git` and `lintdiff-app-io`
  - Converts I/O failures into tool-error receipts when possible
- `lintdiff-app-git`
  - Git adapter (diff acquisition, repo root, git info)
- `lintdiff-app-io`
  - I/O adapter (config loading, diagnostics reading, artifact writing)
- `lintdiff-feature-flags`
  - Typed feature-flag registry and parsing
- `lintdiff-cli`
  - Clap CLI, subcommands, exit code mapping

## Ports and adapters (hexagonal boundary)

The current API still favors pure-function domain usage. Adapter boundaries
now have dedicated microcrates (`lintdiff-app-git`, `lintdiff-app-io`).

Concrete adapters are orchestrated by `lintdiff-app`.

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
