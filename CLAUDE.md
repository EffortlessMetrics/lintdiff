# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

lintdiff is a Rust CLI tool that filters Rust compiler and Clippy diagnostics to only lines touched by a PR, emitting a schema-validated receipt for CI integration. Core question: "Did this change introduce actionable diagnostics on changed lines?"

## Common Commands

```bash
# Build
cargo build

# Test everything
cargo test --workspace

# Run BDD scenarios only
cargo test -p lintdiff --test bdd

# Format check (CI gate)
cargo fmt --all -- --check

# Format fix
cargo fmt --all

# Run CLI
cargo run -p lintdiff -- ingest --help

# Fuzz (requires nightly + cargo-fuzz)
cd fuzz && cargo fuzz run diff_parser -- -max_total_time=30

# Mutation test (requires cargo-mutants)
cargo mutants -p lintdiff-domain --timeout 300
```

## Architecture

Hexagonal (Ports/Adapters) design with microcrate layout:

```
lintdiff-cli          Clap CLI, subcommand routing
    │
    ▼
lintdiff-app          Orchestration (delegates to app-git, app-io)
    │
    ├──► lintdiff-app-git      Git adapter (diff, repo root, git info)
    ├──► lintdiff-app-io       I/O adapter (config, diagnostics, artifacts)
    ├──► lintdiff-feature-flags Feature-flag registry and parsing
    ├──► lintdiff-diff          Unified diff → DiffMap
    ├──► lintdiff-domain        Facade → lintdiff-core → lintdiff-ingest → lintdiff-ingest-core
    │        ├──► lintdiff-match         Span selection, path matching, filters
    │        └──► lintdiff-policy        Code normalization, verdict, fingerprint
    ├──► lintdiff-render        Markdown and GitHub annotations output
    │
    ▼
lintdiff-types        DTOs, config, schemas, path normalization, ordering
```

**Data flow:** CLI → App acquires diff + diagnostics → Domain matches spans to changed ranges → Report (JSON) → Optional render (markdown/annotations)

## Key Invariants

- **Determinism:** Same inputs must produce byte-identical outputs. No filesystem iteration order, stable ordering, reproducible truncation.
- **No unsafe code:** `unsafe_code = forbid` at workspace level
- **Path canonicalization:** Repo-relative, forward slashes, no leading `./` — enforced in all layers
- **Schema stability:** `lintdiff.report.v1` is a versioned contract; changes require schema validation
- **Failure modes:** Missing inputs → `skip` verdict (not pass), parse errors → tool error (exit 1)
- **Span selection:** Primary spans preferred; falls back to all spans if none marked primary

## Exit Codes

- `0`: pass or warn
- `1`: tool/runtime error (I/O, parse, config)
- `2`: policy failure (blocking findings)

## Test Layers

1. **Unit tests** — inline in each crate
2. **Golden fixtures** — `tests/fixtures/` for schema compliance and determinism
3. **BDD scenarios** — Cucumber tests in `crates/lintdiff-cli/tests/bdd.rs`
4. **Property tests** — proptest in lintdiff-diff
5. **Fuzz tests** — separate `fuzz/` workspace (nightly), runs weekly in CI
6. **Mutation tests** — cargo-mutants on lintdiff-domain, runs weekly in CI

## Linting Configuration

Workspace-level in `Cargo.toml`:
- Clippy: `all`, `pedantic`, `nursery` enabled
- Allowed: `too_many_lines`, `module_name_repetitions`
- `unsafe_code = forbid`

## Configuration

- **CLI config:** `lintdiff.toml` at repo root (auto-discovered)
- **Example:** `lintdiff.toml.example`
- **Policy options:** `fail_on`, `max_findings`, `workspace_only`, path filters, code allow/suppress/deny lists
