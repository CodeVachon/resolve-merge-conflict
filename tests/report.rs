//! Report assembly from a finished run.

use resolve_merge_conflicts::agent::AgentOutcome;
use resolve_merge_conflicts::conflicts::{ConflictState, Operation};
use resolve_merge_conflicts::git::{ConflictKind, UnmergedEntry};
use resolve_merge_conflicts::report::Report;
use resolve_merge_conflicts::runner::{Finished, Plan};
use resolve_merge_conflicts::verify::{FileResult, FileStatus, Strategy, Verification};

fn file(path: &str, status: FileStatus, strategy: Option<Strategy>) -> FileResult {
    FileResult {
        path: path.into(),
        kind: ConflictKind::BothModified,
        status,
        strategy,
        staged: status == FileStatus::Resolved,
        added: 3,
        removed: 1,
    }
}

pub fn finished(message: &str, success: bool, files: Vec<FileResult>) -> Finished {
    let state = ConflictState {
        operation: Operation::Merge,
        files: files
            .iter()
            .map(|f| UnmergedEntry {
                path: f.path.clone(),
                kind: f.kind,
            })
            .collect(),
        ours: "ours".into(),
        theirs: "theirs".into(),
    };
    Finished {
        plan: Plan {
            state,
            agent: "claude".into(),
            model: "sonnet".into(),
            prompt: "p".into(),
            preexisting_changes: vec![],
        },
        outcome: AgentOutcome {
            success,
            exit_code: Some(if success { 0 } else { 1 }),
            final_message: message.into(),
            stderr: "line1\nline2\n".into(),
        },
        verification: Verification {
            files,
            head_moved: false,
            touched_outside: vec![],
        },
        staged: true,
    }
}

