---
id: 25
title: End-to-end CLI tests with a fake agent binary
state: Done
parent: 4
assignee: Christopher Vachon
labels: [cli]
blockedBy: [24]
created: 2026-10-21T13:41:15Z
updated: 2026-10-21T14:08:57Z
---

## Plan

A tiny fake claude/codex script on PATH for tests, driven by env to resolve, partially resolve or fail. Covers scripted -y runs and exit codes.

## Notes

### 2026-10-21T14:08:56Z — Christopher Vachon (user)

tests/cli_flow.rs: the REAL binary run in real conflicted repos with a fake claude script (via RESOLVE_MERGE_CONFLICTS_CLAUDE_BIN; modes resolve/lazy/fail/commit; XDG_CONFIG_HOME isolated). 13 tests: resolve+stage+report with HEAD unchanged; agent receives model, acceptEdits, commit disallowed, and the default prompt; unresolved => exit 1 and nothing staged; failing agent => exit 1 + stderr shown; agent that commits => 'HEAD moved' + exit 1; clean repo => exit 0, agent never ran; --dry-run runs nothing; --json prints only the report on stdout; --prompt-file and the home-config prompt reach the agent; --no-stage; no TTY and no --yes refuses; missing agent; works from a subdirectory.
