# ADR-006: Canonical Package Topology and Publication Closure

## Status

Accepted — 2026-08-02; publication amendment 2026-08-04

## Decision

lintdiff uses four runtime packages and one repository-tooling package:

| Package | Ownership |
| --- | --- |
| `lintdiff-types` | public evidence protocols, wire primitives, report/inventory/delta DTOs, schema fixtures; versioned registry contract |
| `lintdiff-engine` | public embeddable pure diagnostic analysis, source correspondence, matching, identity, policy, and receipt construction |
| `lintdiff-render` | public registry-support projection crate for Markdown, GitHub annotation, and RDJSONL receipts |
| `lintdiff` | primary product crate: application library, installable binary, CLI, Git/filesystem/process adapters, configuration, artifacts, and exit behavior |
| `xtask` | private repository control plane for architecture, schema, fixture, docs, and release-contract checks |

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

The four runtime packages form the intended crates.io publication closure. Their
first coordinated publication targets version `0.1.2`, in dependency order:

```text
lintdiff-types → lintdiff-engine → lintdiff-render → lintdiff
```

`lintdiff` is the primary registry product and supports `cargo install lintdiff`
once the `0.1.2` publication and clean install proof complete. `xtask` remains
private and is not part of the registry closure. The topology contract records
this as `publication_intent`; the existing `publish` fields remain the current
manifest state until the separately reviewable package-preparation change lands.
Publication is an external release operation; this amendment adopts the target
and proof obligations but does not claim that the packages are already published.

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

- Each published package requires a documented consumer contract, package
  metadata, registry dependency closure, and SemVer proof appropriate to its
  support promise.
- The application package is the named product crate; binary release assets and
  the exact-tag Action remain supported distribution surfaces alongside
  `cargo install lintdiff`.
- Pure engine and renderer changes can be tested without Git, filesystem, process,
  environment, or terminal dependencies.
- The protocol package must narrow configuration and engine-policy ownership in a
  separate compatibility-managed API change; that is not part of physical package
  deletion.
- Historical expansion and migration plans remain useful provenance but are not
  current package or module maps.
