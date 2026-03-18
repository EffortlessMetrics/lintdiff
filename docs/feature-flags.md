# Feature Flags

This document describes lintdiff's feature flag system, which allows fine-grained control over matching and filtering behavior.

## Overview

Feature flags in lintdiff provide runtime configuration for enabling or disabling specific behaviors. Flags can be set via:

- Configuration file (`lintdiff.toml`)
- Command-line arguments
- Environment variables (in some contexts)

All feature flags have sensible defaults and are designed to be optional for typical usage.

## Available Flags

| Flag | Default | Description |
|------|---------|-------------|
| `primary_span_matching` | `true` | Prefers primary spans when matching diagnostics against diff lines |
| `path_filters` | `true` | Applies include/exclude path filters against normalized paths |

## Accepted Boolean Values

Feature flags accept multiple boolean representations (case-insensitive):

| True Values | False Values |
|-------------|--------------|
| `true` | `false` |
| `1` | `0` |
| `on` | `off` |
| `enabled` | `disabled` |
| `yes` | `no` |

## Flag Details

### `primary_span_matching`

**Default**: `true`

Controls how lintdiff selects spans from diagnostics when matching against diff lines.

#### When Enabled (default)

Lintdiff prefers the **primary span** from each diagnostic when available. A primary span is the main location associated with a diagnostic - typically where the error or warning originates.

**Behavior**:
1. If a diagnostic has one or more primary spans, only those spans are considered for matching
2. If no primary spans exist, all spans are considered
3. This improves matching accuracy by focusing on the most relevant location

**Example**:

```json
{
  "message": "cannot borrow `x` as mutable more than once at a time",
  "spans": [
    {"file_name": "src/lib.rs", "line_start": 10, "is_primary": false},
    {"file_name": "src/lib.rs", "line_start": 15, "is_primary": true}
  ]
}
```

With `primary_span_matching=true`, only line 15 is matched against the diff.

#### When Disabled

All spans from each diagnostic are considered for matching, regardless of whether they are marked as primary. This can be useful when:

- Diagnostics have inaccurate primary span markings
- You want broader matching coverage
- Debugging span-related issues

### `path_filters`

**Default**: `true`

Controls whether include/exclude path filters are applied when processing diagnostics.

#### When Enabled (default)

Path filters from the `[filter]` section of your configuration are applied:

- **`include_paths`**: Only diagnostics from matching paths are processed
- **`exclude_paths`**: Diagnostics from matching paths are ignored

**Example configuration**:

```toml
[filter]
include_paths = ["src/**"]
exclude_paths = ["**/generated/**", "**/target/**"]
```

**Behavior**:
1. Paths are normalized before matching (e.g., `./src/lib.rs` → `src/lib.rs`)
2. Exclude patterns take precedence over include patterns
3. Glob patterns support `*` (single level) and `**` (multi-level) wildcards

#### When Disabled

All path filters are bypassed, and diagnostics from all files are processed regardless of the `[filter]` configuration. This can be useful when:

- Temporarily seeing all diagnostics without modifying config
- Debugging path filter issues
- Running in environments where path normalization differs

## CLI Usage

### Setting Individual Flags

Use the `--feature` flag to set feature values:

```bash
# Disable primary span matching
lintdiff --feature primary_span_matching=false

# Enable path filters explicitly
lintdiff --feature path_filters=enabled

# Multiple flags
lintdiff --feature primary_span_matching=off --feature path_filters=on
```

### Using with Other Options

```bash
# Combine with diff and diagnostics files
lintdiff diff.patch diagnostics.jsonl --feature path_filters=false

# Use in CI pipelines
lintdiff --feature primary_span_matching=true --fail-on error
```

## Configuration File

Add a `[features]` section to your `lintdiff.toml`:

```toml
# lintdiff.toml

[features]
# Prefer primary spans for accurate matching
primary_span_matching = true

# Apply path filters from [filter] section
path_filters = true

[filter]
include_paths = ["src/**"]
exclude_paths = ["**/tests/**", "**/generated/**"]
```

### Complete Example

```toml
# lintdiff.toml - Complete configuration example

# Global settings
fail_on = "error"
max_findings = 200

[features]
primary_span_matching = true
path_filters = true

[filter]
include_paths = ["src/**", "lib/**"]
exclude_paths = ["**/target/**", "**/generated/**", "**/*.generated.rs"]

[provenance]
record_rustc = true
record_clippy = true
```

## How Flags Affect Behavior

### Decision Flow

