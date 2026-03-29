# Error messages for lintdiff (en-US)
# This file contains all error-related messages.

## Configuration Errors
error-config-invalid = Invalid configuration file.
error-config-not-found = Configuration file not found: { $path }
error-config-parse-error = Failed to parse configuration: { $error }

## Diff Errors
error-diff-parse = Failed to parse diff output.
error-diff-empty = Diff is empty. No changes to analyze.
error-diff-binary = Cannot analyze binary file.

## Diagnostic Errors
error-diagnostic-parse = Failed to parse diagnostic at line { $line }.
error-diagnostic-invalid-json = Invalid JSON in diagnostic output.
error-diagnostic-no-span = Diagnostic has no source location.

## Matching Errors
error-match-failed = Failed to match diagnostic to diff location.

## Policy Errors
error-policy-invalid = Invalid policy configuration.
error-policy-unknown-action = Unknown policy action: { $action }

## I/O Errors
error-io-permission = Permission denied: { $path }
error-io-disk = Disk error: { $error }

## Git Errors
error-git-not-repo = Not a git repository.
error-git-no-head = Could not resolve HEAD commit.
error-git-diff-failed = Git diff command failed: { $error }
