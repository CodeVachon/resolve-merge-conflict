This repository is in the middle of a git {{operation}} that stopped on conflicts. Resolve them.

In this {{operation}}, "ours" is {{ours}} and "theirs" is {{theirs}}.

Conflicted files:
{{files}}

How to work:
- Open each conflicted file, understand what each side was trying to do, and write the correct merged result. Keep the intent of both sides; do not just pick one unless one side is clearly superseded.
- Remove every conflict marker (<<<<<<<, =======, >>>>>>>, and ||||||| if present). The result must be valid, coherent code.
- Where a file was deleted on one side and modified on the other, decide whether it should exist and either keep your merged version or delete the file.
- If the conflict touches code that other files depend on (imports, call sites, types, tests), read enough of the surrounding code to make the result consistent.
- Only edit the conflicted files listed above. Do not touch any other file.
- Do NOT run git add, git commit, git push, git merge, git rebase, git checkout, git reset, git stash or any other command that changes git state, and do not continue or abort the {{operation}}. Staging and committing are left to the user. Read-only commands such as git diff, git show and git log are fine.

{{summary_format}}
