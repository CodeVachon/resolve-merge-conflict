---
id: 23
title: "Prompter: agent select, model select, prompt edit, confirm"
state: Todo
parent: 4
labels: [cli]
blockedBy: [16, 22]
created: 2026-10-21T13:41:15Z
updated: 2026-10-21T13:41:32Z
---

## Description

inquire-backed. Agent select (only installed agents offered, default from config). Model select (default from config, free-text escape hatch). Prompt shown, then edited via inquire Editor / $EDITOR; confirm before running.

## Plan

Reuse merge-pipeline's prompter.rs and ui.rs (colors, title, error block). --yes answers every confirmation with its default and skips the edit step.
