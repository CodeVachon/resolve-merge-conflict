# Agent instructions

<!-- motte:start -->

## Tracking work with motte

This project's work lives in `.motte/issues/` as Markdown files, committed alongside the code. Track
work there rather than in an ad-hoc TODO list, a commit message, or a pull request description.

Start by asking what to do:

```
motte next --why
```

That is not `motte list`, and not quite `motte ready` either. **Ready** means unsettled with every
blocker settled; `next` orders that set by what a piece of work would unblock, how close it is to a leaf,
and how long it has waited — and it leaves out anything somebody else holds. `motte ready --blocked` shows
what is waiting and on what.

The loop for an issue you pick up:

1. Claim it — `motte claim <ref>`. If that fails, somebody else is on it: ask `motte next` again.
2. Read it — `motte show <ref>`
3. Refine its **Plan** if the plan on the file is not what you are actually going to do
4. Add notes as you go, especially for decisions and dead ends — `motte note <ref> "..."`
5. Move it to Done when the verification for that issue passes, or `motte release <ref>` if you stop

A `<ref>` is an issue number or a fragment of the title, so `motte show parser` works as well as
`motte show 12`.

If you discover a prerequisite mid-task, record it with `motte block <ref> <blocker>` rather than
describing it in prose. Prose is not queryable, and `motte ready` is what the next agent reads.

Every read command accepts `--json`. The MCP server exposes the same operations, and notes written
through it are attributed to the agent rather than to the repository's git user — which is the point:
one shared record of who decided what.

<!-- motte:end -->

## Building and testing

```
cargo build
cargo test             # no network, no real agent; fixtures build temp git repos
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

Tests that need a repository in a conflicted state use `tests/common/mod.rs`:
`Fixture::conflicted(Operation::Merge | Rebase | CherryPick | Revert | StashPop)` and
`Fixture::delete_modify_conflict()`. Fixtures isolate git from your global config, hooks and
signing, so they behave the same on every machine. Never point a test at a real agent binary; use a
fake one on `PATH`.

The tool must never commit, push, or run `--continue`/`--abort`. Tests that exercise it should
assert `HEAD` is unchanged.
