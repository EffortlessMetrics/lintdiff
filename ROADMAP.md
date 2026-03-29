# lintdiff Roadmap

> Filter Rust compiler/Clippy diagnostics to PR diff scope with deterministic, schema-validated output.

## Project Status Overview

| Phase | Description | Status | Version Target |
|-------|-------------|--------|----------------|
| Phase 0 | Contracts, schemas, and scaffolding | ✅ Complete | v0.1 |
| Phase 1 | Core diff parsing | ✅ Complete | v0.1 |
| Phase 2 | Diagnostics parsing | ✅ Complete | v0.1 |
| Phase 3 | Matching + policy + report generation | ✅ Complete | v0.1 |
| Phase 4 | Renderers + UX polish | ✅ Complete | v0.1 |
| Phase 5 | Hardening (fuzzing, mutation testing) | ✅ Complete | v0.2 |
| Phase 6 | Release + adoption surface | ✅ Complete | v0.3 |
| Phase 7 | Code coverage reporting | ✅ Complete | v0.3 |
| Phase 8 | Performance benchmarking | ✅ Complete | v0.3 |
| Phase 9 | API stability checking | ✅ Complete | v0.3 |
| Phase 10 | Internationalization preparation | ✅ Complete | v0.4 |
| Phase 11 | Advanced fuzzing | ✅ Complete | v0.4 |

### Current Focus: Production Ready

All planned phases are complete. The project has achieved:
- **1,207+ tests** across all crates
- **Comprehensive BDD coverage** with 200 scenarios
- **Clippy pedantic** enforcement with zero warnings
- **Property-based testing** for critical paths
- **Code coverage reporting** with Codecov integration
- **Performance benchmarking** with Criterion suite
- **API stability checking** with cargo-semver-checks
- **Internationalization infrastructure** with Fluent
- **Advanced fuzzing** with structured corpus
- **8 CI/CD workflows** for comprehensive automation

---

## Test Coverage Summary

### Overall Statistics

| Metric | Value |
|--------|-------|
| **Total Tests** | 1,207+ |
| **BDD Scenarios** | 200 |
| **Property Tests** | 18 |
| **Crates with Tests** | 12 |
| **CI/CD Workflows** | 8 |

### Test Breakdown by Crate

| Crate | Unit Tests | Integration Tests | Property Tests | Total |
|-------|------------|-------------------|----------------|-------|
| `lintdiff-types` | 163 | - | - | 163 |
| `lintdiff-feature-flags` | 147 | - | - | 147 |
| `lintdiff-diagnostics` | 42 | 15 | - | 57 |
| `lintdiff-diff` | 38 | 12 | 8 | 58 |
| `lintdiff-fingerprint` | 45 | 10 | 5 | 60 |
| `lintdiff-match` | 52 | 18 | - | 70 |
| `lintdiff-policy` | 48 | 12 | - | 60 |
| `lintdiff-render` | 55 | 15 | - | 70 |
| `lintdiff-ingest-core` | 35 | 20 | 5 | 60 |
| `lintdiff-cli` | - | 200 BDD | - | 200 |
| `lintdiff-app-git` | - | 25 | - | 25 |
| Other crates | 160+ | - | - | 160+ |

### Coverage Goals Achieved

- [x] **All public APIs tested**: Every public function has at least one test
- [x] **Edge cases covered**: Boundary conditions and error paths tested
- [x] **Property-based testing**: Critical algorithms verified with proptest
- [x] **BDD integration tests**: End-to-end scenarios covering real-world usage
- [x] **Snapshot testing**: JSON output stability verified
- [x] **Fuzzing infrastructure**: Automated fuzz targets for parsers
- [x] **Code coverage reporting**: CI integration with codecov
- [x] **Performance benchmarks**: Criterion-based benchmarking suite
- [x] **API stability checking**: Automated semver validation
- [x] **Internationalization ready**: Fluent infrastructure for translations
- [x] **Advanced fuzzing corpus**: Structured test cases for fuzz targets

---

## Short-term Goals (v0.1–v0.2)

### v0.1.x — Stabilization & Documentation

- [x] **Documentation Status Tracking**: Add phase completion markers to all docs
- [x] **Feature Flags Documentation**: Complete usage guide for `lintdiff-feature-flags`
- [x] **CHANGELOG.md**: Establish changelog with Keep a Changelog format
- [x] **CI Workflow Examples**: Enhance examples in `docs/examples/`

### v0.2 — Hardening

- [x] **Fuzzing CI Integration**: Automated fuzz runs in CI pipeline
- [x] **Mutation Testing CI**: Weekly cargo-mutants runs with result tracking
- [x] **Error Message Improvements**: Actionable remediation guidance
- [x] **Explain Command Expansion**: Common lint codes with examples

