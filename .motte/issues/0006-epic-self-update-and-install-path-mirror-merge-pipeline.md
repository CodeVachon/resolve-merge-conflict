---
id: 6
title: "Epic: Self-update and install path (mirror merge-pipeline)"
state: Todo
labels: [self-update]
created: 2026-10-21T13:40:40Z
updated: 2026-10-21T13:40:40Z
---

## Description

Replicate merge-pipeline's install/upgrade contract so both tools behave identically: ~/.resolve-merge-conflicts/versions/v<X.Y.Z>/bin/, current symlink, ~/.local/bin link, install.sh/install.ps1 with checksum verification, upgrade/uninstall/versions/use/prune subcommands, daily update offer, completion scripts. Env vars RESOLVE_MERGE_CONFLICTS_{VERSION,INSTALL_DIR,BIN_DIR,NO_MODIFY_PATH,DOWNLOAD_BASE,NO_UPDATE_CHECK}.
