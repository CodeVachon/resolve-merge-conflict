---
id: 11
title: "Repo hygiene: .gitignore, AGENTS.md cargo notes, test fixtures helper"
state: Done
parent: 1
assignee: Christopher Vachon
labels: [scaffold]
blockedBy: [9]
created: 2026-10-21T13:41:13Z
updated: 2026-10-21T13:44:10Z
---

## Plan

tests/common/mod.rs builds temp git repos with conflicts (merge, rebase, cherry-pick, stash pop) so no test needs network or a real agent.

## Notes

### 2026-10-21T13:44:10Z — Christopher Vachon (user)

Fixture helper in tests/common/mod.rs: one repo, a.txt conflicted (main vs feature) for merge, rebase, cherry-pick, revert and stash pop, plus a delete/modify merge (b.txt, no markers). Isolation env (no global config, hooks, signing; GIT_EDITOR=true). Helpers: raw/try_raw, unmerged(), staged(), head(), read/write. tests/fixtures.rs proves each scenario is genuinely conflicted, so downstream tests cannot pass vacuously. .gitignore was done in #9; AGENTS.md now has build/test notes and the never-commit invariant. Not committed.
