# Contributing to lintdiff

Thank you for your interest in contributing to lintdiff! This document provides guidelines and instructions to help you contribute effectively.

## Table of Contents

- [Introduction](#introduction)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [PR Review Process](#pr-review-process)

## Introduction

lintdiff is a diff-scoped filter for Rust compiler and Clippy diagnostics. It answers the question: _"Did this change introduce actionable diagnostics on changed lines?"_

The project is organized as a Cargo workspace with multiple crates, each with specific responsibilities:

| Crate | Purpose |
|-------|---------|
| `lintdiff-types` | DTOs, config model, schema IDs, normalization helpers |
| `lintdiff-diff` | Unified diff parsing → changed ranges (new-side) |
| `lintdiff-diagnostics` | Cargo JSON parsing → normalized diagnostics |
| `lintdiff-match` | Path/span matching primitives |
| `lintdiff-policy` | Code normalization, allow/suppress/deny, verdict, fingerprinting |
| `lintdiff-fingerprint` | Stable fingerprint generation for findings |
| `lintdiff-ingest-core` | Core ingest pipeline |
| `lintdiff-render` | Markdown + GitHub annotations renderers |
| `lintdiff-app` | Orchestration layer |
| `lintdiff-app-git` | Git adapter (diff acquisition, repo root) |
| `lintdiff-app-io` | I/O adapter (config loading, diagnostics reading) |
| `lintdiff-cli` | CLI surface (`lintdiff` binary) |
| `lintdiff-bdd-*` | BDD testing infrastructure |

## Getting Started

### Prerequisites

- **Rust 1.92+** (see `rust-version` in `Cargo.toml`)
- **git** for version control

### Building the Project

```bash
# Clone the repository
git clone https://github.com/effortless-metrics/lintdiff.git
cd lintdiff

# Build all workspace crates
cargo build --workspace

# Build with all features enabled
cargo build --workspace --all-features
```

### Running Tests

```bash
# Run all tests across the workspace
cargo test --workspace

# Run tests with all features enabled
cargo test --workspace --all-features

# Run tests for a specific crate
cargo test -p lintdiff-ingest-core

# Run a specific test
cargo test -p lintdiff-ingest-core test_name
```

## Development Workflow

### Creating a Branch

1. Fork the repository (if you don't have write access)
2. Create a feature branch from `main`:

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### Making Changes

- Make focused, atomic commits
- Write clear commit messages (see [Commit Message Format](#commit-message-format))
- Add or update tests as needed
- Update documentation for user-facing changes

### Running Tests Locally

Before submitting a PR, ensure all checks pass:

```bash
# Format check
cargo fmt --all -- --check

# Run clippy (must pass with no warnings)
cargo clippy --all-features -- -D warnings

# Run all tests
cargo test --workspace --all-features

# Run BDD tests specifically
cargo test -p lintdiff --test bdd
```

### Submitting a PR

1. Push your branch to your fork or the main repository
2. Open a Pull Request against the `main` branch
3. Fill out the PR template (if applicable)
4. Ensure all CI checks pass
5. Request review from maintainers

## Code Style

### Rust Formatting

We use `rustfmt` for consistent code formatting. Run before committing:

```bash
# Format all code
cargo fmt --all

# Check formatting without making changes
cargo fmt --all -- --check
```

### Clippy Lints

We enforce Clippy lints at the warning level. All code must pass Clippy without warnings:

```bash
# Run clippy with all features
cargo clippy --all-features -- -D warnings
```

The CI also runs Clippy with pedantic lints enabled:

```bash
# Optional: Check pedantic lints locally
cargo clippy --all-features -- -W clippy::all -W clippy::pedantic
```

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools

**Examples:**
```
feat(match): add support for multi-span diagnostics
fix(diff): handle empty diffs correctly
docs(readme): update installation instructions
test(policy): add property-based tests for fingerprint
```

## Testing Guidelines

lintdiff has a comprehensive test suite with multiple layers:

### Unit Tests

Unit tests are embedded in each crate's `src/lib.rs` file using `#[cfg(test)]` modules.

**Location**: `crates/<crate>/src/lib.rs` (inline `tests` module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

### Integration Tests

Integration tests are located in each crate's `tests/` directory.

**Location**: `crates/<crate>/tests/*.rs`

```bash
# Run integration tests for a specific crate
cargo test -p lintdiff-diff --test diff_parsing
```

### BDD Tests

Behavior-driven tests use Cucumber-style "Given/When/Then" scenarios.

**Location**: `crates/lintdiff-cli/tests/features/*.feature`

```bash
# Run BDD tests
cargo test -p lintdiff --test bdd
```

### Property-based Tests

We use `proptest` for property-based testing of algorithms:

```bash
# Property tests are included in normal test runs
cargo test -p lintdiff-fingerprint
```

### Test Fixtures

Test fixtures are used for integration and golden tests:

- **Inputs**: `tests/fixtures/*.diff`, `tests/fixtures/*.jsonl`
- **Expected outputs**: `tests/fixtures/expected/*.report.json`

When adding new functionality, consider adding fixture-based tests to ensure deterministic behavior.

## Documentation

### Code Documentation

Use doc comments (`///` or `//!`) for public APIs:

```rust
/// Parses a unified diff and extracts changed line ranges.
///
/// # Arguments
///
/// * `input` - The unified diff content as a string
///
/// # Returns
///
/// A vector of `Hunk` objects representing changed regions.
///
/// # Errors
///
/// Returns an error if the diff format is invalid.
pub fn parse_diff(input: &str) -> Result<Vec<Hunk>, DiffError> {
    // ...
}
```

### README Updates

Update the README.md when:
- Adding new user-facing features
- Changing command-line interface
- Modifying configuration options
- Updating installation instructions

## PR Review Process

### What to Expect

1. **Automated CI checks** run on all PRs:
   - `cargo fmt --all -- --check` (formatting)
   - `cargo test --all-features` (tests)
   - `cargo clippy --all-features -- -D warnings` (lints)
   - BDD tests

2. **Code review** by maintainers:
   - Reviewers will provide feedback on code quality, design, and test coverage
   - Address review comments by pushing new commits
   - Mark conversations as resolved when addressed

3. **Approval required** before merging:
   - At least one maintainer approval is required
   - All CI checks must pass

### CI Checks

The following CI workflows run on every PR:

| Check | Description |
|-------|-------------|
| `test` | Format check, all tests, clippy with `-D warnings` |
| `lint` | Clippy with pedantic lints (informational) |
| `bdd` | BDD/Cucumber tests |

### Merging

Once approved and all checks pass:
- Maintainers will squash and merge your PR
- Your commits will be combined into a single commit on `main`

## Questions?

If you have questions about contributing, feel free to:
- Open an issue for discussion
- Check existing documentation in the `docs/` directory

Thank you for contributing to lintdiff!
