# CI and dependency queue continuity

## Objective state (authoritative handoff)
- Baseline source of truth: `origin/main` (latest known merged change is PR #40)
- Legacy high-risk PR lineage:
  - #28 baseline advisories + clippy compatibility (merged)
  - #27 mutation threshold numeric fix (merged)
  - #26 dependency/action updates (merged)
  - #25 dependency updates (closed after split policy applied)
- PR #19 (Factory Droid workflow) is closed as obsolete and should not be reopened without explicit redesign.

## Current queued work (ready order)
1. None currently queued; #32-#36 have been merged in order.
2. Rehydrate only from active dependabot PRs when new queue entries appear.

## Completed queue snapshot
- `origin/main` now includes PR #38, which records queue state after dependency queue completion.
- `#32 -> #33 -> #34 -> #35 -> #36` are merged to `origin/main`.
- Latest dependency queue head SHA: `e647b4e` (`#36`)
- Latest queue-record snapshot SHA: `ba9da87` (`#40`)
- Last dependency queue merge timestamp: `2026-07-31T05:27:58Z` (`#36`).

## Resume playbook (for next maintainer/Codex turn)
1. Keep `main` synced: `git fetch origin && git checkout main && git reset --hard origin/main`.
2. Process any new queue strictly in arrival order to preserve deterministic conflict behavior.
3. Prefer merge/rebase-based progression; avoid duplicating fixes already in `#31`.
4. Do not copy lockfile/clippy behavior changes into other PRs that do not own that scope.
5. Treat PR closure as final ledger state unless replaced by an explicit new queue entry.

## Verification notes
- Source files changed in this model lane are queue metadata files (this document), unless a future lane opens.
- If PR order changes, update this file immediately after merging the queue head.
