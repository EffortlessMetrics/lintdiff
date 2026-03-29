# PR-102: Migration Plan for `lintdiff-bdd-harness`

## Overview

This document outlines the migration of `lintdiff-bdd-harness` from the deprecated `lintdiff-core` façade to `lintdiff-ingest-core`.

| Attribute | Value |
|-----------|-------|
| **PR** | PR-102 |
| **Epic** | EPIC-001 Phase 2 (Internal Migration) |
| **Milestone** | 0.2.2 |
| **Status** | Planned |

---

## 1. Current State Analysis

### 1.1 Dependency Chain

The current façade chain flows as follows:

```mermaid
graph LR
 A[lintdiff-bdd-harness] --> B[lintdiff-core]
 B --> C[lintdiff-ingest]
 C --> D[lintdiff-ingest-core]
 
 style B fill:#ffcccc
 style C fill:#ffcccc
 style D fill:#ccffcc
```

- **Red boxes**: Deprecated façade crates
- **Green box**: Target implementation crate

### 1.2 Current Dependencies

**File**: [`crates/lintdiff-bdd-harness/Cargo.toml`](crates/lintdiff-bdd-harness/Cargo.toml)

```toml
[dependencies]
lintdiff-bdd-grid = { path = "../lintdiff-bdd-grid" }
lintdiff-core = { path = "../lintdiff-core" }# ← TO BE REPLACED
lintdiff-diagnostics = { path = "../lintdiff-diagnostics" }
lintdiff-diff = { path = "../lintdiff-diff" }
lintdiff-feature-flags = { path = "../lintdiff-feature-flags" }
lintdiff-types = { path = "../lintdiff-types" }
```

### 1.3 Current Imports

**File**: [`crates/lintdiff-bdd-harness/src/lib.rs`](crates/lintdiff-bdd-harness/src/lib.rs:4)

```rust
use lintdiff_core::{ingest_on_diff, IngestOnDiffParams};
```

### 1.4 Types and Functions Used from `lintdiff-core`

| Type/Function | Source Location | Usage in Harness |
|---------------|-----------------|------------------|
| [`ingest_on_diff`](crates/lintdiff-ingest-core/src/lib.rs:41) | `lintdiff-ingest-core` | Core pipeline function called in [`run_ingest_from_fixtures()`](crates/lintdiff-bdd-harness/src/lib.rs:75) |
| [`IngestOnDiffParams`](crates/lintdiff-ingest-core/src/lib.rs:20) | `lintdiff-ingest-core` | Parameter struct constructed in [`run_ingest_from_fixtures()`](crates/lintdiff-bdd-harness/src/lib.rs:99) |

---

## 2. Target State

### 2.1 New Dependency Chain

After migration:

```mermaid
graph LR
 A[lintdiff-bdd-harness] --> B[lintdiff-ingest-core]
 
 style B fill:#ccffcc
```

### 2.2 Import Mapping

| Current Import | New Import |
|----------------|------------|
| `lintdiff_core::ingest_on_diff` | `lintdiff_ingest_core::ingest_on_diff` |
| `lintdiff_core::IngestOnDiffParams` | `lintdiff_ingest_core::IngestOnDiffParams` |

### 2.3 API Compatibility

The types are **100% compatible** because:

1. [`lintdiff-core`](crates/lintdiff-core/src/lib.rs:12) re-exports all of `lintdiff-ingest`:
 ```rust
 pub use lintdiff_ingest::*;
 ```

2. [`lintdiff-ingest`](crates/lintdiff-ingest/src/lib.rs:12) re-exports all of `lintdiff-ingest-core`:
 ```rust
 pub use lintdiff_ingest_core::*;
 ```

3. Therefore, `lintdiff_core::*` === `lintdiff_ingest_core::*`

---

## 3. Step-by-Step Migration Instructions

### Step 1: Update `Cargo.toml`

**File**: [`crates/lintdiff-bdd-harness/Cargo.toml`](crates/lintdiff-bdd-harness/Cargo.toml)

Replace line 19:

