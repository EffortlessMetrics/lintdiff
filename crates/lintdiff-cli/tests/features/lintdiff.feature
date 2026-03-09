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
