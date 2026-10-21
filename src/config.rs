//! The user's config file: defaults for the agent, models and prompt.
//!
//! One JSON file in the home config directory. Everything in it is optional, a missing file means
//! built-in defaults, and unknown keys are reported as warnings rather than errors so a newer
//! config does not break an older binary. Flags and environment variables override it; that
//! layering happens in the CLI, not here.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const CONFIG_FILE: &str = "config.json";
pub const CONFIG_DIR: &str = "resolve-merge-conflicts";
pub const ENV_CONFIG: &str = "RESOLVE_MERGE_CONFLICTS_CONFIG";

/// Where the config file comes from, for `config path` and error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Flag,
    Env,
    User,
}

impl Rule {
    pub fn describe(self) -> &'static str {
        match self {
            Self::Flag => "--config",
            Self::Env => "$RESOLVE_MERGE_CONFLICTS_CONFIG",
            Self::User => "the user config directory",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub path: PathBuf,
    pub rule: Rule,
}

/// Per-agent settings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AgentConfig {
    /// The model preselected in the picker.
    pub model: Option<String>,
    /// Models offered in the picker (Codex also discovers its own; these are the fallback).
    pub models: Vec<String>,
    #[serde(flatten, skip_serializing)]
    unknown: BTreeMap<String, serde_json::Value>,
}

/// The config file as written. Every field is optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// `claude` or `codex`; preselected in the picker.
    pub agent: Option<String>,
    /// The prompt template. See the prompt module for placeholders.
    pub prompt: Option<String>,
    /// A file holding the prompt template, instead of `prompt`.
    pub prompt_file: Option<PathBuf>,
    /// Editor for amending the prompt; falls back to `$VISUAL`, `$EDITOR`.
    pub editor: Option<String>,
    /// Stage resolved files with `git add` afterwards. Default true.
    pub auto_stage: Option<bool>,
    pub claude: AgentConfig,
    pub codex: AgentConfig,
    #[serde(flatten, skip_serializing)]
    unknown: BTreeMap<String, serde_json::Value>,
}

pub const DEFAULT_CLAUDE_MODELS: &[&str] = &["opus", "sonnet", "haiku"];
pub const DEFAULT_CLAUDE_MODEL: &str = "sonnet";

impl Config {
    pub fn auto_stage(&self) -> bool {
        self.auto_stage.unwrap_or(true)
    }

    pub fn agent_config(&self, agent: &str) -> Option<&AgentConfig> {
        match agent {
            "claude" => Some(&self.claude),
            "codex" => Some(&self.codex),
            _ => None,
        }
    }

    /// Models to offer for `agent` from config alone, defaults included. The default model, when
    /// there is one, is always in the list.
    pub fn models_for(&self, agent: &str) -> Vec<String> {
        let mut models: Vec<String> = match agent {
            "claude" if self.claude.models.is_empty() => DEFAULT_CLAUDE_MODELS
                .iter()
                .map(|model| model.to_string())
                .collect(),
            _ => self
                .agent_config(agent)
                .map(|config| config.models.clone())
                .unwrap_or_default(),
        };
        if let Some(default) = self.default_model_for(agent)
            && !models.contains(&default)
        {
            models.insert(0, default);
        }
        models
    }

    /// The model to preselect for `agent`, if config or the built-in defaults say.
    pub fn default_model_for(&self, agent: &str) -> Option<String> {
        self.agent_config(agent)
            .and_then(|config| config.model.clone())
            .or_else(|| (agent == "claude").then(|| DEFAULT_CLAUDE_MODEL.to_string()))
    }

    /// Keys this version does not know, as `agent.foo` style paths.
    pub fn unknown_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.unknown.keys().cloned().collect();
        keys.extend(
            self.claude
                .unknown
                .keys()
                .map(|key| format!("claude.{key}")),
        );
        keys.extend(self.codex.unknown.keys().map(|key| format!("codex.{key}")));
        keys.sort();
        keys
    }
}

/// A loaded config and everything worth telling the user about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    pub config: Config,
    /// The file it came from; `None` when there is none and defaults apply.
    pub path: Option<PathBuf>,
    pub warnings: Vec<String>,
}

/// The user config directory: `$XDG_CONFIG_HOME/resolve-merge-conflicts`, else
/// `~/.config/resolve-merge-conflicts` on Unix (macOS included: this is a home-directory dotfile
/// tool, not an app), and the platform config directory on Windows.
pub fn user_config_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        dirs::config_dir().map(|dir| dir.join(CONFIG_DIR))
    }
    #[cfg(not(windows))]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| dirs::home_dir().map(|home| home.join(".config")))?;
        Some(base.join(CONFIG_DIR))
    }
}

/// Which file to use: `--config`, then `$RESOLVE_MERGE_CONFLICTS_CONFIG`, then the user file.
/// The file need not exist.
pub fn locate(explicit: Option<&Path>) -> Option<Location> {
    locate_with(explicit, std::env::var_os(ENV_CONFIG).map(PathBuf::from))
}

fn locate_with(explicit: Option<&Path>, env: Option<PathBuf>) -> Option<Location> {
    if let Some(path) = explicit {
        return Some(Location {
            path: path.to_path_buf(),
            rule: Rule::Flag,
        });
    }
    if let Some(path) = env.filter(|path| !path.as_os_str().is_empty()) {
        return Some(Location {
            path,
            rule: Rule::Env,
        });
    }
    user_config_dir().map(|dir| Location {
        path: dir.join(CONFIG_FILE),
        rule: Rule::User,
    })
}

