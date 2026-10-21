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

## Where to start

- [Install](install.md) it, then run it in a repository that is stopped on conflicts: [Usage](usage.md).
- [What it does, and what it will not do](safety.md) explains why it is safe to point at your work.
- [Configuration](configuration.md) covers the config file and changing the prompt.
