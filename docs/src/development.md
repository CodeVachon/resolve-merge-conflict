# Development

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
