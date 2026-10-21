---
id: 20
title: Prompt template with placeholders and the built-in default prompt
state: Done
parent: 3
assignee: Christopher Vachon
labels: [config]
blockedBy: [19]
created: 2026-10-21T13:41:14Z
updated: 2026-10-21T13:59:00Z
---

## Description

Default prompt instructs: resolve all conflicts in the listed files, preserve both sides' intent, do not run git add/commit/push/merge --continue, do not touch non-conflicted files, finish with a structured per-file summary. Placeholders: {{files}}, {{operation}}, {{ours}}, {{theirs}}, {{branch}}.

## Plan

Simple {{name}} substitution, no template engine. Unknown placeholder is a config doctor error.

## Notes

### 2026-10-21T13:59:00Z — Christopher Vachon (user)

src/prompt.rs. Placeholders: files (bullets with conflict kind), operation, ours, theirs, branch, summary_format. Plain substitution; unknown placeholder is a hard error naming it and the known set (so a typo is never sent to the agent); whitespace inside braces tolerated; dangling '{{' left alone. Template source: promptFile (relative to the config dir, ~ expanded) > prompt > built-in default. Default prompt tells the agent: keep both sides' intent, remove all markers, only edit listed files, NEVER run state-changing git commands, don't continue/abort. DECISION: the JSON summary request lives in a separate {{summary_format}} placeholder that the default template includes; a custom template that omits it simply gets no structured summary and the report (#27/#28) falls back to the agent's raw message. Chose this over silently appending to user prompts because the user must see and own exactly what is sent.
