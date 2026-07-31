# CI and dependency queue continuity

## Objective state (authoritative handoff)
- Baseline source of truth: `origin/main` (latest merged continuity update is PR #65)
- Legacy high-risk PR lineage:
  - #28 baseline advisories + clippy compatibility (merged)
  - #27 mutation threshold numeric fix (merged)
  - #26 dependency/action updates (merged)
  - #25 dependency updates (closed after split policy applied)
- PR #19 (Factory Droid workflow) is closed as obsolete and should not be reopened without explicit redesign.
- Durable queue model:
  - BASELINE: #28
  - READY AFTER BASELINE: #27
  - GENERATED RESTACK: #26
  - REVIEW/REDUCE: #17
  - REPLACE/SPLIT: #25
  - CLOSE: #19

## Latest continuity verification snapshot
- Verified at: `2026-07-31T15:50:07.2555547-04:00`
- git status --short --branch: `## main...origin/main`
- gh pr list --state open --limit 100: no results (0 open PRs)
- gh issue list --state open --limit 100: no results (0 open issues)
- Dependency order: none
- Local branches: main
- Queue branch hygiene candidates to prune: none
- git log -n 1 --oneline on HEAD: `* 8914ade (HEAD -> main, origin/main, origin/HEAD) docs(queue): refresh continuity handoff snapshot (#65)`
- Dependency evidence file: `artifacts/ci-queue-dependency-order.jsonl`

## Repeatable continuity rehydrate command
```powershell
git fetch --prune origin
git status --short --branch
gh pr list --state open --limit 100
gh issue list --state open --limit 100
git branch --sort=-committerdate
git log -n 5 --oneline --decorate --graph --all --max-count=5
git show -s --oneline origin/main
```

To run the same check with JSON output for quick handoff capture, use:

```powershell
./plans/check-ci-queue-continuity.ps1
```

To run and apply the computed dependency restack (only on clean `main` and only when dependency ordering is valid), use:

```powershell
./plans/check-ci-queue-continuity.ps1 -ApplyRestack -ConfirmRestack
```

Use interactive confirmation mode for manual runs:

```powershell
./plans/check-ci-queue-continuity.ps1 -ApplyRestack
```

(Type `APPLY` when prompted to continue.)

To dry-run execution before writing anything back to remote branches:

```powershell
./plans/check-ci-queue-continuity.ps1 -ApplyRestack -DryRunRestack
```

`-DryRunRestack` is non-interactive and reports the planned `dependency_restack_plan` without requiring `APPLY` confirmation.

To refresh this file's verification timestamp automatically during a handoff check:

```powershell
./plans/check-ci-queue-continuity.ps1 -UpdatePlan
```

Each run appends machine-readable evidence to:

`artifacts/ci-queue-continuity-evidence.jsonl`
and `artifacts/ci-queue-dependency-order.jsonl`.

## Current queued work (ready order)
1. No open PR work currently queued in this lane.

## Completed queue snapshot
- origin/main head check: 8914ade docs(queue): refresh continuity handoff snapshot (#65)
- Last continuity verification: 2026-07-31T15:50:07.2555547-04:00
- Snapshot queue order: none

## Resume playbook (for next maintainer/Codex turn)
1. Keep `main` synced: `git fetch origin && git checkout main && git reset --hard origin/main`.
2. Process any new queue strictly in arrival order to preserve deterministic conflict behavior.
3. Before processing, prune stale branch references and confirm a clean baseline:
   - `git remote prune origin`
   - `git branch --merged | ForEach-Object { $_ }`
   - `git branch --no-merged`
4. Prefer merge/rebase-based progression; avoid duplicating fixes already in merged lineage.
5. Use dependency evidence to order work:
   - Run `./plans/check-ci-queue-continuity.ps1` and read `dependency_order` first.
   - If `dependency_warnings` is non-empty, stop and resolve dependency or cycle issues before proceeding.
   - If `dependency_restack_applied` is `false` and `blocked_by_dependency_warning` is printed, do not proceed to restacking; treat warnings as hard blockers.
   - Work PRs in the published `#<number>` ready order (or use `dependency_restack_plan` output when present), then rerun the check.
6. Apply queue restack only when the plan is clean:
   - `./plans/check-ci-queue-continuity.ps1 -ApplyRestack` (interactive confirmation), or
   - `./plans/check-ci-queue-continuity.ps1 -ApplyRestack -ConfirmRestack` (non-interactive automation).
   - `./plans/check-ci-queue-continuity.ps1 -ApplyRestack -DryRunRestack` (non-interactive plan-only mode, no local or remote writes).
7. After merging queued PRs, prune queue-local branches that are merged:
   - `git fetch origin --prune`
   - `git branch -r --merged origin/main`
   - `git branch --merged`
   - Prefer deleting only branches listed in the latest `stale_merged_local_branches` output from `plans/check-ci-queue-continuity.ps1` (or `DependencySnapshot` evidence field `StalePrunableBranches`).
   - If `MergedLocalBranches` is empty in evidence, it simply means no local branches are currently merged relative to `origin/main` and not listed for deletion; this is not an error.
   - Do not remove remote branches from this lane alone; remote branch cleanup needs queue owner coordination.
   - Remote branch removals should be explicit and one-off, only after merge closure proof and queue-owner approval.
8. Do not copy lockfile/clippy behavior changes into other PRs that do not own that scope.
9. Treat PR closure as final ledger state unless replaced by an explicit new queue entry.
10. Treat `blocked_by_dependency_warning` as the canonical hard-stop reason for dependency-graph issues in automation runs.

### Canonical `ApplyRestack` signal matrix

- `open PRs empty` (`dependency_order=[]`), dry-run: emits `no_open_prs_or_missing_order` and `dependency_restack_applied=false`.
- `dependency warnings non-empty`, dry-run: emits `blocked_by_dependency_warning` and `dependency_restack_applied=false`.
- `warnings empty` and `dependency_restack_plan` present:
  - non-dry-run, confirmed: emits `completed` and `dependency_restack_applied=true`.
  - dry-run: emits `dry_run_complete` and `dependency_restack_applied=false`.
- when not using `-ApplyRestack`, emits `dependency_restack_applied=false` and `dependency_restack_applied_reason=not_requested` in check-only mode.

### Canonical sample outcomes

Empty queue (`main` clean, no open PRs):

```text
dependency_order=
dependency_restack_plan=
dependency_restack_applied=false
no_open_prs_or_missing_order
dependency_restack_applied_reason=no_open_prs_or_missing_order
```

Dependency warning case (example mock cycle `#201 -> #202 -> #201`):

```text
dependency_order=
dependency_warnings=
dependency_cycle_or_unknown: no zero-inbound open PRs remain
dependency_restack_plan=[]
dependency_restack_applied=false
blocked_by_dependency_warning
dependency_restack_applied_reason=blocked_by_dependency_warning
```

## Verification notes
- Source files changed in this lane include continuity queue metadata and script contract files [plans/ci-queue-continuity.ps1], unless a future lane opens.
- If PR order changes, update this file immediately after merging the queue head.