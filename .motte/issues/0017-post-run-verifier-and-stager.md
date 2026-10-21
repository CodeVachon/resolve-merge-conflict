---
id: 17
title: Post-run verifier and stager
state: Todo
parent: 2
labels: [core]
blockedBy: [11, 12]
created: 2026-10-21T13:41:13Z
updated: 2026-10-21T13:41:30Z
---

## Description

After the agent exits: for each originally conflicted file confirm no <<<<<<< / ======= / >>>>>>> markers remain and the file still exists (unless a deletion resolution), then git add the resolved files. Files that still have markers are left unstaged and reported. Hard guarantee: no commit, no push, no merge --continue / rebase --continue.

## Plan

Marker scan must ignore legitimate markers in files where they are content (e.g. docs) by checking against the original unmerged set and diff, not a repo-wide grep. Add an integration test that a fake agent script which resolves, leaves markers, or deletes a file yields the right staged state.
