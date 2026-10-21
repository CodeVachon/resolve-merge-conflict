//! Running the Claude adapter against a fake `claude` executable.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use resolve_merge_conflicts::agent::claude::ClaudeAgent;
use resolve_merge_conflicts::agent::{self, AgentRequest};

fn fake(dir: &Path, script: &str) -> std::path::PathBuf {
    let path = dir.join("claude");
    std::fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn request(cwd: &Path) -> AgentRequest {
    AgentRequest {
        prompt: "the prompt".into(),
        model: "opus".into(),
        cwd: cwd.to_path_buf(),
    }
}

#[test]
fn feeds_the_prompt_on_stdin_and_returns_the_result() {
    let dir = tempfile::tempdir().unwrap();
    let program = fake(
        dir.path(),
        r#"printf '{"is_error":false,"result":"got: %s"}' "$(cat)""#,
    );
    let outcome = agent::run(&ClaudeAgent::with_program(program), &request(dir.path())).unwrap();
    assert!(outcome.success);
    assert_eq!(outcome.final_message, "got: the prompt");
}

#[test]
fn runs_in_the_requested_directory() {
    let dir = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let program = fake(
        dir.path(),
        r#"cat >/dev/null; printf '{"result":"%s"}' "$(pwd -P)""#,
    );
    let outcome = agent::run(&ClaudeAgent::with_program(program), &request(work.path())).unwrap();
    assert_eq!(
        outcome.final_message,
        work.path().canonicalize().unwrap().to_string_lossy()
    );
}

#[test]
fn a_failing_agent_is_reported_not_raised() {
    let dir = tempfile::tempdir().unwrap();
    let program = fake(dir.path(), "cat >/dev/null; echo oops >&2; exit 3");
    let outcome = agent::run(&ClaudeAgent::with_program(program), &request(dir.path())).unwrap();
    assert!(!outcome.success);
    assert_eq!(outcome.exit_code, Some(3));
    assert!(outcome.stderr.contains("oops"));
}

#[test]
fn an_error_result_is_a_failure_even_with_exit_zero() {
    let dir = tempfile::tempdir().unwrap();
    let program = fake(
        dir.path(),
        r#"cat >/dev/null; echo '{"is_error":true,"result":"rate limited"}'"#,
    );
    let outcome = agent::run(&ClaudeAgent::with_program(program), &request(dir.path())).unwrap();
    assert!(!outcome.success);
    assert_eq!(outcome.final_message, "rate limited");
}

#[test]
fn a_large_prompt_does_not_deadlock() {
    let dir = tempfile::tempdir().unwrap();
    let program = fake(dir.path(), r#"printf '{"result":"%s"}' "$(wc -c)""#);
    let mut request = request(dir.path());
    request.prompt = "x".repeat(1_000_000);
    let outcome = agent::run(&ClaudeAgent::with_program(program), &request).unwrap();
    assert_eq!(outcome.final_message.trim(), "1000000");
}

#[test]
fn a_missing_program_is_an_error_naming_it() {
    let dir = tempfile::tempdir().unwrap();
    let error = agent::run(
        &ClaudeAgent::with_program(dir.path().join("nope")),
        &request(dir.path()),
    )
    .unwrap_err();
    assert!(error.to_string().contains("nope"));
}
