---
id: 33
title: install.sh and install.ps1
state: Done
parent: 6
assignee: Christopher Vachon
labels: [self-update]
blockedBy: [29]
created: 2026-10-21T13:41:17Z
updated: 2026-10-21T14:17:58Z
---

## Plan

Copy from merge-pipeline, rename env vars. Also writes completions to ~/.resolve-merge-conflicts/completions/.

## Notes

### 2026-10-21T14:17:57Z — Christopher Vachon (user)

install.sh and install.ps1 copied with renames. VERIFIED locally: built release binary, gzipped as resolve-merge-conflicts-darwin-arm64.gz with checksums.txt, served over python http.server, ran install.sh with RESOLVE_MERGE_CONFLICTS_{VERSION,INSTALL_DIR,BIN_DIR,DOWNLOAD_BASE}; got the symlink chain, working binary, 'versions' listing and completion scripts. install.ps1 not run (no Windows here); it is exercised by the release verify job (#37). scripts/smoke.sh rewritten for this tool (init config, completion, a real resolve with a fake claude against a throwaway conflicted repo, HEAD untouched).
