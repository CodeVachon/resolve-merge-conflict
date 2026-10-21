# What it does, and what it will not do

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
