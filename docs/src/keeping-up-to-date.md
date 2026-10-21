# Keeping it up to date

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
