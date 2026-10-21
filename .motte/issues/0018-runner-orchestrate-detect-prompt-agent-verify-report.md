---
id: 18
title: "Runner: orchestrate detect, prompt, agent, verify, report"
state: Done
parent: 2
assignee: Christopher Vachon
labels: [core]
blockedBy: [13, 14, 15, 17, 19, 20]
created: 2026-10-21T13:41:14Z
updated: 2026-10-21T14:01:32Z
---

## Plan

Mirror merge-pipeline's Prompter trait so the runner is testable with a scripted prompter and a fake Agent. Dirty-tree rules: unlike merge-pipeline we EXPECT a dirty tree (conflicts), so refuse only if there are no conflicts. Warn if there are non-conflicted staged/unstaged changes that the agent might touch.

## Notes

### 2026-10-21T14:01:32Z — Christopher Vachon (user)

src/runner.rs. run(&Options, &Config, &mut dyn Prompter, &Deps) -> Outcome{NothingToDo|Cancelled|DryRun(Plan)|Finished}. Prompter trait: select/text/edit/show/info. Deps injects make_agent, installed, model_options (Deps::real() for production), so tests use a shell-script FakeAgent and a scripted prompter; no real agent anywhere in tests. Flow: detect -> list files -> note pre-existing unrelated changes -> choose agent (flag validated; one installed = auto; several = ask, preselect config.agent; --yes = config.agent or first) -> choose model (flag; else list with default; empty list = free text, or error under --yes telling the user to pass --model) -> render template -> show prompt, loop Run/Edit/Cancel (skipped by --yes; --no-edit only shows) -> dry-run stops here -> Snapshot -> agent -> verify+stage. The edited prompt is sent verbatim (not re-rendered, so literal braces in an edit survive). Operates from the repo top level even if cwd is a subdirectory. 10 tests incl. amend, cancel, markers left, staging off, missing/unknown agent. Gotcha recorded: 'git diff --cached' lists UNMERGED paths too, so 'nothing staged' must be asserted via verification.staged, not the index listing.
