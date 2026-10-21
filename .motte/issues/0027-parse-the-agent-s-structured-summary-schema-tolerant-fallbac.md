---
id: 27
title: Parse the agent's structured summary (schema + tolerant fallback)
state: Todo
parent: 5
labels: [report]
blockedBy: [14, 15]
created: 2026-10-21T13:41:16Z
updated: 2026-10-21T13:41:33Z
---

## Plan

Codex via --output-schema; Claude via instruction plus JSON block extraction. If parsing fails, show the raw final message and say the structured summary was unavailable instead of failing the run.
