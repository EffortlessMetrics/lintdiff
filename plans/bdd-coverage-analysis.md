# BDD Test Coverage Analysis for lintdiff

## Executive Summary

The lintdiff project has a comprehensive BDD test suite with approximately **200 scenarios** in [`lintdiff.feature`](../crates/lintdiff-cli/tests/features/lintdiff.feature). However, analysis reveals **~50+ scenarios marked with `@skip`** that need implementation, and several **coverage gaps** when compared against the requirements in [`docs/requirements.md`](../docs/requirements.md).

---

## Current Coverage Analysis

### Scenario Categories

| Category | Line Range | Count | Status |
|----------|------------|-------|--------|
| Core Functionality | 1-100 | ~15 | ✅ Active |
| Rendering (Markdown/Annotations) | 74-140 | ~12 | ✅ Active |
| Path Matching | 140-200 | ~10 | ✅ Active |
| End-to-end Workflows | 186-224 | ~5 | ✅ Active |
| --fail-on Flag | 225-260 | ~8 | ✅ Active |
| Explain Artifact | 253-292 | ~6 | ✅ Active |
| Edge Cases (rename, move, macro) | 293-340 | ~6 | ✅ Active |
| Extended Path Filtering | 342-410 | ~12 | Mostly Active |
| Feature Flag Matrix | 412-465 | ~8 | ✅ Active |
| Verdict/Code Policy | 466-725 | ~20 | Mostly Active |
| Fingerprint Stability | 726-742 | ~2 | ✅ Active |
| CLI Subcommands | 844-870 | ~5 | ✅ Active |
| Error Handling | 872-900 | ~5 | Partially Skipped |
| Configuration Options | 901-962 | ~10 | Partially Skipped |
| Integration Scenarios | 963-1010 | ~8 | Partially Skipped |
| Path Edge Cases | 1012-1050 | ~6 | ✅ Active |
| Finding Field Coverage | 1052-1098 | ~6 | Partially Skipped |
| Additional CLI Flags | 1100-1140 | ~6 | All Skipped |
| Report Structure | 1142-1210 | ~10 | Partially Skipped |
| HIGH Priority | 1211-1352 | ~15 | Mostly Skipped |
| MEDIUM Priority | 1354-1570 | ~25 | Partially Skipped |
| LOW Priority | 1582-1657 | ~12 | Partially Skipped |

### Skipped Scenarios Summary

Approximately **50+ scenarios** are marked with `@skip`:

| Area | Skipped Count | Impact |
|------|---------------|--------|
| Exit Code Testing | 5 | HIGH - Critical for CI integration |
| CLI Flags (--help, --quiet, --verbose, --output) | 6 | MEDIUM - User experience |
| Error Conditions (invalid JSON, corrupted input) | 4 | HIGH - Robustness |
| Report Fields (host, git, duration) | 8 | MEDIUM - Observability |
| Finding Fields (help, url, data) | 4 | LOW - Extended features |
| Config Options (workspace_only, allow_codes) | 3 | MEDIUM - Policy control |
| Provenance | 2 | LOW - Tracking |

---

## Requirements Coverage Matrix

Comparing against [`docs/requirements.md`](../docs/requirements.md):

### ✅ Well Covered

| Requirement | Test Coverage |
|-------------|---------------|
| Warning on changed line → finding | `Scenario: Warning on changed line becomes a finding` |
| Warning outside diff → ignored | `Scenario: Warning outside the diff is ignored` |
| Missing diagnostics → skip | `Scenario: Missing diagnostics yields skip` |
| Deny codes upgrade to error | `Scenario: Deny-listed code upgrades to error and fails` |
| Path filters | Multiple scenarios in Extended path filtering |
| Feature flags | Feature flag matrix scenarios |
| fail_on policy | Multiple fail_on scenarios |
| Markdown rendering | Multiple markdown scenarios |
| GitHub annotations | Multiple annotation scenarios |
| Report schema validation | `Scenario: Report schema validates` |
| Determinism | `Scenario: Report is deterministic` |
| Finding ordering | Implicitly tested via fingerprint scenarios |

### ⚠️ Partially Covered

