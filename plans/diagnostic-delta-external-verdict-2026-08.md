# Diagnostic-delta external verdict — 2026-08

Issue #106 was evaluated against ten real pull requests after the narrow
discovery verdict in `diagnostic-delta-discovery-verdict-2026-08.md`. The
sample contains five consecutive `pst-rs` pull requests for repeated-use
evidence, three `serde` pull requests from an independently maintained Rust
repository, and two `ripgrep` pull requests from an independently maintained
repository with substantial existing Clippy debt.

## Campaign protocol

The final run is `20260803T101626Z`. Its ignored artifact root is:

```text
artifacts/research/diagnostic-delta-external/20260803T101626Z
```

The tracked rerun harness is
`scripts/research/diagnostic-delta/run-external-verdict.ps1`.

Each case preserved the base and head SHA, source diff, exact command vector,
raw Cargo JSON stream and metadata, per-side inventory and report, delta JSON
and Markdown, and product exit codes. The analysis command was:

```text
cargo clippy --workspace --all-targets --all-features --message-format=json
```

The `serde` and `ripgrep` cases used matched nightly analyses where their
selected revisions required unstable features. All ten final pairs were
successful and complete: both sides exited zero, emitted `build-finished`,
and reported `build_success = true`. The harness passed each side's original
analysis root so absolute worktree paths could not become producer identity.

The production delta was not treated as ground truth. Reviewdog was not
available in this campaign; the comparison baseline was the source diff plus
the existing changed-line receipt, with manual adjudication from the discovery
corpus where available.

## Results

| Repository / PR | Comparable | Inventory delta total | New | Resolved | Modified | Ambiguous | Changed-line findings |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `pst-rs` #1311 | yes | 4 | 0 | 0 | 0 | 1 | 1 |
| `pst-rs` #1313 | yes | 4 | 0 | 0 | 0 | 1 | 1 |
| `pst-rs` #1314 | yes | 4 | 0 | 0 | 0 | 1 | 1 |
| `pst-rs` #1315 | yes | 4 | 0 | 0 | 0 | 2 | 1 |
| `pst-rs` #1316 | yes | 4 | 0 | 0 | 0 | 2 | 1 |
| `serde` #3001 | yes | 1 | 0 | 0 | 0 | 1 | 1 |
| `serde` #3034 | yes | 1 | 0 | 0 | 0 | 1 | 1 |
| `serde` #3038 | yes | 1 | 0 | 0 | 0 | 1 | 1 |
| `ripgrep` #3475 | yes | 652 | 0 | 0 | 0 | 254 | 1 |
| `ripgrep` #3482 | yes | 806 | 150 | 144 | 24 | 256 | 1 |

Nine cases produced no confident `new` or `resolved` classification. The
`ripgrep` #3482 result is decision-relevant: the source change adds indexing
code, produces four changed-line findings, and also moves a large existing
warning population. The current delta additionally reports 146 `new` items
outside the diff, 144 `resolved` items, 24 `modified` items, and 256 ambiguous
items.

The 24 modified items remain after transport rendering and suggestion
coordinate normalization because their suggestion replacements differ. They
are not counted here as transport-only false changes.

## Falsification and claim boundary

The run establishes:

* the selected ten pairs can be captured as successful, complete, hard-scope-
  compatible evidence;
* producer identity must be canonicalized across base/head worktrees;
* transport rendering and suggestion coordinates must not create semantic
  changes;
* ambiguity is common in large existing-debt inventories; and
* the delta model can expose movement and off-diff evidence that the current
  changed-line receipt does not expose.

It does not establish the #106 precision threshold. The corpus has not
manually adjudicated every confident unmatched classification, and the
`ripgrep` #3482 output is a known warning sign: repeated producer groups and
source movement produce more confident `new`/`resolved` classifications than
the discovery adjudication supports. Those classifications must not be used
as blocking policy or presented as proven causal changes.

There was no external maintainer retention of the experimental mode for five
pull requests. Reviewdog was not run, so provider-level comparison remains
unverified.

## Verdict: narrow / stop the current buildout

Keep the trustworthy `v0.1.x` changed-line receipt as the maintained product.
Do not authorize a public engine API, automatic base/head orchestration,
strict delta blocking, broader integrations, or the `lintdiff-types` 0.2
compatibility cleanup from this campaign.

The experiment may remain as advisory research evidence, but the current
`new`/`resolved` output is not sufficiently conservative for an earned
external product claim. The next delta work, if reauthorized, must first
address conservation of duplicate/source-correspondence groups and add
manual precision adjudication before changing the product surface.
