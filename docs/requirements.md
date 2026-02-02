# lintdiff requirements

This document describes lintdiff as an implementable, testable system: inputs, outputs, invariants, and the failure modes that must be made explicit.

## Purpose

lintdiff filters Rust compiler / Clippy diagnostics down to **only the lines touched by a PR** and emits a **stable receipt** for cockpit ingestion.

**Primary question:** “Did this change introduce actionable diagnostics on changed lines?”

## Scope

lintdiff must:

- consume diagnostics produced by existing tooling
  - `cargo clippy --message-format=json` (primary)
  - `cargo check --message-format=json` (supported; same message shape)
  - stdin / file (`.jsonl`)
- consume a diff
  - git-based (`--base` + `--head`) or
  - patch file (`--diff-file`)
- emit:
  - canonical receipt JSON (**required**)
  - optional Markdown section
  - optional CI annotations
- enforce strict determinism and stable contracts

lintdiff must not:

- become a linter framework
- become a “whole repo” scanner by default
- embed orchestration into the director role (the runner/workflow owns execution)

## Truth layer

- **Build truth origin**: diagnostics originate from build/analysis execution.
- **Diff truth mapping**: lintdiff maps that output onto the PR diff.

## Inputs

### Required inputs (for meaningful evaluation)

1. Diagnostics stream/file
   - JSON messages from cargo (`--message-format=json`).
2. Diff source
   - `--base` + `--head` OR `--diff-file`.

### Optional inputs

- `--root` repo root (defaults to git toplevel if available, else cwd)
- `--config lintdiff.toml` (local policy knobs; cockpit composition policy lives elsewhere)
- provenance fields (recorded into `report.data`)

## Outputs

### Canonical artifacts

- `artifacts/lintdiff/report.json` (MUST)
- `artifacts/lintdiff/comment.md` (SHOULD when `--md` is provided)
- `artifacts/lintdiff/annotations.txt` (OPTIONAL; may emit to stdout)

### Receipt contract

- Schema id: `lintdiff.report.v1`
- Schema file: `schemas/lintdiff.report.v1.json`
- Top-level is strict (`additionalProperties=false`).
- Tool-specific payload lives under `data` only.

### Finding identity

Each finding MUST include:

- `severity`: `info|warn|error`
- `code`: stable string (namespaced)
- `message`: human readable

SHOULD include when available:

- `location.path` (repo-relative, forward slashes)
- `location.line`, `location.col`
- `fingerprint` for dedupe

MAY include:

- `check_id` (“producer check identity”, usually `diagnostics.on_diff`)
- `data` (structured hints; opaque to director)

## Matching semantics

A diagnostic is “in diff” if any selected span intersects the diff’s **new-side changed line set** for the corresponding file.

Span selection:

1. primary spans (if any)
2. otherwise all spans
3. spans not mappable to repo-relative paths are ignored when `workspace_only=true`

Changed lines definition:

- Derived from unified diff hunks by tracking the new-side line counter.
- Only `+` lines (not headers) are considered changed.
- Pure deletions do not create matchable new-side lines.

## Policy knobs (local)

lintdiff supports a small local config (`lintdiff.toml`) for:

- `fail_on = error|warn|never`
- `max_findings`, `max_annotations`
- `workspace_only`
- path include/exclude filters
- allow/suppress/deny code lists

Cockpit-level decisions (blocking sensor, missing receipt, warn-as-fail) are composition policy and live in `cockpit.toml` (outside lintdiff).

## Budgets and truncation

- The receipt may contain many findings, but surfaced outputs are capped.
- When truncated:
  - outputs must include explicit `truncated=true` metadata in `report.data`
  - the Markdown must include a “truncated” marker and point to the receipt path.

## Exit codes

- `0` — ok (pass or warn unless warn-as-fail configured)
- `2` — policy failure (blocking findings)
- `1` — tool/runtime error

## Determinism and stability

Determinism is contractual:

- stable finding ordering in receipt and renderings
- stable truncation behavior (same inputs ⇒ same cut)
- no reliance on filesystem iteration order

Finding ordering key (normative):

1. severity desc (`error > warn > info`)
2. path lexicographic
3. line asc (missing last)
4. code lexicographic
5. message lexicographic

## Failure modes (normative behavior)

- Missing diagnostics input:
  - verdict: `skip`
  - reason: `missing_diagnostics`
  - exit: `0`
- Missing diff (no base/head and no patch file):
  - tool/runtime error
  - exit: `1`
- Invalid diagnostics JSON:
  - tool/runtime error
  - exit: `1`
- Invalid diff:
  - tool/runtime error
  - exit: `1`
- Path normalization mismatch (0 matches but diagnostics present):
  - verdict: `warn`
  - reason: `path_mismatch` (include guidance in findings)