| Requirement | Gap |
|-------------|-----|
| Exit code 0 (pass/warn) | Scenarios exist but marked @skip |
| Exit code 1 (tool error) | Scenarios exist but marked @skip |
| Exit code 2 (policy failure) | Scenarios exist but marked @skip |
| Invalid diagnostics JSON | Scenario marked @skip |
| Invalid diff | Basic test exists, edge cases skipped |
| workspace_only flag | Scenario marked @skip |
| allow_codes config | Scenario marked @skip |
| max_findings truncation | Tested but truncation marker not verified |
| max_annotations | Partially tested |

### ❌ Not Covered / Gaps

| Requirement | Status |
|-------------|--------|
| Empty diagnostics → skip with reason | @skip |
| Empty diff → pass | @skip |
| Info severity diagnostics | @skip |
| Finding help field | @skip |
| Finding URL field | @skip |
| Report git.head_sha | @skip |
| Report git.head_ref | @skip |
| Report run.host.os/arch | @skip |
| Report run.duration_ms | @skip |
| Provenance rustc/clippy tracking | @skip |
| --help flag output | @skip |
| --quiet flag | @skip |
| --verbose flag | @skip |
| --output json flag | @skip |
| --no-color flag | @skip |
| Explain subcommand unknown codes | @skip |
| Multi-line diagnostic messages | @skip |
| Special characters in messages | @skip |

---

## Identified Coverage Gaps

### HIGH Priority Gaps

1. **Exit Code Testing** - Critical for CI integration
   - Exit code 0 for pass/warn/skip
   - Exit code 1 for tool errors
   - Exit code 2 for policy failures

2. **Error Condition Handling** - Robustness
   - Invalid diagnostics JSON parsing
   - Empty diagnostics handling
   - Corrupted JSONL input

3. **Missing Diff Source** - User guidance
   - Both base/head and diff-file missing
   - Base without head
   - Head without base

### MEDIUM Priority Gaps

4. **CLI Flag Coverage**
   - `--help` output format
   - `--quiet` output suppression
   - `--verbose` detailed output
   - `--output json` format
   - `--no-color` ANSI suppression

5. **Configuration Options**
   - `workspace_only` filtering
   - `allow_codes` permitting
   - Config file with all fields

6. **Report Fields**
   - Git info (head_sha, head_ref)
   - Host info (os, arch)
   - Duration tracking

### LOW Priority Gaps

7. **Extended Finding Fields**
   - `help` field
   - `url` field
   - `data` field

8. **Provenance Tracking**
   - Rustc provenance
   - Clippy provenance

9. **Edge Cases**
   - Multi-line messages
   - Unicode in messages
   - Large file handling

---

## Proposed New Scenarios

### HIGH Priority - Exit Codes

```gherkin
Scenario: Exit code 0 for pass verdict
  Given a diff fixture "simple_addition.diff"
  And a diagnostics fixture "warning_outside_diff.jsonl"
  When lintdiff runs full pipeline
  Then exit code is 0

Scenario: Exit code 0 for warn verdict with default fail_on
  Given a diff fixture "simple_addition.diff"
  And a diagnostics fixture "warning_on_changed_line.jsonl"
  When lintdiff runs full pipeline
  Then exit code is 0

Scenario: Exit code 0 for skip verdict
  Given a diff fixture "simple_addition.diff"
  And empty diagnostics
  When lintdiff runs full pipeline
  Then exit code is 0

Scenario: Exit code 2 for fail verdict with deny code
  Given a diff fixture "simple_addition.diff"
  And a diagnostics fixture "warning_on_changed_line.jsonl"
  And deny code "lintdiff.diagnostic.clippy.let_unit_value"
  When lintdiff runs full pipeline
  Then exit code is 2

Scenario: Exit code 2 for fail_on warn with warnings
  Given a diff fixture "simple_addition.diff"
  And a diagnostics fixture "warning_on_changed_line.jsonl"
  And fail_on is "warn"
  When lintdiff runs full pipeline
  Then exit code is 2
```

### HIGH Priority - Error Conditions

```gherkin
Scenario: Invalid diagnostics JSON produces parse error
  Given a diff fixture "simple_addition.diff"
  And raw diagnostics "{ invalid json }"
  When lintdiff ingests the inputs
  Then error message contains "parse"
  And exit code is 2

Scenario: Empty diagnostics produces skip with reason
  Given a diff fixture "simple_addition.diff"
  And empty diagnostics
  When lintdiff ingests the inputs
  Then verdict status is "skip"
  And report has field "verdict.reasons"

Scenario: Corrupted JSONL produces error
  Given a diff fixture "simple_addition.diff"
  And corrupted diagnostics JSONL
  When lintdiff ingests the inputs
  Then error message contains "parse"
  And exit code is 2
```

