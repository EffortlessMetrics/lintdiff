# Migration Guide: Façade Crate Deprecation

> **📜 HISTORICAL REFERENCE DOCUMENT**
>
> This document is preserved for historical reference. The migration period has ended.
>
> **v1.0.0 (2026-03-25)**: The façade crates (`lintdiff-domain`, `lintdiff-core`, `lintdiff-ingest`)
> have been **removed** from the workspace. Migration to `lintdiff-ingest-core` is now **required**
> to use lintdiff v1.0.0 and later.
>
> If you are still using a pre-v1.0.0 version, this guide will help you migrate before upgrading.

This guide helps you migrate from the removed façade crates (`lintdiff-domain`, `lintdiff-core`, `lintdiff-ingest`) to `lintdiff-ingest-core`.

## Quick Links

- **[Migration Examples](examples/migration-example.md)** - Copy-paste ready code examples
- **[Automated Migration Script](migration-plans/PR-106-automated-migration-script.md)** - Script to automate the migration

## Overview

Three façade crates were deprecated in version 0.2.0 and **removed** in version 1.0.0:

| Removed Crate | Replacement | Status |
|---------------|-------------|--------|
| `lintdiff-domain` | `lintdiff-ingest-core` | Removed in v1.0.0 |
| `lintdiff-core` | `lintdiff-ingest-core` | Removed in v1.0.0 |
| `lintdiff-ingest` | `lintdiff-ingest-core` | Removed in v1.0.0 |

These façade crates were created during refactoring to maintain backward compatibility. They simply re-exported all items from `lintdiff-ingest-core`. Migrating directly to `lintdiff-ingest-core` reduces dependencies and simplifies your crate graph.

## Quick Migration Guide (TL;DR)

### Step 1: Update `Cargo.toml`

Replace any deprecated dependency with `lintdiff-ingest-core`:

```diff
[dependencies]
- lintdiff-domain = "0.1"
+ lintdiff-ingest-core = "0.2"
```

```diff
[dependencies]
- lintdiff-core = "0.1"
+ lintdiff-ingest-core = "0.2"
```

```diff
[dependencies]
- lintdiff-ingest = "0.1"
+ lintdiff-ingest-core = "0.2"
```

### Step 2: Update Import Statements

Use search-and-replace to update your `use` statements:

```diff
- use lintdiff_domain::IngestPipeline;
+ use lintdiff_ingest_core::IngestPipeline;
```

```diff
- use lintdiff_core::verdict::Verdict;
+ use lintdiff_ingest_core::verdict::Verdict;
```

```diff
- use lintdiff_ingest::policy::Policy;
+ use lintdiff_ingest_core::policy::Policy;
```

### Step 3: Verify Build

```bash
cargo build
```

You should no longer see deprecation warnings.

---

## Detailed Migration Steps

### 1. Identify Affected Dependencies

Check your `Cargo.toml` for any of the deprecated crates:

```bash
# Check for deprecated dependencies
grep -E "lintdiff-(domain|core|ingest)" Cargo.toml
```

### 2. Update Dependencies

Edit your `Cargo.toml` to replace deprecated crates:

**Before:**
```toml
[dependencies]
lintdiff-domain = "0.1"
# or
lintdiff-core = "0.1"
# or
lintdiff-ingest = "0.1"
```

**After:**
```toml
[dependencies]
lintdiff-ingest-core = "0.2"
```

### 3. Update Source Files

Update all `use` statements in your Rust source files.

**Option A: Manual Update**

Find and replace in your editor:
- `lintdiff_domain::` → `lintdiff_ingest_core::`
- `lintdiff_core::` → `lintdiff_ingest_core::`
- `lintdiff_ingest::` → `lintdiff_ingest_core::`

**Option B: Automated Script**

On Unix-like systems (Linux, macOS, WSL):

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

On Windows (PowerShell):

