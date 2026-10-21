---
id: 17
title: Post-run verifier and stager
state: Done
parent: 2
assignee: Christopher Vachon
labels: [core]
blockedBy: [11, 12]
created: 2026-10-21T13:41:13Z
updated: 2026-10-21T13:57:04Z
---

## Description

After the agent exits: for each originally conflicted file confirm no <<<<<<< / ======= / >>>>>>> markers remain and the file still exists (unless a deletion resolution), then git add the resolved files. Files that still have markers are left unstaged and reported. Hard guarantee: no commit, no push, no merge --continue / rebase --continue.

## Plan

Marker scan must ignore legitimate markers in files where they are content (e.g. docs) by checking against the original unmerged set and diff, not a repo-wide grep. Add an integration test that a fake agent script which resolves, leaves markers, or deletes a file yields the right staged state.

## Notes

### 2026-10-21T13:57:04Z — Christopher Vachon (user)

src/verify.rs. Snapshot::take before the agent (HEAD + content hash of every non-conflicted changed/untracked file); verify_and_stage after: per file Resolved | MarkersRemain | Missing, strategy Ours|Theirs|Merged|Deleted (decided by comparing against index stages BEFORE git add clears them), then git add of resolved files only (stage flag allows off), numstat for +/-. Also reports head_moved (agent committed) and touched_outside (agent edited/created files outside the conflicted set; pre-existing edits are not blamed). Marker rule needs <<<<<<< then ======= then >>>>>>> in order, so a lone '=======' heading underline is not a conflict; diff3 ||||||| blocks are covered. Missing file is Resolved only for deletion-type conflicts (DU/UD/DD). Known limit: a document that legitimately contains a complete marker block is reported as unresolved rather than staged (safe direction). git.other_changes now includes untracked files. NOT the runner's job to abort/continue anything.
