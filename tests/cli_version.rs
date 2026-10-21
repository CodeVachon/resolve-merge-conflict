use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn version_flag_prints_the_cargo_version() {
    Command::cargo_bin("resolve-merge-conflicts")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains(resolve_merge_conflicts::VERSION));
}
