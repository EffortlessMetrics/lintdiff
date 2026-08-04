# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-08-03

### Added

- Shipped experimental `lintdiff inventory` and `lintdiff compare` commands
  with the `lintdiff.inventory.v1` and `lintdiff.delta.v1` evidence protocols.
  These commands are advisory research surfaces and are not part of the
  maintained blocking contract.

### Changed

- Hardened the exact-tag GitHub Action installation contract: release versions
  are resolved from the invoked tag, checksums are required before execution,
  root-level archives are extracted correctly, and command arguments are passed
  without shell evaluation.
- Preserved complete upstream Cargo evidence in `lintdiff.report.v1`, including
  the exact process exit code, `build-finished` presence, build success, and
  completeness state. Reports are retained when the upstream build fails.
- Corrected repository-path identity so real directories named `a/` or `b/`
  are preserved while Git diff transport prefixes are removed only when earned;
  quoted/spaced paths and rename records are covered by regression tests.
- Added executable Action proof across Linux x86_64, macOS x86_64, macOS arm64,
  and Windows x86_64, including report schema, output, checksum, and artifact
  checks.

### Documentation

- Clarified the narrow changed-line receipt product and its exact-tag support
  boundary.
- Documented that diagnostic delta is shipped as an experimental, advisory
  surface; `cargo install` and public engine-crate support remain unsupported.

## [1.0.0] - 2026-03-25

### Summary

This is the first stable release of lintdiff. The API is now considered stable and
semantic versioning guarantees apply. This release removes the deprecated façade
crates that were announced in v0.2.0.

### Breaking Changes

**Façade Crates Removed**

The following deprecated façade crates have been **removed** from the workspace:

- `lintdiff-domain` - Removed; use `lintdiff-ingest-core` instead
- `lintdiff-core` - Removed; use `lintdiff-ingest-core` instead
- `lintdiff-ingest` - Removed; use `lintdiff-ingest-core` instead

If you were using any of these crates, you **must** migrate to `lintdiff-ingest-core`
before upgrading to v1.0.0. See the [Migration Guide](docs/migration-guide.md) for
detailed instructions.

**Migration is straightforward:**

```diff
[dependencies]
- lintdiff-domain = "0.4"
+ lintdiff-ingest-core = "1.0"
```

```diff
- use lintdiff_domain::IngestPipeline;
+ use lintdiff_ingest_core::IngestPipeline;
```

All public APIs from the deprecated crates are available in `lintdiff-ingest-core`
with identical functionality.

### What's Changed

- Removed `lintdiff-domain` façade crate
- Removed `lintdiff-core` façade crate
- Removed `lintdiff-ingest` façade crate
- Updated all documentation to reflect the simplified crate structure
- Marked EPIC-001 (Façade Deprecation) as complete

### Migration Resources

- **[Migration Guide](docs/migration-guide.md)** - Step-by-step migration instructions
- **[Automated Migration Script](scripts/migrate-to-ingest-core.sh)** - Automate the migration
- **[Migration Examples](docs/examples/migration-example.md)** - Code examples

## [0.4.1] - 2026-03-25

### Internal
- Migrated `lintdiff-app` from deprecated `lintdiff-domain` façade to `lintdiff-ingest-core` (PR-101)
- Migrated `lintdiff-bdd-harness` from deprecated `lintdiff-core` façade to `lintdiff-ingest-core` (PR-102)
- Verified zero internal usage of deprecated façade crates across the entire codebase (PR-109)
- All 155 BDD scenarios pass with the new internal dependencies (PR-105)

### Added
- Automated migration script at `scripts/migrate-to-ingest-core.sh` for external users to update their codebases to use `lintdiff-ingest-core` directly (PR-106)

## [0.4.0] - 2026-03-17

### Added
- Full BDD test coverage with 200 Gherkin scenarios across all features
- Complete 10-phase development roadmap execution
- Comprehensive test suite with 1,207+ tests
- CI/CD infrastructure with 8 GitHub Actions workflows
- BDD test grid for parameterized scenario testing
- BDD test harness for standardized test execution

### Documentation
- Complete architecture documentation
- ADR (Architecture Decision Records) for key design decisions
- Release process documentation
- Deprecation plan for future versions

## [0.3.0] - 2026-03-17

### Added
- Code coverage reporting with codecov integration
- Performance benchmarks for core operations
  - Diagnostics parsing benchmarks
  - Diff parsing benchmarks
  - Fingerprint computation benchmarks
- API stability checking with semver verification
- i18n (internationalization) preparation infrastructure
  - Fluent localization framework integration
  - Locale files for en-US (cli.ftl, errors.ftl, main.ftl, report.ftl)
  - Message extraction and translation support
- Advanced fuzzing infrastructure
  - Diagnostics parser fuzzer
  - Diff parser fuzzer
  - Finding fingerprint fuzzer
- Property-based testing for core algorithms
- Schema validation tests for JSON report structures

### Changed
- Improved test organization with dedicated test crates
- Enhanced CI pipeline with coverage and benchmark reporting

## [0.1.0] - 2026-03-16

### Added
- Initial release of lintdiff CLI tool
- Core subcommands: `ingest`, `run`, `md`, `annotations`, `explain`, `ci github`
- Diff parsing with unified diff format support
- Diagnostics parsing from cargo JSON streams
- Span matching against changed lines
- Policy engine with fail_on, allow/suppress/deny lists
- Report generation with JSON schema validation
- Markdown and GitHub annotations renderers
- Feature flags: `primary_span_matching`, `path_filters`
- BDD test framework with Gherkin scenarios
- Fuzz targets for diff parser, diagnostics parser, and fingerprinting
- GitHub Action for CI integration (`action.yml`)
- JSON schemas for report and receipt envelope

### Documentation
- Architecture documentation
- Design principles and hexagonal architecture
- Implementation plan with phase tracking
- Requirements specification
- Testing strategy
- Feature flags guide
