//! The command-line surface: flags parse, mistakes exit 2, help documents everything.

use assert_cmd::Command;
use clap::Parser;
use predicates::str::contains;
use resolve_merge_conflicts::cli::Cli;

fn parse(args: &[&str]) -> Cli {
    let mut argv = vec!["resolve-merge-conflicts"];
    argv.extend_from_slice(args);
    Cli::try_parse_from(argv).unwrap()
}

#[test]
fn every_run_flag_parses_in_short_and_long_form() {
    let cli = parse(&[
        "-c",
        "/repo",
        "-a",
        "codex",
        "-m",
        "gpt-x",
        "-y",
        "--no-edit",
        "--dry-run",
        "--no-stage",
        "-f",
        "/c.json",
        "--json",
    ]);
    let run = cli.run;
    assert_eq!(run.cwd.unwrap().to_str(), Some("/repo"));
    assert_eq!(run.agent.as_deref(), Some("codex"));
    assert_eq!(run.model.as_deref(), Some("gpt-x"));
    assert!(run.yes && run.no_edit && run.dry_run && run.no_stage && run.json);
    assert_eq!(run.config.unwrap().to_str(), Some("/c.json"));
}

#[test]
fn flags_become_runner_options() {
    let options = parse(&["-c", "/repo", "-y", "--no-stage"])
        .run
        .to_options()
        .unwrap();
    assert_eq!(options.cwd.to_str(), Some("/repo"));
    assert!(options.yes);
    assert_eq!(options.stage, Some(false));

    let options = parse(&["-c", "/repo"]).run.to_options().unwrap();
    assert_eq!(
        options.stage, None,
        "staging follows config unless switched off"
    );
}

#[test]
fn prompt_file_is_read_up_front_and_a_bad_path_fails_early() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("p.md");
    std::fs::write(&file, "custom {{files}}").unwrap();
    let options = parse(&["-c", "/r", "--prompt-file", file.to_str().unwrap()])
        .run
        .to_options()
        .unwrap();
    assert_eq!(options.prompt_template.as_deref(), Some("custom {{files}}"));

    let error = parse(&["--prompt-file", "/no/such/file"])
        .run
        .to_options()
        .unwrap_err();
    assert!(error.to_string().contains("--prompt-file"));
}

#[test]
fn unknown_flags_exit_2() {
    Command::cargo_bin("resolve-merge-conflicts")
        .unwrap()
        .arg("--nope")
        .assert()
        .code(2)
        .stderr(contains("--nope"));
}

#[test]
fn help_documents_the_flags_and_the_no_commit_promise() {
    let help = String::from_utf8(
        Command::cargo_bin("resolve-merge-conflicts")
            .unwrap()
            .arg("--help")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    for flag in [
        "--cwd",
        "--agent",
        "--model",
        "--yes",
        "--dry-run",
        "--no-edit",
        "--prompt-file",
        "--no-stage",
        "--config",
        "--json",
    ] {
        assert!(help.contains(flag), "--help is missing {flag}");
    }
}

#[test]
fn completion_scripts_are_generated() {
    for shell in ["bash", "zsh", "fish"] {
        Command::cargo_bin("resolve-merge-conflicts")
            .unwrap()
            .args(["completion", shell])
            .assert()
            .success()
            .stdout(contains("resolve-merge-conflicts"));
    }
}
