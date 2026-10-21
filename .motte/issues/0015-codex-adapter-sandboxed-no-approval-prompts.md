---
id: 15
title: Codex adapter (sandboxed, no approval prompts)
state: Done
parent: 2
assignee: Christopher Vachon
labels: [core]
blockedBy: [9]
created: 2026-10-21T13:41:13Z
updated: 2026-10-21T13:55:02Z
---

## Description

Runs codex exec with -m <model>, --sandbox workspace-write, no interactive approvals, --output-last-message and --output-schema for the structured summary. The agent only edits files; the tool stages. This removes the git-add approval annoyance because the agent never touches .git.

## Plan

Verify flags against 'codex exec --help' (-s, -C, --ephemeral, -o, --output-schema, --json, approval-policy via -c). Confirm workspace-write can edit conflicted files but we never need git writes. Fallback if the sandbox blocks .git reads for git diff: have the tool inline the three-way blobs into the prompt.

## Notes

### 2026-10-21T13:55:02Z — Christopher Vachon (user)

src/agent/codex.rs. codex exec --model M --sandbox workspace-write -c approval_policy="never" --ephemeral --cd DIR --output-last-message FILE - (prompt on stdin). No dangerously-* flags ever (asserted in a test). Final message read from the output file (Invocation.output_file, added to the shared type; removed after reading; stdout is the fallback). Verified against real codex-cli 0.163.0 with a trivial prompt. Structured --output-schema deliberately NOT used yet: #27 parses a JSON block tolerantly from the final message for both agents; revisit if Codex output proves unreliable. Hooks in ~/.codex still run (--ignore-user-config would drop auth config too, so not used).
