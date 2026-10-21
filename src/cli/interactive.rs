//! The default run: everything from loading config to printing the report.

use anyhow::{Result, bail};

use super::RunArgs;
use super::prompter::{self, Cancelled, InquirePrompter};
use crate::config;
use crate::render;
use crate::report::Report;
use crate::runner::{self, Deps, Outcome};
use crate::ui;

/// Run the tool for `args`; returns the process exit code.
pub fn run_interactive(args: &RunArgs) -> Result<i32> {
    match run(args) {
        Err(error) if error.downcast_ref::<Cancelled>().is_some() => {
            eprintln!("Cancelled.");
            Ok(130)
        }
        other => other,
    }
}

fn run(args: &RunArgs) -> Result<i32> {
    let mut options = args.to_options()?;
    let loaded = config::load(args.config.as_deref())?;
    options.config_dir = loaded
        .path
        .as_ref()
        .and_then(|path| path.parent())
        .map(|dir| dir.to_path_buf());

    if !options.yes && !prompter::is_interactive() {
        bail!(
            "this needs a terminal to ask questions; pass --yes (with --agent and --model) to run unattended"
        );
    }

    // With --json, stdout carries only the report; everything else goes to stderr.
    let quiet = args.json;
    let say = |text: &str| {
        if quiet {
            eprintln!("{text}");
        } else {
            println!("{text}");
        }
    };
    say(&ui::title());
    for warning in &loaded.warnings {
        eprintln!("{}", ui::yellow(&format!("warning: {warning}")));
    }

    let mut prompter = InquirePrompter::new(loaded.config.editor.clone(), quiet);
    let outcome = runner::run(&options, &loaded.config, &mut prompter, &Deps::real())?;

    match outcome {
        Outcome::NothingToDo => {
            if args.json {
                println!("{{\"status\": \"nothing_to_do\"}}");
            } else {
                println!("No merge conflicts found. Nothing to do.");
            }
            Ok(0)
        }
        Outcome::Cancelled => {
            say("Cancelled. Nothing was changed.");
            Ok(0)
        }
        Outcome::DryRun(plan) => {
            if args.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "dry_run",
                        "agent": plan.agent,
                        "model": plan.model,
                        "prompt": plan.prompt,
                    })
                );
            } else {
                println!(
                    "{}",
                    ui::dim(&format!(
                        "Dry run: would run {} ({}). Nothing was changed.",
                        plan.agent, plan.model
                    ))
                );
            }
            Ok(0)
        }
        Outcome::Finished(finished) => {
            let report = Report::build(&finished);
            if args.json {
                println!("{}", render::json(&report));
            } else {
                println!("\n{}", render::text(&report));
            }
            let ok = report.all_resolved() && !report.head_moved;
            Ok(if ok { 0 } else { 1 })
        }
    }
}
