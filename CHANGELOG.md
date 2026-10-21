# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

When cutting a release, rename `Unreleased` to the version and date, add a fresh empty `Unreleased`
above it, and make sure `Cargo.toml` carries the same version as the tag.

## [Unreleased]

### Changed

- Amending the prompt now opens `vi` when neither the config's `editor`, `$VISUAL` nor `$EDITOR` is set, instead of nano.

## [0.1.0] - 2026-10-21

First release.

### Added

- `resolve-merge-conflicts` resolves git conflicts with a CLI agent. Run it in a repository stopped
  on a merge, rebase, cherry-pick, revert or stash pop: pick an agent (Claude Code or Codex) and a
  model, review and optionally amend the prompt, and the agent edits the conflicted files. It then
  stages the files that are really resolved and prints a report. It never commits, pushes,
  continues or aborts.
- The report shows what the tool verified (resolved or not, which side the content matches, lines
  changed) beside what the agent said it did, and flags any disagreement, any commit made by the
  agent, and any file changed outside the conflict. `--json` prints it for scripts.
- Codex runs in its `workspace-write` sandbox with approvals off and never needs git write access,
  so it no longer stops to ask permission to `git add`; this tool stages instead. Claude Code runs
  with read-only git allowed and every state-changing git command denied.
- Config in `~/.config/resolve-merge-conflicts/config.json`: default agent, per-agent default model
  and model list, editor, `autoStage`, and the prompt (inline or as a file) with `{{files}}`,
  `{{operation}}`, `{{ours}}`, `{{theirs}}`, `{{branch}}` and `{{summary_format}}` placeholders.
  `config init`, `config path` and `config doctor` manage it; a JSON schema is published.
- Codex models are discovered from `codex debug models`; Claude models come from config.
- Flags for every question (`--agent`, `--model`, `--yes`, `--no-edit`, `--dry-run`,
  `--prompt-file`, `--no-stage`, `--config`, `--json`) so runs can be scripted.
- Install and update like merge-pipeline: `install.sh` / `install.ps1`, `upgrade`, `versions`,
  `use`, `uninstall`, a daily update offer, and shell completion.
