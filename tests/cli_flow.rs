//! The real binary, run in a conflicted repository, with a fake `claude` standing in for the agent.
#![cfg(unix)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use common::{CONFLICTED, Fixture, Operation};
use predicates::str::contains;

struct Env {
    dir: tempfile::TempDir,
}

impl Env {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let script = r#"#!/bin/sh
printf '%s\n' "$@" > "$FAKE_LOG.args"
cat > "$FAKE_LOG.stdin"
case "$FAKE_MODE" in
  resolve)
    printf 'one\nresolved\nthree\n' > a.txt
    printf '%s' '{"is_error":false,"result":"```json\n{\"files\":[{\"path\":\"a.txt\",\"strategy\":\"merged\",\"summary\":\"combined both edits\"}]}\n```"}'
    ;;
  lazy)
    printf '%s' '{"is_error":false,"result":"all done!"}'
    ;;
  fail)
    echo "it broke" >&2
    exit 3
    ;;
  commit)
    printf 'one\nresolved\nthree\n' > a.txt
    git add a.txt && git commit -q -m "agent commit"
    printf '%s' '{"is_error":false,"result":"committed"}'
    ;;
esac
"#;
        let path = dir.path().join("claude");
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        Self { dir }
    }

    fn log(&self, suffix: &str) -> PathBuf {
        self.dir.path().join(format!("log.{suffix}"))
    }

    fn command(&self, repo: &Path, mode: &str) -> Command {
        let mut command = Command::cargo_bin("resolve-merge-conflicts").unwrap();
        command
            .current_dir(repo)
            .env(
                "RESOLVE_MERGE_CONFLICTS_CLAUDE_BIN",
                self.dir.path().join("claude"),
            )
            .env("XDG_CONFIG_HOME", self.dir.path().join("xdg"))
            .env_remove("RESOLVE_MERGE_CONFLICTS_CONFIG")
            .env("FAKE_MODE", mode)
            .env("FAKE_LOG", self.dir.path().join("log"))
            .env("NO_COLOR", "1")
            // Git state is under test, so keep the agent's own git calls hermetic too.
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "f@example.com")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "f@example.com")
            .args(["-y", "-a", "claude", "-m", "sonnet"]);
        command
    }
}

#[test]
fn resolves_and_stages_and_reports_without_committing() {
    let env = Env::new();
    let fixture = Fixture::conflicted(Operation::Merge);
    let head = fixture.head();

    env.command(&fixture.work, "resolve")
        .assert()
        .success()
        .stdout(contains("1 of 1 file resolved"))
        .stdout(contains("combined both edits"))
        .stdout(contains("git merge --continue"))
        .stdout(contains("Nothing was committed or pushed"));

    assert_eq!(fixture.read(CONFLICTED), "one\nresolved\nthree\n");
    assert_eq!(fixture.staged(), [CONFLICTED]);
    assert!(fixture.unmerged().is_empty());
    assert_eq!(fixture.head(), head);
}

#[test]
fn the_agent_receives_the_model_a_restricted_toolset_and_the_prompt() {
    let env = Env::new();
    let fixture = Fixture::conflicted(Operation::Merge);
    env.command(&fixture.work, "resolve").assert().success();

    let args = std::fs::read_to_string(env.log("args")).unwrap();
    assert!(args.contains("sonnet"));
    assert!(args.contains("acceptEdits"));
    assert!(
        args.contains("Bash(git commit:*)"),
        "commit must be disallowed"
    );

    let prompt = std::fs::read_to_string(env.log("stdin")).unwrap();
    assert!(prompt.contains("a.txt (both modified)"));
    assert!(prompt.contains("Do NOT run git add, git commit, git push"));
}

#[test]
fn an_agent_that_does_not_resolve_exits_1_and_stages_nothing() {
    let env = Env::new();
    let fixture = Fixture::conflicted(Operation::Merge);
    env.command(&fixture.work, "lazy")
        .assert()
        .code(1)
        .stdout(contains("conflict markers remain"))
        .stdout(contains("Still conflicted: a.txt"));
    assert_eq!(fixture.unmerged(), [CONFLICTED]);
}

