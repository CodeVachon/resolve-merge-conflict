---
id: 21
title: config path / init / doctor subcommands
state: Done
parent: 3
assignee: Christopher Vachon
labels: [config]
blockedBy: [19, 20, 22]
created: 2026-10-21T13:41:14Z
updated: 2026-10-21T14:10:30Z
---

## Plan

Port src/cli/config_dir.rs, config_init.rs, doctor.rs. init writes a commented-as-possible default config including the default prompt; doctor validates JSON, prompt placeholders, model lists, and that the configured agents are on PATH.

## Notes

### 2026-10-21T14:10:30Z — Christopher Vachon (user)

src/cli/config_cmd.rs. config path (file + which rule chose it + 'does not exist yet' hint); config init (writes config.json AND prompt.md, the default prompt as an editable markdown file referenced by promptFile, so the prompt is edited as prose not an escaped JSON string; keeps existing unless --force; reports created/kept/replaced); config doctor [--json] (errors: bad JSON, agent not claude|codex, unknown placeholders, unreadable promptFile, no agent installed => exit 1; warnings: unknown keys, prompt lacks {{files}} or {{summary_format}}, prompt+promptFile both set, default model not in models list, editor not on PATH). JSON shape {path, rule, agents[{name,installed}], problems[{level,message}], ok}. Starter $schema points at raw.githubusercontent .../main/schema/config.json, which #40 must publish. 9 CLI tests.
