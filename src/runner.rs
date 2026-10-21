//! The whole run, start to finish, with every question put through a [`Prompter`] and every
//! outside dependency (agents, model lists) injected, so it can be driven by a script in tests.
//!
//! Order of events: detect the conflicted state, choose agent and model, render the prompt and let
//! the user amend it, snapshot, run the agent, verify and stage. The runner never commits, pushes,
//! continues or aborts anything.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::agent::{self, Agent, AgentOutcome, AgentRequest};
use crate::config::Config;
use crate::conflicts::{self, ConflictState};
use crate::git::Git;
use crate::models::{self, ModelOptions};
use crate::prompt::{self, PromptContext};
use crate::verify::{self, Snapshot, Verification};

/// Everything the runner asks of a person.
pub trait Prompter {
    /// Pick one of `options`; `default` is the preselected index. Returns the chosen index.
    fn select(
        &mut self,
        question: &str,
        options: &[String],
        default: Option<usize>,
    ) -> Result<usize>;
    /// Free text, with an optional preset answer.
    fn text(&mut self, question: &str, default: Option<&str>) -> Result<String>;
    /// Let the user amend `initial` and return the result.
    fn edit(&mut self, question: &str, initial: &str) -> Result<String>;
    /// Show a block of text (the prompt) before asking about it.
    fn show(&mut self, title: &str, body: &str);
    /// A status or warning line.
    fn info(&mut self, message: &str);
    /// Show that something slow is running until the returned guard is dropped.
    fn working(&mut self, message: &str) -> Box<dyn std::any::Any> {
        self.info(message);
        Box::new(())
    }
}

/// The outside world, replaceable in tests.
pub struct Deps<'a> {
    pub make_agent: &'a dyn Fn(&str) -> Result<Box<dyn Agent>>,
    pub installed: &'a dyn Fn(&str) -> bool,
    pub model_options: &'a dyn Fn(&str, &Config) -> ModelOptions,
}

pub fn real_agent(name: &str) -> Result<Box<dyn Agent>> {
    match name {
        "claude" => Ok(Box::new(agent::claude::ClaudeAgent::new())),
        "codex" => Ok(Box::new(agent::codex::CodexAgent::new())),
        other => bail!(
            "unknown agent \"{other}\" (known: {})",
            agent::names().join(", ")
        ),
    }
}

