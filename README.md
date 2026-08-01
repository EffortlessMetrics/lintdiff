# lintdiff

[![Coverage](https://codecov.io/gh/effortless-metrics/lintdiff/branch/main/graph/badge.svg)](https://codecov.io/gh/effortless-metrics/lintdiff)

`lintdiff` filters Rust compiler / Clippy diagnostics down to **only the lines touched by a PR** and emits a **stable, schema-validated receipt** suitable for cockpit-style ingestion.

**Question answered:** _"Did this change introduce actionable diagnostics on changed lines?"_

## Project Status

| Metric | Value |
|--------|-------|
| **Development Status** | ✅ Production Ready |
| **All Phases Complete** | 10/10 (100%) |
| **Total Tests** | 1,207+ |
| **BDD Scenarios** | 200 |
| **CI/CD Workflows** | 8 |
| **Clippy Lint Level** | pedantic (zero warnings) |

### Infrastructure Highlights

- **Performance Benchmarks**: Criterion-based benchmarking suite for large repos
- **API Stability**: Automated semver checking with cargo-semver-checks
- **Code Coverage**: Codecov integration with comprehensive reporting
- **Fuzzing**: Advanced fuzzing infrastructure with structured corpus
- **i18n Ready**: Fluent-based internationalization infrastructure

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
      - uses: EffortlessMetrics/lintdiff@v0.1.0
        with:
          diagnostics: clippy.jsonl
          fail_on: warn  # Optional: error, warn, or never
```

See [action.yml](action.yml) for all available inputs and outputs.

## Repo docs

- `docs/architecture.md` – role, boundaries, IO contracts, failure modes
- `docs/requirements.md` – requirements and invariants
- `docs/design.md` – internal design (hexagonal boundaries + microcrates)
- `docs/implementation-plan.md` – phased plan + test strategy

## Workspace layout

- Distribution classes:
  - **CLI/product surface:** `lintdiff` (GitHub Action + GitHub release archives only; not publishable)
  - **Embedded API:** `lintdiff-ingest-core` (published)
  - **Protocol/support root:** `lintdiff-types` (published)
  - **Workspace-internal:** all other workspace crates plus test/tooling crates.
- `cargo install lintdiff` is intentionally not a supported install surface.
- Active workspace and boundary evidence is tracked in:
  - [`contracts/publication.toml`](contracts/publication.toml) (contract)
  - [`docs/architecture.md`](docs/architecture.md) (boundary rationale)
  - [`docs/adr/ADR-005-publication-distribution-model.md`](docs/adr/ADR-005-publication-distribution-model.md) (decision record)

## Migration status note

- `lintdiff-bdd` is retired from active workspace membership.
- Legacy microcrates (`lintdiff-diagnostics`, `lintdiff-diff`, `lintdiff-match`, `lintdiff-policy`, `lintdiff-fingerprint`) were internalized into `lintdiff-ingest-core` during this migration.
- Canonical evidence and follow-up lanes are tracked in:
  - [`plans/microcrate-boundary-audit-2026-08-01.md`](plans/microcrate-boundary-audit-2026-08-01.md)
  - [`plans/microcrate-simplification-follow-up-2026-08-01.md`](plans/microcrate-simplification-follow-up-2026-08-01.md)

## License

Dual-licensed under MIT or Apache-2.0.
