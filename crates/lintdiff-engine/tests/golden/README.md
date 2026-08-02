# Engine cutover golden contract

This directory is the tracked destination for the receipt fixtures listed in
`plans/engine-test-migration.toml`. The current baseline fixtures remain under
`tests/snapshots/` until the coherent MC-7 source move.

The cutover must preserve, byte-for-byte:

- `lintdiff.report.v1` receipt JSON;
- pass, warn, and fail verdict receipts;
- explain dispositions and summary counts;
- finding order and fingerprint values;
- path/rename correspondence and failure/skip behavior.

Fixtures and tests must invoke the production ingest implementation. A test,
benchmark, or fuzz target must not reproduce matching, normalization,
fingerprinting, policy, or receipt-construction algorithms.
