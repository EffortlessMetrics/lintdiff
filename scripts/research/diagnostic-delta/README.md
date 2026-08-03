# Diagnostic-delta discovery harness

This directory contains disposable research helpers for issue #156. Helpers
may acquire external repositories and preserve raw Cargo streams under the
ignored `artifacts/` tree, but must not become a second implementation of
lintdiff parsing, identity, matching, policy, or rendering.

The fixed first-pass command is:

```text
cargo clippy --workspace --all-targets --all-features --message-format=json
```

Run it from a clean base and head checkout with the same pinned Rust toolchain.
Record stdout/stderr separately, retain the process exit code, and inspect the
Cargo JSON for completion evidence. A failed or incomplete run is evidence of
limited comparability, not a basis for inventing a delta.

Human adjudication remains authoritative. The harness may summarize raw files,
line/path diffs, and current changed-line receipts, but it must not call
lintdiff's production matcher to label a case.