impl Deps<'static> {
    /// Real agents, real `PATH` lookup, real model discovery.
    pub fn real() -> Self {
        Deps {
            make_agent: &real_agent,
            installed: &|name| {
                real_agent(name)
                    .map(|agent| agent::program_available(agent.program()))
                    .unwrap_or(false)
            },
            model_options: &models::options,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    /// The repository to work in.
    pub cwd: PathBuf,
    pub agent: Option<String>,
    pub model: Option<String>,
    /// Answer every question with its default and skip the prompt review.
    pub yes: bool,
    /// Show the prompt but do not offer to edit it.
    pub no_edit: bool,
    /// Build and show the prompt, then stop without running the agent.
    pub dry_run: bool,
    /// Use this text as the prompt template instead of the configured one.
    pub prompt_template: Option<String>,
    /// Overrides the config's `autoStage`.
    pub stage: Option<bool>,
    /// Where relative `promptFile` paths resolve from (the config file's directory).
    pub config_dir: Option<PathBuf>,
}

#[derive(Debug)]
pub enum Outcome {
    /// No conflicts anywhere; nothing was run.
    NothingToDo,
    /// The user declined at the prompt review.
    Cancelled,
    /// `--dry-run`: the prompt that would have been sent.
    DryRun(Plan),
    Finished(Box<Finished>),
}

/// What was decided before the agent ran.
#[derive(Debug, Clone)]
pub struct Plan {
    pub state: ConflictState,
    pub agent: String,
    pub model: String,
    pub prompt: String,
    /// Non-conflicted files that already had changes when the run started.
    pub preexisting_changes: Vec<String>,
}

#[derive(Debug)]
pub struct Finished {
    pub plan: Plan,
    pub outcome: AgentOutcome,
    pub verification: Verification,
    pub staged: bool,
}

pub fn run(
    options: &Options,
    config: &Config,
    prompter: &mut dyn Prompter,
    deps: &Deps<'_>,
) -> Result<Outcome> {
    let mut git = Git::new(&options.cwd);
    let Some(state) = conflicts::detect(&mut git)? else {
        return Ok(Outcome::NothingToDo);
    };
    let root = git.top_level()?;
    let mut git = Git::new(&root);

    prompter.info(&format!(
        "{} conflicted file{} in a {}:",
        state.files.len(),
        if state.files.len() == 1 { "" } else { "s" },
        state.operation.describe()
    ));
    for file in &state.files {
        prompter.info(&format!("  {} ({})", file.path, file.kind.describe()));
    }

    let preexisting_changes = git.other_changes()?;
    if !preexisting_changes.is_empty() {
        prompter.info(&format!(
            "Note: {} other file{} already changed; the agent is told not to touch them, and any change it makes there is reported.",
            preexisting_changes.len(),
            if preexisting_changes.len() == 1 { " is" } else { "s are" }
        ));
    }

    let agent_name = choose_agent(options, config, prompter, deps)?;
    let agent = (deps.make_agent)(&agent_name)?;
    let model = choose_model(options, config, &agent_name, prompter, deps)?;

    let template = match &options.prompt_template {
        Some(template) => template.clone(),
        None => prompt::template(config, options.config_dir.as_deref())?,
    };
    let branch = git.current_branch();
    let mut prompt_text = prompt::render(
        &template,
        &PromptContext {
            state: &state,
            branch: branch.as_deref(),
        },
    )?;

    if !options.yes {
        loop {
            prompter.show("Prompt", &prompt_text);
            if options.no_edit {
                break;
            }
            let choices = ["Run with this prompt", "Edit the prompt", "Cancel"].map(String::from);
            match prompter.select("What now?", &choices, Some(0))? {
                0 => break,
                1 => prompt_text = prompter.edit("Amend the prompt", &prompt_text)?,
                _ => return Ok(Outcome::Cancelled),
            }
        }
    }

    let plan = Plan {
        state: state.clone(),
        agent: agent_name,
        model,
        prompt: prompt_text,
        preexisting_changes,
    };
    if options.dry_run {
        return Ok(Outcome::DryRun(plan));
    }

    let snapshot = Snapshot::take(&mut git)?;
    let busy = prompter.working(&format!("Running {} ({})", plan.agent, plan.model));
    let outcome = agent::run(
        agent.as_ref(),
        &AgentRequest {
            prompt: plan.prompt.clone(),
            model: plan.model.clone(),
            cwd: root.clone(),
        },
    )
    .with_context(|| format!("{} did not run", plan.agent));
    drop(busy);
    let outcome = outcome?;

    let stage = options.stage.unwrap_or_else(|| config.auto_stage());
    let verification = verify::verify_and_stage(&mut git, &state, &snapshot, stage)?;
    Ok(Outcome::Finished(Box::new(Finished {
        plan,
        outcome,
        verification,
        staged: stage,
    })))
}

fn choose_agent(
    options: &Options,
    config: &Config,
    prompter: &mut dyn Prompter,
    deps: &Deps<'_>,
) -> Result<String> {
    if let Some(name) = &options.agent {
        if !agent::names().contains(&name.as_str()) {
            bail!(
                "unknown agent \"{name}\" (known: {})",
                agent::names().join(", ")
            );
        }
        if !(deps.installed)(name) {
            bail!("agent \"{name}\" is not installed (not found on PATH)");
        }
        return Ok(name.clone());
    }

    let available: Vec<String> = agent::names()
        .iter()
        .filter(|name| (deps.installed)(name))
        .map(|name| name.to_string())
        .collect();
    let preferred = config
        .agent
        .as_ref()
        .and_then(|agent| available.iter().position(|name| name == agent));
    match available.len() {
        0 => bail!(
            "no agent found: install Claude Code (`claude`) or Codex (`codex`) and make sure it is on PATH"
        ),
        1 => Ok(available[0].clone()),
        _ if options.yes => Ok(available[preferred.unwrap_or(0)].clone()),
        _ => {
            let index = prompter.select("Which agent?", &available, preferred)?;
            Ok(available[index].clone())
        }
    }
}

fn choose_model(
    options: &Options,
    config: &Config,
    agent: &str,
    prompter: &mut dyn Prompter,
    deps: &Deps<'_>,
) -> Result<String> {
    if let Some(model) = &options.model {
        return Ok(model.clone());
    }
    let available = (deps.model_options)(agent, config);
    if let Some(note) = &available.note {
        prompter.info(note);
    }
    let default_index = available
        .default
        .as_ref()
        .and_then(|default| available.models.iter().position(|m| m == default));

    if available.models.is_empty() {
        if options.yes {
            bail!(
                "no models known for {agent}: pass --model or list some under \"{agent}.models\" in the config"
            );
        }
        let model = prompter.text(
            &format!("Which {agent} model?"),
            available.default.as_deref(),
        )?;
        if model.trim().is_empty() {
            bail!("a model is required");
        }
        return Ok(model.trim().to_string());
    }
    if options.yes {
        return Ok(available.models[default_index.unwrap_or(0)].clone());
    }
    let index = prompter.select(
        &format!("Which {agent} model?"),
        &available.models,
        default_index,
    )?;
    Ok(available.models[index].clone())
}
