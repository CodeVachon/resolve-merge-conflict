---
id: 37
title: "Verify job: checksums.txt and installer smoke tests on built assets"
state: Done
parent: 7
assignee: Christopher Vachon
labels: [release]
blockedBy: [33, 35]
created: 2026-10-21T13:41:17Z
updated: 2026-10-21T14:18:27Z
---

## Plan

scripts/smoke.sh from merge-pipeline, using the DOWNLOAD_BASE env override.

## Notes

### 2026-10-21T14:18:26Z — Christopher Vachon (user)

checksums job assembles every asset + checksums.txt once; verify (linux, install.sh + scripts/smoke.sh against a local server faking the releases API) and verify-windows (install.ps1, --version, --help) consume that same bundle. Locally verified the same chain on macOS: install.sh from gzipped release binary + checksums.txt, then smoke.sh (see #33).
