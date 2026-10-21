---
id: 39
title: "README: install, usage, config, flags, upgrade"
state: Done
parent: 8
assignee: Christopher Vachon
labels: [docs]
blockedBy: [24]
created: 2026-10-21T13:41:18Z
updated: 2026-10-21T14:21:14Z
---

## Notes

### 2026-10-21T14:21:14Z — Christopher Vachon (user)

README.md written against the implemented behaviour: install (env vars), usage flow, every flag, exit codes (0/1/2/130), the guarantees (what it will and will not do, incl. why Codex no longer asks for git add approval), conflict kinds and the rebase ours/theirs swap, config (location, resolution order, fields, models, prompt placeholders), the report and its 'Look closer at' list, upgrade/versions/use/uninstall, completion, development. Added LICENSE (MIT, copied from merge-pipeline) because Cargo.toml declares MIT. The mdBook site (#41) will reuse this content.
