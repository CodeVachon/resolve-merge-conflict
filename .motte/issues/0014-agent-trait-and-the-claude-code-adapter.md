---
id: 14
title: Agent trait and the Claude Code adapter
state: Todo
parent: 2
labels: [core]
blockedBy: [9]
created: 2026-10-21T13:41:13Z
updated: 2026-10-21T13:41:29Z
---

## Description

Agent { name, models(), run(prompt, model, cwd) -> AgentOutcome }. Claude adapter runs claude -p with --model, restricted tools (Read, Edit, Write, Grep, Glob, read-only git via Bash allowlist), no git add/commit/push, structured final summary.

## Plan

Verify flags against 'claude --help' (-p, --model, --allowedTools, --disallowedTools, --output-format, --permission-mode). Use Context7 for current docs. Stream or capture output; capture final message for the report.
