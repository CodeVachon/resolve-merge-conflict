---
id: 16
title: "Model discovery: Codex catalog plus config lists"
state: Done
parent: 2
assignee: Christopher Vachon
labels: [core]
blockedBy: [14, 15]
created: 2026-10-21T13:41:13Z
updated: 2026-10-21T13:59:57Z
---

## Description

Codex models come live from 'codex debug models' (JSON), falling back to the config list on failure. Claude models come from the config list, shipped with defaults (opus, sonnet, haiku plus full ids).

## Plan

Parse only the fields we need from the catalog and tolerate unknown shape changes. Short timeout. Cache not required.

## Notes

### 2026-10-21T13:59:57Z — Christopher Vachon (user)

src/models.rs. options(agent, config) -> ModelOptions{models, default, source, note}. Claude: config list (+ built-in opus/sonnet/haiku, default sonnet). Codex: 'codex debug models' parsed for slug where visibility is 'list' (or absent), sorted by priority; config-only models appended so users can still pick them; default = config default else best-priority discovered. Any discovery failure (missing binary, non-zero, 10s timeout, unparseable) falls back to config and sets note explaining why. stdout drained on a thread (catalog is large; 500KB test). Real-codex check available via cargo test -- --ignored (verified against codex-cli 0.163.0).
