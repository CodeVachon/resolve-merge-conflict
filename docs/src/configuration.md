# Config

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

## Models

Claude Code has no command that lists models, so its list is `claude.models` from your config
(default `opus`, `sonnet`, `haiku`, with `sonnet` preselected; full model ids work too). Codex's list
is discovered live from `codex debug models`; anything you list under `codex.models` is added, and
it is the fallback if discovery fails. You can always pass `--model` with any name.

## The prompt

The default prompt is in [`examples/config/prompt.md`](https://github.com/CodeVachon/resolve-merge-conflict/blob/main/examples/config/prompt.md) and is what
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
