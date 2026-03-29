# Compatibility Façade Deprecation Plan

> **📜 HISTORICAL REFERENCE DOCUMENT**
>
> This document is preserved for historical reference. The deprecation process described herein
> has been **completed** as of v1.0.0 (2026-03-25).
>
> The façade crates (`lintdiff-domain`, `lintdiff-core`, `lintdiff-ingest`) have been **removed**
> from the workspace. All code should use `lintdiff-ingest-core` directly.

This document outlines the strategy that was used for deprecating and removing the compatibility façade crates in lintdiff.

## Executive Summary

The lintdiff project contained three compatibility façade crates that were created during refactoring to maintain backward compatibility. These façades added unnecessary layers and complexity. This plan described a phased approach to deprecate and eventually remove them.

**Status: ✅ COMPLETE** - All façade crates were removed in v1.0.0 (2026-03-25).

## 1. Current State

### Façade Chain

The façades form a chain of re-exports:

```
lintdiff-domain → lintdiff-core → lintdiff-ingest → lintdiff-ingest-core
```

Each crate simply re-exports all items from its dependency:

| Crate | Re-exports From | Description |
|-------|-----------------|-------------|
| `lintdiff-domain` | `lintdiff-core` | Backward-compatible façade for legacy consumers |
| `lintdiff-core` | `lintdiff-ingest` | Compatibility façade over ingest |
| `lintdiff-ingest` | `lintdiff-ingest-core` | Preserves historical public surface |

### Current Dependencies

#### `lintdiff-domain`
- **Depends on**: `lintdiff-core`
- **Consumers**: `lintdiff-app`

#### `lintdiff-core`
- **Depends on**: `lintdiff-ingest`
- **Additional dependencies**: `serde_json`
- **Consumers**: `lintdiff-domain`, `lintdiff-bdd-harness`

#### `lintdiff-ingest`
- **Depends on**: `lintdiff-ingest-core`
- **Consumers**: `lintdiff-core`

#### `lintdiff-ingest-core` (Target Crate)
- **Depends on**: `lintdiff-diagnostics`, `lintdiff-diff`, `lintdiff-match`, `lintdiff-policy`, `lintdiff-types`
- **Consumers**: `lintdiff-ingest`
- **Status**: This is the final target crate that contains actual implementation

### Import Path Transformations

| Old Path | New Path |
|----------|----------|
| `lintdiff_domain::Something` | `lintdiff_ingest_core::Something` |
| `lintdiff_core::Something` | `lintdiff_ingest_core::Something` |
| `lintdiff_ingest::Something` | `lintdiff_ingest_core::Something` |

---

## 2. Deprecation Timeline

### Phase 1: Deprecation Warnings (Version 0.2.0)

**Target Date**: Q2 2026

**Actions**:
1. Add `#[deprecated]` attributes to all re-exports in façade crates
2. Add compile-time warnings via `#[warn(deprecated)]`
3. Update crate-level documentation with migration instructions
4. Publish deprecation notice in CHANGELOG.md

**Version Requirements**:
- All crates share workspace version `0.2.0`
- Deprecation attributes will use `since = "0.2.0"`

### Phase 2: Migration Period (Versions 0.2.x - 0.3.x) ✅ COMPLETE

**Target Date**: Q2-Q4 2026
**Completion Date**: 2026-03-25

**Actions**:
1. ✅ Update all internal consumers to use `lintdiff-ingest-core` directly
2. ✅ Add detailed migration examples to documentation
3. ✅ Provide automated migration tooling if feasible
4. Monitor ecosystem for external consumers

**Milestones**:
- `0.2.0`: ✅ Deprecation warnings added
- `0.2.1`: ✅ Internal migration of `lintdiff-app`
- `0.2.2`: ✅ Internal migration of `lintdiff-bdd-harness`
- `0.3.0`: ✅ All internal consumers migrated

### Phase 3: Removal (Version 1.0.0) ✅ COMPLETE

**Target Date**: Q1 2027
**Completion Date**: 2026-03-25

**Actions**:
1. ✅ Remove façade crates from workspace
2. ✅ Update Cargo.toml workspace members
3. ✅ Archive removed crates in a separate branch for reference
4. ✅ Major version bump signals breaking change

**Rationale for 1.0.0 Removal**:
- Semantic versioning requires major version bump for breaking changes
- Removing public APIs is a breaking change
- 1.0.0 signals production-ready stability

