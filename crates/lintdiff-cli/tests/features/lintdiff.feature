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
