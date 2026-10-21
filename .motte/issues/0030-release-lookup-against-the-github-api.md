---
id: 30
title: Release lookup against the GitHub API
state: Done
parent: 6
assignee: Christopher Vachon
labels: [self-update]
blockedBy: [29]
created: 2026-10-21T13:41:16Z
updated: 2026-10-21T14:17:58Z
---

## Plan

Port releases.rs; repo CodeVachon/resolve-merge-conflict (confirm final repo name).

## Notes

### 2026-10-21T14:17:57Z — Christopher Vachon (user)

releases.rs ported: /releases/latest with fallback to newest non-draft, rate-limit detection, semver ordering, RESOLVE_MERGE_CONFLICTS_API_BASE override for tests.
