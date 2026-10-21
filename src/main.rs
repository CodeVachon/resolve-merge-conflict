//! Entry point for resolve-merge-conflicts.

use clap::Parser;
use resolve_merge_conflicts::cli::{self, Cli, Command};
use resolve_merge_conflicts::ui;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        None => cli::run_interactive(&cli.run),
        Some(Command::Completion { shell }) => {
            cli::print_completion(shell);
            Ok(0)
        }
        Some(Command::Config { command }) => match command {
            cli::ConfigCommand::Path { config } => {
                cli::config_cmd::run_path(config.as_deref()).map(|()| 0)
            }
            cli::ConfigCommand::Doctor { config, json } => {
                cli::config_cmd::run_doctor(config.as_deref(), json)
            }
            cli::ConfigCommand::Init { force, config } => {
                cli::config_cmd::run_init(config.as_deref(), force).map(|()| 0)
            }
        },
    };

    match result {
        Ok(0) => {}
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{}", ui::error_block(&format!("{error:#}")));
            std::process::exit(1);
        }
    }
}
