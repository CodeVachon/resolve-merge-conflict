---
id: 10
title: "CI workflow: fmt, clippy, test on ubuntu and macos"
state: Done
parent: 1
assignee: Christopher Vachon
labels: [scaffold]
blockedBy: [9]
created: 2026-10-21T13:41:12Z
updated: 2026-10-21T14:18:07Z
---

## Plan

Copy .github/workflows/ci.yml from merge-pipeline; adapt names. Add pull_request_template.md.

## Notes

### 2026-10-21T14:18:07Z — Christopher Vachon (user)

Workflow copied from merge-pipeline (fmt, clippy -D warnings, cargo test --locked on ubuntu-latest and macos-latest), plus PR template and release-note categories (.github/release.yml). Verified: first push ran green on both OSes. book and schema jobs get added by #41 and #40.
