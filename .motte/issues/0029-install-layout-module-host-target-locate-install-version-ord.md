---
id: 29
title: Install layout module (host, target, locate install, version ordering)
state: Done
parent: 6
assignee: Christopher Vachon
labels: [self-update]
blockedBy: [9]
created: 2026-10-21T13:41:16Z
updated: 2026-10-21T14:17:57Z
---

## Plan

Port src/selfupdate/layout.rs. Add the test that parses install.sh and asserts asset names + directory shape agree.

## Notes

### 2026-10-21T14:17:56Z — Christopher Vachon (user)

Ported src/selfupdate/layout.rs from merge-pipeline with names swapped (REPO CodeVachon/resolve-merge-conflict, root ~/.resolve-merge-conflicts, env RESOLVE_MERGE_CONFLICTS_*). Unit test parses install.sh and asserts asset names and directory shape agree.