#[test]
fn joins_the_verified_facts_with_the_agents_claims_per_file() {
    let run = finished(
        r#"```json
{"files":[{"path":"a.rs","strategy":"merged","summary":"kept both"}]}
```"#,
        true,
        vec![
            file("a.rs", FileStatus::Resolved, Some(Strategy::Merged)),
            file("b.rs", FileStatus::Resolved, Some(Strategy::Ours)),
        ],
    );
    let report = Report::build(&run);
    assert!(report.has_agent_summary);
    assert_eq!(report.files.len(), 2);

    let a = &report.files[0];
    assert_eq!(
        (a.actual, a.claimed),
        (Some(Strategy::Merged), Some(Strategy::Merged))
    );
    assert_eq!(a.explanation, "kept both");
    assert!(a.mentioned && a.staged);
    assert_eq!((a.added, a.removed), (3, 1));

    let b = &report.files[1];
    assert!(!b.mentioned);
    assert_eq!(b.claimed, None);
    assert_eq!(report.resolved_count(), 2);
    assert!(report.all_resolved());
}

#[test]
fn without_a_summary_the_raw_message_is_kept_and_facts_still_stand() {
    let run = finished(
        "I fixed it, trust me.",
        true,
        vec![file("a.rs", FileStatus::Resolved, Some(Strategy::Merged))],
    );
    let report = Report::build(&run);
    assert!(!report.has_agent_summary);
    assert_eq!(report.agent_message, "I fixed it, trust me.");
    assert_eq!(report.files[0].actual, Some(Strategy::Merged));
    assert_eq!(report.files[0].claimed, None);
}

#[test]
fn summary_entries_for_unrelated_paths_are_collected() {
    let run = finished(
        r#"{"files":[{"path":"a.rs","strategy":"ours","summary":"x"},{"path":"other.rs","strategy":"ours","summary":"y"}]}"#,
        true,
        vec![file("a.rs", FileStatus::Resolved, Some(Strategy::Ours))],
    );
    assert_eq!(Report::build(&run).unexpected_summary_paths, ["other.rs"]);
}

#[test]
fn stderr_is_kept_only_for_failed_runs() {
    let ok = Report::build(&finished("", true, vec![]));
    assert_eq!(ok.agent_stderr, "");
    let failed = Report::build(&finished("", false, vec![]));
    assert_eq!(failed.agent_stderr, "line1\nline2");
}

#[test]
fn serialises_to_json_with_snake_case_names() {
    let run = finished(
        "",
        true,
        vec![file("a.rs", FileStatus::MarkersRemain, None)],
    );
    let json = serde_json::to_value(Report::build(&run)).unwrap();
    assert_eq!(json["files"][0]["status"], "markers_remain");
    assert_eq!(json["files"][0]["kind"], "both_modified");
    assert_eq!(json["operation"], "merge");
    assert_eq!(json["agent"], "claude");
}

mod mismatches {
    use super::*;
    use resolve_merge_conflicts::render;
    use resolve_merge_conflicts::report::Mismatch;

    fn with(message: &str, files: Vec<FileResult>) -> Report {
        Report::build(&finished(message, true, files))
    }

    #[test]
    fn an_honest_run_has_no_mismatches() {
        let report = with(
            r#"{"files":[{"path":"a.rs","strategy":"merged","summary":"ok"}]}"#,
            vec![file("a.rs", FileStatus::Resolved, Some(Strategy::Merged))],
        );
        assert!(report.mismatches().is_empty());
    }

    #[test]
    fn claiming_a_file_that_is_still_conflicted_is_flagged() {
        let report = with(
            r#"{"files":[{"path":"a.rs","strategy":"merged","summary":"done"}]}"#,
            vec![file("a.rs", FileStatus::MarkersRemain, None)],
        );
        assert_eq!(
            report.mismatches(),
            [Mismatch::ClaimedDoneButUnresolved {
                path: "a.rs".into()
            }]
        );
    }

    #[test]
    fn a_wrong_strategy_claim_is_flagged() {
        let report = with(
            r#"{"files":[{"path":"a.rs","strategy":"theirs","summary":"took theirs"}]}"#,
            vec![file("a.rs", FileStatus::Resolved, Some(Strategy::Merged))],
        );
        assert_eq!(
            report.mismatches(),
            [Mismatch::StrategyDiffers {
                path: "a.rs".into(),
                claimed: Strategy::Theirs,
                actual: Strategy::Merged
            }]
        );
    }

    #[test]
    fn omitted_and_invented_files_are_flagged_only_when_there_is_a_summary() {
        let report = with(
            r#"{"files":[{"path":"a.rs","strategy":"ours","summary":"x"},{"path":"ghost.rs","summary":"y"}]}"#,
            vec![
                file("a.rs", FileStatus::Resolved, Some(Strategy::Ours)),
                file("b.rs", FileStatus::Resolved, Some(Strategy::Ours)),
            ],
        );
        let found = report.mismatches();
        assert!(found.contains(&Mismatch::NotMentioned {
            path: "b.rs".into()
        }));
        assert!(found.contains(&Mismatch::UnexpectedPath {
            path: "ghost.rs".into()
        }));

        let no_summary = with(
            "prose only",
            vec![file("b.rs", FileStatus::Resolved, Some(Strategy::Ours))],
        );
        assert!(no_summary.mismatches().is_empty());
    }

    #[test]
    fn a_commit_or_stray_edit_is_flagged_first() {
        let mut run = finished(
            "",
            true,
            vec![file("a.rs", FileStatus::Resolved, Some(Strategy::Ours))],
        );
        run.verification.head_moved = true;
        run.verification.touched_outside = vec!["x.rs".into()];
        let found = Report::build(&run).mismatches();
        assert_eq!(found[0], Mismatch::HeadMoved);
        assert_eq!(
            found[1],
            Mismatch::TouchedOutside {
                paths: vec!["x.rs".into()]
            }
        );
    }

    #[test]
    fn the_text_report_says_what_to_do_next_and_that_nothing_was_committed() {
        let report = with(
            r#"{"files":[{"path":"a.rs","strategy":"merged","summary":"kept both imports"}]}"#,
            vec![file("a.rs", FileStatus::Resolved, Some(Strategy::Merged))],
        );
        let text = render::text(&report);
        assert!(text.contains("1 of 1 file resolved"));
        assert!(text.contains("a.rs"));
        assert!(text.contains("merged both"));
        assert!(text.contains("kept both imports"));
        assert!(text.contains("git diff --staged"));
        assert!(text.contains("git merge --continue"));
        assert!(text.contains("Nothing was committed or pushed"));
        assert!(!text.contains("Look closer"));
    }

    #[test]
    fn unresolved_files_are_named_and_no_continue_hint_is_given() {
        let report = with("", vec![file("a.rs", FileStatus::MarkersRemain, None)]);
        let text = render::text(&report);
        assert!(text.contains("conflict markers remain"));
        assert!(text.contains("Still conflicted: a.rs"));
        assert!(!text.contains("--continue"));
    }

    #[test]
    fn a_failed_agent_shows_its_exit_code_and_stderr() {
        let report = Report::build(&finished(
            "",
            false,
            vec![file("a.rs", FileStatus::MarkersRemain, None)],
        ));
        let text = render::text(&report);
        assert!(text.contains("did not finish cleanly (exit 1)"));
        assert!(text.contains("line2"));
    }

    #[test]
    fn without_a_summary_the_agents_own_words_are_shown() {
        let report = with(
            "I did it my way.",
            vec![file("a.rs", FileStatus::Resolved, Some(Strategy::Ours))],
        );
        let text = render::text(&report);
        assert!(text.contains("did not give a structured summary"));
        assert!(text.contains("I did it my way."));
    }

    #[test]
    fn json_includes_the_mismatches() {
        let report = with(
            "",
            vec![file("a.rs", FileStatus::Resolved, Some(Strategy::Ours))],
        );
        let value: serde_json::Value = serde_json::from_str(&render::json(&report)).unwrap();
        assert!(value["mismatches"].is_array());
        assert_eq!(value["files"][0]["path"], "a.rs");
    }
}
