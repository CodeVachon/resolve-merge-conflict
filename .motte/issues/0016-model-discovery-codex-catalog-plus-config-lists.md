---
id: 16
title: "Model discovery: Codex catalog plus config lists"
state: Todo
parent: 2
labels: [core]
blockedBy: [14, 15]
created: 2026-10-21T13:41:13Z
updated: 2026-10-21T13:41:29Z
---

## Description

Codex models come live from 'codex debug models' (JSON), falling back to the config list on failure. Claude models come from the config list, shipped with defaults (opus, sonnet, haiku plus full ids).

## Plan

Parse only the fields we need from the catalog and tolerate unknown shape changes. Short timeout. Cache not required.
