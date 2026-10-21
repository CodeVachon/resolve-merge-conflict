---
id: 12
title: Git wrapper over std::process::Command
state: Done
parent: 2
assignee: Christopher Vachon
labels: [core]
blockedBy: [9]
created: 2026-10-21T13:41:13Z
updated: 2026-10-21T13:45:59Z
---

## Plan

Port merge-pipeline src/git.rs. Needs: status porcelain v2, unmerged paths, ls-files -u, rev-parse --git-dir, git add, diff --stat, show :1/:2/:3 stage blobs.

## Notes

### 2026-10-21T13:45:59Z — Christopher Vachon (user)

src/git.rs: Git over a CommandRunner (fakeable), call/try_call, is_work_tree, top_level, git_dir, head, current_branch, unmerged_entries (status --porcelain=v2 -z; u-lines give ConflictKind: UU/AA/DD/AU/UA/DU/UD; unknown code is a Parse error rather than skipped), other_changes (non-conflict edits, for the runner's warning), stage_blob(Base|Ours|Theirs, path) -> Option (None when that side deleted the file), add (git add -A -- paths, so resolved deletions stage), numstat. Deliberately NO commit/push/continue/abort methods: the type cannot do what the tool promises never to do. Dropped merge-pipeline's verbose log and branch helpers (unused here). Added thiserror. 5 unit + 8 integration tests; fmt and clippy -D warnings clean. Not committed.
