# lintdiff Test Strategy

lintdiff is a gatekeeper-style tool. The "architecture" is mostly the test discipline.

## Test Coverage Summary

### Coverage shape by layer

- Runtime/domain behavior is covered in `crates/lintdiff-ingest-core` through deterministic golden tests, fixture contract suites, and property-oriented checks embedded in the migrated modules.
- CLI and I/O adapters are covered in `crates/lintdiff-app`, `crates/lintdiff-app-git`, and `crates/lintdiff-app-io` integration tests.
- Output/rendering paths are covered in dedicated renderer and report tests.
- Fuzzing continues to cover diff and diagnostic parsers in `fuzz/fuzz_targets/*.rs`.

### Test Categories

| Category | Count | Description |
|----------|-------|-------------|
| Unit Tests | Varies by crate | Fast, isolated tests for individual functions |
| Integration Tests | Varies by crate | Tests that verify component interactions |
| Property-based Tests | Varies by crate | Proptest-driven checks where algorithmic invariants apply |
| Doc Tests | Varies by crate | Inline documentation examples |
| BDD Scenarios | 1 harness + fixture suite | Behavior-driven tests for user-facing features |
| Fuzz Targets | 3 | cargo-fuzz targets for parser robustness |

---

## Test Layers

### Layer 1: Unit Tests

Unit tests are embedded in each crate's `src/lib.rs` file using the `#[cfg(test)]` module. They test individual functions in isolation.

**Location**: `crates/<crate>/src/lib.rs` (inline `tests` module)

**Characteristics**:
- Fast execution (milliseconds)
- No external dependencies
- Test individual functions and edge cases

**Example crates with unit tests**:
- [`lintdiff-ingest-core`](../crates/lintdiff-ingest-core/src/lib.rs) - Diff parsing, matching, policy composition
- [`lintdiff-types`](../crates/lintdiff-types/src/lib.rs) - DTO and config contracts
- [`lintdiff-report-builder`](../crates/lintdiff-report-builder/src/lib.rs) - Receipt assembly

### Layer 2: Integration Tests

Integration tests verify that components work correctly together. They are located in each crate's `tests/` directory.

**Location**: `crates/<crate>/tests/*.rs`

**Representative integration suites**:

- `crates/lintdiff-ingest-core/tests/determinism.rs`
- `crates/lintdiff-ingest-core/tests/fingerprint_integration.rs`
- `crates/lintdiff-app-io/tests/diagnostics_io_tests.rs`
- `fuzz/fuzz_targets/*.rs` are validated through fixture-backed CI targets.

**Golden Fixtures** (contract tests):

Fixtures are the primary contract for integration tests.

- Inputs:
  - `tests/fixtures/*.diff`
  - `tests/fixtures/*.jsonl`
- Outputs (golden):
  - `tests/fixtures/expected/*.report.json`
  - `tests/fixtures/expected/*.comment.md`

Golden tests enforce:
- Schema compliance
- Deterministic ordering
- Stable truncation behavior

### Layer 3: BDD Tests (Behavior Tests)

BDD tests ensure the tool behaves as reviewers expect using "Given/When/Then" scenarios.

**Location**: `crates/lintdiff-cli/tests/features/*.feature`

**Scenarios covered**:
- Match diagnostics on changed lines
- Ignore diagnostics outside the diff
- Primary span selection
- Workspace-only filtering
- Missing input semantics (`skip`, not `pass`)

**Related crates**:
- [`lintdiff-bdd-harness`](../crates/lintdiff-bdd-harness/) - Test harness
- [`lintdiff-bdd-grid`](../crates/lintdiff-bdd-grid/) - Feature flag matrix testing

BDD is intentionally thin; golden fixtures carry the detailed surface contract.

### Layer 4: Property-based Tests

