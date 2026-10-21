---
id: 14
title: Agent trait and the Claude Code adapter
state: Done
parent: 2
assignee: Christopher Vachon
labels: [core]
blockedBy: [9]
created: 2026-10-21T13:41:13Z
updated: 2026-10-21T14:31:58Z
---

## Description

Agent { name, models(), run(prompt, model, cwd) -> AgentOutcome }. Claude adapter runs claude -p with --model, restricted tools (Read, Edit, Write, Grep, Glob, read-only git via Bash allowlist), no git add/commit/push, structured final summary.

## Plan

Verify flags against 'claude --help' (-p, --model, --allowedTools, --disallowedTools, --output-format, --permission-mode). Use Context7 for current docs. Stream or capture output; capture final message for the report.

## Notes

### 2026-10-21T13:53:54Z — Christopher Vachon (user)

src/agent/{mod,claude}.rs. Agent trait only DESCRIBES an Invocation (program, args, stdin, cwd) and parses output; agent::run executes it, so flags are unit-testable. Claude: --print --model M --output-format json --permission-mode acceptEdits --permission-prompts none --no-session-persistence, allowedTools = Read/Edit/Write/Grep/Glob + read-only git (diff/show/log/status/ls-files/blame), disallowedTools lists every state-changing git verb as belt and braces. Prompt goes on stdin (variadic tool flags would swallow a positional prompt). Prompt written on a thread to avoid pipe deadlock (1MB test). Binary override env RESOLVE_MERGE_CONFLICTS_CLAUDE_BIN. Verified flags accepted by the real claude 2.1.306 with a haiku smoke call. Output is captured, not streamed (revisit in the CLI epic if a spinner needs progress).

### 2026-10-21T14:31:58Z — Christopher Vachon (user)

Follow-up after CI flake (ETXTBSY 'Text file busy' on ubuntu when a test had just written a fake agent script while a sibling thread forked): agent::spawn_retrying retries up to 40x25ms on ErrorKind::ExecutableFileBusy; used by agent::run and models::discover_codex. It also covers the real-world case of exec'ing an agent that was just installed or updated. macOS cannot reproduce it, so the fix follows the diagnosis; watch CI.
