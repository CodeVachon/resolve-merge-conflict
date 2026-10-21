---
id: 7
title: "Epic: Release pipeline (GitHub Actions)"
state: Done
labels: [release]
created: 2026-10-21T13:40:41Z
updated: 2026-10-21T14:46:35Z
---

## Description

Tag-driven release workflow copied from merge-pipeline: gate (tag == Cargo version, fmt, clippy, tests), build matrix for darwin-arm64, darwin-x64, linux-x64, linux-arm64, windows-x64 producing gzipped assets, verify job (checksums.txt + installer smoke tests against the built assets), publish job. Plus .github/release.yml and the PR template.
