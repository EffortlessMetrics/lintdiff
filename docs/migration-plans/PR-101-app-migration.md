# PR-101: Migration Plan for `lintdiff-app`

## Overview

This document outlines the migration of `lintdiff-app` from the deprecated `lintdiff-domain` façade to `lintdiff-ingest-core`.

| Attribute | Value |
|-----------|-------|
| **PR** | PR-101 |
| **Epic** | EPIC-001 Phase 2 (Internal Migration) |
| **Status** | Planned |
| **Risk Level** | Low |

## Current State Analysis

### Dependency Chain

The current deprecated crate chain is:

```mermaid
graph LR
    A[lintdiff-app] --> B[lintdiff-domain]
    B --> C[lintdiff-core]
    C --> D[lintdiff-ingest]
    D --> E[lintdiff-ingest-core]
    
    style B fill:#ffcccc
    style C fill:#ffcccc
    style D fill:#ffcccc
    style E fill:#ccffcc
```

- **Red boxes**: Deprecated façade crates
- **Green box**: Target crate with actual implementation

### Current Dependencies

**File**: [`crates/lintdiff-app/Cargo.toml`](crates/lintdiff-app/Cargo.toml:22)

```toml
lintdiff-domain = { path = "../lintdiff-domain" }
```

### Current Imports

**File**: [`crates/lintdiff-app/src/lib.rs`](crates/lintdiff-app/src/lib.rs:11)

```rust
use lintdiff_domain::{ingest_on_diff, IngestOnDiffParams};
```

### Usage Locations

| Location | Type | Usage |
|----------|------|-------|
| [`lib.rs:11`](crates/lintdiff-app/src/lib.rs:11) | Import | `use lintdiff_domain::{ingest_on_diff, IngestOnDiffParams}` |
| [`lib.rs:111`](crates/lintdiff-app/src/lib.rs:111) | Function call | `ingest_on_diff(IngestOnDiffParams { ... })` |
| [`lib.rs:232`](crates/lintdiff-app/src/lib.rs:232) | Function call | `ingest_on_diff(IngestOnDiffParams { ... })` |

## Target State

### Target Dependency

**File**: `crates/lintdiff-app/Cargo.toml`

```toml
lintdiff-ingest-core = { path = "../lintdiff-ingest-core" }
```

### Target Imports

**File**: `crates/lintdiff-app/src/lib.rs`

```rust
use lintdiff_ingest_core::{ingest_on_diff, IngestOnDiffParams};
```

### Type Mapping Table

| Current (lintdiff-domain) | Target (lintdiff-ingest-core) | Notes |
|---------------------------|-------------------------------|-------|
| `ingest_on_diff` | `ingest_on_diff` | Direct mapping - same function |
| `IngestOnDiffParams` | `IngestOnDiffParams` | Direct mapping - same struct |

**No API changes required** - the types and function signatures are identical since `lintdiff-domain` was a pure re-export façade.

## Step-by-Step Migration Instructions

### Step 1: Update Cargo.toml

**File**: [`crates/lintdiff-app/Cargo.toml`](crates/lintdiff-app/Cargo.toml)

1. Remove the deprecated dependency:
   ```diff
   - lintdiff-domain = { path = "../lintdiff-domain" }
   ```

2. Add the new dependency:
   ```diff
   + lintdiff-ingest-core = { path = "../lintdiff-ingest-core" }
   ```

### Step 2: Update lib.rs Imports

**File**: [`crates/lintdiff-app/src/lib.rs`](crates/lintdiff-app/src/lib.rs:11)

Change the import statement:

```diff
- use lintdiff_domain::{ingest_on_diff, IngestOnDiffParams};
+ use lintdiff_ingest_core::{ingest_on_diff, IngestOnDiffParams};
```

### Step 3: Verify Compilation

Run the following command to ensure the code compiles:

```bash
cargo check -p lintdiff-app
```

### Step 4: Run Tests

