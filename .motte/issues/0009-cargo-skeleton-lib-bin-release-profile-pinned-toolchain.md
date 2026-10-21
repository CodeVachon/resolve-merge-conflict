---
id: 9
title: "Cargo skeleton: lib + bin, release profile, pinned toolchain"
state: Done
parent: 1
assignee: Christopher Vachon
labels: [scaffold]
created: 2026-10-21T13:41:12Z
updated: 2026-10-21T13:42:44Z
---

## Description

Mirror merge-pipeline's Cargo.toml, rust-toolchain.toml, src/lib.rs + src/main.rs split, release profile (strip, lto).

## Plan

Check merge-pipeline's Cargo.toml for the dependency set (clap, inquire, serde, serde_json, semver, flate2, ...). Binary name resolve-merge-conflicts. cargo build/test/clippy clean.

## Notes

### 2026-10-21T13:42:43Z — Christopher Vachon (user)

Crate and bin: resolve-merge-conflicts (lib resolve_merge_conflicts); repo URL CodeVachon/resolve-merge-conflict. Toolchain: stable + rustfmt/clippy, edition 2024, rust-version 1.98 (as merge-pipeline). Deliberately minimal deps (anyhow, clap); later issues add inquire, serde, sha2, ureq etc. as they need them. Verified: cargo build, test (version smoke test), fmt --check, clippy -D warnings, release build (~490KB). Not committed.
