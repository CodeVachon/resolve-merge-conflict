---
id: 2
title: "Epic: Core library — conflict detection, agent adapters, run flow"
state: Done
labels: [core]
created: 2026-10-21T13:40:40Z
updated: 2026-10-21T14:24:44Z
---

## Description

Library crate behind the CLI. Detects an in-progress conflict state (merge, rebase, cherry-pick, revert, stash pop) via unmerged paths; never starts a merge, exits cleanly when none. Adapters for Claude Code and Codex run the agent non-interactively with a prompt and chosen model. Agent edits files only; the TOOL verifies no conflict markers remain and stages resolved files with git add. Never commits, never pushes.

## Notes

### 2026-10-21T13:41:36Z — Christopher Vachon (user)

Decisions (2026-10-21, user answered clarifying questions): (1) Tool stages — agent edits files only, tool verifies no markers then runs git add; agent never gets git write access, which removes Codex's approval prompts. (2) Model lists: Codex discovered live via 'codex debug models' with config fallback; Claude from config list (no list command exists). (3) Scope: any in-progress conflict state (merge/rebase/cherry-pick/revert/stash pop); never starts a merge; clean exit if none. (4) Report: tool-verified facts + agent structured summary, mismatches flagged. Never commit, never push, never --continue.

### 2026-10-21T14:24:44Z — Christopher Vachon (user)

VERIFIED END TO END with the real agents (2026-10-21): a conflicted merge in a scratch repo resolved by claude (haiku) and by codex (gpt-6.1-sol, discovered from its catalog, run with --yes): both produced a correct merge of both sides, a per-file explanation in the report, a staged file, HEAD unchanged, and Codex needed zero approval prompts.
