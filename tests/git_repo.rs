//! The git wrapper against real repositories.

mod common;

use common::{CONFLICTED, Fixture, Operation};
use resolve_merge_conflicts::git::{ConflictKind, Stage};

#[test]
fn reads_the_three_sides_of_a_conflict() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let mut git = fixture.git();
    assert_eq!(
        git.stage_blob(Stage::Base, CONFLICTED).unwrap().unwrap(),
        "one\nbase\nthree\n"
    );
    assert_eq!(
        git.stage_blob(Stage::Ours, CONFLICTED).unwrap().unwrap(),
        "one\nmain\nthree\n"
    );
    assert_eq!(
        git.stage_blob(Stage::Theirs, CONFLICTED).unwrap().unwrap(),
        "one\nfeature\nthree\n"
    );
}

#[test]
fn lists_unmerged_entries_for_every_operation() {
    for operation in [
        Operation::Merge,
        Operation::Rebase,
        Operation::CherryPick,
        Operation::Revert,
        Operation::StashPop,
    ] {
        let fixture = Fixture::conflicted(operation);
        let entries = fixture.git().unmerged_entries().unwrap();
        assert_eq!(entries.len(), 1, "{operation:?}");
        assert_eq!(entries[0].path, CONFLICTED, "{operation:?}");
        assert_eq!(entries[0].kind, ConflictKind::BothModified, "{operation:?}");
    }
}

#[test]
fn a_delete_modify_conflict_reports_who_deleted() {
    let fixture = Fixture::delete_modify_conflict();
    let mut git = fixture.git();
    let entries = git.unmerged_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path, "b.txt");
    assert_eq!(entries[0].kind, ConflictKind::DeletedByThem);
    assert_eq!(git.stage_blob(Stage::Theirs, "b.txt").unwrap(), None);
}

#[test]
fn add_stages_a_resolution_and_a_deletion_without_committing() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let head = fixture.head();
    let mut git = fixture.git();
    fixture.write(CONFLICTED, "one\nresolved\nthree\n");
    git.add(&[CONFLICTED]).unwrap();
    assert!(git.unmerged_entries().unwrap().is_empty());
    assert_eq!(fixture.staged(), [CONFLICTED]);
    assert_eq!(fixture.head(), head);

    let fixture = Fixture::delete_modify_conflict();
    std::fs::remove_file(fixture.work.join("b.txt")).unwrap();
    let mut git = fixture.git();
    git.add(&["b.txt"]).unwrap();
    assert!(git.unmerged_entries().unwrap().is_empty());
    assert_eq!(fixture.staged(), ["b.txt"]);
}

#[test]
fn other_changes_excludes_the_conflicted_files() {
    let fixture = Fixture::conflicted(Operation::Merge);
    fixture.write("b.txt", "unrelated edit\n");
    let mut git = fixture.git();
    assert_eq!(git.other_changes().unwrap(), ["b.txt"]);
}

#[test]
fn numstat_counts_lines_and_head_and_branch_are_readable() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let mut git = fixture.git();
    fixture.write(CONFLICTED, "one\nresolved\nthree\n");
    git.add(&[CONFLICTED]).unwrap();
    let stats = git.numstat(true, &[CONFLICTED]).unwrap();
    assert_eq!(stats, [(1, 1, CONFLICTED.to_string())]);
    assert_eq!(git.head().as_deref(), Some(fixture.head().as_str()));
    assert_eq!(git.current_branch().as_deref(), Some("main"));
}

#[test]
fn detached_head_and_non_repos_are_handled() {
    let fixture = Fixture::conflicted(Operation::Rebase);
    assert_eq!(fixture.git().current_branch(), None);

    let empty = tempfile::tempdir().unwrap();
    let mut git = resolve_merge_conflicts::git::Git::new(empty.path()).env(
        "GIT_CEILING_DIRECTORIES",
        empty.path().parent().unwrap().to_string_lossy(),
    );
    assert!(!git.is_work_tree());
    assert!(git.git_dir().is_err());
}

#[test]
fn git_dir_points_at_the_directory_holding_operation_state() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let git_dir = fixture.git().git_dir().unwrap();
    assert!(git_dir.join("MERGE_HEAD").exists());
}