/// Load the config. A missing file is fine for the user file (defaults); a missing file the user
/// explicitly pointed at is an error, since that is a typo, not an absence.
pub fn load(explicit: Option<&Path>) -> Result<Loaded> {
    let Some(location) = locate(explicit) else {
        return Ok(Loaded {
            config: Config::default(),
            path: None,
            warnings: Vec::new(),
        });
    };
    load_from(&location)
}

pub fn load_from(location: &Location) -> Result<Loaded> {
    let text = match std::fs::read_to_string(&location.path) {
        Ok(text) => text,
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound && location.rule == Rule::User =>
        {
            return Ok(Loaded {
                config: Config::default(),
                path: None,
                warnings: Vec::new(),
            });
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("could not read {}", location.path.display()));
        }
    };
    let config: Config = serde_json::from_str(&text)
        .with_context(|| format!("{} is not valid config", location.path.display()))?;
    let warnings = config
        .unknown_keys()
        .into_iter()
        .map(|key| format!("unknown key \"{key}\" in {}", location.path.display()))
        .collect();
    Ok(Loaded {
        config,
        path: Some(location.path.clone()),
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_text(text: &str) -> Loaded {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, text).unwrap();
        load_from(&Location {
            path,
            rule: Rule::Flag,
        })
        .unwrap()
    }

    #[test]
    fn an_empty_object_means_all_defaults() {
        let loaded = load_text("{}");
        assert!(loaded.warnings.is_empty());
        assert_eq!(loaded.config.agent, None);
        assert!(loaded.config.auto_stage());
        assert_eq!(
            loaded.config.models_for("claude"),
            ["opus", "sonnet", "haiku"]
        );
        assert_eq!(loaded.config.default_model_for("claude").unwrap(), "sonnet");
        assert_eq!(loaded.config.default_model_for("codex"), None);
        assert!(loaded.config.models_for("codex").is_empty());
    }

    #[test]
    fn reads_every_documented_field() {
        let loaded = load_text(
            r#"{
              "$schema": "https://example.test/schema.json",
              "agent": "codex",
              "prompt": "Fix {{files}}",
              "promptFile": "/tmp/p.md",
              "editor": "nvim",
              "autoStage": false,
              "claude": {"model": "opus", "models": ["opus", "sonnet"]},
              "codex": {"model": "gpt-x", "models": ["gpt-x", "gpt-y"]}
            }"#,
        );
        let config = loaded.config;
        assert_eq!(config.agent.as_deref(), Some("codex"));
        assert_eq!(config.prompt.as_deref(), Some("Fix {{files}}"));
        assert_eq!(config.prompt_file, Some(PathBuf::from("/tmp/p.md")));
        assert_eq!(config.editor.as_deref(), Some("nvim"));
        assert!(!config.auto_stage());
        assert_eq!(config.models_for("claude"), ["opus", "sonnet"]);
        assert_eq!(config.default_model_for("codex").unwrap(), "gpt-x");
        assert_eq!(config.models_for("codex"), ["gpt-x", "gpt-y"]);
    }

    #[test]
    fn the_default_model_is_always_offered() {
        let loaded = load_text(r#"{"codex": {"model": "gpt-z", "models": ["gpt-x"]}}"#);
        assert_eq!(loaded.config.models_for("codex"), ["gpt-z", "gpt-x"]);
    }

    #[test]
    fn unknown_keys_are_warned_about_not_rejected() {
        let loaded = load_text(r#"{"agnet": "claude", "claude": {"modle": "x"}}"#);
        assert_eq!(loaded.warnings.len(), 2);
        assert!(loaded.warnings[0].contains("agnet"));
        assert!(loaded.warnings[1].contains("claude.modle"));
    }

    #[test]
    fn invalid_json_is_an_error_naming_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, "{ nope").unwrap();
        let error = load_from(&Location {
            path: path.clone(),
            rule: Rule::Flag,
        })
        .unwrap_err();
        assert!(format!("{error:#}").contains(&path.display().to_string()));
    }

    #[test]
    fn a_missing_user_file_is_defaults_but_a_missing_explicit_file_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("absent.json");
        let user = load_from(&Location {
            path: path.clone(),
            rule: Rule::User,
        })
        .unwrap();
        assert_eq!(user.path, None);
        assert!(
            load_from(&Location {
                path,
                rule: Rule::Flag
            })
            .is_err()
        );
    }

    #[test]
    fn flag_beats_env_beats_the_user_file() {
        let flag = locate_with(Some(Path::new("/a.json")), Some("/b.json".into())).unwrap();
        assert_eq!((flag.path, flag.rule), ("/a.json".into(), Rule::Flag));
        let env = locate_with(None, Some("/b.json".into())).unwrap();
        assert_eq!((env.path, env.rule), ("/b.json".into(), Rule::Env));
        let empty_env = locate_with(None, Some("".into())).unwrap();
        assert_eq!(empty_env.rule, Rule::User);
        assert!(
            empty_env
                .path
                .ends_with("resolve-merge-conflicts/config.json")
        );
    }
}
