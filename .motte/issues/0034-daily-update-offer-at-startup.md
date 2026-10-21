---
id: 34
title: Daily update offer at startup
state: Done
parent: 6
assignee: Christopher Vachon
labels: [self-update]
blockedBy: [32]
created: 2026-10-21T13:41:17Z
updated: 2026-10-21T14:17:58Z
---

## Plan

Port update_prompt.rs: at most once a day, interactive only, managed installs only, off via env or CI.

## Notes

### 2026-10-21T14:17:57Z — Christopher Vachon (user)

update_prompt.rs ported with its own tiny Confirm trait (blanket impl over runner::Prompter, AssumeDefault under --yes so unattended runs update without a TTY). Wired into cli/interactive.rs before anything else; skipped with --json. Adapted tests/update_prompt.rs to this tool: the 'run proceeded' sentinel is the not-inside-a-git-repository error. 5 integration tests via a fake GitHub server.
