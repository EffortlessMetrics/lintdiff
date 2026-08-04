# lintdiff-types

Versioned evidence protocols and wire types for [lintdiff](https://crates.io/crates/lintdiff).

This crate defines the serializable contracts used by lintdiff receipts:

- `lintdiff.report.v1` for changed-line diagnostic receipts;
- `lintdiff.inventory.v1` for complete diagnostic observations;
- `lintdiff.delta.v1` for advisory comparison evidence.

It also retains compatibility helpers for configuration, repository paths, and
deterministic finding ordering. Those helpers are part of the current `0.1.x`
API and are not silently removed as part of the publication closure.

The minimum supported Rust version is 1.95. Protocol consumers should deserialize
the versioned envelopes and treat unknown extension fields as forward-compatible.

See the [lintdiff product contract](https://github.com/EffortlessMetrics/lintdiff/blob/main/PRODUCT.md)
for the maintained changed-line workflow and the limits of the experimental
inventory and comparison surfaces.
