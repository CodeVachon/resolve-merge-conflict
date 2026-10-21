---
id: 18
title: "Runner: orchestrate detect, prompt, agent, verify, report"
state: Todo
parent: 2
labels: [core]
blockedBy: [13, 14, 15, 17, 19, 20]
created: 2026-10-21T13:41:14Z
updated: 2026-10-21T13:41:31Z
---

## Plan

Mirror merge-pipeline's Prompter trait so the runner is testable with a scripted prompter and a fake Agent. Dirty-tree rules: unlike merge-pipeline we EXPECT a dirty tree (conflicts), so refuse only if there are no conflicts. Warn if there are non-conflicted staged/unstaged changes that the agent might touch.
