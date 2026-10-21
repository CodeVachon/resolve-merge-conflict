# Install

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