```diff
[dependencies]
lintdiff-bdd-grid = { path = "../lintdiff-bdd-grid" }
-lintdiff-core = { path = "../lintdiff-core" }
+lintdiff-ingest-core = { path = "../lintdiff-ingest-core" }
lintdiff-diagnostics = { path = "../lintdiff-diagnostics" }
lintdiff-diff = { path = "../lintdiff-diff" }
lintdiff-feature-flags = { path = "../lintdiff-feature-flags" }
lintdiff-types = { path = "../lintdiff-types" }
```

### Step 2: Update Import Statement

**File**: [`crates/lintdiff-bdd-harness/src/lib.rs`](crates/lintdiff-bdd-harness/src/lib.rs:4)

Replace line 4:

```diff
use std::io::Cursor;

pub use lintdiff_bdd_grid::{FeatureFlagGrid, FeatureFlagGridRow};
-use lintdiff_core::{ingest_on_diff, IngestOnDiffParams};
+use lintdiff_ingest_core::{ingest_on_diff, IngestOnDiffParams};
use lintdiff_diagnostics::parse_cargo_messages;
```

### Step 3: Verify Compilation

```bash
cargo check -p lintdiff-bdd-harness
```

### Step 4: Run Tests

```bash
# Test the harness crate directly
cargo test -p lintdiff-bdd-harness

# Test the consumer crate (lintdiff-bdd)
cargo test -p lintdiff-bdd
```

### Step 5: Run Full Workspace Check

```bash
cargo check --workspace
cargo test --workspace
```

---

## 4. Testing Strategy

### 4.1 Unit Tests

The `lintdiff-bdd-harness` crate has no dedicated test directory. Testing is performed via:

1. **Consumer crate tests**: [`lintdiff-bdd`](crates/lintdiff-bdd/) consumes this harness
2. **Integration tests**: Any BDD scenario tests that use the harness

### 4.2 Test Commands

| Test Scope | Command | Purpose |
|------------|---------|---------|
| Harness compilation | `cargo check -p lintdiff-bdd-harness` | Verify no compile errors |
| Consumer tests | `cargo test -p lintdiff-bdd` | Verify downstream compatibility |
| Full workspace | `cargo test --workspace` | Catch any unexpected breakage |

### 4.3 Verification Checklist

- [ ] `cargo check -p lintdiff-bdd-harness` passes
- [ ] `cargo test -p lintdiff-bdd` passes
- [ ] `cargo test --workspace` passes
- [ ] No deprecation warnings from `lintdiff-bdd-harness` imports
- [ ] `cargo clippy --workspace` passes

---

## 5. Potential Risks and Mitigations

### 5.1 Risk Matrix

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| API incompatibility | Low | High | Types are re-exported identically; verify with `cargo check` |
| Missing transitive dependencies | Low | Medium | `lintdiff-ingest-core` has same deps as harness already uses |
| Breakage in consumer crates | Low | High | Run full workspace tests before merging |
| Feature flag behavior changes | None | N/A | No feature flags in the migrated code |

### 5.2 Rollback Plan

If issues are discovered post-merge:

1. Revert the PR
2. The change is isolated to 2 lines across 2 files
3. No data migration or state changes involved

---

## 6. Post-Migration Cleanup

After this migration is complete and merged:

1. Update [`docs/deprecation-plan.md`](docs/deprecation-plan.md) to mark `lintdiff-bdd-harness` as migrated
2. Verify no other crates depend on `lintdiff-core` (proceed to next PR if applicable)

---

## 7. Summary

This migration is **low risk** because:

- Only 2 files require changes
- The API is 100% compatible (simple re-export chain)
- No behavior changes - only import path changes
- Comprehensive test coverage via consumer crates

### Files Changed

| File | Change Type |
|------|-------------|
| [`crates/lintdiff-bdd-harness/Cargo.toml`](crates/lintdiff-bdd-harness/Cargo.toml) | Dependency update |
| [`crates/lintdiff-bdd-harness/src/lib.rs`](crates/lintdiff-bdd-harness/src/lib.rs) | Import path update |
