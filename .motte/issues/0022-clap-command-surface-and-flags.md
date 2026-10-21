---
id: 22
title: Clap command surface and flags
state: Todo
parent: 4
labels: [cli]
blockedBy: [9]
created: 2026-10-21T13:41:15Z
updated: 2026-10-21T13:41:31Z
---

## Plan

Flags: -c/--cwd, -a/--agent, -m/--model, -y/--yes, --prompt-file, --no-edit, --dry-run (show the prompt, run nothing), --config. Subcommands: config, upgrade, uninstall, versions, use, install, completion, mcp (if built). Unknown flags exit 2, failures exit 1 with an Error block.
