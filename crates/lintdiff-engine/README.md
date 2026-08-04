# lintdiff-engine

Embeddable pure diagnostic analysis and comparison for [lintdiff](https://crates.io/crates/lintdiff).

The engine accepts acquired Cargo diagnostics and source-diff data, then returns
typed lintdiff evidence. It does not spawn Git or Cargo, read the filesystem,
inspect the environment, or render provider output. The application crate owns
those effects.

The top-level API includes Cargo JSON parsing, complete inventory construction,
source-diff parsing, matching, policy evaluation, receipt construction, and the
advisory inventory comparison experiment. Advanced modules remain available
through deliberate re-exports; callers should begin with the typed analysis and
receipt functions rather than rebuilding the internal pipeline.

The minimum supported Rust version is 1.95. The `0.1.x` API is public but remains
pre-1.0 and follows the compatibility policy documented in release notes.
Diagnostic comparison is advisory and does not provide a general strict-blocking
or causal-detection guarantee.
