# Migration Examples: Façade Crate Deprecation

This document provides concrete, copy-paste friendly examples for migrating from deprecated façade crates to `lintdiff-ingest-core`.

## Quick Reference

| Deprecated Crate | Replacement |
|------------------|-------------|
| `lintdiff-domain` | `lintdiff-ingest-core` |
| `lintdiff-core` | `lintdiff-ingest-core` |
| `lintdiff-ingest` | `lintdiff-ingest-core` |

---

## Cargo.toml Migration Examples

### Single Dependency Migration

#### From `lintdiff-domain`

```toml
# Before (deprecated)
[dependencies]
lintdiff-domain = "0.4"

# After (recommended)
[dependencies]
lintdiff-ingest-core = "0.4"
```

#### From `lintdiff-core`

```toml
# Before (deprecated)
[dependencies]
lintdiff-core = "0.4"

# After (recommended)
[dependencies]
lintdiff-ingest-core = "0.4"
```

#### From `lintdiff-ingest`

```toml
# Before (deprecated)
[dependencies]
lintdiff-ingest = "0.4"

# After (recommended)
[dependencies]
lintdiff-ingest-core = "0.4"
```

### With Feature Flags

```toml
# Before (deprecated)
[dependencies]
lintdiff-domain = { version = "0.4", features = ["serde"] }

# After (recommended)
[dependencies]
lintdiff-ingest-core = { version = "0.4", features = ["serde"] }
```

### Multiple Deprecated Dependencies

If your project uses multiple deprecated crates:

```toml
# Before (deprecated)
[dependencies]
lintdiff-domain = "0.4"
lintdiff-core = "0.4"

# After (recommended) - Only one dependency needed
[dependencies]
lintdiff-ingest-core = "0.4"
```

### Workspace Configuration

#### Root `Cargo.toml`

```toml
# Before (deprecated)
[workspace.dependencies]
lintdiff-domain = { version = "0.4" }

# After (recommended)
[workspace.dependencies]
lintdiff-ingest-core = { version = "0.4" }
```

#### Member Crate `Cargo.toml`

```toml
# Before (deprecated)
[dependencies]
lintdiff-domain = { workspace = true }

# After (recommended)
[dependencies]
lintdiff-ingest-core = { workspace = true }
```

---

## Rust Code Migration Examples

### Basic Import Migration

#### From `lintdiff-domain`

```rust
// Before (deprecated)
use lintdiff_domain::{ingest_on_diff, IngestOnDiffParams};

// After (recommended)
use lintdiff_ingest_core::{ingest_on_diff, IngestOnDiffParams};
```

#### From `lintdiff-core`

```rust
// Before (deprecated)
use lintdiff_core::{verdict::Verdict, policy::Policy};

// After (recommended)
use lintdiff_ingest_core::{verdict::Verdict, policy::Policy};
```

#### From `lintdiff-ingest`

```rust
// Before (deprecated)
use lintdiff_ingest::match_diagnostics;

// After (recommended)
use lintdiff_ingest_core::match_diagnostics;
```

### Complete Function Example

```rust
// Before (deprecated)
use lintdiff_domain::{ingest_on_diff, IngestOnDiffParams};
use lintdiff_domain::verdict::Verdict;

fn run_lintdiff() -> Verdict {
    let params = IngestOnDiffParams {
        diff_content: "...".to_string(),
        diagnostics_content: "...".to_string(),
        // ... other fields
    };
    ingest_on_diff(params).verdict
}

// After (recommended)
use lintdiff_ingest_core::{ingest_on_diff, IngestOnDiffParams};
use lintdiff_ingest_core::verdict::Verdict;

fn run_lintdiff() -> Verdict {
    let params = IngestOnDiffParams {
        diff_content: "...".to_string(),
        diagnostics_content: "...".to_string(),
        // ... other fields
    };
    ingest_on_diff(params).verdict
}
```

### Re-exports in Library Code

If your library re-exports types from the deprecated crates:

```rust
// Before (deprecated)
pub use lintdiff_domain::Verdict;
pub use lintdiff_domain::Policy;

// After (recommended)
pub use lintdiff_ingest_core::Verdict;
pub use lintdiff_ingest_core::Policy;
```

### Module-Level Imports

```rust
// Before (deprecated)
use lintdiff_domain::{
    ingest_on_diff,
    IngestOnDiffParams,
    verdict::{Verdict, VerdictStatus},
    policy::{Policy, PolicyRule},
};

// After (recommended)
use lintdiff_ingest_core::{
    ingest_on_diff,
    IngestOnDiffParams,
    verdict::{Verdict, VerdictStatus},
    policy::{Policy, PolicyRule},
};
```

---

## Step-by-Step Manual Migration

Follow these steps if you prefer to migrate manually or if the automated script is unavailable.

