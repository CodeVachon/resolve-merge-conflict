---
id: 40
title: Publish config JSON schema and examples/config
state: Done
parent: 8
assignee: Christopher Vachon
labels: [docs]
blockedBy: [19]
created: 2026-10-21T13:41:18Z
updated: 2026-10-21T14:19:21Z
---

## Notes

### 2026-10-21T14:19:21Z — Christopher Vachon (user)

schema/config.json (draft-07, additionalProperties:false at both levels so editors flag typos) with $id at raw.githubusercontent .../main/schema/config.json, which is what 'config init' writes as $schema. examples/config: config.json (all fields), inline-prompt.json (prompt with placeholders), prompt.md (the default prompt). tests/schema.rs is the drift alarm: schema keys must equal the keys of the Config struct, agent enum must equal agent::names(), and every example and the starter config must load with zero warnings. CI gets a 'schema' job running check-jsonschema over examples; not runnable locally (no pipx/jsonschema here), so it is verified by the CI run.
