# lintdiff-render

Projection helpers for [lintdiff](https://crates.io/crates/lintdiff) receipts.

This registry-support crate converts typed `lintdiff-types` reports into lossy
consumer views, including:

- GitHub-flavored Markdown;
- GitHub Actions workflow annotations.

The canonical receipt remains the complete `lintdiff.report.v1` artifact. Rendering
does not acquire diagnostics, compare analyses, or apply repository policy. Use
the `lintdiff` product crate for normal CLI and Action workflows.

The minimum supported Rust version is 1.95. The crate follows the `0.1.x`
pre-1.0 compatibility policy and is published so registry consumers can resolve
the `lintdiff` dependency closure.
