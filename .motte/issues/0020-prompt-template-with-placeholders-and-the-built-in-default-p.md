---
id: 20
title: Prompt template with placeholders and the built-in default prompt
state: Todo
parent: 3
labels: [config]
blockedBy: [19]
created: 2026-10-21T13:41:14Z
updated: 2026-10-21T13:41:31Z
---

## Description

Default prompt instructs: resolve all conflicts in the listed files, preserve both sides' intent, do not run git add/commit/push/merge --continue, do not touch non-conflicted files, finish with a structured per-file summary. Placeholders: {{files}}, {{operation}}, {{ours}}, {{theirs}}, {{branch}}.

## Plan

Simple {{name}} substitution, no template engine. Unknown placeholder is a config doctor error.
