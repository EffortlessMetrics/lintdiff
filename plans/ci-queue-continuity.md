# CI and dependency queue continuity

## Purpose

This document defines continuity **policy**. It is not a per-run handoff snapshot.

## Objective

- Baseline source of truth: `origin/main`
- All continuity checks are read-only.
- The checker emits JSONL receipts only; tracked files are not rewritten with runtime state.

## Dependency declaration rules

Only PR body lines matching this exact pattern become queue edges:

```text
Depends-On: #123
```

Regex used by the checker: `^Depends-On:\s+#(\d+)\s*$`

- `Closes #123`, `Fixes #123`, leading-space declarations, and prose references do not create dependencies.
- Non-matching or malformed lines are ignored.
- Duplicate declarations are deduplicated deterministically.
- `Depends-On` entries for missing or closed PRs emit an `unknown_open_pr_dependency` warning.
- Any cycle emits a `dependency_cycle_or_unknown` warning and disables restack actions.
- More than one open-PR dependency produces `manual_integration_required = true` in the plan object and does not claim a synthetic linear `rebase_onto` baseline.
- Unknown edges do not create synthetic PRs.

## Read-only check commands

```powershell
git fetch --prune origin
git status --short --branch --untracked-files=no
gh pr list --state open --limit 100 --json number,title,body,headRefName,baseRefName,author,updatedAt
gh issue list --state open --limit 100 --json number,title,updatedAt
git branch --sort=-committerdate
git branch --merged origin/main
git log -n 5 --oneline --decorate --graph --all --max-count=5
git show -s --oneline origin/main
```

Run:

```powershell
./plans/check-ci-queue-continuity.ps1
```

For local proof in CI and handoff records:

```powershell
pwsh -NoProfile -File plans/check-ci-queue-continuity.ps1
```

The checker writes:

- `artifacts/ci-queue-continuity-evidence.jsonl`
- `artifacts/ci-queue-dependency-order.jsonl`

Both files are ignored and can be used as receipts for per-run state.

## Restack policy

This lane does not perform remote mutation. The restack-applier switches were removed from the checker and the lane emits:

```text
restack_apply_disabled
restack_apply_disabled_reason=remote restacking is intentionally disabled until branch semantics are specified
```

No automatic command in this lane runs:

- `gh pr checkout`
- `git rebase`
- `git push`

Independent PRs are treated as independent by analysis; they are never stacked onto each other by the checker.

## Signal matrix

- `dependency_order=`: deterministic processing order or `[]`.
- `dependency_warnings=`:
  - `unknown_open_pr_dependency: #A references non-open or missing PR #B`
  - `dependency_cycle_or_unknown: no zero-inbound open PRs remain`
  - `unresolved_dependency: X -> Y`
- `dependency_restack_plan=`: explicit plan objects when queue is valid.
  - `rebase_onto` is `origin/main` only for PRs with no dependencies.
  - `rebase_onto` is the single dependency id for PRs with exactly one dependency.
  - PRs with multiple dependencies are marked `manual_integration_required=true`.
- `restack_apply_disabled`: always for this lane.

## Trigger model

Run the continuity check when:

- a new PR appears,
- a PR adds or changes `Depends-On`,
- a dependency warning appears,
- a baseline PR merges,
- stale local branch cleanup is needed,
- or the continuity implementation changes.

Do not poll for an empty queue as a maintenance loop.