```powershell
# Replace imports in source files
Get-ChildItem -Recurse -Filter "*.rs" | ForEach-Object {
    (Get-Content $_.FullName) -replace 'lintdiff_domain', 'lintdiff_ingest_core' | Set-Content $_.FullName
}
Get-ChildItem -Recurse -Filter "*.rs" | ForEach-Object {
    (Get-Content $_.FullName) -replace 'lintdiff_core', 'lintdiff_ingest_core' | Set-Content $_.FullName
}
Get-ChildItem -Recurse -Filter "*.rs" | ForEach-Object {
    (Get-Content $_.FullName) -replace 'lintdiff_ingest::', 'lintdiff_ingest_core::' | Set-Content $_.FullName
}

# Update Cargo.toml files
Get-ChildItem -Recurse -Filter "Cargo.toml" | ForEach-Object {
    (Get-Content $_.FullName) -replace 'lintdiff-domain', 'lintdiff-ingest-core' | Set-Content $_.FullName
}
Get-ChildItem -Recurse -Filter "Cargo.toml" | ForEach-Object {
    (Get-Content $_.FullName) -replace 'lintdiff-core', 'lintdiff-ingest-core' | Set-Content $_.FullName
}
Get-ChildItem -Recurse -Filter "Cargo.toml" | ForEach-Object {
    (Get-Content $_.FullName) -replace 'lintdiff-ingest =', 'lintdiff-ingest-core =' | Set-Content $_.FullName
}
```

### 4. Build and Test

```bash
# Clean build artifacts
cargo clean

# Build to check for compilation errors
cargo build

# Run tests
cargo test
```

---

## Import Path Transformation Table

| Old Path | New Path |
|----------|----------|
| `lintdiff_domain::*` | `lintdiff_ingest_core::*` |
| `lintdiff_core::*` | `lintdiff_ingest_core::*` |
| `lintdiff_ingest::*` | `lintdiff_ingest_core::*` |
| `lintdiff_domain::IngestOnDiffParams` | `lintdiff_ingest_core::IngestOnDiffParams` |
| `lintdiff_domain::ingest_on_diff` | `lintdiff_ingest_core::ingest_on_diff` |
| `lintdiff_core::verdict::Verdict` | `lintdiff_ingest_core::verdict::Verdict` |
| `lintdiff_core::policy::Policy` | `lintdiff_ingest_core::policy::Policy` |
| `lintdiff_ingest::match_diagnostics` | `lintdiff_ingest_core::match_diagnostics` |

**Note:** All public items from the deprecated crates are available in `lintdiff-ingest-core` with identical APIs.

---

## Code Examples

### Example 1: Basic Usage

**Before (using `lintdiff-domain`):**

```rust
// Cargo.toml
[dependencies]
lintdiff-domain = "0.1"

// src/main.rs
use lintdiff_domain::{IngestOnDiffParams, ingest_on_diff};
use lintdiff_domain::verdict::Verdict;

fn main() {
    let params = IngestOnDiffParams {
        // ... parameters ...
    };
    let report = ingest_on_diff(params);
    println!("{:?}", report.verdict);
}
```

**After (using `lintdiff-ingest-core`):**

```rust
// Cargo.toml
[dependencies]
lintdiff-ingest-core = "0.2"

// src/main.rs
use lintdiff_ingest_core::{IngestOnDiffParams, ingest_on_diff};
use lintdiff_ingest_core::verdict::Verdict;

fn main() {
    let params = IngestOnDiffParams {
        // ... parameters ...
    };
    let report = ingest_on_diff(params);
    println!("{:?}", report.verdict);
}
```

### Example 2: Using Policy Types

**Before (using `lintdiff-core`):**

```rust
// Cargo.toml
[dependencies]
lintdiff-core = "0.1"

// src/lib.rs
use lintdiff_core::policy::{Policy, PolicyRule};
use lintdiff_core::verdict::{Verdict, VerdictStatus};
```

**After (using `lintdiff-ingest-core`):**

```rust
// Cargo.toml
[dependencies]
lintdiff-ingest-core = "0.2"

// src/lib.rs
use lintdiff_ingest_core::policy::{Policy, PolicyRule};
use lintdiff_ingest_core::verdict::{Verdict, VerdictStatus};
```

### Example 3: Multiple Dependencies

**Before:**

```toml
[dependencies]
lintdiff-domain = "0.1"
lintdiff-core = "0.1"  # If you had both
```

**After:**

```toml
[dependencies]
lintdiff-ingest-core = "0.2"  # Single dependency replaces both
```

---

## Common Migration Scenarios

### Scenario 1: Library Crate

If you maintain a library that depends on a deprecated crate:

1. Update your `Cargo.toml` to use `lintdiff-ingest-core`
2. Update your re-exports if you expose types from the deprecated crate
3. Bump your minor version (e.g., 0.5.0 → 0.6.0) to signal the change
4. Update your documentation to reflect the new dependency

```rust
// Before
pub use lintdiff_domain::Verdict;

// After
pub use lintdiff_ingest_core::Verdict;
```

### Scenario 2: Binary Application

If you have a binary application:

1. Update `Cargo.toml`
2. Update all `use` statements
3. No version bump needed for internal applications

