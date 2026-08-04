# lintdiff product contract

This document describes the product that lintdiff currently earns. It is the
support boundary for the release-binary CLI and exact-tag GitHub Action; it is
not a promise that every workspace package is a supported public API.

## Target user and job

lintdiff is for Rust maintainers who already run `cargo check`, `cargo test`, or
Clippy and want a deterministic receipt for diagnostics located on lines touched
by a pull request.

The current product answers:

> Which diagnostics from the head analysis are located on PR-touched lines, and
> why were other diagnostics excluded, suppressed, or truncated?

The maintained changed-line workflow does not answer whether a diagnostic is
new relative to a base analysis.
The released binary also contains experimental `inventory` and `compare`
commands for research on that question. They are advisory surfaces, not an
earned claim that general `new` or `resolved` detection is reliable.

## Supported workflow

The CLI consumes structured Rust diagnostics, normally Cargo/Clippy JSONL, and
one of these diff inputs:

- explicit `--base` and `--head` references;
- a supplied unified diff through `--diff-file`.

It emits a canonical `artifacts/lintdiff/report.json`-style receipt and may
also emit Markdown or GitHub annotation projections. The `ingest`, `run`, and
`ci github` commands are the supported application workflows. The receipt is
the durable output; human projections are budgeted views of that receipt.

The Action accepts base/head overrides, a config path, `fail_on`, a diagnostics
JSONL path, a version input, and a working directory. Public examples use an
exact published Action tag:

```yaml
- uses: EffortlessMetrics/lintdiff@v0.1.1
  with:
    diagnostics: artifacts/clippy.jsonl
    fail_on: warn
```

The exact-tag installer contract makes ref resolution, checksums, and execution
fail closed. `v0.1.1` is released and its exact-tag Action canary passed; no
moving default is a trustworthy release contract.

`v0.1.1` also ships two experimental evidence commands:

- `lintdiff inventory` emits `lintdiff.inventory.v1` before diff filtering or
  policy;
- `lintdiff compare` emits `lintdiff.delta.v1` from caller-supplied base/head
  inventories and a source diff.

These commands do not build base and head revisions, and their comparison
results are not authorized for strict blocking or causal claims. The external
verdict and its limitations are recorded in the
[diagnostic-delta verdict memo](plans/diagnostic-delta-external-verdict-2026-08.md).

## Canonical receipt

`lintdiff.report.v1` is the canonical wire protocol. It preserves the report
schema, normalized findings, verdict, provenance, explain evidence, and artifact
paths needed by CI consumers. Markdown and annotations are projections; they
must not become a second evidence model.

The current receipt is location-scoped. It records diagnostics that intersect
the new-side lines in the supplied PR diff and explains relevant exclusions. It
does not compare equivalent base and head analyses, infer diagnostic newness, or
claim that a finding was caused by the change.

## Release and support boundary

- The supported distribution surfaces are the release-binary CLI and the
  GitHub Action at an exact release tag.
- `v0.1.1` is the current released exact-tag example; its post-release Action
  canary passed.
- The current release-asset matrix is Linux x86_64, macOS x86_64, macOS
  arm64, and Windows x86_64. Other host/platform combinations are unsupported.
- A coordinated `0.1.2` crates.io publication is being prepared for the four
  runtime packages. Its intended order is `lintdiff-types`, `lintdiff-engine`,
  `lintdiff-render`, then `lintdiff`.
- The `lintdiff` package is the primary registry product and will support
  `cargo install lintdiff` only after the exact publication and clean install
  proof complete. The current `v0.1.1` release does not make that claim yet.
- `lintdiff-engine` is being prepared as a public embeddable analysis surface;
  `lintdiff-render` is a public registry-support projection surface. Their
  `0.1.x` support boundary is established by the publication release notes and
  docs.rs output, not by the internal package boundary alone.
- `lintdiff-types` contains public evidence protocols, but any future breaking
  protocol narrowing is a separate compatibility decision and release action.
- No moving `v0` Action alias is supported or implied.
- `inventory` and `compare` are shipped experimental commands. Their schemas
  are research contracts, not a supported blocking policy or public engine API.

## Known limitations and non-goals

- The current mode is changed-line location filtering, not causal analysis.
- Inventory and delta receipts exist as experimental surfaces, but the external
  evaluation did not establish sufficiently conservative general `new` or
  `resolved` classifications.
- The tool relies on a structured upstream diagnostics stream; it does not run a
  base and head build or orchestrate those analyses.
- Human output is budgeted. The canonical receipt is the complete evidence
  surface available from the run.
- Internationalization readiness is not a product claim.
- SARIF alert lifecycle, hosted storage, dashboards, and GitHub Apps are not
  product surfaces.

## Adjacent tools

| Tool or mode | Primary job | lintdiff distinction |
| --- | --- | --- |
| reviewdog | Generic diff-filtered review presentation | lintdiff preserves a Rust-aware deterministic receipt and explain evidence. |
| SARIF / code scanning | GitHub alert lifecycle and code-scanning integration | lintdiff emits its own `lintdiff.report.v1`; SARIF is not the current product protocol. |
| Diagnostic delta | Base/head comparison from caller-supplied inventories | Shipped experimental scope; advisory only and not generally validated for strict blocking. |

The product remains intentionally narrow until an external evaluation shows
that diagnostic delta changes decisions or earns continued adoption.
