//! Codex model discovery against a fake `codex` executable.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use resolve_merge_conflicts::models::discover_codex;

fn fake(dir: &Path, body: &str) -> std::path::PathBuf {
    let path = dir.join("codex");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

#[test]
fn discovers_models_from_the_catalog() {
    let dir = tempfile::tempdir().unwrap();
    let program = fake(
        dir.path(),
        r#"[ "$1 $2" = "debug models" ] && echo '{"models":[{"slug":"m2","visibility":"list","priority":1},{"slug":"m1","visibility":"list","priority":0}]}'"#,
    );
    assert_eq!(discover_codex(&program).unwrap(), ["m1", "m2"]);
}

#[test]
fn a_failing_codex_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let program = fake(dir.path(), "exit 4");
    assert!(discover_codex(&program).is_err());
}

#[test]
fn a_missing_codex_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    assert!(discover_codex(&dir.path().join("nope")).is_err());
}

#[test]
fn garbage_output_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let program = fake(dir.path(), "echo not json");
    assert!(discover_codex(&program).is_err());
}

#[test]
fn a_large_catalog_does_not_deadlock_the_pipe() {
    let dir = tempfile::tempdir().unwrap();
    let program = fake(
        dir.path(),
        r#"printf '{"models":[{"slug":"m","visibility":"list","priority":0,"blob":"'; head -c 500000 /dev/zero | tr '\0' x; printf '"}]}'"#,
    );
    assert_eq!(discover_codex(&program).unwrap(), ["m"]);
}

/// Against the real `codex`, if installed: `cargo test -- --ignored`.
#[test]
#[ignore = "needs a real codex on PATH"]
fn the_real_codex_catalog_parses() {
    let models = discover_codex(Path::new("codex")).unwrap();
    assert!(!models.is_empty());
    eprintln!("{models:?}");
}
