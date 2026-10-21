//! The runner end to end, with a scripted prompter and a fake agent that is just a shell script.
#![cfg(unix)]

mod common;

use std::collections::VecDeque;

use anyhow::Result;
use common::{CONFLICTED, Fixture, Operation};
use resolve_merge_conflicts::agent::{Agent, AgentRequest, Invocation, ParsedOutput};
use resolve_merge_conflicts::config::Config;
use resolve_merge_conflicts::models::{ModelOptions, Source};
use resolve_merge_conflicts::runner::{self, Deps, Options, Outcome, Prompter};
use resolve_merge_conflicts::verify::FileStatus;

/// An agent that runs `script` with the prompt on stdin, in the repo.
struct FakeAgent {
    script: String,
}

impl Agent for FakeAgent {
    fn name(&self) -> &'static str {
        "claude"
    }
    fn binary(&self) -> &'static str {
        "claude"
    }
    fn invocation(&self, request: &AgentRequest) -> Invocation {
        Invocation {
            program: "sh".into(),
            args: vec!["-c".into(), self.script.clone()],
            stdin: request.prompt.clone(),
            cwd: request.cwd.clone(),
            output_file: None,
        }
    }
    fn parse_output(&self, stdout: &str) -> ParsedOutput {
        ParsedOutput {
            final_message: stdout.trim().to_string(),
            is_error: false,
        }
    }
}

#[derive(Default)]
struct Scripted {
    selects: VecDeque<usize>,
    edits: VecDeque<String>,
    texts: VecDeque<String>,
    questions: Vec<String>,
    shown: Vec<String>,
    info: Vec<String>,
}

impl Prompter for Scripted {
    fn select(&mut self, question: &str, _: &[String], default: Option<usize>) -> Result<usize> {
        self.questions.push(question.to_string());
        Ok(self.selects.pop_front().or(default).unwrap_or(0))
    }
    fn text(&mut self, question: &str, default: Option<&str>) -> Result<String> {
        self.questions.push(question.to_string());
        Ok(self
            .texts
            .pop_front()
            .or(default.map(str::to_string))
            .unwrap_or_default())
    }
    fn edit(&mut self, _: &str, initial: &str) -> Result<String> {
        Ok(self
            .edits
            .pop_front()
            .unwrap_or_else(|| initial.to_string()))
    }
    fn show(&mut self, _: &str, body: &str) {
        self.shown.push(body.to_string());
    }
    fn info(&mut self, message: &str) {
        self.info.push(message.to_string());
    }
}

fn go(
    fixture: &Fixture,
    script: &str,
    options: Options,
    prompter: &mut Scripted,
    installed: &'static [&'static str],
    models: Vec<&'static str>,
) -> Result<Outcome> {
    let script = script.to_string();
    let make_agent = move |name: &str| -> Result<Box<dyn Agent>> {
        assert_eq!(name, "claude");
        Ok(Box::new(FakeAgent {
            script: script.clone(),
        }))
    };
    let is_installed = move |name: &str| installed.contains(&name);
    let model_options = move |_: &str, _: &Config| ModelOptions {
        models: models.iter().map(|m| m.to_string()).collect(),
        default: models.first().map(|m| m.to_string()),
        source: Source::Config,
        note: None,
    };
    let deps = Deps {
        make_agent: &make_agent,
        installed: &is_installed,
        model_options: &model_options,
    };
    let options = Options {
        cwd: fixture.work.clone(),
        ..options
    };
    runner::run(&options, &Config::default(), prompter, &deps)
}

fn yes() -> Options {
    Options {
        yes: true,
        ..Options::default()
    }
}

const RESOLVE: &str = "cat >/dev/null; printf 'one\\nresolved\\nthree\\n' > a.txt";

#[test]
fn resolves_stages_and_never_commits() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let head = fixture.head();
    let mut prompter = Scripted::default();
    let outcome = go(
        &fixture,
        RESOLVE,
        yes(),
        &mut prompter,
        &["claude"],
        vec!["m1", "m2"],
    )
    .unwrap();

    let Outcome::Finished(finished) = outcome else {
        panic!("expected Finished, got {outcome:?}");
    };
    assert!(finished.outcome.success);
    assert_eq!(finished.plan.model, "m1");
    assert!(finished.verification.all_resolved());
    assert!(finished.staged);
    assert_eq!(fixture.staged(), [CONFLICTED]);
    assert_eq!(fixture.head(), head);
    assert!(prompter.questions.is_empty(), "--yes must not ask anything");
}

#[test]
fn a_clean_repository_has_nothing_to_do() {
    let fixture = Fixture::new();
    let mut prompter = Scripted::default();
    let outcome = go(
        &fixture,
        "exit 1",
        yes(),
        &mut prompter,
        &["claude"],
        vec!["m"],
    )
    .unwrap();
    assert!(matches!(outcome, Outcome::NothingToDo));
}

