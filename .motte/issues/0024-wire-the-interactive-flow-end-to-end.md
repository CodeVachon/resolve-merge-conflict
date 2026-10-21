---
id: 24
title: Wire the interactive flow end to end
state: Done
parent: 4
assignee: Christopher Vachon
labels: [cli]
blockedBy: [18, 23, 28]
created: 2026-10-21T13:41:15Z
updated: 2026-10-21T14:07:57Z
---

## Plan

interactive.rs: title, detect state, list conflicted files, questions, run with a spinner or streamed output, verify+stage, report. Ctrl-C mid-run must leave the repo in its conflicted state with a clear message (we never abort the merge).

## Notes

### 2026-10-21T14:07:57Z — Christopher Vachon (user)

src/cli/interactive.rs wires it: load config (warnings to stderr) -> options (config dir for relative promptFile) -> refuse without a TTY unless --yes -> title -> runner with InquirePrompter -> outcome. Exit codes: 0 = all resolved (or nothing to do / dry run / user chose Cancel); 1 = files still unresolved, HEAD moved, or an error; 130 = Esc/Ctrl-C at a question. --json keeps stdout for the JSON only (status chatter to stderr) and has statuses nothing_to_do / dry_run / full report. Spinner (ui::Spinner, stderr, elapsed time) while the agent runs via new Prompter::working() guard (default impl just info()s). Added Agent::program() + agent::program_available() so the RESOLVE_MERGE_CONFLICTS_{CLAUDE,CODEX}_BIN overrides count as 'installed' (this is what lets #25 drive the real binary with a fake agent). Ctrl-C mid-run: SIGINT hits the whole process group so the agent dies with us, and since we never abort/continue anything the repo stays in its conflicted state.
