//! `config path`, `config init` and `config doctor`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::agent;
use crate::config::{self, Config, Location};
use crate::prompt;
use crate::ui;

pub const SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/CodeVachon/resolve-merge-conflict/main/schema/config.json";
pub const PROMPT_FILE: &str = "prompt.md";

fn location(explicit: Option<&Path>) -> Result<Location> {
    config::locate(explicit)
        .context("could not work out where the config file lives (no home directory?)")
}

/// `config path`: the file in effect and why.
pub fn run_path(explicit: Option<&Path>) -> Result<()> {
    let location = location(explicit)?;
    println!("{}", location.path.display());
    println!(
        "{}",
        ui::dim(&format!(
            "chosen by {}{}",
            location.rule.describe(),
            if location.path.exists() {
                ""
            } else {
                " (does not exist yet; run `config init`)"
            }
        ))
    );
    Ok(())
}

/// The starter config. Models are listed so the picker has something to show; the prompt lives in
/// its own file so it can be edited as prose rather than as an escaped JSON string.
pub fn starter_config() -> String {
    format!(
        r#"{{
  "$schema": "{SCHEMA_URL}",
  "agent": "claude",
  "autoStage": true,
  "promptFile": "{PROMPT_FILE}",
  "claude": {{
    "model": "{}",
    "models": [{}]
  }},
  "codex": {{
    "models": []
  }}
}}
"#,
        config::DEFAULT_CLAUDE_MODEL,
        config::DEFAULT_CLAUDE_MODELS
            .iter()
            .map(|model| format!("\"{model}\""))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Created,
    Kept,
    Overwritten,
}

fn write_file(path: &Path, contents: &str, force: bool) -> Result<Outcome> {
    let existed = path.exists();
    if existed && !force {
        return Ok(Outcome::Kept);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("could not create {}", parent.display()))?;
    }
    std::fs::write(path, contents)
        .with_context(|| format!("could not write {}", path.display()))?;
    Ok(if existed {
        Outcome::Overwritten
    } else {
        Outcome::Created
    })
}

/// The files `config init` manages, with what happened to each.
pub fn init(explicit: Option<&Path>, force: bool) -> Result<Vec<(PathBuf, Outcome)>> {
    let location = location(explicit)?;
    let mut results = vec![(
        location.path.clone(),
        write_file(&location.path, &starter_config(), force)?,
    )];
    if let Some(dir) = location.path.parent() {
        let prompt_path = dir.join(PROMPT_FILE);
        results.push((
            prompt_path.clone(),
            write_file(
                &prompt_path,
                &format!("{}\n", prompt::DEFAULT_TEMPLATE),
                force,
            )?,
        ));
    }
    Ok(results)
}

