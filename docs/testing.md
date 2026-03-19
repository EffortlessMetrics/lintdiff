# lintdiff Test Strategy

lintdiff is a gatekeeper-style tool. The "architecture" is mostly the test discipline.

## Test Coverage Summary

### Test Counts by Crate

| Crate | Unit Tests | Integration Tests | Property Tests | Doc Tests | Total |
|-------|------------|-------------------|----------------|-----------|-------|
| lintdiff-diff | 3 | 106 | 19 | 1 | **110** |
| lintdiff-diagnostics | 1 | 109 | - | 6 | **116** |
| lintdiff-match | 14 | 115 | - | 6 | **135** |
| lintdiff-policy | 15 | 143 | - | 3 | **161** |
| lintdiff-fingerprint | 4 | 65 | 18 | 5 | **92** |
| lintdiff-render | 6 | 119 | - | 5 | **130** |
| lintdiff-app-git | - | 34 | - | - | **34** |
| lintdiff-ingest-core | 11 | 4 | - | - | **15** |
| lintdiff-types | - | 1 | - | - | **1** |
| lintdiff-feature-flags | 6 | - | - | - | **6** |
| lintdiff-bdd-grid | 4 | - | - | - | **4** |
| lintdiff-cli | - | 3 | - | - | **3** |
| **Total** | **64** | **699** | **37** | **26** | **807** |

### Test Categories

| Category | Count | Description |
|----------|-------|-------------|
| Unit Tests | 64 | Fast, isolated tests for individual functions |
| Integration Tests | 699 | Tests that verify component interactions |
| Property-based Tests | 37 | Proptest-driven tests for algorithmic correctness |
| Doc Tests | 26 | Inline documentation examples |
| BDD Scenarios | 6 | Behavior-driven tests for user-facing features |
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
- [`lintdiff-diff`](../crates/lintdiff-diff/src/lib.rs) - Range merging, line parsing
- [`lintdiff-policy`](../crates/lintdiff-policy/src/lib.rs) - Code normalization, verdict logic
- [`lintdiff-fingerprint`](../crates/lintdiff-fingerprint/src/lib.rs) - Fingerprint generation

### Layer 2: Integration Tests

Integration tests verify that components work correctly together. They are located in each crate's `tests/` directory.

**Location**: `crates/<crate>/tests/*.rs`

**Test files by crate**:

| Crate | Test Files | Tests |
|-------|------------|-------|
| lintdiff-diff | `diff_parsing.rs`, `hunk_handling.rs`, `path_normalization.rs` | 87 |
| lintdiff-diagnostics | `json_parsing.rs`, `span_extraction.rs`, `stream_handling.rs` | 109 |
| lintdiff-match | `filter_tests.rs`, `path_tests.rs`, `span_tests.rs` | 115 |
| lintdiff-policy | `code_tests.rs`, `fingerprint_tests.rs`, `verdict_tests.rs` | 143 |
| lintdiff-fingerprint | `fingerprint_tests.rs`, `stability_tests.rs` | 65 |
| lintdiff-render | `annotations_tests.rs`, `budget_tests.rs`, `markdown_tests.rs` | 119 |
| lintdiff-app-git | `integration_tests.rs` | 34 |
| lintdiff-ingest-core | `determinism.rs`, `fingerprint_integration.rs` | 4 |

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
- [`lintdiff-bdd`](../crates/lintdiff-bdd/) - BDD types and utilities
- [`lintdiff-bdd-harness`](../crates/lintdiff-bdd-harness/) - Test harness
- [`lintdiff-bdd-grid`](../crates/lintdiff-bdd-grid/) - Feature flag matrix testing

BDD is intentionally thin; golden fixtures carry the detailed surface contract.

### Layer 4: Property-based Tests

Property tests protect core algorithms using [`proptest`](https://proptest-rs.github.io/proptest/).

**Location**: `crates/<crate>/tests/property_tests.rs`

**Crates with property tests**:
- [`lintdiff-diff`](../crates/lintdiff-diff/tests/property_tests.rs) - 19 tests
  - Diff parsing invariants
  - Path normalization idempotence
  - Line range ordering
- [`lintdiff-fingerprint`](../crates/lintdiff-fingerprint/tests/property_tests.rs) - 18 tests
  - Fingerprint determinism
  - Whitespace normalization
  - Collision resistance

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
cargo test -p lintdiff-diff --lib

# Integration tests for a specific crate
cargo test -p lintdiff-diff --test diff_parsing

# Property tests
cargo test -p lintdiff-diff --test property_tests
cargo test -p lintdiff-fingerprint --test property_tests

# BDD tests
cargo test -p lintdiff --test bdd

# Doc tests only
cargo test --workspace --doc
```

### Run Specific Tests

```bash
# Run tests matching a pattern
cargo test -p lintdiff-diff --test diff_parsing -- parses_simple

# Run a single test
cargo test -p lintdiff-policy --test verdict_tests -- fail_on_error_with_errors_is_fail
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
cargo mutants -p lintdiff-diff
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
