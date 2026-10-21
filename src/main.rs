//! Entry point for resolve-merge-conflicts.

use clap::Parser;
use resolve_merge_conflicts::cli::{self, Cli, Command};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        None => Err(anyhow::anyhow!("the interactive flow is not wired up yet")),
        Some(Command::Completion { shell }) => {
            cli::print_completion(shell);
            Ok(0)
        }
        Some(Command::Config { .. }) => {
            Err(anyhow::anyhow!("config commands are not wired up yet"))
        }
    };

    match result {
        Ok(0) => {}
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("Error: {error:#}");
            std::process::exit(1);
        }
    }
}
