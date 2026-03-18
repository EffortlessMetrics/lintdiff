# ADR-002: Ensure Deterministic Output

## Status

Accepted

## Context

lintdiff is designed to run in CI/CD pipelines where reproducibility is critical. Users and automation systems rely on consistent output for:

- **Fingerprinting**: Identifying unique findings across runs
- **Caching**: Avoiding redundant processing for identical inputs
- **Diff comparison**: Comparing results between branches or commits
- **Audit trails**: Having consistent records of what was detected

Non-deterministic output causes several problems:
- CI systems may incorrectly report changes when none exist
- Caching mechanisms fail or produce false hits
- Debugging becomes difficult when outputs vary between runs
- Trust in the tool's reliability is undermined

Common sources of non-determinism include:
- HashMap iteration order (randomized by default in Rust)
- Parallel execution with non-deterministic scheduling
- Timestamps or other time-dependent values
- System-dependent paths or environment values

## Decision

All outputs produced by lintdiff must be **byte-stable** for identical inputs. This means:

### Output Requirements

1. **JSON output**: Must produce identical bytes for identical inputs
2. **Markdown output**: Must produce identical bytes for identical inputs
3. **Exit codes**: Must be consistent for identical inputs
4. **Log messages**: Should be deterministic (excluding timing information)

### Implementation Rules

1. **No HashMap iteration for output**: Use `BTreeMap` or sort keys before serialization
2. **Stable ordering**: All collections must be serialized in a defined order
3. **No timestamps in output**: Or use deterministic/fixed timestamps when required
4. **Canonical paths**: Normalize paths to a consistent format
5. **Sorted diagnostic output**: Findings must be emitted in a predictable order

### Verification

The project includes determinism tests (see `crates/lintdiff-ingest-core/tests/determinism.rs`) that verify byte-identical output across multiple runs with the same input.

## Consequences

### Positive

- **Stable fingerprinting**: Findings can be reliably tracked across runs
- **CI reliability**: No spurious failures due to output differences
- **Caching effectiveness**: Results can be cached and reused confidently
- **Reproducibility**: Anyone can reproduce the same results from the same inputs
- **Debugging**: Easier to compare outputs and identify actual changes

### Negative

- **Performance cost**: Sorting operations add overhead compared to unordered output
- **Implementation care**: Developers must be mindful of ordering in all output paths
- **Testing burden**: Determinism must be verified, not just correctness

### Mitigations

- Use `BTreeMap` by default for map-like data structures in output contexts
- Implement `Ord` for types that appear in sorted output
- Include determinism checks in the test suite
- Document the determinism requirement in contribution guidelines
