---
id: 32
title: upgrade, uninstall, versions, use, prune subcommands
state: Done
parent: 6
assignee: Christopher Vachon
labels: [self-update]
blockedBy: [31]
created: 2026-10-21T13:41:16Z
updated: 2026-10-21T14:17:58Z
---

## Plan

Port upgrade.rs and versions.rs.

## Notes

### 2026-10-21T14:17:57Z — Christopher Vachon (user)

upgrade.rs + versions.rs ported (upgrade [target] --check --keep --force --json, uninstall, versions [remove|prune], use). clap structs added to src/cli/mod.rs; dispatch in main.rs. merge-pipeline's own 'Task Complete' text paths dropped; no MCP anywhere.
