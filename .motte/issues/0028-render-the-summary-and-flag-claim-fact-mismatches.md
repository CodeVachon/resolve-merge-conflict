---
id: 28
title: Render the summary and flag claim/fact mismatches
state: Done
parent: 5
assignee: Christopher Vachon
labels: [report]
blockedBy: [26, 27]
created: 2026-10-21T13:41:16Z
updated: 2026-10-21T14:06:45Z
---

## Plan

Terminal table plus next steps (review with git diff --staged, then commit/continue yourself). Mismatch examples: agent says resolved but markers remain; agent says file X handled but X untouched. Add --json output for scripts.

## Notes

### 2026-10-21T14:06:44Z — Christopher Vachon (user)

Report::mismatches() (report.rs) and src/render.rs. Mismatch kinds, most serious first: HeadMoved, TouchedOutside, ClaimedDoneButUnresolved, StrategyDiffers (claimed vs content-derived), NotMentioned (only when a summary exists), UnexpectedPath, ReportedFailureButResolved. render::text: header (agent, model, operation, n of m resolved), per-file mark/kind/strategy/+-/staged plus the agent's one-line reason, raw agent words when no structured summary, 'Look closer at' list, agent failure + stderr tail, then Next: unresolved files named, git diff --staged (or add hint if staging was off), the operation's own --continue command only when everything is resolved, and always 'Nothing was committed or pushed'. render::json = report + mismatches. Colors only on a terminal, so tests see plain text.