```
Diagnostic received
        │
        ▼
┌───────────────────┐
│ path_filters=true?│
└───────────────────┘
        │
   ┌────┴────┐
   │         │
  Yes        No
   │         │
   ▼         │
Path      │
Allowed?     │
   │         │
┌──┴──┐      │
│     │      │
Yes   No     │
│     │      │
│     └──────┤
│            │
▼            ▼
┌─────────────────────────┐
│ primary_span_matching=? │
└─────────────────────────┘
        │
   ┌────┴────┐
   │         │
  Yes        No
   │         │
   ▼         │
Filter to   │
Primary     │
Spans       │
   │         │
   └────┬────┘
        │
        ▼
   Match against
     diff lines
```

### Behavior Matrix

| `path_filters` | `primary_span_matching` | Result |
|----------------|------------------------|--------|
| `true` | `true` | Filtered paths, primary spans only (default, recommended) |
| `true` | `false` | Filtered paths, all spans |
| `false` | `true` | All paths, primary spans only |
| `false` | `false` | All paths, all spans (most permissive) |

## Troubleshooting

### Diagnostics Not Being Matched

**Symptom**: Expected diagnostics are not appearing in results.

**Diagnostic steps**:

1. **Check path filters**:
   ```bash
   # Temporarily disable path filters to see all diagnostics
   lintdiff --feature path_filters=false diff.patch diagnostics.jsonl
   ```

2. **Check span matching**:
   ```bash
   # Disable primary span matching to see if spans are the issue
   lintdiff --feature primary_span_matching=false diff.patch diagnostics.jsonl
   ```

3. **Verify file paths**: Ensure paths in diagnostics match your filter patterns. Paths are normalized before matching.

### Too Many Results

**Symptom**: Getting diagnostics from files that should be excluded.

**Solutions**:

1. Verify `path_filters` is enabled (default: `true`)
2. Check your glob patterns in `exclude_paths`
3. Remember: exclude takes precedence over include

### Flag Not Being Recognized

**Symptom**: Error message "unknown feature flag: X"

**Solutions**:

1. Check spelling (flags use snake_case)
2. Verify the flag exists in the [Available Flags](#available-flags) table
3. Ensure you're using `=` for assignment: `--feature flag=value`

### Invalid Boolean Value

**Symptom**: Error message about unknown feature flag value

**Solutions**:

1. Use one of the [Accepted Boolean Values](#accepted-boolean-values)
2. Values are case-insensitive, but must be spelled correctly
3. Example valid values: `true`, `FALSE`, `On`, `OFF`, `1`, `0`

## Contributor Guide

### Adding a New Feature Flag

Feature flags are defined in [`crates/lintdiff-feature-flags/src/lib.rs`](../crates/lintdiff-feature-flags/src/lib.rs).

#### Step 1: Add to the Enum

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeatureFlag {
    PrimarySpanMatching,
    PathFilters,
    YourNewFlag,  // Add here
}
```

#### Step 2: Add to the Registry

```rust
pub const FEATURE_FLAGS: &[FeatureFlagSpec] = &[
    // ... existing flags ...
    FeatureFlagSpec {
        id: FeatureFlag::YourNewFlag,
        key: "your_new_flag",
        description: "Description of what this flag controls.",
        default_enabled: true,  // or false
    },
];
```

#### Step 3: Implement Trait Methods

```rust
impl FeatureFlag {
    pub const fn as_str(self) -> &'static str {
        match self {
            // ... existing patterns ...
            Self::YourNewFlag => "your_new_flag",
        }
    }

    pub const fn default_enabled(self) -> bool {
        match self {
            // ... existing patterns ...
            Self::YourNewFlag => true,  // or false
        }
    }
}
```

#### Step 4: Add to FeatureFlags Struct

In [`crates/lintdiff-types/src/config.rs`](../crates/lintdiff-types/src/config.rs):

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FeatureFlags {
    #[serde(default = "default_true")]
    pub prefer_primary_spans: bool,
    #[serde(default = "default_true")]
    pub path_filters: bool,
    #[serde(default = "default_true")]  // or default_false
    pub your_new_flag: bool,  // Add field
}
```

#### Step 5: Wire Up the Setter

```rust
pub fn set_feature_flag(flags: &mut FeatureFlags, flag: FeatureFlag, enabled: bool) {
    match flag {
        FeatureFlag::PrimarySpanMatching => flags.prefer_primary_spans = enabled,
        FeatureFlag::PathFilters => flags.path_filters = enabled,
        FeatureFlag::YourNewFlag => flags.your_new_flag = enabled,  // Add case
    }
}
```

#### Step 6: Implement the Behavior

Use the flag in your matching/filtering logic:

```rust
if config.features.your_new_flag {
    // Apply the behavior
}
```

#### Step 7: Update Documentation

1. Add the flag to the [Available Flags](#available-flags) table
2. Add a detailed description in [Flag Details](#flag-details)
3. Update behavior matrix if relevant

### Testing

Add tests for:

1. Flag parsing (`parse_feature_flag_assignment`)
2. Default value behavior
3. Enabled/disabled behavior in the relevant component
4. Integration with configuration file loading
