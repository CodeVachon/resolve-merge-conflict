---
id: 27
title: Parse the agent's structured summary (schema + tolerant fallback)
state: Done
parent: 5
assignee: Christopher Vachon
labels: [report]
blockedBy: [14, 15]
created: 2026-10-21T13:41:16Z
updated: 2026-10-21T14:04:29Z
---

## Plan

Codex via --output-schema; Claude via instruction plus JSON block extraction. If parsing fails, show the raw final message and say the structured summary was unavailable instead of failing the run.

## Notes

### 2026-10-21T14:04:29Z — Christopher Vachon (user)

src/summary.rs: parse(message) -> Option<AgentSummary{files:[Entry{path,strategy,summary}]}>. Forgiving about WHERE (fenced block with or without language tag, bare JSON, JSON buried in prose via string-aware brace matching; last valid candidate wins), strict about WHAT (needs a 'files' array of objects with a path). Strategy wording normalised (took theirs, merged both, removed...) and the literal 'ours | theirs | merged | deleted' placeholder copied from the prompt is rejected rather than guessed. 'reason'/'explanation' accepted for 'summary'. None means 'agent did not give a usable summary' and is normal, not an error: the report falls back to the raw message. DECISION: did not use Codex --output-schema (see #15 note); one parser serves both agents.