### Scenario 3: Workspace with Multiple Crates

If you have a workspace with multiple crates using deprecated dependencies:

1. Update the root `Cargo.toml` workspace dependencies (if using shared dependencies)
2. Update each crate's `Cargo.toml` individually
3. Use the automated script to update all source files at once

### Scenario 4: Using Feature Flags

If you were using feature flags from the deprecated crates:

```toml
# Before
[dependencies]
lintdiff-core = { version = "0.1", features = ["serde"] }

# After
[dependencies]
lintdiff-ingest-core = { version = "0.2", features = ["serde"] }
```

**Note:** Feature flags remain identical in `lintdiff-ingest-core`.

---

## Troubleshooting

### Issue: "cannot find type `X` in crate `lintdiff_ingest_core`"

**Cause:** The type may have been renamed or moved to a submodule.

**Solution:** Check the [`lintdiff-ingest-core` documentation](../crates/lintdiff-ingest-core/src/lib.rs) for the correct import path.

### Issue: "trait bounds not satisfied"

**Cause:** The new crate may have slightly different trait requirements.

**Solution:** Ensure all required traits are imported. The API is identical, so this usually indicates a stale build.

```bash
cargo clean
cargo build
```

### Issue: "multiple versions of crate `lintdiff-ingest-core` in dependency graph"

**Cause:** Another dependency still uses the old façade crates.

**Solution:** Check your dependency tree:

```bash
cargo tree -i lintdiff-ingest-core
cargo tree -i lintdiff-domain
cargo tree -i lintdiff-core
cargo tree -i lintdiff-ingest
```

Update or file issues with dependencies that still use deprecated crates.

### Issue: Deprecation warnings still appear after migration

**Cause:** Some imports may have been missed, or a transitive dependency uses deprecated crates.

**Solution:** 
1. Search for remaining old imports:
   ```bash
   grep -r "lintdiff_domain\|lintdiff_core\|lintdiff_ingest::" src/
   ```
2. Check if any dependencies still use deprecated crates:
   ```bash
   cargo tree | grep -E "lintdiff-(domain|core|ingest)"
   ```

### Issue: "use of deprecated function"

**Cause:** You're calling a function marked as deprecated within `lintdiff-ingest-core` itself.

**Solution:** This is separate from the façade deprecation. Check the function's documentation for the recommended alternative.

---

## Timeline and Version Requirements

### Deprecation Timeline

| Version | Phase | Description | Date |
|---------|-------|-------------|------|
| `0.1.x` | Pre-deprecation | Façade crates are active | Prior to Q2 2026 |
| `0.2.0` | **Phase 1** | Deprecation warnings added | Q2 2026 |
| `0.2.1` | Phase 2 | Internal migration begins | Q2 2026 |
| `0.3.0` | Phase 2 | All internal consumers migrated | Q3-Q4 2026 |
| `0.4.0` | Phase 2 | Optional `compile_error!` added | Q4 2026 |
| `1.0.0` | **Phase 3** | Façade crates removed | Q1 2027 |

### Version Requirements for Migration

- **Minimum version for `lintdiff-ingest-core`**: `0.2.0`
- **Recommended version**: Latest stable release

### Semantic Versioning Notes

- During `0.x.x` versions, breaking changes are allowed in minor versions
- Version `1.0.0` will signal production-ready stability
- The façade removal in `1.0.0` is a breaking change, hence the major version bump

### Migration Deadline

**Plan to migrate before version 1.0.0 (Q1 2027).** After 1.0.0, the deprecated crates will be removed from crates.io and the repository.

---

## Getting Help

If you encounter issues during migration:

1. Check this guide's [Troubleshooting](#troubleshooting) section
2. Review the [deprecation plan](deprecation-plan.md) for technical details
3. Search existing GitHub issues or open a new issue with the `migration` label
4. Consult the [architecture documentation](architecture.md) for understanding the crate structure

---

## See Also

- **[Migration Examples](examples/migration-example.md)** - Copy-paste ready code examples for migration
- **[Automated Migration Script](migration-plans/PR-106-automated-migration-script.md)** - Script to automate the migration process
- [Deprecation Plan](deprecation-plan.md) - Technical details of the deprecation process
- [Architecture Documentation](architecture.md) - Overview of the crate structure
- [CHANGELOG.md](../CHANGELOG.md) - Release notes and deprecation announcements
- [ADR-001: Hexagonal Architecture](adr/ADR-001-hexagonal-architecture.md) - Design rationale
