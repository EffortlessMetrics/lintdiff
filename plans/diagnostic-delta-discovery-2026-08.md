# Diagnostic-delta discovery corpus — 2026-08

This is a disposable, research-only corpus for issue #156. It is not a
production matcher and does not define the inventory or delta schemas.

## Analysis protocol

- Initial protocol: `rustc 1.97.1 (8bab26f4f 2026-07-14)` / Cargo `1.97.1`.
- Analysis command: `cargo clippy --workspace --all-targets --all-features
  --message-format=json`.
- The command vector, toolchain, repository commit, package selection, target
  selection, feature selection, and relevant environment are recorded with
  every run.
- Base and head are run independently from clean worktrees. Raw stdout/stderr,
  exit status, and the source diff remain under ignored `artifacts/`.
- `serde` and `ripgrep` could not complete this all-targets/all-features
  protocol on stable because their selected revisions contain unstable feature
  gates. They were rerun as matched base/head pairs with the installed
  `nightly` toolchain (`rustc 1.99.0-nightly`, Cargo `1.99.0-nightly`). The
  stable failures remain retained as provenance and setup-cost evidence.
- No lintdiff production API, parser, matcher, or report is used as ground
  truth. Human adjudication is the authority for the discovery labels.

## Candidate cases

The first pass contains five merged pull requests from each of two
EffortlessMetrics repositories and two independently maintained repositories.
The selections intentionally include code changes, refactors, tests,
release/config work, and likely warning-debt surfaces. The `serde` cases are
also retained as explicit stable-toolchain incompatibility evidence; the
`ripgrep` cases provide an independently maintained stable-toolchain batch.

| Repository | PR | Base | Head | Initial focus |
|---|---:|---|---|---|
| `EffortlessMetrics/cargo-allow` | #3068 | `45017590c23c` | `e54df21b4c40` | policy/diff reclassification |
| `EffortlessMetrics/cargo-allow` | #3060 | `5b8987c3a4fa` | `6fb79be085e3` | legacy source locations |
| `EffortlessMetrics/cargo-allow` | #3058 | `1e32a849a462` | `9f0ea3fb9e43` | policy/config schema |
| `EffortlessMetrics/cargo-allow` | #3054 | `3c82d812cab9` | `3842b08664e5` | release scripts |
| `EffortlessMetrics/cargo-allow` | #3051 | `691447de03ce` | `a7c09a4a08f0` | CI/performance policy |
| `EffortlessMetrics/pst-rs` | #1316 | `3e69c050e38d` | `816237e5222e` | CLI output and snapshots |
| `EffortlessMetrics/pst-rs` | #1315 | `501c4b4ab8a1` | `c0a47b8640c3` | CLI summary refactor |
| `EffortlessMetrics/pst-rs` | #1314 | `a9fbef4b064c` | `38d9721bbafd` | FFI test arithmetic |
| `EffortlessMetrics/pst-rs` | #1313 | `3c6fed8e6997` | `009d7f8b203e` | CLI glob errors |
| `EffortlessMetrics/pst-rs` | #1311 | `32014c6ffa1e` | `18f25fd07361` | benchmark arithmetic |
| `serde-rs/serde` | #3038 | `5fd91333c7dc` | `ea04847eef16` | derive codegen refactor |
| `serde-rs/serde` | #3037 | `5fd91333c7dc` | `1229fc8a6dbd` | AST iteration refactor |
| `serde-rs/serde` | #3035 | `681270e4f096` | `bfc9ccf68697` | bound generation refactor |
| `serde-rs/serde` | #3034 | `681270e4f096` | `2c6d0d2e0969` | filter-map refactor |
| `serde-rs/serde` | #3001 | `4e278703c624` | `086353c58174` | attribute nesting refactor |
| `BurntSushi/ripgrep` | #3496 | `dffd776a737d` | `5467965ad582` | ignore-file reachability |
| `BurntSushi/ripgrep` | #3475 | `5e16a5c9e57e` | `db6b8cd36618` | visitor panic deadlock |
| `BurntSushi/ripgrep` | #3472 | `fc3dd04b7ca1` | `250fe3603946` | incremental checking |
| `BurntSushi/ripgrep` | #3482 | `227381db0ee8` | `b0d8f7c9cb31` | unstable index feature |
| `BurntSushi/ripgrep` | #3487 | `59e318f5ace4` | `b6f2d1ca68e0` | index scaffolding |

## Per-case evidence

Each case gets one JSONL ledger row under ignored `artifacts/` containing:

- base/head SHA and PR URL;
- exact command and toolchain;
- package/target/features/config observations;
- base/head exit and Cargo completion state;
- raw stream paths and source diff path;
- current changed-line receipt when the repository can run lintdiff;
- human-adjudicated observations and decision effect.

The final adjudication is recorded in
[diagnostic-delta-discovery-verdict-2026-08.md](diagnostic-delta-discovery-verdict-2026-08.md).
