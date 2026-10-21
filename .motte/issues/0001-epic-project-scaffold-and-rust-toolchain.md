---
id: 1
title: "Epic: Project scaffold and Rust toolchain"
state: Todo
labels: [scaffold]
created: 2026-10-21T13:40:26Z
updated: 2026-10-21T13:40:40Z
---

## Description

Stand up the Rust crate mirroring CodeVachon/merge-pipeline: lib + bin split, pinned toolchain, release profile, CI (fmt, clippy, test on ubuntu + macos), repo hygiene. Binary is named `resolve-merge-conflicts` (crate dir is resolve-merge-conflict); decide the crate name in the first child and record it as a note. Reference layout: src/{cli,config,git,runner,selfupdate,mcp,ui,prompt}.rs, tests/, examples/, schema/, scripts/, install.sh, install.ps1.
