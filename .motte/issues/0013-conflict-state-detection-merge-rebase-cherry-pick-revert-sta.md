---
id: 13
title: Conflict state detection (merge, rebase, cherry-pick, revert, stash pop)
state: Done
parent: 2
assignee: Christopher Vachon
labels: [core]
blockedBy: [11, 12]
created: 2026-10-21T13:41:13Z
updated: 2026-10-21T13:55:52Z
---

## Description

Detect which operation is in progress (MERGE_HEAD, rebase-merge/rebase-apply, CHERRY_PICK_HEAD, REVERT_HEAD, or unmerged paths with no marker file for stash pop) and list unmerged files with their conflict type (both modified, deleted by us/them, both added). No conflicts => friendly message, exit 0. Never starts a merge.

## Plan

Pure function over a Git handle returning ConflictState { operation, files[] }. Fixture tests per operation.

## Notes

### 2026-10-21T13:55:52Z — Christopher Vachon (user)

src/conflicts.rs: detect(&mut Git) -> Option<ConflictState{operation, files, ours, theirs}>. Operation from marker files in the git dir (MERGE_HEAD, rebase-merge/rebase-apply, CHERRY_PICK_HEAD, REVERT_HEAD); unmerged paths with none of those are Operation::Other (stash pop and friends leave no marker). ours/theirs are plain-language, and deliberately swapped for rebase (ours = branch rebased onto, theirs = your replayed commit) so the prompt does not mislead the agent. All-staged merge returns None (nothing left for an agent; finishing is the user's step). Non-repo is an error; clean repo is None.
