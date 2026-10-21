//! The fixtures themselves: each scenario must leave git genuinely conflicted, or every test built
//! on it would pass for the wrong reason.

mod common;

use common::{CONFLICTED, Fixture, Operation};

#[test]
fn every_operation_leaves_a_conflicted_file_with_markers() {
    for operation in [
        Operation::Merge,
        Operation::Rebase,
        Operation::CherryPick,
        Operation::Revert,
        Operation::StashPop,
    ] {
        let fixture = Fixture::conflicted(operation);
        assert_eq!(fixture.unmerged(), [CONFLICTED], "{operation:?}");
        assert!(
            fixture.read(CONFLICTED).contains("<<<<<<<"),
            "{operation:?} left no conflict markers"
        );
    }
}

#[test]
fn delete_modify_conflict_is_unmerged_without_markers() {
    let fixture = Fixture::delete_modify_conflict();
    assert_eq!(fixture.unmerged(), ["b.txt"]);
    assert!(!fixture.read("b.txt").contains("<<<<<<<"));
}

#[test]
fn staging_the_resolution_clears_the_conflict_without_committing() {
    let fixture = Fixture::conflicted(Operation::Merge);
    let head = fixture.head();
    fixture.write(CONFLICTED, "one\nresolved\nthree\n");
    fixture.raw(&["add", CONFLICTED]);
    assert!(fixture.unmerged().is_empty());
    assert_eq!(fixture.staged(), [CONFLICTED]);
    assert_eq!(fixture.head(), head);
}
