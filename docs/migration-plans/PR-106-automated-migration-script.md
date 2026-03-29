# PR-106: Automated Migration Script

## Overview

This PR provides an automated migration script to help external users migrate from deprecated façade crates to `lintdiff-ingest-core`.

## Deprecated Crates

The following façade crates are deprecated and should be replaced with `lintdiff-ingest-core`:

| Deprecated Crate | Replacement |
|-----------------|-------------|
| `lintdiff-domain` | `lintdiff-ingest-core` |
| `lintdiff-core` | `lintdiff-ingest-core` |
| `lintdiff-ingest` | `lintdiff-ingest-core` |

## Using the Migration Script

### Prerequisites

- Bash shell (Linux, macOS, or WSL on Windows)
- Basic Unix tools: `find`, `sed`, `grep`
- Optional: `perl` (for more reliable regex replacements)

### Running the Script

```bash
# From the lintdiff repository
./scripts/migrate-to-ingest-core.sh /path/to/your/project

# Or with a relative path
./scripts/migrate-to-ingest-core.sh ../my-project
```

### What the Script Does

1. **Updates `Cargo.toml` files**:
   - Finds all `Cargo.toml` files in your project (excluding `target/` directories)
   - Replaces dependency declarations for deprecated crates with `lintdiff-ingest-core`
   - Preserves version constraints and path references

2. **Updates Rust source files**:
   - Finds all `.rs` files in your project
   - Replaces `use lintdiff_domain::` with `use lintdiff_ingest_core::`
   - Replaces `use lintdiff_core::` with `use lintdiff_ingest_core::`
   - Replaces `use lintdiff_ingest::` with `use lintdiff_ingest_core::`

3. **Provides a summary** of changes made

### Example Output

```
[INFO] Starting migration in: /path/to/project

[INFO] Searching for Cargo.toml files...
[INFO] Processing: /path/to/project/Cargo.toml

[INFO] Searching for Rust source files...
[INFO] Processing: /path/to/project/src/main.rs
[INFO] Processing: /path/to/project/src/lib.rs

========================================
           MIGRATION SUMMARY
========================================

[SUCCESS] Migration completed successfully!

Cargo.toml files were updated to use lintdiff-ingest-core
Source files were updated to import from lintdiff_ingest_core

[WARNING] Next steps:
  1. Run 'cargo check' to verify the migration
  2. Run 'cargo test' to ensure tests pass
  3. Review the changes with 'git diff'
  4. Commit the changes
```

### Post-Migration Steps

After running the script:

1. **Verify compilation**:
   ```bash
   cargo check
   ```

2. **Run tests**:
   ```bash
   cargo test
   ```

3. **Review changes**:
   ```bash
   git diff
   ```

4. **Commit the changes**:
   ```bash
   git add -A
   git commit -m "chore: migrate from deprecated façade crates to lintdiff-ingest-core"
   ```

## Manual Migration Steps

If the script fails or you prefer to migrate manually, follow these steps:

### Step 1: Update Cargo.toml

Replace the deprecated dependency with `lintdiff-ingest-core`:

**Before:**
```toml
[dependencies]
lintdiff-domain = "0.1.0"
```

**After:**
```toml
[dependencies]
lintdiff-ingest-core = "0.1.0"
```

The same applies for `lintdiff-core` or `lintdiff-ingest`.

### Step 2: Update Source Imports

Replace the import statements in your Rust files:

**Before:**
```rust
use lintdiff_domain::{ingest_on_diff, IngestOnDiffParams};
```

**After:**
```rust
use lintdiff_ingest_core::{ingest_on_diff, IngestOnDiffParams};
```

### Step 3: Verify and Test

```bash
cargo check
cargo test
```

## API Compatibility

The deprecated façade crates re-export the same types from `lintdiff-ingest-core`:

- `ingest_on_diff` function
- `IngestOnDiffParams` struct

**No API changes are required** - only import paths and dependency names need to be updated.

## Idempotency

The script is designed to be **idempotent** - it can be safely run multiple times without causing issues:

- If a file has already been migrated, it won't be modified again
- If no deprecated crates are found, the script reports "No changes were needed"

## Troubleshooting

### Script doesn't find any files

- Ensure you're passing the correct project directory path
- Check that your project has a `Cargo.toml` file
- Verify that Rust source files have `.rs` extension

### Compilation errors after migration

1. Ensure you've updated all workspace crates if using a workspace
2. Run `cargo clean` and then `cargo build`
3. Check for any hardcoded crate names in build scripts or macros

### Script fails on Windows

- Run the script in WSL (Windows Subsystem for Linux)
- Or use Git Bash
- Or follow the manual migration steps above

## Related Documents

- [Deprecation Plan](../deprecation-plan.md)
- [Migration Guide](../migration-guide.md)
- [PR-101: App Migration](PR-101-app-migration.md)
- [PR-102: BDD Harness Migration](PR-102-bdd-harness-migration.md)
