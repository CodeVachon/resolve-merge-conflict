//! `config path`, `config init` and `config doctor` through the real binary.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use assert_cmd::Command;
use predicates::str::contains;

struct Home {
    dir: tempfile::TempDir,
}

impl Home {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let fake = bin.join("claude");
        std::fs::write(&fake, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        Self { dir }
    }

    fn config_dir(&self) -> std::path::PathBuf {
        self.dir.path().join("xdg/resolve-merge-conflicts")
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::cargo_bin("resolve-merge-conflicts").unwrap();
        command
            .env("XDG_CONFIG_HOME", self.dir.path().join("xdg"))
            .env_remove("RESOLVE_MERGE_CONFLICTS_CONFIG")
            .env(
                "RESOLVE_MERGE_CONFLICTS_CLAUDE_BIN",
                self.dir.path().join("bin/claude"),
            )
            .env(
                "RESOLVE_MERGE_CONFLICTS_CODEX_BIN",
                self.dir.path().join("bin/no-codex"),
            )
            .env("NO_COLOR", "1")
            .args(args);
        command
    }

    fn write_config(&self, text: &str) {
        std::fs::create_dir_all(self.config_dir()).unwrap();
        std::fs::write(self.config_dir().join("config.json"), text).unwrap();
    }
}

#[test]
fn path_prints_the_home_config_file_and_the_rule() {
    let home = Home::new();
    home.command(&["config", "path"])
        .assert()
        .success()
        .stdout(contains("xdg/resolve-merge-conflicts/config.json"))
        .stdout(contains("user config directory"))
        .stdout(contains("does not exist yet"));
}

#[test]
fn path_honours_the_flag_and_the_environment() {
    let home = Home::new();
    home.command(&["config", "path", "-f", "/tmp/x.json"])
        .assert()
        .success()
        .stdout(contains("/tmp/x.json"))
        .stdout(contains("--config"));
    home.command(&["config", "path"])
        .env("RESOLVE_MERGE_CONFLICTS_CONFIG", "/tmp/env.json")
        .assert()
        .success()
        .stdout(contains("/tmp/env.json"))
        .stdout(contains("RESOLVE_MERGE_CONFLICTS_CONFIG"));
}

#[test]
fn init_writes_a_config_and_an_editable_prompt_file_and_keeps_them_on_rerun() {
    let home = Home::new();
    home.command(&["config", "init"])
        .assert()
        .success()
        .stdout(contains("created"));
    let config = std::fs::read_to_string(home.config_dir().join("config.json")).unwrap();
    assert!(config.contains("\"promptFile\": \"prompt.md\""));
    assert!(config.contains("\"sonnet\""));
    let prompt = std::fs::read_to_string(home.config_dir().join("prompt.md")).unwrap();
    assert!(prompt.contains("{{files}}"));

    std::fs::write(home.config_dir().join("prompt.md"), "my edits").unwrap();
    home.command(&["config", "init"])
        .assert()
        .success()
        .stdout(contains("kept"));
    assert_eq!(
        std::fs::read_to_string(home.config_dir().join("prompt.md")).unwrap(),
        "my edits"
    );

    home.command(&["config", "init", "--force"])
        .assert()
        .success()
        .stdout(contains("replaced"));
    assert!(
        std::fs::read_to_string(home.config_dir().join("prompt.md"))
            .unwrap()
            .contains("{{files}}")
    );
}

#[test]
fn a_freshly_initialised_config_passes_doctor() {
    let home = Home::new();
    home.command(&["config", "init"]).assert().success();
    home.command(&["config", "doctor"])
        .assert()
        .success()
        .stdout(contains("No problems found"))
        .stdout(contains("claude"))
        .stdout(contains("codex (not installed)"));
}

#[test]
fn doctor_with_no_config_file_is_fine() {
    let home = Home::new();
    home.command(&["config", "doctor"])
        .assert()
        .success()
        .stdout(contains("none found"));
}

#[test]
fn doctor_catches_a_bad_agent_a_placeholder_typo_and_bad_json() {
    let home = Home::new();
    home.write_config(r#"{"agent": "gemini", "prompt": "fix {{fils}}"}"#);
    home.command(&["config", "doctor"])
        .assert()
        .code(1)
        .stdout(contains("must be one of: claude, codex"))
        .stdout(contains("{{fils}}"));

    home.write_config("{ nope");
    home.command(&["config", "doctor"])
        .assert()
        .code(1)
        .stdout(contains("not valid config"));
}

#[test]
fn doctor_warns_but_passes_for_soft_problems() {
    let home = Home::new();
    home.write_config(
        r#"{"prompt": "just fix it", "agnet": "x", "claude": {"model": "opus", "models": ["sonnet"]}}"#,
    );
    home.command(&["config", "doctor"])
        .assert()
        .success()
        .stdout(contains("unknown key \"agnet\""))
        .stdout(contains("does not use {{files}}"))
        .stdout(contains("does not use {{summary_format}}"))
        .stdout(contains("claude.model \"opus\" is not in claude.models"));
}

#[test]
fn doctor_fails_when_no_agent_is_installed() {
    let home = Home::new();
    home.command(&["config", "doctor"])
        .env(
            "RESOLVE_MERGE_CONFLICTS_CLAUDE_BIN",
            Path::new("/definitely/not/claude"),
        )
        .assert()
        .code(1)
        .stdout(contains("neither `claude` nor `codex`"));
}

#[test]
fn doctor_json_has_the_documented_shape() {
    let home = Home::new();
    home.write_config(r#"{"agent": "gemini"}"#);
    let output = home
        .command(&["config", "doctor", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["ok"], false);
    assert_eq!(value["problems"][0]["level"], "error");
    assert_eq!(value["agents"][0]["name"], "claude");
    assert_eq!(value["agents"][0]["installed"], true);
    assert!(value["path"].as_str().unwrap().ends_with("config.json"));
}
