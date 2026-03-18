# Report output messages for lintdiff (en-US)
# This file contains messages used in report generation.

## Report Header
report-header = Lintdiff Analysis Report
report-generated = Generated at { $timestamp }
report-commit = Commit: { $sha }

## Summary Section
report-summary = Summary
report-summary-files = Files analyzed
report-summary-additions = Lines added
report-summary-deletions = Lines deleted
report-summary-findings = Findings

## Findings
report-findings-title = Findings
report-findings-empty = No findings to report.

report-finding-item = { $severity }: { $code } in { $file }:{ $line }
    .description = { $message }

## Severity Labels
severity-error = Error
severity-warning = Warning
severity-note = Note
severity-help = Help

## Verdict
report-verdict = Verdict: { $verdict ->
    [pass] PASS
    [warn] WARN
   *[fail] FAIL
}
