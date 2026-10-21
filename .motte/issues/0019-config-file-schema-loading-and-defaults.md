---
id: 19
title: Config file schema, loading and defaults
state: Todo
parent: 3
labels: [config]
blockedBy: [9]
created: 2026-10-21T13:41:14Z
updated: 2026-10-21T13:41:31Z
---

## Description

Fields: agent, claude.model, claude.models[], codex.model, codex.models[], prompt (template string), promptFile (optional), editor (optional), autoStage (default true). Missing file means built-in defaults. Unknown keys warned.

## Plan

JSON like merge-pipeline for consistency, with a $schema URL. Precedence: flags > env (RESOLVE_MERGE_CONFLICTS_*) > user config > built-in. Open question to settle in implementation: per-repo override at <cwd>/.resolve-merge-conflicts/config.json (merge-pipeline has cwd-level config).
