# ADR-006: Canonical Package Topology

## Status

Accepted — 2026-08-02

## Decision

lintdiff uses four runtime packages and one repository-tooling package:

| Package | Ownership |
| --- | --- |
| `lintdiff-types` | public evidence protocols, wire primitives, report/inventory/delta DTOs, schema fixtures |
| `lintdiff-engine` | pure diagnostic analysis, source correspondence, matching, identity, policy, and receipt construction |
| `lintdiff-render` | pure Markdown, GitHub annotation, and RDJSONL receipt projections |
| `lintdiff` | application library, binary, CLI, Git/filesystem/process adapters, configuration, artifacts, and exit behavior |
| `xtask` | repository control plane for architecture, schema, fixture, docs, and release-contract checks |

`fuzz/` remains an excluded auxiliary workspace. No dev-support package is retained;
package-local test support is sufficient for the current graph.

The allowed product dependency envelope is:

```text
lintdiff-types       → none
lintdiff-engine      → lintdiff-types
lintdiff-render      → lintdiff-types
lintdiff             → lintdiff-engine, lintdiff-render, lintdiff-types
xtask                → repository tooling dependencies only
```

Only `lintdiff-types` has publication intent. The engine, renderer, application,
and xtask are `publish = false`; internal package boundaries do not create an
external support promise.

## Rationale

The former workspace contained many packages with no independent consumer,
release, or dependency envelope. The collapse preserves the logical pipeline while
placing implementation seams inside the packages that own their shared change
cadence. Protocol compatibility remains a public concern; analysis and rendering
remain pure internal concerns; external effects remain in the application shell.

The complete disposition ledger and registry/consumer evidence live in
[`plans/microcrate-collapse-ledger.toml`](../../plans/microcrate-collapse-ledger.toml).
`xtask architecture-check` enforces the topology from Cargo metadata rather than a
manually maintained package count.

## Consequences

- A new public package requires a named consumer contract and publication decision.
- Pure engine and renderer changes can be tested without Git, filesystem, process,
  environment, or terminal dependencies.
- The protocol package must narrow configuration and engine-policy ownership in a
  separate compatibility-managed API change; that is not part of physical package
  deletion.
- Historical expansion and migration plans remain useful provenance but are not
  current package or module maps.
