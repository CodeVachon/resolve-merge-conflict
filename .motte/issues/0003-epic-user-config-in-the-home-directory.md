---
id: 3
title: "Epic: User config in the home directory"
state: Todo
labels: [config]
created: 2026-10-21T13:40:40Z
updated: 2026-10-21T13:40:40Z
---

## Description

A config file in the user's home config dir (~/.config/resolve-merge-conflicts/ on Linux, ~/Library/Application Support/... on macOS, mirroring merge-pipeline's config_dir.rs) holding defaults: agent, per-agent default model, per-agent model list, and the prompt template. Flags and env override it; a config init/path/doctor trio mirrors merge-pipeline.
