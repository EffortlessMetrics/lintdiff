# lintdiff

[![Coverage](https://codecov.io/gh/effortless-metrics/lintdiff/branch/main/graph/badge.svg)](https://codecov.io/gh/effortless-metrics/lintdiff)

`lintdiff` filters Rust compiler / Clippy diagnostics down to **only the lines touched by a PR** and emits a **stable, schema-validated receipt** suitable for cockpit-style ingestion.

**Question answered:** _"Did this change introduce actionable diagnostics on changed lines?"_

## ⚠️ Deprecation Notice

> **Important:** Three façade crates have been deprecated and will be removed in a future version:
>
> - `lintdiff-domain` → use [`lintdiff-core`](crates/lintdiff-core) directly
> - `lintdiff-core` → use [`lintdiff-ingest-core`](crates/lintdiff-ingest-core) for ingest operations
> - `lintdiff-ingest` → use [`lintdiff-ingest-core`](crates/lintdiff-ingest-core)
>
> 📖 See the [**Migration Guide**](docs/migration-guide.md) for detailed instructions on updating your dependencies.

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
      - uses: effortless-metrics/lintdiff@v1
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

- `lintdiff-types` – DTOs, config model, schema ids, normalization helpers
- `lintdiff-diff` – unified diff parsing → changed ranges (new-side)
- `lintdiff-diagnostics` – cargo JSON parsing → normalized diagnostics
- `lintdiff-match` – path/span matching primitives (filter compilation, span selection)
- `lintdiff-policy` – code normalization, allow/suppress/deny, verdict, fingerprinting
- `lintdiff-ingest-core` – core ingest pipeline (diagnostics + diff → report)
- ~~`lintdiff-ingest`~~ – **DEPRECATED**: compatibility facade over `lintdiff-ingest-core`
- ~~`lintdiff-core`~~ – **DEPRECATED**: pure domain engine (use `lintdiff-ingest-core` instead)
- ~~`lintdiff-domain`~~ – **DEPRECATED**: compatibility facade over `lintdiff-core`
- `lintdiff-render` – Markdown + GitHub annotations renderers
- `lintdiff-app` – orchestration (delegates to `app-git`, `app-io`)
- `lintdiff-app-git` – git adapter (diff acquisition, repo root, git info)
- `lintdiff-app-io` – I/O adapter (config loading, diagnostics reading, artifact writing)
- `lintdiff-feature-flags` – typed feature-flag registry and parsing
- `lintdiff-cli` – CLI surface (`lintdiff` binary)
- `lintdiff-bdd-grid` – BDD matrix helpers (feature-flag rows)
- `lintdiff-bdd-harness` – fixture loading, ingest helpers, feature-flag matrix runners
- `lintdiff-bdd` – fixture and scenario helpers used by tests/BDD

## License

Dual-licensed under MIT or Apache-2.0.
