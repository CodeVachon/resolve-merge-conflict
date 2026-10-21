---
id: 15
title: Codex adapter (sandboxed, no approval prompts)
state: Todo
parent: 2
labels: [core]
blockedBy: [9]
created: 2026-10-21T13:41:13Z
updated: 2026-10-21T13:41:29Z
---

## Description

Runs codex exec with -m <model>, --sandbox workspace-write, no interactive approvals, --output-last-message and --output-schema for the structured summary. The agent only edits files; the tool stages. This removes the git-add approval annoyance because the agent never touches .git.

## Plan

Verify flags against 'codex exec --help' (-s, -C, --ephemeral, -o, --output-schema, --json, approval-policy via -c). Confirm workspace-write can edit conflicted files but we never need git writes. Fallback if the sandbox blocks .git reads for git diff: have the tool inline the three-way blobs into the prompt.
