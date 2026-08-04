# lintdiff

The installable CLI and application library for Rust diagnostic receipts.

The maintained workflow locates rustc or Clippy diagnostics on lines touched by
a pull request and writes a deterministic, schema-validated `lintdiff.report.v1`
receipt. It consumes an existing diagnostics stream and does not build base and
head revisions for you.

## Install

The coordinated `0.1.2` registry release installs the product with:

```text
cargo install lintdiff --version 0.1.2 --locked
lintdiff --version
```

The repository also distributes exact-tag GitHub Action and release-binary
artifacts. The release verification matrix covers Linux x86_64, macOS x86_64,
macOS arm64, and Windows x86_64. Cargo installation requires a Rust 1.95 or
newer toolchain supported on the target host.

## Scope

`ingest`, `run`, and `ci github` are the maintained application workflows.
`inventory` and `compare` are shipped experimental, advisory commands. Their
results are not an externally validated general guarantee of new or resolved
diagnostics and are not the maintained strict-blocking contract.

See the [product contract](https://github.com/EffortlessMetrics/lintdiff/blob/main/PRODUCT.md)
and [release process](https://github.com/EffortlessMetrics/lintdiff/blob/main/docs/release-process.md)
for supported inputs, exact-tag installation, and platform details.
