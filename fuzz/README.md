# lintdiff fuzzing

Fuzz targets are kept out of the main workspace to avoid pulling nightly tooling into normal CI.

## Setup

```bash
cargo install cargo-fuzz
```

## Run

```bash
cd fuzz
cargo fuzz run diff_parser
cargo fuzz run diagnostics_parser
```

CI runs these on a schedule (timeboxed).
