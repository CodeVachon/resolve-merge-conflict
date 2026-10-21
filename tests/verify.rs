//! The verifier and stager against real conflicted repositories.

mod common;

use common::{CONFLICTED, Fixture, Operation};
use resolve_merge_conflicts::conflicts;
use resolve_merge_conflicts::verify::{FileStatus, Snapshot, Strategy, verify_and_stage};

struct Run {
    fixture: Fixture,
    state: conflicts::ConflictState,
    before: Snapshot,
}

fn start(fixture: Fixture) -> Run {
    let mut git = fixture.git();
    let state = conflicts::detect(&mut git).unwrap().unwrap();
    let before = Snapshot::take(&mut git).unwrap();
    Run {
        fixture,
        state,
        before,
    }
}

fn finish(run: &Run, stage: bool) -> resolve_merge_conflicts::verify::Verification {
    verify_and_stage(&mut run.fixture.git(), &run.state, &run.before, stage).unwrap()
}

#[test]
fn a_resolved_file_is_staged_and_nothing_is_committed() {
    let run = start(Fixture::conflicted(Operation::Merge));
    let head = run.fixture.head();
    run.fixture
        .write(CONFLICTED, "one\nboth: main+feature\nthree\n");

    let result = finish(&run, true);
    let file = &result.files[0];
    assert_eq!(file.status, FileStatus::Resolved);
    assert_eq!(file.strategy, Some(Strategy::Merged));
    assert!(file.staged);
    assert_eq!((file.added, file.removed), (1, 1));
    assert!(result.all_resolved());
    assert!(!result.head_moved);
    assert_eq!(run.fixture.head(), head);
    assert_eq!(run.fixture.staged(), [CONFLICTED]);
    assert!(run.fixture.unmerged().is_empty());
}

#[test]
fn a_file_still_carrying_markers_is_left_unstaged() {
    let run = start(Fixture::conflicted(Operation::Merge));
    // Untouched: the conflict markers are exactly as git left them.
    let result = finish(&run, true);
    assert_eq!(result.files[0].status, FileStatus::MarkersRemain);
    assert!(!result.files[0].staged);
    assert!(!result.all_resolved());
    assert_eq!(run.fixture.unmerged(), [CONFLICTED]);
}

#[test]
fn taking_one_side_is_recognised() {
    let run = start(Fixture::conflicted(Operation::Merge));
    run.fixture.write(CONFLICTED, "one\nmain\nthree\n");
    assert_eq!(finish(&run, true).files[0].strategy, Some(Strategy::Ours));

    let run = start(Fixture::conflicted(Operation::Merge));
    run.fixture.write(CONFLICTED, "one\nfeature\nthree\n");
    assert_eq!(finish(&run, true).files[0].strategy, Some(Strategy::Theirs));
}

#[test]
fn staging_can_be_turned_off() {
    let run = start(Fixture::conflicted(Operation::Merge));
    run.fixture.write(CONFLICTED, "one\nfixed\nthree\n");
    let result = finish(&run, false);
    assert_eq!(result.files[0].status, FileStatus::Resolved);
    assert!(!result.files[0].staged);
    assert_eq!(run.fixture.unmerged(), [CONFLICTED]);
}

#[test]
fn deleting_a_file_is_a_valid_resolution_only_where_a_deletion_conflicts() {
    let run = start(Fixture::delete_modify_conflict());
    std::fs::remove_file(run.fixture.work.join("b.txt")).unwrap();
    let result = finish(&run, true);
    assert_eq!(result.files[0].status, FileStatus::Resolved);
    assert_eq!(result.files[0].strategy, Some(Strategy::Deleted));
    assert!(result.files[0].staged);

    let run = start(Fixture::conflicted(Operation::Merge));
    std::fs::remove_file(run.fixture.work.join(CONFLICTED)).unwrap();
    let result = finish(&run, true);
    assert_eq!(result.files[0].status, FileStatus::Missing);
    assert!(!result.files[0].staged);
}

#[test]
fn an_agent_that_commits_is_caught() {
    let run = start(Fixture::conflicted(Operation::Merge));
    run.fixture.write(CONFLICTED, "one\nfixed\nthree\n");
    run.fixture.raw(&["add", CONFLICTED]);
    run.fixture.raw(&["commit", "-m", "agent went rogue"]);
    assert!(finish(&run, true).head_moved);
}

#[test]
fn edits_and_new_files_outside_the_conflict_are_reported() {
    let run = start(Fixture::conflicted(Operation::Merge));
    run.fixture.write(CONFLICTED, "one\nfixed\nthree\n");
    run.fixture.write("b.txt", "agent touched this\n");
    run.fixture.write("notes/new.txt", "and created this\n");
    let result = finish(&run, true);
    assert_eq!(result.touched_outside, ["b.txt", "notes/new.txt"]);
}

#[test]
fn changes_that_were_already_there_are_not_blamed_on_the_agent() {
    let fixture = Fixture::conflicted(Operation::Merge);
    fixture.write("b.txt", "my own edit\n");
    let run = start(fixture);
    run.fixture.write(CONFLICTED, "one\nfixed\nthree\n");
    assert!(finish(&run, true).touched_outside.is_empty());
}