### Step 1: Identify Affected Files

Find all files that reference deprecated crates:

```bash
# Unix/Linux/macOS
grep -r "lintdiff_domain\|lintdiff_core\|lintdiff_ingest::" --include="*.rs" .
grep -r "lintdiff-domain\|lintdiff-core\|lintdiff-ingest" --include="Cargo.toml" .

# Windows PowerShell
Select-String -Path "*.rs" -Pattern "lintdiff_domain|lintdiff_core|lintdiff_ingest::" -Recurse
Select-String -Path "Cargo.toml" -Pattern "lintdiff-domain|lintdiff-core|lintdiff-ingest" -Recurse
```

### Step 2: Update Cargo.toml

For each `Cargo.toml` file found:

1. Replace `lintdiff-domain` with `lintdiff-ingest-core`
2. Replace `lintdiff-core` with `lintdiff-ingest-core`
3. Replace `lintdiff-ingest =` with `lintdiff-ingest-core =`

### Step 3: Update Rust Source Files

For each `.rs` file found:

1. Replace `lintdiff_domain::` with `lintdiff_ingest_core::`
2. Replace `lintdiff_core::` with `lintdiff_ingest_core::`
3. Replace `lintdiff_ingest::` with `lintdiff_ingest_core::`

### Step 4: Verify the Migration

```bash
# Clean build artifacts
cargo clean

# Check for compilation errors
cargo check

# Run tests
cargo test
```

### Step 5: Review and Commit

```bash
# Review changes
git diff

# Stage and commit
git add -A
git commit -m "chore: migrate from deprecated façade crates to lintdiff-ingest-core"
```

---

## Common Patterns and Migrations

### Pattern: Using `ingest_on_diff` Function

```rust
// Before (deprecated)
use lintdiff_domain::ingest_on_diff;

let result = ingest_on_diff(params);

// After (recommended)
use lintdiff_ingest_core::ingest_on_diff;

let result = ingest_on_diff(params);
```

### Pattern: Working with Verdicts

```rust
// Before (deprecated)
use lintdiff_core::verdict::{Verdict, VerdictStatus};

fn check_verdict(verdict: &Verdict) -> bool {
    matches!(verdict.status, VerdictStatus::Pass)
}

// After (recommended)
use lintdiff_ingest_core::verdict::{Verdict, VerdictStatus};

fn check_verdict(verdict: &Verdict) -> bool {
    matches!(verdict.status, VerdictStatus::Pass)
}
```

### Pattern: Policy Configuration

```rust
// Before (deprecated)
use lintdiff_domain::policy::{Policy, PolicyRule};

let policy = Policy {
    rules: vec![PolicyRule::default()],
};

// After (recommended)
use lintdiff_ingest_core::policy::{Policy, PolicyRule};

let policy = Policy {
    rules: vec![PolicyRule::default()],
};
```

### Pattern: Diagnostic Matching

```rust
// Before (deprecated)
use lintdiff_ingest::match_diagnostics;

let matches = match_diagnostics(&diff, &diagnostics);

// After (recommended)
use lintdiff_ingest_core::match_diagnostics;

let matches = match_diagnostics(&diff, &diagnostics);
```

### Pattern: Struct Initialization

```rust
// Before (deprecated)
use lintdiff_domain::IngestOnDiffParams;

let params = IngestOnDiffParams {
    diff_content: diff.to_string(),
    diagnostics_content: json.to_string(),
    policy: None,
};

// After (recommended)
use lintdiff_ingest_core::IngestOnDiffParams;

let params = IngestOnDiffParams {
    diff_content: diff.to_string(),
    diagnostics_content: json.to_string(),
    policy: None,
};
```

---

## Automated Migration

For automated migration, use the provided script:

```bash
# From the lintdiff repository
./scripts/migrate-to-ingest-core.sh /path/to/your/project
```

See [PR-106: Automated Migration Script](../migration-plans/PR-106-automated-migration-script.md) for detailed instructions.

---

## Troubleshooting

### "cannot find type in crate `lintdiff_ingest_core`"

Run `cargo clean` and rebuild:

```bash
cargo clean
cargo build
```

### "multiple versions of crate" Error

Check your dependency tree:

```bash
cargo tree | grep lintdiff
```

Ensure all dependencies use the same version.

### Deprecation Warnings Persist

Search for remaining old imports:

```bash
grep -r "lintdiff_domain\|lintdiff_core\|lintdiff_ingest::" src/
```

---

## Related Documentation

- [Migration Guide](../migration-guide.md) - Comprehensive migration documentation
- [Deprecation Plan](../deprecation-plan.md) - Technical details of the deprecation
- [PR-106: Automated Migration Script](../migration-plans/PR-106-automated-migration-script.md) - Script documentation
