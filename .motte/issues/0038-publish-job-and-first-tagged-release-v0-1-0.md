---
id: 38
title: Publish job and first tagged release (v0.1.0)
state: Todo
parent: 7
labels: [release]
blockedBy: [36, 37]
created: 2026-10-21T13:41:17Z
updated: 2026-10-21T14:18:27Z
---

## Plan

Create the GitHub repo secrets/permissions as needed. Tag v0.1.0 only after the user approves.

## Notes

### 2026-10-21T14:18:27Z — Christopher Vachon (user)

publish job authored (softprops/action-gh-release, files = *.gz + checksums.txt, generated notes via .github/release.yml categories, make_latest). Needs gate, verify and verify-windows. NOT DONE: tagging v0.1.0. Pushing a v* tag publishes a public release, so it waits for the user's go-ahead. To release: update CHANGELOG (#42), confirm Cargo version 0.1.0, git tag v0.1.0 && git push origin v0.1.0, watch the Release workflow; if a verify job fails nothing is published.
