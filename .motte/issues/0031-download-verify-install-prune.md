---
id: 31
title: Download, verify, install, prune
state: Todo
parent: 6
labels: [self-update]
blockedBy: [29, 30]
created: 2026-10-21T13:41:16Z
updated: 2026-10-21T13:41:34Z
---

## Plan

Port download.rs: sha256 checksum from checksums.txt before anything is written, gunzip, chmod 755, repoint current.