Execute the test suite to verify functionality:

```bash
cargo test -p lintdiff-app
```

Note: `lintdiff-app` does not have dedicated test files, so this step verifies compilation and any doc tests.

### Step 5: Run Integration Tests

Run broader integration tests that exercise `lintdiff-app`:

```bash
cargo test -p lintdiff-cli
```

### Step 6: Verify Deprecation Warnings

Confirm that deprecation warnings related to `lintdiff-domain` no longer appear:

```bash
cargo build -p lintdiff-app 2>&1 | grep -i deprecated
```

## Testing Strategy

### Unit Tests

Since `lintdiff-app` has no dedicated unit tests, verification relies on:

1. **Compilation check**: Ensures types and imports resolve correctly
2. **Doc tests**: Any documentation examples in the crate

### Integration Tests

| Test Suite | Crate | Purpose |
|------------|-------|---------|
| BDD Tests | `lintdiff-cli` | End-to-end workflow validation |
| Integration Tests | `lintdiff-app-git` | Git operations with app orchestration |
| Integration Tests | `lintdiff-app-io` | I/O operations with app orchestration |

### Manual Verification

1. Build the CLI:
   ```bash
   cargo build -p lintdiff-cli --release
   ```

2. Run a smoke test:
   ```bash
   ./target/release/lintdiff --help
   ```

### Regression Testing Checklist

- [ ] `cargo check -p lintdiff-app` passes
- [ ] `cargo test -p lintdiff-app` passes
- [ ] `cargo test -p lintdiff-cli` passes
- [ ] No deprecation warnings from `lintdiff-domain`
- [ ] BDD tests pass
- [ ] Manual CLI smoke test succeeds

## Potential Risks and Mitigations

### Risk 1: Transitive Dependency Issues

**Risk**: Other crates in the workspace might still depend on `lintdiff-domain`, causing duplicate dependencies.

**Mitigation**: 
- Check workspace for remaining `lintdiff-domain` usage:
  ```bash
  grep -r "lintdiff-domain" crates/*/Cargo.toml
  ```
- This PR only migrates `lintdiff-app`; other crates will be migrated in separate PRs

### Risk 2: Feature Flag Differences

**Risk**: `lintdiff-ingest-core` might have different feature flags than `lintdiff-domain`.

**Mitigation**:
- Review [`lintdiff-ingest-core/Cargo.toml`](crates/lintdiff-ingest-core/Cargo.toml) for feature flags
- Currently, no feature flags are used in the dependency

### Risk 3: Type Re-exports Missing

**Risk**: `lintdiff-ingest-core` might not re-export all types that `lintdiff-domain` did.

**Mitigation**:
- Analysis confirms `ingest_on_diff` and `IngestOnDiffParams` are directly defined in `lintdiff-ingest-core`
- The façade chain was: `domain → core → ingest → ingest-core`
- All types originate from `lintdiff-ingest-core`

## Post-Migration Cleanup

After this migration is complete:

1. **No immediate cleanup** - `lintdiff-domain` can remain for other consumers
2. **Track remaining consumers** via EPIC-001 Phase 2
3. **Final deprecation** of `lintdiff-domain` occurs in Phase 3 after all consumers migrate

## Related Documents

- [EPIC-001 Roadmap](../EPIC-ROADMAP.md)
- [Deprecation Plan](../deprecation-plan.md)
- [PR-102: BDD Harness Migration](PR-102-bdd-harness-migration.md)

## Summary

This migration is **low risk** because:

1. **Pure re-export façade**: `lintdiff-domain` only re-exports from `lintdiff-ingest-core` through a chain
2. **No API changes**: Types and function signatures remain identical
3. **Simple change**: Only 2 files modified with minimal diff
4. **Good test coverage**: Integration tests in dependent crates validate functionality

The migration involves:
- **2 files changed**
- **2 lines modified** (1 in Cargo.toml, 1 in lib.rs)
- **0 business logic changes**
