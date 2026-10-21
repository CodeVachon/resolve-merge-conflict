# Usage

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

## Exit codes

| Code | Meaning                                                                                       |
| ---- | --------------------------------------------------------------------------------------------- |
| 0    | Every conflicted file is resolved, or there was nothing to do, or you chose Cancel / `--dry-run` |
| 1    | Some files are still conflicted, `HEAD` moved while the agent ran, or an error                 |
| 2    | Unknown flag or bad usage                                                                      |
| 130  | You pressed Esc or Ctrl-C at a question                                                        |