### MEDIUM Priority - CLI Flags

```gherkin
Scenario: Help flag outputs usage information
  When lintdiff runs with flag "--help"
  Then output contains "USAGE"
  And output contains "OPTIONS"
  And exit code is 0

Scenario: Quiet flag suppresses stdout
  Given a diff fixture "simple_addition.diff"
  And a diagnostics fixture "warning_on_changed_line.jsonl"
  When lintdiff runs with flag "--quiet"
  Then stdout is empty

Scenario: JSON output flag produces valid JSON
  Given a diff fixture "simple_addition.diff"
  And a diagnostics fixture "warning_on_changed_line.jsonl"
  When lintdiff runs with flag "--output json"
  Then output is valid JSON
  And JSON has field "version"
  And JSON has field "findings"
```

### MEDIUM Priority - Configuration

```gherkin
Scenario: workspace_only filters non-workspace paths
  Given workspace_only is true
  And a diff fixture "multi_file.diff"
  And a diagnostics fixture with absolute path "/usr/local/lib/rustlib/src/rust/src/lib.rs"
  When lintdiff ingests the inputs
  Then verdict status is "pass"

Scenario: allow_codes permits specific codes
  Given allow code "lintdiff.diagnostic.clippy.let_unit_value"
  And a diff fixture "simple_addition.diff"
  And a diagnostics fixture "warning_on_changed_line.jsonl"
  When lintdiff ingests the inputs
  Then verdict status is "pass"
```

### MEDIUM Priority - Report Fields

```gherkin
Scenario: Report includes git info when available
  Given a diff fixture "simple_addition.diff"
  And a diagnostics fixture "warning_on_changed_line.jsonl"
  And git repository is available
  When lintdiff ingests the inputs
  Then report has field "run.git.head_sha"
  And report has field "run.git.head_ref"

Scenario: Report includes duration
  Given a diff fixture "simple_addition.diff"
  And a diagnostics fixture "warning_on_changed_line.jsonl"
  When lintdiff ingests the inputs
  Then report has field "run.duration_ms"
  And report field "run.duration_ms" is greater than 0
```

---

## Implementation Recommendations

### Phase 1: Enable Skipped Scenarios (HIGH Priority)

1. **Exit Code Scenarios** - Remove @skip and implement step definitions
2. **Error Condition Scenarios** - Remove @skip and add fixtures
3. **Missing Diff Source Scenarios** - Already active, verify coverage

### Phase 2: Add Missing Scenarios (MEDIUM Priority)

4. **CLI Flag Scenarios** - Add new scenarios for --help, --quiet, --verbose
5. **Configuration Scenarios** - Add workspace_only, allow_codes tests
6. **Report Field Scenarios** - Add git info, duration tests

### Phase 3: Extended Coverage (LOW Priority)

7. **Finding Field Scenarios** - Add help, url, data field tests
8. **Provenance Scenarios** - Add rustc/clippy tracking tests
9. **Edge Case Scenarios** - Add multi-line, unicode tests

---

## Infrastructure Assessment

### BDD Harness Quality

The [`lintdiff-bdd-harness`](../crates/lintdiff-bdd-harness/src/lib.rs) provides:

- ✅ `run_ingest_from_fixtures()` - Core ingestion
- ✅ `apply_feature_flag_value()` - Feature flag support
- ✅ `verdict_status()` - Status extraction
- ✅ `read_fixture()` - Fixture loading

### Missing Step Definitions

The following step definitions need implementation in [`bdd.rs`](../crates/lintdiff-cli/tests/bdd.rs):

- `stdout is empty` - For --quiet testing
- `output is valid JSON` - For --output json testing
- `output contains no ANSI codes` - For --no-color testing
- `report field {string} is greater than {int}` - For duration testing
- `git repository is available` - For git info testing

---

## Conclusion

The lintdiff BDD test suite is comprehensive but has approximately **25% of scenarios marked as skipped**. The highest priority gaps are:

1. **Exit code testing** - Critical for CI integration
2. **Error condition handling** - Important for robustness
3. **CLI flag coverage** - Important for user experience

Enabling the skipped scenarios and adding the proposed new scenarios would bring coverage to near-complete levels against the documented requirements.
