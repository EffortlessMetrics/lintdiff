# CI and dependency queue continuity

## Objective state (authoritative handoff)
- Baseline source of truth: `origin/main` (latest known good merge is PR #31)
- Legacy high-risk PR lineage:
  - #28 baseline advisories + clippy compatibility (merged)
  - #27 mutation threshold numeric fix (merged)
  - #26 dependency/action updates (merged)
  - #25 dependency updates (closed after split policy applied)
- PR #19 (Factory Droid workflow) is closed as obsolete and should not be reopened without explicit redesign.

## Current queued work (ready order)
1. `dependabot/cargo/patch-dependencies-5cbec24b7a` (PR #32)
   - Group: patch updates only
2. `dependabot/cargo/uselesskey-0.10.0` (PR #33)
   - High-risk boundary: separate `0.x` minor upgrade
3. `dependabot/cargo/jsonschema-0.46.2` (PR #34)
   - High-risk boundary: separate `0.x` minor upgrade
4. `dependabot/cargo/cucumber-0.23.0` (PR #35)
   - High-risk boundary: separate `0.x` minor upgrade
5. `dependabot/cargo/regex-1.13.1` (PR #36)
   - Routine patch dependency update

## Resume playbook (for next maintainer/Codex turn)
1. Keep `main` synced: `git fetch origin && git checkout main && git reset --hard origin/main`.
2. Process the queue strictly in the order above to preserve deterministic conflict behavior.
3. Prefer merge/rebase-based progression; avoid duplicating fixes already in `#31`.
4. Do not copy lockfile/clippy behavior changes into other PRs that do not own that scope.
5. Treat PR closure as final ledger state unless replaced by an explicit new queue entry.

## Verification notes
- Source files changed in this model lane are only queue metadata files (this document).
- If PR order changes, update this file immediately after merging the head of the queue.