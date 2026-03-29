Feature: Diff-scoped diagnostics

  Scenario: Warning on changed line becomes a finding
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  Scenario: Warning outside the diff is ignored
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_outside_diff.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And warn count is 0

  Scenario: Missing diagnostics yields skip
    Given a diff fixture "simple_addition.diff"
    When lintdiff ingests the inputs
    Then verdict status is "skip"

  Scenario: Deny-listed code upgrades to error and fails
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And deny code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    Then verdict status is "fail"
    And error count is 1

  Scenario: Primary span selection is configurable for fallback span matching
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "mixed_spans.jsonl"
    And feature flag "primary_span_matching" is "false"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  Scenario Outline: Primary-span feature flag matrix
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "mixed_spans.jsonl"
    And feature flag "primary_span_matching" is "<primary_span_matching>"
    When lintdiff ingests the inputs
    Then verdict status is "<status>"
    And warn count is <warn>
    And error count is 0

    Examples:
      | primary_span_matching | status | warn |
      | false                 | warn   | 1    |
      | true                  | pass   | 0    |

  Scenario: Path filters are enforced when enabled
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And filter exclude path "src/lib.rs"
    And feature flag "path_filters" is "true"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And warn count is 0

  Scenario: Path filters can be disabled at runtime
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And filter exclude path "src/lib.rs"
    And feature flag "path_filters" is "false"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  # =============================================================================
  # Rendering scenarios (lintdiff-render)
  # =============================================================================

  Scenario: Markdown rendering shows pass status for clean diff
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_outside_diff.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders markdown output
    Then markdown output contains "PASS"
    And markdown output contains status badge
    And markdown output contains counts summary

  Scenario: Markdown rendering shows findings table for warnings
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders markdown output
    Then markdown output contains "WARN"
    And markdown output contains findings table
    And markdown output contains "src/lib.rs"
    And markdown output contains "lintdiff.diagnostic.clippy.let_unit_value"

  Scenario: Markdown rendering shows skip status for missing diagnostics
    Given a diff fixture "simple_addition.diff"
    When lintdiff ingests the inputs
    And lintdiff renders markdown output
    Then markdown output contains "SKIP"
    And markdown output contains "skipped"

  Scenario: Markdown rendering shows fail status for errors
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And deny code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    And lintdiff renders markdown output
    Then markdown output contains "FAIL"
    And markdown output contains counts summary

  Scenario: Markdown rendering truncates long findings list
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders markdown output with max items 0
    Then markdown output contains "And 1 more"

  Scenario: GitHub annotations format for warnings
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders GitHub annotations
    Then GitHub annotations output contains "::warning"
    Then GitHub annotations output contains "file=src/lib.rs"
    Then GitHub annotations output contains "line=1"
    Then GitHub annotations output contains "lintdiff.diagnostic.clippy.let_unit_value"

  Scenario: GitHub annotations empty for clean diff
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_outside_diff.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders GitHub annotations
    Then GitHub annotations output is empty

  Scenario: GitHub annotations count matches findings count
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders GitHub annotations
    Then GitHub annotations count is 1

  # =============================================================================
  # Path matching scenarios (lintdiff-match)
  # =============================================================================

  Scenario: Path allowed with no filters
    Given a test path "src/lib.rs"
    When lintdiff checks path against filters
    Then path is allowed

  Scenario: Path excluded by exclude pattern
    Given filter exclude path "src/lib.rs"
    And a test path "src/lib.rs"
    When lintdiff checks path against filters
    Then path is filtered out

  Scenario: Path allowed when not matching exclude pattern
    Given filter exclude path "src/lib.rs"
    And a test path "src/main.rs"
    When lintdiff checks path against filters
    Then path is allowed

  Scenario: Path allowed by include pattern
    Given filter include path "src/**/*.rs"
    And a test path "src/lib.rs"
    When lintdiff checks path against filters
    Then path is allowed

  Scenario: Path filtered out when not matching include pattern
    Given filter include path "src/**/*.rs"
    And a test path "tests/integration.rs"
    When lintdiff checks path against filters
    Then path is filtered out

  Scenario: Glob pattern matches multiple files
    Given filter exclude path "**/*.generated.rs"
    And a test path "src/api.generated.rs"
    When lintdiff checks path against filters
    Then path is filtered out

  Scenario: Glob pattern does not match non-matching files
    Given filter exclude path "**/*.generated.rs"
    And a test path "src/api.rs"
    When lintdiff checks path against filters
    Then path is allowed

  # =============================================================================
  # End-to-end workflow scenarios (lintdiff-app integration)
  # =============================================================================

  Scenario: Full pipeline produces consistent output
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff runs full pipeline
    Then verdict status is "warn"
    And findings count is 1
    And markdown output contains "WARN"
    And GitHub annotations count is 1

  Scenario: Full pipeline with denied code produces error
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And deny code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff runs full pipeline
    Then verdict status is "fail"
    And finding 0 has severity "error"

  Scenario: Full pipeline with filtered path produces no findings
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And filter exclude path "src/lib.rs"
    And feature flag "path_filters" is "true"
    When lintdiff runs full pipeline
    Then verdict status is "pass"
    And findings count is 0
    And GitHub annotations output is empty

  Scenario: Equivalent whitespace diagnostics keep stable fingerprints
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 2
    And finding 0 and 1 share fingerprint

  # =============================================================================
  # --fail-on flag scenarios
  # =============================================================================

  Scenario: --fail-on warn causes exit on warnings
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And fail_on is "warn"
    When lintdiff ingests the inputs
    Then verdict status is "fail"
    And warn count is 1

  Scenario: --fail-on never does not fail on warnings only
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And fail_on is "never"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  Scenario: --fail-on error does not fail on warnings only
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And fail_on is "error"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  # =============================================================================
  # Explain artifact scenarios
  # =============================================================================

  Scenario: Explain artifact tracks all diagnostics
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then explain total equals diagnostics total
    And explain has 1 entries with disposition "included"

  Scenario: Explain artifact records outside-diff disposition
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_outside_diff.jsonl"
    When lintdiff ingests the inputs
    Then explain has 1 entries with disposition "dropped_outside_diff"
    And explain has 0 entries with disposition "included"

  Scenario: Explain artifact records no-span disposition
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "no_span_diagnostics.jsonl"
    When lintdiff ingests the inputs
    Then explain has 1 entries with disposition "dropped_no_span"
    And explain has 1 entries with disposition "included"

  Scenario: Explain artifact records suppressed-by-code disposition
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "suppress_code.jsonl"
    And suppress code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    Then explain has 1 entries with disposition "suppressed_by_code"

  Scenario: Explain artifact records path-filter disposition
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And filter exclude path "src/lib.rs"
    And feature flag "path_filters" is "true"
    When lintdiff ingests the inputs
    Then explain has 1 entries with disposition "dropped_by_path_filter"
    And explain has 0 entries with disposition "included"

  # =============================================================================
  # Edge-case fixtures: rename, moved code, multi-span, macro, generated files
  # =============================================================================

  Scenario: Renamed file with diagnostics on new path
    Given a diff fixture "rename.diff"
    And a diagnostics fixture "rename_diagnostics.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  Scenario: Moved code matches diagnostics at new line positions
    Given a diff fixture "moved_code.diff"
    And a diagnostics fixture "moved_code_diagnostics.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 1

  Scenario: Multi-span diagnostic matches primary span in diff
    Given a diff fixture "multi_file.diff"
    And a diagnostics fixture "multi_span.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 1

  Scenario: Macro-expanded diagnostic outside workspace is dropped
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "macro_expanded.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 1

  Scenario: Generated file path can be excluded by filter
    Given a diff fixture "generated_file.diff"
    And a diagnostics fixture "generated_file_diagnostics.jsonl"
    And filter exclude path "src/generated/**"
    And feature flag "path_filters" is "true"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 1

  Scenario: Markdown output includes explain summary
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders markdown output
    Then markdown output contains "Diagnostics:"
    And markdown output contains "1 matched"

  # =============================================================================
  # Extended path filtering scenarios
  # =============================================================================

  Scenario: Exclude specific file patterns with glob
    Given filter exclude path "**/*.test.rs"
    And a test path "src/utils.test.rs"
    When lintdiff checks path against filters
    Then path is filtered out

  Scenario: Exclude multiple patterns with different extensions
    Given filter exclude path "**/*.generated.rs"
    And filter exclude path "**/*.min.js"
    And a test path "src/api.generated.rs"
    When lintdiff checks path against filters
    Then path is filtered out

  Scenario: Exclude nested directory patterns
    Given filter exclude path "vendor/**"
    And a test path "vendor/external/crate.rs"
    When lintdiff checks path against filters
    Then path is filtered out

  Scenario: Include only specific paths with single pattern
    Given filter include path "src/**/*.rs"
    And a test path "src/deeply/nested/module.rs"
    When lintdiff checks path against filters
    Then path is allowed

  Scenario: Include only specific paths filters out non-matching
    Given filter include path "src/**/*.rs"
    And a test path "tests/integration.rs"
    When lintdiff checks path against filters
    Then path is filtered out

  Scenario: Mixed include/exclude patterns with include taking precedence
    Given filter include path "src/**/*.rs"
    And filter exclude path "src/generated/**"
    And a test path "src/generated/api.rs"
    When lintdiff checks path against filters
    Then path is filtered out

  Scenario: Mixed include/exclude allows non-excluded included paths
    Given filter include path "src/**/*.rs"
    And filter exclude path "src/generated/**"
    And a test path "src/lib.rs"
    When lintdiff checks path against filters
    Then path is allowed

  Scenario: Multiple include patterns allow any matching
    Given filter include path "src/**/*.rs"
    And filter include path "tests/**/*.rs"
    And a test path "tests/unit/test_foo.rs"
    When lintdiff checks path against filters
    Then path is allowed

  Scenario: Path filter with directory wildcard
    Given filter exclude path "**/node_modules/**"
    And a test path "project/node_modules/package/index.js"
    When lintdiff checks path against filters
    Then path is filtered out

    Given filter exclude path "src/lib.rs"
    And feature flag "path_filters" is "false"
    And a test path "src/lib.rs"
    When lintdiff checks path against filters
    Then path is allowed

  # =============================================================================
  # Extended feature flag scenarios
  # =============================================================================

  Scenario: Disable primary span matching uses all spans
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "mixed_spans.jsonl"
    And feature flag "primary_span_matching" is "false"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  Scenario: Primary span matching enabled only uses primary spans
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "mixed_spans.jsonl"
    And feature flag "primary_span_matching" is "true"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And warn count is 0

  Scenario: Path filters disabled ignores exclude patterns
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And filter exclude path "src/lib.rs"
    And feature flag "path_filters" is "false"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  Scenario: Both primary_span_matching and path_filters disabled
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "mixed_spans.jsonl"
    And filter exclude path "src/lib.rs"
    And feature flag "primary_span_matching" is "false"
    And feature flag "path_filters" is "false"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  Scenario Outline: Feature flag combination matrix
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And filter exclude path "src/lib.rs"
    And feature flag "path_filters" is "<path_filters>"
    When lintdiff ingests the inputs
    Then verdict status is "<status>"
    And warn count is <warn>

    Examples:
      | path_filters | status | warn |
      | true         | pass   | 0    |
      | false        | warn   | 1    |

  # =============================================================================
  # Extended verdict scenarios
  # =============================================================================

  Scenario: fail_on=error passes with only warnings
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And fail_on is "error"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  Scenario: fail_on=error fails with denied code
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And deny code "lintdiff.diagnostic.clippy.let_unit_value"
    And fail_on is "error"
    When lintdiff ingests the inputs
    Then verdict status is "fail"
    And error count is 1

  Scenario: fail_on=warn fails with warnings
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And fail_on is "warn"
    When lintdiff ingests the inputs
    Then verdict status is "fail"
    And warn count is 1

  Scenario: fail_on=never passes with warnings
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And fail_on is "never"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  Scenario: fail_on=never warns with errors
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And deny code "lintdiff.diagnostic.clippy.let_unit_value"
    And fail_on is "never"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And error count is 1

  Scenario: Suppressed diagnostics not included in counts
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "suppress_code.jsonl"
    And suppress code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And warn count is 0
    And error count is 0

  Scenario: Suppressed diagnostics recorded in explain
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "suppress_code.jsonl"
    And suppress code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    Then explain has 1 entries with disposition "suppressed_by_code"
    And explain has 0 entries with disposition "included"

  Scenario: Multiple suppress codes work together
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    And suppress code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And warn count is 0

  Scenario Outline: fail_on behavior matrix
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And fail_on is "<fail_on>"
    When lintdiff ingests the inputs
    Then verdict status is "<status>"

    Examples:
      | fail_on | status |
      | never   | warn   |
      | warn    | fail   |
      | error   | warn   |

  # =============================================================================
  # Extended edge case scenarios
  # =============================================================================

  @skip
  @skip
  Scenario: Empty diff produces pass
    Given a diff fixture "empty.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And warn count is 0

  @skip
  Scenario: Diagnostics outside diff produce pass
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_outside_diff.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And warn count is 0

  @skip
  Scenario: Multiple diagnostics on same line all match
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 2

  Scenario: Diagnostic with no span is handled gracefully
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "no_span_diagnostics.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 1

  Scenario: Macro expansion span in workspace matches
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "macro_expanded.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 1

  Scenario: Diagnostic on unchanged line is ignored
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_outside_diff.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And explain has 1 entries with disposition "dropped_outside_diff"

  Scenario: Diagnostic exactly on diff boundary matches
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  @skip
  Scenario: Multiple files in diff with selective diagnostics
    Given a diff fixture "multi_file.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And warn count is 0

  # =============================================================================
  # Complex multi-file scenarios
  # =============================================================================

  Scenario: Multi-file changes with mixed diagnostics
    Given a diff fixture "multi_file.diff"
    And a diagnostics fixture "multi_span.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 1

  Scenario: Renamed file with diagnostics on new name
    Given a diff fixture "rename.diff"
    And a diagnostics fixture "rename_diagnostics.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 1

  @skip
  Scenario: Renamed file with diagnostics on old name ignored
    Given a diff fixture "rename.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And warn count is 0

  Scenario: Moved code matches at new position
    Given a diff fixture "moved_code.diff"
    And a diagnostics fixture "moved_code_diagnostics.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 1

  Scenario: Generated file excluded by path filter
    Given a diff fixture "generated_file.diff"
    And a diagnostics fixture "generated_file_diagnostics.jsonl"
    And filter exclude path "src/generated/**"
    And feature flag "path_filters" is "true"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 1

  Scenario: Generated file included when filter disabled
    Given a diff fixture "generated_file.diff"
    And a diagnostics fixture "generated_file_diagnostics.jsonl"
    And filter exclude path "src/generated/**"
    And feature flag "path_filters" is "false"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 2

  Scenario: Multi-span diagnostic with primary in diff
    Given a diff fixture "multi_file.diff"
    And a diagnostics fixture "multi_span.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 1

  # =============================================================================
  # Code policy scenarios
  # =============================================================================

  Scenario: Deny code upgrades warning to error
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And deny code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    Then verdict status is "fail"
    And error count is 1
    And warn count is 0

  Scenario: Deny code only affects matching codes
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    And deny code "lintdiff.diagnostic.clippy.some_other_code"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And warn count is 2
    And error count is 0

  Scenario: Multiple deny codes work together
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    And deny code "lintdiff.diagnostic.clippy.let_unit_value"
    And deny code "lintdiff.diagnostic.clippy.another_code"
    When lintdiff ingests the inputs
    Then verdict status is "fail"
    And error count is 2

  Scenario: Suppress code removes from findings
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And suppress code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And warn count is 0

  Scenario: Suppress code takes precedence over deny
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And deny code "lintdiff.diagnostic.clippy.let_unit_value"
    And suppress code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And warn count is 0
    And error count is 0

  # =============================================================================
  # Fingerprint stability scenarios
  # =============================================================================

  Scenario: Equivalent diagnostics share fingerprint
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"
    And findings count is 2
    And finding 0 and 1 share fingerprint

  Scenario: Different codes have different fingerprints
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "mixed_spans.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "pass"

  # =============================================================================
  # Markdown rendering edge cases
  # =============================================================================

  Scenario: Markdown output with no findings shows pass
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_outside_diff.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders markdown output
    Then markdown output contains "PASS"
    And markdown output contains status badge

  Scenario: Markdown output with many findings truncates
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders markdown output with max items 1
    Then markdown output contains "And 1 more"

  Scenario: Markdown output includes file paths
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders markdown output
    Then markdown output contains "src/lib.rs"

  Scenario: Markdown output includes diagnostic codes
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders markdown output
    Then markdown output contains "lintdiff.diagnostic.clippy.let_unit_value"

  # =============================================================================
  # GitHub annotations edge cases
  # =============================================================================

  Scenario: GitHub annotations with warnings
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders GitHub annotations
    Then GitHub annotations output contains "::warning"
    And GitHub annotations count is 1

  Scenario: GitHub annotations with errors
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And deny code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    And lintdiff renders GitHub annotations
    Then GitHub annotations output contains "::error"
    And GitHub annotations count is 1

  Scenario: GitHub annotations with multiple findings
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders GitHub annotations
    Then GitHub annotations count is 2

  Scenario: GitHub annotations empty for pass verdict
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_outside_diff.jsonl"
    When lintdiff ingests the inputs
    And lintdiff renders GitHub annotations
    Then GitHub annotations output is empty

  # =============================================================================
  # Explain artifact comprehensive scenarios
  # =============================================================================

  Scenario: Explain artifact with all disposition types
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "no_span_diagnostics.jsonl"
    When lintdiff ingests the inputs
    Then explain total equals diagnostics total
    And explain has 1 entries with disposition "included"
    And explain has 1 entries with disposition "dropped_no_span"

  Scenario: Explain artifact tracks path filtered diagnostics
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And filter exclude path "src/lib.rs"
    And feature flag "path_filters" is "true"
    When lintdiff ingests the inputs
    Then explain has 1 entries with disposition "dropped_by_path_filter"

  Scenario: Explain artifact tracks outside diff diagnostics
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_outside_diff.jsonl"
    When lintdiff ingests the inputs
    Then explain has 1 entries with disposition "dropped_outside_diff"

  Scenario: Explain artifact tracks suppressed diagnostics
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "suppress_code.jsonl"
    And suppress code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    Then explain has 1 entries with disposition "suppressed_by_code"

  # =============================================================================
  # CLI Subcommands scenarios
  # =============================================================================

  Scenario: lintdiff run executes command and captures output
    Given a diff fixture "simple_addition.diff"
    When lintdiff runs command "echo test"
    Then verdict status is "skip"

  Scenario: lintdiff ci github auto-detects environment
    Given environment variable "GITHUB_BASE_REF" is "main"
    And environment variable "GITHUB_SHA" is "abc123"
    When lintdiff runs ci github
    Then exit code is 0

  @skip
  Scenario: lintdiff md renders markdown from report
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff generates markdown
    Then markdown output contains "warning"

  Scenario: lintdiff annotations generates github annotations
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff generates annotations
    Then annotation output contains "::warning"

  # =============================================================================
  # Error Handling scenarios
  # =============================================================================

  Scenario: Invalid diff produces error
    Given raw diff "this is not a valid diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then error message contains "diff"

  @skip
  Scenario: Invalid diagnostics JSON produces error
    Given a diff fixture "simple_addition.diff"
    And raw diagnostics "{ invalid json }"
    When lintdiff ingests the inputs
    Then error message contains "parse"

  Scenario: Missing diff file produces error
    Given a missing diff file
    When lintdiff ingests the inputs
    Then error message contains "not found"

  @skip
  Scenario: Empty diagnostics produces skip verdict
    Given a diff fixture "simple_addition.diff"
    And empty diagnostics
    When lintdiff ingests the inputs
    Then verdict status is "skip"

  # =============================================================================
  # Configuration Options scenarios
  # =============================================================================

  Scenario: Profile strict fails on warnings
    Given profile is "strict"
    And a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "fail"

  Scenario: Profile advisory warns on warnings
    Given profile is "advisory"
    And a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "warn"

  Scenario: max_findings limits output
    Given max_findings is 1
    And a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    When lintdiff ingests the inputs
    Then findings count is 1
    And explain has 1 entries with disposition "cut_by_budget"

  Scenario: max_annotations limits annotation output
    Given max_annotations is 1
    And a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    When lintdiff ingests the inputs
    Then annotation count is 1

  @skip
  Scenario: workspace_only filters non-workspace paths
    Given workspace_only is true
    And a diff fixture "multi_file.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "skip"

  @skip
  Scenario: allow_codes permits specific codes
    Given allow code "unused_variable"
    And a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "skip"

  Scenario Outline: Profile modes affect verdict
    Given profile is <profile>
    And a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is <expected_status>

    Examples:
      | profile   | expected_status |
      | default   | warn            |
      | strict    | fail            |
      | advisory  | warn            |

  # =============================================================================
  # Integration Scenarios
  # =============================================================================

  @skip
  Scenario: Exit code 0 for skip verdict
    Given a diff fixture "simple_addition.diff"
    And empty diagnostics
    When lintdiff runs full pipeline
    Then exit code is 0

  @skip
  Scenario: Exit code 1 for fail verdict
    Given fail-on is "error"
    And a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff runs full pipeline
    Then exit code is 1

  @skip
  Scenario: Exit code 2 for error
    Given raw diff "invalid"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff runs full pipeline
    Then exit code is 2

  Scenario: Report JSON is valid
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then report JSON is valid

  Scenario: Report contains required fields
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then report has field "tool.version"
    And report has field "tool"
    And report has field "run"
    And report has field "findings"

  Scenario: cut_by_budget disposition is tracked
    Given max_findings is 1
    And a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    When lintdiff ingests the inputs
    Then explain has entries with disposition "cut_by_budget"

  # =============================================================================
  # Path Matching Edge Cases
  # =============================================================================

  Scenario: Windows backslash paths are normalized
    Given a diff fixture "simple_addition.diff"
    And diagnostics with windows path "src\\lib.rs"
    When lintdiff ingests the inputs
    Then verdict status is not "error"

  Scenario: Absolute paths are relativized
    Given a diff fixture "simple_addition.diff"
    And diagnostics with absolute path "/home/user/project/src/lib.rs"
    When lintdiff ingests the inputs
    Then verdict status is not "error"

  Scenario: Path with spaces is handled
    Given a diff fixture "simple_addition.diff"
    And diagnostics with path "src/my module.rs"
    When lintdiff ingests the inputs
    Then verdict status is not "error"

  Scenario: Unicode paths are handled
    Given a diff fixture "simple_addition.diff"
    And diagnostics with path "src/日本語.rs"
    When lintdiff ingests the inputs
    Then verdict status is not "error"

  Scenario: Symlink paths are resolved
    Given a diff fixture "simple_addition.diff"
    And diagnostics with symlink path "src/link.rs"
    When lintdiff ingests the inputs
    Then verdict status is not "error"

  Scenario: Case sensitivity matches platform
    Given a diff fixture "simple_addition.diff"
    And diagnostics with path "SRC/lib.rs"
    When lintdiff ingests the inputs
    Then verdict status is not "error"

  # =============================================================================
  # Finding Field Coverage
  # =============================================================================

  Scenario: Finding includes check_id field
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then findings have field "check_id"

  Scenario: Finding includes severity field
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then findings have field "severity"

  Scenario: Finding includes message field
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then findings have field "message"

  Scenario: Finding includes location field
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then findings have field "location"

  Scenario: Finding includes fingerprint field
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then findings have field "fingerprint"

  @skip
  Scenario: Finding with help field
    Given a diff fixture "simple_addition.diff"
    And diagnostics with help text "Try using foo instead"
    When lintdiff ingests the inputs
    Then findings have field "help"

  @skip
  Scenario: Finding with URL field
    Given a diff fixture "simple_addition.diff"
    And diagnostics with url "https://docs.rs/lint"
    When lintdiff ingests the inputs
    Then findings have field "url"

  # =============================================================================
  # Additional CLI Scenarios
  # =============================================================================

  Scenario: Version flag outputs version
    When lintdiff runs with flag "--version"
    Then output contains "lintdiff"

  Scenario: Help flag outputs help
    When lintdiff runs with flag "--help"
    Then output contains "USAGE"
    And output contains "OPTIONS"

  @skip
  Scenario: Quiet flag suppresses output
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff runs with flag "--quiet"
    Then stdout is empty

  @skip
  Scenario: Verbose flag shows more details
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff runs with flag "--verbose"
    Then output contains "findings"

  @skip
  Scenario: JSON output flag produces valid JSON
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff runs with flag "--output json"
    Then output is valid JSON

  Scenario: No color flag disables colors
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff runs with flag "--no-color"
    Then output contains no ANSI codes

  # =============================================================================
  # Report Structure scenarios
  # =============================================================================

  @skip
  Scenario: Report includes tool info
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then report tool name is "lintdiff"
    And report tool version matches semver

  Scenario: Report includes run timestamps
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then report has field "run.started_at"
    And report has field "run.ended_at"

  @skip
  Scenario: Report includes run duration
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then report has field "run.duration_ms"

  @skip
  Scenario: Report includes host info
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then report has field "run.host.os"
    And report has field "run.host.arch"

  @skip
  Scenario: Report includes git info when available
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And git repository is available
    When lintdiff ingests the inputs
    Then report has field "run.git.head_sha"
    And report has field "run.git.head_ref"

  Scenario: Report includes verdict counts
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then report has field "verdict.counts.warn"
    And report has field "verdict.counts.error"
    And report has field "verdict.counts.info"

  Scenario: Report version is current
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then report version is "lintdiff.report.v1"

  Scenario: Report is deterministic
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs twice
    Then reports are identical

  Scenario: Report schema validates
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then report validates against schema

  # =============================================================================
  # HIGH Priority: Explain Subcommand scenarios
  # =============================================================================

  @skip
  Scenario: explain subcommand shows lint information
    When lintdiff explains code "clippy::unwrap_used"
    Then output contains "unwrap"

  @skip
  Scenario: explain subcommand handles unknown codes
    When lintdiff explains code "unknown::lint"
    Then output contains "No local explanation"

  # =============================================================================
  # HIGH Priority: Config File Loading scenarios
  # =============================================================================

  Scenario: Config file from custom path is loaded
    Given a config file at custom path with profile "strict"
    And a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests with config path
    Then verdict status is "fail"

  @skip
  Scenario: Config file with all fields is respected
    Given a config file with deny code "unused_variables"
    And a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests with config path
    Then verdict status is "fail"

  # =============================================================================
  # HIGH Priority: Root Flag scenarios
  # =============================================================================

  Scenario: Custom root path is respected
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests with root path
    Then verdict status is "warn"

  # =============================================================================
  # HIGH Priority: Info Severity scenarios
  # =============================================================================

  @skip
  Scenario: Info severity diagnostics are tracked
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture with info severity
    When lintdiff ingests the inputs
    Then info count is at least 1
    And verdict status is "pass"

  # =============================================================================
  # HIGH Priority: Missing Diff Source Error scenarios
  # =============================================================================

  Scenario: Missing both base/head and diff-file produces error
    When lintdiff ingests without diff source
    Then error message contains "diff"
    And exit code is 2

  Scenario: Base without head produces error
    Given base ref is "main"
    When lintdiff ingests without head ref
    Then error message contains "head"
    And exit code is 2

  Scenario: Head without base produces error
    Given head ref is "feature"
    When lintdiff ingests without base ref
    Then error message contains "base"
    And exit code is 2

  # =============================================================================
  # HIGH Priority: Invalid Feature Flag scenarios
  # =============================================================================

  Scenario: Malformed feature flag produces error
    Given a diff fixture "simple_addition.diff"
    And feature flag "invalid_format"
    When lintdiff ingests the inputs
    Then error message contains "feature flag"
    And exit code is 2

  # =============================================================================
  # HIGH Priority: Output Path scenarios
  # =============================================================================

  Scenario: Custom output path for report
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests with output path "custom/report.json"
    Then report exists at "custom/report.json"

  # =============================================================================
  # HIGH Priority: Provenance Config scenarios
  # =============================================================================

  @skip
  Scenario: Provenance records rustc diagnostics
    Given provenance config with record_rustc true
    And a diff fixture "simple_addition.diff"
    And rustc diagnostics fixture
    When lintdiff ingests the inputs
    Then report contains rustc provenance

  @skip
  Scenario: Provenance records clippy diagnostics
    Given provenance config with record_clippy true
    And a diff fixture "simple_addition.diff"
    And clippy diagnostics fixture
    When lintdiff ingests the inputs
    Then report contains clippy provenance

  # =============================================================================
  # HIGH Priority: Missing Report Error scenarios
  # =============================================================================

  Scenario: md subcommand with missing report shows error
    When lintdiff renders markdown from missing report
    Then error message contains "not found"
    And exit code is 1

  Scenario: annotations subcommand with missing report shows error
    When lintdiff renders annotations from missing report
    Then error message contains "not found"
    And exit code is 1

  # =============================================================================
  # HIGH Priority: Corrupted Input scenarios
  # =============================================================================

  @skip
  Scenario: Corrupted JSONL produces error
    Given a diff fixture "simple_addition.diff"
    And corrupted diagnostics JSONL
    When lintdiff ingests the inputs
    Then error message contains "parse"
    And exit code is 2

  # =============================================================================
  # MEDIUM Priority: CI GitHub Overrides scenarios
  # =============================================================================

  Scenario: ci github with fail_on override
    Given environment variable "GITHUB_BASE_REF" is "main"
    And environment variable "GITHUB_SHA" is "abc123"
    And fail_on override is "never"
    And diagnostics with warnings
    When lintdiff runs ci github
    Then exit code is 0

  Scenario: ci github with diagnostics file
    Given environment variable "GITHUB_BASE_REF" is "main"
    And environment variable "GITHUB_SHA" is "abc123"
    And diagnostics file path is "diagnostics.jsonl"
    When lintdiff runs ci github
    Then exit code is 0

  Scenario: ci github with feature flags
    Given environment variable "GITHUB_BASE_REF" is "main"
    And environment variable "GITHUB_SHA" is "abc123"
    And feature flag "primary_span_matching" is "false"
    When lintdiff runs ci github
    Then exit code is 0

  # =============================================================================
  # MEDIUM Priority: Annotations Options scenarios
  # =============================================================================

  Scenario: annotations none produces no output
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests with annotations "none"
    Then annotation output is empty

  Scenario: annotations with custom max
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    When lintdiff ingests with annotations max 1
    Then annotation count is 1

  # =============================================================================
  # MEDIUM Priority: Markdown Options scenarios
  # =============================================================================

  Scenario: md with custom max_items
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    When lintdiff generates markdown with max_items 1
    Then markdown output has 1 finding

  # =============================================================================
  # MEDIUM Priority: Filter Config scenarios
  # =============================================================================

  Scenario: allow_codes from config permits codes
    Given config with allow_codes "unused_variable,dead_code"
    And a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "pass"

  # =============================================================================
  # MEDIUM Priority: Edge Cases scenarios
  # =============================================================================

  Scenario: Binary files in diff are handled gracefully
    Given a diff with binary file changes
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is not "error"

  @skip
  Scenario: Multi-line diagnostic messages are handled
    Given a diff fixture "simple_addition.diff"
    And diagnostics with multi-line message
    When lintdiff ingests the inputs
    Then findings have field "message"

  @skip
  Scenario: Special characters in messages are handled
    Given a diff fixture "simple_addition.diff"
    And diagnostics with unicode message
    When lintdiff ingests the inputs
    Then findings have field "message"

  Scenario: Deleted files only diff is handled
    Given a diff with only deletions
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is "pass"

  # =============================================================================
  # MEDIUM Priority: Error Paths scenarios
  # =============================================================================

  Scenario: Invalid report JSON for md produces error
    Given a corrupted report file
    When lintdiff renders markdown from report
    Then error message contains "parse"
    And exit code is 1

  Scenario: Invalid report JSON for annotations produces error
    Given a corrupted report file
    When lintdiff renders annotations from report
    Then error message contains "parse"
    And exit code is 1

  @skip
  Scenario: Git not available produces error
    Given git command is not available
    And base ref is "main"
    And head ref is "feature"
    When lintdiff ingests with git refs
    Then error message contains "git"
    And exit code is 2

  Scenario: Not a git repository produces error
    Given not in a git repository
    And base ref is "main"
    And head ref is "feature"
    When lintdiff ingests with git refs
    Then error message contains "repository"
    And exit code is 2

  # =============================================================================
  # MEDIUM Priority: Output Format Details scenarios
  # =============================================================================

  @skip
  Scenario: JSON output is valid and complete
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff runs with flag "--output json"
    Then output is valid JSON
    And JSON has field "version"
    And JSON has field "findings"

  @skip
  Scenario: Quiet mode suppresses all output
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff runs with flag "--quiet"
    Then stdout is empty
    And stderr is empty

  @skip
  Scenario: Verbose mode shows detailed output
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff runs with flag "--verbose"
    Then output contains "findings"
    And output contains "verdict"

  Scenario: No-color mode disables ANSI codes
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff runs with flag "--no-color"
    Then output contains no ANSI codes

  @skip
  Scenario: GitHub annotations with columns
    Given a diff fixture "simple_addition.diff"
    And diagnostics with column info
    When lintdiff generates annotations
    Then annotation output contains "col"

  # =============================================================================
  # MEDIUM Priority: Report Fields scenarios
  # =============================================================================

  @skip
  Scenario: Report contains host info
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then report has field "run.host.os"
    And report has field "run.host.arch"

  @skip
  Scenario: Report contains git repo field
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And git repository is available
    When lintdiff ingests the inputs
    Then report has field "run.git.repo"

  @skip
  Scenario: Report contains git merge_base field
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And git repository is available
    When lintdiff ingests the inputs
    Then report has field "run.git.merge_base"

  @skip
  Scenario: Finding with data field
    Given a diff fixture "simple_addition.diff"
    And diagnostics with custom data
    When lintdiff ingests the inputs
    Then findings have field "data"

  Scenario: Report with custom data
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And report data is configured
    When lintdiff ingests the inputs
    Then report has field "data"

  @skip
  Scenario: Verdict with reasons
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then report has field "verdict.reasons"

  # =============================================================================
  # MEDIUM Priority: Markdown Output Details scenarios
  # =============================================================================

  @skip
  Scenario: Markdown with all finding fields
    Given a diff fixture "simple_addition.diff"
    And diagnostics with all fields populated
    When lintdiff generates markdown
    Then markdown output contains "help"
    And markdown output contains "url"

  # =============================================================================
  # LOW Priority: Large File Handling scenarios
  # =============================================================================

  Scenario: Large diagnostics file is handled
    Given a diff fixture "simple_addition.diff"
    And a large diagnostics fixture with 10000 entries
    When lintdiff ingests the inputs
    Then verdict status is not "error"

  Scenario: Large diff file is handled
    Given a large diff fixture with 1000 files
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is not "error"

  # =============================================================================
  # LOW Priority: Special Diff Cases scenarios
  # =============================================================================

  Scenario: Merge conflict markers in diff are handled
    Given a diff with merge conflict markers
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then verdict status is not "error"

  # =============================================================================
  # LOW Priority: Error Recovery scenarios
  # =============================================================================

  Scenario: Command spawn failure produces error
    Given a diff fixture "simple_addition.diff"
    And command "nonexistent_command_12345" will be run
    When lintdiff runs the command
    Then error message contains "command"
    And exit code is 2

  Scenario: Config parse error produces error
    Given a malformed config file
    And a diff fixture "simple_addition.diff"
    When lintdiff ingests with config path
    Then error message contains "config"
    And exit code is 2

  # =============================================================================
  # LOW Priority: Permission Errors scenarios
  # =============================================================================

  Scenario: Permission denied on diff file produces error
    Given a diff file with no read permission
    When lintdiff ingests the inputs
    Then error message contains "permission"
    And exit code is 2

  Scenario: Permission denied on output file produces error
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And output path has no write permission
    When lintdiff ingests with output path
    Then error message contains "permission"
    And exit code is 2

  # =============================================================================
  # LOW Priority: Network Errors scenarios
  # =============================================================================

  @skip
  @skip
  Scenario: Git network failure produces error
    Given git network is unavailable
    And base ref is "main"
    And head ref is "feature"
    When lintdiff ingests with git refs
    Then error message contains "network"
    And exit code is 2

  # =============================================================================
  # HIGH Priority: Exit Code scenarios (NEW - addressing coverage gap)
  # =============================================================================

  Scenario: Exit code 0 for pass verdict with clean diff
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_outside_diff.jsonl"
    When lintdiff runs full pipeline
    Then exit code is 0

  Scenario: Exit code 0 for warn verdict with default fail_on
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff runs full pipeline
    Then exit code is 0

  Scenario: Exit code 0 for skip verdict with missing diagnostics
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

  Scenario: Exit code 2 for fail_on warn with warnings present
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And fail_on is "warn"
    When lintdiff runs full pipeline
    Then exit code is 2

  Scenario: Exit code 2 for tool error with invalid diff
    Given raw diff "this is not a valid diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    When lintdiff ingests the inputs
    Then error message contains "diff"
    And exit code is 2

  # =============================================================================
  # HIGH Priority: Error Condition scenarios (NEW - addressing coverage gap)
  # =============================================================================

  Scenario: Invalid diagnostics JSON produces parse error
    Given a diff fixture "simple_addition.diff"
    And raw diagnostics "this is not json"
    When lintdiff ingests the inputs
    Then error message contains "parse"
    And exit code is 2

  Scenario: Empty diagnostics produces pass verdict
    Given a diff fixture "simple_addition.diff"
    And empty diagnostics
    When lintdiff ingests the inputs
    Then verdict status is "pass"

  Scenario: Corrupted JSONL produces parse error
    Given a diff fixture "simple_addition.diff"
    And raw diagnostics "not valid json at all"
    When lintdiff ingests the inputs
    Then error message contains "parse"
    And exit code is 2

  # =============================================================================
  # HIGH Priority: Configuration scenarios (NEW - addressing coverage gap)
  # =============================================================================

  Scenario: workspace_only filters non-workspace paths
    Given workspace_only is true
    And a diff fixture "multi_file.diff"
    And diagnostics with absolute path "/usr/local/lib/rustlib/src/rust/src/lib.rs"
    When lintdiff ingests the inputs
    Then verdict status is "pass"

  Scenario: Suppress code removes matching findings
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "warning_on_changed_line.jsonl"
    And suppress code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    Then verdict status is "pass"
    And warn count is 0

  Scenario: Multiple deny codes work together
    Given a diff fixture "simple_addition.diff"
    And a diagnostics fixture "fingerprint_whitespace_equivalent.jsonl"
    And deny code "lintdiff.diagnostic.clippy.let_unit_value"
    When lintdiff ingests the inputs
    Then verdict status is "fail"
    And error count is 2