---

## 3. Migration Guide

### For `lintdiff-domain` Users

**Before**:
```rust
// Cargo.toml
[dependencies]
lintdiff-domain = "0.1"

// lib.rs
use lintdiff_domain::IngestPipeline;
use lintdiff_domain::verdict::Verdict;
```

**After**:
```rust
// Cargo.toml
[dependencies]
lintdiff-ingest-core = "0.2"

// lib.rs
use lintdiff_ingest_core::IngestPipeline;
use lintdiff_ingest_core::verdict::Verdict;
```

### For `lintdiff-core` Users

**Before**:
```rust
// Cargo.toml
[dependencies]
lintdiff-core = "0.1"

// lib.rs
use lintdiff_core::IngestPipeline;
use lintdiff_core::match_diagnostics;
```

**After**:
```rust
// Cargo.toml
[dependencies]
lintdiff-ingest-core = "0.2"

// lib.rs
use lintdiff_ingest_core::IngestPipeline;
use lintdiff_ingest_core::match_diagnostics;
```

### For `lintdiff-ingest` Users

**Before**:
```rust
// Cargo.toml
[dependencies]
lintdiff-ingest = "0.1"

// lib.rs
use lintdiff_ingest::IngestPipeline;
use lintdiff_ingest::policy::Policy;
```

**After**:
```rust
// Cargo.toml
[dependencies]
lintdiff-ingest-core = "0.2"

// lib.rs
use lintdiff_ingest_core::IngestPipeline;
use lintdiff_ingest_core::policy::Policy;
```

### Automated Migration

A `sed` or `ripgrep` replacement can be used:

```bash
# Replace imports in source files
find . -name "*.rs" -exec sed -i 's/lintdiff_domain/lintdiff_ingest_core/g' {} \;
find . -name "*.rs" -exec sed -i 's/lintdiff_core/lintdiff_ingest_core/g' {} \;
find . -name "*.rs" -exec sed -i 's/lintdiff_ingest::/lintdiff_ingest_core::/g' {} \;

# Update Cargo.toml
find . -name "Cargo.toml" -exec sed -i 's/lintdiff-domain/lintdiff-ingest-core/g' {} \;
find . -name "Cargo.toml" -exec sed -i 's/lintdiff-core/lintdiff-ingest-core/g' {} \;
find . -name "Cargo.toml" -exec sed -i 's/lintdiff-ingest =/lintdiff-ingest-core =/g' {} \;
```

---

## 4. Implementation Steps

### Step 1: Add Deprecation Attributes

#### `crates/lintdiff-domain/src/lib.rs`

```rust
//! **DEPRECATED**: Use `lintdiff-ingest-core` instead.
//!
//! This crate is a compatibility façade that will be removed in version 1.0.0.
//! See the migration guide at `docs/deprecation-plan.md`.

#[deprecated(
    since = "0.2.0",
    note = "Use lintdiff-ingest-core instead. This façade will be removed in 1.0.0."
)]
pub use lintdiff_core::*;
```

#### `crates/lintdiff-core/src/lib.rs`

```rust
//! **DEPRECATED**: Use `lintdiff-ingest-core` instead.
//!
//! This crate is a compatibility façade that will be removed in version 1.0.0.
//! See the migration guide at `docs/deprecation-plan.md`.

#[deprecated(
    since = "0.2.0",
    note = "Use lintdiff-ingest-core instead. This façade will be removed in 1.0.0."
)]
pub use lintdiff_ingest::*;
```

#### `crates/lintdiff-ingest/src/lib.rs`

```rust
//! **DEPRECATED**: Use `lintdiff-ingest-core` instead.
//!
//! This crate is a compatibility façade that will be removed in version 1.0.0.
//! See the migration guide at `docs/deprecation-plan.md`.

#[deprecated(
    since = "0.2.0",
    note = "Use lintdiff-ingest-core instead. This façade will be removed in 1.0.0."
)]
pub use lintdiff_ingest_core::*;
```

### Step 2: Update Crate Metadata

Add deprecation notice to each crate's `Cargo.toml`:

```toml
[package]
# ... existing fields ...
description = "DEPRECATED: Use lintdiff-ingest-core instead. This crate will be removed in 1.0.0."
```

### Step 3: Add Cargo Warnings

Add to each façade crate's `lib.rs`:

