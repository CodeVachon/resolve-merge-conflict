---
id: 19
title: Config file schema, loading and defaults
state: Done
parent: 3
assignee: Christopher Vachon
labels: [config]
blockedBy: [9]
created: 2026-10-21T13:41:14Z
updated: 2026-10-21T13:58:06Z
---

## Description

Fields: agent, claude.model, claude.models[], codex.model, codex.models[], prompt (template string), promptFile (optional), editor (optional), autoStage (default true). Missing file means built-in defaults. Unknown keys warned.

## Plan

JSON like merge-pipeline for consistency, with a $schema URL. Precedence: flags > env (RESOLVE_MERGE_CONFLICTS_*) > user config > built-in. Open question to settle in implementation: per-repo override at <cwd>/.resolve-merge-conflicts/config.json (merge-pipeline has cwd-level config).

## Notes

### 2026-10-21T13:58:06Z — Christopher Vachon (user)

src/config.rs. JSON, camelCase, all optional: $schema, agent, prompt, promptFile, editor, autoStage, claude{model,models}, codex{model,models}. Missing user file = defaults; missing --config/env file = error (typo, not absence); unknown keys (incl. nested) are warnings. Claude defaults opus/sonnet/haiku, default model sonnet; default model is always in the offered list. Location: --config > $RESOLVE_MERGE_CONFLICTS_CONFIG > $XDG_CONFIG_HOME or ~/.config/resolve-merge-conflicts/config.json. DECISION: ~/.config on macOS too (user asked for a 'home dir' file), unlike merge-pipeline's ~/Library/Application Support; Windows uses dirs::config_dir. DECISION on the open question: NO per-repo override in v1. Flag/env overrides of agent/model are applied in the CLI (#22), not here.
