//! The command reference in `docs/src/help/` is the real `--help` output, kept in sync by this
//! test. After changing a flag or its description, regenerate with:
//!
//! ```sh
//! UPDATE_DOCS=1 cargo test --test docs
//! ```

use std::path::PathBuf;

use clap::CommandFactory;
use resolve_merge_conflicts::cli::Cli;

fn help_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/src/help")
}

/// Every command in the tree as `(file stem, rendered long help)`.
fn pages() -> Vec<(String, String)> {
    fn walk(command: &clap::Command, stem: &str, out: &mut Vec<(String, String)>) {
        let mut command = command
            .clone()
            .term_width(80)
            .color(clap::ColorChoice::Never);
        out.push((stem.to_string(), command.render_long_help().to_string()));
        let subs: Vec<clap::Command> = command.get_subcommands().cloned().collect();
        for sub in subs {
            if sub.get_name() == "help" {
                continue;
            }
            walk(&sub, &format!("{stem}_{}", sub.get_name()), out);
        }
    }
    let mut out = Vec::new();
    walk(&Cli::command(), "root", &mut out);
    for (stem, _) in &mut out {
        *stem = stem.strip_prefix("root_").unwrap_or(stem).to_string();
    }
    out
}

#[test]
fn the_reference_pages_match_the_binary() {
    let update = std::env::var_os("UPDATE_DOCS").is_some();
    if update {
        std::fs::create_dir_all(help_dir()).unwrap();
    }
    let mut stale = Vec::new();
    for (stem, help) in pages() {
        let path = help_dir().join(format!("{stem}.txt"));
        if update {
            std::fs::write(&path, &help).unwrap();
        } else if std::fs::read_to_string(&path).ok().as_deref() != Some(help.as_str()) {
            stale.push(path.display().to_string());
        }
    }
    assert!(
        stale.is_empty(),
        "out of date (run `UPDATE_DOCS=1 cargo test --test docs`): {stale:#?}"
    );
}

#[test]
fn every_help_file_is_included_by_the_reference_page() {
    let reference = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/src/reference.md"),
    )
    .unwrap();
    for (stem, _) in pages() {
        assert!(
            reference.contains(&format!("{{{{#include help/{stem}.txt}}}}")),
            "reference.md does not include help/{stem}.txt"
        );
    }
}
