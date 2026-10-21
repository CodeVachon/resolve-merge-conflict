---
id: 24
title: Wire the interactive flow end to end
state: Todo
parent: 4
labels: [cli]
blockedBy: [18, 23, 28]
created: 2026-10-21T13:41:15Z
updated: 2026-10-21T13:41:32Z
---

## Plan

interactive.rs: title, detect state, list conflicted files, questions, run with a spinner or streamed output, verify+stage, report. Ctrl-C mid-run must leave the repo in its conflicted state with a clear message (we never abort the merge).
