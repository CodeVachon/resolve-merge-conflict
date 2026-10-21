---
id: 22
title: Clap command surface and flags
state: Done
parent: 4
assignee: Christopher Vachon
labels: [cli]
blockedBy: [9]
created: 2026-10-21T13:41:15Z
updated: 2026-10-21T14:02:32Z
---

## Plan

Flags: -c/--cwd, -a/--agent, -m/--model, -y/--yes, --prompt-file, --no-edit, --dry-run (show the prompt, run nothing), --config. Subcommands: config, upgrade, uninstall, versions, use, install, completion, mcp (if built). Unknown flags exit 2, failures exit 1 with an Error block.

## Notes

### 2026-10-21T14:02:32Z — Christopher Vachon (user)

src/cli/mod.rs: Cli{RunArgs flattened, Option<Command>}. Flags: -c/--cwd, -a/--agent (env RESOLVE_MERGE_CONFLICTS_AGENT), -m/--model (env ..._MODEL), -y/--yes, --no-edit, --dry-run, --prompt-file, --no-stage, -f/--config, --json. RunArgs::to_options reads --prompt-file immediately so a bad path fails before any question. Subcommands so far: config {path, doctor, init} and completion; self-update subcommands (upgrade/uninstall/versions/use) are added by #32 to keep issues independent. Unknown flag = exit 2 (clap). main.rs dispatches completion; the interactive flow and config subcommands still return a 'not wired up' error until #24 and #21. Added clap 'env' feature and clap_complete.