#[test]
fn dry_run_shows_the_prompt_and_runs_nothing() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let mut prompter = Scripted::default();
    let options = Options {
        dry_run: true,
        ..yes()
    };
    let outcome = go(
        &fixture,
        "touch ran.txt",
        options,
        &mut prompter,
        &["claude"],
        vec!["m"],
    )
    .unwrap();
    let Outcome::DryRun(plan) = outcome else {
        panic!("expected DryRun");
    };
    assert!(plan.prompt.contains(CONFLICTED));
    assert!(!fixture.work.join("ran.txt").exists());
    assert_eq!(fixture.unmerged(), [CONFLICTED]);
}

#[test]
fn the_user_can_amend_the_prompt_before_it_is_sent() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let mut prompter = Scripted {
        // model pick (index 1), then "Edit the prompt", then "Run with this prompt".
        selects: [1, 1, 0].into(),
        edits: ["AMENDED PROMPT".to_string()].into(),
        ..Scripted::default()
    };
    let script = "cat > received.txt; printf 'one\\nok\\nthree\\n' > a.txt";
    let outcome = go(
        &fixture,
        script,
        Options::default(),
        &mut prompter,
        &["claude"],
        vec!["m1", "m2"],
    )
    .unwrap();
    let Outcome::Finished(finished) = outcome else {
        panic!("expected Finished");
    };
    assert_eq!(finished.plan.model, "m2");
    assert_eq!(fixture.read("received.txt"), "AMENDED PROMPT");
    assert_eq!(
        prompter.shown.len(),
        2,
        "the prompt is shown again after an edit"
    );
}

#[test]
fn cancelling_at_the_prompt_review_runs_nothing() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let mut prompter = Scripted {
        selects: [0, 2].into(),
        ..Scripted::default()
    };
    let outcome = go(
        &fixture,
        "touch ran.txt",
        Options::default(),
        &mut prompter,
        &["claude"],
        vec!["m"],
    )
    .unwrap();
    assert!(matches!(outcome, Outcome::Cancelled));
    assert!(!fixture.work.join("ran.txt").exists());
}

#[test]
fn an_agent_that_leaves_markers_leaves_the_file_unstaged() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let mut prompter = Scripted::default();
    let outcome = go(
        &fixture,
        "cat >/dev/null",
        yes(),
        &mut prompter,
        &["claude"],
        vec!["m"],
    )
    .unwrap();
    let Outcome::Finished(finished) = outcome else {
        panic!("expected Finished");
    };
    assert_eq!(
        finished.verification.files[0].status,
        FileStatus::MarkersRemain
    );
    assert_eq!(fixture.unmerged(), [CONFLICTED]);
    assert!(!finished.verification.files[0].staged);
}

#[test]
fn staging_can_be_switched_off() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let mut prompter = Scripted::default();
    let options = Options {
        stage: Some(false),
        ..yes()
    };
    go(
        &fixture,
        RESOLVE,
        options,
        &mut prompter,
        &["claude"],
        vec!["m"],
    )
    .unwrap();
    assert_eq!(fixture.unmerged(), [CONFLICTED]);
}

#[test]
fn an_unknown_or_missing_agent_is_a_clear_error() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let options = |agent: &str| Options {
        agent: Some(agent.to_string()),
        ..yes()
    };
    let mut prompter = Scripted::default();
    let error = go(
        &fixture,
        "",
        options("gemini"),
        &mut prompter,
        &["claude"],
        vec!["m"],
    )
    .unwrap_err();
    assert!(error.to_string().contains("unknown agent"));

    let error = go(
        &fixture,
        "",
        options("codex"),
        &mut prompter,
        &["claude"],
        vec!["m"],
    )
    .unwrap_err();
    assert!(error.to_string().contains("not installed"));

    let error = go(&fixture, "", yes(), &mut prompter, &[], vec!["m"]).unwrap_err();
    assert!(error.to_string().contains("no agent found"));
}

#[test]
fn with_no_known_models_a_person_is_asked_and_a_script_is_told_to_pass_one() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let mut prompter = Scripted {
        texts: ["typed-model".to_string()].into(),
        selects: [0].into(),
        ..Scripted::default()
    };
    let outcome = go(
        &fixture,
        RESOLVE,
        Options::default(),
        &mut prompter,
        &["claude"],
        vec![],
    )
    .unwrap();
    let Outcome::Finished(finished) = outcome else {
        panic!("expected Finished");
    };
    assert_eq!(finished.plan.model, "typed-model");

    let fixture = Fixture::conflicted(Operation::Merge);
    let error = go(
        &fixture,
        RESOLVE,
        yes(),
        &mut Scripted::default(),
        &["claude"],
        vec![],
    )
    .unwrap_err();
    assert!(error.to_string().contains("--model"));
}

#[test]
fn work_that_was_already_in_progress_is_mentioned() {
    let fixture = Fixture::conflicted(Operation::Merge);
    fixture.write("b.txt", "my edit\n");
    let mut prompter = Scripted::default();
    go(
        &fixture,
        RESOLVE,
        yes(),
        &mut prompter,
        &["claude"],
        vec!["m"],
    )
    .unwrap();
    assert!(
        prompter
            .info
            .iter()
            .any(|line| line.contains("1 other file"))
    );
}
