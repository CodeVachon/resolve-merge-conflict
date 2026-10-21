---
id: 35
title: Build matrix producing gzipped assets per target
state: Done
parent: 7
assignee: Christopher Vachon
labels: [release]
blockedBy: [9]
created: 2026-10-21T13:41:17Z
updated: 2026-10-21T14:18:26Z
---

## Plan

Copy release.yml build job from merge-pipeline.

## Notes

### 2026-10-21T14:18:26Z — Christopher Vachon (user)

release.yml build matrix ported: native runner per target (macos-14 for both darwin, ubuntu-latest + ubuntu-24.04-arm with musl for static linux, windows-latest), cargo build --release --locked, asset resolve-merge-conflicts-<os>-<arch>[.exe].gz via gzip -9 -n (reproducible), uploaded as artifacts. YAML parses; actual runs happen on the first tag.
