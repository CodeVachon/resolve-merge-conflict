---
id: 26
title: Report data model and collection
state: Done
parent: 5
assignee: Christopher Vachon
labels: [report]
blockedBy: [17]
created: 2026-10-21T13:41:15Z
updated: 2026-10-21T14:05:30Z
---

## Plan

Per file: conflict type, resolved?, markers left?, staged?, +/- lines, strategy claimed by agent (ours/theirs/merged/custom) and the agent's rationale. Collected from verifier output plus parsed agent summary.

## Notes

### 2026-10-21T14:05:30Z — Christopher Vachon (user)

src/report.rs: Report::build(&Finished) joins verifier facts (status, actual strategy, staged, +/-) with the agent's claims (claimed strategy, explanation, mentioned) per file, plus head_moved, touched_outside, has_agent_summary, unexpected_summary_paths, raw agent_message, stderr tail (failed runs only). Facts and claims are separate fields so nothing downstream can confuse them. Serialize derived (snake_case) for --json; added Serialize to ConflictKind, Operation, Strategy, FileStatus. Comparison of claim vs fact is #28's job (Report::mismatches), computed on demand from this model rather than stored.
