---
id: 21
title: config path / init / doctor subcommands
state: Todo
parent: 3
labels: [config]
blockedBy: [19, 20, 22]
created: 2026-10-21T13:41:14Z
updated: 2026-10-21T13:41:31Z
---

## Plan

Port src/cli/config_dir.rs, config_init.rs, doctor.rs. init writes a commented-as-possible default config including the default prompt; doctor validates JSON, prompt placeholders, model lists, and that the configured agents are on PATH.
