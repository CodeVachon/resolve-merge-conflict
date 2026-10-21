//! Conflict detection against real repositories left mid-operation.

mod common;

use common::{CONFLICTED, Fixture, Operation as Fixed};
use resolve_merge_conflicts::conflicts::{self, Operation};

#[test]
fn names_the_operation_for_each_kind_of_conflict() {
    for (fixed, expected) in [
        (Fixed::Merge, Operation::Merge),
        (Fixed::Rebase, Operation::Rebase),
        (Fixed::CherryPick, Operation::CherryPick),
        (Fixed::Revert, Operation::Revert),
        (Fixed::StashPop, Operation::Other),
    ] {
        let fixture = Fixture::conflicted(fixed);
        let state = conflicts::detect(&mut fixture.git()).unwrap().unwrap();
        assert_eq!(state.operation, expected, "{fixed:?}");
        assert_eq!(state.files.len(), 1, "{fixed:?}");
        assert_eq!(state.files[0].path, CONFLICTED, "{fixed:?}");
    }
}

#[test]
fn a_clean_repository_has_nothing_to_resolve() {
    let fixture = Fixture::new();
    assert_eq!(conflicts::detect(&mut fixture.git()).unwrap(), None);
}

#[test]
fn a_merge_whose_conflicts_are_all_staged_has_nothing_left() {
    let fixture = Fixture::conflicted(Fixed::Merge);
    fixture.write(CONFLICTED, "one\nresolved\nthree\n");
    fixture.raw(&["add", CONFLICTED]);
    assert_eq!(conflicts::detect(&mut fixture.git()).unwrap(), None);
}

#[test]
fn outside_a_repository_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let mut git = resolve_merge_conflicts::git::Git::new(dir.path()).env(
        "GIT_CEILING_DIRECTORIES",
        dir.path().parent().unwrap().to_string_lossy(),
    );
    let error = conflicts::detect(&mut git).unwrap_err();
    assert!(error.to_string().contains("not inside a git repository"));
}

#[test]
fn the_sides_are_described_the_right_way_round_for_a_rebase() {
    let merge = Fixture::conflicted(Fixed::Merge);
    let state = conflicts::detect(&mut merge.git()).unwrap().unwrap();
    assert!(state.ours.contains("main"), "{}", state.ours);
    assert!(state.theirs.contains("feature"), "{}", state.theirs);

    let rebase = Fixture::conflicted(Fixed::Rebase);
    let state = conflicts::detect(&mut rebase.git()).unwrap().unwrap();
    assert!(state.ours.contains("rebased onto"));
    assert!(state.theirs.contains("replayed"));
}

#[test]
fn delete_modify_is_detected_as_a_merge() {
    let fixture = Fixture::delete_modify_conflict();
    let state = conflicts::detect(&mut fixture.git()).unwrap().unwrap();
    assert_eq!(state.operation, Operation::Merge);
    assert_eq!(state.files[0].path, "b.txt");
}
