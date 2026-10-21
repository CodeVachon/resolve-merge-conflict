---
id: 41
title: mdBook site on GitHub Pages plus CI book job
state: Done
parent: 8
assignee: Christopher Vachon
labels: [docs]
blockedBy: [39]
created: 2026-10-21T13:41:18Z
updated: 2026-10-21T14:23:20Z
---

## Plan

Copy docs/, book.toml, pages.yml from merge-pipeline.

## Notes

### 2026-10-21T14:23:20Z — Christopher Vachon (user)

mdBook site in docs/ (book.toml, SUMMARY, introduction, install, usage, safety, configuration, report, keeping-up-to-date, command reference, development, changelog via {{#include}}), content derived from the README so the two cannot disagree. docs/src/help/*.txt are the real clap --help output; tests/docs.rs fails if they drift (UPDATE_DOCS=1 cargo test --test docs regenerates) and checks reference.md includes every one. Verified locally with mdbook 0.5.4: builds with 0 WARN, check-links.py reports 14 pages and no broken internal links. CI gets a 'book' job (pinned, checksum-verified mdBook; fails on any WARN or broken link); .github/workflows/pages.yml builds and deploys to GitHub Pages and then checks the live site serves the book. NOT DONE (repo setting, publishes a public site): enabling Pages with source 'GitHub Actions' in the repo settings. Until then the deploy job will fail on pushes that touch docs/ (the build job still validates). Known edge: help text contains '[env: X=]' so the sync test would flag a developer who has RESOLVE_MERGE_CONFLICTS_AGENT or _MODEL exported.