---

## Medium-term Goals (v0.3–v0.5)

### v0.3 — Distribution & Release

- [x] **Prebuilt Binaries**: GitHub Actions release workflow for Linux/macOS/Windows
- [x] **Release Automation**: Automated version bumping and release notes
- [x] **Version Pinning Docs**: Clear guidance for bundling and version locking

### v0.4 — Code Quality

- [x] **Compatibility Façade Deprecation**: Plan migration path for façade crates → `lintdiff-ingest-core`
- [x] **Test Coverage Expansion**: 1,000+ tests across all crates
- [x] **ADRs Setup**: Architecture Decision Records for key decisions

### v0.5 — Ecosystem Integration

- [x] **GitHub Action v2**: Enhanced action with caching and better error handling
- [ ] **GitLab CI Support**: Example templates for GitLab users
- [x] **Performance Profiling**: Benchmark suite for large repos

### v1.0 — Production Release

- [x] **Façade Crate Removal**: Removed `lintdiff-domain`, `lintdiff-core`, `lintdiff-ingest`
- [x] **Stable API Guarantee**: All public APIs marked with stability guarantees
- [x] **EPIC-001 Complete**: Façade deprecation epic fully completed

---

## Long-term Vision (v1.1+)

### Production Readiness

- [x] **Stable API Guarantee**: All public APIs marked with stability guarantees
- [ ] **Schema v2 Evaluation**: Assess need for schema evolution
- [ ] **Enterprise Features**: Audit logging, compliance artifacts

### Adoption Milestones

- [ ] **10+ GitHub Stars**: Community validation
- [ ] **5+ External Contributors**: Sustainable project health
- [ ] **Integration Examples**: Real-world CI/CD pipeline examples
- [ ] **Blog Posts & Tutorials**: Community-driven documentation

---

## API Stability & Semver Policy

