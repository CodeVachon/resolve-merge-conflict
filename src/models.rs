//! Which models to offer for each agent.
//!
//! Claude Code has no command that lists models, so its list comes from config (with sensible
//! aliases built in). Codex can print its catalog, so its list is discovered live and falls back to
//! config when `codex` is missing, slow, or prints something unexpected.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::config::Config;

const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10);

/// Where a list of models came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Config,
    Discovered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOptions {
    pub models: Vec<String>,
    /// The model to preselect.
    pub default: Option<String>,
    pub source: Source,
    /// Why discovery was not used, when it was tried and failed.
    pub note: Option<String>,
}

/// The models to offer for `agent`.
pub fn options(agent: &str, config: &Config) -> ModelOptions {
    let configured = config.models_for(agent);
    let default = config.default_model_for(agent);
    if agent != "codex" {
        return ModelOptions {
            models: configured,
            default,
            source: Source::Config,
            note: None,
        };
    }

    let program = std::env::var_os("RESOLVE_MERGE_CONFLICTS_CODEX_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex"));
    match discover_codex(&program) {
        Ok(discovered) if !discovered.is_empty() => {
            let default = default.or_else(|| discovered.first().cloned());
            let mut models = discovered;
            // Config entries the catalog does not list (a model on a plan this machine cannot
            // see yet, say) are still the user's to pick.
            for model in configured {
                if !models.contains(&model) {
                    models.push(model);
                }
            }
            if let Some(default) = &default
                && !models.contains(default)
            {
                models.insert(0, default.clone());
            }
            ModelOptions {
                models,
                default,
                source: Source::Discovered,
                note: None,
            }
        }
        Ok(_) => ModelOptions {
            models: configured,
            default,
            source: Source::Config,
            note: Some("codex listed no models".to_string()),
        },
        Err(error) => ModelOptions {
            models: configured,
            default,
            source: Source::Config,
            note: Some(format!("could not discover codex models: {error:#}")),
        },
    }
}

#[derive(Debug, Deserialize)]
struct Catalog {
    models: Vec<CatalogModel>,
}

#[derive(Debug, Deserialize)]
struct CatalogModel {
    slug: String,
    #[serde(default)]
    visibility: Option<String>,
    #[serde(default)]
    priority: Option<i64>,
}

/// Model slugs from `codex debug models` output: the ones Codex itself lists, best first. Hidden
/// models (visibility other than `list`) are skipped.
pub fn parse_codex_catalog(json: &str) -> Result<Vec<String>> {
    let catalog: Catalog = serde_json::from_str(json)
        .context("codex printed a model catalog this tool cannot read")?;
    let mut listed: Vec<&CatalogModel> = catalog
        .models
        .iter()
        .filter(|model| model.visibility.as_deref().is_none_or(|v| v == "list"))
        .collect();
    listed.sort_by_key(|model| model.priority.unwrap_or(i64::MAX));
    Ok(listed.into_iter().map(|model| model.slug.clone()).collect())
}

/// Run `<program> debug models` and parse it, giving up after ten seconds.
pub fn discover_codex(program: &Path) -> Result<Vec<String>> {
    let mut child = Command::new(program)
        .args(["debug", "models"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("could not start {}", program.display()))?;

    let mut stdout = child.stdout.take().context("no stdout")?;
    let reader = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stdout.read_to_string(&mut text);
        text
    });

    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() > DISCOVERY_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            bail!("timed out after {}s", DISCOVERY_TIMEOUT.as_secs());
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let text = reader.join().unwrap_or_default();
    if !status.success() {
        bail!("`codex debug models` exited with {status}");
    }
    parse_codex_catalog(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_listed_models_best_priority_first_and_skips_hidden_ones() {
        let json = r#"{"models":[
            {"slug":"b","visibility":"list","priority":2,"extra":{"x":1}},
            {"slug":"hidden","visibility":"hide","priority":0},
            {"slug":"a","visibility":"list","priority":1},
            {"slug":"no-visibility"}
        ]}"#;
        assert_eq!(
            parse_codex_catalog(json).unwrap(),
            ["a", "b", "no-visibility"]
        );
    }

    #[test]
    fn something_that_is_not_a_catalog_is_an_error() {
        assert!(parse_codex_catalog("<html>").is_err());
        assert!(parse_codex_catalog(r#"{"models": "nope"}"#).is_err());
    }

    #[test]
    fn claude_models_come_from_config_with_builtin_defaults() {
        let options = options("claude", &Config::default());
        assert_eq!(options.models, ["opus", "sonnet", "haiku"]);
        assert_eq!(options.default.as_deref(), Some("sonnet"));
        assert_eq!(options.source, Source::Config);
    }
}
