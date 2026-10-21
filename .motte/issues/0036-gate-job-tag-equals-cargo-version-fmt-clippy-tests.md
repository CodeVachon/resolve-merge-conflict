---
id: 36
title: "Gate job: tag equals Cargo version, fmt, clippy, tests"
state: Done
parent: 7
assignee: Christopher Vachon
labels: [release]
blockedBy: [10]
created: 2026-10-21T13:41:17Z
updated: 2026-10-21T14:18:27Z
---

## Notes

### 2026-10-21T14:18:26Z — Christopher Vachon (user)

gate job: tag must equal v<Cargo version>, then fmt, clippy -D warnings, cargo test --locked on ubuntu and macos. Runs on tag push or workflow_dispatch with an existing tag.
