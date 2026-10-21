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
