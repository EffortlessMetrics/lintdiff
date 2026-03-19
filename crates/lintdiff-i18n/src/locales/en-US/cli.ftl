# CLI-specific messages for lintdiff (en-US)
# This file contains command-line interface messages.

## Command Help
cli-help = Diff-scoped filter for rustc/Clippy diagnostics

## Run Command
cli-run-starting = Starting lint analysis...
cli-run-complete = Analysis complete. Found { $count ->
    [one] one finding
   *[other] { $count } findings
}.

## Exit Codes
cli-exit-pass = All checks passed.
cli-exit-warn = Checks completed with warnings.
cli-exit-fail = Checks failed.

## Version
cli-version = { brand-name } version { $version }

## Locale Flag (reserved for future use)
cli-locale-hint = Set the output locale (e.g., en-US, de-DE)
