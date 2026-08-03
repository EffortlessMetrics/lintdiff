# Diagnostic-delta discovery verdict — 2026-08

Issue #156 examined 20 historical pull requests across four repositories:
EffortlessMetrics `cargo-allow` and `pst-rs`, and independently maintained
`serde` and `ripgrep`. The corpus includes code changes, refactors, tests,
configuration, release, and unstable-feature work. Raw Cargo streams, stderr,
metadata, source diffs, worktrees, and run ledgers are retained under the
ignored `artifacts/research/diagnostic-delta/` tree.

## Protocol result

The command was the same within each base/head pair:

```text
cargo clippy --workspace --all-targets --all-features --message-format=json
```

The initial stable protocol used Rust/Cargo 1.97.1. The five `serde` pairs and
the five `ripgrep` pairs could not all complete on stable because the selected
revisions contain unstable feature gates, including benchmark or indexing
features. Those cases were rerun as matched pairs with the installed nightly
toolchain (`rustc 1.99.0-nightly`, Cargo 1.99.0-nightly). This is an explicit
comparability and setup-cost finding, not a silently discarded harness error.

Final pair status:

| Result | Cases |
| --- | ---: |
| Successful and comparable | 15 |
| Failed-complete or otherwise incomparable | 5 |
| Historical PRs examined | 20 |

The five final incomparable cases are `cargo-allow` #3060, #3058, #3054, and
#3051, whose base and head builds both failed after Cargo emitted
`build-finished`, plus `ripgrep` #3487, whose base completed but whose head
failed while the new index scaffolding was still missing `IndexDiscovery` and
`Index` symbols.

## Human adjudication

The labels below are discovery conclusions from the raw observations and source
diffs. They are not calls to a lintdiff production matcher.

| Repository / PR | Analysis | Adjudicated signal |
| --- | --- | --- |
| `cargo-allow` #3068 | comparable | No compiler diagnostics on either side; no delta signal. |
| `cargo-allow` #3060 | incomparable | Failed-complete on both sides; no new/resolved claim. |
| `cargo-allow` #3058 | incomparable | Failed-complete on both sides; no new/resolved claim. |
| `cargo-allow` #3054 | incomparable | Failed-complete on both sides; no new/resolved claim. |
| `cargo-allow` #3051 | incomparable | Failed-complete on both sides; no new/resolved claim. |
| `pst-rs` #1316 | comparable | Warning set was unchanged and no warning was on a touched line. |
| `pst-rs` #1315 | comparable | Warning set was unchanged and no warning was on a touched line. |
| `pst-rs` #1314 | comparable | `manual_checked_ops` remained on line 172 while the edit was line 173; existing, not touched. |
| `pst-rs` #1313 | comparable | Warning set was unchanged and no warning was on a touched line. |
| `pst-rs` #1311 | comparable | Warning set was unchanged and no warning was on a touched line. |
| `serde` #3038 | comparable under nightly | The warning set was unchanged; stable setup was not comparable. |
| `serde` #3037 | comparable under nightly | The warning set was unchanged; stable setup was not comparable. |
| `serde` #3035 | comparable under nightly | The warning set was unchanged; stable setup was not comparable. |
| `serde` #3034 | comparable under nightly | The warning set was unchanged; stable setup was not comparable. |
| `serde` #3001 | comparable under nightly | The warning set was unchanged; stable setup was not comparable. |
| `ripgrep` #3496 | comparable under nightly | Existing warnings remained outside touched ranges; no new/resolved signal. |
| `ripgrep` #3475 | comparable under nightly | Warning set was unchanged and no touched-line warning was adjudicated. |
| `ripgrep` #3472 | comparable under nightly | Warning set was unchanged and no touched-line warning was adjudicated. |
| `ripgrep` #3482 | comparable under nightly | Added indexing code produced head-only Clippy warnings on changed lines; large insertions also moved existing warning locations. |
| `ripgrep` #3487 | incomparable | Base completed; head failed-complete while index symbols were missing. |

The `ripgrep` #3482 result is the strongest product signal in this corpus:
it contains both a real `new_on_diff` candidate and line movement that would be
misleading under location-only identity. The corpus did not produce a repeated
`new_off_diff`, `resolved`, or message-modified case. It also did not contain a
duplicate group that required an ambiguity decision, so ambiguity precision is
not established by this discovery pass.

## Verdict: narrow proceed

Proceed with a narrower diagnostic-delta experiment, not a general claim that
base/head comparison can classify every diagnostic change.

The first implementation contract should require:

1. complete, successful, hard-scope-compatible base and head analyses for
   confident `new` and `resolved` claims;
2. explicit failed-complete and incomplete-stream provenance that blocks a
   confident delta rather than fabricating one;
3. one observation per Cargo compiler emission, including producer package and
   target identity;
4. source correspondence that can represent movement and unmappable regions;
5. policy-free pairing evidence with ambiguity separate from `DeltaKind`;
6. advisory output for the first delta receipt; and
7. the #3482, #3487, and #1314 cases promoted into the ratified #101 corpus.

This verdict supports completing #101 from the discovery evidence and then
starting the existing #157 and #103 lanes. It does not justify a public engine
API, automatic base/head orchestration, a breaking `lintdiff-types` release,
or strict default delta blocking. Those remain gated by #106.