#[test]
fn a_failing_agent_exits_1_and_shows_why() {
    let env = Env::new();
    let fixture = Fixture::conflicted(Operation::Merge);
    env.command(&fixture.work, "fail")
        .assert()
        .code(1)
        .stdout(contains("did not finish cleanly (exit 3)"))
        .stdout(contains("it broke"));
}

#[test]
fn an_agent_that_commits_is_called_out_and_fails_the_run() {
    let env = Env::new();
    let fixture = Fixture::conflicted(Operation::Merge);
    env.command(&fixture.work, "commit")
        .assert()
        .code(1)
        .stdout(contains("HEAD moved"));
}

#[test]
fn a_clean_repository_exits_0_with_nothing_to_do() {
    let env = Env::new();
    let fixture = Fixture::new();
    env.command(&fixture.work, "resolve")
        .assert()
        .success()
        .stdout(contains("No merge conflicts found"));
    assert!(!env.log("args").exists(), "the agent must not have run");
}

#[test]
fn dry_run_runs_nothing() {
    let env = Env::new();
    let fixture = Fixture::conflicted(Operation::Merge);
    env.command(&fixture.work, "resolve")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(contains("Dry run"));
    assert!(!env.log("args").exists());
    assert_eq!(fixture.unmerged(), [CONFLICTED]);
}

#[test]
fn json_mode_prints_only_the_report_on_stdout() {
    let env = Env::new();
    let fixture = Fixture::conflicted(Operation::Merge);
    let output = env
        .command(&fixture.work, "resolve")
        .arg("--json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["files"][0]["path"], "a.txt");
    assert_eq!(value["files"][0]["status"], "resolved");
    assert_eq!(value["agent"], "claude");
    assert!(value["mismatches"].as_array().unwrap().is_empty());
}

#[test]
fn a_custom_prompt_file_replaces_the_default_prompt() {
    let env = Env::new();
    let fixture = Fixture::conflicted(Operation::Merge);
    let prompt = env.dir.path().join("mine.md");
    std::fs::write(&prompt, "Please fix these: {{files}}").unwrap();
    env.command(&fixture.work, "resolve")
        .args(["--prompt-file", prompt.to_str().unwrap()])
        .assert()
        .success();
    let sent = std::fs::read_to_string(env.log("stdin")).unwrap();
    assert!(sent.starts_with("Please fix these: - a.txt (both modified)"));
}

#[test]
fn the_home_config_prompt_is_used_and_no_stage_leaves_files_unstaged() {
    let env = Env::new();
    let fixture = Fixture::conflicted(Operation::Merge);
    let config_dir = env.dir.path().join("xdg/resolve-merge-conflicts");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(
        config_dir.join("config.json"),
        r#"{"prompt": "From my config: {{operation}}"}"#,
    )
    .unwrap();
    env.command(&fixture.work, "resolve")
        .arg("--no-stage")
        .assert()
        .success()
        .stdout(contains("git add"));
    assert_eq!(
        std::fs::read_to_string(env.log("stdin")).unwrap(),
        "From my config: merge"
    );
    assert_eq!(fixture.unmerged(), [CONFLICTED], "not staged");
}

#[test]
fn without_a_terminal_or_yes_it_refuses_instead_of_hanging() {
    let env = Env::new();
    let fixture = Fixture::conflicted(Operation::Merge);
    let mut command = Command::cargo_bin("resolve-merge-conflicts").unwrap();
    command
        .current_dir(&fixture.work)
        .env("XDG_CONFIG_HOME", env.dir.path().join("xdg"))
        .assert()
        .code(1)
        .stderr(contains("needs a terminal"));
}

#[test]
fn an_agent_that_is_not_installed_is_a_clear_error() {
    let env = Env::new();
    let fixture = Fixture::conflicted(Operation::Merge);
    env.command(&fixture.work, "resolve")
        .env(
            "RESOLVE_MERGE_CONFLICTS_CLAUDE_BIN",
            env.dir.path().join("missing"),
        )
        .assert()
        .code(1)
        .stderr(contains("not installed"));
}

#[test]
fn works_from_a_subdirectory_of_the_repository() {
    let env = Env::new();
    let fixture = Fixture::conflicted(Operation::Merge);
    let sub = fixture.work.join("nested/dir");
    std::fs::create_dir_all(&sub).unwrap();
    env.command(&sub, "resolve").assert().success();
    assert_eq!(fixture.staged(), [CONFLICTED]);
}