pub fn run_init(explicit: Option<&Path>, force: bool) -> Result<()> {
    for (path, outcome) in init(explicit, force)? {
        let word = match outcome {
            Outcome::Created => ui::green("created"),
            Outcome::Kept => ui::dim("kept   "),
            Outcome::Overwritten => ui::yellow("replaced"),
        };
        println!("{word} {}", path.display());
    }
    println!(
        "{}",
        ui::dim(
            "Edit prompt.md to change the prompt sent to the agent; `config doctor` checks it."
        )
    );
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Problem {
    pub level: Level,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentStatus {
    pub name: String,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Doctor {
    pub path: Option<PathBuf>,
    pub rule: &'static str,
    pub agents: Vec<AgentStatus>,
    pub problems: Vec<Problem>,
    pub ok: bool,
}

/// Check the config a run would use. `installed` answers whether an agent is available.
pub fn diagnose(explicit: Option<&Path>, installed: &dyn Fn(&str) -> bool) -> Result<Doctor> {
    let location = location(explicit)?;
    let mut problems = Vec::new();
    let mut error = |message: String| {
        problems.push(Problem {
            level: Level::Error,
            message,
        })
    };

    let (config, path) = match config::load_from(&location) {
        Ok(loaded) => {
            let path = loaded.path.clone();
            let config = loaded.config;
            for warning in loaded.warnings {
                problems.push(Problem {
                    level: Level::Warning,
                    message: warning,
                });
            }
            (config, path)
        }
        Err(failure) => {
            error(format!("{failure:#}"));
            (Config::default(), None)
        }
    };
    let mut error = |message: String| {
        problems.push(Problem {
            level: Level::Error,
            message,
        })
    };
    if let Some(agent) = &config.agent
        && !agent::names().contains(&agent.as_str())
    {
        error(format!(
            "\"agent\" is \"{agent}\"; it must be one of: {}",
            agent::names().join(", ")
        ));
    }

    let base = location.path.parent();
    match prompt::template(&config, base) {
        Ok(template) => {
            let unknown = prompt::unknown_placeholders(&template);
            if !unknown.is_empty() {
                error(format!(
                    "the prompt uses unknown placeholder{}: {} (known: {})",
                    if unknown.len() == 1 { "" } else { "s" },
                    unknown
                        .iter()
                        .map(|name| format!("{{{{{name}}}}}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    prompt::PLACEHOLDERS
                        .iter()
                        .map(|name| format!("{{{{{name}}}}}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            if !template.contains("{{files}}") {
                problems.push(Problem {
                    level: Level::Warning,
                    message: "the prompt does not use {{files}}, so the agent is not told which files are conflicted".into(),
                });
            }
            if !template.contains("{{summary_format}}") {
                problems.push(Problem {
                    level: Level::Warning,
                    message: "the prompt does not use {{summary_format}}, so the report will not have the agent's per-file explanation".into(),
                });
            }
        }
        Err(failure) => {
            problems.push(Problem {
                level: Level::Error,
                message: format!("{failure:#}"),
            });
        }
    }
    if config.prompt.is_some() && config.prompt_file.is_some() {
        problems.push(Problem {
            level: Level::Warning,
            message: "both \"prompt\" and \"promptFile\" are set; promptFile wins".into(),
        });
    }

    for name in agent::names() {
        if let Some(settings) = config.agent_config(name)
            && let Some(model) = &settings.model
            && !settings.models.is_empty()
            && !settings.models.contains(model)
        {
            problems.push(Problem {
                level: Level::Warning,
                message: format!("{name}.model \"{model}\" is not in {name}.models"),
            });
        }
    }

    let agents: Vec<AgentStatus> = agent::names()
        .iter()
        .map(|name| AgentStatus {
            name: name.to_string(),
            installed: installed(name),
        })
        .collect();
    if agents.iter().all(|agent| !agent.installed) {
        problems.push(Problem {
            level: Level::Error,
            message: "neither `claude` nor `codex` was found on PATH".into(),
        });
    }

    if let Some(editor) = &config.editor
        && let Some((program, _)) = crate::cli::prompter::split_editor(editor)
        && !agent::program_available(Path::new(&program))
    {
        problems.push(Problem {
            level: Level::Warning,
            message: format!("editor \"{editor}\" was not found on PATH"),
        });
    }

    let ok = problems.iter().all(|problem| problem.level != Level::Error);
    Ok(Doctor {
        path,
        rule: location.rule.describe(),
        agents,
        problems,
        ok,
    })
}

/// `config doctor`; returns the exit code.
pub fn run_doctor(explicit: Option<&Path>, json: bool) -> Result<i32> {
    let report = diagnose(explicit, &|name| {
        crate::runner::real_agent(name)
            .map(|agent| agent::program_available(agent.program()))
            .unwrap_or(false)
    })?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        match &report.path {
            Some(path) => println!("{} {}", ui::bold("Config:"), path.display()),
            None => println!(
                "{} none found (defaults apply); `config init` writes one",
                ui::bold("Config:")
            ),
        }
        for agent in &report.agents {
            let mark = if agent.installed {
                ui::green("✓")
            } else {
                ui::dim("–")
            };
            println!(
                "  {mark} {}{}",
                agent.name,
                if agent.installed {
                    ""
                } else {
                    " (not installed)"
                }
            );
        }
        for problem in &report.problems {
            match problem.level {
                Level::Error => println!("{} {}", ui::red("error:"), problem.message),
                Level::Warning => println!("{} {}", ui::yellow("warning:"), problem.message),
            }
        }
        if report.problems.is_empty() {
            println!("{}", ui::green("No problems found."));
        }
    }
    Ok(if report.ok { 0 } else { 1 })
}
