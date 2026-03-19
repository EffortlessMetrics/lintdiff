# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Crates.io publication readiness metadata

### Changed
- Documentation updates for public API

### Deprecated
- `lintdiff-ingest` façade crate: Use `lintdiff-ingest-core` instead
- `lintdiff-core` façade crate: Use `lintdiff-ingest-core` instead
- `lintdiff-domain` façade crate: Use `lintdiff-ingest-core` instead

  All three façade crates now emit deprecation warnings at compile time.
  They will be maintained for backward compatibility until v1.0.0.
  
  **Migration Guide**: See the [deprecation plan](docs/deprecation-plan.md) for:
  - Import path transformations (e.g., `lintdiff_domain::Something` → `lintdiff_ingest_core::Something`)
  - Timeline and version requirements
  - Migration examples and tooling

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
