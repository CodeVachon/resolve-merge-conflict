//! Running the Codex adapter against a fake `codex` executable.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use resolve_merge_conflicts::agent::codex::CodexAgent;
use resolve_merge_conflicts::agent::{self, AgentRequest};

/// A fake that writes `$message` to the file named by --output-last-message, as codex does.
fn fake(dir: &Path, body: &str) -> std::path::PathBuf {
    let path = dir.join("codex");
    let script = format!(
        r#"#!/bin/sh
out=""
while [ $# -gt 0 ]; do
  if [ "$1" = "--output-last-message" ]; then out="$2"; fi
  shift
done
{body}
"#
    );
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn request(cwd: &Path) -> AgentRequest {
    AgentRequest {
        prompt: "the prompt".into(),
        model: "m".into(),
        cwd: cwd.to_path_buf(),
    }
}

#[test]
fn reads_the_final_message_from_the_output_file_and_removes_it() {
    let dir = tempfile::tempdir().unwrap();
    let seen = dir.path().join("seen-path");
    let program = fake(
        dir.path(),
        &format!(
            r#"echo noisy progress; echo "$out" > {}; printf 'got: %s' "$(cat)" > "$out""#,
            seen.display()
        ),
    );
    let outcome = agent::run(&CodexAgent::with_program(program), &request(dir.path())).unwrap();
    assert!(outcome.success);
    assert_eq!(outcome.final_message, "got: the prompt");
    let written = std::fs::read_to_string(seen).unwrap();
    assert!(
        !Path::new(written.trim()).exists(),
        "the output file should be cleaned up"
    );
}

#[test]
fn falls_back_to_stdout_when_no_file_was_written() {
    let dir = tempfile::tempdir().unwrap();
    let program = fake(dir.path(), "cat >/dev/null; echo from stdout");
    let outcome = agent::run(&CodexAgent::with_program(program), &request(dir.path())).unwrap();
    assert_eq!(outcome.final_message, "from stdout");
}

#[test]
fn a_nonzero_exit_is_a_failure() {
    let dir = tempfile::tempdir().unwrap();
    let program = fake(dir.path(), "cat >/dev/null; echo bad >&2; exit 2");
    let outcome = agent::run(&CodexAgent::with_program(program), &request(dir.path())).unwrap();
    assert!(!outcome.success);
    assert_eq!(outcome.exit_code, Some(2));
}
