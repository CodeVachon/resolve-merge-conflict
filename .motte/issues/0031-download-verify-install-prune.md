---
id: 31
title: Download, verify, install, prune
state: Done
parent: 6
assignee: Christopher Vachon
labels: [self-update]
blockedBy: [29, 30]
created: 2026-10-21T13:41:16Z
updated: 2026-10-21T14:17:58Z
---

## Plan

Port download.rs: sha256 checksum from checksums.txt before anything is written, gunzip, chmod 755, repoint current.

## Notes

### 2026-10-21T14:17:57Z — Christopher Vachon (user)

download.rs ported: checksums.txt sha256 verified before anything is written, gunzip, chmod 755, repoint current, prune keeping the running version.
