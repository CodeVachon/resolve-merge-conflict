//! Entry point for resolve-merge-conflicts.

use clap::Parser;

/// Resolve git merge conflicts with a CLI agent. Never commits or pushes.
#[derive(Parser)]
#[command(name = "resolve-merge-conflicts", version = resolve_merge_conflicts::VERSION)]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
}