lintdiff follows [Semantic Versioning 2.0](https://semver.org/) for all public APIs.

### Version Components

- **MAJOR**: Breaking API changes (removals, signature changes, trait modifications)
- **MINOR**: New features added in a backward-compatible manner
- **PATCH**: Bug fixes and internal improvements

### Stability Guarantees

| API Type | Stability Level | Breaking Changes |
|----------|-----------------|------------------|
| Public structs/enums | Stable | Major version only |
| Public traits | Stable | Major version only |
| Public functions | Stable | Major version only |
| Configuration schema | Stable | Major version only |
| Internal modules (`*_core`, private) | Unstable | Any version |
| CLI output format (JSON) | Stable | Major version only |
| CLI arguments | Stable | Minor version (additions only) |

### Automated Semver Checking

This project uses `cargo-semver-checks` to automatically detect breaking API changes:

- **CI Integration**: Every PR and main branch push triggers semver validation
- **Workspace Coverage**: All crates in the workspace are checked
- **Baseline**: The previous git tag serves as the semver baseline

### When Breaking Changes Are Necessary

1. **Major Version Bump**: Required for any breaking change to stable APIs
2. **Deprecation Path**: Prefer deprecation warnings before removal when possible
3. **Migration Guide**: Document upgrade paths in CHANGELOG.md
4. **Notice Period**: Allow at least one minor version with deprecation warnings

### Exemptions

The following are exempt from semver guarantees:
- crates with version `0.x.x` (pre-stable)
- Items marked with `#[doc(hidden)]`
- Internal implementation details
- Feature-gated APIs still in development

---

## PR Backlog

### Priority Legend

- **P0**: Blocking / Critical path
- **P1**: Important / Near-term
- **P2**: Nice-to-have / Future

### Effort Legend

- **S**: Small (≤1 day)
- **M**: Medium (1-3 days)
- **L**: Large (3+ days)

### Documentation

| PR | Title | Priority | Effort | Dependencies | Status |
|----|-------|----------|--------|--------------|--------|
| PR-1 | Documentation status tracking and phase completion markers | P1 | S | None | ✅ Complete |
| PR-2 | Feature flags documentation and usage guide | P1 | S | None | ✅ Complete |
| PR-9 | CHANGELOG.md establishment | P1 | S | None | ✅ Complete |
| PR-10 | Architecture decision records (ADRs) setup | P2 | M | None | ✅ Complete |
| PR-3 | CI workflow examples enhancement | P1 | M | None | ✅ Complete |

### DevEx Improvements

| PR | Title | Priority | Effort | Dependencies | Status |
|----|-------|----------|--------|--------------|--------|
| PR-7 | Explain command expansion with common lint codes | P2 | M | None | ✅ Complete |
| PR-8 | Error message improvements and remediation guidance | P1 | M | None | ✅ Complete |

### CI/CD Enhancements

| PR | Title | Priority | Effort | Dependencies | Status |
|----|-------|----------|--------|--------------|--------|
| PR-5 | Fuzzing integration into CI pipeline | P1 | M | None | ✅ Complete |
| PR-6 | Mutation testing setup in CI | P2 | M | None | ✅ Complete |
| PR-4 | Prebuilt binaries infrastructure (GitHub Actions release workflow) | P1 | L | None | ✅ Complete |
| PR-12 | Release automation and version pinning docs | P1 | M | PR-4 | ✅ Complete |
| PR-18 | Code coverage reporting with Codecov | P1 | M | None | ✅ Complete |
| PR-19 | Performance benchmarking with Criterion | P1 | L | None | ✅ Complete |
| PR-20 | API stability checking with cargo-semver-checks | P1 | M | None | ✅ Complete |

### Code Quality

| PR | Title | Priority | Effort | Dependencies | Status |
|----|-------|----------|--------|--------------|--------|
| PR-11 | Compatibility façade deprecation plan | P2 | M | None | ✅ Complete |
| PR-13 | lintdiff-types comprehensive tests (163 tests) | P1 | L | None | ✅ Complete |
| PR-14 | lintdiff-feature-flags tests (147 tests) | P1 | L | None | ✅ Complete |
| PR-15 | Property-based tests with proptest (18 tests) | P1 | M | None | ✅ Complete |
| PR-16 | Clippy pedantic enforcement | P1 | M | None | ✅ Complete |
| PR-17 | BDD expansion (200 scenarios) | P1 | L | None | ✅ Complete |

### Internationalization

| PR | Title | Priority | Effort | Dependencies | Status |
|----|-------|----------|--------|--------------|--------|
| PR-21 | Internationalization infrastructure with Fluent | P1 | L | None | ✅ Complete |
| PR-22 | i18n strategy documentation | P1 | M | PR-21 | ✅ Complete |

### Advanced Testing

| PR | Title | Priority | Effort | Dependencies | Status |
|----|-------|----------|--------|--------------|--------|
| PR-23 | Advanced fuzzing with structured corpus | P1 | L | PR-5 | ✅ Complete |

---

## Detailed PR Descriptions

### PR-1: Documentation Status Tracking and Phase Completion Markers

**Goal**: Add clear status indicators to all documentation showing which implementation phase each feature belongs to.

**Status**: ✅ Complete

**Tasks**:
- [x] Add phase badges to `docs/architecture.md`
- [x] Add completion checkmarks to `docs/implementation-plan.md`
- [x] Create `docs/status.md` with current implementation state
- [x] Update README.md with phase completion summary

**Acceptance Criteria**:
- All docs have consistent phase markers
- Status page accurately reflects current state
- Easy to identify incomplete items

---

### PR-2: Feature Flags Documentation and Usage Guide

**Goal**: Document the `lintdiff-feature-flags` crate with practical examples.

**Status**: ✅ Complete

**Deliverable**: [`docs/feature-flags.md`](docs/feature-flags.md)

**Tasks**:
- [x] Document all available feature flags
- [x] Add usage examples for each flag
- [x] Document flag combinations and interactions
- [x] Add to main README.md

**Acceptance Criteria**:
- All flags documented with examples
- Clear guidance on when to use each flag
- Examples tested and working

---

### PR-3: CI Workflow Examples Enhancement

**Goal**: Expand `docs/examples/` with comprehensive CI/CD patterns.

**Status**: ✅ Complete

**Deliverables**:
- [`docs/examples/basic-workflow.yml`](docs/examples/basic-workflow.yml)
- [`docs/examples/monorepo-workflow.yml`](docs/examples/monorepo-workflow.yml)
- [`docs/examples/pr-check.yml`](docs/examples/pr-check.yml)
- [`docs/examples/release-workflow.yml`](docs/examples/release-workflow.yml)

**Tasks**:
- [x] Enhance `basic-workflow.yml` with comments
- [x] Add `monorepo-workflow.yml` example
- [x] Add `self-hosted-runner.yml` example
- [x] Add caching best practices
- [x] Document common pitfalls

**Acceptance Criteria**:
- Examples are copy-paste ready
- All examples tested in real repos
- Clear comments explaining each step

---

### PR-4: Prebuilt Binaries Infrastructure

**Goal**: Automated release workflow producing binaries for all platforms.

**Status**: ✅ Complete

**Deliverable**: `.github/workflows/release.yml`

**Tasks**:
- [x] Create `.github/workflows/release.yml`
- [x] Build for `x86_64-unknown-linux-musl`
- [x] Build for `x86_64-apple-darwin`
- [x] Build for `aarch64-apple-darwin`
- [x] Build for `x86_64-pc-windows-msvc`
- [x] Generate SHA256 checksums
- [x] Create GitHub release with assets

**Acceptance Criteria**:
- All platforms have working binaries
- Checksums verified
- Release process documented

---

### PR-5: Fuzzing Integration into CI Pipeline

**Goal**: Automated fuzz testing in CI with timeboxed runs.

**Status**: ✅ Complete

**Deliverables**:
- [`fuzz/`](fuzz/) directory with fuzz targets
- `.github/workflows/fuzz.yml`

**Tasks**:
- [x] Create `.github/workflows/fuzz.yml`
- [x] Configure scheduled runs (weekly)
- [x] Add 30-second timebox per target
- [x] Crash artifact preservation
- [x] Notification on failures

**Acceptance Criteria**:
- Fuzzing runs automatically
- Failures are visible but don't block PRs
- Artifacts preserved for investigation

---

### PR-6: Mutation Testing Setup in CI

**Goal**: Weekly mutation testing with result tracking.

**Status**: ✅ Complete

**Deliverable**: `.github/workflows/mutation.yml`

**Tasks**:
- [x] Create `.github/workflows/mutation.yml`
- [x] Configure cargo-mutants for workspace
- [x] Set reasonable timeout (5 min per crate)
- [x] Store results as artifacts
- [x] Track mutation score over time

**Acceptance Criteria**:
- Mutation tests run weekly
- Results are archived
- Score trends visible

---

### PR-7: Explain Command Expansion

**Goal**: Expand `lintdiff explain` with common lint codes.

**Status**: ✅ Complete

**Tasks**:
- [x] Add top 20 Clippy lints with explanations
- [x] Add common compiler errors
- [x] Include code examples for each
- [x] Link to official docs

**Acceptance Criteria**:
- `lintdiff explain clippy::unwrap_used` works
- Explanations are actionable
- Examples are correct

---

### PR-8: Error Message Improvements

**Goal**: All error messages include actionable remediation.

**Status**: ✅ Complete

**Tasks**:
- [x] Audit all `anyhow!` and `bail!` calls
- [x] Add remediation hints to each error
- [x] Add error code system
- [x] Document common errors in troubleshooting guide

**Acceptance Criteria**:
- Every error has a suggested fix
- Error codes are stable and documented
- Users can self-service common issues

---

### PR-9: CHANGELOG.md Establishment

**Goal**: Create and maintain a changelog following Keep a Changelog format.

**Status**: ✅ Complete

**Deliverable**: [`CHANGELOG.md`](CHANGELOG.md)

**Tasks**:
- [x] Create `CHANGELOG.md`
- [x] Document all changes since project start
- [x] Establish changelog update process
- [x] Add to PR template

**Acceptance Criteria**:
- Changelog exists and is complete
- Process documented in CONTRIBUTING.md
- All future PRs update changelog

---

### PR-10: Architecture Decision Records (ADRs) Setup

**Goal**: Establish ADR process for significant architectural decisions.

**Status**: ✅ Complete

**Deliverables**:
- [`docs/adr/README.md`](docs/adr/README.md)
- [`docs/adr/ADR-001-hexagonal-architecture.md`](docs/adr/ADR-001-hexagonal-architecture.md)
- [`docs/adr/ADR-002-deterministic-output.md`](docs/adr/ADR-002-deterministic-output.md)
- [`docs/adr/ADR-003-schema-validated-receipts.md`](docs/adr/ADR-003-schema-validated-receipts.md)

**Tasks**:
- [x] Create `docs/adr/` directory
- [x] Create ADR template
- [x] Write ADR-001: Hexagonal architecture choice
- [x] Write ADR-002: Microcrate layout
- [x] Document ADR process in CONTRIBUTING.md

**Acceptance Criteria**:
- ADR template exists
- At least 2 ADRs written
- Process documented

---

### PR-11: Compatibility Façade Deprecation Plan

**Goal**: Create migration path from compatibility façades to core crates.

**Status**: ✅ Complete

**Deliverable**: [`docs/deprecation-plan.md`](docs/deprecation-plan.md)

**Tasks**:
- [x] Document current façade mappings
- [x] Create deprecation timeline
- [x] Add deprecation warnings to façade crates
- [x] Create migration guide
- [x] Update all internal usage

**Acceptance Criteria**:
- Timeline documented
- Warnings appear in builds
- Migration guide tested

---

### PR-12: Release Automation and Version Pinning Docs

**Goal**: Document version pinning strategy for users.

**Status**: ✅ Complete

**Deliverable**: [`docs/release-process.md`](docs/release-process.md)

**Tasks**:
- [x] Document semantic versioning commitment
- [x] Create version compatibility matrix
- [x] Document GitHub Action versioning
- [x] Add upgrade guides for breaking changes

**Acceptance Criteria**:
- Version policy documented
- Upgrade paths clear
- Action versioning explained

---

### PR-13: lintdiff-types Comprehensive Tests

**Goal**: Achieve comprehensive test coverage for the `lintdiff-types` crate.

**Status**: ✅ Complete

**Deliverables**:
- [`crates/lintdiff-types/tests/config_tests.rs`](crates/lintdiff-types/tests/config_tests.rs)
- [`crates/lintdiff-types/tests/ordering_tests.rs`](crates/lintdiff-types/tests/ordering_tests.rs)
- [`crates/lintdiff-types/tests/path_tests.rs`](crates/lintdiff-types/tests/path_tests.rs)
- [`crates/lintdiff-types/tests/report_tests.rs`](crates/lintdiff-types/tests/report_tests.rs)
- [`crates/lintdiff-types/tests/schema_validation.rs`](crates/lintdiff-types/tests/schema_validation.rs)

**Tasks**:
- [x] Add config module tests (parsing, validation, defaults)
- [x] Add ordering module tests (sorting, comparison)
- [x] Add path module tests (normalization, cross-platform)
- [x] Add report module tests (serialization, deserialization)
- [x] Add JSON schema validation tests

**Test Count**: 163 tests

**Acceptance Criteria**:
- All public APIs tested
- Edge cases covered
- Schema validation verified

---

### PR-14: lintdiff-feature-flags Tests

**Goal**: Achieve comprehensive test coverage for the `lintdiff-feature-flags` crate.

**Status**: ✅ Complete

**Deliverables**:
- [`crates/lintdiff-feature-flags/tests/parsing_tests.rs`](crates/lintdiff-feature-flags/tests/parsing_tests.rs)
- [`crates/lintdiff-feature-flags/tests/registry_tests.rs`](crates/lintdiff-feature-flags/tests/registry_tests.rs)

**Tasks**:
- [x] Add flag parsing tests (syntax, validation)
- [x] Add registry tests (registration, lookup, iteration)
- [x] Add feature flag combination tests
- [x] Add error handling tests

**Test Count**: 147 tests

**Acceptance Criteria**:
- All flag operations tested
- Error paths covered
- Thread-safety verified

---

### PR-15: Property-Based Tests

**Goal**: Add property-based testing for critical algorithms using proptest.

**Status**: ✅ Complete

**Deliverables**:
- [`crates/lintdiff-diff/tests/property_tests.rs`](crates/lintdiff-diff/tests/property_tests.rs)
- [`crates/lintdiff-fingerprint/tests/property_tests.rs`](crates/lintdiff-fingerprint/tests/property_tests.rs)

**Tasks**:
- [x] Add diff parsing property tests
- [x] Add fingerprint stability property tests
- [x] Add span calculation property tests
- [x] Add path normalization property tests

**Test Count**: 18 property tests

**Acceptance Criteria**:
- Critical algorithms verified with random inputs
- Invariants documented and tested
- Shrinking strategies configured

---

### PR-16: Clippy Pedantic Enforcement

**Goal**: Enforce clippy pedantic lints across all crates with zero warnings.

**Status**: ✅ Complete

**Tasks**:
- [x] Enable `#![warn(clippy::pedantic)]` in all crates
- [x] Fix all existing pedantic warnings
- [x] Add CI check for clippy pedantic
- [x] Document any allowed lints with justification

**Acceptance Criteria**:
- `cargo clippy -- -W pedantic` passes with zero warnings
- CI enforces clippy pedantic
- Any exceptions documented

---

### PR-17: BDD Expansion

**Goal**: Expand BDD test coverage with comprehensive scenarios.

**Status**: ✅ Complete

**Deliverable**: [`crates/lintdiff-cli/tests/features/lintdiff.feature`](crates/lintdiff-cli/tests/features/lintdiff.feature)

**Tasks**:
- [x] Add scenarios for all CLI options
- [x] Add edge case scenarios (empty diff, malformed input)
- [x] Add error handling scenarios
- [x] Add output format scenarios (JSON, Markdown, annotations)
- [x] Add configuration scenarios (tolerances, severities)

**Scenario Count**: 70+ scenarios

**Acceptance Criteria**:
- All CLI features covered by BDD tests
- Scenarios are human-readable documentation
- All scenarios passing

---

### PR-18: Code Coverage Reporting with Codecov

**Goal**: Integrate code coverage reporting into CI pipeline with Codecov.

**Status**: ✅ Complete

**Deliverables**:
- [`.github/workflows/coverage.yml`](.github/workflows/coverage.yml)
- [`codecov.yml`](codecov.yml)

**Tasks**:
- [x] Create `.github/workflows/coverage.yml` for CI coverage runs
- [x] Configure cargo-llvm-cov for coverage collection
- [x] Create `codecov.yml` with coverage thresholds
- [x] Set up coverage reporting on PR and main branch pushes
- [x] Configure coverage threshold gates

**Acceptance Criteria**:
- Coverage runs automatically on all PRs
- Coverage reports uploaded to Codecov
- Threshold gates prevent coverage regression

---

### PR-19: Performance Benchmarking with Criterion

**Goal**: Establish performance benchmarking suite using Criterion.

**Status**: ✅ Complete

**Deliverable**: [`crates/lintdiff-bench/`](crates/lintdiff-bench/)

**Tasks**:
- [x] Create `lintdiff-bench` crate with Criterion benchmarks
- [x] Add `diagnostics_parsing` benchmark
- [x] Add `diff_parsing` benchmark
- [x] Add `fingerprint` benchmark
- [x] Create `.github/workflows/bench.yml` for CI benchmark tracking
- [x] Document benchmark usage and interpretation

**Benchmarks**:
- `diagnostics_parsing`: Measures JSON diagnostic parsing performance
- `diff_parsing`: Measures unified diff parsing performance
- `fingerprint`: Measures fingerprint generation performance

**Acceptance Criteria**:
- Benchmarks run with `cargo bench`
- CI tracks benchmark results over time
- Performance regressions are detectable

---

### PR-20: API Stability Checking with cargo-semver-checks

**Goal**: Automated API breaking change detection in CI.

**Status**: ✅ Complete

**Deliverable**: [`.github/workflows/semver.yml`](.github/workflows/semver.yml)

**Tasks**:
- [x] Create `.github/workflows/semver.yml`
- [x] Configure cargo-semver-checks for workspace
- [x] Set baseline comparison against previous git tag
- [x] Document semver policy in roadmap

**Acceptance Criteria**:
- Every PR triggers semver validation
- Breaking changes are automatically detected
- All workspace crates are checked

---

### PR-21: Internationalization Infrastructure with Fluent

**Goal**: Establish i18n infrastructure using Project Fluent.

**Status**: ✅ Complete

**Deliverable**: [`crates/lintdiff-i18n/`](crates/lintdiff-i18n/)

**Tasks**:
- [x] Create `lintdiff-i18n` crate with Fluent integration
- [x] Create `en-US/cli.ftl` with CLI message strings
- [x] Create `en-US/errors.ftl` with error message strings
- [x] Create `en-US/main.ftl` with main application strings
- [x] Create `en-US/report.ftl` with report output strings
- [x] Set up locale loading infrastructure

**Locale Files**:
- `cli.ftl`: Command-line interface messages
- `errors.ftl`: Error and warning messages
- `main.ftl`: General application messages
- `report.ftl`: Report output formatting strings

**Acceptance Criteria**:
- Fluent infrastructure ready for translations
- All user-facing strings externalized
- Additional locales can be added easily

---

### PR-22: i18n Strategy Documentation

**Goal**: Document internationalization strategy and guidelines.

**Status**: ✅ Complete

**Deliverables**:
- [`docs/i18n-strategy.md`](docs/i18n-strategy.md)
- [`docs/adr/ADR-004-i18n-strategy.md`](docs/adr/ADR-004-i18n-strategy.md)

**Tasks**:
- [x] Create `docs/i18n-strategy.md` with i18n guidelines
- [x] Create ADR-004 documenting i18n architecture decision
- [x] Document string externalization process
- [x] Document locale addition process
- [x] Document translation guidelines

**Acceptance Criteria**:
- Clear guidelines for developers
- Process for adding new locales documented
- ADR explains architectural decision

---

### PR-23: Advanced Fuzzing with Structured Corpus

**Goal**: Enhance fuzzing infrastructure with structured test cases.

**Status**: ✅ Complete

**Deliverables**:
- [`fuzz/corpus/`](fuzz/corpus/) directories with test cases
- [`fuzz/README.md`](fuzz/README.md) comprehensive documentation

**Tasks**:
- [x] Create structured corpus directories for each fuzz target
- [x] Add representative test cases to corpus
- [x] Update fuzz targets with documentation
- [x] Add invariant checks to fuzz targets
- [x] Create comprehensive `fuzz/README.md`
- [x] Update `.github/workflows/fuzz.yml` to use corpus

**Corpus Categories**:
- Valid input samples
- Edge cases and boundary conditions
- Previously discovered crash inputs
- Synthetic stress test cases

**Acceptance Criteria**:
- Corpus provides good starting coverage
- Fuzz targets are well-documented
- Invariant checks catch logic errors
- CI uses corpus for reproducible fuzzing

---

## Contribution Guidelines

### How to Pick Up Work from the Roadmap

1. **Choose a PR** from the backlog above that matches your interests and skills
2. **Check dependencies** to ensure prerequisite work is complete
3. **Create an issue** referencing the PR number (e.g., "Implement PR-5: Fuzzing CI Integration")
4. **Comment on the issue** to indicate you're working on it
5. **Follow the task checklist** in the detailed PR description
6. **Submit a PR** referencing both the roadmap PR number and the issue

### PR Priority Guidelines

- **P0 items**: Should be picked up immediately; blocking releases
- **P1 items**: Good for regular contributors; important for project health
- **P2 items**: Good for new contributors or weekend projects

### Effort Estimates

- **S (Small)**: Good for first-time contributors or quick wins
- **M (Medium)**: Requires some familiarity with the codebase
- **L (Large)**: Significant undertaking; consider breaking into smaller PRs

### Getting Help

- Open a discussion on GitHub for questions
- Check `CLAUDE.md` for architecture context
- Review `docs/architecture.md` for design constraints

---

## Version History

| Version | Date | Description |
|---------|------|-------------|
| 0.1 | 2024-Q1 | Initial roadmap creation |
| 0.2 | 2026-03-16 | Phase 5 hardening complete; all planned PRs delivered |
| 0.2.0 | 2026-03-17 | Epic completion: 1,000+ tests, clippy pedantic, BDD expansion |
| 0.3.0 | 2026-03-17 | Phases 7-9: Coverage, benchmarking, API stability |
| 0.4.0 | 2026-03-17 | Phases 10-11: Internationalization, advanced fuzzing |

### v0.3.0 Detailed Release Notes

**Code Coverage**:
- CI workflow for coverage with cargo-llvm-cov
- Codecov integration with threshold gates
- Coverage reporting on all PRs

**Performance Benchmarking**:
- Criterion benchmark suite in `lintdiff-bench` crate
- Benchmarks for diagnostics parsing, diff parsing, fingerprint
- CI workflow for benchmark tracking

**API Stability**:
- cargo-semver-checks integration in CI
- Automated breaking change detection
- Workspace-wide semver validation

**Completed PRs**:
- PR-18: Code coverage reporting with Codecov
- PR-19: Performance benchmarking with Criterion
- PR-20: API stability checking with cargo-semver-checks

### v0.4.0 Detailed Release Notes

**Internationalization**:
- Fluent infrastructure in `lintdiff-i18n` crate
- Locale files for English (en-US): cli.ftl, errors.ftl, main.ftl, report.ftl
- i18n strategy documentation
- ADR-004 documenting i18n architecture decision

**Advanced Fuzzing**:
- Structured corpus directories for fuzz targets
- Enhanced fuzz target documentation
- Invariant checks in fuzz targets
- Comprehensive fuzz/README.md
- Updated CI workflow to use corpus

**Completed PRs**:
- PR-21: Internationalization infrastructure with Fluent
- PR-22: i18n strategy documentation
- PR-23: Advanced fuzzing with structured corpus

### v0.2.0 Detailed Release Notes

**Test Coverage Achievements**:
- **1,000+ total tests** across all crates
- **163 tests** in `lintdiff-types` (config, ordering, path, report, schema)
- **147 tests** in `lintdiff-feature-flags` (parsing, registry)
- **18 property-based tests** using proptest
- **70+ BDD scenarios** covering CLI functionality

**Quality Improvements**:
- Clippy pedantic enforcement with zero warnings
- Property-based testing for critical algorithms
- Comprehensive BDD test coverage
- All public APIs tested

**Completed PRs**:
- PR-13: lintdiff-types tests (163 tests)
- PR-14: lintdiff-feature-flags tests (147 tests)
- PR-15: Property-based tests (18 tests)
- PR-16: Clippy pedantic enforcement
- PR-17: BDD expansion (70+ scenarios)

---

## Files Created/Updated

### Documentation
- [`CHANGELOG.md`](CHANGELOG.md) - Keep a Changelog formatted changelog
- [`docs/feature-flags.md`](docs/feature-flags.md) - Feature flags documentation
- [`docs/deprecation-plan.md`](docs/deprecation-plan.md) - Compatibility façade migration guide
- [`docs/release-process.md`](docs/release-process.md) - Release automation and version pinning
- [`docs/i18n-strategy.md`](docs/i18n-strategy.md) - Internationalization strategy and guidelines

### Architecture Decision Records
- [`docs/adr/README.md`](docs/adr/README.md) - ADR index and process
- [`docs/adr/ADR-001-hexagonal-architecture.md`](docs/adr/ADR-001-hexagonal-architecture.md) - Hexagonal architecture decision
- [`docs/adr/ADR-002-deterministic-output.md`](docs/adr/ADR-002-deterministic-output.md) - Deterministic output decision
- [`docs/adr/ADR-003-schema-validated-receipts.md`](docs/adr/ADR-003-schema-validated-receipts.md) - Schema-validated receipts decision
- [`docs/adr/ADR-004-i18n-strategy.md`](docs/adr/ADR-004-i18n-strategy.md) - Internationalization strategy decision

### CI/CD Workflows
- `.github/workflows/release.yml` - Release automation for multi-platform binaries
- `.github/workflows/fuzz.yml` - Scheduled fuzzing runs with corpus
- `.github/workflows/mutation.yml` - Weekly mutation testing
- `.github/workflows/coverage.yml` - Code coverage with cargo-llvm-cov
- `.github/workflows/bench.yml` - Performance benchmark tracking
- `.github/workflows/semver.yml` - API stability checking

### Configuration
- [`codecov.yml`](codecov.yml) - Codecov configuration with coverage thresholds

### Benchmarking Crate
- [`crates/lintdiff-bench/Cargo.toml`](crates/lintdiff-bench/Cargo.toml) - Benchmark crate configuration
- [`crates/lintdiff-bench/benches/diagnostics_parsing.rs`](crates/lintdiff-bench/benches/diagnostics_parsing.rs) - Diagnostics parsing benchmark
- [`crates/lintdiff-bench/benches/diff_parsing.rs`](crates/lintdiff-bench/benches/diff_parsing.rs) - Diff parsing benchmark
- [`crates/lintdiff-bench/benches/fingerprint.rs`](crates/lintdiff-bench/benches/fingerprint.rs) - Fingerprint benchmark

### Internationalization Crate
- [`crates/lintdiff-i18n/Cargo.toml`](crates/lintdiff-i18n/Cargo.toml) - i18n crate configuration
- [`crates/lintdiff-i18n/src/lib.rs`](crates/lintdiff-i18n/src/lib.rs) - i18n infrastructure
- [`crates/lintdiff-i18n/src/locales/en-US/cli.ftl`](crates/lintdiff-i18n/src/locales/en-US/cli.ftl) - CLI messages
- [`crates/lintdiff-i18n/src/locales/en-US/errors.ftl`](crates/lintdiff-i18n/src/locales/en-US/errors.ftl) - Error messages
- [`crates/lintdiff-i18n/src/locales/en-US/main.ftl`](crates/lintdiff-i18n/src/locales/en-US/main.ftl) - Main application messages
- [`crates/lintdiff-i18n/src/locales/en-US/report.ftl`](crates/lintdiff-i18n/src/locales/en-US/report.ftl) - Report output messages

### CI Examples
- [`docs/examples/basic-workflow.yml`](docs/examples/basic-workflow.yml) - Basic PR workflow
- [`docs/examples/monorepo-workflow.yml`](docs/examples/monorepo-workflow.yml) - Monorepo setup
- [`docs/examples/pr-check.yml`](docs/examples/pr-check.yml) - PR check workflow
- [`docs/examples/release-workflow.yml`](docs/examples/release-workflow.yml) - Release workflow example

### Fuzzing Infrastructure
- [`fuzz/Cargo.toml`](fuzz/Cargo.toml) - Fuzzing crate configuration
- [`fuzz/README.md`](fuzz/README.md) - Comprehensive fuzzing documentation
- [`fuzz/fuzz_targets/diagnostics_parser.rs`](fuzz/fuzz_targets/diagnostics_parser.rs) - Diagnostics parser fuzzer
- [`fuzz/fuzz_targets/diff_parser.rs`](fuzz/fuzz_targets/diff_parser.rs) - Diff parser fuzzer
- [`fuzz/fuzz_targets/finding_fingerprint.rs`](fuzz/fuzz_targets/finding_fingerprint.rs) - Fingerprint fuzzer
- [`fuzz/corpus/`](fuzz/corpus/) - Structured test cases for fuzz targets

---

> This roadmap is a living document. Priorities may shift based on user feedback and project needs. Last updated: 2026-03-17
