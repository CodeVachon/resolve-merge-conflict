//! The command line: flags, subcommands, and the pieces that run them.

pub mod interactive;
pub mod prompter;

use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

use crate::runner::Options;

pub use interactive::run_interactive;

/// Resolve git merge conflicts with a CLI agent (Claude Code or Codex).
///
/// Run it in a repository that is stopped on conflicts. It asks which agent and model to use,
/// shows you the prompt so you can amend it, lets the agent edit the conflicted files, then stages
/// what was resolved and reports what happened. It never commits or pushes.
#[derive(Parser, Debug)]
#[command(
    name = "resolve-merge-conflicts",
    version = crate::VERSION,
    about,
    long_about,
    arg_required_else_help = false
)]
pub struct Cli {
    #[command(flatten)]
    pub run: RunArgs,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Flags for the default (interactive) run. Every question has a flag, so a run can be scripted.
#[derive(Args, Debug, Clone, Default)]
pub struct RunArgs {
    /// Working directory of the repository [default: current]
    #[arg(short = 'c', long, value_name = "DIR")]
    pub cwd: Option<PathBuf>,

    /// Agent to use: claude or codex
    #[arg(
        short = 'a',
        long,
        value_name = "AGENT",
        env = "RESOLVE_MERGE_CONFLICTS_AGENT"
    )]
    pub agent: Option<String>,

    /// Model to use, as the agent names it
    #[arg(
        short = 'm',
        long,
        value_name = "MODEL",
        env = "RESOLVE_MERGE_CONFLICTS_MODEL"
    )]
    pub model: Option<String>,

    /// Answer every question with its default and send the prompt without review
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Show the prompt but do not offer to edit it
    #[arg(long)]
    pub no_edit: bool,

    /// Build and show the prompt, then stop without running the agent
    #[arg(long)]
    pub dry_run: bool,

    /// Use the prompt template in this file instead of the configured one
    #[arg(long, value_name = "FILE")]
    pub prompt_file: Option<PathBuf>,

    /// Leave resolved files unstaged (by default they are `git add`ed)
    #[arg(long)]
    pub no_stage: bool,

    /// Config file to use instead of the one in your home directory
    #[arg(short = 'f', long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Print the final report as JSON instead of text
    #[arg(long)]
    pub json: bool,
}

impl RunArgs {
    /// The runner options these flags describe. `--prompt-file` is read here so a bad path fails
    /// before anything is asked.
    pub fn to_options(&self) -> anyhow::Result<Options> {
        use anyhow::Context;
        let prompt_template = self
            .prompt_file
            .as_ref()
            .map(|path| {
                std::fs::read_to_string(path)
                    .with_context(|| format!("could not read --prompt-file {}", path.display()))
            })
            .transpose()?;
        Ok(Options {
            cwd: match &self.cwd {
                Some(cwd) => cwd.clone(),
                None => std::env::current_dir().context("could not read the current directory")?,
            },
            agent: self.agent.clone(),
            model: self.model.clone(),
            yes: self.yes,
            no_edit: self.no_edit,
            dry_run: self.dry_run,
            prompt_template,
            stage: self.no_stage.then_some(false),
            config_dir: None,
        })
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Inspect and manage the config file
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Print a shell completion script
    Completion {
        /// bash, zsh, fish, elvish or powershell
        shell: Shell,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Print the config file in effect and the rule that chose it
    Path {
        /// Config file to use instead of the default location
        #[arg(short = 'f', long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
    /// Validate the config file; exit 1 if a run would fail
    Doctor {
        #[arg(short = 'f', long, value_name = "FILE")]
        config: Option<PathBuf>,
        /// Machine-readable output
        #[arg(long)]
        json: bool,
    },
    /// Write a starter config file (existing files are kept unless --force)
    Init {
        /// Overwrite an existing file
        #[arg(long)]
        force: bool,
        #[arg(short = 'f', long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
}

/// Write the completion script for `shell` to stdout.
pub fn print_completion(shell: Shell) {
    let mut command = Cli::command();
    clap_complete::generate(
        shell,
        &mut command,
        "resolve-merge-conflicts",
        &mut std::io::stdout(),
    );
}