```rust
// Compile-time warning for users
#[cfg(not(test))]
compile_error!(
    "lintdiff-domain is deprecated. Migrate to lintdiff-ingest-core. \
     See docs/deprecation-plan.md for migration instructions."
);
```

**Note**: The `compile_error!` approach is aggressive. Consider using `#[deprecated]` first,
then add `compile_error!` in a later version (0.3.0+) to force migration.

### Step 4: Update Documentation

1. Add migration notice to README.md
2. Update architecture.md to reflect new structure
3. Add deprecation status to CLAUDE.md

### Step 5: Internal Migration

#### `lintdiff-app/Cargo.toml`

```diff
[dependencies]
lintdiff-diff = { path = "../lintdiff-diff" }
- lintdiff-domain = { path = "../lintdiff-domain" }
+ lintdiff-ingest-core = { path = "../lintdiff-ingest-core" }
lintdiff-render = { path = "../lintdiff-render" }
```

#### `lintdiff-bdd-harness/Cargo.toml`

```diff
[dependencies]
lintdiff-bdd-grid = { path = "../lintdiff-bdd-grid" }
- lintdiff-core = { path = "../lintdiff-core" }
+ lintdiff-ingest-core = { path = "../lintdiff-ingest-core" }
lintdiff-diagnostics = { path = "../lintdiff-diagnostics" }
```

---

## 5. Version Strategy

### Version Timeline

| Version | Phase | Description |
|---------|-------|-------------|
| `0.1.0` | Current | Initial release with façades |
| `0.2.0` | Phase 1 | Deprecation warnings added |
| `0.2.1` | Phase 2 | Internal migration begins |
| `0.3.0` | Phase 2 | All internal consumers migrated |
| `0.4.0` | Phase 2 | `compile_error!` added (optional) |
| `1.0.0` | Phase 3 | Façades removed |

### Semantic Versioning Compliance

- **0.x.x**: Breaking changes allowed in minor versions
- **1.0.0+**: Breaking changes require major version bump

### Deprecation Policy

Following the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

1. **Announce deprecation** in the release notes and documentation
2. **Add `#[deprecated]` attribute** with `since` and `note` fields
3. **Maintain for at least one major version cycle** before removal
4. **Provide clear migration path** in documentation

### Release Checklist

#### For 0.2.0 (Deprecation)
- [x] Add `#[deprecated]` attributes to all façade crates
- [x] Update crate descriptions in Cargo.toml
- [x] Update documentation with migration guide
- [x] Add CHANGELOG entry for deprecation
- [x] Ensure CI passes with deprecation warnings

#### For 1.0.0 (Removal)
- [x] Remove façade crates from workspace
- [x] Update all documentation
- [x] Archive removed crates
- [ ] Major version bump
- [ ] Update CHANGELOG with removal notice

---

## 6. Rollback Plan

If critical issues arise during deprecation:

1. **Phase 1 Rollback**: Remove `#[deprecated]` attributes, release as patch version
2. **Phase 2 Rollback**: Revert internal migrations, release as minor version
3. **Phase 3 Rollback**: Restore façade crates from archive branch, release as 1.1.0

---

## 7. Communication Plan

### Internal
- Update ROADMAP.md with deprecation timeline
- Add deprecation status to CLAUDE.md
- Document in architecture.md

### External
- CHANGELOG.md entries for each phase
- README.md deprecation notice
- GitHub release notes
- GitHub issue for tracking migration progress

---

## 8. Success Criteria

The deprecation is considered complete when:

1. ✅ All façade crates have deprecation warnings (Phase 1)
2. ✅ All internal consumers use `lintdiff-ingest-core` directly (Phase 2)
3. ✅ Documentation reflects new architecture (Phase 2)
4. ✅ Façade crates are removed from workspace (Phase 3)
5. ✅ No compilation errors for users following migration guide (All phases)

---

## Appendix A: Façade Crate Locations (Historical)

The following façade crates were removed in Phase 3 (v1.0.0):

- `lintdiff-domain/` - Removed
- `lintdiff-core/` - Removed
- `lintdiff-ingest/` - Removed

The target crate remains:

```
crates/
└── lintdiff-ingest-core/     # Active: Complete public API
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

## Appendix B: Related Documentation

- [Architecture Documentation](architecture.md)
- [CHANGELOG.md](../CHANGELOG.md)
- [ROADMAP.md](../ROADMAP.md)
- [ADR-001: Hexagonal Architecture](adr/ADR-001-hexagonal-architecture.md)
