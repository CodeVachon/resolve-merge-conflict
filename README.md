# resolve-merge-conflicts

Resolve git merge conflicts with a CLI agent, then review the result yourself.

Run it in a repository that is stopped on conflicts. It asks which agent (Claude Code or Codex) and
which model to use, shows you the prompt so you can amend it, lets the agent edit the conflicted
files, stages what was really resolved, and prints a report of what happened.

**It never commits and never pushes.** It does not run `--continue` or `--abort` either. What you do
with the result is up to you.

A single self-contained binary with `git` as its only prerequisite (plus the agent you want to use:
[`claude`](https://claude.com/claude-code) and/or [`codex`](https://github.com/openai/codex)). It
updates itself in place, the same way [merge-pipeline](https://github.com/CodeVachon/merge-pipeline)
does.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/CodeVachon/resolve-merge-conflict/main/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/CodeVachon/resolve-merge-conflict/main/install.ps1 | iex
```

The installer verifies the download against the release's `checksums.txt`, puts the binary in
`~/.resolve-merge-conflicts/versions/v<X.Y.Z>/bin/`, points `~/.resolve-merge-conflicts/current` at
it, and links `~/.local/bin/resolve-merge-conflicts`. It never edits your shell profile; it tells you
if `~/.local/bin` is not on your `PATH`. Installer environment variables:

| Variable                                | Purpose                                                |
| --------------------------------------- | ------------------------------------------------------ |
| `RESOLVE_MERGE_CONFLICTS_VERSION`       | install a specific version instead of the latest       |
| `RESOLVE_MERGE_CONFLICTS_INSTALL_DIR`   | root instead of `~/.resolve-merge-conflicts`           |
| `RESOLVE_MERGE_CONFLICTS_BIN_DIR`       | symlink directory instead of `~/.local/bin`            |
| `RESOLVE_MERGE_CONFLICTS_NO_MODIFY_PATH`| skip the PATH advice                                   |
| `RESOLVE_MERGE_CONFLICTS_DOWNLOAD_BASE` | fetch assets from a mirror or a local directory server |

Builds are published for darwin-arm64, darwin-x64, linux-x64, linux-arm64 and windows-x64.

## Usage

A merge (or rebase, cherry-pick, revert, stash pop) stops on conflicts. In that repository:

```
$ resolve-merge-conflicts
```

1. It lists the conflicted files and what kind of conflict each is.
2. You pick an agent (only installed ones are offered) and a model.
3. You see the prompt, with the conflicted files filled in. Choose **Run with this prompt**,
   **Edit the prompt** (opens your editor), or **Cancel**.
4. The agent runs and edits the conflicted files. Nothing else.
5. The tool checks every file for leftover conflict markers, stages the ones that are clean, and
   prints the report.

Then review with `git diff --staged` and finish the operation yourself (`git merge --continue`,
`git rebase --continue`, `git commit`, ...). The report tells you which.

Every question can be answered up front:

| Flag                    | Effect                                                                         |
| ----------------------- | ------------------------------------------------------------------------------ |
| `-c, --cwd <DIR>`       | Repository to work in (default: current directory)                             |
| `-a, --agent <AGENT>`   | `claude` or `codex` (env `RESOLVE_MERGE_CONFLICTS_AGENT`)                      |
| `-m, --model <MODEL>`   | Model, as the agent names it (env `RESOLVE_MERGE_CONFLICTS_MODEL`)             |
| `-y, --yes`             | Answer every question with its default and send the prompt without review      |
| `--no-edit`             | Show the prompt but do not offer to edit it                                    |
| `--dry-run`             | Build and show the prompt, then stop without running the agent                 |
| `--prompt-file <FILE>`  | Use the prompt template in this file instead of the configured one             |
| `--no-stage`            | Leave resolved files unstaged                                                  |
| `-f, --config <FILE>`   | Config file to use instead of the one in your home directory                   |
| `--json`                | Print the final report as JSON on stdout (everything else goes to stderr)      |

A fully scripted run:

```sh
resolve-merge-conflicts -y -a claude -m sonnet
```

Without `--yes` it needs a terminal and refuses to run otherwise rather than hang.

### Exit codes

| Code | Meaning                                                                                       |
| ---- | --------------------------------------------------------------------------------------------- |
| 0    | Every conflicted file is resolved, or there was nothing to do, or you chose Cancel / `--dry-run` |
| 1    | Some files are still conflicted, `HEAD` moved while the agent ran, or an error                 |
| 2    | Unknown flag or bad usage                                                                      |
| 130  | You pressed Esc or Ctrl-C at a question                                                        |

## What it does, and what it will not do

- **Only edits conflicted files.** The prompt says so, and afterwards the tool reports any other file
  that changed (or appeared) while the agent ran.
- **Never gives the agent git write access.** Claude Code is run with an allow-list of read-only git
  commands and an explicit deny-list of everything that changes git state. Codex is run in its
  `workspace-write` sandbox with approvals off, so it can edit files but is never stopped to ask for
  permission. This is what removes Codex's `git add` approval prompt: the agent does not stage;
  **this tool does**, afterwards.
- **Stages only what is really resolved.** A file is staged only if no conflict markers remain. A
  file the agent left conflicted stays conflicted and is named in the report.
- **Never commits, pushes, continues or aborts.** If the agent commits anyway, `HEAD` moving is
  detected, flagged at the top of the report, and the run exits 1.
- **Does not trust the agent's account.** The report shows what the tool verified (resolved or not,
  which side the content matches, lines changed) next to what the agent *said* it did, and flags
  every disagreement.

Conflicts it understands: both modified, both added, both deleted, added by us/them, deleted by
us/them. Deleting a file is accepted as a resolution only for the conflict kinds where a deletion is
one of the choices.

During a **rebase**, git swaps the meaning of the two sides ("ours" is the branch being rebased
onto; "theirs" is your commit being replayed). The prompt describes the sides in plain language for
the operation in progress, so the agent is not misled by the words.

## Config

An optional JSON file in your home config directory:

| Platform        | Default location                                         |
| --------------- | -------------------------------------------------------- |
| macOS and Linux | `~/.config/resolve-merge-conflicts/config.json` (honours `$XDG_CONFIG_HOME`) |
| Windows         | `%APPDATA%\resolve-merge-conflicts\config.json`          |

Resolved in this order: `--config <FILE>`, then `$RESOLVE_MERGE_CONFLICTS_CONFIG`, then the file
above. A missing default file just means built-in defaults; a missing file you named explicitly is an
error.

```sh
resolve-merge-conflicts config init     # writes config.json and an editable prompt.md
resolve-merge-conflicts config path     # prints the file in effect and the rule that chose it
resolve-merge-conflicts config doctor   # validates it; exit 1 if a run would fail (--json too)
```

```json
{
  "$schema": "https://raw.githubusercontent.com/CodeVachon/resolve-merge-conflict/main/schema/config.json",
  "agent": "claude",
  "autoStage": true,
  "promptFile": "prompt.md",
  "editor": "code --wait",
  "claude": { "model": "sonnet", "models": ["opus", "sonnet", "haiku"] },
  "codex": { "model": "gpt-6.1-sol", "models": [] }
}
```

| Field        | Meaning                                                                                       |
| ------------ | --------------------------------------------------------------------------------------------- |
| `agent`      | `claude` or `codex`; preselected in the picker, and what `--yes` uses when both are installed |
| `prompt`     | The prompt template, inline                                                                   |
| `promptFile` | The prompt template in a file (relative to the config file; `~` allowed). Wins over `prompt`  |
| `editor`     | Command for amending the prompt, e.g. `code --wait`. Default: `$VISUAL`, then `$EDITOR`       |
| `autoStage`  | `git add` resolved files afterwards. Default `true`                                           |
| `claude`, `codex` | `model` (preselected) and `models` (offered in the picker)                               |

### Models

Claude Code has no command that lists models, so its list is `claude.models` from your config
(default `opus`, `sonnet`, `haiku`, with `sonnet` preselected; full model ids work too). Codex's list
is discovered live from `codex debug models`; anything you list under `codex.models` is added, and
it is the fallback if discovery fails. You can always pass `--model` with any name.

### The prompt

The default prompt is in [`examples/config/prompt.md`](examples/config/prompt.md) and is what
`config init` writes to `prompt.md`. Edit that file to change what every run sends, or amend a single
run at the review step. Placeholders:

| Placeholder        | Filled with                                                                    |
| ------------------ | ------------------------------------------------------------------------------ |
| `{{files}}`        | A bullet list of the conflicted files and their conflict kinds                 |
| `{{operation}}`    | `merge`, `rebase`, `cherry-pick`, `revert`, or `stash pop or similar`          |
| `{{ours}}`, `{{theirs}}` | Plain-language names for the two sides in this operation                 |
| `{{branch}}`       | The current branch                                                             |
| `{{summary_format}}` | The request for a per-file JSON summary that powers the report                |

An unknown placeholder is an error, never sent to the agent. If your prompt omits
`{{summary_format}}` the run still works; the report just shows the agent's own words instead of a
per-file explanation. `config doctor` warns about that.

## The report

```
Resolution report
claude (sonnet) · merge · 2 of 2 files resolved

  ✓ src/app.rs   both modified  merged both  +6 −2  staged
      kept the new retry logic and the renamed config field
  ✓ Cargo.lock   both modified  took theirs  +40 −38  staged
      regenerated lockfile content from the incoming side

Next
  Review what was resolved:  git diff --staged
  When you are happy:        git merge --continue
Nothing was committed or pushed; that is left to you.
```

If anything looks off it appears under **Look closer at**: the agent claiming a file was handled when
markers remain, claiming one strategy when the content looks like another, skipping a file in its
summary, mentioning a file that was never in conflict, editing files outside the conflict, or moving
`HEAD`. `--json` prints the same data, plus the mismatches, for scripts.

## Keeping it up to date

```sh
resolve-merge-conflicts upgrade            # install the newest release and switch to it
resolve-merge-conflicts upgrade --check    # just report whether one exists
resolve-merge-conflicts upgrade 0.2.0      # install a specific version
resolve-merge-conflicts upgrade --keep 3   # how many versions to leave on disk (default 2)
resolve-merge-conflicts uninstall          # show what would be removed (add --yes to do it)
```

Several versions can sit side by side and you switch between them like `nvm use`:

| Command                                          | What it does                                                        |
| ------------------------------------------------ | ------------------------------------------------------------------- |
| `resolve-merge-conflicts versions`               | List installed versions; `*` is active, `running` is the one executing |
| `resolve-merge-conflicts use <version>`          | Switch to a version, downloading and verifying it first if needed   |
| `resolve-merge-conflicts versions remove <v>...` | Delete versions (never the active or running one)                   |
| `resolve-merge-conflicts versions prune [--keep N]` | Delete all but the newest N                                      |

No shell restart is needed: `~/.local/bin/resolve-merge-conflicts` is a symlink through `current`, and
`upgrade` and `use` only move `current`. Your config is never touched.

At most once a day an interactive run starts by asking `resolve-merge-conflicts v0.3.0 is available
(you have v0.2.0). Update now? (Y/n)`; yes installs it and continues your run on the new version.
Turn it off with `RESOLVE_MERGE_CONFLICTS_NO_UPDATE_CHECK=1` (also off when `CI` is set, and for
binaries not installed by the installer).

Shell completion:

```sh
resolve-merge-conflicts completion zsh > ~/.zfunc/_resolve-merge-conflicts   # bash, zsh, fish, elvish, powershell
```

## Development

```sh
cargo build
cargo test             # no network and no real agent; fixtures build temp git repos with real conflicts
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo build --release
scripts/smoke.sh target/release/resolve-merge-conflicts 0.1.0   # end-to-end with a fake agent
cargo test -- --ignored   # also checks Codex model discovery against a real `codex`
```

Tests that need a conflicted repository use `tests/common/mod.rs`; agents are replaced by tiny shell
scripts, so nothing in the suite calls a model. Work is tracked with [motte](https://github.com/CodeVachon/motte)
in `.motte/issues/`. Release builds are produced by `.github/workflows/release.yml` on a `v*` tag,
verified with the installers against the freshly built assets, and only then published.

## License

MIT. See [LICENSE](LICENSE).
