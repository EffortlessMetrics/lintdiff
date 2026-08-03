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

It does not yet answer whether a diagnostic is new relative to a base analysis.
That diagnostic-delta capability is experimental future scope.

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
fail closed. The `v0.1.1` example is gated on the exact tag and its post-release
Action canary; no moving default is a trustworthy release contract.

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
- The planned `v0.1.1` example is supported only after the exact-tag release and
  post-release Action canary gates pass.
- The current release-asset matrix is Linux x86_64, macOS x86_64, macOS
  arm64, and Windows x86_64. Other host/platform combinations are unsupported.
- There is no supported `cargo install lintdiff` workflow.
- `lintdiff-engine` is an internal implementation seam, not a supported public
  crates.io contract.
- `lintdiff-types` contains public evidence protocols, but any future breaking
  protocol narrowing is a separate compatibility decision and release action.
- No moving `v0` Action alias is supported or implied.

## Known limitations and non-goals

- The current mode is changed-line location filtering, not causal analysis.
- No baseline diagnostic inventory or delta receipt is implemented yet.
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
| Diagnostic delta | Future base/head diagnostic comparison | Experimental scope; not implemented by the current location-scoped mode. |

The product remains intentionally narrow until an external evaluation shows
that diagnostic delta changes decisions or earns continued adoption.
