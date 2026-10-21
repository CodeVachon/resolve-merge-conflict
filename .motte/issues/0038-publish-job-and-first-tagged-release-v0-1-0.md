---
id: 38
title: Publish job and first tagged release (v0.1.0)
state: Done
parent: 7
assignee: Christopher Vachon
labels: [release]
blockedBy: [36, 37]
created: 2026-10-21T13:41:17Z
updated: 2026-10-21T14:46:35Z
---

## Plan

Create the GitHub repo secrets/permissions as needed. Tag v0.1.0 only after the user approves.

## Notes

### 2026-10-21T14:18:27Z — Christopher Vachon (user)

publish job authored (softprops/action-gh-release, files = *.gz + checksums.txt, generated notes via .github/release.yml categories, make_latest). Needs gate, verify and verify-windows. NOT DONE: tagging v0.1.0. Pushing a v* tag publishes a public release, so it waits for the user's go-ahead. To release: update CHANGELOG (#42), confirm Cargo version 0.1.0, git tag v0.1.0 && git push origin v0.1.0, watch the Release workflow; if a verify job fails nothing is published.

### 2026-10-21T14:46:35Z — Christopher Vachon (user)

RELEASED v0.1.0 on user approval (2026-10-21). Release workflow run 39619035573: gate (ubuntu, macos), build x5, checksums, verify linux install.sh + smoke, verify windows install.ps1, publish — all green on the first run. Verified from the outside afterwards: the public curl|sh installer fetched the release, checksum verified, binary reports 0.1.0, and 'upgrade --check' says it is the newest release. Docs site live at https://codevachon.github.io/resolve-merge-conflict/ (Pages enabled on approval; deploy verified it serves the book).
