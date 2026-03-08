# lintdiff

`lintdiff` filters Rust compiler / Clippy diagnostics down to **only the lines touched by a PR** and emits a **stable, schema-validated receipt** suitable for cockpit-style ingestion.

**Question answered:** _"Did this change introduce actionable diagnostics on changed lines?"_

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
- `lintdiff-ingest` – compatibility facade over `lintdiff-ingest-core`
- `lintdiff-core` – pure domain engine (matching + policy + verdict + report generation)
- `lintdiff-domain` – compatibility facade over `lintdiff-core`
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
