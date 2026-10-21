---
id: 23
title: "Prompter: agent select, model select, prompt edit, confirm"
state: Done
parent: 4
assignee: Christopher Vachon
labels: [cli]
blockedBy: [16, 22]
created: 2026-10-21T13:41:15Z
updated: 2026-10-21T14:03:36Z
---

## Description

inquire-backed. Agent select (only installed agents offered, default from config). Model select (default from config, free-text escape hatch). Prompt shown, then edited via inquire Editor / $EDITOR; confirm before running.

## Plan

Reuse merge-pipeline's prompter.rs and ui.rs (colors, title, error block). --yes answers every confirmation with its default and skips the edit step.

## Notes

### 2026-10-21T14:03:36Z — Christopher Vachon (user)

src/cli/prompter.rs + src/ui.rs. InquirePrompter implements runner::Prompter: Select (with starting cursor = default), Text (with default), Editor (inquire 'editor' feature; uses config 'editor' split into program+args, else $VISUAL/$EDITOR; .md temp file), show() prints the prompt in a ruled block. Esc/Ctrl-C map to a typed Cancelled error (not a failure) for main to turn into a quiet exit. is_interactive() = stdin and stdout are TTYs. ui.rs: color only on a terminal and when NO_COLOR is unset; title/block/error_block. Interactive behaviour itself can't run in CI; covered by unit tests on editor splitting and cancel mapping, with the logic around it tested via the scripted prompter in #18. Needs a manual run in a real terminal at the end.
