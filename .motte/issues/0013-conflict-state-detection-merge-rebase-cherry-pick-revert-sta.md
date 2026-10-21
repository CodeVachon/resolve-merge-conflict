---
id: 13
title: Conflict state detection (merge, rebase, cherry-pick, revert, stash pop)
state: Todo
parent: 2
labels: [core]
blockedBy: [11, 12]
created: 2026-10-21T13:41:13Z
updated: 2026-10-21T13:41:29Z
---

## Description

Detect which operation is in progress (MERGE_HEAD, rebase-merge/rebase-apply, CHERRY_PICK_HEAD, REVERT_HEAD, or unmerged paths with no marker file for stash pop) and list unmerged files with their conflict type (both modified, deleted by us/them, both added). No conflicts => friendly message, exit 0. Never starts a merge.

## Plan

Pure function over a Git handle returning ConflictState { operation, files[] }. Fixture tests per operation.
