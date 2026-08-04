# lintdiff

[![Coverage](https://codecov.io/gh/effortless-metrics/lintdiff/branch/main/graph/badge.svg)](https://codecov.io/gh/effortless-metrics/lintdiff)

`lintdiff` maps structured Rust compiler / Clippy diagnostics to **lines touched by a PR** and emits a **stable, schema-validated receipt** suitable for CI ingestion.

**Question answered:** _"Which diagnostics from the head analysis are located on PR-touched lines, and why?"_

See [PRODUCT.md](PRODUCT.md) for the supported product boundary, limitations, and
comparison with adjacent tools.

## Product Status

The supported product is the release-binary CLI and the GitHub Action. The
`v0.1.1` is released, and its exact-tag Action canary passed. Use the exact
`v0.1.1` Action tag for the maintained changed-line receipt product.

### Infrastructure Highlights

- **Deterministic receipt**: `lintdiff.report.v1` is the canonical output
  protocol.
- **Repository proof**: tests, benchmarks, fuzzing, coverage, and `xtask` checks
  support maintenance; their counts are not product readiness claims.
- **Narrow scope**: the current mode locates diagnostics on changed lines. It
  does not establish diagnostic newness relative to a base analysis.
- **Experimental evidence**: `lintdiff inventory` and `lintdiff compare` ship
  in `v0.1.1` as advisory research surfaces. Their `lintdiff.inventory.v1` and
  `lintdiff.delta.v1` artifacts are not a promise of generally reliable
  `new`/`resolved` detection or strict blocking.

## Design constraints (non-negotiable)

- **Build-truth consumer**: it consumes an existing diagnostics stream (usually `cargo clippy --message-format=json`).
- **Diff-scoped**: it maps diagnostics onto the PR diff (new-side line numbers).
- **Deterministic**: same inputs → byte-stable JSON + Markdown.
- **Protocol-shaped**: emits `artifacts/lintdiff/report.json` in a strict envelope.
- **Budgeted**: capped surfaced findings; full detail lives in artifacts.

## Quickstart

1. Produce a diagnostics stream:

```bash
cargo clippy --message-format=json > artifacts/clippy.jsonl
```

2. Produce a diff:

```bash
git diff --unified=0 "$BASE_SHA..$HEAD_SHA" > artifacts/patch.diff
```

3. Run lintdiff:

```bash
lintdiff ingest       --diagnostics artifacts/clippy.jsonl       --diff-file artifacts/patch.diff       --out artifacts/lintdiff/report.json       --md artifacts/lintdiff/comment.md       --annotations github
```

### GitHub Actions

The easiest way to use lintdiff is with our GitHub Action:

```yaml
name: Lintdiff
on: pull_request
jobs:
  lintdiff:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Required for git diff
      - run: cargo clippy --message-format=json > clippy.jsonl
      - uses: EffortlessMetrics/lintdiff@v0.1.1
        with:
          diagnostics: clippy.jsonl
          fail_on: warn  # Optional: error, warn, or never
```

See [action.yml](action.yml) for all available inputs and outputs.

The released binary also includes the experimental `inventory` and `compare`
commands. They require complete evidence supplied by the caller; lintdiff does
not build base and head revisions automatically. See
[docs/inventory.md](docs/inventory.md) and
[docs/diagnostic-delta.md](docs/diagnostic-delta.md) for their research
contracts and limitations.

## Repo docs

- `docs/architecture.md` – role, boundaries, IO contracts, failure modes
- `docs/requirements.md` – requirements and invariants
- `PRODUCT.md` – supported product contract and limitations
- `docs/release-process.md` – exact-tag release and support boundary
- `docs/design.md` – internal layered engine/application design
- `docs/implementation-plan.md` – phased plan + test strategy

## Workspace layout

The enforced workspace currently contains five members: four runtime packages and
the repository-only `xtask` control plane. `fuzz/` remains excluded. The normal
runtime graph is `lintdiff → {lintdiff-engine, lintdiff-render, lintdiff-types}`;
the engine and renderer depend on `lintdiff-types` only among lintdiff packages.

`lintdiff-types` is the only package with publication intent. The engine, renderer,
application, and xtask remain `publish = false`; no public engine crate is implied
by the internal package boundary. The dated disposition ledger at
[plans/microcrate-collapse-ledger.toml](plans/microcrate-collapse-ledger.toml) is
the migration record, and `cargo run -p xtask -- architecture-check` is the
enforcement surface.

## License

Dual-licensed under MIT or Apache-2.0.
