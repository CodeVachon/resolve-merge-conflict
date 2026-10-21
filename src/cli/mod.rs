//! The command line: flags, subcommands, and the pieces that run them.

pub mod config_cmd;
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
    /// Update resolve-merge-conflicts in place, or check whether an update is available
    Upgrade(UpgradeArgs),
    /// List installed versions; `remove` and `prune` delete old ones
    Versions(VersionsArgs),
    /// Switch to an installed version, downloading it first if needed
    Use(UseArgs),
    /// Remove the managed installation from this machine
    Uninstall(UninstallArgs),
    /// Print a shell completion script
    Completion {
        /// bash, zsh, fish, elvish or powershell
        shell: Shell,
    },
}

#[derive(Debug, Clone, Default, Args)]
pub struct UpgradeArgs {
    /// Install this version instead of the newest
    pub target: Option<String>,

    /// Report whether an update is available, changing nothing
    #[arg(long)]
    pub check: bool,

    /// How many versions to keep on disk
    #[arg(long, default_value_t = 2, value_name = "N")]
    pub keep: usize,

    /// Reinstall even if already on the target version
    #[arg(long)]
    pub force: bool,

    /// Machine-readable output
    #[arg(long)]
    pub json: bool,
}

/// `versions`: the list by default, or one of its subcommands.
#[derive(Debug, Clone, Default, Args)]
pub struct VersionsArgs {
    #[command(subcommand)]
    pub command: Option<VersionsCommand>,

    /// Ask GitHub for the newest release instead of using the last check's result
    #[arg(long)]
    pub check: bool,

    /// Machine-readable output
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum VersionsCommand {
    /// Delete installed versions (never the active one or the one running)
    Remove {
        /// Versions to delete, e.g. 0.1.0 or v0.1.0
        #[arg(required = true, value_name = "VERSION")]
        versions: Vec<String>,
        /// Machine-readable output
        #[arg(long)]
        json: bool,
    },
    /// Delete all but the newest N versions (never the active one or the one running)
    Prune {
        /// How many versions to keep on disk
        #[arg(long, default_value_t = 2, value_name = "N")]
        keep: usize,
        /// Machine-readable output
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Default, Args)]
pub struct UseArgs {
    /// Version to switch to, e.g. 0.2.0 or v0.2.0
    #[arg(value_name = "VERSION")]
    pub version: String,

    /// Machine-readable output
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Default, Args)]
pub struct UninstallArgs {
    /// Skip the confirmation
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Machine-readable output
    #[arg(long)]
    pub json: bool,
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