Property tests protect core algorithms using [`proptest`](https://proptest-rs.github.io/proptest/).

**Location**: `crates/<crate>/tests/property_tests.rs`

**Crates with property tests**:
- `lintdiff-ingest-core` (range merging, path normalization and fingerprint invariants)
- Additional utility crates in workspace modules where algorithmic invariants are useful.

**Properties tested**:
- Range merge/idempotence
- Intersection correctness
- Fingerprint stability
- Path normalization idempotence

### Layer 5: Fuzz Tests

Fuzzing protects parsers against panics and pathological inputs using [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz).

**Location**: `fuzz/fuzz_targets/*.rs`

**Fuzz targets**:
- [`diagnostics_parser.rs`](../fuzz/fuzz_targets/diagnostics_parser.rs) - JSON diagnostic parsing
- [`diff_parser.rs`](../fuzz/fuzz_targets/diff_parser.rs) - Unified diff parsing
- [`finding_fingerprint.rs`](../fuzz/fuzz_targets/finding_fingerprint.rs) - Fingerprint generation

**Execution**: Run timeboxed (CI schedule or on-demand), not per PR by default.

---

## Running Tests

### Run All Tests

```bash
# Run all tests in the workspace
cargo test --workspace

# Run all tests including doc tests
cargo test --workspace --all-features
```

### Run Specific Test Suites

```bash
# Unit tests for a specific crate
cargo test -p lintdiff-ingest-core --lib

# Integration tests for a specific crate
cargo test -p lintdiff-ingest-core --test determinism

# Property tests
cargo test -p lintdiff-ingest-core --test fingerprint_integration

# BDD tests
cargo test -p lintdiff --test bdd

# Doc tests only
cargo test --workspace --doc
```

### Run Specific Tests

```bash
# Run tests matching a pattern
cargo test -p lintdiff-ingest-core --test determinism -- no-skip

# Run a single test
cargo test -p lintdiff-app-io --test diagnostics_io_tests -- no-skip
```

### Run Fuzz Tests

```bash
# Install cargo-fuzz if not already installed
cargo install cargo-fuzz

# Run a fuzz target (timeboxed)
cargo fuzz run diff_parser -- -max_total_time=60

# Run all fuzz targets
cargo fuzz run diagnostics_parser -- -max_total_time=60
cargo fuzz run finding_fingerprint -- -max_total_time=60
```

### CI Integration

The CI pipeline runs tests in multiple stages:

1. **Fast Feedback** (every PR):
   - `cargo test --workspace --lib` (unit tests)
   - `cargo test --workspace --tests` (integration tests)
   - `cargo clippy --workspace`

2. **Full Validation** (main branch):
   - All unit and integration tests
   - BDD tests
   - Doc tests
   - Schema validation

3. **Scheduled** (daily/weekly):
   - Fuzz testing (timeboxed)
   - Mutation testing (`cargo-mutants`)

---

## Coverage Goals

### Target Coverage Percentages

| Component | Target | Notes |
|-----------|--------|-------|
| Core algorithms | 90%+ | Diff parsing, fingerprinting, matching |
| Policy logic | 85%+ | Verdict computation, code normalization |
| Rendering | 80%+ | Markdown, annotations |
| I/O layer | 70%+ | Git operations, file handling |

### Critical Path Coverage Requirements

The following code paths **must** have 100% test coverage:

1. **Diff parsing** - All unified diff formats
2. **Diagnostic parsing** - All cargo/clippy message formats
3. **Fingerprint generation** - Determinism and stability
4. **Verdict computation** - All `fail_on` modes
5. **Output rendering** - Markdown and annotation formats

### Coverage Measurement

```bash
# Generate coverage report using cargo-tarpaulin
cargo tarpaulin --workspace --out Html --output-dir coverage/

# Generate coverage using cargo-llvm-cov (requires nightly)
cargo llvm-cov --workspace --html
```

---

## Adding New Tests

### Guidelines for Contributors

1. **Unit Tests**: Add inline in `src/lib.rs` within a `#[cfg(test)] mod tests` block.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_new_function_works() {
        // Arrange
        let input = "test input";

        // Act
        let result = my_function(input);

        // Assert
        assert_eq!(result, expected_output);
    }
}
```

2. **Integration Tests**: Create a new file in `tests/` directory.

```rust
// crates/lintdiff-<crate>/tests/my_feature_tests.rs
use lintdiff_<crate>::MyStruct;

#[test]
fn my_feature_integration() {
    // Test component interaction
}
```

3. **Property Tests**: Add to `tests/property_tests.rs` using proptest.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn my_property_holds(input in ".*") {
        let result = my_function(&input);
        prop_assert!(my_invariant(&result));
    }
}
```

4. **BDD Tests**: Add scenarios to `crates/lintdiff-cli/tests/features/*.feature`.

```gherkin
Scenario: New feature behaves correctly
  Given a diff with changes
  When I run lintdiff
  Then the output should be valid
```

### Test Naming Conventions

- Use descriptive names: `test_<function>_<scenario>_<expected_result>`
- Example: `test_parse_diff_with_multiple_files_returns_all_files`
- Group related tests in modules: `mod parsing { ... }`

### Test Organization

```
crates/lintdiff-<crate>/
├── src/
│   └── lib.rs          # Unit tests in #[cfg(test)] mod tests
└── tests/
    ├── <feature>_tests.rs    # Integration tests
    ├── property_tests.rs     # Property-based tests
    └── fixtures/             # Test data files
```

---

## Mutation Testing

Mutation testing keeps you honest about coverage and the quality of assertions using [`cargo-mutants`](https://github.com/sourcefrog/cargo-mutants).

**Recommendation**:
- Run on schedule (weekly) or on demand
- Treat "mutants survived" as a prompt to improve tests, not a release blocker

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation testing
cargo mutants --workspace

# Run on specific crate
cargo mutants -p lintdiff-ingest-core
```

---

## Runtime Crypto Fixtures

If a test needs secrets-shaped input (PKCS#8 PEM, SPKI, JWK/JWKS, tokens, X.509 certs), generate it at runtime with `uselesskey` instead of committing fixture files.

- Keep PEM/JWK/certificate blobs out of git history
- Avoid false positives from secret scanners
- Prefer deterministic seeds for normal tests so snapshots stay stable
- Use `Factory::random()` only for fuzz/property-style coverage where shape variation is the point

**Recommended pattern**:
- Seed deterministic factories from a stable identifier such as `module_path!()`
- Keep `uselesskey` in `dev-dependencies`
- Fail tests if new committed secret-shaped fixture files appear in the repo

The CLI integration tests include a guard for committed PEM/JWK-style fixtures and a smoke test that exercises deterministic RSA, token, JWK, and X.509 generation.

---

## Determinism Tests

Add a determinism test that runs the same ingest twice and asserts:

- JSON bytes identical
- Markdown bytes identical

This prevents "churny" PR comments.

**Location**: [`crates/lintdiff-ingest-core/tests/determinism.rs`](../crates/lintdiff-ingest-core/tests/determinism.rs)

**Snapshot files**:
- [`fail_verdict.json`](../crates/lintdiff-ingest-core/tests/snapshots/fail_verdict.json)
- [`pass_verdict.json`](../crates/lintdiff-ingest-core/tests/snapshots/pass_verdict.json)
- [`warn_verdict.json`](../crates/lintdiff-ingest-core/tests/snapshots/warn_verdict.json)
